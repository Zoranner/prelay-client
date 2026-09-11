use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use prelay_protocol::{ExtensionFile, ExtensionMcpManifest, ExtensionMcpTransport};
use serde::{Deserialize, Serialize};

use crate::{
    agents::{
        claude_code, codex_mcp_server_exists, opencode, upsert_codex_mcp_server, AgentClient,
    },
    relay::client::ClientError,
};

use super::{atomic_write, decode_extension_file};

const PRELAY_STATE_DIRECTORY: &str = ".prelay";
const MCP_PACKAGE_STATE_FILE: &str = "mcp.json";

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
struct InstalledMcpPackages(BTreeMap<String, InstalledMcpPackage>);

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct InstalledMcpPackage {
    server_name: String,
    version: String,
    commit_sha: String,
}

pub(crate) struct McpInstallationStatus {
    pub action: McpInstallAction,
    pub clients: Vec<AgentClient>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum McpInstallAction {
    Install,
    Partial,
    Update,
    Installed,
}

pub(super) fn read_mcp_manifest(file: &ExtensionFile) -> Result<ExtensionMcpManifest, ClientError> {
    if file.path != "server.json" {
        return Err(ClientError::new(
            "invalid_response",
            "MCP 安装包缺少 server.json。",
        ));
    }
    let content = decode_extension_file(file)?;
    let manifest = serde_json::from_slice::<ExtensionMcpManifest>(&content)
        .map_err(|_| ClientError::new("invalid_response", "MCP 清单不是有效的 JSON。"))?;
    validate_mcp_manifest(&manifest)?;
    Ok(manifest)
}

pub(super) fn install_mcp(
    home: &Path,
    clients: &[AgentClient],
    package: &str,
    version: &str,
    commit_sha: &str,
    manifest: &ExtensionMcpManifest,
    overwrite: bool,
) -> Result<(), ClientError> {
    for client in clients {
        let exists = mcp_server_exists(home, *client, &manifest.name)?;
        let mut installed = read_installed_mcp_packages(home, *client)?;
        let owned = installed
            .0
            .get(package)
            .is_some_and(|entry| entry.server_name == manifest.name);
        if exists && !owned && !overwrite {
            return Err(ClientError::new(
                "extension_target_exists",
                "MCP 服务名已存在，确认后可覆盖安装。",
            ));
        }
        installed.0.retain(|installed_package, entry| {
            installed_package == package || entry.server_name != manifest.name
        });
        upsert_mcp_server(home, *client, manifest)?;
        installed.0.insert(
            package.to_string(),
            InstalledMcpPackage {
                server_name: manifest.name.clone(),
                version: version.to_string(),
                commit_sha: commit_sha.to_string(),
            },
        );
        write_installed_mcp_packages(home, *client, &installed)?;
    }
    Ok(())
}

fn validate_mcp_manifest(manifest: &ExtensionMcpManifest) -> Result<(), ClientError> {
    if manifest.name.trim().is_empty() || manifest.name.chars().any(char::is_control) {
        return Err(ClientError::new("invalid_response", "MCP 服务名无效。"));
    }
    match &manifest.transport {
        ExtensionMcpTransport::Stdio {
            command,
            cwd,
            environment,
            ..
        } => {
            if command
                .first()
                .is_none_or(|program| program.trim().is_empty())
                || cwd.is_some()
                || environment.iter().any(|(name, value)| {
                    !is_environment_variable_name(name)
                        || !is_environment_variable_name(value)
                        || name != value
                })
            {
                return Err(ClientError::new(
                    "invalid_response",
                    "MCP 本地进程配置无效。",
                ));
            }
        }
        ExtensionMcpTransport::Http { url, headers, .. } => {
            let url = reqwest::Url::parse(url)
                .map_err(|_| ClientError::new("invalid_response", "MCP HTTP 地址无效。"))?;
            if !matches!(url.scheme(), "http" | "https")
                || url.host().is_none()
                || !url.username().is_empty()
                || url.password().is_some()
                || headers.iter().any(|(name, variable)| {
                    reqwest::header::HeaderName::from_bytes(name.as_bytes()).is_err()
                        || !is_environment_variable_name(variable)
                })
            {
                return Err(ClientError::new("invalid_response", "MCP HTTP 配置无效。"));
            }
        }
    }
    Ok(())
}

fn is_environment_variable_name(value: &str) -> bool {
    let mut characters = value.chars();
    matches!(characters.next(), Some('A'..='Z' | 'a'..='z' | '_'))
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

pub(crate) fn mcp_installation_status(
    home: &Path,
    clients: &[AgentClient],
    package: &str,
    version: &str,
    commit_sha: &str,
) -> Result<McpInstallationStatus, ClientError> {
    let mut installed_clients = Vec::new();
    let mut missing = false;
    let mut outdated = false;

    for client in clients {
        let mut installed = read_installed_mcp_packages(home, *client)?;
        let Some(current) = installed.0.get(package) else {
            missing = true;
            continue;
        };
        if !mcp_server_exists(home, *client, &current.server_name)? {
            installed.0.remove(package);
            write_installed_mcp_packages(home, *client, &installed)?;
            missing = true;
            continue;
        }
        installed_clients.push(*client);
        if current.version != version || current.commit_sha != commit_sha {
            outdated = true;
        }
    }

    let action = if outdated {
        McpInstallAction::Update
    } else if !installed_clients.is_empty() && missing {
        McpInstallAction::Partial
    } else if installed_clients.is_empty() {
        McpInstallAction::Install
    } else {
        McpInstallAction::Installed
    };
    Ok(McpInstallationStatus {
        action,
        clients: installed_clients,
    })
}

pub(crate) fn retain_listed_mcp_packages(
    home: &Path,
    clients: &[AgentClient],
    listed_packages: &BTreeSet<String>,
) -> Result<(), ClientError> {
    for client in clients {
        let mut installed = read_installed_mcp_packages(home, *client)?;
        let initial_len = installed.0.len();
        installed
            .0
            .retain(|package, _| listed_packages.contains(package));
        if installed.0.len() != initial_len {
            write_installed_mcp_packages(home, *client, &installed)?;
        }
    }
    Ok(())
}

fn mcp_server_exists(home: &Path, client: AgentClient, name: &str) -> Result<bool, ClientError> {
    match client {
        AgentClient::CodexCli => codex_mcp_server_exists(home, name),
        AgentClient::ClaudeCode => claude_code::mcp_server_exists(home, name),
        AgentClient::OpenCode => opencode::mcp_server_exists(home, name),
        AgentClient::ChatGpt => {
            return Err(ClientError::new(
                "extension_client_unavailable",
                "ChatGPT 当前不支持 MCP 安装。",
            ));
        }
    }
    .map_err(|error| ClientError::new("local_extensions_error", error))
}

fn upsert_mcp_server(
    home: &Path,
    client: AgentClient,
    manifest: &ExtensionMcpManifest,
) -> Result<(), ClientError> {
    match client {
        AgentClient::CodexCli => upsert_codex_mcp_server(home, manifest),
        AgentClient::ClaudeCode => claude_code::upsert_mcp_server(home, manifest),
        AgentClient::OpenCode => opencode::upsert_mcp_server(home, manifest),
        AgentClient::ChatGpt => {
            return Err(ClientError::new(
                "extension_client_unavailable",
                "ChatGPT 当前不支持 MCP 安装。",
            ));
        }
    }
    .map_err(|error| ClientError::new("local_extensions_error", error))
}

fn read_installed_mcp_packages(
    home: &Path,
    client: AgentClient,
) -> Result<InstalledMcpPackages, ClientError> {
    let path = mcp_package_state_path(home, client)?;
    match fs::read(path) {
        Ok(contents) => serde_json::from_slice(&contents).map_err(|error| {
            ClientError::new(
                "local_extensions_error",
                format!("无法读取已安装 MCP 状态：{error}"),
            )
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(InstalledMcpPackages::default())
        }
        Err(error) => Err(ClientError::new(
            "local_extensions_error",
            format!("无法读取已安装 MCP 状态：{error}"),
        )),
    }
}

fn write_installed_mcp_packages(
    home: &Path,
    client: AgentClient,
    packages: &InstalledMcpPackages,
) -> Result<(), ClientError> {
    let path = mcp_package_state_path(home, client)?;
    let contents = serde_json::to_vec(packages).map_err(|error| {
        ClientError::new(
            "local_extensions_error",
            format!("无法保存已安装 MCP 状态：{error}"),
        )
    })?;
    atomic_write(&path, &contents)
}

fn mcp_package_state_path(home: &Path, client: AgentClient) -> Result<PathBuf, ClientError> {
    let root = match client {
        AgentClient::CodexCli => home.join(".codex"),
        AgentClient::ChatGpt => {
            return Err(ClientError::new(
                "extension_client_unavailable",
                "ChatGPT 当前不支持 MCP 安装。",
            ));
        }
        AgentClient::ClaudeCode => home.join(".claude"),
        AgentClient::OpenCode => home.join(".config").join("opencode"),
    };
    Ok(root
        .join(PRELAY_STATE_DIRECTORY)
        .join(MCP_PACKAGE_STATE_FILE))
}

#[cfg(test)]
#[path = "mcp_tests.rs"]
mod tests;

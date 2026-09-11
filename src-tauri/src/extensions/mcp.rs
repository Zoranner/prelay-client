use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use prelay_protocol::{ExtensionFile, ExtensionMcpManifest, ExtensionMcpTransport};
use serde::{Deserialize, Serialize};

use crate::{
    agents::{
        claude_code, codex_mcp_server_exists, codex_mcp_server_matches, opencode,
        remove_codex_mcp_server, upsert_codex_mcp_server, AgentClient,
    },
    relay::client::ClientError,
};

use super::{atomic_write, decode_extension_file};

const PRELAY_STATE_DIRECTORY: &str = ".prelay";
const MCP_PACKAGE_STATE_FILE: &str = "mcp.json";

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
struct InstalledMcpPackages(BTreeMap<String, InstalledMcpPackage>);

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct InstalledMcpPackage {
    server_name: String,
    version: String,
    commit_sha: String,
    #[serde(default)]
    manifest: Option<ExtensionMcpManifest>,
}

struct PreparedMcpInstallation {
    client: AgentClient,
    installed: InstalledMcpPackages,
    previous_server: Option<String>,
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
    let mut targets = Vec::with_capacity(clients.len());
    for client in clients {
        let exists = mcp_server_exists(home, *client, &manifest.name)?;
        let installed = read_installed_mcp_packages(home, *client)?;
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
        let previous_server = if let Some(previous) = installed
            .0
            .get(package)
            .filter(|entry| entry.server_name != manifest.name)
        {
            match previous.manifest.as_ref() {
                Some(current) if mcp_server_matches(home, *client, current)? => {
                    Some(previous.server_name.clone())
                }
                _ => None,
            }
        } else {
            None
        };
        targets.push(PreparedMcpInstallation {
            client: *client,
            installed,
            previous_server,
        });
    }

    for mut target in targets {
        if let Some(previous_server) = target.previous_server {
            remove_mcp_server(home, target.client, &previous_server)?;
        }
        target.installed.0.retain(|installed_package, entry| {
            installed_package == package || entry.server_name != manifest.name
        });
        upsert_mcp_server(home, target.client, manifest)?;
        target.installed.0.insert(
            package.to_string(),
            InstalledMcpPackage {
                server_name: manifest.name.clone(),
                version: version.to_string(),
                commit_sha: commit_sha.to_string(),
                manifest: Some(manifest.clone()),
            },
        );
        write_installed_mcp_packages(home, target.client, &target.installed)?;
    }
    Ok(())
}

fn validate_mcp_manifest(manifest: &ExtensionMcpManifest) -> Result<(), ClientError> {
    if !is_mcp_server_name(&manifest.name) {
        return Err(ClientError::new("invalid_response", "MCP 服务名无效。"));
    }
    match &manifest.transport {
        ExtensionMcpTransport::Stdio {
            command,
            cwd,
            environment,
            enabled,
            ..
        } => {
            if command
                .first()
                .is_none_or(|program| program.trim().is_empty())
                || cwd.is_some()
                || !enabled
                || command_has_plaintext_secret(command)
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
        ExtensionMcpTransport::Http {
            url,
            headers,
            enabled,
            ..
        } => {
            if !enabled
                || !is_safe_http_url(url)
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

fn is_mcp_server_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
}

fn is_safe_http_url(value: &str) -> bool {
    let Ok(url) = reqwest::Url::parse(value) else {
        return false;
    };
    matches!(url.scheme(), "http" | "https")
        && url.host().is_some()
        && url.username().is_empty()
        && url.password().is_none()
        && url.fragment().is_none()
        && !url
            .query_pairs()
            .any(|(name, _)| is_sensitive_name(name.as_ref()))
}

fn command_has_plaintext_secret(command: &[String]) -> bool {
    command
        .iter()
        .skip(1)
        .any(|argument| is_sensitive_command_option(argument))
}

fn is_sensitive_command_option(argument: &str) -> bool {
    let option = argument
        .trim_start_matches('-')
        .split_once('=')
        .map_or(argument.trim_start_matches('-'), |(name, _)| name);
    is_sensitive_name(option)
}

fn is_sensitive_name(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase().replace('_', "-");
    normalized.contains("api-key")
        || normalized.contains("apikey")
        || normalized.contains("token")
        || normalized.contains("secret")
        || normalized.contains("password")
        || normalized.contains("authorization")
        || matches!(normalized.as_str(), "auth" | "key")
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
        if let Some(manifest) = current.manifest.as_ref() {
            if !mcp_server_matches(home, *client, manifest)? {
                outdated = true;
            }
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

fn mcp_server_matches(
    home: &Path,
    client: AgentClient,
    manifest: &ExtensionMcpManifest,
) -> Result<bool, ClientError> {
    match client {
        AgentClient::CodexCli => codex_mcp_server_matches(home, manifest),
        AgentClient::ClaudeCode => claude_code::mcp_server_matches(home, manifest),
        AgentClient::OpenCode => opencode::mcp_server_matches(home, manifest),
        AgentClient::ChatGpt => {
            return Err(ClientError::new(
                "extension_client_unavailable",
                "ChatGPT 当前不支持 MCP 安装。",
            ));
        }
    }
    .map_err(|error| ClientError::new("local_extensions_error", error))
}

fn remove_mcp_server(home: &Path, client: AgentClient, name: &str) -> Result<(), ClientError> {
    if !mcp_server_exists(home, client, name)? {
        return Ok(());
    }
    match client {
        AgentClient::CodexCli => remove_codex_mcp_server(home, name),
        AgentClient::ClaudeCode => claude_code::remove_mcp_server(home, name),
        AgentClient::OpenCode => opencode::remove_mcp_server(home, name),
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
#[path = "mcp_state_tests.rs"]
mod state_tests;
#[cfg(test)]
#[path = "mcp_tests.rs"]
mod tests;

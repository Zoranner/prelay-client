use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use prelay_protocol::{ExtensionFile, ExtensionMcpManifest, ExtensionMcpTransport};
use serde::{Deserialize, Serialize};

use crate::{
    agents::{
        codex_mcp_server_exists, codex_mcp_server_matches, opencode, remove_codex_mcp_server,
        upsert_codex_mcp_server, AgentClient,
    },
    relay::client::ClientError,
};

use super::{atomic_write, decode_extension_file};

const PRELAY_STATE_DIRECTORY: &str = ".prelay";
const MCP_PACKAGE_STATE_FILE: &str = "mcp.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum McpHost {
    Codex,
    OpenCode,
}

fn mcp_host(client: AgentClient) -> McpHost {
    match client {
        AgentClient::CodexCli | AgentClient::ChatGpt => McpHost::Codex,
        AgentClient::OpenCode => McpHost::OpenCode,
    }
}

fn mcp_hosts(clients: &[AgentClient]) -> Vec<McpHost> {
    clients
        .iter()
        .map(|client| mcp_host(*client))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

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
    host: McpHost,
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
    let expanded = expand_manifest(manifest, home);
    let mut targets = Vec::new();
    for host in mcp_hosts(clients) {
        let exists = mcp_server_exists(home, host, &expanded.name)?;
        let installed = read_installed_mcp_packages(home, host)?;
        let owned = installed
            .0
            .get(package)
            .is_some_and(|entry| entry.server_name == expanded.name);
        if exists && !owned && !overwrite {
            return Err(ClientError::new(
                "extension_target_exists",
                "MCP 服务名已存在，确认后可覆盖安装。",
            ));
        }
        let previous_server = if let Some(previous) = installed
            .0
            .get(package)
            .filter(|entry| entry.server_name != expanded.name)
        {
            match previous.manifest.as_ref() {
                Some(current)
                    if mcp_server_matches(home, host, &expand_manifest(current, home))? =>
                {
                    Some(previous.server_name.clone())
                }
                _ => None,
            }
        } else {
            None
        };
        targets.push(PreparedMcpInstallation {
            host,
            installed,
            previous_server,
        });
    }

    for mut target in targets {
        if let Some(previous_server) = target.previous_server {
            remove_mcp_server(home, target.host, &previous_server)?;
        }
        target.installed.0.retain(|installed_package, entry| {
            installed_package == package || entry.server_name != expanded.name
        });
        upsert_mcp_server(home, target.host, &expanded)?;
        target.installed.0.insert(
            package.to_string(),
            InstalledMcpPackage {
                server_name: expanded.name.clone(),
                version: version.to_string(),
                commit_sha: commit_sha.to_string(),
                manifest: Some(manifest.clone()),
            },
        );
        write_installed_mcp_packages(home, target.host, &target.installed)?;
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
        && !matches!(
            value.to_ascii_lowercase().as_str(),
            "workspace" | "cli" | "project" | "user" | "local" | "claude" | "builtin"
        )
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
        && url.query().is_none()
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
    matches!(
        normalized.as_str(),
        "api-key"
            | "apikey"
            | "access-key"
            | "accesskey"
            | "token"
            | "secret"
            | "password"
            | "authorization"
            | "auth"
            | "credential"
            | "credentials"
            | "key"
    )
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
        let host = mcp_host(*client);
        let mut installed = read_installed_mcp_packages(home, host)?;
        let Some(current) = installed.0.get(package) else {
            missing = true;
            continue;
        };
        if !mcp_server_exists(home, host, &current.server_name)? {
            installed.0.remove(package);
            write_installed_mcp_packages(home, host, &installed)?;
            missing = true;
            continue;
        }
        installed_clients.push(*client);
        if current.version != version || current.commit_sha != commit_sha {
            outdated = true;
        }
        match current.manifest.as_ref() {
            Some(manifest)
                if !mcp_server_matches(home, host, &expand_manifest(manifest, home))? =>
            {
                outdated = true;
            }
            None => outdated = true,
            Some(_) => {}
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
    for host in mcp_hosts(clients) {
        let mut installed = read_installed_mcp_packages(home, host)?;
        let initial_len = installed.0.len();
        installed
            .0
            .retain(|package, _| listed_packages.contains(package));
        if installed.0.len() != initial_len {
            write_installed_mcp_packages(home, host, &installed)?;
        }
    }
    Ok(())
}

fn mcp_server_exists(home: &Path, host: McpHost, name: &str) -> Result<bool, ClientError> {
    match host {
        McpHost::Codex => codex_mcp_server_exists(home, name),
        McpHost::OpenCode => opencode::mcp_server_exists(home, name),
    }
    .map_err(|error| ClientError::new("local_extensions_error", error))
}

fn upsert_mcp_server(
    home: &Path,
    host: McpHost,
    manifest: &ExtensionMcpManifest,
) -> Result<(), ClientError> {
    match host {
        McpHost::Codex => upsert_codex_mcp_server(home, manifest),
        McpHost::OpenCode => opencode::upsert_mcp_server(home, manifest),
    }
    .map_err(|error| ClientError::new("local_extensions_error", error))
}

fn mcp_server_matches(
    home: &Path,
    host: McpHost,
    manifest: &ExtensionMcpManifest,
) -> Result<bool, ClientError> {
    match host {
        McpHost::Codex => codex_mcp_server_matches(home, manifest),
        McpHost::OpenCode => opencode::mcp_server_matches(home, manifest),
    }
    .map_err(|error| ClientError::new("local_extensions_error", error))
}

fn remove_mcp_server(home: &Path, host: McpHost, name: &str) -> Result<(), ClientError> {
    if !mcp_server_exists(home, host, name)? {
        return Ok(());
    }
    match host {
        McpHost::Codex => remove_codex_mcp_server(home, name),
        McpHost::OpenCode => opencode::remove_mcp_server(home, name),
    }
    .map_err(|error| ClientError::new("local_extensions_error", error))
}

fn read_installed_mcp_packages(
    home: &Path,
    host: McpHost,
) -> Result<InstalledMcpPackages, ClientError> {
    let path = mcp_package_state_path(home, host);
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
    host: McpHost,
    packages: &InstalledMcpPackages,
) -> Result<(), ClientError> {
    let path = mcp_package_state_path(home, host);
    let contents = serde_json::to_vec(packages).map_err(|error| {
        ClientError::new(
            "local_extensions_error",
            format!("无法保存已安装 MCP 状态：{error}"),
        )
    })?;
    atomic_write(&path, &contents)
}

fn mcp_package_state_path(home: &Path, host: McpHost) -> PathBuf {
    let root = match host {
        McpHost::Codex => home.join(".codex"),
        McpHost::OpenCode => home.join(".config").join("opencode"),
    };
    root.join(PRELAY_STATE_DIRECTORY)
        .join(MCP_PACKAGE_STATE_FILE)
}

fn expand_manifest(manifest: &ExtensionMcpManifest, home: &Path) -> ExtensionMcpManifest {
    let mut expanded = manifest.clone();
    if let ExtensionMcpTransport::Stdio { command, .. } = &mut expanded.transport {
        for argument in command.iter_mut().skip(1) {
            *argument = expand_user_directories(argument, home);
        }
    }
    expanded
}

fn expand_user_directories(value: &str, home: &Path) -> String {
    let variables = [
        ("%USERPROFILE%".to_string(), home.display().to_string()),
        (
            "%APPDATA%".to_string(),
            home.join("AppData").join("Roaming").display().to_string(),
        ),
        (
            "%LOCALAPPDATA%".to_string(),
            home.join("AppData").join("Local").display().to_string(),
        ),
    ];
    let mut expanded = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(index) = rest.find('%') {
        expanded.push_str(&rest[..index]);
        let tail = &rest[index..];
        if let Some(stripped) = tail.strip_prefix("%%") {
            expanded.push('%');
            rest = stripped;
            continue;
        }
        match variables
            .iter()
            .find(|(name, _)| tail.starts_with(name.as_str()))
        {
            Some((name, replacement)) => {
                expanded.push_str(replacement);
                rest = &tail[name.len()..];
            }
            None => {
                expanded.push('%');
                rest = &tail[1..];
            }
        }
    }
    expanded.push_str(rest);
    expanded
}

#[cfg(test)]
#[path = "mcp_state_tests.rs"]
mod state_tests;
#[cfg(test)]
#[path = "mcp_tests.rs"]
mod tests;

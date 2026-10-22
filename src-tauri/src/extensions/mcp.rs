use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use prelay_protocol::ExtensionMcpManifest;
use serde::{Deserialize, Serialize};

use crate::{
    agents::{
        codex_mcp_server_exists, codex_mcp_server_matches, opencode, remove_codex_mcp_server,
        upsert_codex_mcp_server, AgentClient,
    },
    relay::client::ClientError,
};

use super::{
    atomic_write,
    mcp_manifest::{app_directory, expand_manifest},
    ExtensionInstallAction,
};

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
    expanded: ExtensionMcpManifest,
}

pub(crate) struct McpInstallationStatus {
    pub action: ExtensionInstallAction,
    pub clients: Vec<AgentClient>,
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
    let targets = prepare_mcp_installation(home, clients, package, manifest, overwrite)?;
    for mut target in targets {
        if let Some(previous_server) = target.previous_server {
            remove_mcp_server(home, target.host, &previous_server)?;
        }
        target.installed.0.retain(|installed_package, entry| {
            installed_package == package || entry.server_name != target.expanded.name
        });
        upsert_mcp_server(home, target.host, &target.expanded)?;
        target.installed.0.insert(
            package.to_string(),
            InstalledMcpPackage {
                server_name: target.expanded.name.clone(),
                version: version.to_string(),
                commit_sha: commit_sha.to_string(),
                manifest: Some(manifest.clone()),
            },
        );
        write_installed_mcp_packages(home, target.host, &target.installed)?;
    }
    Ok(())
}

pub(crate) fn preflight_mcp_install(
    home: &Path,
    clients: &[AgentClient],
    package: &str,
    manifest: &ExtensionMcpManifest,
    overwrite: bool,
) -> Result<(), ClientError> {
    prepare_mcp_installation(home, clients, package, manifest, overwrite).map(|_| ())
}

fn prepare_mcp_installation(
    home: &Path,
    clients: &[AgentClient],
    package: &str,
    manifest: &ExtensionMcpManifest,
    overwrite: bool,
) -> Result<Vec<PreparedMcpInstallation>, ClientError> {
    let app_directory = app_directory()?;
    let expanded = expand_manifest(manifest, home, &app_directory)?;
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
        let previous_server = match installed
            .0
            .get(package)
            .filter(|entry| entry.server_name != expanded.name)
        {
            Some(previous) => {
                let still_installed = match previous.manifest.as_ref() {
                    Some(current) => mcp_server_matches(
                        home,
                        host,
                        &expand_manifest(current, home, &app_directory)?,
                    )?,
                    None => false,
                };
                still_installed.then(|| previous.server_name.clone())
            }
            None => None,
        };
        targets.push(PreparedMcpInstallation {
            host,
            installed,
            previous_server,
            expanded: expanded.clone(),
        });
    }
    Ok(targets)
}

pub(crate) fn mcp_installation_status(
    home: &Path,
    clients: &[AgentClient],
    package: &str,
    version: &str,
    commit_sha: &str,
) -> Result<McpInstallationStatus, ClientError> {
    let mut installed_hosts = BTreeSet::new();
    let mut missing = false;
    let mut outdated = false;
    let app_directory = app_directory()?;

    for host in mcp_hosts(clients) {
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
        installed_hosts.insert(host);
        if current.version != version || current.commit_sha != commit_sha {
            outdated = true;
        }
        match current.manifest.as_ref() {
            Some(manifest) => {
                if !mcp_server_matches(
                    home,
                    host,
                    &expand_manifest(manifest, home, &app_directory)?,
                )? {
                    outdated = true;
                }
            }
            None => outdated = true,
        }
    }

    let clients = clients
        .iter()
        .copied()
        .filter(|client| installed_hosts.contains(&mcp_host(*client)))
        .collect::<Vec<_>>();
    let action = if outdated {
        ExtensionInstallAction::Update
    } else if !clients.is_empty() && missing {
        ExtensionInstallAction::Partial
    } else if clients.is_empty() {
        ExtensionInstallAction::Install
    } else {
        ExtensionInstallAction::Installed
    };
    Ok(McpInstallationStatus { action, clients })
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

pub(crate) fn uninstall_mcp_package(
    home: &Path,
    client: AgentClient,
    package: &str,
) -> Result<(), ClientError> {
    let host = mcp_host(client);
    let mut installed = read_installed_mcp_packages(home, host)?;
    let Some(entry) = installed.0.remove(package) else {
        return Ok(());
    };
    remove_mcp_server(home, host, &entry.server_name)?;
    write_installed_mcp_packages(home, host, &installed)
}

#[derive(Clone, Debug)]
pub(crate) struct ManagedMcpServer {
    pub package: String,
    pub version: String,
}

/// 扩展库装进来的 MCP 服务，以服务名为键。
///
/// 条目扫描只读得到宿主配置里的服务名，分不清来源、也拿不到版本，
/// 所以按宿主读一次 `.prelay/mcp.json` 认领；状态读不到时按个人条目处理。
pub(crate) fn managed_mcp_servers(
    home: &Path,
    client: AgentClient,
) -> BTreeMap<String, ManagedMcpServer> {
    read_installed_mcp_packages(home, mcp_host(client))
        .map(|installed| {
            installed
                .0
                .into_iter()
                .map(|(package, entry)| {
                    (
                        entry.server_name,
                        ManagedMcpServer {
                            package,
                            version: entry.version,
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default()
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

#[cfg(test)]
#[path = "mcp_state_tests.rs"]
mod state_tests;
#[cfg(test)]
#[path = "mcp_tests.rs"]
mod tests;

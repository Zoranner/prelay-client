use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use prelay_protocol::ExtensionMcpManifest;
use serde::{Deserialize, Serialize};

use crate::{
    agents::{
        codex_mcp_server_exists, codex_mcp_server_matches, remove_codex_mcp_server,
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
    if clients.is_empty() {
        return Ok(());
    }
    let mut target = prepare_mcp_installation(home, package, manifest, overwrite)?;
    if let Some(previous_server) = target.previous_server {
        remove_mcp_server(home, &previous_server)?;
    }
    target.installed.0.retain(|installed_package, entry| {
        installed_package == package || entry.server_name != target.expanded.name
    });
    upsert_mcp_server(home, &target.expanded)?;
    target.installed.0.insert(
        package.to_string(),
        InstalledMcpPackage {
            server_name: target.expanded.name.clone(),
            version: version.to_string(),
            commit_sha: commit_sha.to_string(),
            manifest: Some(manifest.clone()),
        },
    );
    write_installed_mcp_packages(home, &target.installed)?;
    Ok(())
}

pub(crate) fn preflight_mcp_install(
    home: &Path,
    clients: &[AgentClient],
    package: &str,
    manifest: &ExtensionMcpManifest,
    overwrite: bool,
) -> Result<(), ClientError> {
    if clients.is_empty() {
        return Ok(());
    }
    prepare_mcp_installation(home, package, manifest, overwrite).map(|_| ())
}

fn prepare_mcp_installation(
    home: &Path,
    package: &str,
    manifest: &ExtensionMcpManifest,
    overwrite: bool,
) -> Result<PreparedMcpInstallation, ClientError> {
    let app_directory = app_directory()?;
    let expanded = expand_manifest(manifest, home, &app_directory)?;
    let exists = mcp_server_exists(home, &expanded.name)?;
    let installed = read_installed_mcp_packages(home)?;
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
                Some(current) => {
                    mcp_server_matches(home, &expand_manifest(current, home, &app_directory)?)?
                }
                None => false,
            };
            still_installed.then(|| previous.server_name.clone())
        }
        None => None,
    };
    Ok(PreparedMcpInstallation {
        installed,
        previous_server,
        expanded,
    })
}

pub(crate) fn mcp_installation_status(
    home: &Path,
    clients: &[AgentClient],
    package: &str,
    version: &str,
    commit_sha: &str,
) -> Result<McpInstallationStatus, ClientError> {
    if clients.is_empty() {
        return Ok(McpInstallationStatus {
            action: ExtensionInstallAction::Install,
            clients: Vec::new(),
        });
    }
    let app_directory = app_directory()?;
    let mut installed = read_installed_mcp_packages(home)?;
    let Some(current) = installed.0.get(package).cloned() else {
        return Ok(McpInstallationStatus {
            action: ExtensionInstallAction::Install,
            clients: Vec::new(),
        });
    };
    if !mcp_server_exists(home, &current.server_name)? {
        installed.0.remove(package);
        write_installed_mcp_packages(home, &installed)?;
        return Ok(McpInstallationStatus {
            action: ExtensionInstallAction::Install,
            clients: Vec::new(),
        });
    }
    let outdated = current.version != version
        || current.commit_sha != commit_sha
        || match current.manifest.as_ref() {
            Some(manifest) => {
                !mcp_server_matches(home, &expand_manifest(manifest, home, &app_directory)?)?
            }
            None => true,
        };
    let action = if outdated {
        ExtensionInstallAction::Update
    } else {
        ExtensionInstallAction::Installed
    };
    Ok(McpInstallationStatus {
        action,
        clients: clients.to_vec(),
    })
}

pub(crate) fn retain_listed_mcp_packages(
    home: &Path,
    clients: &[AgentClient],
    listed_packages: &BTreeSet<String>,
) -> Result<(), ClientError> {
    if clients.is_empty() {
        return Ok(());
    }
    let mut installed = read_installed_mcp_packages(home)?;
    let initial_len = installed.0.len();
    installed
        .0
        .retain(|package, _| listed_packages.contains(package));
    if installed.0.len() != initial_len {
        write_installed_mcp_packages(home, &installed)?;
    }
    Ok(())
}

pub(crate) fn uninstall_mcp_package(
    home: &Path,
    _client: AgentClient,
    package: &str,
) -> Result<(), ClientError> {
    let mut installed = read_installed_mcp_packages(home)?;
    let Some(entry) = installed.0.remove(package) else {
        return Ok(());
    };
    remove_mcp_server(home, &entry.server_name)?;
    write_installed_mcp_packages(home, &installed)
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
    _client: AgentClient,
) -> BTreeMap<String, ManagedMcpServer> {
    read_installed_mcp_packages(home)
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

fn mcp_server_exists(home: &Path, name: &str) -> Result<bool, ClientError> {
    codex_mcp_server_exists(home, name)
        .map_err(|error| ClientError::new("local_extensions_error", error))
}

fn upsert_mcp_server(home: &Path, manifest: &ExtensionMcpManifest) -> Result<(), ClientError> {
    upsert_codex_mcp_server(home, manifest)
        .map_err(|error| ClientError::new("local_extensions_error", error))
}

fn mcp_server_matches(home: &Path, manifest: &ExtensionMcpManifest) -> Result<bool, ClientError> {
    codex_mcp_server_matches(home, manifest)
        .map_err(|error| ClientError::new("local_extensions_error", error))
}

fn remove_mcp_server(home: &Path, name: &str) -> Result<(), ClientError> {
    if !mcp_server_exists(home, name)? {
        return Ok(());
    }
    remove_codex_mcp_server(home, name)
        .map_err(|error| ClientError::new("local_extensions_error", error))
}

fn read_installed_mcp_packages(home: &Path) -> Result<InstalledMcpPackages, ClientError> {
    let path = mcp_package_state_path(home);
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
    packages: &InstalledMcpPackages,
) -> Result<(), ClientError> {
    let path = mcp_package_state_path(home);
    let contents = serde_json::to_vec(packages).map_err(|error| {
        ClientError::new(
            "local_extensions_error",
            format!("无法保存已安装 MCP 状态：{error}"),
        )
    })?;
    atomic_write(&path, &contents)
}

fn mcp_package_state_path(home: &Path) -> PathBuf {
    home.join(".codex")
        .join(PRELAY_STATE_DIRECTORY)
        .join(MCP_PACKAGE_STATE_FILE)
}

#[cfg(test)]
#[path = "mcp_state_tests.rs"]
mod state_tests;
#[cfg(test)]
#[path = "mcp_tests.rs"]
mod tests;

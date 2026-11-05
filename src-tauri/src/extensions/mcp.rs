use std::{collections::BTreeMap, path::Path};

use prelay_protocol::ExtensionMcpManifest;

use crate::{
    agents::{
        codex_mcp_manifest_fingerprint, codex_mcp_server_exists, codex_mcp_server_fingerprint,
        remove_codex_mcp_server, upsert_codex_mcp_server,
    },
    relay::client::ClientError,
};

use super::{
    host_state::{ArtifactHealth, HostState, PluginArtifact},
    mcp_manifest::{app_directory, expand_manifest},
};

#[derive(Clone, Debug)]
pub(crate) struct McpInstall {
    pub server_name: String,
    pub manifest_hash: String,
}

#[derive(Clone, Debug)]
pub(crate) struct ManagedMcpServer {
    pub package: String,
    pub version: String,
}

pub(crate) fn preflight_mcp_install(
    home: &Path,
    package: &str,
    manifest: &ExtensionMcpManifest,
    overwrite: bool,
    state: &HostState,
) -> Result<McpInstall, ClientError> {
    let expanded = expand_manifest(manifest, home, &app_directory()?)?;
    let expected = codex_mcp_manifest_fingerprint(&expanded)
        .map_err(|error| ClientError::new("local_extensions_error", error))?;
    let exists = codex_mcp_server_exists(home, &expanded.name)
        .map_err(|error| ClientError::new("local_extensions_error", error))?;
    let owned = state.plugin(package).is_some_and(|plugin| {
        plugin.artifacts.iter().any(|artifact| {
            matches!(
                artifact,
                PluginArtifact::Mcp { server_name, .. } if server_name == &expanded.name
            )
        })
    });
    let owned_by_other = state.active_plugins().any(|plugin| {
        plugin.name != package
            && plugin.artifacts.iter().any(|artifact| {
                matches!(
                    artifact,
                    PluginArtifact::Mcp { server_name, .. } if server_name == &expanded.name
                )
            })
    });
    if owned_by_other {
        return Err(ClientError::new(
            "extension_target_exists",
            "MCP 服务名已由另一个扩展包管理。",
        ));
    }
    if exists && !owned && !overwrite {
        return Err(ClientError::new(
            "extension_target_exists",
            "MCP 服务名已存在，确认后可覆盖安装。",
        ));
    }
    Ok(McpInstall {
        server_name: expanded.name,
        manifest_hash: expected,
    })
}

pub(super) fn install_mcp(
    home: &Path,
    package: &str,
    manifest: &ExtensionMcpManifest,
    overwrite: bool,
    state: &HostState,
) -> Result<McpInstall, ClientError> {
    let install = preflight_mcp_install(home, package, manifest, overwrite, state)?;
    let expanded = expand_manifest(manifest, home, &app_directory()?)?;
    let previous = state.plugin(package).and_then(|plugin| {
        plugin.artifacts.iter().find_map(|artifact| match artifact {
            PluginArtifact::Mcp { server_name, .. } if server_name != &install.server_name => {
                Some(server_name.clone())
            }
            _ => None,
        })
    });
    upsert_codex_mcp_server(home, &expanded)
        .map_err(|error| ClientError::new("local_extensions_error", error))?;
    if let Some(previous) = previous {
        remove_mcp_server(home, &previous)?;
    }
    Ok(install)
}

pub(crate) fn remove_mcp(home: &Path, server_name: &str) -> Result<(), ClientError> {
    remove_mcp_server(home, server_name)
}

pub(crate) fn health(
    home: &Path,
    server_name: &str,
    manifest_hash: &str,
) -> Result<ArtifactHealth, ClientError> {
    let actual = codex_mcp_server_fingerprint(home, server_name)
        .map_err(|error| ClientError::new("local_extensions_error", error))?;
    Ok(match actual {
        None => ArtifactHealth::Missing,
        Some(actual) if actual == manifest_hash => ArtifactHealth::Managed,
        Some(_) => ArtifactHealth::Modified,
    })
}

pub(crate) fn managed_mcp_servers(state: &HostState) -> BTreeMap<String, ManagedMcpServer> {
    let mut servers = BTreeMap::new();
    for plugin in state.active_plugins() {
        for artifact in &plugin.artifacts {
            if let PluginArtifact::Mcp { server_name, .. } = artifact {
                servers.insert(
                    server_name.clone(),
                    ManagedMcpServer {
                        package: plugin.name.clone(),
                        version: plugin.version.clone(),
                    },
                );
            }
        }
    }
    servers
}

pub(crate) fn manifest_fingerprint(
    home: &Path,
    manifest: &ExtensionMcpManifest,
) -> Result<String, ClientError> {
    let expanded = expand_manifest(manifest, home, &app_directory()?)?;
    codex_mcp_manifest_fingerprint(&expanded)
        .map_err(|error| ClientError::new("local_extensions_error", error))
}

fn remove_mcp_server(home: &Path, name: &str) -> Result<(), ClientError> {
    let exists = codex_mcp_server_exists(home, name)
        .map_err(|error| ClientError::new("local_extensions_error", error))?;
    if !exists {
        return Ok(());
    }
    remove_codex_mcp_server(home, name)
        .map_err(|error| ClientError::new("local_extensions_error", error))
}

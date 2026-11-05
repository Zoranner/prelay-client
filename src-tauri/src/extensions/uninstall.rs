use std::path::Path;

use crate::{agents::AgentClient, relay::client::ClientError};

use super::{
    host_state::{self, ExtensionHost, HostState, PluginArtifact, PluginState},
    mcp, skills,
};

pub(crate) fn uninstall_plugin_for_client(
    home: &Path,
    client: AgentClient,
    package: &str,
) -> Result<(), ClientError> {
    let host = ExtensionHost::from_client(client);
    let _lock = host_state::lock(home, host)?;
    host_state::ensure_migrated_locked(home)?;
    let state = HostState::load_locked(home, host)?;
    state.ensure_writable()?;
    let Some(record) = state.plugin(package).cloned() else {
        return Err(ClientError::new(
            "local_extensions_error",
            format!("本机没有扩展包 {package} 的安装记录。"),
        ));
    };
    preflight(home, &record)?;

    let mut removing = record.clone();
    removing.state = PluginState::Removing;
    host_state::write_plugin(home, host, &removing)?;

    for artifact in &record.artifacts {
        if shared_with_active_plugin(&state, package, artifact) {
            continue;
        }
        match artifact {
            PluginArtifact::Skill { target, names } => {
                skills::remove_skill_roots(Path::new(target), names)?;
            }
            PluginArtifact::Mcp { server_name, .. } => {
                mcp::remove_mcp(home, server_name)?;
            }
            PluginArtifact::Hook { handler_hash, .. } => {
                super::hooks::remove_hook_handler(home, handler_hash)?;
            }
        }
    }
    host_state::remove_plugin(home, host, package)?;
    cleanup_package_root(home, &record.name, &record.commit_sha)?;
    Ok(())
}

fn preflight(home: &Path, record: &super::host_state::PluginRecord) -> Result<(), ClientError> {
    for artifact in &record.artifacts {
        if let PluginArtifact::Mcp {
            server_name,
            manifest_hash,
            ..
        } = artifact
        {
            if mcp::health(home, server_name, manifest_hash)?
                == super::host_state::ArtifactHealth::Modified
            {
                return Err(ClientError::new(
                    "local_extensions_conflict",
                    format!("MCP 服务 {server_name} 已被修改，未卸载扩展包。"),
                ));
            }
        }
    }
    Ok(())
}

fn shared_with_active_plugin(state: &HostState, package: &str, artifact: &PluginArtifact) -> bool {
    state.active_plugins().any(|plugin| {
        plugin.name != package
            && plugin
                .artifacts
                .iter()
                .any(|candidate| artifacts_overlap(candidate, artifact))
    })
}

fn artifacts_overlap(left: &PluginArtifact, right: &PluginArtifact) -> bool {
    match (left, right) {
        (
            PluginArtifact::Skill {
                target: left_target,
                names: left_names,
            },
            PluginArtifact::Skill {
                target: right_target,
                names: right_names,
            },
        ) => {
            left_target == right_target && left_names.iter().any(|name| right_names.contains(name))
        }
        (
            PluginArtifact::Mcp {
                target: left_target,
                server_name: left_server,
                ..
            },
            PluginArtifact::Mcp {
                target: right_target,
                server_name: right_server,
                ..
            },
        ) => left_target == right_target && left_server == right_server,
        (
            PluginArtifact::Hook {
                target: left_target,
                event: left_event,
                matcher: left_matcher,
                handler_hash: left_hash,
            },
            PluginArtifact::Hook {
                target: right_target,
                event: right_event,
                matcher: right_matcher,
                handler_hash: right_hash,
            },
        ) => {
            left_target == right_target
                && left_event == right_event
                && left_matcher == right_matcher
                && left_hash == right_hash
        }
        _ => false,
    }
}

pub(crate) fn cleanup_package_root(
    home: &Path,
    name: &str,
    commit_sha: &str,
) -> Result<(), ClientError> {
    for host in ExtensionHost::ALL {
        let Ok(state) = HostState::load_locked(home, host) else {
            return Ok(());
        };
        if !state.errors.is_empty() {
            return Ok(());
        }
        if state.plugins.iter().any(|plugin| {
            plugin.name == name
                && plugin.commit_sha == commit_sha
                && plugin
                    .artifacts
                    .iter()
                    .any(|artifact| matches!(artifact, PluginArtifact::Hook { .. }))
        }) {
            return Ok(());
        }
    }
    super::hooks::remove_package_root(home, name, commit_sha)
}

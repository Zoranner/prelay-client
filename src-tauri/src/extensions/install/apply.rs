use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use prelay_protocol::{ExtensionInstallBundle, ExtensionKind};

use crate::{
    agents::{agent_skill_target_roots, AgentClient},
    relay::client::ClientError,
};

use super::super::{
    host_state::{normalize_path, HostState, PluginArtifact},
    mcp, skills,
};

pub(super) fn apply_artifacts(
    home: &Path,
    bundle: &ExtensionInstallBundle,
    clients: &[AgentClient],
    overwrite: bool,
    state: &HostState,
    manifests: &BTreeMap<String, prelay_protocol::ExtensionMcpManifest>,
) -> Result<
    (
        Vec<PluginArtifact>,
        Vec<super::super::model::HookReviewItem>,
    ),
    ClientError,
> {
    let mut artifacts = Vec::new();
    let mut review = Vec::new();
    for artifact in &bundle.artifacts {
        match artifact.kind {
            ExtensionKind::Skill => {
                for target in agent_skill_target_roots(clients, home) {
                    let names = skills::install_skill_files(
                        &target,
                        &bundle.name,
                        &artifact.files,
                        overwrite,
                        state,
                    )?;
                    artifacts.push(PluginArtifact::Skill {
                        target: normalize_path(&target),
                        names,
                    });
                }
            }
            ExtensionKind::Mcp => {
                let manifest = manifests
                    .get(&bundle.name)
                    .ok_or_else(|| ClientError::new("invalid_response", "MCP 制品缺少清单。"))?;
                let install = mcp::install_mcp(home, &bundle.name, manifest, overwrite, state)?;
                artifacts.push(PluginArtifact::Mcp {
                    target: normalize_path(&home.join(".codex").join("config.toml")),
                    server_name: install.server_name,
                    manifest_hash: install.manifest_hash,
                });
            }
            ExtensionKind::Hook => {
                let previous = state
                    .plugin(&bundle.name)
                    .into_iter()
                    .flat_map(|plugin| plugin.hooks())
                    .filter_map(|artifact| match artifact {
                        PluginArtifact::Hook { handler_hash, .. } => Some(handler_hash.clone()),
                        _ => None,
                    })
                    .collect::<BTreeSet<_>>();
                let installed = super::super::hooks::install_hooks(
                    home,
                    &bundle.name,
                    &bundle.version.commit_sha,
                    &artifact.files,
                    &previous,
                )?;
                for entry in installed {
                    review.push(entry.review);
                    artifacts.push(PluginArtifact::Hook {
                        target: normalize_path(&home.join(".codex").join("hooks.json")),
                        event: entry.handler.event,
                        matcher: entry.handler.matcher,
                        handler_hash: entry.handler.hash,
                    });
                }
            }
            ExtensionKind::Rule | ExtensionKind::Unknown => {}
        }
    }
    Ok((artifacts, review))
}

pub(super) fn rollback(home: &Path, artifacts: &[PluginArtifact]) {
    for artifact in artifacts.iter().rev() {
        let _ = match artifact {
            PluginArtifact::Skill { target, names } => {
                skills::remove_skill_roots(Path::new(target), names)
            }
            PluginArtifact::Mcp { server_name, .. } => mcp::remove_mcp(home, server_name),
            PluginArtifact::Hook { handler_hash, .. } => {
                super::super::hooks::remove_hook_handler(home, handler_hash)
            }
        };
    }
}

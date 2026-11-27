use std::path::Path;

use crate::relay::client::ClientError;

use super::super::{
    hooks,
    host_state::{PluginArtifact, PluginRecord},
    mcp, skills,
};

pub(super) fn remove_obsolete(
    home: &Path,
    previous: &PluginRecord,
    planned: &[PluginArtifact],
) -> Result<(), ClientError> {
    for artifact in &previous.artifacts {
        if let PluginArtifact::Skill { target, names } = artifact {
            let removed = names
                .iter()
                .filter(|name| {
                    !planned.iter().any(|artifact| {
                        matches!(
                            artifact,
                            PluginArtifact::Skill {
                                target: planned_target,
                                names: planned_names,
                            } if planned_target == target && planned_names.contains(name)
                        )
                    })
                })
                .cloned()
                .collect::<Vec<_>>();
            if !removed.is_empty() {
                skills::remove_skill_roots(Path::new(target), &removed)?;
            }
            continue;
        }
        if artifact_is_kept(artifact, planned) {
            continue;
        }
        match artifact {
            PluginArtifact::Mcp { server_name, .. } => {
                mcp::remove_mcp(home, server_name)?;
            }
            PluginArtifact::Hook { handler_hash, .. } => {
                hooks::remove_hook_handler(home, handler_hash)?;
            }
            PluginArtifact::Skill { .. } => {}
        }
    }
    Ok(())
}

fn artifact_is_kept(current: &PluginArtifact, planned: &[PluginArtifact]) -> bool {
    match current {
        PluginArtifact::Skill { target, names } => names.iter().all(|name| {
            planned.iter().any(|artifact| {
                matches!(
                    artifact,
                    PluginArtifact::Skill {
                        target: planned_target,
                        names: planned_names,
                    } if planned_target == target && planned_names.contains(name)
                )
            })
        }),
        PluginArtifact::Mcp {
            target,
            server_name,
            ..
        } => planned.iter().any(|artifact| {
            matches!(
                artifact,
                PluginArtifact::Mcp {
                    target: planned_target,
                    server_name: planned_server,
                    ..
                } if planned_target == target && planned_server == server_name
            )
        }),
        PluginArtifact::Hook {
            target,
            event,
            matcher,
            handler_hash,
        } => planned.iter().any(|artifact| {
            matches!(
                artifact,
                PluginArtifact::Hook {
                    target: planned_target,
                    event: planned_event,
                    matcher: planned_matcher,
                    handler_hash: planned_hash,
                } if planned_target == target
                    && planned_event == event
                    && planned_matcher == matcher
                    && planned_hash == handler_hash
            )
        }),
    }
}

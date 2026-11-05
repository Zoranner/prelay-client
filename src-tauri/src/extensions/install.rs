use std::{collections::BTreeMap, path::Path};

use prelay_protocol::{ExtensionInstallBundle, ExtensionKind};

use super::{
    host_state::{
        self, normalize_path, ExtensionHost, HostState, PluginArtifact, PluginRecord, PluginState,
    },
    mcp, mcp_manifest,
    model::{
        ExtensionInstallRequest, ExtensionInstallResult, ExtensionPackage, HookReviewItem,
        McpInstallPreview,
    },
    rules, skills,
};

use crate::{
    agents::{agent_rule_targets, agent_skill_target_roots, AgentClient},
    identity::registration::authenticated_api,
    relay::client::ClientError,
    NativeState,
};

mod apply;
mod validate;
pub(crate) use validate::{validate_bundle, validate_package_bundle};

pub async fn install_extension(
    home: &Path,
    state: &NativeState,
    request: &ExtensionInstallRequest,
) -> Result<ExtensionInstallResult, ClientError> {
    let client = authenticated_api(state).await?;
    let bundle: ExtensionInstallBundle = client
        .get(&format!(
            "/api/extensions/{}/versions/{}/install",
            request.package.name, request.package.version
        ))
        .await?;
    validate_package_bundle(&request.package, &bundle)?;
    let review = install_prepared_bundle(home, &bundle, &request.clients, request.overwrite)?;
    Ok(ExtensionInstallResult {
        message: if bundle.skipped.is_empty() {
            format!("已安装{}。", bundle.name)
        } else {
            format!(
                "已安装{}，跳过 {} 个未匹配文件。",
                bundle.name,
                bundle.skipped.len()
            )
        },
        skipped: bundle.skipped,
        review,
    })
}

pub async fn read_mcp_install_manifest(
    state: &NativeState,
    package: &ExtensionPackage,
) -> Result<McpInstallPreview, ClientError> {
    if !package
        .artifacts
        .iter()
        .any(|artifact| artifact.kind == ExtensionKind::Mcp)
    {
        return Err(ClientError::new(
            "invalid_request",
            "只有包含 MCP 制品的扩展可以读取安装配置。",
        ));
    }
    let client = authenticated_api(state).await?;
    let bundle: ExtensionInstallBundle = client
        .get(&format!(
            "/api/extensions/{}/versions/{}/install",
            package.name, package.version
        ))
        .await?;
    validate_package_bundle(package, &bundle)?;
    let artifact = bundle
        .artifacts
        .iter()
        .find(|artifact| artifact.kind == ExtensionKind::Mcp)
        .ok_or_else(|| ClientError::new("invalid_response", "安装包缺少 MCP 制品。"))?;
    Ok(McpInstallPreview {
        name: bundle.name,
        version: bundle.version.tag,
        commit_sha: bundle.version.commit_sha,
        manifest: mcp_manifest::read_mcp_manifest(
            artifact
                .files
                .first()
                .ok_or_else(|| ClientError::new("invalid_response", "MCP 制品为空。"))?,
        )?,
    })
}

pub(crate) fn install_prepared_bundle(
    home: &Path,
    bundle: &ExtensionInstallBundle,
    clients: &[AgentClient],
    overwrite: bool,
) -> Result<Vec<HookReviewItem>, ClientError> {
    validate_bundle(bundle)?;
    let host = ExtensionHost::from_clients(clients)
        .ok_or_else(|| ClientError::new("invalid_request", "未选择可安装的客户端。"))?;
    let _lock = host_state::lock(home, host)?;
    host_state::ensure_migrated_locked(home)?;
    let state = HostState::load_locked(home, host)?;
    state.ensure_writable()?;
    if bundle
        .artifacts
        .iter()
        .any(|artifact| artifact.kind == ExtensionKind::Rule)
    {
        install_rule_bundle(home, host, clients, bundle, overwrite)?;
        return Ok(Vec::new());
    }
    install_plugin_bundle(home, host, clients, bundle, overwrite, &state)
}

fn install_rule_bundle(
    home: &Path,
    host: ExtensionHost,
    clients: &[AgentClient],
    bundle: &ExtensionInstallBundle,
    overwrite: bool,
) -> Result<(), ClientError> {
    let artifact = &bundle.artifacts[0];
    let file = artifact
        .files
        .first()
        .ok_or_else(|| ClientError::new("invalid_response", "规则制品为空。"))?;
    let mut rules_state = HostState::load_locked(home, host)?.rules;
    let mut plans = Vec::new();
    for target in agent_rule_targets(clients, home) {
        let key = normalize_path(&target);
        let previous = rules_state.targets.get(&key).cloned();
        if let Some(entry) = &previous {
            if entry.package != bundle.name {
                return Err(ClientError::new(
                    "extension_target_exists",
                    "规则目标已由另一个扩展包管理。",
                ));
            }
        } else if target.exists() && !overwrite {
            return Err(ClientError::new(
                "extension_target_exists",
                "规则目标已存在，确认后可覆盖安装。",
            ));
        }
        let (content, planned) = rules::plan_rule(
            &bundle.name,
            &bundle.version.tag,
            &bundle.version.commit_sha,
            file,
        )?;
        plans.push((target, key, previous, content, planned));
    }
    for (target, key, previous, content, planned) in plans {
        rules::remember(&mut rules_state, &target, planned);
        host_state::write_rules(home, host, &rules_state)?;
        if let Err(error) = rules::write_rule(&target, &content) {
            if let Some(previous) = previous {
                rules::remember(&mut rules_state, &target, previous);
            } else {
                rules::forget(&mut rules_state, &target);
            }
            host_state::write_rules(home, host, &rules_state)?;
            return Err(error);
        }
        if let Some(entry) = rules_state.targets.get_mut(&key) {
            entry.state = PluginState::Installed;
        }
        host_state::write_rules(home, host, &rules_state)?;
    }
    Ok(())
}

fn install_plugin_bundle(
    home: &Path,
    host: ExtensionHost,
    clients: &[AgentClient],
    bundle: &ExtensionInstallBundle,
    overwrite: bool,
    state: &HostState,
) -> Result<Vec<HookReviewItem>, ClientError> {
    let skill_targets = agent_skill_target_roots(clients, home);
    let mcp_target = normalize_path(&home.join(".codex").join("config.toml"));
    let hook_target = normalize_path(&home.join(".codex").join("hooks.json"));
    let mut planned = Vec::new();
    let mut manifests = BTreeMap::new();
    let mut planned_hook_handlers = Vec::new();
    let mut planned_hook_artifacts = Vec::new();
    let previous = state.plugin(&bundle.name).cloned();
    for artifact in &bundle.artifacts {
        match artifact.kind {
            ExtensionKind::Skill => {
                for target in &skill_targets {
                    let names = skills::preflight_skill_files(
                        target,
                        &bundle.name,
                        &artifact.files,
                        overwrite,
                        state,
                    )?;
                    planned.push(PluginArtifact::Skill {
                        target: normalize_path(target),
                        names,
                    });
                }
            }
            ExtensionKind::Mcp => {
                let manifest = mcp_manifest::read_mcp_manifest(
                    artifact
                        .files
                        .first()
                        .ok_or_else(|| ClientError::new("invalid_response", "MCP 制品为空。"))?,
                )?;
                let install =
                    mcp::preflight_mcp_install(home, &bundle.name, &manifest, overwrite, state)?;
                planned.push(PluginArtifact::Mcp {
                    target: mcp_target.clone(),
                    server_name: install.server_name.clone(),
                    manifest_hash: install.manifest_hash.clone(),
                });
                manifests.insert(bundle.name.clone(), manifest);
            }
            ExtensionKind::Hook => {
                let handlers = super::hooks::plan_hooks(
                    home,
                    &bundle.name,
                    &bundle.version.commit_sha,
                    &artifact.files,
                )?;
                planned_hook_handlers = handlers;
                planned_hook_artifacts = planned_hook_handlers
                    .iter()
                    .map(|handler| PluginArtifact::Hook {
                        target: hook_target.clone(),
                        event: handler.event.clone(),
                        matcher: handler.matcher.clone(),
                        handler_hash: handler.hash.clone(),
                    })
                    .collect();
            }
            ExtensionKind::Rule | ExtensionKind::Unknown => {}
        }
    }
    planned.extend(planned_hook_artifacts);
    if !planned_hook_handlers.is_empty() {
        super::hooks::preflight_hook_install(
            home,
            &bundle.name,
            &planned_hook_handlers,
            overwrite,
            state,
        )?;
    }
    if let Some(previous) = &previous {
        preflight_obsolete(home, previous, &planned)?;
    }
    let installing = PluginRecord {
        schema_version: host_state::PLUGIN_SCHEMA_VERSION,
        name: bundle.name.clone(),
        version: bundle.version.tag.clone(),
        commit_sha: bundle.version.commit_sha.clone(),
        state: PluginState::Installing,
        artifacts: planned.clone(),
    };
    host_state::write_plugin(home, host, &installing)?;

    let existing = previous.is_some();
    let applied = apply::apply_artifacts(home, bundle, clients, overwrite, state, &manifests);
    let (artifacts, review) = match applied {
        Ok(applied) => applied,
        Err(error) => {
            if overwrite || existing {
                return Err(error);
            }
            apply::rollback(home, &planned);
            let _ = host_state::remove_plugin(home, host, &bundle.name);
            return Err(error);
        }
    };
    if let Some(previous) = &previous {
        remove_obsolete(home, previous, &artifacts)?;
    }
    let removes_hook = previous.as_ref().is_some_and(|plugin| {
        plugin
            .artifacts
            .iter()
            .any(|artifact| matches!(artifact, PluginArtifact::Hook { .. }))
    }) && !artifacts
        .iter()
        .any(|artifact| matches!(artifact, PluginArtifact::Hook { .. }));
    let record = PluginRecord {
        schema_version: host_state::PLUGIN_SCHEMA_VERSION,
        name: bundle.name.clone(),
        version: bundle.version.tag.clone(),
        commit_sha: bundle.version.commit_sha.clone(),
        state: PluginState::Installed,
        artifacts,
    };
    host_state::write_plugin(home, host, &record)?;
    if removes_hook {
        let _ =
            super::uninstall::cleanup_package_root(home, &bundle.name, &bundle.version.commit_sha);
    }
    if let Some(previous) = previous.filter(|plugin| plugin.commit_sha != bundle.version.commit_sha)
    {
        let _ = super::uninstall::cleanup_package_root(home, &previous.name, &previous.commit_sha);
    }
    Ok(review)
}

fn preflight_obsolete(
    home: &Path,
    previous: &PluginRecord,
    planned: &[PluginArtifact],
) -> Result<(), ClientError> {
    for artifact in &previous.artifacts {
        if artifact_is_kept(artifact, planned) {
            continue;
        }
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
                    format!("MCP 服务 {server_name} 已被修改，未更新扩展包。"),
                ));
            }
        }
    }
    Ok(())
}

fn remove_obsolete(
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
                super::hooks::remove_hook_handler(home, handler_hash)?;
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

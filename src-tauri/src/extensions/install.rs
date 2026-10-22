use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use prelay_protocol::{ExtensionFile, ExtensionInstallBundle, ExtensionKind};

use super::{
    hooks, mcp, mcp_manifest,
    model::{ExtensionInstallRequest, ExtensionInstallResult, ExtensionPackage, McpInstallPreview},
    record::{client_key, upsert_extension_record, InstalledTargetRecord},
    rules, skills,
};
use crate::{
    agents::{agent_rule_targets, agent_skill_target_roots, AgentClient},
    identity::registration::authenticated_api,
    relay::client::ClientError,
    NativeState,
};

const RULES_PATH: &str = "AGENTS.md";

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

pub(crate) fn install_prepared_bundle(
    home: &Path,
    bundle: &ExtensionInstallBundle,
    clients: &[AgentClient],
    overwrite: bool,
) -> Result<Vec<super::model::HookReviewItem>, ClientError> {
    validate_bundle(bundle)?;
    for artifact in &bundle.artifacts {
        match artifact.kind {
            ExtensionKind::Skill => {
                for target_root in agent_skill_target_roots(clients, home) {
                    skills::preflight_skill_files(
                        &target_root,
                        &bundle.name,
                        &artifact.files,
                        overwrite,
                    )?;
                }
            }
            ExtensionKind::Mcp => {
                let manifest = mcp_manifest::read_mcp_manifest(
                    artifact
                        .files
                        .first()
                        .ok_or_else(|| ClientError::new("invalid_response", "MCP 制品为空。"))?,
                )?;
                mcp::preflight_mcp_install(home, clients, &bundle.name, &manifest, overwrite)?;
            }
            ExtensionKind::Hook => {
                hooks::preflight_hooks(home, clients, &artifact.files)?;
            }
            ExtensionKind::Rule | ExtensionKind::Unknown => {}
        }
    }
    let mut review = Vec::new();
    let mut targets = clients
        .iter()
        .map(|client| {
            (
                client_key(*client).to_string(),
                InstalledTargetRecord::default(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for artifact in &bundle.artifacts {
        match artifact.kind {
            ExtensionKind::Rule => {
                let rules_file = artifact
                    .files
                    .first()
                    .ok_or_else(|| ClientError::new("invalid_response", "规则制品为空。"))?;
                for target in agent_rule_targets(clients, home) {
                    rules::install_rule(
                        &target,
                        &bundle.name,
                        &bundle.version.tag,
                        &bundle.version.commit_sha,
                        rules_file,
                    )?;
                }
                for target in targets.values_mut() {
                    target.rule = true;
                }
            }
            ExtensionKind::Skill => {
                for target_root in agent_skill_target_roots(clients, home) {
                    skills::install_skill_files(
                        &target_root,
                        &bundle.name,
                        &bundle.version.tag,
                        &bundle.version.commit_sha,
                        &artifact.files,
                        overwrite,
                    )?;
                }
                let roots = skill_roots(&artifact.files);
                for target in targets.values_mut() {
                    target.skills = roots.clone();
                }
            }
            ExtensionKind::Mcp => {
                let manifest = mcp_manifest::read_mcp_manifest(
                    artifact
                        .files
                        .first()
                        .ok_or_else(|| ClientError::new("invalid_response", "MCP 制品为空。"))?,
                )?;
                mcp::install_mcp(
                    home,
                    clients,
                    &bundle.name,
                    &bundle.version.tag,
                    &bundle.version.commit_sha,
                    &manifest,
                    overwrite,
                )?;
                for target in targets.values_mut() {
                    target.mcps = vec![manifest.name.clone()];
                }
            }
            ExtensionKind::Hook => {
                let hook_review = hooks::install_hooks(
                    home,
                    clients,
                    &bundle.name,
                    &bundle.version.tag,
                    &bundle.version.commit_sha,
                    &artifact.files,
                    overwrite,
                )?;
                if let Some(target) = targets.get_mut("codexCli") {
                    target.hooks = hook_labels(&hook_review);
                }
                review.extend(hook_review);
            }
            ExtensionKind::Unknown => {
                return Err(ClientError::new(
                    "extension_install_unsupported",
                    "当前客户端不支持该扩展制品，请升级 Prelay。",
                ));
            }
        }
    }
    upsert_extension_record(
        home,
        &bundle.name,
        &bundle.version.tag,
        &bundle.version.commit_sha,
        targets,
    )?;
    Ok(review)
}

pub(crate) fn validate_package_bundle(
    package: &ExtensionPackage,
    bundle: &ExtensionInstallBundle,
) -> Result<(), ClientError> {
    validate_bundle(bundle)?;
    if bundle.name != package.name
        || bundle.version.tag != package.version
        || bundle.version.commit_sha != package.commit_sha
    {
        return Err(ClientError::new(
            "invalid_response",
            "扩展安装包与已选择的固定版本不一致。",
        ));
    }
    if !package.artifacts.is_empty() {
        let expected = package
            .artifacts
            .iter()
            .map(|artifact| artifact.kind)
            .collect::<Vec<_>>();
        let actual = bundle
            .artifacts
            .iter()
            .map(|artifact| artifact.kind)
            .collect::<Vec<_>>();
        if expected.len() != actual.len() || expected.iter().any(|kind| !actual.contains(kind)) {
            return Err(ClientError::new(
                "invalid_response",
                "扩展安装包与目录声明的制品不一致。",
            ));
        }
    }
    Ok(())
}

fn validate_bundle(bundle: &ExtensionInstallBundle) -> Result<(), ClientError> {
    if bundle.artifacts.is_empty() {
        return Err(ClientError::new(
            "invalid_response",
            "extension install bundle is empty",
        ));
    }
    let mut kinds = Vec::new();
    for artifact in &bundle.artifacts {
        if kinds.contains(&artifact.kind) {
            return Err(ClientError::new(
                "invalid_response",
                "extension install bundle contains duplicate artifacts",
            ));
        }
        kinds.push(artifact.kind);
        if artifact.files.is_empty() {
            return Err(ClientError::new(
                "invalid_response",
                "extension install bundle contains an empty artifact",
            ));
        }
        match artifact.kind {
            ExtensionKind::Rule
                if bundle.artifacts.len() == 1
                    && artifact.files.len() == 1
                    && artifact.files[0].path == RULES_PATH => {}
            ExtensionKind::Skill
                if artifact
                    .files
                    .iter()
                    .all(|file| safe_skill_path(&file.path))
                    && artifact
                        .files
                        .iter()
                        .any(|file| file.path.ends_with("/SKILL.md")) => {}
            ExtensionKind::Mcp
                if artifact.files.len() == 1
                    && mcp_manifest::read_mcp_manifest(&artifact.files[0]).is_ok() => {}
            ExtensionKind::Hook
                if artifact.files.iter().all(|file| safe_hook_path(&file.path))
                    && hooks::validate_hook_files(&artifact.files).is_ok() => {}
            ExtensionKind::Unknown => {
                return Err(ClientError::new(
                    "extension_install_unsupported",
                    "当前客户端不支持该扩展制品，请升级 Prelay。",
                ));
            }
            _ => {
                return Err(ClientError::new(
                    "invalid_response",
                    "extension install bundle is invalid",
                ));
            }
        }
    }
    Ok(())
}

fn safe_skill_path(path: &str) -> bool {
    path.starts_with(super::model::SKILLS_PREFIX)
        && !path.contains('\\')
        && path
            .split('/')
            .all(|part| !part.is_empty() && !matches!(part, "." | ".."))
}

fn safe_hook_path(path: &str) -> bool {
    path.strip_prefix("hooks/").is_some_and(|name| {
        !name.is_empty()
            && !name.contains('/')
            && name.ends_with(".json")
            && !path.contains('\\')
            && path
                .split('/')
                .all(|part| !part.is_empty() && !matches!(part, "." | ".."))
    })
}

#[cfg(test)]
#[path = "install_tests.rs"]
mod tests;

fn skill_roots(files: &[ExtensionFile]) -> Vec<String> {
    files
        .iter()
        .filter_map(|file| {
            file.path
                .strip_prefix(super::model::SKILLS_PREFIX)?
                .split('/')
                .next()
                .filter(|root| !root.is_empty())
                .map(str::to_string)
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn hook_labels(review: &[super::model::HookReviewItem]) -> Vec<String> {
    review
        .iter()
        .map(|item| match &item.matcher {
            Some(matcher) => format!("{}:{}", item.event, matcher),
            None => item.event.clone(),
        })
        .collect()
}

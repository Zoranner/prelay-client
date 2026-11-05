use prelay_protocol::{ExtensionInstallBundle, ExtensionKind, HOOKS_MANIFEST_PATH};

use super::super::{mcp_manifest, model::ExtensionPackage};
use crate::relay::client::ClientError;

pub(super) const RULES_PATH: &str = "AGENTS.md";

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

pub(crate) fn validate_bundle(bundle: &ExtensionInstallBundle) -> Result<(), ClientError> {
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
                    && artifact
                        .files
                        .iter()
                        .filter(|file| file.path == HOOKS_MANIFEST_PATH)
                        .count()
                        == 1
                    && super::super::hooks::validate_hook_files(&artifact.files).is_ok() => {}
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
    path.starts_with(super::super::model::SKILLS_PREFIX)
        && !path.contains('\\')
        && path
            .split('/')
            .all(|part| !part.is_empty() && !matches!(part, "." | ".."))
}

fn safe_hook_path(path: &str) -> bool {
    path.starts_with("hooks/")
        && !path.contains('\\')
        && !path.contains('\0')
        && path
            .split('/')
            .all(|part| !part.is_empty() && !matches!(part, "." | ".."))
}

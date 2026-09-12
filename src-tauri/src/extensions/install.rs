use std::path::Path;

use prelay_protocol::{ExtensionInstallBundle, ExtensionKind};

use super::{
    local::{local_mcp_test_manifest, LOCAL_MCP_TEST_REPOSITORY},
    mcp,
    model::{ExtensionInstallRequest, ExtensionInstallResult, ExtensionPackage, McpInstallPreview},
    rules, skills,
};
use crate::{
    agents::{agent_rule_targets, agent_skill_target_roots},
    identity::registration::authenticated_api,
    relay::client::ClientError,
    NativeState,
};

const RULES_PATH: &str = "AGENTS.md";

pub async fn read_mcp_install_manifest(
    state: &NativeState,
    package: &ExtensionPackage,
) -> Result<McpInstallPreview, ClientError> {
    if package.kind != ExtensionKind::Mcp {
        return Err(ClientError::new(
            "invalid_request",
            "只有 MCP 扩展可以读取安装配置。",
        ));
    }
    if package.repository == LOCAL_MCP_TEST_REPOSITORY {
        let manifest = local_mcp_test_manifest();
        return Ok(McpInstallPreview {
            name: package.name.clone(),
            version: package.version.clone(),
            commit_sha: package.commit_sha.clone(),
            manifest,
        });
    }
    let client = authenticated_api(state).await?;
    let bundle: ExtensionInstallBundle = client
        .get(&format!(
            "/api/extensions/{}/versions/{}/install",
            package.name, package.version
        ))
        .await?;
    validate_package_bundle(package, &bundle)?;
    Ok(McpInstallPreview {
        name: bundle.name,
        version: bundle.version.tag,
        commit_sha: bundle.version.commit_sha,
        manifest: mcp::read_mcp_manifest(
            bundle.files.first().expect("validated MCP install bundle"),
        )?,
    })
}

pub async fn install_extension(
    home: &Path,
    state: &NativeState,
    request: &ExtensionInstallRequest,
) -> Result<ExtensionInstallResult, ClientError> {
    if request.package.repository == LOCAL_MCP_TEST_REPOSITORY {
        if request.package.kind != ExtensionKind::Mcp {
            return Err(ClientError::new(
                "invalid_request",
                "本地测试包必须是 MCP。",
            ));
        }
        let manifest = local_mcp_test_manifest();
        mcp::install_mcp(
            home,
            &request.clients,
            &request.package.name,
            &request.package.version,
            &request.package.commit_sha,
            &manifest,
            request.overwrite,
        )?;
        return Ok(ExtensionInstallResult {
            message: format!("已安装{}。", request.package.name),
        });
    }
    let client = authenticated_api(state).await?;
    let bundle: ExtensionInstallBundle = client
        .get(&format!(
            "/api/extensions/{}/versions/{}/install",
            request.package.name, request.package.version
        ))
        .await?;
    validate_package_bundle(&request.package, &bundle)?;

    match bundle.kind {
        ExtensionKind::Rule => {
            let rules_file = bundle.files.first().expect("validated rule bundle");
            for target in agent_rule_targets(&request.clients, home) {
                rules::install_rule(
                    &target,
                    &bundle.name,
                    &bundle.version.tag,
                    &bundle.version.commit_sha,
                    rules_file,
                )?;
            }
        }
        ExtensionKind::Skill => {
            for target_root in agent_skill_target_roots(&request.clients, home) {
                skills::install_skill_files(
                    &target_root,
                    &bundle.name,
                    &bundle.version.tag,
                    &bundle.version.commit_sha,
                    &bundle.files,
                    request.overwrite,
                )?;
            }
        }
        ExtensionKind::Mcp => {
            let manifest = mcp::read_mcp_manifest(
                bundle.files.first().expect("validated MCP install bundle"),
            )?;
            mcp::install_mcp(
                home,
                &request.clients,
                &bundle.name,
                &bundle.version.tag,
                &bundle.version.commit_sha,
                &manifest,
                request.overwrite,
            )?;
        }
    }
    Ok(ExtensionInstallResult {
        message: format!("已安装{}。", bundle.name),
    })
}

pub(crate) fn validate_package_bundle(
    package: &ExtensionPackage,
    bundle: &ExtensionInstallBundle,
) -> Result<(), ClientError> {
    validate_bundle(bundle)?;
    if bundle.name != package.name
        || bundle.kind != package.kind
        || bundle.version.tag != package.version
        || bundle.version.commit_sha != package.commit_sha
    {
        return Err(ClientError::new(
            "invalid_response",
            "扩展安装包与已选择的固定版本不一致。",
        ));
    }
    Ok(())
}

fn validate_bundle(bundle: &ExtensionInstallBundle) -> Result<(), ClientError> {
    match bundle.kind {
        ExtensionKind::Rule if bundle.files.len() == 1 && bundle.files[0].path == RULES_PATH => {
            Ok(())
        }
        ExtensionKind::Skill
            if !bundle.files.is_empty()
                && bundle.files.iter().all(|file| safe_skill_path(&file.path)) =>
        {
            Ok(())
        }
        ExtensionKind::Mcp if bundle.files.len() == 1 => {
            mcp::read_mcp_manifest(&bundle.files[0]).map(|_| ())
        }
        _ => Err(ClientError::new(
            "invalid_response",
            "extension install bundle is invalid",
        )),
    }
}

fn safe_skill_path(path: &str) -> bool {
    path.starts_with(super::model::SKILLS_PREFIX)
        && !path.contains('\\')
        && path
            .split('/')
            .all(|part| !part.is_empty() && !matches!(part, "." | ".."))
}

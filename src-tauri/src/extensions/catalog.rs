use std::path::Path;

use prelay_protocol::{ExtensionInstallBundle, ExtensionSummary};

use super::{
    install::validate_package_bundle,
    mcp, mcp_manifest,
    model::{ExtensionCatalogSnapshot, ExtensionInstallAction, ExtensionKind, ExtensionPackage},
    rules, skills,
};
use crate::{
    agents::{
        agent_rule_targets, agent_rule_targets_with_clients, agent_skill_target_roots,
        agent_skill_targets, installed_agent_clients, AgentClient,
    },
    identity::registration::authenticated_api,
    relay::client::ClientError,
    NativeState,
};

pub async fn list_extensions(
    home: &Path,
    state: &NativeState,
    kind: ExtensionKind,
) -> Result<ExtensionCatalogSnapshot, ClientError> {
    let path = match kind {
        ExtensionKind::Rule => "/api/extensions/rules",
        ExtensionKind::Skill => "/api/extensions/skills",
        ExtensionKind::Mcp => "/api/extensions/mcp",
    };
    let client = authenticated_api(state).await?;
    let summaries: Vec<ExtensionSummary> = client.get(path).await?;
    let mut packages = summaries
        .into_iter()
        .map(|summary| ExtensionPackage {
            name: summary.name,
            repository: summary.repository,
            commit_sha: summary.latest.commit_sha,
            version: summary.latest.tag,
            kind,
            install_action: ExtensionInstallAction::Install,
            installed_clients: Vec::new(),
        })
        .collect::<Vec<_>>();
    let clients = installed_agent_clients();
    match kind {
        ExtensionKind::Rule => {
            let targets = agent_rule_targets_with_clients(&clients, home);
            let listed = packages
                .iter()
                .map(|package| package.name.clone())
                .collect();
            rules::retain_listed_rule_packages(&targets, &listed)?;
            for package in &mut packages {
                let status = rules::rule_installation_status(
                    &targets,
                    &package.name,
                    &package.version,
                    &package.commit_sha,
                )?;
                package.install_action = status.action;
                package.installed_clients = status.clients;
            }
        }
        ExtensionKind::Skill => {
            let targets = agent_skill_targets(&clients, home);
            let target_roots = agent_skill_target_roots(&clients, home);
            let listed = packages
                .iter()
                .map(|package| package.name.clone())
                .collect();
            skills::retain_listed_skill_packages(&target_roots, &listed)?;
            for package in &mut packages {
                let status = skills::skill_installation_status(
                    &targets,
                    &package.name,
                    &package.version,
                    &package.commit_sha,
                )?;
                package.install_action = status.action;
                package.installed_clients = status.clients;
            }
        }
        ExtensionKind::Mcp => {
            let listed = packages
                .iter()
                .map(|package| package.name.clone())
                .collect();
            mcp::retain_listed_mcp_packages(home, &clients, &listed)?;
            for package in &mut packages {
                let status = mcp::mcp_installation_status(
                    home,
                    &clients,
                    &package.name,
                    &package.version,
                    &package.commit_sha,
                )?;
                package.install_action = status.action;
                package.installed_clients = status.clients;
            }
        }
    }
    Ok(ExtensionCatalogSnapshot { packages })
}

pub async fn read_extension_readme(
    state: &NativeState,
    package: &ExtensionPackage,
) -> Result<String, ClientError> {
    let client = authenticated_api(state).await?;
    let bytes = client
        .get_bytes(&format!(
            "/api/extensions/{}/versions/{}/readme",
            package.name, package.version
        ))
        .await?;
    String::from_utf8(bytes)
        .map_err(|_| ClientError::new("invalid_response", "extension README is not UTF-8"))
}

pub async fn update_all_extensions(
    home: &Path,
    state: &NativeState,
    kind: ExtensionKind,
) -> Result<super::model::ExtensionInstallResult, ClientError> {
    let mut catalog = list_extensions(home, state, kind).await?;
    catalog.packages.retain(|package| {
        package.kind == kind && package.install_action == ExtensionInstallAction::Update
    });
    if catalog.packages.is_empty() {
        return Ok(super::model::ExtensionInstallResult {
            message: "没有可更新扩展。".to_string(),
        });
    }

    let client = authenticated_api(state).await?;
    let updated = catalog.packages.len();
    for package in catalog.packages {
        let bundle: ExtensionInstallBundle = client
            .get(&format!(
                "/api/extensions/{}/versions/{}/install",
                package.name, package.version
            ))
            .await?;
        validate_package_bundle(&package, &bundle)?;
        install_prepared_bundle(home, &bundle, &package.installed_clients)?;
    }
    Ok(super::model::ExtensionInstallResult {
        message: format!("已更新 {updated} 个扩展。"),
    })
}

/// 已通过 `validate_package_bundle` 校验的安装包按类型落到本机目标；
/// 只覆盖该扩展已经拥有的目标，不接管同名外部内容。
fn install_prepared_bundle(
    home: &Path,
    bundle: &ExtensionInstallBundle,
    clients: &[AgentClient],
) -> Result<(), ClientError> {
    match bundle.kind {
        ExtensionKind::Rule => {
            let rules_file = bundle.files.first().expect("validated rule bundle");
            for target in agent_rule_targets(clients, home) {
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
            for target_root in agent_skill_target_roots(clients, home) {
                skills::install_skill_files(
                    &target_root,
                    &bundle.name,
                    &bundle.version.tag,
                    &bundle.version.commit_sha,
                    &bundle.files,
                    false,
                )?;
            }
        }
        ExtensionKind::Mcp => {
            let manifest = mcp_manifest::read_mcp_manifest(
                bundle.files.first().expect("validated MCP install bundle"),
            )?;
            mcp::install_mcp(
                home,
                clients,
                &bundle.name,
                &bundle.version.tag,
                &bundle.version.commit_sha,
                &manifest,
                false,
            )?;
        }
    }
    Ok(())
}

use std::path::Path;

use prelay_protocol::{ExtensionInstallBundle, ExtensionSummary};

use super::{
    install::validate_package_bundle,
    mcp,
    model::{ExtensionCatalogSnapshot, ExtensionInstallAction, ExtensionKind, ExtensionPackage},
    rules, skills,
};
use crate::{
    agents::{
        agent_rule_targets_with_clients, agent_skill_target_roots, agent_skill_targets,
        installed_agent_clients,
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
                package.install_action = status.action.into();
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
                package.install_action = status.action.into();
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
                package.install_action = status.action.into();
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

pub async fn update_all_skill_extensions(
    home: &Path,
    state: &NativeState,
) -> Result<super::model::ExtensionInstallResult, ClientError> {
    let catalog = list_extensions(home, state, ExtensionKind::Skill).await?;
    let clients = installed_agent_clients();
    let target_roots = agent_skill_target_roots(&clients, home);
    let targets = skills::outdated_skill_package_targets(&target_roots, &catalog.packages)?;
    if targets.is_empty() {
        return Ok(super::model::ExtensionInstallResult {
            message: "没有可更新扩展。".to_string(),
        });
    }

    let client = authenticated_api(state).await?;
    for (name, target_roots) in &targets {
        let package = catalog
            .packages
            .iter()
            .find(|package| package.name == *name)
            .expect("outdated package exists in the catalog");
        let bundle: ExtensionInstallBundle = client
            .get(&format!(
                "/api/extensions/{}/versions/{}/install",
                package.name, package.version
            ))
            .await?;
        validate_package_bundle(package, &bundle)?;
        for target_root in target_roots {
            skills::install_skill_files(
                target_root,
                &bundle.name,
                &bundle.version.tag,
                &bundle.version.commit_sha,
                &bundle.files,
                false,
            )?;
        }
    }
    Ok(super::model::ExtensionInstallResult {
        message: format!("已更新 {} 个扩展。", targets.len()),
    })
}

impl From<rules::RuleInstallAction> for ExtensionInstallAction {
    fn from(action: rules::RuleInstallAction) -> Self {
        match action {
            rules::RuleInstallAction::Install => Self::Install,
            rules::RuleInstallAction::Partial => Self::Partial,
            rules::RuleInstallAction::Update => Self::Update,
            rules::RuleInstallAction::Installed => Self::Installed,
        }
    }
}

impl From<skills::SkillInstallAction> for ExtensionInstallAction {
    fn from(action: skills::SkillInstallAction) -> Self {
        match action {
            skills::SkillInstallAction::Install => Self::Install,
            skills::SkillInstallAction::Partial => Self::Partial,
            skills::SkillInstallAction::Update => Self::Update,
            skills::SkillInstallAction::Installed => Self::Installed,
        }
    }
}

impl From<mcp::McpInstallAction> for ExtensionInstallAction {
    fn from(action: mcp::McpInstallAction) -> Self {
        match action {
            mcp::McpInstallAction::Install => Self::Install,
            mcp::McpInstallAction::Partial => Self::Partial,
            mcp::McpInstallAction::Update => Self::Update,
            mcp::McpInstallAction::Installed => Self::Installed,
        }
    }
}

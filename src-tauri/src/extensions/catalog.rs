use std::path::Path;

use std::collections::BTreeSet;

use prelay_protocol::{
    CatalogKind, ExtensionArtifactSummary, ExtensionInstallBundle, ExtensionKind, ExtensionSummary,
};

use super::{
    hooks,
    install::{install_prepared_bundle, validate_package_bundle},
    mcp,
    model::{ExtensionCatalogSnapshot, ExtensionInstallAction, ExtensionPackage},
    rules, skills,
};
use crate::{
    agents::{
        agent_rule_targets_with_clients, agent_skill_target_roots, agent_skill_targets,
        installed_agent_clients, AgentClient,
    },
    identity::registration::authenticated_api,
    relay::client::ClientError,
    NativeState,
};

pub async fn list_extensions(
    home: &Path,
    state: &NativeState,
    catalog_kind: CatalogKind,
) -> Result<ExtensionCatalogSnapshot, ClientError> {
    let client = authenticated_api(state).await?;
    let summaries = load_summaries(&client, catalog_kind).await?;
    let mut packages = summaries
        .into_iter()
        .map(|summary| ExtensionPackage {
            name: summary.name,
            repository: summary.repository,
            commit_sha: summary.latest.commit_sha,
            version: summary.latest.tag,
            catalog_kind: summary.catalog_kind.unwrap_or(catalog_kind),
            artifacts: summary.artifacts,
            install_action: ExtensionInstallAction::Install,
            installed_clients: Vec::new(),
        })
        .collect::<Vec<_>>();
    let clients = installed_agent_clients();
    match catalog_kind {
        CatalogKind::Rule => {
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
        CatalogKind::Plugin => {
            let listed = packages
                .iter()
                .map(|package| package.name.clone())
                .collect();
            skills::retain_listed_skill_packages(
                &agent_skill_target_roots(&clients, home),
                &listed,
            )?;
            mcp::retain_listed_mcp_packages(home, &clients, &listed)?;
            hooks::retain_listed_hook_packages(home, &listed)?;
            for package in &mut packages {
                let status = package_installation_status(home, &clients, package)?;
                package.install_action = status.action;
                package.installed_clients = status.clients;
            }
        }
    }
    Ok(ExtensionCatalogSnapshot { packages })
}

async fn load_summaries(
    client: &crate::relay::client::ApiClient<'_>,
    catalog_kind: CatalogKind,
) -> Result<Vec<ExtensionSummary>, ClientError> {
    match catalog_kind {
        CatalogKind::Rule => client.get("/api/extensions/rules").await,
        CatalogKind::Plugin => match client.get("/api/extensions/plugins").await {
            Ok(summaries) => Ok(summaries),
            Err(error) if error.code() == "not_found" => {
                // 旧服务端没有 /plugins；把两个过渡接口合并为单制品插件。
                let mut summaries: Vec<ExtensionSummary> =
                    client.get("/api/extensions/skills").await?;
                let mcp: Vec<ExtensionSummary> = client.get("/api/extensions/mcp").await?;
                for summary in &mut summaries {
                    summary.catalog_kind = Some(CatalogKind::Plugin);
                    summary.artifacts = vec![ExtensionArtifactSummary {
                        kind: ExtensionKind::Skill,
                        file_count: 0,
                    }];
                }
                for mut summary in mcp {
                    summary.catalog_kind = Some(CatalogKind::Plugin);
                    summary.artifacts = vec![ExtensionArtifactSummary {
                        kind: ExtensionKind::Mcp,
                        file_count: 0,
                    }];
                    summaries.push(summary);
                }
                Ok(summaries)
            }
            Err(error) => Err(error),
        },
    }
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
    catalog_kind: CatalogKind,
) -> Result<super::model::ExtensionInstallResult, ClientError> {
    let mut catalog = list_extensions(home, state, catalog_kind).await?;
    catalog
        .packages
        .retain(|package| package.install_action == ExtensionInstallAction::Update);
    if catalog.packages.is_empty() {
        return Ok(super::model::ExtensionInstallResult {
            message: "没有可更新扩展。".to_string(),
            skipped: Vec::new(),
            review: Vec::new(),
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
        install_prepared_bundle(home, &bundle, &package.installed_clients, false)?;
    }
    Ok(super::model::ExtensionInstallResult {
        message: format!("已更新 {updated} 个扩展。"),
        skipped: Vec::new(),
        review: Vec::new(),
    })
}

struct PackageInstallationStatus {
    action: ExtensionInstallAction,
    clients: Vec<AgentClient>,
}

fn package_installation_status(
    home: &Path,
    clients: &[AgentClient],
    package: &ExtensionPackage,
) -> Result<PackageInstallationStatus, ClientError> {
    let artifact_kinds = effective_artifact_kinds(package);
    if artifact_kinds.is_empty() {
        return Ok(PackageInstallationStatus {
            action: ExtensionInstallAction::Install,
            clients: Vec::new(),
        });
    }
    let mut statuses = Vec::new();
    for kind in artifact_kinds {
        let status = match kind {
            prelay_protocol::ExtensionKind::Rule => {
                let targets = agent_rule_targets_with_clients(clients, home);
                rules::rule_installation_status(
                    &targets,
                    &package.name,
                    &package.version,
                    &package.commit_sha,
                )
                .map(|status| (status.action, status.clients))
            }
            prelay_protocol::ExtensionKind::Skill => {
                let targets = agent_skill_targets(clients, home);
                skills::skill_installation_status(
                    &targets,
                    &package.name,
                    &package.version,
                    &package.commit_sha,
                )
                .map(|status| (status.action, status.clients))
            }
            prelay_protocol::ExtensionKind::Mcp => mcp::mcp_installation_status(
                home,
                clients,
                &package.name,
                &package.version,
                &package.commit_sha,
            )
            .map(|status| (status.action, status.clients)),
            prelay_protocol::ExtensionKind::Hook => hooks::hook_installation_status(
                home,
                clients,
                &package.name,
                &package.version,
                &package.commit_sha,
            )
            .map(|status| (status.action, status.clients)),
            prelay_protocol::ExtensionKind::Unknown => continue,
        }?;
        statuses.push((kind, status.0, status.1));
    }
    if statuses.is_empty() {
        return Ok(PackageInstallationStatus {
            action: ExtensionInstallAction::Install,
            clients: Vec::new(),
        });
    }

    let applicable = clients
        .iter()
        .copied()
        .filter(|client| {
            statuses
                .iter()
                .any(|(kind, _, _)| artifact_supports_client(*kind, *client))
        })
        .collect::<Vec<_>>();
    let installed = applicable
        .iter()
        .copied()
        .filter(|client| {
            statuses.iter().all(|(kind, _, installed_clients)| {
                !artifact_supports_client(*kind, *client) || installed_clients.contains(client)
            })
        })
        .collect::<BTreeSet<_>>();
    let outdated = statuses
        .iter()
        .any(|(_, action, _)| *action == ExtensionInstallAction::Update);
    let partial = statuses
        .iter()
        .any(|(_, action, _)| *action == ExtensionInstallAction::Partial)
        || (!installed.is_empty() && installed.len() < applicable.len());
    let action = if outdated {
        ExtensionInstallAction::Update
    } else if applicable.is_empty() || installed.is_empty() {
        ExtensionInstallAction::Install
    } else if partial {
        ExtensionInstallAction::Partial
    } else {
        ExtensionInstallAction::Installed
    };
    Ok(PackageInstallationStatus {
        action,
        clients: installed.into_iter().collect(),
    })
}

fn effective_artifact_kinds(package: &ExtensionPackage) -> Vec<prelay_protocol::ExtensionKind> {
    if !package.artifacts.is_empty() {
        return package
            .artifacts
            .iter()
            .map(|artifact| artifact.kind)
            .collect();
    }
    if package.catalog_kind == CatalogKind::Rule {
        return vec![prelay_protocol::ExtensionKind::Rule];
    }
    Vec::new()
}

fn artifact_supports_client(kind: prelay_protocol::ExtensionKind, client: AgentClient) -> bool {
    match kind {
        prelay_protocol::ExtensionKind::Hook => client == AgentClient::CodexCli,
        prelay_protocol::ExtensionKind::Rule
        | prelay_protocol::ExtensionKind::Skill
        | prelay_protocol::ExtensionKind::Mcp => true,
        prelay_protocol::ExtensionKind::Unknown => false,
    }
}

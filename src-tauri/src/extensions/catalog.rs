use std::path::Path;

use prelay_protocol::{
    CatalogKind, ExtensionArtifactSummary, ExtensionInstallBundle, ExtensionKind, ExtensionSummary,
};

use super::{
    host_state::{ArtifactHealth, ExtensionHost, HostState, PluginArtifact, PluginState},
    install::{install_prepared_bundle, validate_package_bundle},
    mcp,
    model::{ExtensionCatalogSnapshot, ExtensionInstallAction, ExtensionPackage},
    rules, skills,
};
use crate::{
    agents::{agent_rule_targets_with_clients, AgentClient},
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
    let clients = crate::agents::installed_agent_clients();
    let host = ExtensionHost::from_clients(&clients).unwrap_or(ExtensionHost::Codex);
    let host_state = HostState::load(home, host)?;
    for package in &mut packages {
        let status = package_installation_status(home, &host_state, &clients, package)?;
        package.install_action = status.0;
        package.installed_clients = status.1;
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

fn package_installation_status(
    home: &Path,
    state: &HostState,
    clients: &[AgentClient],
    package: &ExtensionPackage,
) -> Result<(ExtensionInstallAction, Vec<AgentClient>), ClientError> {
    if package.catalog_kind == CatalogKind::Rule {
        return rule_status(home, state, clients, package);
    }
    let Some(record) = state.plugin(&package.name) else {
        return Ok((ExtensionInstallAction::Install, Vec::new()));
    };
    if record.state != PluginState::Installed {
        return Ok((ExtensionInstallAction::Partial, clients.to_vec()));
    }
    if record.version != package.version || record.commit_sha != package.commit_sha {
        return Ok((ExtensionInstallAction::Update, clients.to_vec()));
    }
    let mut health = ArtifactHealth::Managed;
    for artifact in &record.artifacts {
        let current = match artifact {
            PluginArtifact::Skill { target, names } => skills::health(Path::new(target), names),
            PluginArtifact::Mcp {
                server_name,
                manifest_hash,
                ..
            } => mcp::health(home, server_name, manifest_hash)?,
            PluginArtifact::Hook {
                event,
                matcher,
                handler_hash,
                ..
            } => super::hooks::health(home, event, matcher.as_deref(), handler_hash)?,
        };
        if current != ArtifactHealth::Managed {
            health = current;
            break;
        }
    }
    Ok((
        match health {
            ArtifactHealth::Managed => ExtensionInstallAction::Installed,
            ArtifactHealth::Missing | ArtifactHealth::Modified => ExtensionInstallAction::Partial,
        },
        clients.to_vec(),
    ))
}

fn rule_status(
    home: &Path,
    state: &HostState,
    clients: &[AgentClient],
    package: &ExtensionPackage,
) -> Result<(ExtensionInstallAction, Vec<AgentClient>), ClientError> {
    let targets = agent_rule_targets_with_clients(clients, home);
    let mut installed = Vec::new();
    let mut action = ExtensionInstallAction::Install;
    for (client, target) in targets {
        let Some(entry) = state.rule_entry(&target) else {
            continue;
        };
        if entry.state != PluginState::Installed {
            action = ExtensionInstallAction::Partial;
            continue;
        }
        if entry.package != package.name {
            action = ExtensionInstallAction::Partial;
            continue;
        }
        if rules::health(&target, entry) != ArtifactHealth::Managed {
            action = ExtensionInstallAction::Partial;
            continue;
        }
        if entry.version != package.version || entry.commit_sha != package.commit_sha {
            action = ExtensionInstallAction::Update;
            installed.push(client);
            continue;
        }
        installed.push(client);
    }
    if !installed.is_empty() {
        action = ExtensionInstallAction::Installed;
    }
    Ok((action, installed))
}

use std::{collections::BTreeMap, path::Path};

use prelay_protocol::{CatalogKind, ExtensionInstallBundle, ExtensionPolicy, ExtensionSummary};

use super::{
    catalog::load_summaries,
    hooks,
    host_state::{ArtifactHealth, ExtensionHost, HostState, PluginArtifact, PluginState},
    install::{install_prepared_bundle, validate_package_bundle},
    mcp,
    model::ExtensionPackage,
    skills,
    uninstall::uninstall_plugin,
};
use crate::{
    agents::{installed_agent_clients, AgentClient},
    identity::registration::authenticated_api,
    relay::client::ClientError,
    NativeState,
};

#[derive(Debug, Default, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyOutcome {
    pub uninstalled: Vec<String>,
    pub installed: Vec<String>,
    pub skipped: Vec<PolicySkip>,
    pub failed: Vec<PolicyFailure>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicySkip {
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyFailure {
    pub name: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Default)]
pub(crate) struct PolicyPlan {
    pub uninstall: Vec<String>,
    pub install: Vec<ExtensionPackage>,
    pub skip: Vec<PolicySkip>,
}

static SYNC_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub(crate) fn plan(
    home: &Path,
    host: ExtensionHost,
    packages: &[ExtensionPackage],
    clients: &[AgentClient],
) -> Result<PolicyPlan, ClientError> {
    let state = HostState::load(home, host)?;
    let mut plan = PolicyPlan::default();
    for package in packages {
        match package.policy {
            ExtensionPolicy::Normal => {}
            ExtensionPolicy::Deprecated => {
                if state.plugin(&package.name).is_some() {
                    plan.uninstall.push(package.name.clone());
                } else {
                    plan.skip.push(PolicySkip {
                        name: package.name.clone(),
                        reason: "not-installed".to_string(),
                    });
                }
            }
            ExtensionPolicy::Required => {
                if clients.is_empty() {
                    plan.skip.push(PolicySkip {
                        name: package.name.clone(),
                        reason: "no-client".to_string(),
                    });
                    continue;
                }
                if required_needs_install(home, &state, package)? {
                    plan.install.push(package.clone());
                } else {
                    plan.skip.push(PolicySkip {
                        name: package.name.clone(),
                        reason: "up-to-date".to_string(),
                    });
                }
            }
        }
    }
    plan.uninstall.sort();
    plan.install
        .sort_by(|left, right| left.name.cmp(&right.name));
    Ok(plan)
}

fn required_needs_install(
    home: &Path,
    state: &HostState,
    package: &ExtensionPackage,
) -> Result<bool, ClientError> {
    let Some(record) = state.plugin(&package.name) else {
        return Ok(true);
    };
    if record.state != PluginState::Installed
        || record.version != package.version
        || record.commit_sha != package.commit_sha
    {
        return Ok(true);
    }
    for artifact in &record.artifacts {
        let present = match artifact {
            PluginArtifact::Skill { target, names } => {
                skills::health(Path::new(target), names) != ArtifactHealth::Missing
            }
            PluginArtifact::Mcp { server_name, .. } => mcp::server_exists(home, server_name)?,
            PluginArtifact::Hook { handler_hash, .. } => hooks::handlers(home)?
                .iter()
                .any(|handler| &handler.hash == handler_hash),
        };
        if !present {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(crate) fn execute(
    home: &Path,
    host: ExtensionHost,
    plan: &PolicyPlan,
    clients: &[AgentClient],
    bundles: &BTreeMap<String, ExtensionInstallBundle>,
) -> PolicyOutcome {
    let mut outcome = PolicyOutcome::default();
    for name in &plan.uninstall {
        match uninstall_plugin(home, host, name) {
            Ok(()) => outcome.uninstalled.push(name.clone()),
            Err(error) => outcome.failed.push(failure(name, &error)),
        }
    }
    for package in &plan.install {
        let Some(bundle) = bundles.get(&package.name) else {
            outcome.failed.push(PolicyFailure {
                name: package.name.clone(),
                code: "extension_bundle_missing".to_string(),
                message: "安装包未下载。".to_string(),
            });
            continue;
        };
        if let Err(error) = validate_package_bundle(package, bundle) {
            outcome.failed.push(failure(&package.name, &error));
            continue;
        }
        match install_prepared_bundle(home, bundle, clients, false) {
            Ok(_) => outcome.installed.push(package.name.clone()),
            Err(error) => outcome.failed.push(failure(&package.name, &error)),
        }
    }
    outcome.skipped = plan.skip.clone();
    outcome
}

fn failure(name: &str, error: &ClientError) -> PolicyFailure {
    PolicyFailure {
        name: name.to_string(),
        code: error.code().to_string(),
        message: error.message.clone(),
    }
}

pub async fn sync(home: &Path, state: &NativeState) -> Result<PolicyOutcome, ClientError> {
    let _guard = SYNC_LOCK.lock().await;
    let clients = installed_agent_clients();
    let host = ExtensionHost::from_clients(&clients).unwrap_or(ExtensionHost::Codex);
    let summaries: Vec<ExtensionSummary> = {
        let client = authenticated_api(state).await?;
        load_summaries(&client, CatalogKind::Plugin).await?
    };
    let packages = summaries
        .iter()
        .map(|summary| ExtensionPackage {
            name: summary.name.clone(),
            repository: summary.repository.clone(),
            commit_sha: summary.latest.commit_sha.clone(),
            version: summary.latest.tag.clone(),
            catalog_kind: summary.catalog_kind.unwrap_or(CatalogKind::Plugin),
            artifacts: summary.artifacts.clone(),
            install_action: Default::default(),
            installed_clients: Vec::new(),
            policy: summary.policy,
        })
        .collect::<Vec<_>>();
    let mut planned = plan(home, host, &packages, &clients)?;
    let mut bundles = BTreeMap::new();
    let mut failures = Vec::new();
    for package in &planned.install {
        match fetch_bundle(state, package).await {
            Ok(bundle) => {
                bundles.insert(package.name.clone(), bundle);
            }
            Err(error) => failures.push(failure(&package.name, &error)),
        }
    }
    planned
        .install
        .retain(|package| bundles.contains_key(&package.name));
    let mut outcome = execute(home, host, &planned, &clients, &bundles);
    outcome.failed.extend(failures);
    Ok(outcome)
}

async fn fetch_bundle(
    state: &NativeState,
    package: &ExtensionPackage,
) -> Result<ExtensionInstallBundle, ClientError> {
    let client = authenticated_api(state).await?;
    client
        .get(&format!(
            "/api/extensions/{}/versions/{}/install",
            package.name, package.version
        ))
        .await
}

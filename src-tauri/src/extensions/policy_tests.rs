use std::collections::BTreeMap;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use prelay_protocol::{
    CatalogKind, ExtensionArtifact, ExtensionFile, ExtensionInstallBundle, ExtensionKind,
    ExtensionPolicy, ExtensionVersion,
};
use tempfile::tempdir;

use crate::agents::AgentClient;

use super::{
    host_state::{ExtensionHost, HostState},
    install::install_prepared_bundle,
    model::ExtensionPackage,
    policy::{execute, plan},
};

fn skill_bundle(name: &str, tag: &str, skill: &str) -> ExtensionInstallBundle {
    ExtensionInstallBundle {
        name: name.to_string(),
        version: ExtensionVersion {
            tag: tag.to_string(),
            commit_sha: "abc".to_string(),
            updated_at: "2026-11-27T00:00:00Z".to_string(),
        },
        artifacts: vec![ExtensionArtifact {
            kind: ExtensionKind::Skill,
            files: vec![ExtensionFile {
                path: format!("skills/{skill}/SKILL.md"),
                content_base64: BASE64.encode("# Managed\n"),
            }],
        }],
    }
}

fn package(name: &str, tag: &str, policy: ExtensionPolicy) -> ExtensionPackage {
    ExtensionPackage {
        name: name.to_string(),
        repository: format!("https://example.test/agents/{name}"),
        commit_sha: "abc".to_string(),
        version: tag.to_string(),
        catalog_kind: CatalogKind::Plugin,
        artifacts: Vec::new(),
        install_action: Default::default(),
        installed_clients: Vec::new(),
        policy,
    }
}

#[test]
fn uninstalls_a_deprecated_package_before_installing() {
    let directory = tempdir().unwrap();
    let home = directory.path();
    let clients = [AgentClient::CodexCli];
    install_prepared_bundle(
        home,
        &skill_bundle("old", "v1.0.0", "shared"),
        &clients,
        false,
    )
    .unwrap();
    let packages = vec![package("old", "v1.0.0", ExtensionPolicy::Deprecated)];

    let planned = plan(home, ExtensionHost::Codex, &packages, &clients).unwrap();
    assert_eq!(planned.uninstall, vec!["old".to_string()]);

    let outcome = execute(
        home,
        ExtensionHost::Codex,
        &planned,
        &clients,
        &BTreeMap::new(),
    );
    assert_eq!(outcome.uninstalled, vec!["old".to_string()]);
    assert!(HostState::load(home, ExtensionHost::Codex)
        .unwrap()
        .plugin("old")
        .is_none());
}

#[test]
fn installs_missing_and_skips_up_to_date_required_packages() {
    let directory = tempdir().unwrap();
    let home = directory.path();
    let clients = [AgentClient::CodexCli];
    let packages = vec![package("must", "v1.0.0", ExtensionPolicy::Required)];
    let bundles = BTreeMap::from([("must".to_string(), skill_bundle("must", "v1.0.0", "must"))]);

    let planned = plan(home, ExtensionHost::Codex, &packages, &clients).unwrap();
    assert_eq!(planned.install.len(), 1);
    let outcome = execute(home, ExtensionHost::Codex, &planned, &clients, &bundles);
    assert_eq!(outcome.installed, vec!["must".to_string()]);

    let planned = plan(home, ExtensionHost::Codex, &packages, &clients).unwrap();
    assert!(planned.install.is_empty());
    assert_eq!(planned.skip[0].name, "must");
    assert_eq!(planned.skip[0].reason, "up-to-date");
}

#[test]
fn repairs_a_required_package_whose_artifact_was_removed() {
    let directory = tempdir().unwrap();
    let home = directory.path();
    let clients = [AgentClient::CodexCli];
    let packages = vec![package("must", "v1.0.0", ExtensionPolicy::Required)];
    let bundles = BTreeMap::from([("must".to_string(), skill_bundle("must", "v1.0.0", "must"))]);
    let planned = plan(home, ExtensionHost::Codex, &packages, &clients).unwrap();
    execute(home, ExtensionHost::Codex, &planned, &clients, &bundles);
    std::fs::remove_dir_all(home.join(".agents").join("skills").join("must")).unwrap();

    let planned = plan(home, ExtensionHost::Codex, &packages, &clients).unwrap();
    assert_eq!(planned.install.len(), 1, "缺失制品应触发补装");
}

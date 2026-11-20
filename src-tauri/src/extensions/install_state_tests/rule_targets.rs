use std::fs;

use prelay_protocol::{ExtensionArtifact, ExtensionKind};
use tempfile::tempdir;

use crate::agents::AgentClient;

use super::super::{
    host_state::{ExtensionHost, HostState, PluginState},
    install::install_prepared_bundle,
};
use super::{bundle, file};

#[test]
fn refuses_to_replace_an_unmanaged_rule_without_overwrite() {
    let directory = tempdir().unwrap();
    let home = directory.path();
    let target = home.join(".codex").join("AGENTS.md");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, "# Personal\n").unwrap();
    let rule = bundle(
        "development-rules",
        vec![ExtensionArtifact {
            kind: ExtensionKind::Rule,
            files: vec![file("AGENTS.md", "# Managed\n")],
        }],
    );

    let error = install_prepared_bundle(home, &rule, &[AgentClient::CodexCli], false).unwrap_err();
    assert_eq!(error.code(), "extension_target_exists");
    assert_eq!(fs::read_to_string(&target).unwrap(), "# Personal\n");

    install_prepared_bundle(home, &rule, &[AgentClient::CodexCli], true).unwrap();
    assert_eq!(fs::read_to_string(&target).unwrap(), "# Managed\n");
}

#[test]
fn replaces_another_rule_package_only_with_overwrite() {
    let directory = tempdir().unwrap();
    let home = directory.path();
    let target = home.join(".codex").join("AGENTS.md");
    let first = bundle(
        "development-rules",
        vec![ExtensionArtifact {
            kind: ExtensionKind::Rule,
            files: vec![file("AGENTS.md", "# First\n")],
        }],
    );
    let second = bundle(
        "personal-rules",
        vec![ExtensionArtifact {
            kind: ExtensionKind::Rule,
            files: vec![file("AGENTS.md", "# Second\n")],
        }],
    );
    install_prepared_bundle(home, &first, &[AgentClient::CodexCli], false).unwrap();

    let error =
        install_prepared_bundle(home, &second, &[AgentClient::CodexCli], false).unwrap_err();
    assert_eq!(error.code(), "extension_target_exists");
    assert_eq!(fs::read_to_string(&target).unwrap(), "# First\n");

    install_prepared_bundle(home, &second, &[AgentClient::CodexCli], true).unwrap();
    assert_eq!(fs::read_to_string(&target).unwrap(), "# Second\n");
    let state = HostState::load(home, ExtensionHost::Codex).unwrap();
    let entry = state.rule_entry(&target).expect("规则记录应指向当前规则包");
    assert_eq!(entry.package, "personal-rules");
    assert_eq!(entry.state, PluginState::Installed);
}

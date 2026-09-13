use std::fs;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use prelay_protocol::ExtensionFile;
use tempfile::tempdir;

use super::super::{ExtensionInstallAction, ExtensionKind, ExtensionPackage};
use super::{install_skill_files, outdated_skill_package_targets, skill_installation_status};
use crate::agents::AgentClient;

fn skill_file(path: &str, content: &str) -> ExtensionFile {
    ExtensionFile {
        path: path.to_string(),
        content_base64: BASE64.encode(content),
    }
}

#[test]
fn reinstalling_a_skill_package_keeps_removed_skill_directories() {
    let directory = tempdir().unwrap();
    let root = directory.path().join("skills");

    install_skill_files(
        &root,
        "engineering",
        "1.0.0",
        "commit",
        &[
            skill_file("skills/check/SKILL.md", "old"),
            skill_file("skills/retired/SKILL.md", "retired"),
        ],
        false,
    )
    .unwrap();
    fs::write(root.join("check").join("stale.md"), "stale").unwrap();

    install_skill_files(
        &root,
        "engineering",
        "1.1.0",
        "commit2",
        &[skill_file("skills/check/SKILL.md", "new")],
        false,
    )
    .unwrap();

    assert_eq!(
        fs::read_to_string(root.join("check").join("SKILL.md")).unwrap(),
        "new"
    );
    assert!(!root.join("check").join("stale.md").exists());
    assert!(root.join("retired").exists());
}

#[test]
fn migrating_legacy_skill_records_writes_a_single_state_file_in_the_skill_root() {
    let directory = tempdir().unwrap();
    let root = directory.path().join("skills");
    fs::create_dir_all(directory.path().join(".prelay").join("skills")).unwrap();
    fs::write(
        directory
            .path()
            .join(".prelay")
            .join("skills")
            .join("656e67696e656572696e67.json"),
        r#"{
  "package": "engineering",
  "version": "v1.0.0",
  "commitSha": "legacy",
  "roots": ["check"]
}"#,
    )
    .unwrap();

    install_skill_files(
        &root,
        "review",
        "v1.1.0",
        "current",
        &[skill_file("skills/review/SKILL.md", "review")],
        false,
    )
    .unwrap();

    let state: serde_json::Value = serde_json::from_slice(
        &fs::read(directory.path().join(".prelay").join("skill.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(state["engineering"]["version"], "v1.0.0");
    assert_eq!(state["engineering"]["skills"], serde_json::json!(["check"]));
    assert_eq!(state["review"]["commitSha"], "current");
    assert!(!directory.path().join(".prelay").join("skills").exists());
}

#[test]
fn installing_a_skill_does_not_replace_another_packages_directory() {
    let directory = tempdir().unwrap();
    let root = directory.path().join("skills");
    let files = [skill_file("skills/shared/SKILL.md", "first")];

    install_skill_files(&root, "first-package", "1.0.0", "commit", &files, false).unwrap();
    let result = install_skill_files(&root, "second-package", "1.0.0", "commit", &files, false);

    assert!(result.is_err());
    assert_eq!(
        fs::read_to_string(root.join("shared").join("SKILL.md")).unwrap(),
        "first"
    );
}

#[test]
fn installing_a_skill_does_not_replace_an_unmanaged_directory() {
    let directory = tempdir().unwrap();
    let root = directory.path().join("skills");
    fs::create_dir_all(root.join("manual")).unwrap();
    fs::write(root.join("manual").join("SKILL.md"), "manual").unwrap();

    let result = install_skill_files(
        &root,
        "managed-package",
        "1.0.0",
        "commit",
        &[skill_file("skills/manual/SKILL.md", "managed")],
        false,
    );

    assert!(result.is_err());
    assert_eq!(
        fs::read_to_string(root.join("manual").join("SKILL.md")).unwrap(),
        "manual"
    );
}

#[test]
fn overwrite_replaces_an_unmanaged_skill_directory_after_confirmation() {
    let directory = tempdir().unwrap();
    let root = directory.path().join("skills");
    fs::create_dir_all(root.join("manual")).unwrap();
    fs::write(root.join("manual").join("SKILL.md"), "manual").unwrap();

    install_skill_files(
        &root,
        "managed-package",
        "1.0.0",
        "commit",
        &[skill_file("skills/manual/SKILL.md", "managed")],
        true,
    )
    .unwrap();

    assert_eq!(
        fs::read_to_string(root.join("manual").join("SKILL.md")).unwrap(),
        "managed"
    );
}

#[test]
fn overwrite_transfers_managed_skill_directory_to_the_new_package() {
    let directory = tempdir().unwrap();
    let root = directory.path().join("skills");
    let files = [skill_file("skills/shared/SKILL.md", "first")];

    install_skill_files(&root, "first-package", "1.0.0", "commit", &files, false).unwrap();
    install_skill_files(
        &root,
        "second-package",
        "1.0.0",
        "commit",
        &[skill_file("skills/shared/SKILL.md", "second")],
        true,
    )
    .unwrap();
    install_skill_files(
        &root,
        "second-package",
        "1.1.0",
        "commit2",
        &[skill_file("skills/shared/SKILL.md", "third")],
        false,
    )
    .unwrap();

    assert_eq!(
        fs::read_to_string(root.join("shared").join("SKILL.md")).unwrap(),
        "third"
    );
}

#[test]
fn finds_outdated_packages_only_in_the_skill_roots_that_record_them() {
    let directory = tempdir().unwrap();
    let codex_root = directory.path().join("agents").join("skills");
    let opencode_root = directory.path().join("opencode").join("skills");
    let files = [skill_file("skills/review/SKILL.md", "old")];

    install_skill_files(&codex_root, "engineering", "v1.0.0", "old", &files, false).unwrap();
    install_skill_files(
        &opencode_root,
        "engineering",
        "v1.1.0",
        "current",
        &files,
        false,
    )
    .unwrap();

    let packages = [ExtensionPackage {
        name: "engineering".to_string(),
        repository: "https://git.example.test/engineering".to_string(),
        commit_sha: "current".to_string(),
        version: "v1.1.0".to_string(),
        kind: ExtensionKind::Skill,
        install_action: ExtensionInstallAction::Update,
        installed_clients: Vec::new(),
    }];
    let targets =
        outdated_skill_package_targets(&[codex_root.clone(), opencode_root], &packages).unwrap();

    assert_eq!(targets["engineering"], vec![codex_root]);
}

#[test]
fn reports_partial_installation_for_detected_clients_missing_a_skill_root() {
    let directory = tempdir().unwrap();
    let shared_root = directory.path().join("agents").join("skills");
    let opencode_root = directory.path().join("opencode").join("skills");
    let files = [skill_file("skills/review/SKILL.md", "current")];

    install_skill_files(
        &shared_root,
        "engineering",
        "v1.1.0",
        "current",
        &files,
        false,
    )
    .unwrap();

    let status = skill_installation_status(
        &[
            (AgentClient::CodexCli, shared_root.clone()),
            (AgentClient::ChatGpt, shared_root),
            (AgentClient::OpenCode, opencode_root),
        ],
        "engineering",
        "v1.1.0",
        "current",
    )
    .unwrap();

    assert_eq!(status.action, ExtensionInstallAction::Partial);
    assert_eq!(
        status.clients,
        vec![AgentClient::CodexCli, AgentClient::ChatGpt]
    );
}

use std::fs;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use prelay_protocol::ExtensionFile;
use tempfile::tempdir;

use super::super::ExtensionInstallAction;
use super::{install_skill_files, skill_installation_status, uninstall_skill_package};
use crate::agents::AgentClient;

fn skill_file(path: &str, content: &str) -> ExtensionFile {
    ExtensionFile {
        path: path.to_string(),
        content_base64: BASE64.encode(content),
    }
}

fn read_state(target_root: &std::path::Path) -> serde_json::Value {
    let path = target_root
        .parent()
        .unwrap()
        .join(".prelay")
        .join("skill.json");
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
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

#[test]
fn uninstalling_a_skill_package_removes_its_skills_and_state() {
    let directory = tempdir().unwrap();
    let root = directory.path().join("skills");
    install_skill_files(
        &root,
        "engineering",
        "v1.0.0",
        "commit",
        &[
            skill_file("skills/check/SKILL.md", "check"),
            skill_file("skills/review/SKILL.md", "review"),
        ],
        false,
    )
    .unwrap();
    install_skill_files(
        &root,
        "other",
        "v1.0.0",
        "commit",
        &[skill_file("skills/keep/SKILL.md", "keep")],
        false,
    )
    .unwrap();

    uninstall_skill_package(&root, "engineering").unwrap();

    assert!(!root.join("check").exists());
    assert!(!root.join("review").exists());
    assert!(root.join("keep").exists());
    let state = read_state(&root);
    assert!(state.get("engineering").is_none());
    assert_eq!(state["other"]["skills"], serde_json::json!(["keep"]));
}

#[test]
fn reports_install_and_drops_the_record_when_the_managed_skills_are_gone() {
    let directory = tempdir().unwrap();
    let root = directory.path().join("skills");
    install_skill_files(
        &root,
        "engineering",
        "v1.0.0",
        "commit",
        &[skill_file("skills/check/SKILL.md", "check")],
        false,
    )
    .unwrap();
    fs::remove_dir_all(root.join("check")).unwrap();

    let status = skill_installation_status(
        &[(AgentClient::CodexCli, root.clone())],
        "engineering",
        "v1.0.0",
        "commit",
    )
    .unwrap();

    assert_eq!(status.action, ExtensionInstallAction::Install);
    assert!(status.clients.is_empty());
    assert!(read_state(&root).get("engineering").is_none());
}

#[test]
fn reports_partial_and_keeps_the_record_when_one_managed_skill_is_gone() {
    let directory = tempdir().unwrap();
    let root = directory.path().join("skills");
    install_skill_files(
        &root,
        "engineering",
        "v1.0.0",
        "commit",
        &[
            skill_file("skills/check/SKILL.md", "check"),
            skill_file("skills/review/SKILL.md", "review"),
        ],
        false,
    )
    .unwrap();
    fs::remove_dir_all(root.join("review")).unwrap();

    let status = skill_installation_status(
        &[(AgentClient::CodexCli, root.clone())],
        "engineering",
        "v1.0.0",
        "commit",
    )
    .unwrap();

    assert_eq!(status.action, ExtensionInstallAction::Partial);
    assert_eq!(status.clients, vec![AgentClient::CodexCli]);
    assert_eq!(
        read_state(&root)["engineering"]["skills"],
        serde_json::json!(["check", "review"])
    );
}

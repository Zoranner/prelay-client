use std::fs;

use serde_json::json;
use tempfile::tempdir;

use crate::agents::AgentClient;
use crate::extensions::ExtensionInstallAction;

use super::test_support::*;
use super::{hook_installation_status, install_hooks};

#[test]
fn installs_support_files_and_expands_the_managed_root() {
    let directory = tempdir().unwrap();
    let file = hook_file(json!({
        "hooks": {
            "SessionStart": [{
                "hooks": [{
                    "type": "command",
                    "command": "sh \"${PRELAY_HOOK_ROOT}/scripts/session-start.sh\" ${HOME}",
                    "commandWindows": "powershell -File \"${PRELAY_HOOK_ROOT}\\scripts\\session-start.ps1\""
                }]
            }]
        }
    }));
    let support = support_file("hooks/scripts/session-start.sh", "#!/bin/sh\n");

    install_hooks(
        directory.path(),
        &[AgentClient::CodexCli],
        "superpowers",
        "v1",
        "commit-1",
        &[file, support],
        false,
    )
    .unwrap();

    let root = hook_root(directory.path(), "commit-1");
    assert!(root.join("hooks.json").is_file());
    assert_eq!(
        fs::read_to_string(root.join("scripts").join("session-start.sh")).unwrap(),
        "#!/bin/sh\n"
    );
    let document = hooks_document(directory.path());
    let handler = &document["hooks"]["SessionStart"][0]["hooks"][0];
    assert_eq!(
        handler["command"],
        format!(
            "sh \"{}/scripts/session-start.sh\" ${{HOME}}",
            root.display()
        )
    );
    assert_eq!(
        handler["commandWindows"],
        format!(
            "powershell -File \"{}\\scripts\\session-start.ps1\"",
            root.display()
        )
    );
    assert!(!handler["command"]
        .as_str()
        .unwrap()
        .contains("${PRELAY_HOOK_ROOT}"));
}

#[test]
fn reports_partial_when_the_managed_root_is_missing() {
    let directory = tempdir().unwrap();
    let file = hook_file(json!({
        "hooks": {
            "SessionStart": [{
                "hooks": [{"type": "command", "command": "prelay-command"}]
            }]
        }
    }));
    install_hooks(
        directory.path(),
        &[AgentClient::CodexCli],
        "superpowers",
        "v1",
        "commit",
        &[file],
        false,
    )
    .unwrap();
    fs::remove_dir_all(
        directory
            .path()
            .join(".prelay")
            .join("packages")
            .join("superpowers")
            .join("commit"),
    )
    .unwrap();

    let status = hook_installation_status(
        directory.path(),
        &[AgentClient::CodexCli],
        "superpowers",
        "v1",
        "commit",
    )
    .unwrap();
    assert_eq!(status.action, ExtensionInstallAction::Partial);
}

#[test]
fn rejects_native_plugin_root_placeholders() {
    let directory = tempdir().unwrap();
    let file = hook_file(json!({
        "hooks": {
            "SessionStart": [{
                "hooks": [{
                    "type": "command",
                    "command": "${PLUGIN_ROOT}/scripts/session-start.sh"
                }]
            }]
        }
    }));

    let result = install_hooks(
        directory.path(),
        &[AgentClient::CodexCli],
        "superpowers",
        "v1",
        "commit",
        &[file],
        false,
    );
    assert!(result.is_err());
    assert!(!directory.path().join(".codex").join("hooks.json").exists());
}

#[test]
fn installs_a_package_with_only_the_definition_file() {
    let directory = tempdir().unwrap();
    let file = hook_file(json!({
        "hooks": {
            "SessionStart": [{
                "hooks": [{"type": "command", "command": "prelay-session"}]
            }]
        }
    }));

    let review = install_hooks(
        directory.path(),
        &[AgentClient::CodexCli],
        "superpowers",
        "v1",
        "commit",
        &[file],
        false,
    )
    .unwrap();

    assert_eq!(review.len(), 1);
    assert!(hook_root(directory.path(), "commit")
        .join("hooks.json")
        .is_file());
    assert_eq!(commands(directory.path()), ["prelay-session"]);
}

#[test]
fn installs_multiple_events_matchers_and_handlers() {
    let directory = tempdir().unwrap();
    let file = hook_file(json!({
        "hooks": {
            "SessionStart": [
                {
                    "matcher": "startup",
                    "hooks": [{"type": "command", "command": "start-one"}]
                },
                {
                    "matcher": "resume",
                    "hooks": [{"type": "command", "command": "start-two"}]
                }
            ],
            "Stop": [{
                "hooks": [
                    {"type": "command", "command": "stop-one"},
                    {"type": "command", "command": "stop-two"}
                ]
            }]
        }
    }));

    let review = install_hooks(
        directory.path(),
        &[AgentClient::CodexCli],
        "superpowers",
        "v1",
        "commit",
        &[file],
        false,
    )
    .unwrap();

    assert_eq!(review.len(), 4);
    let document = hooks_document(directory.path());
    assert_eq!(
        document["hooks"]["SessionStart"].as_array().unwrap().len(),
        2
    );
    assert_eq!(
        document["hooks"]["Stop"][0]["hooks"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(commands(directory.path()), ["start-one", "start-two"]);
}

#[test]
fn other_json_files_are_installed_as_support_files_without_parsing() {
    let directory = tempdir().unwrap();
    let file = hook_file(json!({
        "hooks": {
            "SessionStart": [{
                "hooks": [{"type": "command", "command": "prelay-session"}]
            }]
        }
    }));
    let other = binary_support_file("hooks/other.json", b"{not-json");

    let review = install_hooks(
        directory.path(),
        &[AgentClient::CodexCli],
        "superpowers",
        "v1",
        "commit",
        &[file, other],
        false,
    )
    .unwrap();

    assert_eq!(review.len(), 1);
    assert_eq!(
        fs::read(hook_root(directory.path(), "commit").join("other.json")).unwrap(),
        b"{not-json"
    );
    assert_eq!(commands(directory.path()), ["prelay-session"]);
}

#[test]
fn copies_binary_support_files_byte_for_byte() {
    let directory = tempdir().unwrap();
    let bytes = [0x00, 0x9f, 0x92, 0x96, 0xff];
    let file = hook_file(json!({
        "hooks": {
            "SessionStart": [{
                "hooks": [{"type": "command", "command": "prelay-session"}]
            }]
        }
    }));
    let binary = binary_support_file("hooks/lib/runner.bin", &bytes);

    install_hooks(
        directory.path(),
        &[AgentClient::CodexCli],
        "superpowers",
        "v1",
        "commit",
        &[file, binary],
        false,
    )
    .unwrap();

    assert_eq!(
        fs::read(
            hook_root(directory.path(), "commit")
                .join("lib")
                .join("runner.bin")
        )
        .unwrap(),
        bytes
    );
}

#[test]
fn skips_hook_installation_for_non_codex_clients() {
    let directory = tempdir().unwrap();
    let file = hook_file(json!({
        "hooks": {
            "SessionStart": [{
                "hooks": [{"type": "command", "command": "prelay-session"}]
            }]
        }
    }));

    let review = install_hooks(
        directory.path(),
        &[AgentClient::ChatGpt, AgentClient::OpenCode],
        "superpowers",
        "v1",
        "commit",
        &[file],
        false,
    )
    .unwrap();

    assert!(review.is_empty());
    assert!(!hooks_path(directory.path()).exists());
    assert!(!directory.path().join(".prelay").exists());
}

#[test]
fn rejects_missing_and_duplicate_definition_files() {
    let directory = tempdir().unwrap();
    let support = support_file("hooks/scripts/run.sh", "#!/bin/sh\n");
    assert!(install_hooks(
        directory.path(),
        &[AgentClient::CodexCli],
        "superpowers",
        "v1",
        "commit",
        &[support],
        false,
    )
    .is_err());
    assert!(!hooks_path(directory.path()).exists());

    let first = hook_file(json!({"hooks": {"SessionStart": [{"hooks": []}]}}));
    let second = hook_file(json!({"hooks": {"SessionStart": [{"hooks": []}]}}));
    assert!(install_hooks(
        directory.path(),
        &[AgentClient::CodexCli],
        "superpowers",
        "v1",
        "commit",
        &[first, second],
        false,
    )
    .is_err());
    assert!(!hooks_path(directory.path()).exists());
}

#[test]
fn rejects_empty_commands() {
    let directory = tempdir().unwrap();
    let empty_command = hook_file(json!({
        "hooks": {
            "SessionStart": [{
                "hooks": [{"type": "command", "command": "   "}]
            }]
        }
    }));
    assert!(install_hooks(
        directory.path(),
        &[AgentClient::CodexCli],
        "superpowers",
        "v1",
        "commit",
        &[empty_command],
        false,
    )
    .is_err());

    let empty_windows = hook_file(json!({
        "hooks": {
            "SessionStart": [{
                "hooks": [{
                    "type": "command",
                    "command": "prelay-session",
                    "commandWindows": " "
                }]
            }]
        }
    }));
    assert!(install_hooks(
        directory.path(),
        &[AgentClient::CodexCli],
        "superpowers",
        "v1",
        "commit",
        &[empty_windows],
        false,
    )
    .is_err());
}

#[test]
fn failed_support_install_leaves_the_host_config_untouched() {
    let directory = tempdir().unwrap();
    let package_root = directory
        .path()
        .join(".prelay")
        .join("packages")
        .join("superpowers");
    fs::create_dir_all(&package_root).unwrap();
    fs::write(package_root.join("commit"), "blocker").unwrap();
    let file = hook_file(json!({
        "hooks": {
            "SessionStart": [{
                "hooks": [{"type": "command", "command": "prelay-session"}]
            }]
        }
    }));

    assert!(install_hooks(
        directory.path(),
        &[AgentClient::CodexCli],
        "superpowers",
        "v1",
        "commit",
        &[file],
        false,
    )
    .is_err());
    assert!(!hooks_path(directory.path()).exists());
    assert_eq!(
        fs::read_to_string(package_root.join("commit")).unwrap(),
        "blocker"
    );
}

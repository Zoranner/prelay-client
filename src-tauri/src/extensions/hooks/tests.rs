use std::fs;

use serde_json::{json, Value};
use tempfile::tempdir;

use crate::agents::AgentClient;
use crate::extensions::ExtensionInstallAction;

use super::test_support::*;
use super::{hook_installation_status, install_hooks, uninstall_hooks};

#[test]
fn reports_installed_hook_for_both_codex_clients() {
    let directory = tempdir().unwrap();
    let file = hook_file(json!({
        "hooks": {
            "SessionStart": [{
                "hooks": [{"type": "command", "command": "prelay-session"}]
            }]
        }
    }));

    install_hooks(
        directory.path(),
        &[AgentClient::CodexCli, AgentClient::ChatGpt],
        "superpowers",
        "v1",
        "commit",
        &[file],
        false,
    )
    .unwrap();

    let status = hook_installation_status(
        directory.path(),
        &[AgentClient::CodexCli, AgentClient::ChatGpt],
        "superpowers",
        "v1",
        "commit",
    )
    .unwrap();

    assert_eq!(status.action, ExtensionInstallAction::Installed);
    assert_eq!(
        status.clients,
        vec![AgentClient::CodexCli, AgentClient::ChatGpt]
    );
}

#[test]
fn merges_user_hooks_and_replaces_the_same_handler() {
    let directory = tempdir().unwrap();
    let hooks_path = directory.path().join(".codex").join("hooks.json");
    fs::create_dir_all(hooks_path.parent().unwrap()).unwrap();
    fs::write(
        &hooks_path,
        serde_json::to_vec_pretty(&json!({
            "description": "user hooks",
            "hooks": {
                "SessionStart": [{
                    "matcher": "startup",
                    "hooks": [{
                        "type": "command",
                        "command": "user-command"
                    }]
                }]
            }
        }))
        .unwrap(),
    )
    .unwrap();
    let file = hook_file(json!({
        "hooks": {
            "SessionStart": [{
                "matcher": "startup|resume",
                "hooks": [{
                    "type": "command",
                    "command": "prelay-command",
                    "commandWindows": "prelay-command.exe"
                }]
            }]
        }
    }));

    let review = install_hooks(
        directory.path(),
        &[AgentClient::CodexCli, AgentClient::ChatGpt],
        "superpowers",
        "v1",
        "commit",
        std::slice::from_ref(&file),
        false,
    )
    .unwrap();
    assert_eq!(review.len(), 1);
    assert_eq!(review[0].event, "SessionStart");
    assert_eq!(
        commands(directory.path()),
        ["user-command", "prelay-command"]
    );
    assert_eq!(
        hooks_document(directory.path())["description"],
        "user hooks"
    );

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
    assert_eq!(
        commands(directory.path()),
        ["user-command", "prelay-command"]
    );
    assert!(hook_root(directory.path(), "commit").is_dir());
}

#[test]
fn updating_a_package_removes_its_old_handlers() {
    let directory = tempdir().unwrap();
    let first = hook_file(json!({
        "hooks": {
            "SessionStart": [{
                "hooks": [{"type": "command", "command": "first-command"}]
            }]
        }
    }));
    let second = hook_file(json!({
        "hooks": {
            "SessionStart": [{
                "hooks": [{"type": "command", "command": "second-command"}]
            }]
        }
    }));

    install_hooks(
        directory.path(),
        &[AgentClient::CodexCli],
        "superpowers",
        "v1",
        "commit-1",
        &[first],
        false,
    )
    .unwrap();
    install_hooks(
        directory.path(),
        &[AgentClient::CodexCli],
        "superpowers",
        "v2",
        "commit-2",
        &[second],
        false,
    )
    .unwrap();

    assert_eq!(commands(directory.path()), ["second-command"]);
    assert!(!directory
        .path()
        .join(".prelay")
        .join("packages")
        .join("superpowers")
        .join("commit-1")
        .exists());
    assert!(hook_root(directory.path(), "commit-2").is_dir());
}

#[test]
fn uninstall_removes_only_the_packages_handlers() {
    let directory = tempdir().unwrap();
    let hooks_path = directory.path().join(".codex").join("hooks.json");
    fs::create_dir_all(hooks_path.parent().unwrap()).unwrap();
    fs::write(
        &hooks_path,
        serde_json::to_vec(&json!({
            "hooks": {
                "SessionStart": [{
                    "hooks": [{"type": "command", "command": "user-command"}]
                }]
            }
        }))
        .unwrap(),
    )
    .unwrap();
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
    uninstall_hooks(directory.path(), "superpowers").unwrap();

    assert_eq!(commands(directory.path()), ["user-command"]);
    assert!(!hook_root(directory.path(), "commit").exists());
    let state = fs::read(
        directory
            .path()
            .join(".codex")
            .join(".prelay")
            .join("hooks.json"),
    )
    .unwrap();
    assert_eq!(serde_json::from_slice::<Value>(&state).unwrap(), json!({}));
}

#[test]
fn invalid_hooks_json_is_never_overwritten() {
    let directory = tempdir().unwrap();
    let hooks_path = directory.path().join(".codex").join("hooks.json");
    fs::create_dir_all(hooks_path.parent().unwrap()).unwrap();
    fs::write(&hooks_path, "{not-json").unwrap();
    let file = hook_file(json!({
        "hooks": {
            "SessionStart": [{
                "hooks": [{"type": "command", "command": "prelay-command"}]
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
    assert_eq!(fs::read_to_string(&hooks_path).unwrap(), "{not-json");
}

#[test]
fn rejects_unknown_hook_events_and_non_command_handlers() {
    let directory = tempdir().unwrap();
    let unknown = hook_file(json!({
        "hooks": {
            "UnknownEvent": [{
                "hooks": [{"type": "command", "command": "prelay-command"}]
            }]
        }
    }));
    assert!(install_hooks(
        directory.path(),
        &[AgentClient::CodexCli],
        "superpowers",
        "v1",
        "commit",
        &[unknown],
        false,
    )
    .is_err());

    let unsupported = hook_file(json!({
        "hooks": {
            "SessionStart": [{
                "hooks": [{"type": "mcp_tool", "server": "scanner", "tool": "scan"}]
            }]
        }
    }));
    assert!(install_hooks(
        directory.path(),
        &[AgentClient::CodexCli],
        "superpowers",
        "v1",
        "commit",
        &[unsupported],
        false,
    )
    .is_err());
}

#[test]
fn uninstall_reports_a_conflict_when_managed_handlers_were_edited() {
    let directory = tempdir().unwrap();
    let file = hook_file(json!({
        "hooks": {
            "SessionStart": [{
                "hooks": [{"type": "command", "command": "prelay-session"}]
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

    let mut document = hooks_document(directory.path());
    document["hooks"]["SessionStart"][0]["hooks"][0]["command"] =
        serde_json::Value::String("user-edited".to_string());
    fs::write(
        hooks_path(directory.path()),
        serde_json::to_vec_pretty(&document).unwrap(),
    )
    .unwrap();

    let error = uninstall_hooks(directory.path(), "superpowers").unwrap_err();
    assert_eq!(error.code(), "local_extensions_conflict");
    assert_eq!(commands(directory.path()), ["user-edited"]);
    assert!(hook_root(directory.path(), "commit").is_dir());
    let state: Value = serde_json::from_slice(
        &fs::read(
            directory
                .path()
                .join(".codex")
                .join(".prelay")
                .join("hooks.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(state.get("superpowers").is_some());
}

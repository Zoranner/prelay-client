use std::fs;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use prelay_protocol::ExtensionFile;
use serde_json::{json, Value};
use tempfile::tempdir;

use crate::agents::AgentClient;

use super::{install_hooks, uninstall_hooks};

fn hook_file(value: Value) -> ExtensionFile {
    ExtensionFile {
        path: "hooks/hooks.json".to_string(),
        content_base64: BASE64.encode(serde_json::to_vec(&value).unwrap()),
    }
}

fn hooks_document(home: &std::path::Path) -> Value {
    serde_json::from_slice(
        &fs::read(home.join(".codex").join("hooks.json")).expect("read hooks.json"),
    )
    .expect("parse hooks.json")
}

fn commands(home: &std::path::Path) -> Vec<String> {
    hooks_document(home)["hooks"]["SessionStart"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|group| group.get("hooks").and_then(Value::as_array))
        .flatten()
        .filter_map(|handler| handler.get("command").and_then(Value::as_str))
        .map(str::to_string)
        .collect()
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
        &[AgentClient::CodexCli, AgentClient::OpenCode],
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

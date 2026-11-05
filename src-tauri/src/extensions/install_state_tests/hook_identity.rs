use std::fs;

use serde_json::json;
use tempfile::tempdir;

use crate::agents::{AgentClient, AgentItemKind};

#[test]
fn unmanaged_hooks_use_event_and_handler_hash_without_fabricated_names() {
    let directory = tempdir().unwrap();
    let home = directory.path();
    let codex = home.join(".codex");
    fs::create_dir_all(&codex).unwrap();
    fs::write(
        codex.join("hooks.json"),
        json!({
            "hooks": {
                "PreToolUse": [
                    {
                        "matcher": "Bash",
                        "hooks": [{"type": "command", "command": "first-hook"}]
                    },
                    {
                        "matcher": "^Bash$",
                        "hooks": [{"type": "command", "command": "second-hook"}]
                    }
                ]
            }
        })
        .to_string(),
    )
    .unwrap();

    let hooks = crate::agents::scan_agent_items(home, AgentClient::CodexCli)
        .items
        .into_iter()
        .filter(|item| item.kind == AgentItemKind::Hook)
        .collect::<Vec<_>>();

    assert_eq!(hooks.len(), 2);
    assert!(hooks.iter().all(|item| item.name == "PreToolUse"));
    assert_eq!(hooks[0].matcher.as_deref(), Some("Bash"));
    assert_eq!(hooks[1].matcher.as_deref(), Some("^Bash$"));
    assert!(hooks.iter().all(|item| item.command.is_some()));
    assert_ne!(hooks[0].hook_hash, hooks[1].hook_hash);
    assert!(hooks.iter().all(|item| item.hook_hash.is_some()));
}

use serde_json::json;

use super::codex_hook_trust_hash;

#[test]
fn matches_codex_pre_tool_use_fixture() {
    let handler = json!({
        "type": "command",
        "command": "python3 /tmp/listed-hook.py",
        "timeout": 5,
        "async": true,
        "statusMessage": "running listed hook",
        "additionalContextLimit": 4096
    });
    assert_eq!(
        codex_hook_trust_hash("PreToolUse", Some("Bash"), &handler).unwrap(),
        "sha256:77af324465e1bd97065037ee4fb25d0d8f60bd6aa701379f5a6749d96dbae897"
    );
}

#[test]
fn omits_absent_matcher_and_uses_default_timeout() {
    let handler = json!({
        "type": "command",
        "command": "demo-hook"
    });
    assert_eq!(
        codex_hook_trust_hash("SessionStart", None, &handler).unwrap(),
        "sha256:7ad93463b3562fa3e8a50925c2a46fb4f4c09946dff77e73f65e334799894540"
    );
}

#[cfg(windows)]
#[test]
fn uses_command_windows_for_the_normalized_hash() {
    let handler = json!({
        "type": "command",
        "command": "fallback-hook",
        "commandWindows": "win-hook",
        "timeout": 10
    });
    assert_eq!(
        codex_hook_trust_hash("PreToolUse", Some("Bash"), &handler).unwrap(),
        "sha256:978b42210c8c5945b6dc2ee4007fca224537da15035a7ff392a4c8f81896d610"
    );
}

#[cfg(windows)]
#[test]
fn matches_a_real_codex_trust_record_with_windows_command_and_limits() {
    // Captured from a Codex CLI 0.161.0 trust entry for the same hook.
    let handler = json!({
        "type": "command",
        "command": "sh \"C:\\Users\\yangxinran\\.prelay\\packages\\superpowers\\0a98854bd9653a01e188ff567803a54457bc9b2a\\hooks/scripts/session-start.sh\"",
        "commandWindows": "powershell -NoProfile -ExecutionPolicy Bypass -File \"C:\\Users\\yangxinran\\.prelay\\packages\\superpowers\\0a98854bd9653a01e188ff567803a54457bc9b2a\\hooks\\scripts\\session-start.ps1\"",
        "timeout": 10,
        "statusMessage": "Loading superpowers",
        "additionalContextLimit": 5000
    });
    assert_eq!(
        codex_hook_trust_hash(
            "SessionStart",
            Some("startup|resume|clear|compact|fork"),
            &handler
        )
        .unwrap(),
        "sha256:bb797c82ae4ee85722041ce5bc67d5c52e5b6cfc86e5d77403ebb58c480fadf7"
    );
}

#[test]
fn ignores_matcher_for_user_prompt_submit() {
    let handler = json!({
        "type": "command",
        "command": "python3 /tmp/user_prompt_submit_hook.py"
    });
    assert_eq!(
        codex_hook_trust_hash("UserPromptSubmit", Some("ignored"), &handler).unwrap(),
        "sha256:3e5280b458f75c4ca5e3af342d197820adc572203b46a411d49887a102b30646"
    );
}

#[test]
fn toml_serialization_omits_a_missing_matcher() {
    #[derive(serde::Serialize)]
    struct Group {
        matcher: Option<String>,
        hooks: Vec<u8>,
    }

    let value = toml::Value::try_from(Group {
        matcher: None,
        hooks: vec![1],
    })
    .unwrap();
    assert!(!value
        .as_table()
        .expect("group should serialize as a table")
        .contains_key("matcher"));
}

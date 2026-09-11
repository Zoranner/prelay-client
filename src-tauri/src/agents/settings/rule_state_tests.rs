use std::fs;

use tempfile::tempdir;

use super::{save_user_settings, AgentSettings, CodexSettings};

#[test]
fn saving_changed_rules_clears_the_managed_rule_state() {
    let directory = tempdir().unwrap();
    let codex_root = directory.path().join(".codex");
    fs::create_dir_all(&codex_root).unwrap();
    fs::create_dir_all(codex_root.join(".prelay")).unwrap();
    fs::write(codex_root.join("AGENTS.md"), "published rule").unwrap();
    fs::write(
        codex_root.join(".prelay").join("rule.json"),
        r#"{
  "package": "development-rules",
  "version": "v1.0.0",
  "commitSha": "commit"
}"#,
    )
    .unwrap();

    save_user_settings(
        directory.path(),
        &AgentSettings::CodexCli(CodexSettings {
            rules: Some("user rule".to_string()),
            ..Default::default()
        }),
        None,
    )
    .unwrap();

    assert!(!codex_root.join(".prelay").join("rule.json").exists());
}

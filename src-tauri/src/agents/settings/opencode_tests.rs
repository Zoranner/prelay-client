use std::fs;

use tempfile::tempdir;

use super::test_helpers::catalog_model;
use super::{
    save_user_settings, AgentConnection, AgentSettings, OpenCodeConnection, OpenCodeSettings,
};

#[test]
fn clears_the_opencode_connection_and_keeps_other_configuration() {
    let directory = tempdir().unwrap();
    let config_directory = directory.path().join(".config").join("opencode");
    fs::create_dir_all(&config_directory).unwrap();
    fs::write(
        config_directory.join("opencode.jsonc"),
        r#"{ "mcp": { "keep": { "type": "local", "command": ["keep"] } } }"#,
    )
    .unwrap();

    let settings = OpenCodeSettings {
        model: Some("team-flash".to_string()),
        ..Default::default()
    };
    save_user_settings(
        directory.path(),
        &AgentSettings::OpenCode(settings.clone()),
        Some(&AgentConnection::OpenCode(OpenCodeConnection::Prelay {
            endpoint_id: "endpoint-id".to_string(),
            relay_url: "https://relay.example.test".to_string(),
            endpoint_token: "endpoint-token".to_string(),
            models: Some(vec![catalog_model("team-flash", "Team Flash")]),
        })),
    )
    .unwrap();

    let changed =
        save_user_settings(directory.path(), &AgentSettings::OpenCode(settings), None).unwrap();

    let config: serde_json::Value =
        json5::from_str(&fs::read_to_string(config_directory.join("opencode.jsonc")).unwrap())
            .unwrap();
    assert!(changed, "解除接入应报告内容变化");
    assert!(config.get("provider").is_none());
    assert!(config.get("model").is_none());
    assert_eq!(config["mcp"]["keep"]["command"][0], "keep");
}

#[test]
fn saves_opencode_prelay_provider_without_replacing_other_configuration() {
    let directory = tempdir().unwrap();
    let config_directory = directory.path().join(".config").join("opencode");
    fs::create_dir_all(&config_directory).unwrap();
    fs::write(
        config_directory.join("opencode.jsonc"),
        r#"{
  // This provider is not managed by Prelay.
  "provider": {
    "other": { "options": { "apiKey": "other-token" } },
    "prelay": { "models": { "retired-model": {} } }
  },
  "mcp": { "keep": { "type": "local", "command": ["keep"] } }
}"#,
    )
    .unwrap();

    let settings = OpenCodeSettings {
        model: Some("deepseek-coder".to_string()),
        rules: Some("始终先阅读仓库约束。".to_string()),
        ..Default::default()
    };
    let connection = OpenCodeConnection::Prelay {
        endpoint_id: "endpoint-id".to_string(),
        relay_url: "https://relay.example.test/".to_string(),
        endpoint_token: "endpoint-token".to_string(),
        models: Some(vec![
            catalog_model("deepseek-coder", "DeepSeek Coder"),
            catalog_model("deepseek-reasoner", "DeepSeek Reasoner"),
        ]),
    };

    save_user_settings(
        directory.path(),
        &AgentSettings::OpenCode(settings),
        Some(&AgentConnection::OpenCode(connection)),
    )
    .unwrap();

    let saved = fs::read_to_string(config_directory.join("opencode.jsonc")).unwrap();
    let config: serde_json::Value = json5::from_str(&saved).unwrap();
    assert_eq!(
        config["provider"]["other"]["options"]["apiKey"],
        "other-token"
    );
    assert_eq!(config["mcp"]["keep"]["command"][0], "keep");
    assert_eq!(
        config["provider"]["prelay"]["npm"],
        "@ai-sdk/openai-compatible"
    );
    assert_eq!(
        config["provider"]["prelay"]["options"]["baseURL"],
        "https://relay.example.test/v1"
    );
    assert_eq!(
        config["provider"]["prelay"]["options"]["apiKey"],
        "endpoint-token"
    );
    assert_eq!(config["model"], "prelay/deepseek-coder");
    assert_eq!(
        config["provider"]["prelay"]["models"]["deepseek-coder"]["name"],
        "DeepSeek Coder"
    );
    assert_eq!(
        config["provider"]["prelay"]["models"]["deepseek-reasoner"]["name"],
        "DeepSeek Reasoner"
    );
    assert!(
        config["provider"]["prelay"]["models"]
            .get("retired-model")
            .is_none(),
        "接入点已下架的模型不应留在 OpenCode 配置里"
    );
    assert_eq!(
        fs::read_to_string(config_directory.join("AGENTS.md")).unwrap(),
        "始终先阅读仓库约束。"
    );
}

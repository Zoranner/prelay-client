use std::fs;

use tempfile::tempdir;

use crate::agents::AgentClient;

use super::{
    read_user_settings, save_user_settings, AgentConnection, AgentSettings, ChatGptSettings,
    CodexConnection, CodexSettings, OpenCodeConnection, OpenCodeSettings,
};

#[test]
fn chatgpt_settings_read_and_write_the_codex_configuration() {
    let directory = tempdir().unwrap();
    let codex_root = directory.path().join(".codex");
    fs::create_dir_all(&codex_root).unwrap();
    fs::write(
        codex_root.join("config.toml"),
        "model = \"initial-model\"\n",
    )
    .unwrap();

    let settings = read_user_settings(directory.path(), AgentClient::ChatGpt);
    assert!(matches!(
        settings,
        AgentSettings::ChatGpt(ChatGptSettings(CodexSettings {
            model: Some(ref model),
            ..
        })) if model == "initial-model"
    ));

    save_user_settings(
        directory.path(),
        &AgentSettings::ChatGpt(ChatGptSettings(CodexSettings {
            model: Some("chatgpt-model".to_string()),
            ..Default::default()
        })),
        None,
    )
    .unwrap();

    let saved = fs::read_to_string(codex_root.join("config.toml")).unwrap();
    assert!(saved.contains("model = \"chatgpt-model\""));
}

#[test]
fn saves_prelay_connection_for_initial_codex_config_without_provider_entries() {
    let directory = tempdir().unwrap();
    let codex_root = directory.path().join(".codex");
    fs::create_dir_all(&codex_root).unwrap();
    fs::write(
        codex_root.join("config.toml"),
        r#"
model_reasoning_effort = "high"
personality = "pragmatic"
sandbox_mode = "workspace-write"
disable_response_storage = true
web_search = "live"

[features]
memories = true
goals = true
workspace_dependencies = false

[agents]
max_threads = 16
max_depth = 1
job_max_runtime_seconds = 1800

[sandbox_workspace_write]
network_access = true

[shell_environment_policy]
inherit = "all"

[windows]
sandbox = "unelevated"
"#,
    )
    .unwrap();

    let settings = read_user_settings(directory.path(), AgentClient::CodexCli);
    let connection = CodexConnection::Prelay {
        endpoint_id: "endpoint-id".to_string(),
        endpoint_name: "Prelay".to_string(),
        relay_url: "https://relay.example.test/".to_string(),
        endpoint_token: "endpoint-token".to_string(),
    };

    save_user_settings(
        directory.path(),
        &settings,
        Some(&AgentConnection::CodexCli(connection)),
    )
    .unwrap();

    let saved = fs::read_to_string(codex_root.join("config.toml")).unwrap();
    let config: toml::Value = toml::from_str(&saved).unwrap();
    assert_eq!(config["model_provider"].as_str(), Some("custom"));
    assert_eq!(
        config["model_providers"]["custom"]["name"].as_str(),
        Some("Prelay")
    );
    assert_eq!(
        config["model_providers"]["custom"]["base_url"].as_str(),
        Some("https://relay.example.test/v1")
    );

    let auth: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(codex_root.join("auth.json")).unwrap()).unwrap();
    assert_eq!(auth["OPENAI_API_KEY"], "endpoint-token");
}

#[test]
fn saves_codex_settings_when_auth_json_is_empty() {
    let directory = tempdir().unwrap();
    let codex_root = directory.path().join(".codex");
    fs::create_dir_all(&codex_root).unwrap();
    fs::write(codex_root.join("config.toml"), "").unwrap();
    fs::write(codex_root.join("auth.json"), "").unwrap();

    let settings = read_user_settings(directory.path(), AgentClient::CodexCli);
    let connection = CodexConnection::Custom {
        base_url: "https://relay.example.test/v1".to_string(),
        token: "endpoint-token".to_string(),
    };
    save_user_settings(
        directory.path(),
        &settings,
        Some(&AgentConnection::CodexCli(connection)),
    )
    .unwrap();

    let auth: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(codex_root.join("auth.json")).unwrap()).unwrap();
    assert_eq!(auth["OPENAI_API_KEY"], "endpoint-token");
    assert!(codex_root.join("config.toml").is_file());
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
    "other": { "options": { "apiKey": "other-token" } }
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
        relay_url: "https://relay.example.test/".to_string(),
        endpoint_token: "endpoint-token".to_string(),
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
        fs::read_to_string(config_directory.join("AGENTS.md")).unwrap(),
        "始终先阅读仓库约束。"
    );
}

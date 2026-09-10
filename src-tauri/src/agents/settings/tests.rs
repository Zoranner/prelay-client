use std::fs;

use tempfile::tempdir;

use crate::agents::AgentClient;

use super::{
    read_user_settings, save_user_settings, AgentConnection, AgentSettings, ChatGptSettings,
    ClaudeCodeConnection, ClaudeCodeSettings, CodexConnection, CodexSettings, OpenCodeConnection,
    OpenCodeSettings,
};

#[test]
fn saves_claude_code_prelay_settings_without_replacing_other_environment_values() {
    let directory = tempdir().unwrap();
    let claude_root = directory.path().join(".claude");
    fs::create_dir_all(&claude_root).unwrap();
    fs::write(
        claude_root.join("settings.json"),
        r#"{"env":{"OTHER_SETTING":"keep-me"}}"#,
    )
    .unwrap();

    save_user_settings(
        directory.path(),
        &AgentSettings::ClaudeCode(ClaudeCodeSettings {
            model: Some("claude-sonnet".to_string()),
            rules: Some("遵守项目规则。".to_string()),
            ..Default::default()
        }),
        Some(&AgentConnection::ClaudeCode(ClaudeCodeConnection::Prelay {
            relay_url: "https://relay.example.test/".to_string(),
            endpoint_token: "endpoint-token".to_string(),
        })),
    )
    .unwrap();

    let config: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(claude_root.join("settings.json")).unwrap())
            .unwrap();
    assert_eq!(config["env"]["OTHER_SETTING"], "keep-me");
    assert_eq!(
        config["env"]["ANTHROPIC_BASE_URL"],
        "https://relay.example.test"
    );
    assert_eq!(config["env"]["ANTHROPIC_AUTH_TOKEN"], "endpoint-token");
    assert_eq!(config["env"]["ANTHROPIC_MODEL"], "claude-sonnet");
    assert_eq!(
        fs::read_to_string(claude_root.join("CLAUDE.md")).unwrap(),
        "遵守项目规则。"
    );
}

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
        models: Vec::new(),
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

fn console_payload(
    client: &str,
    settings: serde_json::Value,
    connection: serde_json::Value,
) -> (AgentSettings, AgentConnection) {
    let settings = serde_json::from_value(serde_json::json!({
        "client": client,
        "settings": settings,
    }))
    .unwrap_or_else(|error| panic!("{client} settings payload does not deserialize: {error}"));
    let connection = serde_json::from_value(serde_json::json!({
        "client": client,
        "connection": connection,
    }))
    .unwrap_or_else(|error| panic!("{client} connection payload does not deserialize: {error}"));
    (settings, connection)
}

#[test]
fn saves_claude_code_model_assignment_and_relay_settings() {
    let directory = tempdir().unwrap();
    let claude_root = directory.path().join(".claude");
    fs::create_dir_all(&claude_root).unwrap();
    fs::write(
        claude_root.join("settings.json"),
        r#"{"env":{"OTHER_SETTING":"keep-me"},"permissions":{"defaultMode":"acceptEdits"}}"#,
    )
    .unwrap();

    save_user_settings(
        directory.path(),
        &AgentSettings::ClaudeCode(ClaudeCodeSettings {
            model: Some("deepseek-v4-pro".to_string()),
            opus_model: Some("deepseek-v4-pro".to_string()),
            sonnet_model: Some("deepseek-v4-pro".to_string()),
            haiku_model: Some("deepseek-v4-flash".to_string()),
            subagent_model: Some("deepseek-v4-flash".to_string()),
            api_timeout_ms: Some(300_000),
            max_output_tokens: Some(64_000),
            tool_search_enabled: Some(true),
            nonessential_traffic_disabled: Some(true),
            ..Default::default()
        }),
        Some(&AgentConnection::ClaudeCode(ClaudeCodeConnection::Prelay {
            relay_url: "https://relay.example.test".to_string(),
            endpoint_token: "endpoint-token".to_string(),
        })),
    )
    .unwrap();

    let config: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(claude_root.join("settings.json")).unwrap())
            .unwrap();
    assert_eq!(config["env"]["OTHER_SETTING"], "keep-me");
    assert_eq!(config["permissions"]["defaultMode"], "acceptEdits");
    assert_eq!(config["env"]["ANTHROPIC_MODEL"], "deepseek-v4-pro");
    assert_eq!(
        config["env"]["ANTHROPIC_DEFAULT_OPUS_MODEL"],
        "deepseek-v4-pro"
    );
    assert_eq!(
        config["env"]["ANTHROPIC_DEFAULT_SONNET_MODEL"],
        "deepseek-v4-pro"
    );
    assert_eq!(
        config["env"]["ANTHROPIC_DEFAULT_HAIKU_MODEL"],
        "deepseek-v4-flash"
    );
    assert_eq!(
        config["env"]["CLAUDE_CODE_SUBAGENT_MODEL"],
        "deepseek-v4-flash"
    );
    assert_eq!(config["env"]["API_TIMEOUT_MS"], "300000");
    assert_eq!(config["env"]["CLAUDE_CODE_MAX_OUTPUT_TOKENS"], "64000");
    assert_eq!(config["env"]["ENABLE_TOOL_SEARCH"], "true");
    assert_eq!(
        config["env"]["CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC"],
        "1"
    );
}

#[test]
fn clears_claude_code_model_assignment_and_relay_settings_when_unset() {
    let directory = tempdir().unwrap();
    let claude_root = directory.path().join(".claude");
    fs::create_dir_all(&claude_root).unwrap();
    fs::write(
        claude_root.join("settings.json"),
        r#"{"env":{"API_TIMEOUT_MS":"300000","ENABLE_TOOL_SEARCH":"true","OTHER_SETTING":"keep-me"}}"#,
    )
    .unwrap();

    save_user_settings(
        directory.path(),
        &AgentSettings::ClaudeCode(ClaudeCodeSettings {
            model: Some("deepseek-v4-pro".to_string()),
            ..Default::default()
        }),
        Some(&AgentConnection::ClaudeCode(ClaudeCodeConnection::Prelay {
            relay_url: "https://relay.example.test".to_string(),
            endpoint_token: "endpoint-token".to_string(),
        })),
    )
    .unwrap();

    let config: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(claude_root.join("settings.json")).unwrap())
            .unwrap();
    assert_eq!(config["env"]["OTHER_SETTING"], "keep-me");
    assert!(config["env"].get("API_TIMEOUT_MS").is_none());
    assert!(config["env"].get("ENABLE_TOOL_SEARCH").is_none());
    assert!(config["env"].get("ANTHROPIC_DEFAULT_HAIKU_MODEL").is_none());
}

#[test]
fn reads_claude_code_model_assignment_and_relay_settings() {
    let directory = tempdir().unwrap();
    let claude_root = directory.path().join(".claude");
    fs::create_dir_all(&claude_root).unwrap();
    fs::write(
        claude_root.join("settings.json"),
        r#"{"env":{"ANTHROPIC_DEFAULT_HAIKU_MODEL":"deepseek-v4-flash","API_TIMEOUT_MS":"300000","ENABLE_TOOL_SEARCH":"true","CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC":"1"}}"#,
    )
    .unwrap();

    let settings = read_user_settings(directory.path(), AgentClient::ClaudeCode);
    let AgentSettings::ClaudeCode(settings) = settings else {
        panic!("expected Claude Code settings");
    };

    assert_eq!(settings.haiku_model.as_deref(), Some("deepseek-v4-flash"));
    assert_eq!(settings.api_timeout_ms, Some(300_000));
    assert_eq!(settings.tool_search_enabled, Some(true));
    assert_eq!(settings.nonessential_traffic_disabled, Some(true));
    assert_eq!(settings.opus_model, None);
    assert_eq!(settings.max_output_tokens, None);
}

#[test]
fn deserializes_the_settings_save_payload_sent_by_the_desktop_console() {
    let codex_settings = serde_json::json!({
        "endpoint": "endpoint-1",
        "model": "gpt-5-codex",
        "personality": "pragmatic",
        "webSearch": true,
        "sandbox": "workspace-write",
        "disableResponseStorage": true,
        "maxThreads": 16,
        "maxDepth": 1,
        "jobMaxRuntimeSeconds": 1800,
        "networkAccess": true,
        "shellEnvironmentInherit": "all",
        "windowsSandbox": "unelevated",
        "features": {
            "memories": true,
            "goals": true,
            "workspaceDependencies": false,
        },
        "rules": "",
    });
    let codex_connection = serde_json::json!({
        "kind": "prelay",
        "endpointId": "endpoint-1",
        "endpointName": "Prelay",
        "endpointToken": "endpoint-token",
        "relayUrl": "https://relay.example.test",
        "models": [],
    });
    let (settings, connection) =
        console_payload("codexCli", codex_settings.clone(), codex_connection.clone());
    assert!(matches!(settings, AgentSettings::CodexCli(_)));
    assert!(matches!(
        connection,
        AgentConnection::CodexCli(CodexConnection::Prelay { .. })
    ));

    let (settings, connection) = console_payload("chatgpt", codex_settings, codex_connection);
    assert!(matches!(settings, AgentSettings::ChatGpt(_)));
    assert!(matches!(
        connection,
        AgentConnection::ChatGpt(CodexConnection::Prelay { .. })
    ));

    let simple_connection = serde_json::json!({
        "kind": "prelay",
        "endpointToken": "endpoint-token",
        "relayUrl": "https://relay.example.test",
    });
    let (settings, connection) = console_payload(
        "openCode",
        serde_json::json!({ "endpoint": "endpoint-1", "model": "deepseek-coder", "rules": "" }),
        simple_connection.clone(),
    );
    assert!(matches!(settings, AgentSettings::OpenCode(_)));
    assert!(matches!(
        connection,
        AgentConnection::OpenCode(OpenCodeConnection::Prelay { .. })
    ));

    let (settings, connection) = console_payload(
        "claudeCode",
        serde_json::json!({
            "endpoint": "endpoint-1",
            "model": "claude-sonnet",
            "opusModel": "claude-opus",
            "sonnetModel": "claude-sonnet",
            "haikuModel": "claude-haiku",
            "subagentModel": "claude-haiku",
            "apiTimeoutMs": 300000,
            "maxOutputTokens": 64000,
            "toolSearchEnabled": true,
            "nonessentialTrafficDisabled": true,
            "rules": "",
        }),
        simple_connection,
    );
    let AgentSettings::ClaudeCode(settings) = settings else {
        panic!("expected Claude Code settings");
    };
    assert!(matches!(
        connection,
        AgentConnection::ClaudeCode(ClaudeCodeConnection::Prelay { .. })
    ));
    assert_eq!(settings.opus_model.as_deref(), Some("claude-opus"));
    assert_eq!(settings.sonnet_model.as_deref(), Some("claude-sonnet"));
    assert_eq!(settings.haiku_model.as_deref(), Some("claude-haiku"));
    assert_eq!(settings.subagent_model.as_deref(), Some("claude-haiku"));
    assert_eq!(settings.api_timeout_ms, Some(300_000));
    assert_eq!(settings.max_output_tokens, Some(64_000));
    assert_eq!(settings.tool_search_enabled, Some(true));
    assert_eq!(settings.nonessential_traffic_disabled, Some(true));
}

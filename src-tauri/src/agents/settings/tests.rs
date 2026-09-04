use std::fs;

use prelay_protocol::CatalogLanguageModelResponse;
use serde_json::json;
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
fn saves_every_prelay_model_alias_to_the_codex_catalog() {
    let directory = tempdir().unwrap();
    let codex_root = directory.path().join(".codex");
    fs::create_dir_all(&codex_root).unwrap();
    fs::write(codex_root.join("config.toml"), "").unwrap();

    let connection = CodexConnection::Prelay {
        endpoint_id: "endpoint-id".to_string(),
        endpoint_name: "Endpoint 1".to_string(),
        relay_url: "https://relay.example.test".to_string(),
        endpoint_token: "endpoint-token".to_string(),
        models: vec![
            catalog_model("team-flash", "Team Flash"),
            catalog_model("minimax-main", "MiniMax Main"),
        ],
    };
    let settings = CodexSettings {
        model: Some("team-flash".to_string()),
        ..Default::default()
    };

    save_user_settings(
        directory.path(),
        &AgentSettings::CodexCli(settings),
        Some(&AgentConnection::CodexCli(connection)),
    )
    .unwrap();

    let catalog: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(codex_root.join("models.json")).unwrap()).unwrap();
    assert_eq!(catalog["models"].as_array().unwrap().len(), 2);
    assert_eq!(catalog["models"][0]["slug"], "team-flash");
    assert_eq!(catalog["models"][1]["slug"], "minimax-main");
    assert_eq!(catalog["models"][0]["id"], "team-flash");
    assert_eq!(catalog["models"][0]["display_name"], "Team Flash");
    assert_eq!(catalog["models"][0]["reasoning_efforts"][0], "low");
    assert_eq!(catalog["models"][0]["context_window"], 131072);
    assert_eq!(
        catalog["models"][0]["base_instructions"],
        "Use the team policy."
    );
    assert!(catalog["models"][0].get("prefer_websockets").is_none());

    let saved = fs::read_to_string(codex_root.join("config.toml")).unwrap();
    let config: toml::Value = toml::from_str(&saved).unwrap();
    assert_eq!(
        config["model_catalog_json"].as_str(),
        Some(
            codex_root
                .join("models.json")
                .to_string_lossy()
                .replace('\\', "/")
                .as_str()
        )
    );
}

#[test]
fn rejects_a_default_model_that_is_not_mapped_by_the_prelay_endpoint() {
    let directory = tempdir().unwrap();
    let codex_root = directory.path().join(".codex");
    fs::create_dir_all(&codex_root).unwrap();
    fs::write(codex_root.join("config.toml"), "").unwrap();

    let connection = CodexConnection::Prelay {
        endpoint_id: "endpoint-id".to_string(),
        endpoint_name: "Endpoint 1".to_string(),
        relay_url: "https://relay.example.test".to_string(),
        endpoint_token: "endpoint-token".to_string(),
        models: vec![catalog_model("team-flash", "Team Flash")],
    };
    let settings = CodexSettings {
        model: Some("old-endpoint-model".to_string()),
        ..Default::default()
    };

    let error = save_user_settings(
        directory.path(),
        &AgentSettings::CodexCli(settings),
        Some(&AgentConnection::CodexCli(connection)),
    )
    .unwrap_err();

    assert_eq!(error, "默认模型不属于所选接入点。");
    assert!(!codex_root.join("models.json").exists());
}

#[test]
fn rejects_prelay_model_without_required_catalog_fields() {
    let error = serde_json::from_value::<CodexConnection>(json!({
        "kind": "prelay",
        "endpointId": "endpoint-id",
        "endpointName": "Endpoint 1",
        "relayUrl": "https://relay.example.test",
        "endpointToken": "endpoint-token",
        "models": [{ "id": "team-flash" }]
    }))
    .unwrap_err();
    assert!(error.to_string().contains("display_name"));
}

#[test]
fn custom_connection_does_not_create_or_clear_model_catalog() {
    let directory = tempdir().unwrap();
    let codex_root = directory.path().join(".codex");
    fs::create_dir_all(&codex_root).unwrap();
    fs::write(
        codex_root.join("config.toml"),
        "model_catalog_json = \"existing.json\"\n",
    )
    .unwrap();
    let settings = CodexSettings::default();
    let connection = CodexConnection::Custom {
        base_url: "https://custom.example.test/v1".to_string(),
        token: "custom-token".to_string(),
    };

    save_user_settings(
        directory.path(),
        &AgentSettings::CodexCli(settings),
        Some(&AgentConnection::CodexCli(connection)),
    )
    .unwrap();

    let config: toml::Value =
        toml::from_str(&fs::read_to_string(codex_root.join("config.toml")).unwrap()).unwrap();
    assert_eq!(config["model_catalog_json"].as_str(), Some("existing.json"));
    assert!(!codex_root.join("models.json").exists());
}

#[test]
fn failed_catalog_write_does_not_write_config() {
    let directory = tempdir().unwrap();
    let codex_root = directory.path().join(".codex");
    fs::create_dir_all(codex_root.join("models.json")).unwrap();
    fs::write(codex_root.join("config.toml"), "model = \"before\"\n").unwrap();
    let settings = CodexSettings {
        model: Some("team-flash".to_string()),
        ..Default::default()
    };
    let connection = CodexConnection::Prelay {
        endpoint_id: "endpoint-id".to_string(),
        endpoint_name: "Endpoint 1".to_string(),
        relay_url: "https://relay.example.test".to_string(),
        endpoint_token: "endpoint-token".to_string(),
        models: vec![catalog_model("team-flash", "Team Flash")],
    };

    assert!(save_user_settings(
        directory.path(),
        &AgentSettings::CodexCli(settings),
        Some(&AgentConnection::CodexCli(connection)),
    )
    .is_err());
    assert_eq!(
        fs::read_to_string(codex_root.join("config.toml")).unwrap(),
        "model = \"before\"\n"
    );
}

fn catalog_model(id: &str, display_name: &str) -> CatalogLanguageModelResponse {
    CatalogLanguageModelResponse {
        id: id.to_string(),
        display_name: display_name.to_string(),
        description: Some("Catalog description".to_string()),
        reasoning_efforts: Some(vec!["low".to_string(), "high".to_string()]),
        default_reasoning_effort: Some("high".to_string()),
        context_window: Some(131072),
        max_context_window: Some(131072),
        effective_context_window_percent: Some(95),
        input_modalities: Some(vec!["text".to_string()]),
        supports_parallel_tool_calls: Some(true),
        supports_reasoning_summaries: Some(true),
        supports_image_detail_original: Some(false),
        support_verbosity: Some(true),
        default_verbosity: Some("low".to_string()),
        apply_patch_tool_type: None,
        web_search_tool_type: Some("text".to_string()),
        truncation_policy: None,
        reasoning_summary_format: Some("experimental".to_string()),
        default_reasoning_summary: Some("none".to_string()),
        shell_type: None,
        visibility: Some("public".to_string()),
        supported_in_api: Some(true),
        priority: Some(1),
        base_instructions: Some("Use the team policy.".to_string()),
        experimental_supported_tools: None,
        minimal_client_version: None,
    }
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

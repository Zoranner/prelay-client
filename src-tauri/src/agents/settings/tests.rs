use std::fs;

use tempfile::tempdir;

use crate::agents::AgentClient;

use super::test_helpers::catalog_model;
use super::{
    read_user_settings, save_user_settings, AgentConnection, AgentSettings, ChatGptSettings,
    CodexConnection, CodexSettings, OpenCodeConnection,
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
fn reports_no_change_when_the_same_connection_is_saved_twice() {
    let directory = tempdir().unwrap();
    let codex_root = directory.path().join(".codex");
    fs::create_dir_all(&codex_root).unwrap();
    fs::write(codex_root.join("config.toml"), "").unwrap();

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
    let save = || {
        save_user_settings(
            directory.path(),
            &AgentSettings::CodexCli(settings.clone()),
            Some(&AgentConnection::CodexCli(connection.clone())),
        )
        .unwrap()
    };

    assert!(save(), "首次保存应写入配置");
    assert!(!save(), "内容一致时不应再改写配置");
}

#[test]
fn clears_the_codex_connection_without_touching_other_settings() {
    let directory = tempdir().unwrap();
    let codex_root = directory.path().join(".codex");
    fs::create_dir_all(&codex_root).unwrap();
    fs::write(codex_root.join("config.toml"), "").unwrap();

    let settings = CodexSettings {
        model: Some("team-flash".to_string()),
        sandbox: Some("workspace-write".to_string()),
        ..Default::default()
    };
    let connection = CodexConnection::Prelay {
        endpoint_id: "endpoint-id".to_string(),
        endpoint_name: "Endpoint 1".to_string(),
        relay_url: "https://relay.example.test".to_string(),
        endpoint_token: "endpoint-token".to_string(),
        models: vec![catalog_model("team-flash", "Team Flash")],
    };
    save_user_settings(
        directory.path(),
        &AgentSettings::CodexCli(settings.clone()),
        Some(&AgentConnection::CodexCli(connection)),
    )
    .unwrap();
    assert!(codex_root.join("models.json").exists());

    let changed =
        save_user_settings(directory.path(), &AgentSettings::CodexCli(settings), None).unwrap();

    let config: toml::Value =
        toml::from_str(&fs::read_to_string(codex_root.join("config.toml")).unwrap()).unwrap();
    assert!(changed, "解除接入应报告内容变化");
    assert!(config.get("model_provider").is_none());
    assert!(config.get("model_catalog_json").is_none());
    assert!(config.get("model_providers").is_none());
    assert!(!codex_root.join("models.json").exists());
    let auth: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(codex_root.join("auth.json")).unwrap()).unwrap();
    assert!(auth.get("OPENAI_API_KEY").is_none());
    // 用户自己的设置保留。
    assert_eq!(config["model"].as_str(), Some("team-flash"));
    assert_eq!(config["sandbox_mode"].as_str(), Some("workspace-write"));
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
fn deserializes_the_settings_save_payload_sent_by_the_desktop_console() {
    let codex_settings = serde_json::json!({
        "endpoint": "endpoint-1",
        "model": "gpt-5-codex",
        "personality": "pragmatic",
        "webSearch": true,
        "sandbox": "workspace-write",
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
        "endpointId": "endpoint-1",
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
}

#[test]
fn saves_auto_compact_limits_from_the_selected_model() {
    let directory = tempdir().unwrap();
    let codex_root = directory.path().join(".codex");
    fs::create_dir_all(&codex_root).unwrap();
    fs::write(
        codex_root.join("config.toml"),
        "model_context_window = 1\nmodel_auto_compact_token_limit = 1\n",
    )
    .unwrap();

    let settings = CodexSettings {
        model: Some("deepseek-v4-pro".to_string()),
        ..Default::default()
    };
    let connection = CodexConnection::Prelay {
        endpoint_id: "endpoint-id".to_string(),
        endpoint_name: "Prelay".to_string(),
        relay_url: "https://relay.example.test/".to_string(),
        endpoint_token: "endpoint-token".to_string(),
        models: vec![serde_json::from_value(serde_json::json!({
            "id": "deepseek-v4-pro",
            "display_name": "DeepSeek V4 Pro",
            "context_window": 1048576,
        }))
        .expect("catalog model")],
    };

    save_user_settings(
        directory.path(),
        &AgentSettings::CodexCli(settings),
        Some(&AgentConnection::CodexCli(connection)),
    )
    .unwrap();

    let saved = fs::read_to_string(codex_root.join("config.toml")).unwrap();
    let config: toml::Value = toml::from_str(&saved).unwrap();
    assert_eq!(config["model_context_window"].as_integer(), Some(1048576));
    assert_eq!(
        config["model_auto_compact_token_limit"].as_integer(),
        Some(943718)
    );
}

#[test]
fn uses_the_catalog_effective_context_percent_for_the_auto_compact_limit() {
    let directory = tempdir().unwrap();
    let codex_root = directory.path().join(".codex");
    fs::create_dir_all(&codex_root).unwrap();

    let settings = CodexSettings {
        model: Some("deepseek-flash".to_string()),
        ..Default::default()
    };
    let connection = CodexConnection::Prelay {
        endpoint_id: "endpoint-id".to_string(),
        endpoint_name: "Prelay".to_string(),
        relay_url: "https://relay.example.test/".to_string(),
        endpoint_token: "endpoint-token".to_string(),
        models: vec![serde_json::from_value(serde_json::json!({
            "id": "deepseek-flash",
            "display_name": "DeepSeek V4.1 Flash",
            "context_window": 1048576,
            "effective_context_window_percent": 55,
        }))
        .expect("catalog model")],
    };

    save_user_settings(
        directory.path(),
        &AgentSettings::CodexCli(settings),
        Some(&AgentConnection::CodexCli(connection)),
    )
    .unwrap();

    let saved = fs::read_to_string(codex_root.join("config.toml")).unwrap();
    let config: toml::Value = toml::from_str(&saved).unwrap();
    assert_eq!(config["model_context_window"].as_integer(), Some(1048576));
    assert_eq!(
        config["model_auto_compact_token_limit"].as_integer(),
        Some(576716)
    );
}

#[test]
fn removes_auto_compact_limits_when_the_model_window_is_unknown() {
    let directory = tempdir().unwrap();
    let codex_root = directory.path().join(".codex");
    fs::create_dir_all(&codex_root).unwrap();
    fs::write(
        codex_root.join("config.toml"),
        "model = \"unknown-model\"\nmodel_context_window = 1\nmodel_auto_compact_token_limit = 1\n",
    )
    .unwrap();

    let settings = CodexSettings {
        model: Some("unknown-model".to_string()),
        ..Default::default()
    };
    let connection = CodexConnection::Prelay {
        endpoint_id: "endpoint-id".to_string(),
        endpoint_name: "Prelay".to_string(),
        relay_url: "https://relay.example.test/".to_string(),
        endpoint_token: "endpoint-token".to_string(),
        models: vec![serde_json::from_value(serde_json::json!({
            "id": "unknown-model",
            "display_name": "Unknown Model",
        }))
        .expect("catalog model")],
    };

    save_user_settings(
        directory.path(),
        &AgentSettings::CodexCli(settings),
        Some(&AgentConnection::CodexCli(connection)),
    )
    .unwrap();

    let saved = fs::read_to_string(codex_root.join("config.toml")).unwrap();
    let config: toml::Value = toml::from_str(&saved).unwrap();
    assert!(config.get("model_context_window").is_none());
    assert!(config.get("model_auto_compact_token_limit").is_none());
}

use std::fs;

use prelay_protocol::CatalogLanguageModelResponse;
use serde_json::json;
use tempfile::tempdir;

use super::{
    save_user_settings, test_helpers::assert_no_null_values, AgentConnection, AgentSettings,
    CodexConnection, CodexSettings,
};

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
    assert_no_null_values(&catalog);
    assert_eq!(catalog["models"].as_array().unwrap().len(), 2);
    assert_eq!(catalog["models"][0]["slug"], "team-flash");
    assert_eq!(catalog["models"][1]["slug"], "minimax-main");
    assert_eq!(catalog["models"][0]["display_name"], "Team Flash");
    assert_eq!(catalog["models"][0]["shell_type"], "shell_command");
    assert_eq!(catalog["models"][0]["visibility"], "public");
    assert_eq!(catalog["models"][0]["priority"], 1);
    assert_eq!(catalog["models"][0]["support_verbosity"], true);
    assert_eq!(
        catalog["models"][0]["supported_reasoning_levels"][0]["effort"],
        "low"
    );
    assert_eq!(catalog["models"][0]["default_reasoning_level"], "high");
    assert_eq!(catalog["models"][0]["context_window"], 131072);
    assert_eq!(
        catalog["models"][0]["base_instructions"],
        "Use the team policy."
    );
    assert!(catalog["models"][0].get("id").is_none());
    assert!(catalog["models"][0].get("reasoning_efforts").is_none());
    assert!(catalog["models"][0].get("apply_patch_tool_type").is_none());
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
fn omits_a_default_reasoning_level_that_is_not_supported() {
    let directory = tempdir().unwrap();
    let codex_root = directory.path().join(".codex");
    fs::create_dir_all(&codex_root).unwrap();
    fs::write(codex_root.join("config.toml"), "").unwrap();

    let mut model = catalog_model("no-reasoning", "No Reasoning");
    model.reasoning_efforts = Some(Vec::new());
    model.default_reasoning_effort = Some("max".to_string());
    model.shell_type = None;
    model.supports_parallel_tool_calls = None;
    model.support_verbosity = None;
    model.truncation_policy = None;
    let connection = CodexConnection::Prelay {
        endpoint_id: "endpoint-id".to_string(),
        endpoint_name: "Endpoint 1".to_string(),
        relay_url: "https://relay.example.test".to_string(),
        endpoint_token: "endpoint-token".to_string(),
        models: vec![model],
    };

    save_user_settings(
        directory.path(),
        &AgentSettings::CodexCli(CodexSettings {
            model: Some("no-reasoning".to_string()),
            reasoning_effort: None,
            ..Default::default()
        }),
        Some(&AgentConnection::CodexCli(connection)),
    )
    .unwrap();

    let catalog: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(codex_root.join("models.json")).unwrap()).unwrap();
    assert_no_null_values(&catalog);
    assert_eq!(
        catalog["models"][0]["supported_reasoning_levels"],
        json!([])
    );
    assert_eq!(catalog["models"][0]["shell_type"], "shell_command");
    assert!(catalog["models"][0].get("truncation_policy").is_none());
    assert!(catalog["models"][0]
        .get("supports_parallel_tool_calls")
        .is_none());
    assert!(catalog["models"][0].get("support_verbosity").is_none());
    assert!(catalog["models"][0]
        .get("default_reasoning_level")
        .is_none());
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

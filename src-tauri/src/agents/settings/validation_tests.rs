use std::fs;

use prelay_protocol::CatalogLanguageModelResponse;
use tempfile::tempdir;

use super::{
    save_user_settings, tests::assert_no_null_values, AgentConnection, AgentSettings,
    CodexConnection, CodexSettings,
};

#[test]
fn default_settings_use_the_model_catalog_default_reasoning_effort() {
    assert!(CodexSettings::default().reasoning_effort.is_none());
}

#[test]
fn accepts_a_reasoning_effort_declared_by_the_selected_prelay_model() {
    let directory = tempdir().unwrap();
    let codex_root = prepare_codex_root(directory.path(), "model = \"before\"\n");
    let connection = prelay_connection(vec![catalog_model("team", vec!["low", "max"])]);
    let settings = CodexSettings {
        model: Some("team".to_string()),
        reasoning_effort: Some("max".to_string()),
        ..Default::default()
    };

    save_user_settings(
        directory.path(),
        &AgentSettings::CodexCli(settings),
        Some(&AgentConnection::CodexCli(connection)),
    )
    .unwrap();

    let config: toml::Value =
        toml::from_str(&fs::read_to_string(codex_root.join("config.toml")).unwrap()).unwrap();
    assert_eq!(config["model_reasoning_effort"].as_str(), Some("max"));
    let catalog: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(codex_root.join("models.json")).unwrap()).unwrap();
    assert_no_null_values(&catalog);
}

#[test]
fn rejects_an_unsupported_reasoning_effort_before_writing_files() {
    let directory = tempdir().unwrap();
    let codex_root = prepare_codex_root(directory.path(), "model = \"before\"\n");
    fs::write(
        codex_root.join("models.json"),
        "{\"models\":[{\"slug\":\"before\"}]}\n",
    )
    .unwrap();
    let connection = prelay_connection(vec![catalog_model("team", vec!["low", "high"])]);
    let settings = CodexSettings {
        model: Some("team".to_string()),
        reasoning_effort: Some("medium".to_string()),
        ..Default::default()
    };

    let error = save_user_settings(
        directory.path(),
        &AgentSettings::CodexCli(settings),
        Some(&AgentConnection::CodexCli(connection)),
    )
    .unwrap_err();

    assert_eq!(error, "推理强度不属于所选模型支持的档位。");
    assert_eq!(
        fs::read_to_string(codex_root.join("config.toml")).unwrap(),
        "model = \"before\"\n"
    );
    assert_eq!(
        fs::read_to_string(codex_root.join("models.json")).unwrap(),
        "{\"models\":[{\"slug\":\"before\"}]}\n"
    );
}

#[test]
fn rejects_any_non_empty_reasoning_effort_when_model_has_no_reasoning_efforts() {
    let directory = tempdir().unwrap();
    let codex_root = prepare_codex_root(directory.path(), "model = \"before\"\n");
    let connection = prelay_connection(vec![catalog_model("team", Vec::new())]);
    let settings = CodexSettings {
        model: Some("team".to_string()),
        reasoning_effort: Some("max".to_string()),
        ..Default::default()
    };

    let error = save_user_settings(
        directory.path(),
        &AgentSettings::CodexCli(settings),
        Some(&AgentConnection::CodexCli(connection)),
    )
    .unwrap_err();

    assert_eq!(error, "推理强度不属于所选模型支持的档位。");
    assert_eq!(
        fs::read_to_string(codex_root.join("config.toml")).unwrap(),
        "model = \"before\"\n"
    );
    assert!(!codex_root.join("models.json").exists());
}

#[test]
fn empty_reasoning_override_removes_the_config_value() {
    let directory = tempdir().unwrap();
    let codex_root = prepare_codex_root(
        directory.path(),
        "model_reasoning_effort = \"high\"\nmodel = \"team\"\n",
    );
    let connection = prelay_connection(vec![catalog_model("team", vec!["low", "high"])]);
    let settings = CodexSettings {
        model: Some("team".to_string()),
        reasoning_effort: Some("  ".to_string()),
        ..Default::default()
    };

    save_user_settings(
        directory.path(),
        &AgentSettings::CodexCli(settings),
        Some(&AgentConnection::CodexCli(connection)),
    )
    .unwrap();

    let config: toml::Value =
        toml::from_str(&fs::read_to_string(codex_root.join("config.toml")).unwrap()).unwrap();
    assert!(config.get("model_reasoning_effort").is_none());
}

fn prepare_codex_root(home: &std::path::Path, config: &str) -> std::path::PathBuf {
    let codex_root = home.join(".codex");
    fs::create_dir_all(&codex_root).unwrap();
    fs::write(codex_root.join("config.toml"), config).unwrap();
    codex_root
}

fn prelay_connection(models: Vec<CatalogLanguageModelResponse>) -> CodexConnection {
    CodexConnection::Prelay {
        endpoint_id: "endpoint-id".to_string(),
        endpoint_name: "Endpoint".to_string(),
        relay_url: "https://relay.example.test".to_string(),
        endpoint_token: "endpoint-token".to_string(),
        models,
    }
}

fn catalog_model(id: &str, reasoning_efforts: Vec<&str>) -> CatalogLanguageModelResponse {
    CatalogLanguageModelResponse {
        id: id.to_string(),
        display_name: id.to_string(),
        description: None,
        reasoning_efforts: Some(reasoning_efforts.into_iter().map(str::to_string).collect()),
        default_reasoning_effort: None,
        context_window: None,
        max_context_window: None,
        effective_context_window_percent: None,
        input_modalities: None,
        supports_parallel_tool_calls: None,
        supports_reasoning_summaries: None,
        supports_image_detail_original: None,
        support_verbosity: None,
        default_verbosity: None,
        apply_patch_tool_type: None,
        web_search_tool_type: None,
        truncation_policy: None,
        reasoning_summary_format: None,
        default_reasoning_summary: None,
        shell_type: None,
        visibility: None,
        supported_in_api: None,
        priority: None,
        base_instructions: None,
        experimental_supported_tools: None,
        minimal_client_version: None,
    }
}

#[test]
fn rejects_prelay_model_without_required_catalog_fields() {
    let error = serde_json::from_value::<CodexConnection>(serde_json::json!({
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

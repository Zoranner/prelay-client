use std::path::Path;

use prelay_protocol::CatalogLanguageModelResponse;

use super::{
    codex::prelay_base_url,
    document::{
        json_string, read_jsonc, read_jsonc_document, read_optional_text, write_text_if_changed,
    },
    OpenCodeConnection, OpenCodeSettings,
};
use crate::agents::opencode_configuration_path;

pub(super) fn save_opencode_settings(
    home: &Path,
    settings: &OpenCodeSettings,
    connection: Option<&OpenCodeConnection>,
) -> Result<bool, String> {
    let config_path = opencode_configuration_path(home);
    let mut document = read_jsonc_document(&config_path, "OpenCode config")?;
    let root = document
        .as_object_mut()
        .ok_or_else(|| "OpenCode config root must be an object".to_string())?;
    root.entry("$schema")
        .or_insert_with(|| serde_json::json!("https://opencode.ai/config.json"));

    let model = settings
        .model
        .as_deref()
        .filter(|model| !model.trim().is_empty());
    if let Some(model) = model {
        root.insert(
            "model".to_string(),
            serde_json::Value::String(format!("prelay/{model}")),
        );
    }

    if connection.is_none() {
        // 没有接入点时按“解除接入”处理：删掉 provider 与 prelay 模型选择。
        clear_opencode_connection(root);
    }
    if let Some(OpenCodeConnection::Prelay {
        relay_url,
        endpoint_token,
        models,
        ..
    }) = connection
    {
        let providers = root
            .entry("provider")
            .or_insert_with(|| serde_json::json!({}));
        let providers = providers
            .as_object_mut()
            .ok_or_else(|| "OpenCode providers must be an object".to_string())?;
        let prelay = providers
            .entry("prelay")
            .or_insert_with(|| serde_json::json!({}));
        let prelay = prelay
            .as_object_mut()
            .ok_or_else(|| "OpenCode Prelay provider must be an object".to_string())?;
        prelay.insert(
            "npm".to_string(),
            serde_json::Value::String("@ai-sdk/openai-compatible".to_string()),
        );
        prelay.insert(
            "name".to_string(),
            serde_json::Value::String("Prelay".to_string()),
        );
        let options = prelay
            .entry("options")
            .or_insert_with(|| serde_json::json!({}));
        let options = options
            .as_object_mut()
            .ok_or_else(|| "OpenCode Prelay provider options must be an object".to_string())?;
        options.insert(
            "baseURL".to_string(),
            serde_json::Value::String(prelay_base_url(relay_url)),
        );
        options.insert(
            "apiKey".to_string(),
            serde_json::Value::String(endpoint_token.to_string()),
        );
        // 接入点模型清单整体替换，避免接入点删掉模型后配置里留下旧条目。
        if let Some(models_list) = models {
            let models = prelay
                .entry("models")
                .or_insert_with(|| serde_json::json!({}));
            let models = models
                .as_object_mut()
                .ok_or_else(|| "OpenCode Prelay models must be an object".to_string())?;
            models.clear();
            for entry in models_list {
                models.insert(entry.id.clone(), prelay_model_entry(entry));
            }
        }
        if let Some(model) = model {
            let models = prelay
                .entry("models")
                .or_insert_with(|| serde_json::json!({}));
            let models = models
                .as_object_mut()
                .ok_or_else(|| "OpenCode Prelay models must be an object".to_string())?;
            models
                .entry(model.to_string())
                .or_insert_with(|| serde_json::json!({}));
        }
    }

    let contents = serde_json::to_vec_pretty(&document)
        .map_err(|error| format!("OpenCode config cannot be serialized: {error}"))?;
    let mut changed = write_text_if_changed(&config_path, &contents)?;
    changed |= write_text_if_changed(
        &config_path.with_file_name("AGENTS.md"),
        settings.rules.as_deref().unwrap_or_default().as_bytes(),
    )?;
    Ok(changed)
}

fn clear_opencode_connection(root: &mut serde_json::Map<String, serde_json::Value>) {
    if let Some(providers) = root
        .get_mut("provider")
        .and_then(|value| value.as_object_mut())
    {
        providers.remove("prelay");
        if providers.is_empty() {
            root.remove("provider");
        }
    }
    let model_is_prelay = root
        .get("model")
        .and_then(|value| value.as_str())
        .is_some_and(|model| model.starts_with("prelay/"));
    if model_is_prelay {
        root.remove("model");
    }
}

/// OpenCode 的模型条目：目前只写显示名。
///
/// schema 里 `limit` 要求同时给出 `context` 和 `output`，而目录只声明上下文窗口，
/// 缺输出上限时宁可不写，也不填推测值。
fn prelay_model_entry(model: &CatalogLanguageModelResponse) -> serde_json::Value {
    let mut entry = serde_json::Map::new();
    let display_name = model.display_name.trim();
    if !display_name.is_empty() {
        entry.insert(
            "name".to_string(),
            serde_json::Value::String(display_name.to_string()),
        );
    }
    serde_json::Value::Object(entry)
}

pub(super) fn read_opencode_settings(home: &Path) -> OpenCodeSettings {
    let config = read_jsonc(&opencode_configuration_path(home));
    let model = json_string(config.as_ref(), &["model"])
        .and_then(|model| model.strip_prefix("prelay/").map(str::to_string));
    OpenCodeSettings {
        base_url: json_string(
            config.as_ref(),
            &["provider", "prelay", "options", "baseURL"],
        ),
        endpoint_token: json_string(
            config.as_ref(),
            &["provider", "prelay", "options", "apiKey"],
        ),
        model,
        rules: read_optional_text(&opencode_configuration_path(home).with_file_name("AGENTS.md")),
    }
}

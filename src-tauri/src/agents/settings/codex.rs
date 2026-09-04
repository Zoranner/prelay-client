use std::path::Path;

use serde_json::{json, Value};
use toml_edit::{value, DocumentMut, Item, Table};

use super::{
    document::{
        json_string, read_json, read_json_document, read_optional_text, read_toml,
        read_toml_document, set_bool, set_item, set_table_bool, set_table_integer,
        set_table_string, table_mut, toml_bool, toml_integer, toml_string, toml_web_search,
        write_text,
    },
    CodexConnection, CodexFeatures, CodexSettings,
};
use prelay_protocol::CatalogLanguageModelResponse;

pub(super) fn save_codex_settings(
    home: &Path,
    settings: &CodexSettings,
    connection: Option<&CodexConnection>,
) -> Result<(), String> {
    let config_path = home.join(".codex").join("config.toml");
    let mut document = read_toml_document(&config_path)?;
    set_item(&mut document, "model", settings.model.as_deref());
    set_item(
        &mut document,
        "model_reasoning_effort",
        settings.reasoning_effort.as_deref(),
    );
    set_item(
        &mut document,
        "personality",
        settings.personality.as_deref(),
    );
    set_item(&mut document, "sandbox_mode", settings.sandbox.as_deref());
    set_bool(
        &mut document,
        "disable_response_storage",
        settings.disable_response_storage,
    );
    set_item(
        &mut document,
        "web_search",
        settings
            .web_search
            .map(|enabled| if enabled { "live" } else { "disabled" }),
    );

    let agents = table_mut(&mut document, "agents");
    set_table_integer(agents, "max_threads", settings.max_threads);
    set_table_integer(agents, "max_depth", settings.max_depth);
    set_table_integer(
        agents,
        "job_max_runtime_seconds",
        settings.job_max_runtime_seconds,
    );

    let features = table_mut(&mut document, "features");
    set_table_bool(features, "memories", settings.features.memories);
    set_table_bool(features, "goals", settings.features.goals);
    set_table_bool(
        features,
        "workspace_dependencies",
        settings.features.workspace_dependencies,
    );

    let workspace = table_mut(&mut document, "sandbox_workspace_write");
    set_table_bool(workspace, "network_access", settings.network_access);
    let shell = table_mut(&mut document, "shell_environment_policy");
    set_table_string(
        shell,
        "inherit",
        settings.shell_environment_inherit.as_deref(),
    );
    let windows = table_mut(&mut document, "windows");
    set_table_string(windows, "sandbox", settings.windows_sandbox.as_deref());

    apply_codex_connection(home, &mut document, settings.model.as_deref(), connection)?;

    write_text(&config_path, document.to_string().as_bytes())?;
    if let Some(connection) = connection {
        match connection {
            CodexConnection::Prelay { endpoint_token, .. } => {
                write_codex_auth_token(home, endpoint_token)?;
            }
            CodexConnection::Custom { token, .. } if !token.trim().is_empty() => {
                write_codex_auth_token(home, token)?;
            }
            CodexConnection::Custom { .. } => {}
        }
    }
    write_text(
        &home.join(".codex").join("AGENTS.md"),
        settings.rules.as_deref().unwrap_or_default().as_bytes(),
    )
}

fn apply_codex_connection(
    home: &Path,
    document: &mut DocumentMut,
    model: Option<&str>,
    connection: Option<&CodexConnection>,
) -> Result<(), String> {
    let Some(connection) = connection else {
        return Ok(());
    };
    match connection {
        CodexConnection::Prelay { models, .. } => {
            validate_prelay_default_model(model, models)?;
            let path = write_prelay_model_catalog(home, models)?;
            set_item(
                document,
                "model_catalog_json",
                Some(&path.to_string_lossy().replace('\\', "/")),
            );
        }
        CodexConnection::Custom { .. } => {}
    }
    let provider_id = document
        .as_table()
        .get("model_provider")
        .and_then(Item::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("custom")
        .to_string();
    document["model_provider"] = value(&provider_id);

    if !document.as_table().contains_key("model_providers") {
        document["model_providers"] = Item::Table(Table::new());
    }
    let providers = document
        .as_table_mut()
        .get_mut("model_providers")
        .and_then(Item::as_table_mut)
        .ok_or_else(|| "Codex model providers must be a table".to_string())?;
    if !providers.contains_key(&provider_id) {
        providers.insert(&provider_id, Item::Table(Table::new()));
    }
    let provider = providers
        .get_mut(&provider_id)
        .and_then(Item::as_table_mut)
        .ok_or_else(|| "active Codex model provider must be a table".to_string())?;
    match connection {
        CodexConnection::Prelay {
            endpoint_name,
            relay_url,
            ..
        } => {
            provider["name"] = value(endpoint_name);
            provider["base_url"] = value(prelay_base_url(relay_url));
        }
        CodexConnection::Custom { base_url, .. } => {
            provider["name"] = value("Custom");
            provider["base_url"] = value(base_url.trim());
        }
    }
    provider["requires_openai_auth"] = value(true);
    provider["wire_api"] = value("responses");
    provider.remove("experimental_bearer_token");
    provider.remove("env_key");
    Ok(())
}

fn validate_prelay_default_model(
    model: Option<&str>,
    models: &[CatalogLanguageModelResponse],
) -> Result<(), String> {
    if models.is_empty() {
        return if model
            .map(str::trim)
            .filter(|model| !model.is_empty())
            .is_some()
        {
            Err("默认模型不属于所选接入点。".to_string())
        } else {
            Ok(())
        };
    }
    let model = model.map(str::trim).filter(|model| !model.is_empty());
    if model.is_some_and(|model| models.iter().any(|item| item.id == model)) {
        Ok(())
    } else {
        Err("默认模型不属于所选接入点。".to_string())
    }
}

fn write_prelay_model_catalog(
    home: &Path,
    models: &[CatalogLanguageModelResponse],
) -> Result<std::path::PathBuf, String> {
    let catalog = models
        .iter()
        .map(|model| {
            let mut profile = serde_json::to_value(model)
                .map_err(|error| format!("Codex 模型档案无法序列化: {error}"))?;
            profile["slug"] = Value::String(model.id.clone());
            Ok::<Value, String>(profile)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let contents = serde_json::to_vec_pretty(&json!({ "models": catalog }))
        .map_err(|error| format!("Codex 模型目录无法序列化: {error}"))?;
    let path = home.join(".codex").join("models.json");
    write_text(&path, &contents)?;
    Ok(path)
}

fn write_codex_auth_token(home: &Path, token: &str) -> Result<(), String> {
    let path = home.join(".codex").join("auth.json");
    let mut document = read_json_document(&path, "Codex auth")?;
    let root = document
        .as_object_mut()
        .ok_or_else(|| "Codex auth root must be an object".to_string())?;
    root.insert(
        "OPENAI_API_KEY".to_string(),
        serde_json::Value::String(token.to_string()),
    );
    let contents = serde_json::to_vec_pretty(&document)
        .map_err(|error| format!("Codex auth cannot be serialized: {error}"))?;
    write_text(&path, &contents)
}

pub(super) fn prelay_base_url(relay_url: &str) -> String {
    let relay_url = relay_url.trim_end_matches('/');
    if relay_url.ends_with("/v1") {
        relay_url.to_string()
    } else {
        format!("{relay_url}/v1")
    }
}

pub(super) fn read_codex_settings(home: &Path) -> CodexSettings {
    let config = read_toml(&home.join(".codex").join("config.toml"));
    let provider_id = toml_string(config.as_ref(), &["model_provider"]);
    let endpoint_name = provider_id.as_deref().and_then(|provider_id| {
        toml_string(config.as_ref(), &["model_providers", provider_id, "name"])
    });
    let base_url = provider_id.as_deref().and_then(|provider_id| {
        toml_string(
            config.as_ref(),
            &["model_providers", provider_id, "base_url"],
        )
    });
    let custom_token = json_string(
        read_json(&home.join(".codex").join("auth.json")).as_ref(),
        &["OPENAI_API_KEY"],
    );
    CodexSettings {
        endpoint_name,
        base_url,
        custom_token,
        model: toml_string(config.as_ref(), &["model"]),
        reasoning_effort: toml_string(config.as_ref(), &["model_reasoning_effort"]),
        personality: toml_string(config.as_ref(), &["personality"]),
        web_search: toml_web_search(config.as_ref()),
        sandbox: toml_string(config.as_ref(), &["sandbox_mode"]),
        disable_response_storage: toml_bool(config.as_ref(), &["disable_response_storage"]),
        max_threads: toml_integer(config.as_ref(), &["agents", "max_threads"]),
        max_depth: toml_integer(config.as_ref(), &["agents", "max_depth"]),
        job_max_runtime_seconds: toml_integer(
            config.as_ref(),
            &["agents", "job_max_runtime_seconds"],
        ),
        network_access: toml_bool(
            config.as_ref(),
            &["sandbox_workspace_write", "network_access"],
        ),
        shell_environment_inherit: toml_string(
            config.as_ref(),
            &["shell_environment_policy", "inherit"],
        ),
        windows_sandbox: toml_string(config.as_ref(), &["windows", "sandbox"]),
        features: CodexFeatures {
            memories: toml_bool(config.as_ref(), &["features", "memories"]),
            goals: toml_bool(config.as_ref(), &["features", "goals"]),
            workspace_dependencies: toml_bool(
                config.as_ref(),
                &["features", "workspace_dependencies"],
            ),
        },
        rules: read_optional_text(&home.join(".codex").join("AGENTS.md")),
    }
}

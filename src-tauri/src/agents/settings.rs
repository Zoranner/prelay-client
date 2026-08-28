use std::{fs, io::Write, path::Path};

use atomic_write_file::AtomicWriteFile;
use serde::{Deserialize, Serialize};
use toml_edit::{value, DocumentMut, Item, Table};

use crate::agents::{opencode_configuration_path, AgentClient};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", tag = "client", content = "settings")]
pub enum AgentSettings {
    CodexCli(CodexSettings),
    #[serde(rename = "chatgpt")]
    ChatGpt(ChatGptSettings),
    OpenCode(OpenCodeSettings),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ChatGptSettings(pub CodexSettings);

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(rename_all_fields = "camelCase")]
#[serde(tag = "kind")]
pub enum CodexConnection {
    Prelay {
        endpoint_id: String,
        endpoint_name: String,
        relay_url: String,
        endpoint_token: String,
    },
    Custom {
        base_url: String,
        token: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(rename_all_fields = "camelCase")]
#[serde(tag = "kind")]
pub enum OpenCodeConnection {
    Prelay {
        relay_url: String,
        endpoint_token: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", tag = "client", content = "connection")]
pub enum AgentConnection {
    CodexCli(CodexConnection),
    #[serde(rename = "chatgpt")]
    ChatGpt(CodexConnection),
    OpenCode(OpenCodeConnection),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub personality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_response_storage: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_threads: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_max_runtime_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_access: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell_environment_inherit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows_sandbox: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<String>,
    pub features: CodexFeatures,
}

impl Default for CodexSettings {
    fn default() -> Self {
        Self {
            endpoint_name: None,
            base_url: None,
            custom_token: None,
            model: None,
            reasoning_effort: Some("high".to_string()),
            personality: Some("pragmatic".to_string()),
            web_search: Some(true),
            sandbox: Some("workspace-write".to_string()),
            disable_response_storage: Some(true),
            max_threads: Some(16),
            max_depth: Some(1),
            job_max_runtime_seconds: Some(1800),
            network_access: Some(true),
            shell_environment_inherit: Some("all".to_string()),
            windows_sandbox: Some("unelevated".to_string()),
            rules: Some(String::new()),
            features: CodexFeatures::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexFeatures {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memories: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goals: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_dependencies: Option<bool>,
}

impl Default for CodexFeatures {
    fn default() -> Self {
        Self {
            memories: Some(true),
            goals: Some(true),
            workspace_dependencies: Some(false),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodeSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<String>,
}

pub fn read_user_settings(home: &Path, client: AgentClient) -> AgentSettings {
    match client {
        AgentClient::CodexCli => AgentSettings::CodexCli(read_codex_settings(home)),
        AgentClient::ChatGpt => AgentSettings::ChatGpt(ChatGptSettings(read_codex_settings(home))),
        AgentClient::OpenCode => AgentSettings::OpenCode(read_opencode_settings(home)),
    }
}

pub fn save_user_settings(
    home: &Path,
    settings: &AgentSettings,
    connection: Option<&AgentConnection>,
) -> Result<(), String> {
    match (settings, connection) {
        (AgentSettings::CodexCli(settings), None) => save_codex_settings(home, settings, None),
        (AgentSettings::CodexCli(settings), Some(AgentConnection::CodexCli(connection))) => {
            save_codex_settings(home, settings, Some(connection))
        }
        (AgentSettings::ChatGpt(settings), None) => save_codex_settings(home, &settings.0, None),
        (AgentSettings::ChatGpt(settings), Some(AgentConnection::ChatGpt(connection))) => {
            save_codex_settings(home, &settings.0, Some(connection))
        }
        (AgentSettings::OpenCode(settings), None) => save_opencode_settings(home, settings, None),
        (AgentSettings::OpenCode(settings), Some(AgentConnection::OpenCode(connection))) => {
            save_opencode_settings(home, settings, Some(connection))
        }
        _ => Err("智能体设置与接入配置不匹配".to_string()),
    }
}

fn save_codex_settings(
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

    apply_codex_connection(&mut document, connection)?;

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
    document: &mut DocumentMut,
    connection: Option<&CodexConnection>,
) -> Result<(), String> {
    let Some(connection) = connection else {
        return Ok(());
    };
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

fn prelay_base_url(relay_url: &str) -> String {
    let relay_url = relay_url.trim_end_matches('/');
    if relay_url.ends_with("/v1") {
        relay_url.to_string()
    } else {
        format!("{relay_url}/v1")
    }
}

fn read_codex_settings(home: &Path) -> CodexSettings {
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

fn save_opencode_settings(
    home: &Path,
    settings: &OpenCodeSettings,
    connection: Option<&OpenCodeConnection>,
) -> Result<(), String> {
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

    if let Some(OpenCodeConnection::Prelay {
        relay_url,
        endpoint_token,
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
    write_text(&config_path, &contents)?;
    write_text(
        &config_path.with_file_name("AGENTS.md"),
        settings.rules.as_deref().unwrap_or_default().as_bytes(),
    )
}

fn read_opencode_settings(home: &Path) -> OpenCodeSettings {
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

fn read_toml(path: &Path) -> Option<toml::Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|contents| toml::from_str(&contents).ok())
}

fn read_json(path: &Path) -> Option<serde_json::Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|contents| serde_json::from_str(&contents).ok())
}

fn read_jsonc(path: &Path) -> Option<serde_json::Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|contents| json5::from_str(&contents).ok())
}

fn read_optional_text(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

fn read_toml_document(path: &Path) -> Result<DocumentMut, String> {
    match fs::read_to_string(path) {
        Ok(contents) => contents
            .parse()
            .map_err(|error| format!("Codex config is not valid TOML: {error}")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(DocumentMut::new()),
        Err(error) => Err(format!("Codex config cannot be read: {error}")),
    }
}

fn read_json_document(path: &Path, description: &str) -> Result<serde_json::Value, String> {
    match fs::read_to_string(path) {
        Ok(contents) if contents.trim().is_empty() => Ok(serde_json::json!({})),
        Ok(contents) => serde_json::from_str(&contents)
            .map_err(|error| format!("{description} is not valid JSON: {error}")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(serde_json::json!({})),
        Err(error) => Err(format!("{description} cannot be read: {error}")),
    }
}

fn read_jsonc_document(path: &Path, description: &str) -> Result<serde_json::Value, String> {
    match fs::read_to_string(path) {
        Ok(contents) if contents.trim().is_empty() => Ok(serde_json::json!({})),
        Ok(contents) => json5::from_str(&contents)
            .map_err(|error| format!("{description} is not valid JSONC: {error}")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(serde_json::json!({})),
        Err(error) => Err(format!("{description} cannot be read: {error}")),
    }
}

fn write_text(path: &Path, contents: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| format!("settings directory cannot be created: {error}"))?;
    let mut file = AtomicWriteFile::open(path)
        .map_err(|error| format!("settings file cannot be opened: {error}"))?;
    file.write_all(contents)
        .map_err(|error| format!("settings file cannot be written: {error}"))?;
    file.sync_all()
        .map_err(|error| format!("settings file cannot be synchronized: {error}"))?;
    file.commit()
        .map_err(|error| format!("settings file cannot be committed: {error}"))
}

fn table_mut<'a>(document: &'a mut DocumentMut, key: &str) -> &'a mut Table {
    if !document.as_table().contains_key(key) {
        document[key] = Item::Table(Table::new());
    }
    document[key]
        .as_table_mut()
        .expect("managed Codex settings section must be a table")
}

fn set_item(document: &mut DocumentMut, key: &str, setting: Option<&str>) {
    match setting.filter(|value| !value.trim().is_empty()) {
        Some(setting) => document[key] = value(setting),
        None => {
            document.as_table_mut().remove(key);
        }
    }
}

fn set_bool(document: &mut DocumentMut, key: &str, setting: Option<bool>) {
    match setting {
        Some(setting) => document[key] = value(setting),
        None => {
            document.as_table_mut().remove(key);
        }
    }
}

fn set_table_string(table: &mut Table, key: &str, setting: Option<&str>) {
    match setting.filter(|value| !value.trim().is_empty()) {
        Some(setting) => table[key] = value(setting),
        None => {
            table.remove(key);
        }
    }
}

fn set_table_bool(table: &mut Table, key: &str, setting: Option<bool>) {
    match setting {
        Some(setting) => table[key] = value(setting),
        None => {
            table.remove(key);
        }
    }
}

fn set_table_integer(table: &mut Table, key: &str, setting: Option<u64>) {
    match setting {
        Some(setting) => table[key] = value(i64::try_from(setting).unwrap_or(i64::MAX)),
        None => {
            table.remove(key);
        }
    }
}

fn toml_value<'a>(value: Option<&'a toml::Value>, path: &[&str]) -> Option<&'a toml::Value> {
    let mut current = value?;
    for segment in path {
        current = current.get(*segment)?;
    }
    Some(current)
}

fn toml_string(value: Option<&toml::Value>, path: &[&str]) -> Option<String> {
    toml_value(value, path)
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
}

fn toml_bool(value: Option<&toml::Value>, path: &[&str]) -> Option<bool> {
    toml_value(value, path).and_then(toml::Value::as_bool)
}

fn toml_integer(value: Option<&toml::Value>, path: &[&str]) -> Option<u64> {
    toml_value(value, path)
        .and_then(toml::Value::as_integer)
        .and_then(|number| u64::try_from(number).ok())
}

fn toml_web_search(value: Option<&toml::Value>) -> Option<bool> {
    let setting = toml_value(value, &["web_search"])?;
    setting.as_bool().or_else(|| {
        setting
            .as_str()
            .map(|mode| !matches!(mode, "disabled" | "off" | "false"))
    })
}

fn json_value<'a>(
    value: Option<&'a serde_json::Value>,
    path: &[&str],
) -> Option<&'a serde_json::Value> {
    let mut current = value?;
    for segment in path {
        current = current.get(*segment)?;
    }
    Some(current)
}

fn json_string(value: Option<&serde_json::Value>, path: &[&str]) -> Option<String> {
    json_value(value, path)
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
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
            serde_json::from_str(&fs::read_to_string(codex_root.join("auth.json")).unwrap())
                .unwrap();
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
            serde_json::from_str(&fs::read_to_string(codex_root.join("auth.json")).unwrap())
                .unwrap();
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
}

use std::{fs, io::Write, path::Path};

use atomic_write_file::AtomicWriteFile;
use serde::{Deserialize, Serialize};
use toml_edit::{value, DocumentMut, Item, Table};

use crate::agents::AgentClient;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSettingsSnapshot {
    pub codex: CodexSettings,
    pub claude_code: ClaudeCodeSettings,
}

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
pub enum ClaudeCodeConnection {
    Prelay {
        relay_url: String,
        endpoint_token: String,
    },
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
    pub features: CodexFeatures,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<String>,
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
            features: CodexFeatures::default(),
            rules: Some(String::new()),
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opus_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sonnet_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub haiku_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subagent_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<String>,
}

impl Default for ClaudeCodeSettings {
    fn default() -> Self {
        Self {
            base_url: None,
            endpoint_token: None,
            opus_model: None,
            sonnet_model: None,
            haiku_model: None,
            subagent_model: None,
            effort: Some("high".to_string()),
            language: Some("中文".to_string()),
            permission_mode: Some("acceptEdits".to_string()),
            rules: Some(String::new()),
        }
    }
}

pub fn read_user_settings(home: &Path) -> AgentSettingsSnapshot {
    AgentSettingsSnapshot {
        codex: read_codex_settings(home),
        claude_code: read_claude_code_settings(home),
    }
}

pub fn read_or_initialize_user_settings(home: &Path) -> Result<AgentSettingsSnapshot, String> {
    initialize_missing_user_settings(home)?;
    Ok(read_user_settings(home))
}

pub fn initialize_missing_user_settings(home: &Path) -> Result<(), String> {
    let codex_root = home.join(".codex");
    let codex_config = codex_root.join("config.toml");
    if codex_root.is_dir() {
        if !codex_config.exists() {
            save_codex_settings(home, &CodexSettings::default(), None)?;
        } else {
            fill_missing_codex_settings(&codex_config)?;
            let rules_path = codex_root.join("AGENTS.md");
            if !rules_path.exists() {
                write_text(&rules_path, b"")?;
            }
        }
    }

    let claude_root = home.join(".claude");
    let claude_settings = claude_root.join("settings.json");
    if claude_root.is_dir() {
        if !claude_settings.exists() {
            save_claude_code_settings(home, &ClaudeCodeSettings::default(), None)?;
        } else {
            fill_missing_claude_code_settings(&claude_settings)?;
            let rules_path = claude_root.join("CLAUDE.md");
            if !rules_path.exists() {
                write_text(&rules_path, b"")?;
            }
        }
    }

    Ok(())
}

fn fill_missing_codex_settings(config_path: &Path) -> Result<(), String> {
    let mut document = read_toml_document(config_path)?;
    let defaults = CodexSettings::default();
    let mut changed = remove_deprecated_codex_settings(&mut document);

    changed |= set_missing_item(
        &mut document,
        "model_reasoning_effort",
        defaults.reasoning_effort.as_deref(),
    );
    changed |= set_missing_item(
        &mut document,
        "personality",
        defaults.personality.as_deref(),
    );
    changed |= set_missing_item(&mut document, "sandbox_mode", defaults.sandbox.as_deref());
    changed |= set_missing_bool(
        &mut document,
        "disable_response_storage",
        defaults.disable_response_storage,
    );
    changed |= set_missing_item(
        &mut document,
        "web_search",
        defaults
            .web_search
            .map(|enabled| if enabled { "live" } else { "disabled" }),
    );

    changed |= fill_missing_table_integers(
        &mut document,
        "agents",
        [
            ("max_threads", defaults.max_threads),
            ("max_depth", defaults.max_depth),
            ("job_max_runtime_seconds", defaults.job_max_runtime_seconds),
        ],
    );
    changed |= fill_missing_table_bools(
        &mut document,
        "features",
        [
            ("memories", defaults.features.memories),
            ("goals", defaults.features.goals),
            (
                "workspace_dependencies",
                defaults.features.workspace_dependencies,
            ),
        ],
    );
    changed |= fill_missing_table_bools(
        &mut document,
        "sandbox_workspace_write",
        [("network_access", defaults.network_access)],
    );
    changed |= fill_missing_table_strings(
        &mut document,
        "shell_environment_policy",
        [("inherit", defaults.shell_environment_inherit.as_deref())],
    );
    changed |= fill_missing_table_strings(
        &mut document,
        "windows",
        [("sandbox", defaults.windows_sandbox.as_deref())],
    );

    if changed {
        write_text(config_path, document.to_string().as_bytes())?;
    }
    Ok(())
}

fn remove_deprecated_codex_settings(document: &mut DocumentMut) -> bool {
    const DEPRECATED_FEATURES: [&str; 3] =
        ["responses_websockets_v2", "remote_control", "rmcp_client"];

    let Some(features) = document["features"].as_table_mut() else {
        return false;
    };
    let mut changed = false;
    for key in DEPRECATED_FEATURES {
        changed |= features.remove(key).is_some();
    }
    changed
}

fn fill_missing_claude_code_settings(settings_path: &Path) -> Result<(), String> {
    let mut document = read_json_document(settings_path)?;
    let Some(root) = document.as_object_mut() else {
        return Ok(());
    };
    let defaults = ClaudeCodeSettings::default();
    let mut changed = set_missing_json_string(root, "effortLevel", defaults.effort.as_deref())
        | set_missing_json_string(root, "language", defaults.language.as_deref());

    if !root.contains_key("permissions") {
        root.insert(
            "permissions".to_string(),
            serde_json::json!({ "defaultMode": claude_permission_mode("acceptEdits") }),
        );
        changed = true;
    } else if let Some(permissions) = root
        .get_mut("permissions")
        .and_then(serde_json::Value::as_object_mut)
    {
        changed |= set_missing_json_string(
            permissions,
            "defaultMode",
            defaults
                .permission_mode
                .as_deref()
                .map(claude_permission_mode),
        );
    }

    if changed {
        let contents = serde_json::to_vec_pretty(&document)
            .map_err(|error| format!("Claude Code settings cannot be serialized: {error}"))?;
        write_text(settings_path, &contents)?;
    }
    Ok(())
}

pub fn save_user_settings(
    home: &Path,
    client: AgentClient,
    settings: &AgentSettingsSnapshot,
    codex_connection: Option<&CodexConnection>,
    claude_code_connection: Option<&ClaudeCodeConnection>,
) -> Result<(), String> {
    match client {
        AgentClient::Codex => save_codex_settings(home, &settings.codex, codex_connection),
        AgentClient::ClaudeCode => {
            save_claude_code_settings(home, &settings.claude_code, claude_code_connection)
        }
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
    let provider_id = document["model_provider"]
        .as_str()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("custom")
        .to_string();
    document["model_provider"] = value(&provider_id);

    let providers = table_mut(document, "model_providers");
    if !providers.contains_key(&provider_id) {
        providers[&provider_id] = Item::Table(Table::new());
    }
    let provider = providers[&provider_id]
        .as_table_mut()
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
    let mut document = read_json_document(&path)?;
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

fn save_claude_code_settings(
    home: &Path,
    settings: &ClaudeCodeSettings,
    connection: Option<&ClaudeCodeConnection>,
) -> Result<(), String> {
    let settings_path = home.join(".claude").join("settings.json");
    let mut document = read_json_document(&settings_path)?;
    let root = document
        .as_object_mut()
        .ok_or_else(|| "Claude Code settings root must be an object".to_string())?;
    set_json_string(root, "effortLevel", settings.effort.as_deref());
    set_json_string(root, "language", settings.language.as_deref());
    let permissions = root
        .entry("permissions")
        .or_insert_with(|| serde_json::json!({}));
    let permissions = permissions
        .as_object_mut()
        .ok_or_else(|| "Claude Code permissions must be an object".to_string())?;
    set_json_string(
        permissions,
        "defaultMode",
        settings
            .permission_mode
            .as_deref()
            .map(claude_permission_mode),
    );

    if root.contains_key("env")
        || connection.is_some()
        || settings.opus_model.is_some()
        || settings.sonnet_model.is_some()
        || settings.haiku_model.is_some()
        || settings.subagent_model.is_some()
    {
        let env = root.entry("env").or_insert_with(|| serde_json::json!({}));
        let env = env
            .as_object_mut()
            .ok_or_else(|| "Claude Code env must be an object".to_string())?;
        set_json_string(
            env,
            "ANTHROPIC_DEFAULT_OPUS_MODEL",
            settings.opus_model.as_deref(),
        );
        set_json_string(
            env,
            "ANTHROPIC_DEFAULT_SONNET_MODEL",
            settings.sonnet_model.as_deref(),
        );
        set_json_string(
            env,
            "ANTHROPIC_DEFAULT_HAIKU_MODEL",
            settings.haiku_model.as_deref(),
        );
        set_json_string(
            env,
            "CLAUDE_CODE_SUBAGENT_MODEL",
            settings.subagent_model.as_deref(),
        );
        if let Some(ClaudeCodeConnection::Prelay {
            relay_url,
            endpoint_token,
        }) = connection
        {
            let base_url = prelay_base_url(relay_url);
            set_json_string(env, "ANTHROPIC_BASE_URL", Some(&base_url));
            set_json_string(env, "ANTHROPIC_AUTH_TOKEN", Some(endpoint_token));
        }
    }

    let contents = serde_json::to_vec_pretty(&document)
        .map_err(|error| format!("Claude Code settings cannot be serialized: {error}"))?;
    write_text(&settings_path, &contents)?;
    write_text(
        &home.join(".claude").join("CLAUDE.md"),
        settings.rules.as_deref().unwrap_or_default().as_bytes(),
    )
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

fn read_claude_code_settings(home: &Path) -> ClaudeCodeSettings {
    let settings = read_json(&home.join(".claude").join("settings.json"));
    ClaudeCodeSettings {
        base_url: json_string(settings.as_ref(), &["env", "ANTHROPIC_BASE_URL"]),
        endpoint_token: json_string(settings.as_ref(), &["env", "ANTHROPIC_AUTH_TOKEN"]),
        opus_model: json_string(settings.as_ref(), &["env", "ANTHROPIC_DEFAULT_OPUS_MODEL"]),
        sonnet_model: json_string(
            settings.as_ref(),
            &["env", "ANTHROPIC_DEFAULT_SONNET_MODEL"],
        ),
        haiku_model: json_string(settings.as_ref(), &["env", "ANTHROPIC_DEFAULT_HAIKU_MODEL"]),
        subagent_model: json_string(settings.as_ref(), &["env", "CLAUDE_CODE_SUBAGENT_MODEL"]),
        effort: json_string(settings.as_ref(), &["effortLevel"]),
        language: json_string(settings.as_ref(), &["language"]),
        permission_mode: json_permission_mode(settings.as_ref()),
        rules: read_optional_text(&home.join(".claude").join("CLAUDE.md")),
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

fn read_json_document(path: &Path) -> Result<serde_json::Value, String> {
    match fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str(&contents)
            .map_err(|error| format!("Claude Code settings are not valid JSON: {error}")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(serde_json::json!({})),
        Err(error) => Err(format!("Claude Code settings cannot be read: {error}")),
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

fn set_missing_item(document: &mut DocumentMut, key: &str, setting: Option<&str>) -> bool {
    if document.as_table().contains_key(key) {
        return false;
    }
    let Some(setting) = setting.filter(|value| !value.trim().is_empty()) else {
        return false;
    };
    document[key] = value(setting);
    true
}

fn set_bool(document: &mut DocumentMut, key: &str, setting: Option<bool>) {
    match setting {
        Some(setting) => document[key] = value(setting),
        None => {
            document.as_table_mut().remove(key);
        }
    }
}

fn set_missing_bool(document: &mut DocumentMut, key: &str, setting: Option<bool>) -> bool {
    if document.as_table().contains_key(key) {
        return false;
    }
    let Some(setting) = setting else {
        return false;
    };
    document[key] = value(setting);
    true
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

fn table_mut_if_missing<'a>(
    document: &'a mut DocumentMut,
    key: &str,
    changed: &mut bool,
) -> Option<&'a mut Table> {
    if !document.as_table().contains_key(key) {
        document[key] = Item::Table(Table::new());
        *changed = true;
    }
    document[key].as_table_mut()
}

fn fill_missing_table_integers<const N: usize>(
    document: &mut DocumentMut,
    table_key: &str,
    settings: [(&str, Option<u64>); N],
) -> bool {
    let mut changed = false;
    let Some(table) = table_mut_if_missing(document, table_key, &mut changed) else {
        return false;
    };
    for (key, setting) in settings {
        if !table.contains_key(key) {
            if let Some(setting) = setting {
                table[key] = value(i64::try_from(setting).unwrap_or(i64::MAX));
                changed = true;
            }
        }
    }
    changed
}

fn fill_missing_table_bools<const N: usize>(
    document: &mut DocumentMut,
    table_key: &str,
    settings: [(&str, Option<bool>); N],
) -> bool {
    let mut changed = false;
    let Some(table) = table_mut_if_missing(document, table_key, &mut changed) else {
        return false;
    };
    for (key, setting) in settings {
        if !table.contains_key(key) {
            if let Some(setting) = setting {
                table[key] = value(setting);
                changed = true;
            }
        }
    }
    changed
}

fn fill_missing_table_strings<const N: usize>(
    document: &mut DocumentMut,
    table_key: &str,
    settings: [(&str, Option<&str>); N],
) -> bool {
    let mut changed = false;
    let Some(table) = table_mut_if_missing(document, table_key, &mut changed) else {
        return false;
    };
    for (key, setting) in settings {
        if !table.contains_key(key) {
            if let Some(setting) = setting.filter(|value| !value.trim().is_empty()) {
                table[key] = value(setting);
                changed = true;
            }
        }
    }
    changed
}

fn set_json_string(
    object: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    setting: Option<&str>,
) {
    match setting.filter(|value| !value.trim().is_empty()) {
        Some(setting) => {
            object.insert(
                key.to_string(),
                serde_json::Value::String(setting.to_string()),
            );
        }
        None => {
            object.remove(key);
        }
    }
}

fn set_missing_json_string(
    object: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    setting: Option<&str>,
) -> bool {
    if object.contains_key(key) {
        return false;
    }
    let Some(setting) = setting.filter(|value| !value.trim().is_empty()) else {
        return false;
    };
    object.insert(
        key.to_string(),
        serde_json::Value::String(setting.to_string()),
    );
    true
}

fn claude_permission_mode(mode: &str) -> &str {
    match mode {
        "acceptEdits" => "acceptEdits",
        "auto" => "bypassPermissions",
        _ => "ask",
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

fn json_permission_mode(value: Option<&serde_json::Value>) -> Option<String> {
    match json_string(value, &["permissions", "defaultMode"])?.as_str() {
        "acceptEdits" => Some("acceptEdits".to_string()),
        "bypassPermissions" => Some("auto".to_string()),
        _ => Some("manual".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{save_claude_code_settings, ClaudeCodeConnection, ClaudeCodeSettings};

    #[test]
    fn saves_claude_code_connection_and_model_aliases() {
        let directory = tempdir().unwrap();
        let claude_root = directory.path().join(".claude");
        fs::create_dir_all(&claude_root).unwrap();
        fs::write(claude_root.join("settings.json"), "{}").unwrap();

        let settings = ClaudeCodeSettings {
            opus_model: Some("opus-model".to_string()),
            sonnet_model: Some("sonnet-model".to_string()),
            haiku_model: Some("haiku-model".to_string()),
            subagent_model: Some("subagent-model".to_string()),
            ..Default::default()
        };
        let connection = ClaudeCodeConnection::Prelay {
            relay_url: "https://relay.example.test/".to_string(),
            endpoint_token: "endpoint-token".to_string(),
        };

        save_claude_code_settings(directory.path(), &settings, Some(&connection)).unwrap();

        let saved = fs::read_to_string(claude_root.join("settings.json")).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&saved).unwrap();
        assert_eq!(
            settings["env"]["ANTHROPIC_BASE_URL"],
            "https://relay.example.test/v1"
        );
        assert_eq!(settings["env"]["ANTHROPIC_AUTH_TOKEN"], "endpoint-token");
        assert_eq!(
            settings["env"]["ANTHROPIC_DEFAULT_OPUS_MODEL"],
            "opus-model"
        );
        assert_eq!(
            settings["env"]["ANTHROPIC_DEFAULT_SONNET_MODEL"],
            "sonnet-model"
        );
        assert_eq!(
            settings["env"]["ANTHROPIC_DEFAULT_HAIKU_MODEL"],
            "haiku-model"
        );
        assert_eq!(
            settings["env"]["CLAUDE_CODE_SUBAGENT_MODEL"],
            "subagent-model"
        );
    }
}

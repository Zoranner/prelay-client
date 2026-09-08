use std::path::Path;

use super::{
    document::{json_string, read_json, read_json_document, read_optional_text, write_text},
    ClaudeCodeConnection, ClaudeCodeSettings,
};
use crate::agents::claude_code_configuration_path;

pub(super) fn save_claude_code_settings(
    home: &Path,
    settings: &ClaudeCodeSettings,
    connection: Option<&ClaudeCodeConnection>,
) -> Result<(), String> {
    let config_path = claude_code_configuration_path(home);
    let mut document = read_json_document(&config_path, "Claude Code config")?;
    let root = document
        .as_object_mut()
        .ok_or_else(|| "Claude Code config root must be an object".to_string())?;
    let env = root.entry("env").or_insert_with(|| serde_json::json!({}));
    let env = env
        .as_object_mut()
        .ok_or_else(|| "Claude Code env must be an object".to_string())?;

    if let Some(ClaudeCodeConnection::Prelay {
        relay_url,
        endpoint_token,
    }) = connection
    {
        env.insert(
            "ANTHROPIC_BASE_URL".to_string(),
            serde_json::Value::String(relay_url.trim_end_matches('/').to_string()),
        );
        env.insert(
            "ANTHROPIC_AUTH_TOKEN".to_string(),
            serde_json::Value::String(endpoint_token.clone()),
        );
    }
    set_env(env, "ANTHROPIC_MODEL", settings.model.as_deref());

    let contents = serde_json::to_vec_pretty(&document)
        .map_err(|error| format!("Claude Code config cannot be serialized: {error}"))?;
    write_text(&config_path, &contents)?;
    write_text(
        &config_path.with_file_name("CLAUDE.md"),
        settings.rules.as_deref().unwrap_or_default().as_bytes(),
    )
}

fn set_env(env: &mut serde_json::Map<String, serde_json::Value>, key: &str, value: Option<&str>) {
    match value.filter(|value| !value.trim().is_empty()) {
        Some(value) => {
            env.insert(
                key.to_string(),
                serde_json::Value::String(value.to_string()),
            );
        }
        None => {
            env.remove(key);
        }
    }
}

pub(super) fn read_claude_code_settings(home: &Path) -> ClaudeCodeSettings {
    let path = claude_code_configuration_path(home);
    let config = read_json(&path);
    ClaudeCodeSettings {
        base_url: json_string(config.as_ref(), &["env", "ANTHROPIC_BASE_URL"]),
        endpoint_token: json_string(config.as_ref(), &["env", "ANTHROPIC_AUTH_TOKEN"]),
        model: json_string(config.as_ref(), &["env", "ANTHROPIC_MODEL"]),
        rules: read_optional_text(&path.with_file_name("CLAUDE.md")),
    }
}

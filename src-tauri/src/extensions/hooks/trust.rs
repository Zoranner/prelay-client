use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use toml_edit::{value, DocumentMut, Item, Table};

use crate::relay::client::ClientError;

use super::super::host_state::{HostState, PluginArtifact, PluginState};
use super::{
    hooks_path,
    merge::{hash_review, read_hook_document, review_item, storage_error},
};

const DEFAULT_ADDITIONAL_CONTEXT_LIMIT: u64 = 2_500;
const DEFAULT_TIMEOUT_SECONDS: u64 = 600;
const SESSION_END_DEFAULT_TIMEOUT_SECONDS: u64 = 1;
const SESSION_END_MAX_TIMEOUT_SECONDS: u64 = 3;

/// Collects the Codex trust entries that belong to hooks currently installed by
/// Prelay. The returned key is Codex's persisted hook-state key and the value
/// is the normalized hook hash Codex expects in `trusted_hash`.
pub(crate) fn entries(
    home: &Path,
    state: &HostState,
) -> Result<BTreeMap<String, String>, ClientError> {
    let managed = managed_hashes(state);
    if managed.is_empty() {
        return Ok(BTreeMap::new());
    }
    let hooks_path = hooks_path(home);
    let document = read_hook_document(&hooks_path)?;
    let Some(hooks) = document.get("hooks").and_then(Value::as_object) else {
        return Ok(BTreeMap::new());
    };
    let mut entries = BTreeMap::new();
    for (event, groups) in hooks {
        let Some(event_label) = event_key_label(event) else {
            continue;
        };
        let Some(groups) = groups.as_array() else {
            continue;
        };
        for (group_index, group) in groups.iter().enumerate() {
            let Some(group) = group.as_object() else {
                continue;
            };
            let matcher = group.get("matcher").and_then(Value::as_str);
            let Some(handlers) = group.get("hooks").and_then(Value::as_array) else {
                continue;
            };
            for (handler_index, handler) in handlers.iter().enumerate() {
                let Ok(review) = review_item(event, matcher, handler) else {
                    continue;
                };
                let Ok(handler_hash) = hash_review(&review) else {
                    continue;
                };
                if !managed.contains(&handler_hash) {
                    continue;
                }
                let trusted_hash = codex_hook_trust_hash(event, matcher, handler)?;
                let key = format!(
                    "{}:{}:{group_index}:{handler_index}",
                    hooks_path.display(),
                    event_label
                );
                entries.insert(key, trusted_hash);
            }
        }
    }
    Ok(entries)
}

/// Persists only Prelay-owned trust entries. Existing entries keep their
/// `enabled` or `trusted_content_hash` fields; user-owned hook state is not
/// touched.
pub(crate) fn write(
    home: &Path,
    entries: &BTreeMap<String, String>,
    stale: &BTreeMap<String, String>,
) -> Result<(), ClientError> {
    if entries.is_empty() && stale.is_empty() {
        return Ok(());
    }
    let config_path = home.join(".codex").join("config.toml");
    let contents = match fs::read_to_string(&config_path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(storage_error(error)),
    };
    let mut document = contents.parse::<DocumentMut>().map_err(|error| {
        ClientError::new(
            "local_extensions_error",
            format!("Codex config.toml 无法解析：{error}"),
        )
    })?;
    let hooks = document
        .as_table_mut()
        .entry("hooks")
        .or_insert(Item::Table(Table::new()));
    let hooks = hooks.as_table_mut().ok_or_else(|| {
        ClientError::new(
            "local_extensions_error",
            "Codex config.toml 的 hooks 不是表。",
        )
    })?;
    let state = hooks.entry("state").or_insert(Item::Table(Table::new()));
    let state = state.as_table_mut().ok_or_else(|| {
        ClientError::new(
            "local_extensions_error",
            "Codex config.toml 的 hooks.state 不是表。",
        )
    })?;

    for (key, expected_hash) in stale {
        let matches = state
            .get(key)
            .and_then(Item::as_table)
            .and_then(|table| table.get("trusted_hash"))
            .and_then(Item::as_value)
            .and_then(toml_edit::Value::as_str)
            == Some(expected_hash.as_str());
        if matches {
            state.remove(key);
        }
    }
    for (key, trusted_hash) in entries {
        let mut entry = match state.get(key) {
            Some(Item::Table(table)) => table.clone(),
            _ => Table::new(),
        };
        entry.insert("trusted_hash", value(trusted_hash));
        state.insert(key, Item::Table(entry));
    }

    super::super::atomic_write(&config_path, document.to_string().as_bytes())
}

fn managed_hashes(state: &HostState) -> BTreeSet<String> {
    // Host state is the ownership authority. Matching by the recorded handler
    // hash keeps trust scoped to hooks Prelay actually installed, including
    // packages whose commands do not point into the Prelay cache directory.
    state
        .plugins
        .iter()
        .filter(|plugin| {
            plugin.state == PluginState::Installed && !state.is_conflicted(&plugin.name)
        })
        .flat_map(|plugin| plugin.artifacts.iter())
        .filter_map(|artifact| match artifact {
            PluginArtifact::Hook { handler_hash, .. } => Some(handler_hash.clone()),
            _ => None,
        })
        .collect()
}

fn codex_hook_trust_hash(
    event: &str,
    matcher: Option<&str>,
    handler: &Value,
) -> Result<String, ClientError> {
    let handler = handler.as_object().ok_or_else(|| {
        ClientError::new(
            "invalid_response",
            format!("Hook 事件 {event} 的 handler 无效。"),
        )
    })?;
    let kind = handler
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| ClientError::new("invalid_response", "Hook handler 缺少 type。"))?;
    if kind != "command" {
        return Err(ClientError::new(
            "extension_install_unsupported",
            "当前只支持 command Hook。",
        ));
    }
    let command = required_string(handler, "command")?;
    let command_windows = optional_string(handler, "commandWindows")?;
    let command = if cfg!(windows) {
        command_windows.as_deref().unwrap_or(command)
    } else {
        command
    };
    let timeout_sec = normalize_timeout(event, optional_u64(handler, "timeout")?);
    let is_async = optional_bool(handler, "async")?.unwrap_or(false);
    let status_message = optional_string(handler, "statusMessage")?;
    let additional_context_limit = if supports_additional_context(event) {
        optional_u64(handler, "additionalContextLimit")?
            .filter(|limit| *limit != DEFAULT_ADDITIONAL_CONTEXT_LIMIT)
    } else {
        None
    };
    let event_label = event_key_label(event)
        .ok_or_else(|| ClientError::new("invalid_response", format!("Hook 事件名无效：{event}")))?;

    let mut identity = Map::new();
    identity.insert(
        "event_name".to_string(),
        Value::String(event_label.to_string()),
    );
    if let Some(matcher) = matcher_for_event(event, matcher) {
        identity.insert("matcher".to_string(), Value::String(matcher.to_string()));
    }
    let mut normalized = Map::new();
    normalized.insert("type".to_string(), Value::String("command".to_string()));
    normalized.insert("command".to_string(), Value::String(command.to_string()));
    normalized.insert("timeout".to_string(), Value::Number(timeout_sec.into()));
    normalized.insert("async".to_string(), Value::Bool(is_async));
    if let Some(status_message) = status_message {
        normalized.insert("statusMessage".to_string(), Value::String(status_message));
    }
    if let Some(additional_context_limit) = additional_context_limit {
        normalized.insert(
            "additionalContextLimit".to_string(),
            Value::Number(additional_context_limit.into()),
        );
    }
    identity.insert(
        "hooks".to_string(),
        Value::Array(vec![Value::Object(normalized)]),
    );
    let mut identity = Value::Object(identity);
    sort_json(&mut identity);
    let bytes = serde_json::to_vec(&identity).map_err(|error| {
        ClientError::new(
            "local_extensions_error",
            format!("无法计算 Codex Hook 信任哈希：{error}"),
        )
    })?;
    let digest = Sha256::digest(&bytes);
    Ok(format!(
        "sha256:{}",
        digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    ))
}

fn required_string<'a>(handler: &'a Map<String, Value>, key: &str) -> Result<&'a str, ClientError> {
    handler
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            ClientError::new(
                "invalid_response",
                format!("Hook handler 的 {key} 不是有效字符串。"),
            )
        })
}

fn optional_string(handler: &Map<String, Value>, key: &str) -> Result<Option<String>, ClientError> {
    match handler.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(ClientError::new(
            "invalid_response",
            format!("Hook handler 的 {key} 不是字符串。"),
        )),
    }
}

fn optional_u64(handler: &Map<String, Value>, key: &str) -> Result<Option<u64>, ClientError> {
    match handler.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value.as_u64().map(Some).ok_or_else(|| {
            ClientError::new(
                "invalid_response",
                format!("Hook handler 的 {key} 不是非负整数。"),
            )
        }),
    }
}

fn optional_bool(handler: &Map<String, Value>, key: &str) -> Result<Option<bool>, ClientError> {
    match handler.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value.as_bool().map(Some).ok_or_else(|| {
            ClientError::new(
                "invalid_response",
                format!("Hook handler 的 {key} 不是布尔值。"),
            )
        }),
    }
}

fn normalize_timeout(event: &str, timeout_sec: Option<u64>) -> u64 {
    match event {
        "SessionEnd" | "Interrupt" => timeout_sec
            .unwrap_or(SESSION_END_DEFAULT_TIMEOUT_SECONDS)
            .clamp(
                SESSION_END_DEFAULT_TIMEOUT_SECONDS,
                SESSION_END_MAX_TIMEOUT_SECONDS,
            ),
        _ => timeout_sec.unwrap_or(DEFAULT_TIMEOUT_SECONDS).max(1),
    }
}

fn matcher_for_event<'a>(event: &str, matcher: Option<&'a str>) -> Option<&'a str> {
    match event {
        "PreModelRequest" | "PostModelRequest" | "UserPromptSubmit" | "Stop" | "Interrupt" => None,
        _ => matcher,
    }
}

fn supports_additional_context(event: &str) -> bool {
    matches!(
        event,
        "PreToolUse" | "PostToolUse" | "SessionStart" | "UserPromptSubmit" | "SubagentStart"
    )
}

fn event_key_label(event: &str) -> Option<&'static str> {
    match event {
        "PreModelRequest" => Some("pre_model_request"),
        "PostModelRequest" => Some("post_model_request"),
        "PreToolUse" => Some("pre_tool_use"),
        "PermissionRequest" => Some("permission_request"),
        "PostToolUse" => Some("post_tool_use"),
        "PreCompact" => Some("pre_compact"),
        "PostCompact" => Some("post_compact"),
        "SessionStart" => Some("session_start"),
        "SessionEnd" => Some("session_end"),
        "UserPromptSubmit" => Some("user_prompt_submit"),
        "SubagentStart" => Some("subagent_start"),
        "SubagentStop" => Some("subagent_stop"),
        "Stop" => Some("stop"),
        "Interrupt" => Some("interrupt"),
        _ => None,
    }
}

fn sort_json(value: &mut Value) {
    match value {
        Value::Object(map) => {
            let mut entries = std::mem::take(map).into_iter().collect::<Vec<_>>();
            entries.sort_by(|(left, _), (right, _)| left.cmp(right));
            for (_, child) in &mut entries {
                sort_json(child);
            }
            map.extend(entries);
        }
        Value::Array(items) => {
            for item in items {
                sort_json(item);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
#[path = "trust_tests.rs"]
mod tests;

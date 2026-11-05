use std::path::Path;

use prelay_protocol::{ExtensionFile, HOOKS_MANIFEST_PATH, HOOK_ROOT_PLACEHOLDER};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::{extensions::decode_extension_file, relay::client::ClientError};

use super::super::model::HookReviewItem;

const HOOK_EVENTS: [&str; 12] = [
    "PreToolUse",
    "PermissionRequest",
    "PostToolUse",
    "PreCompact",
    "PostCompact",
    "UserPromptSubmit",
    "SubagentStop",
    "Stop",
    "Interrupt",
    "SessionStart",
    "SubagentStart",
    "SessionEnd",
];

pub(super) struct IncomingHook {
    pub event: String,
    pub matcher: Option<String>,
    pub handler: Value,
    pub hash: String,
    pub review: HookReviewItem,
}

pub(super) struct DocumentHook {
    pub(super) event: String,
    pub(super) matcher: Option<String>,
    pub(super) hash: String,
    pub(super) command: String,
}

pub(super) fn collect_incoming_hooks(
    files: &[ExtensionFile],
    root: Option<&Path>,
) -> Result<Vec<IncomingHook>, ClientError> {
    let mut definitions = files.iter().filter(|file| file.path == HOOKS_MANIFEST_PATH);
    let file = definitions.next().ok_or_else(|| {
        ClientError::new("invalid_response", "Hook 安装包缺少 hooks/hooks.json。")
    })?;
    if definitions.next().is_some() {
        return Err(ClientError::new(
            "invalid_response",
            "Hook 安装包包含多个 hooks/hooks.json。",
        ));
    }
    let content = decode_extension_file(file)?;
    let document: Value = serde_json::from_slice(&content).map_err(|error| {
        ClientError::new(
            "invalid_response",
            format!("Hook 文件不是有效的 JSON：{error}"),
        )
    })?;
    let Some(hooks) = document.get("hooks").and_then(Value::as_object) else {
        return Err(ClientError::new(
            "invalid_response",
            "Hook 文件缺少 hooks 对象。",
        ));
    };
    let mut incoming = Vec::new();
    for (event, groups) in hooks {
        if !HOOK_EVENTS.contains(&event.as_str()) {
            return Err(ClientError::new(
                "invalid_response",
                format!("Hook 事件名无效：{event}"),
            ));
        }
        let Some(groups) = groups.as_array() else {
            return Err(ClientError::new(
                "invalid_response",
                format!("Hook 事件 {event} 的 matcher 列表无效。"),
            ));
        };
        for group in groups {
            let Some(group) = group.as_object() else {
                return Err(ClientError::new(
                    "invalid_response",
                    format!("Hook 事件 {event} 的 matcher 组无效。"),
                ));
            };
            let matcher = group
                .get("matcher")
                .map(|value| {
                    value.as_str().map(str::to_string).ok_or_else(|| {
                        ClientError::new(
                            "invalid_response",
                            format!("Hook 事件 {event} 的 matcher 不是字符串。"),
                        )
                    })
                })
                .transpose()?;
            let Some(handlers) = group.get("hooks").and_then(Value::as_array) else {
                return Err(ClientError::new(
                    "invalid_response",
                    format!("Hook 事件 {event} 的 hooks 列表无效。"),
                ));
            };
            for handler in handlers {
                let handler = expand_handler(handler, root)?;
                let review = review_item(event, matcher.as_deref(), &handler)?;
                incoming.push(IncomingHook {
                    event: event.clone(),
                    matcher: matcher.clone(),
                    handler: handler.clone(),
                    hash: handler_hash(event, matcher.as_deref(), &handler)?,
                    review,
                });
            }
        }
    }
    Ok(incoming)
}

fn expand_handler(handler: &Value, root: Option<&Path>) -> Result<Value, ClientError> {
    let mut expanded = handler.clone();
    let Some(object) = expanded.as_object_mut() else {
        return Err(ClientError::new("invalid_response", "Hook handler 无效。"));
    };
    for key in ["command", "commandWindows"] {
        let Some(value) = object.get_mut(key) else {
            continue;
        };
        let Some(text) = value.as_str() else {
            return Err(ClientError::new(
                "invalid_response",
                format!("Hook handler 的 {key} 不是字符串。"),
            ));
        };
        if text.contains("${PLUGIN_ROOT}") || text.contains("${CLAUDE_PLUGIN_ROOT}") {
            return Err(ClientError::new(
                "extension_install_unsupported",
                "Prelay 包不能使用 Codex 原生插件的目录变量。",
            ));
        }
        if text.trim().is_empty() {
            return Err(ClientError::new(
                "invalid_response",
                format!("Hook handler 的 {key} 为空。"),
            ));
        }
        if let Some(root) = root {
            *value =
                Value::String(text.replace(HOOK_ROOT_PLACEHOLDER, &root.display().to_string()));
        }
    }
    Ok(expanded)
}

pub(super) fn read_hook_document(path: &Path) -> Result<Value, ClientError> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(serde_json::json!({ "hooks": {} }));
        }
        Err(error) => return Err(storage_error(error)),
    };
    let document: Value = serde_json::from_slice(&bytes).map_err(|error| {
        ClientError::new(
            "local_extensions_error",
            format!("Codex hooks.json 不是有效的 JSON：{error}"),
        )
    })?;
    if !document.is_object() {
        return Err(ClientError::new(
            "local_extensions_error",
            "Codex hooks.json 的根节点不是对象。",
        ));
    }
    if let Some(hooks) = document.get("hooks") {
        validate_hook_document_shape(hooks)?;
    }
    Ok(document)
}

pub(super) fn hooks_object_mut(
    document: &mut Value,
) -> Result<&mut Map<String, Value>, ClientError> {
    let object = document.as_object_mut().ok_or_else(|| {
        ClientError::new(
            "local_extensions_error",
            "Codex hooks.json 的根节点不是对象。",
        )
    })?;
    if !object.contains_key("hooks") {
        object.insert("hooks".to_string(), Value::Object(Map::new()));
    }
    object
        .get_mut("hooks")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| {
            ClientError::new(
                "local_extensions_error",
                "Codex hooks.json 的 hooks 不是对象。",
            )
        })
}

pub(super) fn remove_hash(hooks: &mut Map<String, Value>, hash: &str) {
    let events = hooks.keys().cloned().collect::<Vec<_>>();
    for event in events {
        let Some(groups) = hooks.get_mut(&event).and_then(Value::as_array_mut) else {
            continue;
        };
        for group in groups.iter_mut() {
            let Some(group) = group.as_object_mut() else {
                continue;
            };
            let matcher = group
                .get("matcher")
                .and_then(Value::as_str)
                .map(str::to_string);
            let Some(handlers) = group.get_mut("hooks").and_then(Value::as_array_mut) else {
                continue;
            };
            handlers.retain(|handler| {
                handler_hash(&event, matcher.as_deref(), handler)
                    .map(|candidate| candidate != hash)
                    .unwrap_or(true)
            });
        }
    }
}

pub(super) fn add_handler(
    hooks: &mut Map<String, Value>,
    event: &str,
    matcher: Option<&str>,
    handler: Value,
) {
    let groups = hooks
        .entry(event.to_string())
        .or_insert_with(|| Value::Array(Vec::new()))
        .as_array_mut()
        .expect("hooks event is an array");
    let group = groups.iter_mut().find(|group| {
        group
            .get("matcher")
            .and_then(Value::as_str)
            .map(str::to_string)
            .as_deref()
            == matcher
    });
    if let Some(group) = group {
        group
            .get_mut("hooks")
            .and_then(Value::as_array_mut)
            .expect("hooks group is an array")
            .push(handler);
        return;
    }
    let mut group = Map::new();
    if let Some(matcher) = matcher {
        group.insert("matcher".to_string(), Value::String(matcher.to_string()));
    }
    group.insert("hooks".to_string(), Value::Array(vec![handler]));
    groups.push(Value::Object(group));
}

pub(super) fn document_hooks(document: &Value) -> Vec<DocumentHook> {
    let mut hooks = Vec::new();
    let Some(entries) = document.get("hooks").and_then(Value::as_object) else {
        return hooks;
    };
    for (event, groups) in entries {
        let Some(groups) = groups.as_array() else {
            continue;
        };
        for group in groups {
            let Some(group) = group.as_object() else {
                continue;
            };
            let matcher = group
                .get("matcher")
                .and_then(Value::as_str)
                .map(str::to_string);
            let Some(handlers) = group.get("hooks").and_then(Value::as_array) else {
                continue;
            };
            for handler in handlers {
                let Ok(review) = review_item(event, matcher.as_deref(), handler) else {
                    continue;
                };
                let Ok(hash) = hash_review(&review) else {
                    continue;
                };
                let command = review
                    .command_windows
                    .clone()
                    .unwrap_or_else(|| review.command.clone());
                hooks.push(DocumentHook {
                    event: event.clone(),
                    matcher: matcher.clone(),
                    hash,
                    command,
                });
            }
        }
    }
    hooks
}

fn review_item(
    event: &str,
    matcher: Option<&str>,
    handler: &Value,
) -> Result<HookReviewItem, ClientError> {
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
    let command = handler
        .get("command")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| ClientError::new("invalid_response", "command Hook 缺少 command。"))?;
    let command_windows = handler
        .get("commandWindows")
        .map(|value| {
            let command = value.as_str().ok_or_else(|| {
                ClientError::new("invalid_response", "commandWindows 不是字符串。")
            })?;
            if command.trim().is_empty() {
                return Err(ClientError::new(
                    "invalid_response",
                    "commandWindows 为空。",
                ));
            }
            Ok(command.to_string())
        })
        .transpose()?;
    Ok(HookReviewItem {
        event: event.to_string(),
        matcher: matcher.map(str::to_string),
        command: command.to_string(),
        command_windows,
    })
}

fn handler_hash(
    event: &str,
    matcher: Option<&str>,
    handler: &Value,
) -> Result<String, ClientError> {
    hash_review(&review_item(event, matcher, handler)?)
}

fn hash_review(review: &HookReviewItem) -> Result<String, ClientError> {
    let identity = serde_json::json!({
        "event": review.event,
        "matcher": review.matcher,
        "type": "command",
        "command": review.command,
        "commandWindows": review.command_windows,
    });
    let bytes = serde_json::to_vec(&identity).map_err(|error| {
        ClientError::new(
            "local_extensions_error",
            format!("无法计算 Hook 标识：{error}"),
        )
    })?;
    Ok(Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn validate_hook_document_shape(hooks: &Value) -> Result<(), ClientError> {
    let Some(hooks) = hooks.as_object() else {
        return Err(ClientError::new(
            "local_extensions_error",
            "Codex hooks.json 的 hooks 不是对象。",
        ));
    };
    for (event, groups) in hooks {
        let Some(groups) = groups.as_array() else {
            return Err(ClientError::new(
                "local_extensions_error",
                format!("Codex hooks.json 的事件 {event} 不是数组。"),
            ));
        };
        for group in groups {
            if !group.is_object()
                || !group
                    .get("hooks")
                    .is_some_and(|handlers| handlers.as_array().is_some())
            {
                return Err(ClientError::new(
                    "local_extensions_error",
                    format!("Codex hooks.json 的事件 {event} 包含无效 matcher 组。"),
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn storage_error(error: std::io::Error) -> ClientError {
    ClientError::new(
        "local_extensions_error",
        format!("无法读写 Codex Hook 配置：{error}"),
    )
}

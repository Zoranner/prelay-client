use std::{
    fs,
    path::{Path, PathBuf},
};

use prelay_protocol::{ExtensionMcpManifest, ExtensionMcpTransport};
use serde_json::{json, Map, Value};

use super::{AgentIntegration, AgentItem, AgentItemKind};
use crate::agents::{
    command_client_version, command_path, deduplicate, error_item, remove_skill_directory,
    scan_skills, write_json, AgentItemStatus,
};

pub static OPENCODE: OpenCodeIntegration = OpenCodeIntegration;
pub struct OpenCodeIntegration;

pub(crate) fn config_directory(home: &Path) -> PathBuf {
    home.join(".config").join("opencode")
}

pub(crate) fn configuration_path(home: &Path) -> PathBuf {
    config_directory(home).join("opencode.jsonc")
}

impl AgentIntegration for OpenCodeIntegration {
    fn is_installed(&self) -> bool {
        command_path("opencode").is_some()
    }

    fn scan(&self, home: &Path) -> Vec<AgentItem> {
        let path = configuration_path(home);
        let mut items = if path.is_file() {
            match read_config(&path) {
                Ok(config) => [mcp_items(&config, &path), plugin_items(&config, &path)].concat(),
                Err(()) => vec![error_item(AgentItemKind::Mcp, &path)],
            }
        } else {
            Vec::new()
        };
        items.extend(local_plugin_items(&config_directory(home).join("plugins")));
        items.extend(scan_skills(home.join(".agents").join("skills")));
        deduplicate(items)
    }

    fn version(&self) -> Option<String> {
        command_path("opencode").and_then(|path| command_client_version(&path))
    }

    fn rule_target(&self, home: &Path) -> Option<PathBuf> {
        Some(config_directory(home).join("AGENTS.md"))
    }

    fn skill_target_root(&self, home: &Path) -> Option<PathBuf> {
        Some(home.join(".agents").join("skills"))
    }

    fn uninstall(
        &self,
        home: &Path,
        kind: AgentItemKind,
        name: &str,
        source_path: &str,
    ) -> Result<(), String> {
        match kind {
            AgentItemKind::Skill => remove_skill_directory(source_path),
            AgentItemKind::Mcp => remove_config_entry(&configuration_path(home), "mcp", name),
            AgentItemKind::Plugin if Path::new(source_path).is_file() => {
                fs::remove_file(source_path)
                    .map_err(|error| format!("无法删除 OpenCode 插件文件：{error}"))
            }
            AgentItemKind::Plugin => remove_plugin_entry(&configuration_path(home), name),
        }
    }
}

fn read_config(path: &Path) -> Result<Value, ()> {
    fs::read_to_string(path)
        .ok()
        .and_then(|contents| json5::from_str(&contents).ok())
        .filter(Value::is_object)
        .ok_or(())
}

fn mcp_items(config: &Value, path: &Path) -> Vec<AgentItem> {
    config
        .get("mcp")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .map(|(name, entry)| AgentItem {
            kind: AgentItemKind::Mcp,
            name: name.to_string(),
            version: None,
            source_path: path.display().to_string(),
            status: if entry
                .get("enabled")
                .and_then(Value::as_bool)
                .is_some_and(|enabled| !enabled)
            {
                AgentItemStatus::Disabled
            } else {
                AgentItemStatus::Enabled
            },
            error_message: None,
        })
        .collect()
}

fn plugin_items(config: &Value, path: &Path) -> Vec<AgentItem> {
    config
        .get("plugin")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(|name| AgentItem {
            kind: AgentItemKind::Plugin,
            name: name.to_string(),
            version: None,
            source_path: path.display().to_string(),
            status: AgentItemStatus::Enabled,
            error_message: None,
        })
        .collect()
}

fn local_plugin_items(root: &Path) -> Vec<AgentItem> {
    let mut items = Vec::new();
    visit_plugin_directory(root, &mut items);
    items
}

fn visit_plugin_directory(path: &Path, items: &mut Vec<AgentItem>) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit_plugin_directory(&path, items);
            continue;
        }
        if !matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("js" | "ts")
        ) {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|name| name.to_str()) else {
            continue;
        };
        items.push(AgentItem {
            kind: AgentItemKind::Plugin,
            name: name.to_string(),
            version: None,
            source_path: path.display().to_string(),
            status: AgentItemStatus::Enabled,
            error_message: None,
        });
    }
}

fn remove_config_entry(path: &Path, section: &str, name: &str) -> Result<(), String> {
    let mut document =
        read_config(path).map_err(|_| "OpenCode 配置不是有效的 JSONC。".to_string())?;
    let entries = document
        .get_mut(section)
        .and_then(Value::as_object_mut)
        .ok_or_else(|| "未找到要卸载的配置项。".to_string())?;
    if entries.remove(name).is_none() {
        return Err("未找到要卸载的配置项。".to_string());
    }
    write_json(path, &document)
}

fn remove_plugin_entry(path: &Path, name: &str) -> Result<(), String> {
    let mut document =
        read_config(path).map_err(|_| "OpenCode 配置不是有效的 JSONC。".to_string())?;
    let plugins = document
        .get_mut("plugin")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "未找到要卸载的插件登记。".to_string())?;
    let original_len = plugins.len();
    plugins.retain(|entry| entry.as_str() != Some(name));
    if plugins.len() == original_len {
        return Err("未找到要卸载的插件登记。".to_string());
    }
    write_json(path, &document)
}

pub(crate) fn upsert_mcp_server(
    home: &Path,
    manifest: &ExtensionMcpManifest,
) -> Result<(), String> {
    let path = configuration_path(home);
    let mut document = if path.is_file() {
        read_config(&path).map_err(|_| "OpenCode 配置不是有效的 JSONC。".to_string())?
    } else {
        json!({})
    };
    let server = match &manifest.transport {
        ExtensionMcpTransport::Stdio {
            command,
            cwd,
            environment,
            enabled,
            timeout_ms,
        } => {
            let mut server = Map::from_iter([
                ("type".to_string(), Value::String("local".to_string())),
                (
                    "command".to_string(),
                    Value::Array(command.iter().cloned().map(Value::String).collect()),
                ),
                ("enabled".to_string(), Value::Bool(*enabled)),
            ]);
            if let Some(cwd) = cwd {
                server.insert("cwd".to_string(), Value::String(cwd.clone()));
            }
            if !environment.is_empty() {
                server.insert(
                    "environment".to_string(),
                    serde_json::to_value(environment)
                        .map_err(|error| format!("无法序列化 MCP 环境变量：{error}"))?,
                );
            }
            if let Some(timeout_ms) = timeout_ms {
                server.insert("timeout".to_string(), Value::Number((*timeout_ms).into()));
            }
            Value::Object(server)
        }
        ExtensionMcpTransport::Http {
            url,
            headers,
            enabled,
            timeout_ms,
        } => {
            let mut server = Map::from_iter([
                ("type".to_string(), Value::String("remote".to_string())),
                ("url".to_string(), Value::String(url.clone())),
                ("enabled".to_string(), Value::Bool(*enabled)),
            ]);
            if !headers.is_empty() {
                server.insert(
                    "headers".to_string(),
                    serde_json::to_value(headers)
                        .map_err(|error| format!("无法序列化 MCP 请求头：{error}"))?,
                );
            }
            if let Some(timeout_ms) = timeout_ms {
                server.insert("timeout".to_string(), Value::Number((*timeout_ms).into()));
            }
            Value::Object(server)
        }
    };
    let config = document
        .as_object_mut()
        .ok_or_else(|| "OpenCode 配置必须是对象。".to_string())?;
    let mcp = config
        .entry("mcp")
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .ok_or_else(|| "OpenCode MCP 配置必须是对象。".to_string())?;
    mcp.insert(manifest.name.clone(), server);
    write_json(&path, &document)
}

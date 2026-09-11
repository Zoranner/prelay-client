use std::{
    fs,
    path::{Path, PathBuf},
};

use prelay_protocol::{ExtensionMcpManifest, ExtensionMcpTransport};
use serde_json::{Map, Value};

use super::{AgentIntegration, AgentItem, AgentItemKind};
use crate::agents::{
    command_client_version, command_path, deduplicate, error_item, remove_skill_directory,
    scan_skills, write_json, AgentItemSource, AgentItemStatus,
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
                Ok(config) => mcp_items(&config, &path),
                Err(()) => vec![error_item(AgentItemKind::Mcp, &path)],
            }
        } else {
            Vec::new()
        };
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
            source: AgentItemSource::Personal,
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

pub(crate) fn mcp_server_exists(home: &Path, name: &str) -> Result<bool, String> {
    let path = configuration_path(home);
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(format!("无法读取 OpenCode 配置：{error}")),
    };
    let document = json5::from_str::<Value>(&contents)
        .map_err(|error| format!("OpenCode 配置不是有效的 JSONC：{error}"))?;
    Ok(document
        .get("mcp")
        .and_then(Value::as_object)
        .is_some_and(|servers| servers.contains_key(name)))
}

pub(crate) fn upsert_mcp_server(
    home: &Path,
    manifest: &ExtensionMcpManifest,
) -> Result<(), String> {
    let path = configuration_path(home);
    let contents = match fs::read_to_string(&path) {
        Ok(contents) if contents.trim().is_empty() => "{}".to_string(),
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => "{}".to_string(),
        Err(error) => return Err(format!("无法读取 OpenCode 配置：{error}")),
    };
    let mut document = json5::from_str::<Value>(&contents)
        .map_err(|error| format!("OpenCode 配置不是有效的 JSONC：{error}"))?;
    let root = document
        .as_object_mut()
        .ok_or_else(|| "OpenCode 配置根节点必须是对象。".to_string())?;
    let servers = root
        .entry("mcp".to_string())
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .ok_or_else(|| "OpenCode MCP 配置不是对象。".to_string())?;
    servers.insert(manifest.name.clone(), mcp_server_entry(manifest)?);
    write_json(&path, &document)
}

fn mcp_server_entry(manifest: &ExtensionMcpManifest) -> Result<Value, String> {
    let mut entry = Map::new();
    match &manifest.transport {
        ExtensionMcpTransport::Stdio {
            command,
            cwd,
            environment,
            enabled,
            timeout_ms,
        } => {
            if command
                .first()
                .is_none_or(|program| program.trim().is_empty())
            {
                return Err("MCP 命令不能为空。".to_string());
            }
            if cwd.is_some() {
                return Err("MCP 工作目录当前不受支持。".to_string());
            }
            if environment.iter().any(|(name, value)| name != value) {
                return Err("MCP 环境变量必须使用同名引用。".to_string());
            }
            entry.insert("type".to_string(), Value::String("local".to_string()));
            entry.insert(
                "command".to_string(),
                Value::Array(command.iter().cloned().map(Value::String).collect()),
            );
            if !environment.is_empty() {
                entry.insert(
                    "environment".to_string(),
                    Value::Object(
                        environment
                            .keys()
                            .map(|name| (name.clone(), Value::String(format!("{{env:{name}}}"))))
                            .collect(),
                    ),
                );
            }
            entry.insert("enabled".to_string(), Value::Bool(*enabled));
            set_timeout(&mut entry, *timeout_ms);
        }
        ExtensionMcpTransport::Http {
            url,
            headers,
            enabled,
            timeout_ms,
        } => {
            entry.insert("type".to_string(), Value::String("remote".to_string()));
            entry.insert("url".to_string(), Value::String(url.clone()));
            if !headers.is_empty() {
                entry.insert(
                    "headers".to_string(),
                    Value::Object(
                        headers
                            .iter()
                            .map(|(name, variable)| {
                                (name.clone(), Value::String(format!("{{env:{variable}}}")))
                            })
                            .collect(),
                    ),
                );
            }
            entry.insert("enabled".to_string(), Value::Bool(*enabled));
            set_timeout(&mut entry, *timeout_ms);
        }
    }
    Ok(Value::Object(entry))
}

fn set_timeout(entry: &mut Map<String, Value>, timeout_ms: Option<u64>) {
    if let Some(timeout_ms) = timeout_ms {
        entry.insert("timeout".to_string(), Value::Number(timeout_ms.into()));
    }
}

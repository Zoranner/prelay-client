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

pub static CLAUDE_CODE: ClaudeCodeIntegration = ClaudeCodeIntegration;
pub struct ClaudeCodeIntegration;

pub(crate) fn configuration_path(home: &Path) -> PathBuf {
    home.join(".claude").join("settings.json")
}

pub(crate) fn mcp_configuration_path(home: &Path) -> PathBuf {
    home.join(".claude.json")
}

impl AgentIntegration for ClaudeCodeIntegration {
    fn is_installed(&self) -> bool {
        command_path("claude").is_some()
    }

    fn scan(&self, home: &Path) -> Vec<AgentItem> {
        let mut items = scan_mcp_servers(home);
        items.extend(scan_skills(home.join(".claude").join("skills")));
        deduplicate(items)
    }

    fn version(&self) -> Option<String> {
        command_path("claude").and_then(|path| command_client_version(&path))
    }

    fn rule_target(&self, home: &Path) -> Option<PathBuf> {
        Some(home.join(".claude").join("CLAUDE.md"))
    }

    fn skill_target_root(&self, home: &Path) -> Option<PathBuf> {
        Some(home.join(".claude").join("skills"))
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
            AgentItemKind::Mcp => remove_mcp_server(home, name),
        }
    }
}

fn scan_mcp_servers(home: &Path) -> Vec<AgentItem> {
    let path = mcp_configuration_path(home);
    if !path.is_file() {
        return Vec::new();
    }
    let Ok(contents) = fs::read_to_string(&path) else {
        return vec![error_item(AgentItemKind::Mcp, &path)];
    };
    let Ok(config) = serde_json::from_str::<Value>(&contents) else {
        return vec![error_item(AgentItemKind::Mcp, &path)];
    };
    config
        .get("mcpServers")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .map(|(name, _)| AgentItem {
            kind: AgentItemKind::Mcp,
            name: name.to_string(),
            version: None,
            source: AgentItemSource::Personal,
            source_path: path.display().to_string(),
            status: AgentItemStatus::Enabled,
            error_message: None,
        })
        .collect()
}

pub(crate) fn remove_mcp_server(home: &Path, name: &str) -> Result<(), String> {
    let path = mcp_configuration_path(home);
    let contents =
        fs::read_to_string(&path).map_err(|error| format!("无法读取 Claude Code 配置：{error}"))?;
    let mut document = serde_json::from_str::<Value>(&contents)
        .map_err(|error| format!("Claude Code 配置不是有效的 JSON：{error}"))?;
    let servers = document
        .get_mut("mcpServers")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| "未找到要卸载的配置项。".to_string())?;
    if servers.remove(name).is_none() {
        return Err("未找到要卸载的配置项。".to_string());
    }
    write_json(&path, &document)
}

pub(crate) fn mcp_server_exists(home: &Path, name: &str) -> Result<bool, String> {
    let path = mcp_configuration_path(home);
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(format!("无法读取 Claude Code 配置：{error}")),
    };
    let document = serde_json::from_str::<Value>(&contents)
        .map_err(|error| format!("Claude Code 配置不是有效的 JSON：{error}"))?;
    Ok(document
        .get("mcpServers")
        .and_then(Value::as_object)
        .is_some_and(|servers| servers.contains_key(name)))
}

pub(crate) fn mcp_server_matches(
    home: &Path,
    manifest: &ExtensionMcpManifest,
) -> Result<bool, String> {
    let path = mcp_configuration_path(home);
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(format!("无法读取 Claude Code 配置：{error}")),
    };
    let document = serde_json::from_str::<Value>(&contents)
        .map_err(|error| format!("Claude Code 配置不是有效的 JSON：{error}"))?;
    let actual = document
        .get("mcpServers")
        .and_then(Value::as_object)
        .and_then(|servers| servers.get(&manifest.name));
    Ok(actual == Some(&mcp_server_entry(manifest)?))
}

pub(crate) fn upsert_mcp_server(
    home: &Path,
    manifest: &ExtensionMcpManifest,
) -> Result<(), String> {
    let path = mcp_configuration_path(home);
    let contents = match fs::read_to_string(&path) {
        Ok(contents) if contents.trim().is_empty() => "{}".to_string(),
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => "{}".to_string(),
        Err(error) => return Err(format!("无法读取 Claude Code 配置：{error}")),
    };
    let mut document = serde_json::from_str::<Value>(&contents)
        .map_err(|error| format!("Claude Code 配置不是有效的 JSON：{error}"))?;
    let root = document
        .as_object_mut()
        .ok_or_else(|| "Claude Code 配置根节点必须是对象。".to_string())?;
    let servers = root
        .entry("mcpServers".to_string())
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .ok_or_else(|| "Claude Code MCP 配置不是对象。".to_string())?;
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
            let (program, arguments) = command
                .split_first()
                .filter(|(program, _)| !program.trim().is_empty())
                .ok_or_else(|| "MCP 命令不能为空。".to_string())?;
            if cwd.is_some() {
                return Err("MCP 工作目录当前不受支持。".to_string());
            }
            if environment.iter().any(|(name, value)| name != value) {
                return Err("MCP 环境变量必须使用同名引用。".to_string());
            }
            entry.insert("type".to_string(), Value::String("stdio".to_string()));
            entry.insert("command".to_string(), Value::String(program.to_string()));
            if !arguments.is_empty() {
                entry.insert(
                    "args".to_string(),
                    Value::Array(arguments.iter().cloned().map(Value::String).collect()),
                );
            }
            if !environment.is_empty() {
                entry.insert(
                    "env".to_string(),
                    Value::Object(
                        environment
                            .keys()
                            .map(|name| (name.clone(), Value::String(format!("${{{name}}}"))))
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
            entry.insert("type".to_string(), Value::String("http".to_string()));
            entry.insert("url".to_string(), Value::String(url.clone()));
            if !headers.is_empty() {
                entry.insert(
                    "headers".to_string(),
                    Value::Object(
                        headers
                            .iter()
                            .map(|(name, variable)| {
                                (name.clone(), Value::String(format!("${{{variable}}}")))
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

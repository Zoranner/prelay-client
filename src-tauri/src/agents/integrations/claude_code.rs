use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;

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

fn remove_mcp_server(home: &Path, name: &str) -> Result<(), String> {
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

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

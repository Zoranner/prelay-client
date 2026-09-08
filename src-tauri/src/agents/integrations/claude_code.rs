use std::path::{Path, PathBuf};

use super::{AgentIntegration, AgentItem, AgentItemKind};
use crate::agents::{
    command_client_version, command_path, deduplicate, remove_skill_directory, scan_skills,
};

pub static CLAUDE_CODE: ClaudeCodeIntegration = ClaudeCodeIntegration;
pub struct ClaudeCodeIntegration;

pub(crate) fn configuration_path(home: &Path) -> PathBuf {
    home.join(".claude").join("settings.json")
}

impl AgentIntegration for ClaudeCodeIntegration {
    fn is_installed(&self) -> bool {
        command_path("claude").is_some()
    }

    fn scan(&self, home: &Path) -> Vec<AgentItem> {
        deduplicate(scan_skills(home.join(".claude").join("skills")))
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
        _home: &Path,
        kind: AgentItemKind,
        _name: &str,
        source_path: &str,
    ) -> Result<(), String> {
        match kind {
            AgentItemKind::Skill => remove_skill_directory(source_path),
            AgentItemKind::Mcp => Err("Claude Code MCP 配置当前不支持卸载。".to_string()),
        }
    }
}

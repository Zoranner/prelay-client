use std::path::Path;

use super::{AgentIntegration, AgentItem, AgentItemKind};
use crate::agents::{
    command_client_version, command_path, remove_claude_mcp_item, remove_claude_plugin,
    remove_skill_directory, scan_claude_code,
};

pub static CLAUDE_CODE: ClaudeCodeIntegration = ClaudeCodeIntegration;
pub struct ClaudeCodeIntegration;

impl AgentIntegration for ClaudeCodeIntegration {
    fn is_installed(&self) -> bool {
        command_path("claude").is_some()
    }
    fn scan(&self, home: &Path) -> Vec<AgentItem> {
        scan_claude_code(home)
    }
    fn version(&self) -> Option<String> {
        command_path("claude").and_then(|path| command_client_version(&path))
    }
    fn uninstall(
        &self,
        home: &Path,
        kind: AgentItemKind,
        name: &str,
        source_path: &str,
    ) -> Result<(), String> {
        match kind {
            AgentItemKind::Mcp => remove_claude_mcp_item(home, name),
            AgentItemKind::Plugin => remove_claude_plugin(home, name),
            AgentItemKind::Skill => remove_skill_directory(source_path),
        }
    }
}

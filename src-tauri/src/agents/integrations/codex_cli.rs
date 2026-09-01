use std::path::{Path, PathBuf};

use super::{AgentIntegration, AgentItem, AgentItemKind};
use crate::agents::{
    command_client_version, command_path, remove_codex_config_item, remove_skill_directory,
    scan_codex,
};

pub static CODEX_CLI: CodexCliIntegration = CodexCliIntegration;
pub struct CodexCliIntegration;

impl AgentIntegration for CodexCliIntegration {
    fn is_installed(&self) -> bool {
        command_path("codex").is_some()
    }
    fn scan(&self, home: &Path) -> Vec<AgentItem> {
        scan_codex(home)
    }
    fn version(&self) -> Option<String> {
        command_path("codex").and_then(|path| command_client_version(&path))
    }

    fn rule_target(&self, home: &Path) -> Option<PathBuf> {
        Some(home.join(".codex").join("AGENTS.md"))
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
            AgentItemKind::Mcp => remove_codex_config_item(home, "mcp_servers", name),
            AgentItemKind::Skill => remove_skill_directory(source_path),
        }
    }
}

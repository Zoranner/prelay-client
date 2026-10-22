use std::path::{Path, PathBuf};

use super::{AgentIntegration, AgentItem, AgentItemKind};
use crate::agents::{
    chatgpt_desktop_version, remove_codex_config_item, remove_skill_directory, scan_codex,
};

pub static CHATGPT: ChatGptIntegration = ChatGptIntegration;
pub struct ChatGptIntegration;

impl AgentIntegration for ChatGptIntegration {
    fn is_installed(&self) -> bool {
        chatgpt_desktop_version().is_some()
    }
    fn scan(&self, home: &Path) -> Vec<AgentItem> {
        scan_codex(home)
            .into_iter()
            .filter(|item| item.kind != AgentItemKind::Hook)
            .collect()
    }
    fn version(&self) -> Option<String> {
        chatgpt_desktop_version()
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
            AgentItemKind::Hook => Err("Hook 仅支持 Codex CLI。".to_string()),
            AgentItemKind::Plugin => Err("插件需要通过扩展包记录卸载。".to_string()),
        }
    }
}

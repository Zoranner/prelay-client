use std::path::Path;

use super::{AgentIntegration, AgentItem, AgentItemKind};
use crate::agents::{
    chatgpt_desktop_version, remove_codex_config_item, remove_codex_plugin, remove_skill_directory,
    scan_codex,
};

pub static CHATGPT: ChatGptIntegration = ChatGptIntegration;
pub struct ChatGptIntegration;

impl AgentIntegration for ChatGptIntegration {
    fn is_installed(&self) -> bool {
        chatgpt_desktop_version().is_some()
    }
    fn scan(&self, home: &Path) -> Vec<AgentItem> {
        scan_codex(home)
    }
    fn version(&self) -> Option<String> {
        chatgpt_desktop_version()
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
            AgentItemKind::Plugin => remove_codex_plugin(home, name),
            AgentItemKind::Skill => remove_skill_directory(source_path),
        }
    }
}

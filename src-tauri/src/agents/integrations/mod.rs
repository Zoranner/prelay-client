mod chatgpt;
mod codex_cli;
pub(crate) mod opencode;

use std::path::{Path, PathBuf};

use super::{AgentClient, AgentItem, AgentItemKind};

pub trait AgentIntegration: Sync {
    fn is_installed(&self) -> bool;
    fn scan(&self, home: &Path) -> Vec<AgentItem>;
    fn version(&self) -> Option<String>;
    fn rule_target(&self, home: &Path) -> Option<PathBuf>;
    fn skill_target_root(&self, home: &Path) -> Option<PathBuf>;
    fn uninstall(
        &self,
        home: &Path,
        kind: AgentItemKind,
        name: &str,
        source_path: &str,
    ) -> Result<(), String>;
}

pub fn integration(client: AgentClient) -> &'static dyn AgentIntegration {
    match client {
        AgentClient::CodexCli => &codex_cli::CODEX_CLI,
        AgentClient::ChatGpt => &chatgpt::CHATGPT,
        AgentClient::OpenCode => &opencode::OPENCODE,
    }
}

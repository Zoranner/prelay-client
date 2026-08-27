mod chatgpt;
mod claude_code;
mod codex_cli;

use std::path::Path;

use super::{AgentClient, AgentItem, AgentItemKind};

pub trait AgentIntegration: Sync {
    fn is_installed(&self) -> bool;
    fn scan(&self, home: &Path) -> Vec<AgentItem>;
    fn version(&self) -> Option<String>;
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
        AgentClient::ClaudeCode => &claude_code::CLAUDE_CODE,
    }
}

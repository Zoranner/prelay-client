use std::{fs, path::Path};

use prelay_protocol::CatalogLanguageModelResponse;

mod codex_catalog;
use serde::{Deserialize, Serialize};

use super::AgentClient;
use crate::{agents::agent_rule_targets, extensions::rules};

mod codex;
mod document;
mod opencode;

#[cfg(test)]
mod catalog_tests;
#[cfg(test)]
mod rule_state_tests;
#[cfg(test)]
mod test_helpers;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod validation_tests;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", tag = "client", content = "settings")]
pub enum AgentSettings {
    CodexCli(CodexSettings),
    #[serde(rename = "chatgpt")]
    ChatGpt(ChatGptSettings),
    OpenCode(OpenCodeSettings),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ChatGptSettings(pub CodexSettings);

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(rename_all_fields = "camelCase")]
#[serde(tag = "kind")]
pub enum CodexConnection {
    Prelay {
        endpoint_id: String,
        endpoint_name: String,
        relay_url: String,
        endpoint_token: String,
        models: Vec<CatalogLanguageModelResponse>,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(rename_all_fields = "camelCase")]
#[serde(tag = "kind")]
pub enum OpenCodeConnection {
    Prelay {
        relay_url: String,
        endpoint_token: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", tag = "client", content = "connection")]
pub enum AgentConnection {
    CodexCli(CodexConnection),
    #[serde(rename = "chatgpt")]
    ChatGpt(CodexConnection),
    OpenCode(OpenCodeConnection),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub personality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_response_storage: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_threads: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_max_runtime_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_access: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell_environment_inherit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows_sandbox: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<String>,
    pub features: CodexFeatures,
}

impl Default for CodexSettings {
    fn default() -> Self {
        Self {
            endpoint_name: None,
            base_url: None,
            custom_token: None,
            model: None,
            reasoning_effort: None,
            personality: Some("pragmatic".to_string()),
            web_search: Some(true),
            sandbox: Some("workspace-write".to_string()),
            disable_response_storage: Some(true),
            max_threads: Some(16),
            max_depth: Some(1),
            job_max_runtime_seconds: Some(1800),
            network_access: Some(true),
            shell_environment_inherit: Some("all".to_string()),
            windows_sandbox: Some("unelevated".to_string()),
            rules: Some(String::new()),
            features: CodexFeatures::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexFeatures {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memories: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goals: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_dependencies: Option<bool>,
}

impl Default for CodexFeatures {
    fn default() -> Self {
        Self {
            memories: Some(true),
            goals: Some(true),
            workspace_dependencies: Some(false),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodeSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<String>,
}

pub fn read_user_settings(home: &Path, client: AgentClient) -> AgentSettings {
    match client {
        AgentClient::CodexCli => AgentSettings::CodexCli(codex::read_codex_settings(home)),
        AgentClient::ChatGpt => {
            AgentSettings::ChatGpt(ChatGptSettings(codex::read_codex_settings(home)))
        }
        AgentClient::OpenCode => AgentSettings::OpenCode(opencode::read_opencode_settings(home)),
    }
}

pub fn save_user_settings(
    home: &Path,
    settings: &AgentSettings,
    connection: Option<&AgentConnection>,
) -> Result<(), String> {
    let client = settings_client(settings);
    let targets = agent_rule_targets(&[client], home);
    let rules = settings_rules(settings);
    let rules_changed = targets
        .iter()
        .any(|target| fs::read_to_string(target).unwrap_or_default() != rules);
    let result = match (settings, connection) {
        (AgentSettings::CodexCli(settings), None) => {
            codex::save_codex_settings(home, settings, None)
        }
        (AgentSettings::CodexCli(settings), Some(AgentConnection::CodexCli(connection))) => {
            codex::save_codex_settings(home, settings, Some(connection))
        }
        (AgentSettings::ChatGpt(settings), None) => {
            codex::save_codex_settings(home, &settings.0, None)
        }
        (AgentSettings::ChatGpt(settings), Some(AgentConnection::ChatGpt(connection))) => {
            codex::save_codex_settings(home, &settings.0, Some(connection))
        }
        (AgentSettings::OpenCode(settings), None) => {
            opencode::save_opencode_settings(home, settings, None)
        }
        (AgentSettings::OpenCode(settings), Some(AgentConnection::OpenCode(connection))) => {
            opencode::save_opencode_settings(home, settings, Some(connection))
        }
        _ => Err("智能体设置与接入配置不匹配".to_string()),
    };
    result?;
    if rules_changed {
        for target in targets {
            rules::clear_rule_package_state(&target)?;
        }
    }
    Ok(())
}

fn settings_client(settings: &AgentSettings) -> AgentClient {
    match settings {
        AgentSettings::CodexCli(_) => AgentClient::CodexCli,
        AgentSettings::ChatGpt(_) => AgentClient::ChatGpt,
        AgentSettings::OpenCode(_) => AgentClient::OpenCode,
    }
}

fn settings_rules(settings: &AgentSettings) -> &str {
    match settings {
        AgentSettings::CodexCli(settings) => settings.rules.as_deref().unwrap_or_default(),
        AgentSettings::ChatGpt(settings) => settings.0.rules.as_deref().unwrap_or_default(),
        AgentSettings::OpenCode(settings) => settings.rules.as_deref().unwrap_or_default(),
    }
}

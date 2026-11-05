use std::{fs, path::Path};

use prelay_protocol::CatalogLanguageModelResponse;

mod codex_catalog;
use serde::{Deserialize, Serialize};

use super::AgentClient;
use crate::{
    agents::agent_rule_targets,
    extensions::{
        host_state::{self, ExtensionHost, HostState},
        rules,
    },
};

mod codex;
mod document;

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
#[serde(rename_all = "camelCase", tag = "client", content = "connection")]
pub enum AgentConnection {
    CodexCli(CodexConnection),
    #[serde(rename = "chatgpt")]
    ChatGpt(CodexConnection),
}

impl AgentConnection {
    /// 这条接入配置引用的接入点。
    pub(crate) fn endpoint_id(&self) -> &str {
        match self {
            Self::CodexCli(CodexConnection::Prelay { endpoint_id, .. })
            | Self::ChatGpt(CodexConnection::Prelay { endpoint_id, .. }) => endpoint_id,
        }
    }
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

pub fn read_user_settings(home: &Path, client: AgentClient) -> AgentSettings {
    match client {
        AgentClient::CodexCli => AgentSettings::CodexCli(codex::read_codex_settings(home)),
        AgentClient::ChatGpt => {
            AgentSettings::ChatGpt(ChatGptSettings(codex::read_codex_settings(home)))
        }
    }
}

pub fn save_user_settings(
    home: &Path,
    settings: &AgentSettings,
    connection: Option<&AgentConnection>,
) -> Result<bool, String> {
    let client = settings.client();
    let host = ExtensionHost::from_client(client);
    let _lock = host_state::lock(home, host).map_err(|error| error.message)?;
    host_state::ensure_migrated_locked(home).map_err(|error| error.message)?;
    let targets = agent_rule_targets(&[client], home);
    let rules = settings_rules(settings);
    let rules_changed = targets
        .iter()
        .any(|target| fs::read_to_string(target).unwrap_or_default() != rules);
    let changed = match (settings, connection) {
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
        _ => Err("智能体设置与接入配置不匹配".to_string()),
    };
    let changed = changed?;
    if rules_changed {
        clear_rule_states(home, client, &targets)?;
    }
    Ok(changed || rules_changed)
}

fn clear_rule_states(
    home: &Path,
    client: AgentClient,
    targets: &[std::path::PathBuf],
) -> Result<(), String> {
    let host = ExtensionHost::from_client(client);
    let mut state = HostState::load_locked(home, host).map_err(|error| error.message)?;
    state.ensure_writable().map_err(|error| error.message)?;
    for target in targets {
        rules::forget(&mut state.rules, target);
    }
    host_state::write_rules(home, host, &state.rules).map_err(|error| error.message)
}

impl AgentSettings {
    pub(crate) fn client(&self) -> AgentClient {
        match self {
            Self::CodexCli(_) => AgentClient::CodexCli,
            Self::ChatGpt(_) => AgentClient::ChatGpt,
        }
    }
}

fn settings_rules(settings: &AgentSettings) -> &str {
    match settings {
        AgentSettings::CodexCli(settings) => settings.rules.as_deref().unwrap_or_default(),
        AgentSettings::ChatGpt(settings) => settings.0.rules.as_deref().unwrap_or_default(),
    }
}

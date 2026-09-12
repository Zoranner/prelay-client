use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentClient {
    CodexCli,
    #[serde(rename = "chatgpt")]
    ChatGpt,
    OpenCode,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentItemKind {
    Mcp,
    Skill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentItemStatus {
    Enabled,
    Disabled,
    Error,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentItemSource {
    Personal,
    Team,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentItem {
    pub kind: AgentItemKind,
    pub name: String,
    pub version: Option<String>,
    pub source: AgentItemSource,
    pub source_path: String,
    pub status: AgentItemStatus,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentClientItems {
    pub client: AgentClient,
    pub version: Option<String>,
    pub items: Vec<AgentItem>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentItemsSnapshot {
    pub clients: Vec<AgentClientItems>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentClientVersion {
    pub client: AgentClient,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentClientStatus {
    pub client: AgentClient,
    pub installed: bool,
    pub version: Option<String>,
}

pub const REGISTERED_AGENT_CLIENTS: [AgentClient; 3] = [
    AgentClient::CodexCli,
    AgentClient::ChatGpt,
    AgentClient::OpenCode,
];

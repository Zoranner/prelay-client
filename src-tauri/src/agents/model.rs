use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentClient {
    CodexCli,
    #[serde(rename = "chatgpt")]
    ChatGpt,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentItemKind {
    Mcp,
    Skill,
    Hook,
    Plugin,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentItemSource {
    Personal,
    Team,
    System,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentItem {
    pub kind: AgentItemKind,
    pub name: String,
    pub version: Option<String>,
    pub package: Option<String>,
    pub hook_hash: Option<String>,
    pub matcher: Option<String>,
    pub command: Option<String>,
    pub transport: Option<String>,
    pub target: Option<String>,
    pub description: Option<String>,
    pub file_count: Option<u32>,
    pub members: Vec<String>,
    pub artifacts: Vec<AgentArtifactGroup>,
    pub source: AgentItemSource,
    pub source_path: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentArtifactGroup {
    pub kind: AgentItemKind,
    pub items: Vec<String>,
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

pub const REGISTERED_AGENT_CLIENTS: [AgentClient; 2] =
    [AgentClient::CodexCli, AgentClient::ChatGpt];

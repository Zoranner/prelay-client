use serde::{Deserialize, Serialize};

use crate::agents::AgentClient;

pub use prelay_protocol::ExtensionKind;

pub(crate) const SKILLS_PREFIX: &str = "skills/";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionPackage {
    pub name: String,
    pub repository: String,
    pub commit_sha: String,
    pub version: String,
    pub kind: ExtensionKind,
    #[serde(default)]
    pub install_action: ExtensionInstallAction,
    #[serde(default)]
    pub installed_clients: Vec<AgentClient>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExtensionInstallAction {
    #[default]
    Install,
    Partial,
    Update,
    Installed,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionCatalogSnapshot {
    pub packages: Vec<ExtensionPackage>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionInstallRequest {
    pub package: ExtensionPackage,
    pub clients: Vec<AgentClient>,
    pub overwrite: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionInstallResult {
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpInstallPreview {
    pub name: String,
    pub version: String,
    pub commit_sha: String,
    pub manifest: prelay_protocol::ExtensionMcpManifest,
}

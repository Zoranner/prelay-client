use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::extensions::atomic_write;

use super::AgentClient;

/// 宿主维度：Codex CLI 与 ChatGPT 共用一份配置，接入点引用按宿主记，不按客户端记。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AgentHost {
    Codex,
    OpenCode,
}

impl AgentHost {
    pub(crate) fn from_client(client: AgentClient) -> Self {
        match client {
            AgentClient::CodexCli | AgentClient::ChatGpt => Self::Codex,
            AgentClient::OpenCode => Self::OpenCode,
        }
    }
}

/// 每个宿主当前引用的接入点。
///
/// 只用来判断“谁在用这个接入点”，接入点本身仍以管理服务为唯一来源；
/// 记录缺失时退化成不自动同步，不影响其它功能。
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentEndpointLinks {
    pub codex: Option<String>,
    pub opencode: Option<String>,
}

impl AgentEndpointLinks {
    fn set(&mut self, host: AgentHost, endpoint_id: &str) {
        let slot = match host {
            AgentHost::Codex => &mut self.codex,
            AgentHost::OpenCode => &mut self.opencode,
        };
        *slot = Some(endpoint_id.to_string());
    }
}

#[derive(Clone, Debug)]
pub struct AgentEndpointLinksStore {
    path: PathBuf,
}

impl AgentEndpointLinksStore {
    pub fn at(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// 读不到或内容损坏时按“没有记录”处理：少一次自动同步好过挡住命令。
    pub fn load(&self) -> AgentEndpointLinks {
        fs::read_to_string(&self.path)
            .ok()
            .and_then(|contents| serde_json::from_str::<AgentEndpointLinks>(&contents).ok())
            .unwrap_or_default()
    }

    pub(crate) fn record(&self, host: AgentHost, endpoint_id: &str) -> Result<(), String> {
        let mut links = self.load();
        links.set(host, endpoint_id);
        self.write(&links)
    }

    /// 解除接入后把记录清掉，避免下次对账继续按旧引用比对。
    pub(crate) fn clear(&self, host: AgentHost) -> Result<(), String> {
        let mut links = self.load();
        let slot = match host {
            AgentHost::Codex => &mut links.codex,
            AgentHost::OpenCode => &mut links.opencode,
        };
        if slot.is_none() {
            return Ok(());
        }
        *slot = None;
        self.write(&links)
    }

    fn write(&self, links: &AgentEndpointLinks) -> Result<(), String> {
        let contents = serde_json::to_vec_pretty(&links)
            .map_err(|error| format!("接入点引用无法序列化：{error}"))?;
        if let Some(parent) = self
            .path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| format!("接入点引用目录无法创建：{error}"))?;
        }
        atomic_write(&self.path, &contents).map_err(|error| error.message)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn records_and_reads_endpoint_links_per_host() {
        let directory = tempdir().unwrap();
        let store = AgentEndpointLinksStore::at(directory.path().join("agent-endpoints.json"));

        assert_eq!(store.load().codex, None);

        store.record(AgentHost::Codex, "endpoint-codex").unwrap();
        store
            .record(AgentHost::OpenCode, "endpoint-opencode")
            .unwrap();

        let links = store.load();
        assert_eq!(links.codex.as_deref(), Some("endpoint-codex"));
        assert_eq!(links.opencode.as_deref(), Some("endpoint-opencode"));
    }

    #[test]
    fn damaged_state_is_treated_as_no_links() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("agent-endpoints.json");
        fs::write(&path, "{ not json").unwrap();

        let links = AgentEndpointLinksStore::at(&path).load();

        assert_eq!(links.codex, None);
        assert_eq!(links.opencode, None);
    }
}

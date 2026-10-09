use std::path::{Path, PathBuf};

use crate::{
    agents::links::{AgentEndpointLinks, AgentHost},
    agents::settings::{read_user_settings, save_user_settings, AgentConnection, AgentSettings},
    agents::{
        agent_client_statuses, scan_agent_items, uninstall_user_item, AgentClient,
        AgentClientItems, AgentClientStatus, AgentItemKind,
    },
    relay::client::ClientError,
    NativeState,
};
use tauri::State;

#[tauri::command]
pub fn agents_status() -> Vec<AgentClientStatus> {
    agent_client_statuses()
}

#[tauri::command]
pub fn agent_items_get(client: AgentClient) -> Result<AgentClientItems, ClientError> {
    let home = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .ok_or_else(|| ClientError::new("local_agents_error", "USERPROFILE is unavailable"))?;
    Ok(agent_items_from_home(&home, client))
}

#[tauri::command]
pub fn agents_remove(
    client: AgentClient,
    kind: AgentItemKind,
    name: String,
    source_path: String,
) -> Result<(), ClientError> {
    let home = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .ok_or_else(|| ClientError::new("local_agents_error", "USERPROFILE is unavailable"))?;
    uninstall_user_item(&home, client, kind, &name, &source_path)
        .map_err(|error| ClientError::new("local_agents_error", error))
}

#[tauri::command]
pub fn agent_settings_get(client: AgentClient) -> Result<AgentSettings, ClientError> {
    let home = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .ok_or_else(|| ClientError::new("local_agents_error", "USERPROFILE is unavailable"))?;
    Ok(agent_settings_from_home(&home, client))
}

#[tauri::command]
pub fn agent_settings_save(
    state: State<'_, NativeState>,
    settings: AgentSettings,
    connection: Option<AgentConnection>,
) -> Result<bool, ClientError> {
    let home = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .ok_or_else(|| {
            ClientError::new("local_agent_settings_error", "USERPROFILE is unavailable")
        })?;
    let changed = save_user_settings(&home, &settings, connection.as_ref())
        .map_err(|error| ClientError::new("local_agent_settings_error", error))?;
    record_endpoint_link(&state, settings.client(), connection.as_ref())
        .map_err(|error| ClientError::new("local_agent_settings_error", error))?;
    Ok(changed)
}

/// 记录（或解除）智能体当前引用的接入点，供接入点改动后自动对账使用。
fn record_endpoint_link(
    state: &NativeState,
    client: AgentClient,
    connection: Option<&AgentConnection>,
) -> Result<(), String> {
    let host = AgentHost::from_client(client);
    match connection {
        Some(connection) => {
            let (_, endpoint_id) = connection.endpoint_link();
            state.agent_endpoint_links.record(host, endpoint_id)
        }
        None => state.agent_endpoint_links.clear(host),
    }
}

#[tauri::command]
pub fn agent_endpoint_links_get(state: State<'_, NativeState>) -> AgentEndpointLinks {
    state.agent_endpoint_links.load()
}

fn agent_items_from_home(home: &Path, client: AgentClient) -> AgentClientItems {
    scan_agent_items(home, client)
}

fn agent_settings_from_home(home: &Path, client: AgentClient) -> AgentSettings {
    read_user_settings(home, client)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::agents::settings::{AgentConnection, CodexConnection};
    use crate::agents::AgentClient;
    use crate::NativeState;

    use super::{agent_items_from_home, record_endpoint_link};

    #[test]
    fn scans_agents_without_relay_state_or_credentials() {
        let directory = tempdir().unwrap();

        let snapshot = agent_items_from_home(directory.path(), AgentClient::OpenCode);

        assert!(snapshot.items.is_empty());
    }

    #[test]
    fn records_the_endpoint_each_host_was_configured_with() {
        let directory = tempdir().unwrap();
        let state = NativeState::for_app_data_dir(directory.path().to_path_buf());
        let connection = AgentConnection::CodexCli(CodexConnection::Prelay {
            endpoint_id: "endpoint-codex".to_string(),
            endpoint_name: "Endpoint 1".to_string(),
            relay_url: "https://relay.example.test".to_string(),
            endpoint_token: "endpoint-token".to_string(),
            models: Vec::new(),
        });

        record_endpoint_link(&state, AgentClient::CodexCli, Some(&connection)).unwrap();
        record_endpoint_link(&state, AgentClient::OpenCode, None).unwrap();

        let links = state.agent_endpoint_links.load();
        assert_eq!(links.codex.as_deref(), Some("endpoint-codex"));
        assert_eq!(links.opencode, None);

        // 解除接入后记录随之清空，不再按旧引用对账。
        record_endpoint_link(&state, AgentClient::CodexCli, None).unwrap();
        assert_eq!(state.agent_endpoint_links.load().codex, None);
    }
}

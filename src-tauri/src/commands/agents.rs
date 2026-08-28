use std::path::{Path, PathBuf};

use crate::{
    agent_settings::{read_user_settings, save_user_settings, AgentConnection, AgentSettings},
    agents::{
        agent_client_statuses, scan_agent_items, uninstall_user_item, AgentClient,
        AgentClientItems, AgentClientStatus, AgentItemKind,
    },
    api_client::ClientError,
};

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
    settings: AgentSettings,
    connection: Option<AgentConnection>,
) -> Result<(), ClientError> {
    let home = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .ok_or_else(|| {
            ClientError::new("local_agent_settings_error", "USERPROFILE is unavailable")
        })?;
    save_user_settings(&home, &settings, connection.as_ref())
        .map_err(|error| ClientError::new("local_agent_settings_error", error))
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

    use crate::agents::AgentClient;

    use super::agent_items_from_home;

    #[test]
    fn scans_agents_without_relay_state_or_credentials() {
        let directory = tempdir().unwrap();

        let snapshot = agent_items_from_home(directory.path(), AgentClient::OpenCode);

        assert!(snapshot.items.is_empty());
    }
}

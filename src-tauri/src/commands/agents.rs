use std::path::{Path, PathBuf};

use crate::{
    agent_settings::{read_user_settings, save_user_settings, AgentConnection, AgentSettings},
    agents::{
        scan_user_items, uninstall_user_item, AgentClient, AgentItemKind, AgentItemsSnapshot,
    },
    api_client::ClientError,
};

#[tauri::command]
pub fn agents_list() -> Result<AgentItemsSnapshot, ClientError> {
    let home = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .ok_or_else(|| ClientError::new("local_agents_error", "USERPROFILE is unavailable"))?;
    Ok(agents_from_home(&home))
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

fn agents_from_home(home: &Path) -> AgentItemsSnapshot {
    scan_user_items(home)
}

fn agent_settings_from_home(home: &Path, client: AgentClient) -> AgentSettings {
    read_user_settings(home, client)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use crate::agents::AgentClient;

    use super::{agent_settings_from_home, agents_from_home};

    #[test]
    fn scans_agents_without_relay_state_or_credentials() {
        let directory = tempdir().unwrap();

        assert!(agents_from_home(directory.path()).clients.is_empty());
    }

    #[test]
    fn reads_codex_settings_without_parsing_invalid_claude_settings() {
        let directory = tempdir().unwrap();
        let claude_root = directory.path().join(".claude");
        fs::create_dir_all(&claude_root).unwrap();
        fs::write(claude_root.join("settings.json"), "not JSON").unwrap();

        assert!(matches!(
            agent_settings_from_home(directory.path(), AgentClient::Codex),
            crate::agent_settings::AgentSettings::Codex(_)
        ));
    }
}

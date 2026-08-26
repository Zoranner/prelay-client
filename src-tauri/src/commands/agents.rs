use std::path::{Path, PathBuf};

use crate::{
    agent_settings::{
        read_or_initialize_user_settings, save_user_settings, AgentSettingsSnapshot,
        ClaudeCodeConnection, CodexConnection,
    },
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
pub fn agent_settings_get() -> Result<AgentSettingsSnapshot, ClientError> {
    let home = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .ok_or_else(|| ClientError::new("local_agents_error", "USERPROFILE is unavailable"))?;
    read_or_initialize_user_settings(&home)
        .map_err(|error| ClientError::new("local_agent_settings_error", error))
}

#[tauri::command]
pub fn agent_settings_save(
    client: AgentClient,
    settings: AgentSettingsSnapshot,
    codex_connection: Option<CodexConnection>,
    claude_code_connection: Option<ClaudeCodeConnection>,
) -> Result<(), ClientError> {
    let home = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .ok_or_else(|| {
            ClientError::new("local_agent_settings_error", "USERPROFILE is unavailable")
        })?;
    save_user_settings(
        &home,
        client,
        &settings,
        codex_connection.as_ref(),
        claude_code_connection.as_ref(),
    )
    .map_err(|error| ClientError::new("local_agent_settings_error", error))
}

fn agents_from_home(home: &Path) -> AgentItemsSnapshot {
    scan_user_items(home)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::agents_from_home;

    #[test]
    fn scans_agents_without_relay_state_or_credentials() {
        let directory = tempdir().unwrap();

        assert!(agents_from_home(directory.path()).clients.is_empty());
    }
}

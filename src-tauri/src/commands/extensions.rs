use std::path::{Path, PathBuf};

use crate::{
    agent_extensions::{scan_user_extensions, AgentExtensionsSnapshot},
    api_client::ClientError,
};

#[tauri::command]
pub fn extensions_list() -> Result<AgentExtensionsSnapshot, ClientError> {
    let home = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .ok_or_else(|| ClientError::new("local_extensions_error", "USERPROFILE is unavailable"))?;
    Ok(extensions_from_home(&home))
}

fn extensions_from_home(home: &Path) -> AgentExtensionsSnapshot {
    scan_user_extensions(home)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::extensions_from_home;

    #[test]
    fn scans_extensions_without_relay_state_or_credentials() {
        let directory = tempdir().unwrap();

        assert!(extensions_from_home(directory.path()).clients.is_empty());
    }
}

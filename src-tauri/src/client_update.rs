use std::{
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use prelay_protocol::ClientUpdateResponse;
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::{api_client::ClientError, commands::authenticated_api, NativeState};

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadedClientUpdate {
    pub version: String,
}

#[tauri::command]
pub async fn client_update_prepare(
    app: AppHandle,
    state: tauri::State<'_, NativeState>,
) -> Result<Option<DownloadedClientUpdate>, ClientError> {
    if !cfg!(target_os = "windows") {
        return Ok(None);
    }

    let client = authenticated_api(&state).await?;
    let update: ClientUpdateResponse = match client.get("/api/client-update").await {
        Ok(update) => update,
        Err(error) if error.code() == "client_update_unavailable" => return Ok(None),
        Err(error) => return Err(error),
    };
    if !is_newer_version(&update.version, CURRENT_VERSION) {
        return Ok(None);
    }

    let app_data_directory = app
        .path()
        .app_data_dir()
        .map_err(|error| ClientError::new("client_update_storage_error", error.to_string()))?;
    let installer_path = installer_path(&app_data_directory, &update.version)?;
    if !installer_path.is_file() {
        let bytes = client.get_bytes(&update.download_path).await?;
        write_installer(&installer_path, &bytes)
            .map_err(|error| ClientError::new("client_update_storage_error", error.to_string()))?;
    }

    Ok(Some(DownloadedClientUpdate {
        version: update.version,
    }))
}

#[tauri::command]
pub async fn client_update_install(app: AppHandle, version: String) -> Result<(), ClientError> {
    if !cfg!(target_os = "windows") {
        return Err(ClientError::new(
            "client_update_unsupported_platform",
            "client updates are only supported on Windows",
        ));
    }

    let app_data_directory = app
        .path()
        .app_data_dir()
        .map_err(|error| ClientError::new("client_update_storage_error", error.to_string()))?;
    let installer_path = installer_path(&app_data_directory, &version)?;
    if !installer_path.is_file() {
        return Err(ClientError::new(
            "client_update_not_downloaded",
            "client update installer is unavailable",
        ));
    }
    Command::new(installer_path)
        .spawn()
        .map_err(|error| ClientError::new("client_update_install_failed", error.to_string()))?;
    app.exit(0);
    Ok(())
}

fn installer_path(app_data_directory: &Path, version: &str) -> Result<PathBuf, ClientError> {
    if !is_safe_version(version) {
        return Err(ClientError::new(
            "invalid_client_update_version",
            "client update version is invalid",
        ));
    }
    Ok(app_data_directory
        .join("Prelay")
        .join("updates")
        .join(format!("prelay-client-{version}.exe")))
}

fn write_installer(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let parent = path
        .parent()
        .expect("client installer path always has an update directory");
    std::fs::create_dir_all(parent)?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after Unix epoch")
        .as_nanos();
    let temporary = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name().unwrap().to_string_lossy(),
        nonce
    ));
    std::fs::write(&temporary, bytes)?;
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    std::fs::rename(temporary, path)
}

fn is_newer_version(remote: &str, local: &str) -> bool {
    match (parse_version(remote), parse_version(local)) {
        (Some(remote), Some(local)) => remote > local,
        _ => false,
    }
}

fn is_safe_version(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_')
        })
}

fn parse_version(value: &str) -> Option<[u64; 3]> {
    let parts = value.split('.').collect::<Vec<_>>();
    (parts.len() == 3).then_some(())?;
    Some([
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ])
}

#[cfg(test)]
mod tests {
    use super::{installer_path, is_newer_version};

    #[test]
    fn compares_release_versions_and_rejects_unsafe_file_names() {
        assert!(is_newer_version("0.2.0", "0.1.0"));
        assert!(!is_newer_version("0.1.0", "0.1.0"));
        assert!(!is_newer_version("next", "0.1.0"));
        assert!(installer_path(std::path::Path::new("updates"), "0.2.0").is_ok());
        assert!(installer_path(std::path::Path::new("updates"), "../0.2.0").is_err());
    }
}

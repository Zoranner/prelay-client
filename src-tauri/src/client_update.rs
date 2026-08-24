use std::{
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use prelay_protocol::{ClientUpdateResponse, ClientUpdateTarget};
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::{api_client::ClientError, commands::authenticated_api, NativeState};

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadedClientUpdate {
    pub version: String,
    pub file_name: String,
}

#[tauri::command]
pub async fn client_update_prepare(
    app: AppHandle,
    state: tauri::State<'_, NativeState>,
    version: Option<String>,
    file_name: Option<String>,
) -> Result<Option<DownloadedClientUpdate>, ClientError> {
    match (version, file_name) {
        (None, None) => check_for_client_update(&state).await,
        (Some(version), Some(file_name)) => {
            let update = DownloadedClientUpdate { version, file_name };
            download_client_update(app, &state, &update).await?;
            Ok(Some(update))
        }
        _ => Err(ClientError::new(
            "invalid_client_update",
            "client update package is invalid",
        )),
    }
}

async fn check_for_client_update(
    state: &NativeState,
) -> Result<Option<DownloadedClientUpdate>, ClientError> {
    let target = match current_update_target() {
        Some(target) => target,
        None => return Ok(None),
    };

    let client = authenticated_api(state).await?;
    let update_path = format!(
        "/api/client-update?platform={}&architecture={}",
        target.platform, target.architecture
    );
    let update: ClientUpdateResponse = match client.get(&update_path).await {
        Ok(update) => update,
        Err(error) if error.code() == "client_update_unavailable" => return Ok(None),
        Err(error) => return Err(error),
    };
    if !is_newer_version(&update.version, CURRENT_VERSION) {
        return Ok(None);
    }

    if !is_safe_version(&update.version) || !is_safe_file_name(&update.file_name) {
        return Err(ClientError::new(
            "invalid_client_update",
            "client update package is invalid",
        ));
    }

    Ok(Some(DownloadedClientUpdate {
        version: update.version,
        file_name: update.file_name,
    }))
}

async fn download_client_update(
    app: AppHandle,
    state: &NativeState,
    update: &DownloadedClientUpdate,
) -> Result<(), ClientError> {
    let target = current_update_target().ok_or_else(|| {
        ClientError::new(
            "client_update_unsupported_platform",
            "client updates are only supported on Windows",
        )
    })?;

    let app_cache_directory = app
        .path()
        .app_cache_dir()
        .map_err(|error| ClientError::new("client_update_storage_error", error.to_string()))?;
    let installer_path = installer_path(
        &app_cache_directory,
        &target,
        &update.version,
        &update.file_name,
    )?;
    if !installer_path.is_file() {
        let client = authenticated_api(state).await?;
        let download_path = format!(
            "/api/client-update/download?platform={}&architecture={}",
            target.platform, target.architecture
        );
        let bytes = client.get_bytes(&download_path).await?;
        write_installer(&installer_path, &bytes)
            .map_err(|error| ClientError::new("client_update_storage_error", error.to_string()))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn client_update_install(
    app: AppHandle,
    version: String,
    file_name: String,
) -> Result<(), ClientError> {
    let target = current_update_target().ok_or_else(|| {
        ClientError::new(
            "client_update_unsupported_platform",
            "client updates are only supported on Windows",
        )
    })?;

    let app_cache_directory = app
        .path()
        .app_cache_dir()
        .map_err(|error| ClientError::new("client_update_storage_error", error.to_string()))?;
    let installer_path = installer_path(&app_cache_directory, &target, &version, &file_name)?;
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

fn current_update_target() -> Option<ClientUpdateTarget> {
    if !cfg!(target_os = "windows") {
        return None;
    }
    let architecture = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "x86") {
        "x86"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        return None;
    };
    Some(ClientUpdateTarget {
        platform: "windows".to_string(),
        architecture: architecture.to_string(),
    })
}

fn installer_path(
    app_cache_directory: &Path,
    target: &ClientUpdateTarget,
    version: &str,
    file_name: &str,
) -> Result<PathBuf, ClientError> {
    if !is_safe_version(version)
        || !is_safe_path_component(&target.platform)
        || !is_safe_path_component(&target.architecture)
        || !is_safe_file_name(file_name)
    {
        return Err(ClientError::new(
            "invalid_client_update",
            "client update package is invalid",
        ));
    }
    Ok(app_cache_directory
        .join("updates")
        .join(&target.platform)
        .join(&target.architecture)
        .join(version)
        .join(file_name))
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

fn is_safe_path_component(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
}

fn is_safe_file_name(value: &str) -> bool {
    !value.is_empty()
        && Path::new(value)
            .file_name()
            .is_some_and(|file_name| file_name == value)
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
    use prelay_protocol::ClientUpdateTarget;

    #[test]
    fn compares_release_versions_and_rejects_unsafe_file_names() {
        assert!(is_newer_version("0.2.0", "0.1.0"));
        assert!(!is_newer_version("0.1.0", "0.1.0"));
        assert!(!is_newer_version("next", "0.1.0"));
        let target = ClientUpdateTarget {
            platform: "windows".to_string(),
            architecture: "x64".to_string(),
        };
        assert_eq!(
            installer_path(
                std::path::Path::new("cache"),
                &target,
                "0.2.0",
                "Prelay_0.2.0_x64-setup.exe",
            )
            .unwrap(),
            std::path::Path::new("cache/updates/windows/x64/0.2.0/Prelay_0.2.0_x64-setup.exe")
        );
        assert!(installer_path(
            std::path::Path::new("cache"),
            &target,
            "../0.2.0",
            "Prelay_0.2.0_x64-setup.exe",
        )
        .is_err());
    }
}

use std::path::PathBuf;

use crate::{
    api_client::ClientError,
    extensions::{
        install_extension, list_extensions, preview_extension_install, read_extension_readme,
        ExtensionCatalogSnapshot, ExtensionInstallPreview, ExtensionInstallRequest,
        ExtensionInstallResult, ExtensionPackage,
    },
};

fn user_home() -> Result<PathBuf, ClientError> {
    std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .ok_or_else(|| ClientError::new("local_extensions_error", "USERPROFILE is unavailable"))
}

#[tauri::command]
pub async fn extensions_list() -> Result<ExtensionCatalogSnapshot, ClientError> {
    list_extensions()
        .await
        .map_err(|error| ClientError::new("local_extensions_error", error))
}

#[tauri::command]
pub async fn extension_readme(package: ExtensionPackage) -> Result<String, ClientError> {
    read_extension_readme(&package)
        .await
        .map_err(|error| ClientError::new("local_extensions_error", error))
}

#[tauri::command]
pub async fn extension_install_preview(
    request: ExtensionInstallRequest,
) -> Result<ExtensionInstallPreview, ClientError> {
    preview_extension_install(&request)
        .await
        .map_err(|error| ClientError::new("local_extensions_error", error))
}

#[tauri::command]
pub async fn extensions_install(
    request: ExtensionInstallRequest,
) -> Result<ExtensionInstallResult, ClientError> {
    let home = user_home()?;
    install_extension(&home, &request)
        .await
        .map_err(|error| ClientError::new("local_extensions_error", error))
}

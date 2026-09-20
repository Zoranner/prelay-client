use std::path::PathBuf;

use crate::{
    extensions::{
        install_extension, list_extensions, read_extension_readme, read_mcp_install_manifest,
        update_all_extensions, ExtensionCatalogSnapshot, ExtensionInstallRequest,
        ExtensionInstallResult, ExtensionKind, ExtensionPackage, McpInstallPreview,
    },
    relay::client::ClientError,
    NativeState,
};

fn user_home() -> Result<PathBuf, ClientError> {
    std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .ok_or_else(|| ClientError::new("local_extensions_error", "USERPROFILE is unavailable"))
}

#[tauri::command]
pub async fn extensions_list(
    state: tauri::State<'_, NativeState>,
    kind: ExtensionKind,
) -> Result<ExtensionCatalogSnapshot, ClientError> {
    let home = user_home()?;
    list_extensions(&home, &state, kind).await
}

#[tauri::command]
pub async fn extension_readme(
    state: tauri::State<'_, NativeState>,
    package: ExtensionPackage,
) -> Result<String, ClientError> {
    read_extension_readme(&state, &package).await
}

#[tauri::command]
pub async fn extensions_mcp_preview(
    state: tauri::State<'_, NativeState>,
    package: ExtensionPackage,
) -> Result<McpInstallPreview, ClientError> {
    read_mcp_install_manifest(&state, &package).await
}

#[tauri::command]
pub async fn extensions_install(
    state: tauri::State<'_, NativeState>,
    request: ExtensionInstallRequest,
) -> Result<ExtensionInstallResult, ClientError> {
    let home = user_home()?;
    install_extension(&home, &state, &request).await
}

#[tauri::command]
pub async fn extensions_update_all(
    state: tauri::State<'_, NativeState>,
    kind: ExtensionKind,
) -> Result<ExtensionInstallResult, ClientError> {
    let home = user_home()?;
    update_all_extensions(&home, &state, kind).await
}

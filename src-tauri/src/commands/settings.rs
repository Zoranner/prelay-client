use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::{
    api_client::{normalize_relay_url, ApiClient, ClientError},
    autostart,
    desktop_preferences::{DesktopPreferences, DesktopPreferencesStore, ThemeMode},
    identity::IdentitySource,
    relay_settings::RelaySettingsStore,
    NativeState,
};

#[derive(Debug, Serialize)]
pub struct RelaySettingsResponse {
    pub relay_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopPreferencesInput {
    pub theme: ThemeMode,
    pub autostart_enabled: bool,
    pub silent_start: bool,
    pub minimize_to_tray: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopPreferencesResponse {
    pub theme: ThemeMode,
    pub autostart_enabled: bool,
    pub silent_start: bool,
    pub minimize_to_tray: bool,
}

#[tauri::command]
pub fn desktop_preferences_get(
    app: AppHandle,
    state: State<'_, NativeState>,
) -> Result<DesktopPreferencesResponse, ClientError> {
    let preferences = state
        .desktop_preferences
        .load()
        .map_err(|error| ClientError::new("desktop_preferences_error", error))?;
    let autostart_enabled = autostart::is_enabled(&app)
        .map_err(|error| ClientError::new("desktop_preferences_error", error))?;
    Ok(DesktopPreferencesResponse {
        theme: preferences.theme,
        autostart_enabled,
        silent_start: preferences.silent_start,
        minimize_to_tray: preferences.minimize_to_tray,
    })
}

#[tauri::command]
pub fn desktop_preferences_save(
    preferences: DesktopPreferencesInput,
    app: AppHandle,
    state: State<'_, NativeState>,
) -> Result<DesktopPreferencesResponse, ClientError> {
    autostart::set_enabled(&app, preferences.autostart_enabled)
        .map_err(|error| ClientError::new("desktop_preferences_error", error))?;
    let desktop_preferences = DesktopPreferences {
        theme: preferences.theme,
        silent_start: preferences.silent_start,
        minimize_to_tray: preferences.minimize_to_tray,
    };
    state
        .desktop_preferences
        .save(&desktop_preferences)
        .map_err(|error| ClientError::new("desktop_preferences_error", error))?;
    Ok(DesktopPreferencesResponse {
        theme: desktop_preferences.theme,
        autostart_enabled: preferences.autostart_enabled,
        silent_start: desktop_preferences.silent_start,
        minimize_to_tray: desktop_preferences.minimize_to_tray,
    })
}

#[tauri::command]
pub fn relay_settings_get(
    state: State<'_, NativeState>,
) -> Result<RelaySettingsResponse, ClientError> {
    let relay_url = state
        .relay_settings
        .load()
        .map_err(|error| ClientError::new("relay_settings_error", error))?;
    Ok(RelaySettingsResponse { relay_url })
}

#[tauri::command]
pub fn relay_settings_save(
    relay_url: String,
    state: State<'_, NativeState>,
) -> Result<RelaySettingsResponse, ClientError> {
    let relay_url = normalize_relay_url(&relay_url)?;
    state
        .relay_settings
        .save(&relay_url)
        .map_err(|error| ClientError::new("relay_settings_error", error))?;
    Ok(RelaySettingsResponse {
        relay_url: Some(relay_url),
    })
}

#[tauri::command]
pub async fn relay_settings_connect(
    relay_url: String,
    state: State<'_, NativeState>,
) -> Result<RelaySettingsResponse, ClientError> {
    connect_and_save_relay_settings(state.inner(), &relay_url).await
}

pub async fn connect_and_save_relay_settings(
    state: &NativeState,
    relay_url: &str,
) -> Result<RelaySettingsResponse, ClientError> {
    let relay_url = normalize_relay_url(relay_url)?;
    let identity = state
        .identity
        .identity()
        .map_err(|error| ClientError::new("internal", error))?;
    let _credential_lifecycle_guard = state.credential_lifecycle_gate.lock().await;
    let _file_credential_lifecycle_guard = state
        .credentials
        .acquire_lifecycle_lock()
        .await
        .map_err(|error| ClientError::new("credential_store_error", error))?;
    let client =
        ApiClient::new(&relay_url, &state.credentials)?.with_display_name(&identity.display_name);
    client
        .ensure_registered_once(&identity, &state.registration_gate)
        .await?;
    let _: serde_json::Value = client.get("/api/providers").await?;
    state
        .relay_settings
        .save(&relay_url)
        .map_err(|error| ClientError::new("relay_settings_error", error))?;
    Ok(RelaySettingsResponse {
        relay_url: Some(relay_url),
    })
}

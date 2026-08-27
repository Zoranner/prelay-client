//! Prelay desktop client crate.

use tauri::Manager;

pub mod agent_settings;
pub mod agents;
pub mod api_client;
pub mod autostart;
pub mod client_update;
pub mod commands;
pub mod credential_store;
pub mod desktop_preferences;
pub mod extensions;
pub mod identity;
pub mod relay_settings;
pub mod tray;

pub struct NativeState {
    pub identity: identity::WindowsIdentitySource,
    pub credentials: credential_store::FileCredentialStore,
    pub desktop_preferences: desktop_preferences::FileDesktopPreferencesStore,
    pub relay_settings: relay_settings::FileRelaySettingsStore,
    pub registration_gate: api_client::RegistrationGate,
    pub credential_lifecycle_gate: tokio::sync::Mutex<()>,
}

impl NativeState {
    pub fn for_app_data_dir(app_data_dir: std::path::PathBuf) -> Self {
        Self {
            identity: identity::WindowsIdentitySource,
            credentials: credential_store::FileCredentialStore::at(
                app_data_dir.join("Prelay").join("device-credential.json"),
            ),
            desktop_preferences: desktop_preferences::FileDesktopPreferencesStore::at(
                app_data_dir.join("Prelay").join("desktop-preferences.json"),
            ),
            relay_settings: relay_settings::FileRelaySettingsStore::at(
                app_data_dir.join("Prelay").join("relay-settings.json"),
            ),
            registration_gate: api_client::RegistrationGate::default(),
            credential_lifecycle_gate: tokio::sync::Mutex::new(()),
        }
    }
}

impl Default for NativeState {
    fn default() -> Self {
        Self::for_app_data_dir(std::env::temp_dir())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::show_main_window(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap::bootstrap,
            client_update::client_update_prepare,
            client_update::client_update_install,
            commands::settings::relay_settings_get,
            commands::settings::relay_settings_save,
            commands::settings::relay_settings_connect,
            commands::settings::desktop_preferences_get,
            commands::settings::desktop_preferences_save,
            commands::providers::providers_list,
            commands::providers::providers_save,
            commands::providers::providers_delete,
            commands::providers::providers_ping,
            commands::providers::providers_discover_models,
            commands::providers::providers_test_protocol,
            commands::endpoints::endpoints_list,
            commands::endpoints::endpoints_save,
            commands::endpoints::endpoints_delete,
            commands::endpoints::endpoints_regenerate_token,
            commands::agents::agents_list,
            commands::agents::agents_versions,
            commands::agents::agents_remove,
            commands::agents::agent_settings_get,
            commands::agents::agent_settings_save,
            commands::extensions::extensions_list,
            commands::extensions::extension_readme,
            commands::extensions::extensions_install,
            commands::stats::stats_overview,
            commands::stats::stats_timeline,
            commands::stats::stats_requests,
            commands::stats::stats_models,
            commands::stats::stats_providers,
            commands::credential_rotate
        ])
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let state = NativeState::for_app_data_dir(app_data_dir);
            autostart::apply_default_when_preferences_are_missing(
                app.handle(),
                &state.desktop_preferences,
            )
            .map_err(std::io::Error::other)?;
            let start_silently = autostart::should_start_silently(&state.desktop_preferences)
                .map_err(std::io::Error::other)?;
            app.manage(state);
            tray::install(app.handle())?;
            if !start_silently {
                tray::show_main_window(app.handle());
            }
            Ok(())
        })
        .on_window_event(tray::hide_on_close)
        .run(tauri::generate_context!())
        .expect("failed to run Prelay desktop client");
}

use prelay_client::commands::settings::{DesktopPreferencesInput, DesktopPreferencesResponse};
use prelay_client::desktop_preferences::{
    DesktopPreferences, DesktopPreferencesStore, FileDesktopPreferencesStore, ThemeMode,
};
use serde_json::json;
use tempfile::tempdir;

#[test]
fn desktop_preferences_ipc_uses_the_client_camel_case_contract() {
    let response = serde_json::to_value(DesktopPreferencesResponse {
        theme: ThemeMode::System,
        autostart_enabled: false,
        silent_start: false,
        minimize_to_tray: true,
    })
    .expect("serialize desktop preferences response");
    let input: DesktopPreferencesInput = serde_json::from_value(json!({
        "theme": "dark",
        "autostartEnabled": true,
        "silentStart": true,
        "minimizeToTray": false,
    }))
    .expect("deserialize desktop preferences input");

    assert_eq!(
        response,
        json!({
            "theme": "system",
            "autostartEnabled": false,
            "silentStart": false,
            "minimizeToTray": true,
        })
    );
    assert!(input.autostart_enabled);
    assert!(input.silent_start);
    assert!(!input.minimize_to_tray);
}

#[test]
fn desktop_preferences_default_to_system_theme_and_tray_minimization() {
    let directory = tempdir().expect("create preferences directory");
    let store = FileDesktopPreferencesStore::at(directory.path().join("desktop-preferences.json"));

    assert_eq!(
        store.load().expect("load default preferences"),
        DesktopPreferences {
            theme: ThemeMode::System,
            silent_start: true,
            minimize_to_tray: true,
        }
    );
}

#[test]
fn desktop_preferences_round_trip() {
    let directory = tempdir().expect("create preferences directory");
    let store = FileDesktopPreferencesStore::at(directory.path().join("desktop-preferences.json"));
    let preferences = DesktopPreferences {
        theme: ThemeMode::Dark,
        silent_start: true,
        minimize_to_tray: false,
    };

    store.save(&preferences).expect("save preferences");

    assert_eq!(store.load().expect("load saved preferences"), preferences);
}

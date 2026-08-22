use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

use crate::desktop_preferences::DesktopPreferencesStore;

pub fn is_enabled(app: &AppHandle) -> Result<bool, String> {
    let manager = app.autolaunch();
    manager
        .is_enabled()
        .map_err(|error| format!("unable to read autostart state: {error}"))
}

pub fn set_enabled(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let manager = app.autolaunch();
    if enabled {
        manager
            .enable()
            .map_err(|error| format!("unable to enable autostart: {error}"))?;
    } else {
        manager
            .disable()
            .map_err(|error| format!("unable to disable autostart: {error}"))?;
    }
    Ok(())
}

pub fn should_start_silently(preferences: &impl DesktopPreferencesStore) -> Result<bool, String> {
    Ok(std::env::args().any(|argument| argument == "--autostart")
        && preferences.load()?.silent_start)
}

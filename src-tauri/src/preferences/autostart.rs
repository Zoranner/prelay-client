use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

use super::desktop::{DesktopPreferences, DesktopPreferencesStore};

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

pub fn apply_default_when_preferences_are_missing(
    app: &AppHandle,
    preferences: &impl DesktopPreferencesStore,
) {
    match save_default_preferences_if_missing(preferences) {
        Ok(false) => {}
        Ok(true) => {
            if let Err(error) = set_enabled(app, true) {
                eprintln!("failed to enable Prelay autostart: {error}");
            }
        }
        Err(error) => eprintln!("failed to initialize Prelay desktop preferences: {error}"),
    }
}

fn save_default_preferences_if_missing(
    preferences: &impl DesktopPreferencesStore,
) -> Result<bool, String> {
    if preferences.exists()? {
        return Ok(false);
    }

    preferences.save(&DesktopPreferences::default())?;
    Ok(true)
}

pub fn should_start_silently(preferences: &impl DesktopPreferencesStore) -> Result<bool, String> {
    Ok(std::env::args().any(|argument| argument == "--autostart")
        && preferences.load()?.silent_start)
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    struct MemoryPreferencesStore {
        exists: Mutex<bool>,
        saved: Mutex<Vec<DesktopPreferences>>,
    }

    impl MemoryPreferencesStore {
        fn missing() -> Self {
            Self {
                exists: Mutex::new(false),
                saved: Mutex::new(Vec::new()),
            }
        }

        fn existing() -> Self {
            Self {
                exists: Mutex::new(true),
                saved: Mutex::new(Vec::new()),
            }
        }
    }

    impl DesktopPreferencesStore for MemoryPreferencesStore {
        fn exists(&self) -> Result<bool, String> {
            Ok(*self.exists.lock().expect("lock exists state"))
        }

        fn load(&self) -> Result<DesktopPreferences, String> {
            Ok(DesktopPreferences::default())
        }

        fn save(&self, preferences: &DesktopPreferences) -> Result<(), String> {
            *self.exists.lock().expect("lock exists state") = true;
            self.saved
                .lock()
                .expect("lock saved preferences")
                .push(preferences.clone());
            Ok(())
        }
    }

    #[test]
    fn missing_preferences_are_saved_before_autostart_is_attempted() {
        let preferences = MemoryPreferencesStore::missing();

        assert!(save_default_preferences_if_missing(&preferences).expect("save defaults"));
        assert_eq!(
            preferences
                .saved
                .lock()
                .expect("lock saved preferences")
                .as_slice(),
            &[DesktopPreferences::default()]
        );
        assert!(!save_default_preferences_if_missing(&preferences).expect("keep defaults"));
    }

    #[test]
    fn existing_preferences_are_not_replaced() {
        let preferences = MemoryPreferencesStore::existing();

        assert!(!save_default_preferences_if_missing(&preferences).expect("keep preferences"));
        assert!(preferences
            .saved
            .lock()
            .expect("lock saved preferences")
            .is_empty());
    }
}

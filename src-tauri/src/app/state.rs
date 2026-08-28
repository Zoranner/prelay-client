use crate::{
    identity::{credentials::FileCredentialStore, windows::WindowsIdentitySource},
    preferences::desktop::FileDesktopPreferencesStore,
    relay::{client::RegistrationGate, settings::FileRelaySettingsStore},
};

pub struct NativeState {
    pub identity: WindowsIdentitySource,
    pub credentials: FileCredentialStore,
    pub desktop_preferences: FileDesktopPreferencesStore,
    pub relay_settings: FileRelaySettingsStore,
    pub registration_gate: RegistrationGate,
    pub credential_lifecycle_gate: tokio::sync::Mutex<()>,
}

impl NativeState {
    pub fn for_app_data_dir(app_data_dir: std::path::PathBuf) -> Self {
        Self {
            identity: WindowsIdentitySource,
            credentials: FileCredentialStore::at(
                app_data_dir.join("Prelay").join("device-credential.json"),
            ),
            desktop_preferences: FileDesktopPreferencesStore::at(
                app_data_dir.join("Prelay").join("desktop-preferences.json"),
            ),
            relay_settings: FileRelaySettingsStore::at(
                app_data_dir.join("Prelay").join("relay-settings.json"),
            ),
            registration_gate: RegistrationGate::default(),
            credential_lifecycle_gate: tokio::sync::Mutex::new(()),
        }
    }
}

impl Default for NativeState {
    fn default() -> Self {
        Self::for_app_data_dir(std::env::temp_dir())
    }
}

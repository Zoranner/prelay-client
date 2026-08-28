//! Prelay desktop client crate.

pub mod agents;
pub mod app;
pub mod commands;
pub mod extensions;
pub mod identity;
pub mod preferences;
pub mod relay;

pub use app::NativeState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    app::run();
}

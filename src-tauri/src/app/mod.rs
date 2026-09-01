use tauri::Manager;

pub mod state;
pub mod update;

pub use state::NativeState;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            crate::preferences::tray::show_main_window(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .invoke_handler(tauri::generate_handler![
            crate::commands::bootstrap::bootstrap,
            update::client_update_prepare,
            update::client_update_install,
            crate::commands::settings::relay_settings_get,
            crate::commands::settings::relay_settings_save,
            crate::commands::settings::relay_settings_connect,
            crate::commands::settings::desktop_preferences_get,
            crate::commands::settings::desktop_preferences_save,
            crate::commands::providers::providers_list,
            crate::commands::providers::providers_save,
            crate::commands::providers::providers_delete,
            crate::commands::providers::providers_ping,
            crate::commands::providers::providers_discover_models,
            crate::commands::providers::providers_test_protocol,
            crate::commands::endpoints::endpoints_list,
            crate::commands::endpoints::endpoints_save,
            crate::commands::endpoints::endpoints_delete,
            crate::commands::endpoints::endpoints_regenerate_token,
            crate::commands::agents::agents_status,
            crate::commands::agents::agent_items_get,
            crate::commands::agents::agents_remove,
            crate::commands::agents::agent_settings_get,
            crate::commands::agents::agent_settings_save,
            crate::commands::extensions::extensions_list,
            crate::commands::extensions::extension_readme,
            crate::commands::extensions::extensions_install,
            crate::commands::stats::stats_overview,
            crate::commands::stats::stats_timeline,
            crate::commands::stats::stats_activities,
            crate::commands::stats::stats_models,
            crate::commands::stats::stats_providers,
            crate::commands::stats::stats_leaderboard,
            crate::commands::identity::credential_rotate
        ])
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let state = NativeState::for_app_data_dir(app_data_dir);
            crate::preferences::autostart::apply_default_when_preferences_are_missing(
                app.handle(),
                &state.desktop_preferences,
            );
            let start_silently = match crate::preferences::autostart::should_start_silently(
                &state.desktop_preferences,
            ) {
                Ok(start_silently) => start_silently,
                Err(error) => {
                    eprintln!("failed to read Prelay desktop preferences: {error}");
                    false
                }
            };
            app.manage(state);
            crate::preferences::tray::install(app.handle())?;
            if !start_silently {
                crate::preferences::tray::show_main_window(app.handle());
            }
            Ok(())
        })
        .on_window_event(crate::preferences::tray::hide_on_close)
        .run(tauri::generate_context!())
        .expect("failed to run Prelay desktop client");
}

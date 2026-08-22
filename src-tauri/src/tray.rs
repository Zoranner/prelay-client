use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime, WindowEvent,
};

use crate::desktop_preferences::DesktopPreferencesStore;

const SHOW_MENU_ID: &str = "show";
const SETTINGS_MENU_ID: &str = "settings";
const QUIT_MENU_ID: &str = "quit";
const TRAY_OPEN_SETTINGS_EVENT: &str = "tray:open-settings";

pub fn install<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let version = MenuItem::with_id(
        app,
        "version",
        format!("Prelay v{}", app.package_info().version),
        false,
        None::<&str>,
    )?;
    let show = MenuItem::with_id(app, SHOW_MENU_ID, "显示窗口", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, SETTINGS_MENU_ID, "设置", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT_MENU_ID, "退出", true, None::<&str>)?;
    let header_separator = PredefinedMenuItem::separator(app)?;
    let exit_separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[
            &version,
            &header_separator,
            &show,
            &settings,
            &exit_separator,
            &quit,
        ],
    )?;
    let tray_icon = app
        .default_window_icon()
        .cloned()
        .expect("the packaged Prelay icon should be available at runtime");

    TrayIconBuilder::with_id("prelay")
        .icon(tray_icon)
        .tooltip("Prelay")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if matches!(event, TrayIconEvent::DoubleClick { .. }) {
                show_main_window(tray.app_handle());
            }
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
            SHOW_MENU_ID => show_main_window(app),
            SETTINGS_MENU_ID => {
                show_main_window(app);
                app.emit(TRAY_OPEN_SETTINGS_EVENT, ())
                    .expect("the tray navigation event name should be valid");
            }
            QUIT_MENU_ID => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}

pub fn hide_on_close<R: Runtime>(window: &tauri::Window<R>, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        let preferences = window.state::<crate::NativeState>();
        let minimize_to_tray = preferences
            .desktop_preferences
            .load()
            .map(|preferences| preferences.minimize_to_tray)
            .unwrap_or(true);
        if minimize_to_tray {
            let _ = window.hide();
            api.prevent_close();
        } else {
            window.app_handle().exit(0);
        }
    }
}

pub fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

use prelay_client::NativeState;
use std::path::Path;

#[test]
fn native_state_has_a_default_constructor() {
    let _state = NativeState::default();
}

#[test]
fn tray_uses_the_packaged_default_icon() {
    let tray_source = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/preferences/tray.rs"),
    )
    .expect("tray source should be readable");

    assert!(tray_source.contains("default_window_icon()"));
    assert!(tray_source.contains(".icon(tray_icon)"));
}

#[test]
fn tray_restores_a_minimized_main_window_before_focusing_it() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/preferences/tray.rs"
    ))
    .expect("read tray implementation");

    let unminimize = source
        .find("window.unminimize()")
        .expect("unminimize main window");
    let show = source.find("window.show()").expect("show main window");
    let focus = source
        .find("window.set_focus()")
        .expect("focus main window");

    assert!(unminimize < show);
    assert!(show < focus);
}

#[test]
fn tray_double_click_restores_the_main_window() {
    let tray_source = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/preferences/tray.rs"),
    )
    .expect("tray source should be readable");

    assert!(tray_source.contains("TrayIconEvent::DoubleClick"));
    assert!(tray_source.contains("show_main_window(tray.app_handle())"));
    assert!(tray_source.contains("api.prevent_close()"));
}

#[test]
fn tray_menu_opens_only_on_right_click() {
    let tray_source = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/preferences/tray.rs"),
    )
    .expect("tray source should be readable");

    assert!(tray_source.contains(".show_menu_on_left_click(false)"));
}

#[test]
fn tray_menu_groups_version_window_settings_and_exit() {
    let tray_source = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/preferences/tray.rs"),
    )
    .expect("tray source should be readable");

    assert!(tray_source.contains("Prelay v{}"));
    assert!(tray_source.contains("\"version\""));
    assert!(tray_source.contains("false,"));
    assert_eq!(
        tray_source
            .matches("PredefinedMenuItem::separator(app)?")
            .count(),
        2
    );
    assert!(tray_source.contains("SETTINGS_MENU_ID"));
    assert!(tray_source.contains("TRAY_OPEN_SETTINGS_EVENT"));
    assert!(tray_source.contains("tray:open-settings"));
    assert!(!tray_source.contains("ABOUT_MENU_ID"));
}

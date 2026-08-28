use std::fs;

#[test]
fn desktop_client_activates_the_existing_window_when_started_again() {
    let source = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/app/mod.rs"))
        .expect("read desktop client entrypoint");

    assert!(source.contains("tauri_plugin_single_instance::init"));
    assert!(source.contains("preferences::tray::show_main_window(app)"));
}

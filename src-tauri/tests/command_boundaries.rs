use std::{fs, path::Path};

#[test]
fn command_module_only_declares_command_handlers() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let command_module =
        fs::read_to_string(source_root.join("commands/mod.rs")).expect("read command module");

    assert!(!command_module.contains("authenticated_api"));
    assert!(!command_module.contains("credential_lifecycle_gate"));
    assert!(!command_module.contains("credential_rotate"));
}

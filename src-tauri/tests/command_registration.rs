use std::{fs, path::PathBuf};

fn source_file(relative_path: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    fs::read_to_string(path).unwrap().replace("\r\n", "\n")
}

#[test]
fn registers_the_bootstrap_command_without_a_legacy_alias() {
    let application = source_file("src/app/mod.rs");
    let command = source_file("src/commands/bootstrap.rs");

    assert!(application.contains("crate::commands::bootstrap::bootstrap"));
    assert!(!application.contains("bootstrap_client"));
    assert!(command.contains("#[tauri::command]\npub async fn bootstrap("));
    assert!(!command.contains("bootstrap_client"));
}

#[test]
fn bootstrap_does_not_access_the_credential_store_directly() {
    let command = source_file("src/commands/bootstrap.rs");

    assert!(!command.contains("credential_store"));
    assert!(!command.contains("CredentialStore"));
    assert!(command.contains("relay::client::{ApiClient, ClientError}"));
    assert!(command.contains("has_stored_credential"));
}

#[test]
fn registers_the_authenticated_provider_catalog_command() {
    let application = source_file("src/app/mod.rs");
    let command = source_file("src/commands/providers.rs");

    assert!(application.contains("crate::commands::providers::catalog_providers_list"));
    assert!(command.contains("pub async fn catalog_providers_list("));
    assert!(command.contains(".get(\"/api/catalog/providers\")"));
}

#[test]
fn registers_the_complete_authenticated_catalog_command() {
    let application = source_file("src/app/mod.rs");
    let command = source_file("src/commands/providers.rs");

    assert!(application.contains("crate::commands::providers::catalog_models_get"));
    assert!(command.contains("pub async fn catalog_models_get("));
    assert!(command.contains(".get(\"/api/catalog\")"));
}

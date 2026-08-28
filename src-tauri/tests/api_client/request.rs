use prelay_client::{
    identity::credentials::MemoryCredentialStore,
    relay::client::{ApiClient, ClientError},
};

use super::support::one_response_server;

#[test]
fn management_request_reads_credential_store_and_uses_bearer_authorization() {
    let store = MemoryCredentialStore::with_record("device-secret", None);
    let client = ApiClient::new("https://relay.example.test/", &store).expect("create client");

    let request = client
        .authenticated_request("GET", "/api/providers")
        .expect("build authenticated request");

    assert_eq!(request.url(), "https://relay.example.test/api/providers");
    assert_eq!(request.authorization(), "Bearer device-secret");
}

#[test]
fn management_request_without_credential_returns_stable_error_code() {
    let store = MemoryCredentialStore::default();
    let client = ApiClient::new("https://relay.example.test", &store).expect("create client");

    let error = client
        .authenticated_request("GET", "/api/providers")
        .expect_err("missing credential must not create an unauthenticated request");

    assert_eq!(error.code(), ClientError::MISSING_DEVICE_CREDENTIAL);
}

#[test]
fn api_client_reports_whether_a_device_credential_is_stored() {
    let empty_store = MemoryCredentialStore::default();
    let empty_client =
        ApiClient::new("https://relay.example.test", &empty_store).expect("create client");
    assert!(!empty_client
        .has_stored_credential()
        .expect("read empty credential store"));

    let populated_store = MemoryCredentialStore::with_record("device-secret", None);
    let populated_client =
        ApiClient::new("https://relay.example.test", &populated_store).expect("create client");
    assert!(populated_client
        .has_stored_credential()
        .expect("read populated credential store"));

    let empty_value_store = MemoryCredentialStore::with_record("   ", None);
    let empty_value_client =
        ApiClient::new("https://relay.example.test", &empty_value_store).expect("create client");
    assert!(!empty_value_client
        .has_stored_credential()
        .expect("empty credential value is unavailable"));
}

#[test]
fn management_request_preserves_nested_server_error_message() {
    let (base_url, server) = one_response_server(
        "400 Bad Request",
        r#"{"error":{"code":"unsupported_protocol","message":"provider does not support messages"}}"#,
        |request| assert!(request.starts_with("POST /api/providers HTTP/1.1")),
    );
    let store = MemoryCredentialStore::with_record("device-secret", None);
    let client = ApiClient::new(base_url, &store).expect("create client");

    let error = tauri::async_runtime::block_on(
        client.post::<serde_json::Value, _>("/api/providers", &serde_json::json!({})),
    )
    .expect_err("server validation failure must be returned");
    server.join().expect("join test relay");

    assert_eq!(error.code(), "unsupported_protocol");
    assert_eq!(error.message, "provider does not support messages");
}

#[test]
fn management_request_preserves_string_server_error() {
    let (base_url, server) = one_response_server(
        "400 Bad Request",
        r#"{"error":"provider does not have any models"}"#,
        |request| assert!(request.starts_with("POST /api/providers HTTP/1.1")),
    );
    let store = MemoryCredentialStore::with_record("device-secret", None);
    let client = ApiClient::new(base_url, &store).expect("create client");

    let error = tauri::async_runtime::block_on(
        client.post::<serde_json::Value, _>("/api/providers", &serde_json::json!({})),
    )
    .expect_err("server validation failure must be returned");
    server.join().expect("join test relay");

    assert_eq!(error.code(), "validation_failed");
    assert_eq!(error.message, "provider does not have any models");
}

#[test]
fn management_request_hides_structured_server_error_message_on_internal_failure() {
    let (base_url, server) = one_response_server(
        "500 Internal Server Error",
        r#"{"error":{"code":"internal","message":"database error: no such table: identity_provider_configs"}}"#,
        |request| assert!(request.starts_with("POST /api/providers HTTP/1.1")),
    );
    let store = MemoryCredentialStore::with_record("device-secret", None);
    let client = ApiClient::new(base_url, &store).expect("create client");

    let error = tauri::async_runtime::block_on(
        client.post::<serde_json::Value, _>("/api/providers", &serde_json::json!({})),
    )
    .expect_err("server internal failure must be returned safely");
    server.join().expect("join test relay");

    assert_eq!(error.code(), "internal");
    assert_eq!(
        error.message,
        "management API returned HTTP 500 Internal Server Error"
    );
}

#[test]
fn management_request_uses_safe_fallback_for_empty_string_server_error() {
    let (base_url, server) =
        one_response_server("400 Bad Request", r#"{"error":"   "}"#, |request| {
            assert!(request.starts_with("POST /api/providers HTTP/1.1"))
        });
    let store = MemoryCredentialStore::with_record("device-secret", None);
    let client = ApiClient::new(base_url, &store).expect("create client");

    let error = tauri::async_runtime::block_on(
        client.post::<serde_json::Value, _>("/api/providers", &serde_json::json!({})),
    )
    .expect_err("server validation failure must be returned");
    server.join().expect("join test relay");

    assert_eq!(error.code(), "validation_failed");
    assert_eq!(
        error.message,
        "management API returned HTTP 400 Bad Request"
    );
}

#[test]
fn management_request_includes_http_reason_when_server_omits_the_error_body() {
    let (base_url, server) = one_response_server("405 Method Not Allowed", "", |request| {
        assert!(request.starts_with("POST /api/providers HTTP/1.1"))
    });
    let store = MemoryCredentialStore::with_record("device-secret", None);
    let client = ApiClient::new(base_url, &store).expect("create client");

    let error = tauri::async_runtime::block_on(
        client.post::<serde_json::Value, _>("/api/providers", &serde_json::json!({})),
    )
    .expect_err("method rejection must be returned");
    server.join().expect("join test relay");

    assert_eq!(error.code(), "internal");
    assert_eq!(
        error.message,
        "management API returned HTTP 405 Method Not Allowed"
    );
}

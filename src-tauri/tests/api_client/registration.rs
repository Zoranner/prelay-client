use prelay_client::{
    identity::credentials::{CredentialStore, MemoryCredentialStore},
    relay::client::{ApiClient, RegistrationGate},
};
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Barrier,
    },
    thread,
};

use super::support::{
    assert_header_value, identity, one_response_server, registration_server,
    retry_registration_server, two_registration_response_server, two_response_server,
};

#[test]
fn first_registration_persists_and_sends_a_client_generated_credential() {
    let (base_url, server) = one_response_server(
        "201 Created",
        r#"{"identity_id":"identity-a","created":true}"#,
        |request| {
            assert!(request.starts_with("POST /api/identities HTTP/1.1"));
            assert!(!request.contains("Authorization:"));
            assert!(request.contains("\"machine_id\":\"machine-a\""));
            assert!(request.contains("\"account_sid\":\"S-1-5-21-100\""));
            assert!(request.contains("\"credential\":\""));
            assert!(request.contains("\"display_name\":\"Ada\""));
            assert!(request
                .to_ascii_lowercase()
                .contains("x-prelay-display-name: qwrh"));
        },
    );
    let store = MemoryCredentialStore::default();
    let client = ApiClient::new(base_url, &store)
        .expect("create client")
        .with_display_name("Ada");

    tauri::async_runtime::block_on(client.ensure_registered(&identity()))
        .expect("register identity");
    server.join().expect("join test relay");

    let credential = store
        .load()
        .expect("load credential")
        .expect("client credential is persisted");
    assert!(credential.current.len() >= 43);
}

#[test]
fn authenticated_management_request_sends_the_display_name() {
    let (base_url, server) = one_response_server("200 OK", r#"{}"#, |request| {
        assert_header_value(request, "authorization", "Bearer device-secret");
        assert!(request
            .to_ascii_lowercase()
            .contains("x-prelay-display-name: 5l2g5aw9"));
    });
    let store = MemoryCredentialStore::with_record("device-secret", None);
    let client = ApiClient::new(base_url, &store)
        .expect("create client")
        .with_display_name("你好");

    tauri::async_runtime::block_on(client.get::<serde_json::Value>("/api/providers"))
        .expect("authenticated request succeeds");
    server.join().expect("join test relay");
}

#[test]
fn missing_local_credential_registers_against_existing_identity() {
    let (base_url, server) = one_response_server(
        "200 OK",
        r#"{"identity_id":"identity-a","created":false}"#,
        |request| {
            assert!(request.starts_with("POST /api/identities HTTP/1.1"));
            assert!(request.contains("\"credential\":\""));
        },
    );
    let store = MemoryCredentialStore::default();
    let client = ApiClient::new(base_url, &store).expect("create client");

    tauri::async_runtime::block_on(client.ensure_registered(&identity()))
        .expect("existing identity accepts a newly generated credential");
    server.join().expect("join test relay");

    assert!(
        store
            .load()
            .expect("load credential")
            .expect("persisted credential")
            .current
            .len()
            >= 43
    );
}

#[test]
fn stored_credential_retries_registration_with_the_same_identity() {
    let requests = Arc::new(AtomicUsize::new(0));
    let (base_url, server) = registration_server(requests.clone());
    let store = MemoryCredentialStore::with_record("persisted-device-secret", None);
    let client = ApiClient::new(base_url, &store).expect("create client");

    tauri::async_runtime::block_on(client.ensure_registered(&identity()))
        .expect("stored credential registration retry succeeds");
    let captured_requests = server.join().expect("join test relay");

    assert_eq!(requests.load(Ordering::SeqCst), 1);
    assert_eq!(
        store.load().expect("load credential"),
        Some(prelay_client::identity::credentials::CredentialRecord {
            current: "persisted-device-secret".into(),
            pending: None,
        })
    );
    let request = captured_requests
        .first()
        .expect("registration request is captured");
    assert!(request.contains("\"machine_id\":\"machine-a\""));
    assert!(request.contains("\"account_sid\":\"S-1-5-21-100\""));
    assert!(request.contains("\"credential\":\"persisted-device-secret\""));
}

#[test]
fn failed_registration_retries_with_the_persisted_credential() {
    let (base_url, server) = retry_registration_server();
    let store = MemoryCredentialStore::default();
    let gate = RegistrationGate::default();
    let client = ApiClient::new(base_url, &store).expect("create client");

    tauri::async_runtime::block_on(client.ensure_registered_once(&identity(), &gate))
        .expect_err("dropped registration response must fail");

    tauri::async_runtime::block_on(client.ensure_registered_once(&identity(), &gate))
        .expect("persisted credential registration retry succeeds");
    let requests = server.join().expect("join test relay");
    let credential = store
        .load()
        .expect("load credential")
        .expect("client credential is persisted");

    assert_eq!(requests.len(), 2);
    for request in requests {
        assert!(request.contains("\"machine_id\":\"machine-a\""));
        assert!(request.contains("\"account_sid\":\"S-1-5-21-100\""));
        assert!(request.contains(&format!("\"credential\":\"{}\"", credential.current)));
    }
}

#[test]
fn registration_gate_serializes_calls_without_caching_registration_state() {
    let (base_url, server) = two_registration_response_server();
    let store = MemoryCredentialStore::with_record("persisted-device-secret", None);
    let gate = RegistrationGate::default();
    let client = ApiClient::new(base_url, &store).expect("create client");

    tauri::async_runtime::block_on(client.ensure_registered_once(&identity(), &gate))
        .expect("first registration confirmation succeeds");
    tauri::async_runtime::block_on(client.ensure_registered_once(&identity(), &gate))
        .expect("second registration confirmation succeeds");

    assert_eq!(server.join().expect("join test relay"), 2);
}

#[test]
fn pending_credential_registration_confirms_a_rotation_whose_response_was_lost() {
    let (base_url, server) = two_response_server(
        [
            ("200 OK", r#"{"identity_id":"identity-a","created":false}"#),
            ("200 OK", r#"{}"#),
        ],
        |requests| {
            assert!(requests[0].starts_with("POST /api/identities HTTP/1.1"));
            assert!(requests[0].contains("\"credential\":\"credential-new\""));
            assert_header_value(&requests[1], "authorization", "Bearer credential-new");
        },
    );
    let store = MemoryCredentialStore::with_record("credential-old", Some("credential-new"));
    let client = ApiClient::new(base_url, &store).expect("create client");

    tauri::async_runtime::block_on(client.ensure_registered(&identity()))
        .expect("pending credential confirms existing identity");
    tauri::async_runtime::block_on(client.get::<serde_json::Value>("/api/providers"))
        .expect("confirmed credential authenticates requests");
    server.join().expect("join test relay");

    assert_eq!(
        store.load().expect("load credential"),
        Some(prelay_client::identity::credentials::CredentialRecord {
            current: "credential-new".into(),
            pending: None,
        })
    );
}

#[test]
fn rejected_pending_registration_falls_back_to_current_credential() {
    let (base_url, server) = two_response_server(
        [
            (
                "400 Bad Request",
                r#"{"error":{"code":"identity_already_registered","message":"already registered"}}"#,
            ),
            ("200 OK", r#"{"identity_id":"identity-a","created":false}"#),
        ],
        |requests| {
            assert!(requests[0].contains("\"credential\":\"credential-new\""));
            assert!(requests[1].contains("\"credential\":\"credential-old\""));
        },
    );
    let store = MemoryCredentialStore::with_record("credential-old", Some("credential-new"));
    let client = ApiClient::new(base_url, &store).expect("create client");

    tauri::async_runtime::block_on(client.ensure_registered(&identity()))
        .expect("current credential confirms the unrotated identity");
    server.join().expect("join test relay");

    assert_eq!(
        store.load().expect("load credential"),
        Some(prelay_client::identity::credentials::CredentialRecord {
            current: "credential-old".into(),
            pending: None,
        })
    );
}

#[test]
fn accepted_pending_credential_becomes_current_after_an_authenticated_request() {
    let (base_url, server) = one_response_server("200 OK", r#"{}"#, |request| {
        assert_header_value(request, "authorization", "Bearer credential-new");
    });
    let store = MemoryCredentialStore::with_record("credential-old", Some("credential-new"));
    let client = ApiClient::new(base_url, &store).expect("create client");

    tauri::async_runtime::block_on(client.get::<serde_json::Value>("/api/providers"))
        .expect("pending credential is accepted");
    server.join().expect("join test relay");

    assert_eq!(
        store.load().expect("load credential"),
        Some(prelay_client::identity::credentials::CredentialRecord {
            current: "credential-new".into(),
            pending: None,
        })
    );
}

#[test]
fn rejected_pending_credential_falls_back_to_current_and_is_discarded() {
    let (base_url, server) = two_response_server(
        [
            ("401 Unauthorized", r#"{"error":"invalid credential"}"#),
            ("200 OK", r#"{}"#),
        ],
        |requests| {
            assert_header_value(&requests[0], "authorization", "Bearer credential-new");
            assert_header_value(&requests[1], "authorization", "Bearer credential-old");
        },
    );
    let store = MemoryCredentialStore::with_record("credential-old", Some("credential-new"));
    let client = ApiClient::new(base_url, &store).expect("create client");

    tauri::async_runtime::block_on(client.get::<serde_json::Value>("/api/providers"))
        .expect("current credential recovers the request");
    server.join().expect("join test relay");

    assert_eq!(
        store.load().expect("load credential"),
        Some(prelay_client::identity::credentials::CredentialRecord {
            current: "credential-old".into(),
            pending: None,
        })
    );
}

#[test]
fn server_failure_preserves_pending_credential_for_later_recovery() {
    let (base_url, server) = one_response_server(
        "500 Internal Server Error",
        r#"{"error":{"code":"internal","message":"ignored"}}"#,
        |request| assert_header_value(request, "authorization", "Bearer credential-new"),
    );
    let store = MemoryCredentialStore::with_record("credential-old", Some("credential-new"));
    let client = ApiClient::new(base_url, &store).expect("create client");

    let error = tauri::async_runtime::block_on(client.get::<serde_json::Value>("/api/providers"))
        .expect_err("server failure must fail the request");
    server.join().expect("join test relay");

    assert_eq!(error.code(), "internal");
    assert_eq!(
        store.load().expect("load credential"),
        Some(prelay_client::identity::credentials::CredentialRecord {
            current: "credential-old".into(),
            pending: Some("credential-new".into()),
        })
    );
}

#[test]
fn concurrent_registration_confirmation_is_serialized_without_being_cached() {
    let (base_url, server) = two_registration_response_server();
    let store = Arc::new(MemoryCredentialStore::default());
    let gate = Arc::new(RegistrationGate::default());
    let barrier = Arc::new(Barrier::new(3));

    let workers = (0..2)
        .map(|_| {
            let base_url = base_url.clone();
            let store = store.clone();
            let gate = gate.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                let client = ApiClient::new(base_url, store.as_ref()).expect("create client");
                barrier.wait();
                tauri::async_runtime::block_on(
                    client.ensure_registered_once(&identity(), gate.as_ref()),
                )
            })
        })
        .collect::<Vec<_>>();
    barrier.wait();

    for worker in workers {
        worker
            .join()
            .expect("join registration worker")
            .expect("concurrent registration succeeds");
    }
    assert_eq!(server.join().expect("join test relay"), 2);
    assert!(
        store
            .load()
            .expect("load credential")
            .expect("client credential is persisted")
            .current
            .len()
            >= 43
    );
}

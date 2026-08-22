use std::{
    io::{Read, Write},
    net::TcpListener,
};

use prelay_client::{
    commands::settings::connect_and_save_relay_settings,
    relay_settings::{FileRelaySettingsStore, RelaySettingsStore},
    NativeState,
};
use tempfile::tempdir;

#[test]
fn relay_settings_persists_the_selected_management_url() {
    let directory = tempdir().expect("create settings directory");
    let store = FileRelaySettingsStore::at(directory.path().join("relay-settings.json"));

    assert_eq!(store.load().expect("load empty settings"), None);

    store
        .save("https://relay.example.test")
        .expect("save relay URL");

    assert_eq!(
        store.load().expect("load saved settings").as_deref(),
        Some("https://relay.example.test")
    );
}

#[cfg(windows)]
#[test]
fn failed_initial_registration_does_not_persist_the_relay_url() {
    tauri::async_runtime::block_on(async {
        let directory = tempdir().expect("create settings directory");
        let state = NativeState::for_app_data_dir(directory.path().to_path_buf());
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test relay");
        let relay_url = format!(
            "http://{}",
            listener.local_addr().expect("read relay address")
        );
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept registration request");
            let request = read_http_request(&mut stream);
            assert!(request.starts_with("POST /api/identities HTTP/1.1"));
            assert!(request.contains("\"machine_id\":"));
            assert!(request.contains("\"account_sid\":"));
            assert!(request.contains("\"credential\":"));
            let body = r#"{"error":{"code":"validation_failed","message":"rejected"}}"#;
            let response = format!(
                "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream
                .write_all(response.as_bytes())
                .expect("write registration response");
        });

        let error = connect_and_save_relay_settings(&state, &format!("  {relay_url}/  "))
            .await
            .expect_err("rejected registration must fail the initial connection");
        server.join().expect("join test relay");

        assert_eq!(error.code(), "validation_failed");
        assert_eq!(
            state
                .relay_settings
                .load()
                .expect("read relay settings after a failed registration"),
            None
        );
    });
}

#[cfg(windows)]
#[test]
fn successful_initial_registration_persists_the_normalized_relay_url() {
    tauri::async_runtime::block_on(async {
        let directory = tempdir().expect("create settings directory");
        let state = NativeState::for_app_data_dir(directory.path().to_path_buf());
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test relay");
        let relay_url = format!(
            "http://{}",
            listener.local_addr().expect("read relay address")
        );
        let server = std::thread::spawn(move || {
            let (mut registration_stream, _) =
                listener.accept().expect("accept registration request");
            let registration = read_http_request(&mut registration_stream);
            assert!(registration.starts_with("POST /api/identities HTTP/1.1"));
            let registration_body = r#"{"identity_id":"identity-a","created":true}"#;
            let registration_response = format!(
                "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{registration_body}",
                registration_body.len()
            );
            registration_stream
                .write_all(registration_response.as_bytes())
                .expect("write registration response");

            let (mut management_stream, _) = listener.accept().expect("accept management request");
            let management_request = read_http_request(&mut management_stream);
            assert!(management_request.starts_with("GET /api/providers HTTP/1.1"));
            assert!(management_request.contains("Authorization: Bearer "));
            let management_body = "[]";
            let management_response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{management_body}",
                management_body.len()
            );
            management_stream
                .write_all(management_response.as_bytes())
                .expect("write management response");
        });

        let response = connect_and_save_relay_settings(&state, &format!("  {relay_url}/  "))
            .await
            .expect("registered identity must save the relay URL");
        server.join().expect("join test relay");

        assert_eq!(response.relay_url.as_deref(), Some(relay_url.as_str()));
        assert_eq!(
            state
                .relay_settings
                .load()
                .expect("read saved relay settings")
                .as_deref(),
            Some(relay_url.as_str())
        );
    });
}

#[cfg(windows)]
#[test]
fn management_connection_failure_does_not_persist_the_relay_url() {
    tauri::async_runtime::block_on(async {
        let directory = tempdir().expect("create settings directory");
        let state = NativeState::for_app_data_dir(directory.path().to_path_buf());
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test relay");
        let relay_url = format!(
            "http://{}",
            listener.local_addr().expect("read relay address")
        );
        let server = std::thread::spawn(move || {
            let (mut registration_stream, _) = listener.accept().expect("accept registration");
            let registration = read_http_request(&mut registration_stream);
            assert!(registration.starts_with("POST /api/identities HTTP/1.1"));
            let registration_body = r#"{"identity_id":"identity-a","created":true}"#;
            let registration_response = format!(
                "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{registration_body}",
                registration_body.len()
            );
            registration_stream
                .write_all(registration_response.as_bytes())
                .expect("write registration response");

            let (mut management_stream, _) = listener.accept().expect("accept management request");
            let management_request = read_http_request(&mut management_stream);
            assert!(management_request.starts_with("GET /api/providers HTTP/1.1"));
            assert!(management_request.contains("Authorization: Bearer "));
            let management_body = r#"{"error":"unavailable"}"#;
            let management_response = format!(
                "HTTP/1.1 503 Service Unavailable\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{management_body}",
                management_body.len()
            );
            management_stream
                .write_all(management_response.as_bytes())
                .expect("write management response");
        });

        let error = connect_and_save_relay_settings(&state, &relay_url)
            .await
            .expect_err("unavailable management API must reject the connection");
        server.join().expect("join test relay");

        assert_eq!(error.code(), "internal");
        assert_eq!(
            state
                .relay_settings
                .load()
                .expect("read relay settings after a failed management connection"),
            None
        );
    });
}

fn read_http_request(stream: &mut std::net::TcpStream) -> String {
    let mut request = Vec::new();
    let mut chunk = [0_u8; 1024];
    let header_end = loop {
        let read = stream.read(&mut chunk).expect("read registration request");
        assert_ne!(read, 0, "registration request ended before its headers");
        request.extend_from_slice(&chunk[..read]);
        if let Some(index) = request.windows(4).position(|window| window == b"\r\n\r\n") {
            break index + 4;
        }
    };
    let headers = std::str::from_utf8(&request[..header_end]).expect("request headers are UTF-8");
    let content_length = headers
        .lines()
        .find_map(|line| line.strip_prefix("content-length: "))
        .or_else(|| {
            headers
                .lines()
                .find_map(|line| line.strip_prefix("Content-Length: "))
        })
        .map(|value| {
            value
                .parse::<usize>()
                .expect("request content length is numeric")
        })
        .unwrap_or_default();
    while request.len() < header_end + content_length {
        let read = stream
            .read(&mut chunk)
            .expect("read registration request body");
        assert_ne!(read, 0, "registration request ended before its body");
        request.extend_from_slice(&chunk[..read]);
    }
    String::from_utf8(request).expect("registration request is UTF-8")
}

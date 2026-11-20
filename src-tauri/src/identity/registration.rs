use std::ops::Deref;

use crate::{
    identity::{
        credentials::{CredentialStore, FileCredentialLifecycleGuard},
        windows::{IdentitySource, WindowsIdentity},
    },
    relay::{
        client::{generate_device_credential, ApiClient, ClientError},
        settings::RelaySettingsStore,
    },
    NativeState,
};

pub(crate) struct AuthenticatedApi<'a> {
    _credential_lifecycle_lease: CredentialLifecycleLease<'a>,
    _file_credential_lifecycle_guard: FileCredentialLifecycleGuard,
    client: ApiClient<'a>,
}

enum CredentialLifecycleLease<'a> {
    Shared {
        _guard: tokio::sync::RwLockReadGuard<'a, ()>,
    },
    Exclusive {
        _guard: tokio::sync::RwLockWriteGuard<'a, ()>,
    },
}

#[derive(Clone, Copy)]
enum CredentialLifecycleMode {
    Shared,
    Exclusive,
}

impl<'a> Deref for AuthenticatedApi<'a> {
    type Target = ApiClient<'a>;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

pub(crate) async fn authenticated_api(
    state: &NativeState,
) -> Result<AuthenticatedApi<'_>, ClientError> {
    let identity = state
        .identity
        .identity()
        .map_err(|error| ClientError::new("internal", error))?;
    let relay_url = state
        .relay_settings
        .load()
        .map_err(|error| ClientError::new("relay_settings_error", error))?
        .ok_or_else(|| {
            ClientError::new("relay_url_not_configured", "relay URL is not configured")
        })?;
    authenticated_api_with_identity(state, &identity, &relay_url).await
}

async fn authenticated_api_with_identity<'a>(
    state: &'a NativeState,
    identity: &WindowsIdentity,
    relay_url: &str,
) -> Result<AuthenticatedApi<'a>, ClientError> {
    let mode = credential_lifecycle_mode(state)?;
    authenticated_api_with_identity_mode(state, identity, relay_url, mode, false).await
}

async fn authenticated_api_with_identity_mode<'a>(
    state: &'a NativeState,
    identity: &WindowsIdentity,
    relay_url: &str,
    mode: CredentialLifecycleMode,
    refresh_registration: bool,
) -> Result<AuthenticatedApi<'a>, ClientError> {
    let (credential_lifecycle_lease, file_credential_lifecycle_guard) =
        acquire_credential_lifecycle_lease(state, mode).await?;
    if refresh_registration {
        state.registration_gate.clear().await;
    }
    let client = api_client(state, identity, relay_url)?;
    client
        .ensure_registered_once(identity, &state.registration_gate)
        .await?;
    Ok(AuthenticatedApi {
        _credential_lifecycle_lease: credential_lifecycle_lease,
        _file_credential_lifecycle_guard: file_credential_lifecycle_guard,
        client,
    })
}

pub(crate) async fn authenticated_api_with_identity_exclusive<'a>(
    state: &'a NativeState,
    identity: &WindowsIdentity,
    relay_url: &str,
) -> Result<AuthenticatedApi<'a>, ClientError> {
    authenticated_api_with_identity_mode(
        state,
        identity,
        relay_url,
        CredentialLifecycleMode::Exclusive,
        true,
    )
    .await
}

async fn authenticated_api_exclusive(
    state: &NativeState,
) -> Result<AuthenticatedApi<'_>, ClientError> {
    let identity = state
        .identity
        .identity()
        .map_err(|error| ClientError::new("internal", error))?;
    let relay_url = state
        .relay_settings
        .load()
        .map_err(|error| ClientError::new("relay_settings_error", error))?
        .ok_or_else(|| {
            ClientError::new("relay_url_not_configured", "relay URL is not configured")
        })?;
    authenticated_api_with_identity_exclusive(state, &identity, &relay_url).await
}

fn credential_lifecycle_mode(state: &NativeState) -> Result<CredentialLifecycleMode, ClientError> {
    let has_credential = state
        .credentials
        .load()
        .map_err(|error| ClientError::new("credential_store_error", error))?
        .is_some_and(|record| !record.current.trim().is_empty());
    Ok(if has_credential {
        CredentialLifecycleMode::Shared
    } else {
        CredentialLifecycleMode::Exclusive
    })
}

async fn acquire_credential_lifecycle_lease<'a>(
    state: &'a NativeState,
    mode: CredentialLifecycleMode,
) -> Result<(CredentialLifecycleLease<'a>, FileCredentialLifecycleGuard), ClientError> {
    match mode {
        CredentialLifecycleMode::Shared => {
            let guard = state.credential_lifecycle_gate.read().await;
            let file_guard = state
                .credentials
                .acquire_shared_lifecycle_lock()
                .await
                .map_err(|error| ClientError::new("credential_store_error", error))?;
            Ok((
                CredentialLifecycleLease::Shared { _guard: guard },
                file_guard,
            ))
        }
        CredentialLifecycleMode::Exclusive => {
            let guard = state.credential_lifecycle_gate.write().await;
            let file_guard = state
                .credentials
                .acquire_exclusive_lifecycle_lock()
                .await
                .map_err(|error| ClientError::new("credential_store_error", error))?;
            Ok((
                CredentialLifecycleLease::Exclusive { _guard: guard },
                file_guard,
            ))
        }
    }
}

fn api_client<'a>(
    state: &'a NativeState,
    identity: &WindowsIdentity,
    relay_url: &str,
) -> Result<ApiClient<'a>, ClientError> {
    Ok(ApiClient::new(relay_url, &state.credentials)?.with_display_name(&identity.display_name))
}

pub async fn rotate_credential(state: &NativeState) -> Result<(), ClientError> {
    let client = authenticated_api_exclusive(state).await?;
    let current_credential = state
        .credentials
        .load()
        .map_err(|error| ClientError::new("credential_store_error", error))?
        .filter(|record| !record.current.trim().is_empty())
        .ok_or_else(|| {
            ClientError::new(
                ClientError::MISSING_DEVICE_CREDENTIAL,
                "device credential is unavailable; identity cannot be restored automatically",
            )
        })?
        .current;
    let new_credential = generate_device_credential();
    state
        .credentials
        .begin_rotation(&new_credential)
        .map_err(|error| ClientError::new("credential_store_error", error))?;
    let response: prelay_protocol::RotateCredentialResponse = client
        .post_with_credential(
            "/api/identity/credential/rotate",
            &prelay_protocol::RotateCredentialRequest {
                new_credential: new_credential.clone(),
            },
            &current_credential,
        )
        .await?;
    debug_assert!(response.rotated);
    state
        .credentials
        .complete_rotation(&new_credential)
        .map_err(|error| ClientError::new("credential_store_error", error))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        sync::{mpsc, Arc},
        time::Duration,
    };

    use tokio::sync::oneshot;

    use super::authenticated_api_with_identity;
    use crate::{
        identity::{credentials::CredentialStore, windows::WindowsIdentity},
        NativeState,
    };
    use tempfile::tempdir;

    #[test]
    fn authenticated_api_allows_concurrent_readers_and_holds_write_until_they_drop() {
        tauri::async_runtime::block_on(async {
            let directory = tempdir().unwrap();
            let state = Arc::new(NativeState::for_app_data_dir(directory.path().into()));
            state.credentials.save_initial("credential-a").unwrap();
            let identity = WindowsIdentity {
                machine_id: "machine-a".into(),
                account_sid: "S-1-5-21-100".into(),
                display_name: "Ada".into(),
            };
            let (base_url, registration_events, server) = registration_server();
            let first = authenticated_api_with_identity(&state, &identity, &base_url)
                .await
                .unwrap();
            assert_eq!(registration_events.recv().unwrap(), "registration");

            let second = tokio::time::timeout(
                Duration::from_millis(250),
                authenticated_api_with_identity(&state, &identity, &base_url),
            )
            .await
            .expect("a second reader must not wait for the first reader")
            .unwrap();

            let (writer_acquired, mut writer_acquired_rx) = oneshot::channel();
            let writer_state = state.clone();
            let writer = tauri::async_runtime::spawn(async move {
                let _guard = writer_state.credential_lifecycle_gate.write().await;
                writer_acquired.send(()).unwrap();
            });
            tokio::task::yield_now().await;

            assert!(matches!(
                writer_acquired_rx.try_recv(),
                Err(tokio::sync::oneshot::error::TryRecvError::Empty)
            ));
            drop(first);
            tokio::task::yield_now().await;
            assert!(matches!(
                writer_acquired_rx.try_recv(),
                Err(tokio::sync::oneshot::error::TryRecvError::Empty)
            ));
            drop(second);
            writer.await.unwrap();
            writer_acquired_rx.await.unwrap();
            assert!(matches!(
                registration_events.try_recv(),
                Err(mpsc::TryRecvError::Empty | mpsc::TryRecvError::Disconnected)
            ));
            server.join().unwrap();
        });
    }

    fn registration_server() -> (
        String,
        mpsc::Receiver<&'static str>,
        std::thread::JoinHandle<()>,
    ) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let (events_tx, events_rx) = mpsc::channel();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let request = read_http_request(&mut stream);
            assert!(request.starts_with("POST /api/identities HTTP/1.1"));
            events_tx.send("registration").unwrap();
            let body = r#"{"identity_id":"identity-a","created":false}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).unwrap();
        });
        (format!("http://{address}"), events_rx, server)
    }

    fn read_http_request(stream: &mut std::net::TcpStream) -> String {
        let mut request = Vec::new();
        let mut chunk = [0_u8; 1024];
        let header_end = loop {
            let read = stream.read(&mut chunk).unwrap();
            assert_ne!(read, 0);
            request.extend_from_slice(&chunk[..read]);
            if let Some(index) = request.windows(4).position(|window| window == b"\r\n\r\n") {
                break index + 4;
            }
        };
        let headers = std::str::from_utf8(&request[..header_end]).unwrap();
        let content_length = headers
            .lines()
            .find_map(|line| line.strip_prefix("content-length: "))
            .or_else(|| {
                headers
                    .lines()
                    .find_map(|line| line.strip_prefix("Content-Length: "))
            })
            .unwrap()
            .parse::<usize>()
            .unwrap();
        while request.len() < header_end + content_length {
            let read = stream.read(&mut chunk).unwrap();
            assert_ne!(read, 0);
            request.extend_from_slice(&chunk[..read]);
        }
        String::from_utf8(request).unwrap()
    }
}

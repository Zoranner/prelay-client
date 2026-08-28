use std::fmt;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
pub struct ClientError {
    pub code: String,
    pub message: String,
}

impl ClientError {
    pub const MISSING_DEVICE_CREDENTIAL: &'static str = "missing_device_credential";

    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn code(&self) -> &str {
        &self.code
    }
}

impl fmt::Display for ClientError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ClientError {}

pub fn normalize_relay_url(value: &str) -> Result<String, ClientError> {
    let value = value.trim().trim_end_matches('/');
    if !(value.starts_with("https://") || value.starts_with("http://")) {
        return Err(ClientError::new(
            "invalid_relay_url",
            "PROVIDER_RELAY_URL must be an HTTP or HTTPS URL",
        ));
    }
    Ok(value.to_owned())
}

pub(crate) fn credential_store_error(error: String) -> ClientError {
    ClientError::new("credential_store_error", error)
}

pub(crate) fn network_error(error: reqwest::Error) -> ClientError {
    let diagnostic = network_error_diagnostic(error);
    eprintln!("relay management request failed: {}", diagnostic);
    ClientError::new(
        "network_error",
        format!("unable to reach the relay management API: {diagnostic}"),
    )
}

fn network_error_diagnostic(error: reqwest::Error) -> String {
    format!("{:?}", error.without_url())
}

#[cfg(test)]
mod tests {
    use super::{network_error, network_error_diagnostic};
    use std::net::TcpListener;

    #[test]
    fn network_error_logs_a_url_redacted_reqwest_diagnostic_and_keeps_a_stable_message() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("reserve unavailable relay port");
        let address = listener
            .local_addr()
            .expect("read unavailable relay address");
        drop(listener);
        let error = tauri::async_runtime::block_on(async move {
            reqwest::Client::builder()
                .no_proxy()
                .build()
                .expect("build HTTP client")
                .get(format!(
                    "http://diagnostic-user:diagnostic-secret@{address}"
                ))
                .send()
                .await
                .expect_err("unavailable relay must fail the request")
        });
        let diagnostic = network_error_diagnostic(error);

        assert!(diagnostic.contains("kind: Request"));
        assert!(diagnostic.contains("source:"));
        assert!(!diagnostic.contains("diagnostic-user"));
        assert!(!diagnostic.contains("diagnostic-secret"));

        let listener = TcpListener::bind("127.0.0.1:0").expect("reserve unavailable relay port");
        let address = listener
            .local_addr()
            .expect("read unavailable relay address");
        drop(listener);
        let error = tauri::async_runtime::block_on(async move {
            reqwest::Client::builder()
                .no_proxy()
                .build()
                .expect("build HTTP client")
                .get(format!("http://{address}"))
                .send()
                .await
                .expect_err("unavailable relay must fail the request")
        });
        let client_error = network_error(error);

        assert_eq!(client_error.code(), "network_error");
        assert!(client_error
            .message
            .starts_with("unable to reach the relay management API: reqwest::Error"));
        assert!(!client_error.message.contains("diagnostic-user"));
        assert!(!client_error.message.contains("diagnostic-secret"));
    }
}

pub(crate) async fn response_error(response: reqwest::Response) -> ClientError {
    let status = response.status();
    let (server_code, server_message) = if status.is_client_error() {
        response
            .json::<ServerErrorEnvelope>()
            .await
            .ok()
            .map(|body| body.error.into_parts())
            .unwrap_or_default()
    } else {
        (None, None)
    };
    let code = server_code.unwrap_or_else(|| status_code(status).to_owned());
    let message = server_message.unwrap_or_else(|| {
        if code == "identity_already_registered" {
            "this Windows identity is already registered and cannot be restored automatically"
                .into()
        } else {
            let reason = status.canonical_reason().unwrap_or("Unknown Status");
            format!("management API returned HTTP {} {reason}", status.as_u16())
        }
    });
    ClientError::new(code, message)
}

fn status_code(status: StatusCode) -> &'static str {
    match status {
        StatusCode::UNAUTHORIZED => "invalid_credential",
        StatusCode::NOT_FOUND => "not_found",
        StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => "validation_failed",
        _ => "internal",
    }
}

#[derive(Deserialize)]
struct ServerErrorEnvelope {
    error: ServerErrorBody,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ServerErrorBody {
    Structured {
        code: Option<String>,
        message: Option<String>,
    },
    Message(String),
}

impl ServerErrorBody {
    fn into_parts(self) -> (Option<String>, Option<String>) {
        match self {
            Self::Structured { code, message } => {
                (code, message.filter(|message| !message.trim().is_empty()))
            }
            Self::Message(message) => (None, (!message.trim().is_empty()).then_some(message)),
        }
    }
}

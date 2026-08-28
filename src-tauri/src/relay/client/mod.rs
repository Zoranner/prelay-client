mod error;
mod registration;
mod request;

pub use error::{normalize_relay_url, ClientError};
pub use registration::{generate_device_credential, RegistrationGate};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedRequest {
    url: String,
    authorization: String,
}

impl PreparedRequest {
    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn authorization(&self) -> &str {
        &self.authorization
    }
}

pub struct ApiClient<'a> {
    pub(super) base_url: String,
    pub(super) credential_store: &'a dyn crate::identity::credentials::CredentialStore,
    pub(super) http: reqwest::Client,
    pub(super) display_name: Option<String>,
}

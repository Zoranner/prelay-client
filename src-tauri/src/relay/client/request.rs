use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use reqwest::{header::AUTHORIZATION, Method};
use serde::{de::DeserializeOwned, Serialize};

use crate::identity::credentials::{CredentialRecord, CredentialStore};

use super::{
    error::{credential_store_error, network_error, response_error},
    normalize_relay_url, ApiClient, ClientError, PreparedRequest,
};

impl<'a> ApiClient<'a> {
    pub fn new(
        base_url: impl AsRef<str>,
        credential_store: &'a dyn CredentialStore,
    ) -> Result<Self, ClientError> {
        let base_url = normalize_relay_url(base_url.as_ref())?;
        Ok(Self {
            base_url,
            credential_store,
            http: reqwest::Client::new(),
            display_name: None,
        })
    }

    pub fn with_display_name(mut self, display_name: &str) -> Self {
        self.display_name = (!display_name.trim().is_empty()).then(|| display_name.trim().into());
        self
    }

    pub fn has_stored_credential(&self) -> Result<bool, ClientError> {
        Ok(self
            .credential_store
            .load()
            .map_err(credential_store_error)?
            .is_some_and(|record| !record.current.trim().is_empty()))
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn authenticated_request(
        &self,
        method: &str,
        path: &str,
    ) -> Result<PreparedRequest, ClientError> {
        Method::from_bytes(method.as_bytes()).map_err(|_| {
            ClientError::new("invalid_request", "management request method is invalid")
        })?;
        let credential = self.load_credential_record()?.preferred().to_owned();
        Ok(PreparedRequest {
            url: self.url(path)?,
            authorization: format!("Bearer {credential}"),
        })
    }
}

impl<'a> ApiClient<'a> {
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ClientError> {
        self.send_authenticated_json::<T, ()>(Method::GET, path, None)
            .await
    }

    pub async fn get_bytes(&self, path: &str) -> Result<Vec<u8>, ClientError> {
        self.send_authenticated_response(Method::GET, path)
            .await?
            .bytes()
            .await
            .map(|bytes| bytes.to_vec())
            .map_err(network_error)
    }

    pub async fn post<T: DeserializeOwned, B: Serialize + ?Sized>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ClientError> {
        self.send_authenticated_json(Method::POST, path, Some(body))
            .await
    }

    pub async fn post_with_credential<T: DeserializeOwned, B: Serialize + ?Sized>(
        &self,
        path: &str,
        body: &B,
        credential: &str,
    ) -> Result<T, ClientError> {
        self.send_json(Method::POST, path, Some(body), Some(credential.to_owned()))
            .await
    }

    pub async fn patch<T: DeserializeOwned, B: Serialize + ?Sized>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ClientError> {
        self.send_authenticated_json(Method::PATCH, path, Some(body))
            .await
    }

    pub async fn delete(&self, path: &str) -> Result<(), ClientError> {
        self.send_authenticated_empty(Method::DELETE, path).await
    }

    fn load_credential_record(&self) -> Result<CredentialRecord, ClientError> {
        self.credential_store
            .load()
            .map_err(credential_store_error)?
            .filter(|record| !record.current.trim().is_empty())
            .ok_or_else(|| {
                ClientError::new(
                    ClientError::MISSING_DEVICE_CREDENTIAL,
                    "device credential is unavailable; identity cannot be restored automatically",
                )
            })
    }

    async fn send_authenticated_json<T: DeserializeOwned, B: Serialize + ?Sized>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T, ClientError> {
        let record = self.load_credential_record()?;
        let preferred = record.preferred().to_owned();
        match self
            .send_json(method.clone(), path, body, Some(preferred))
            .await
        {
            Ok(value) => {
                if record.pending.is_some() {
                    self.credential_store
                        .confirm_pending()
                        .map_err(credential_store_error)?;
                }
                Ok(value)
            }
            Err(error) if record.pending.is_some() && error.code() == "invalid_credential" => {
                let value = self
                    .send_json(method, path, body, Some(record.current))
                    .await?;
                self.credential_store
                    .discard_pending()
                    .map_err(credential_store_error)?;
                Ok(value)
            }
            Err(error) => Err(error),
        }
    }

    async fn send_authenticated_empty(
        &self,
        method: Method,
        path: &str,
    ) -> Result<(), ClientError> {
        let record = self.load_credential_record()?;
        let preferred = record.preferred().to_owned();
        match self.send_empty(method.clone(), path, preferred).await {
            Ok(()) => {
                if record.pending.is_some() {
                    self.credential_store
                        .confirm_pending()
                        .map_err(credential_store_error)?;
                }
                Ok(())
            }
            Err(error) if record.pending.is_some() && error.code() == "invalid_credential" => {
                self.send_empty(method, path, record.current).await?;
                self.credential_store
                    .discard_pending()
                    .map_err(credential_store_error)
            }
            Err(error) => Err(error),
        }
    }

    async fn send_authenticated_response(
        &self,
        method: Method,
        path: &str,
    ) -> Result<reqwest::Response, ClientError> {
        let record = self.load_credential_record()?;
        let preferred = record.preferred().to_owned();
        match self.send_response(method.clone(), path, preferred).await {
            Ok(response) => {
                if record.pending.is_some() {
                    self.credential_store
                        .confirm_pending()
                        .map_err(credential_store_error)?;
                }
                Ok(response)
            }
            Err(error) if record.pending.is_some() && error.code() == "invalid_credential" => {
                let response = self.send_response(method, path, record.current).await?;
                self.credential_store
                    .discard_pending()
                    .map_err(credential_store_error)?;
                Ok(response)
            }
            Err(error) => Err(error),
        }
    }

    async fn send_empty(
        &self,
        method: Method,
        path: &str,
        credential: String,
    ) -> Result<(), ClientError> {
        let response = self
            .http
            .request(method, self.url(path)?)
            .header(AUTHORIZATION, format!("Bearer {credential}"))
            .send()
            .await
            .map_err(network_error)?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(response_error(response).await)
        }
    }

    async fn send_response(
        &self,
        method: Method,
        path: &str,
        credential: String,
    ) -> Result<reqwest::Response, ClientError> {
        let mut request = self
            .http
            .request(method, self.url(path)?)
            .header(AUTHORIZATION, format!("Bearer {credential}"));
        if let Some(display_name) = &self.display_name {
            request = request.header(
                "x-prelay-display-name",
                URL_SAFE_NO_PAD.encode(display_name.as_bytes()),
            );
        }
        let response = request.send().await.map_err(network_error)?;
        if response.status().is_success() {
            Ok(response)
        } else {
            Err(response_error(response).await)
        }
    }

    pub(crate) async fn send_json<T: DeserializeOwned, B: Serialize + ?Sized>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
        credential: Option<String>,
    ) -> Result<T, ClientError> {
        let mut request = self.http.request(method, self.url(path)?);
        if let Some(credential) = credential {
            request = request.header(AUTHORIZATION, format!("Bearer {credential}"));
        }
        if let Some(display_name) = &self.display_name {
            request = request.header(
                "x-prelay-display-name",
                URL_SAFE_NO_PAD.encode(display_name.as_bytes()),
            );
        }
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = request.send().await.map_err(network_error)?;
        if !response.status().is_success() {
            return Err(response_error(response).await);
        }
        response.json().await.map_err(|_| {
            ClientError::new(
                "invalid_response",
                "relay returned an invalid management response",
            )
        })
    }

    fn url(&self, path: &str) -> Result<String, ClientError> {
        if !path.starts_with('/') {
            return Err(ClientError::new(
                "invalid_request",
                "management request path must be absolute",
            ));
        }
        Ok(format!("{}{}", self.base_url, path))
    }
}

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use prelay_protocol::{CreateIdentityRequest, CreateIdentityResponse};
use rand::RngCore;
use reqwest::Method;

use crate::identity::{credentials::CredentialRecord, windows::WindowsIdentity};

use super::{error::credential_store_error, ApiClient, ClientError};

#[derive(Default)]
pub struct RegistrationGate(pub(super) tokio::sync::Mutex<()>);

impl<'a> ApiClient<'a> {
    pub async fn ensure_registered(&self, identity: &WindowsIdentity) -> Result<(), ClientError> {
        let record = match self
            .credential_store
            .load()
            .map_err(credential_store_error)?
        {
            Some(record) => record,
            None => self
                .credential_store
                .save_initial(&generate_device_credential())
                .map_err(credential_store_error)?,
        };

        if let Some(pending) = record.pending {
            match self.register_identity(identity, pending).await {
                Ok(()) => {
                    self.credential_store
                        .confirm_pending()
                        .map_err(credential_store_error)?;
                    return Ok(());
                }
                Err(error) if error.code() == "identity_already_registered" => {
                    self.register_identity(identity, record.current).await?;
                    self.credential_store
                        .discard_pending()
                        .map_err(credential_store_error)?;
                    return Ok(());
                }
                Err(error) => return Err(error),
            }
        }

        self.register_identity(identity, record.current).await
    }

    pub async fn ensure_registered_once(
        &self,
        identity: &WindowsIdentity,
        gate: &RegistrationGate,
    ) -> Result<(), ClientError> {
        let _guard = gate.0.lock().await;
        self.ensure_registered(identity).await
    }

    async fn register_identity(
        &self,
        identity: &WindowsIdentity,
        credential: String,
    ) -> Result<(), ClientError> {
        let _: CreateIdentityResponse = self
            .send_json(
                Method::POST,
                "/api/identities",
                Some(&CreateIdentityRequest {
                    machine_id: identity.machine_id.clone(),
                    account_sid: identity.account_sid.clone(),
                    credential,
                    display_name: self.display_name.clone(),
                }),
                None,
            )
            .await?;
        Ok(())
    }
}

impl CredentialRecord {
    pub(crate) fn preferred(&self) -> &str {
        self.pending.as_deref().unwrap_or(&self.current)
    }
}

pub fn generate_device_credential() -> String {
    let mut bytes = [0_u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

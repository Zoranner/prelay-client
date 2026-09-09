use prelay_protocol::IdentityDirectoryEntry;
use tauri::State;

use crate::{
    identity::registration::{authenticated_api, rotate_credential},
    relay::client::ClientError,
    NativeState,
};

use super::status::OperationStatus;

#[tauri::command]
pub async fn credential_rotate(
    state: State<'_, NativeState>,
) -> Result<OperationStatus, ClientError> {
    rotate_credential(&state).await?;
    Ok(OperationStatus {
        message: "device credential rotated".to_string(),
    })
}

#[tauri::command]
pub async fn identity_directory_list(
    state: State<'_, NativeState>,
) -> Result<Vec<IdentityDirectoryEntry>, ClientError> {
    authenticated_api(&state)
        .await?
        .get("/api/identities")
        .await
}

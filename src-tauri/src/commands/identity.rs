use tauri::State;

use crate::{identity::registration::rotate_credential, relay::client::ClientError, NativeState};

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

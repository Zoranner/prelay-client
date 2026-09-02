use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    identity::registration::authenticated_api,
    identity::windows::IdentitySource,
    relay::client::{ApiClient, ClientError},
    NativeState,
};

#[derive(Debug, Serialize)]
pub struct BootstrapResponse {
    pub identity_id: String,
    pub relay_url: String,
    pub display_name: String,
    pub avatar_seed: String,
    pub has_device_credential: bool,
}

pub fn collect_bootstrap(
    identity_source: &impl IdentitySource,
    api_client: &ApiClient<'_>,
    identity_id: String,
) -> Result<BootstrapResponse, ClientError> {
    let identity = identity_source
        .identity()
        .map_err(|error| ClientError::new("internal", error))?;
    let has_device_credential = api_client.has_stored_credential()?;
    let relay_url = api_client.base_url().to_owned();

    Ok(BootstrapResponse {
        identity_id,
        relay_url,
        display_name: identity.display_name,
        avatar_seed: avatar_seed(&identity.account_sid),
        has_device_credential,
    })
}

#[tauri::command]
pub async fn bootstrap(state: State<'_, NativeState>) -> Result<BootstrapResponse, ClientError> {
    let api_client = authenticated_api(&state).await?;
    let identity: CurrentIdentityResponse = api_client.get("/api/identity").await?;
    collect_bootstrap(&state.identity, &api_client, identity.identity_id)
}

#[derive(Debug, Deserialize)]
struct CurrentIdentityResponse {
    identity_id: String,
}

fn avatar_seed(account_sid: &str) -> String {
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x00000100000001b3;

    let hash = account_sid.bytes().fold(OFFSET_BASIS, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(PRIME)
    });
    format!("{hash:016x}")
}

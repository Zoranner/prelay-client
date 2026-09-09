use prelay_protocol::{
    CatalogProviderResponse, CreateProviderRequest, ProviderCapabilityOverrides,
    ProviderCatalogResponse, ProviderListItemResponse, ProviderOperationRequest,
    ProviderOperationResponse, ProviderResponse, ProviderSharingResponse, ProviderUsageResponse,
    UpdateProviderRequest, UpdateProviderSharingRequest,
};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    commands::status::OperationStatus, identity::registration::authenticated_api,
    relay::client::ClientError, NativeState,
};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ProviderSaveInput {
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub api_key: String,
    pub capabilities: Option<ProviderCapabilityOverrides>,
}

#[tauri::command]
pub async fn providers_list(
    state: State<'_, NativeState>,
) -> Result<Vec<ProviderListItemResponse>, ClientError> {
    authenticated_api(&state).await?.get("/api/providers").await
}

#[tauri::command]
pub async fn catalog_providers_list(
    state: State<'_, NativeState>,
) -> Result<Vec<CatalogProviderResponse>, ClientError> {
    authenticated_api(&state)
        .await?
        .get("/api/catalog/providers")
        .await
}

#[tauri::command]
pub async fn catalog_models_get(
    state: State<'_, NativeState>,
) -> Result<ProviderCatalogResponse, ClientError> {
    authenticated_api(&state).await?.get("/api/catalog").await
}

#[tauri::command]
pub async fn providers_save(
    state: State<'_, NativeState>,
    provider_id: Option<String>,
    input: ProviderSaveInput,
) -> Result<ProviderResponse, ClientError> {
    let client = authenticated_api(&state).await?;
    match provider_id {
        Some(provider_id) => {
            let input = UpdateProviderRequest {
                name: Some(input.name),
                provider_type: Some(input.provider_type),
                base_url: Some(input.base_url),
                api_key: non_empty(input.api_key),
                capabilities: input.capabilities,
            };
            client
                .patch(&format!("/api/providers/{provider_id}"), &input)
                .await
        }
        None => {
            let api_key = non_empty(input.api_key).ok_or_else(|| {
                ClientError::new(
                    "validation_failed",
                    "provider API key is required when creating",
                )
            })?;
            let input = CreateProviderRequest {
                name: input.name,
                provider_type: input.provider_type,
                base_url: input.base_url,
                api_key,
                capabilities: input.capabilities,
            };
            client.post("/api/providers", &input).await
        }
    }
}

#[tauri::command]
pub async fn providers_delete(
    state: State<'_, NativeState>,
    provider_id: String,
) -> Result<OperationStatus, ClientError> {
    authenticated_api(&state)
        .await?
        .delete(&format!("/api/providers/{provider_id}"))
        .await?;
    Ok(OperationStatus {
        message: "provider deleted".to_string(),
    })
}

#[tauri::command]
pub async fn providers_ping(
    state: State<'_, NativeState>,
    provider_id: String,
) -> Result<ProviderOperationResponse, ClientError> {
    authenticated_api(&state)
        .await?
        .post(&format!("/api/providers/{provider_id}/ping"), &())
        .await
}

#[tauri::command]
pub async fn providers_test_protocol(
    state: State<'_, NativeState>,
    input: ProviderOperationRequest,
) -> Result<ProviderOperationResponse, ClientError> {
    authenticated_api(&state)
        .await?
        .post("/api/providers/test-protocol", &input)
        .await
}

#[tauri::command]
pub async fn providers_sharing_get(
    state: State<'_, NativeState>,
    provider_id: String,
) -> Result<ProviderSharingResponse, ClientError> {
    authenticated_api(&state)
        .await?
        .get(&format!("/api/providers/{provider_id}/sharing"))
        .await
}

#[tauri::command]
pub async fn providers_sharing_save(
    state: State<'_, NativeState>,
    provider_id: String,
    input: UpdateProviderSharingRequest,
) -> Result<ProviderSharingResponse, ClientError> {
    authenticated_api(&state)
        .await?
        .patch(&format!("/api/providers/{provider_id}/sharing"), &input)
        .await
}

#[tauri::command]
pub async fn providers_usage_get(
    state: State<'_, NativeState>,
    provider_id: String,
    range: String,
) -> Result<ProviderUsageResponse, ClientError> {
    authenticated_api(&state)
        .await?
        .get(&format!("/api/providers/{provider_id}/usage?range={range}"))
        .await
}

fn non_empty(value: String) -> Option<String> {
    (!value.trim().is_empty()).then_some(value)
}

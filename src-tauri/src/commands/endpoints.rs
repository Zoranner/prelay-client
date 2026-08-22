use prelay_protocol::{
    CreateEndpointRequest, EndpointModelInput, EndpointResponse, UpdateEndpointRequest,
};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    api_client::ClientError,
    commands::{authenticated_api, OperationStatus},
    NativeState,
};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct EndpointSaveInput {
    pub name: String,
    pub protocol: String,
    pub models: Vec<EndpointModelInput>,
}

#[tauri::command]
pub async fn endpoints_list(
    state: State<'_, NativeState>,
) -> Result<Vec<EndpointResponse>, ClientError> {
    authenticated_api(&state).await?.get("/api/endpoints").await
}

#[tauri::command]
pub async fn endpoints_save(
    state: State<'_, NativeState>,
    endpoint_id: Option<String>,
    input: EndpointSaveInput,
) -> Result<EndpointResponse, ClientError> {
    let client = authenticated_api(&state).await?;
    match endpoint_id {
        Some(endpoint_id) => {
            let input = UpdateEndpointRequest {
                name: Some(input.name),
                protocol: Some(input.protocol),
                models: Some(input.models),
            };
            client
                .patch(&format!("/api/endpoints/{endpoint_id}"), &input)
                .await
        }
        None => {
            let input = CreateEndpointRequest {
                name: input.name,
                protocol: Some(input.protocol),
                models: input.models,
            };
            client.post("/api/endpoints", &input).await
        }
    }
}

#[tauri::command]
pub async fn endpoints_delete(
    state: State<'_, NativeState>,
    endpoint_id: String,
) -> Result<OperationStatus, ClientError> {
    authenticated_api(&state)
        .await?
        .delete(&format!("/api/endpoints/{endpoint_id}"))
        .await?;
    Ok(OperationStatus {
        message: "endpoint deleted".to_string(),
    })
}

#[tauri::command]
pub async fn endpoints_regenerate_token(
    state: State<'_, NativeState>,
    endpoint_id: String,
) -> Result<EndpointResponse, ClientError> {
    authenticated_api(&state)
        .await?
        .post(
            &format!("/api/endpoints/{endpoint_id}/regenerate-token"),
            &serde_json::json!({}),
        )
        .await
}

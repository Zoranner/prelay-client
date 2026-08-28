use prelay_protocol::{
    ModelStatsSummary, ProviderStatsSummary, RequestLogSummary, StatsOverview,
    TokenUsageTimelinePoint,
};
use tauri::State;

use crate::{identity::registration::authenticated_api, relay::client::ClientError, NativeState};

fn range_path(path: &str, range: Option<String>) -> String {
    match range {
        Some(range) => format!("{path}?range={range}"),
        None => path.to_string(),
    }
}

#[tauri::command]
pub async fn stats_overview(
    state: State<'_, NativeState>,
    range: Option<String>,
) -> Result<StatsOverview, ClientError> {
    let path = range_path("/api/stats/overview", range);
    authenticated_api(&state).await?.get(&path).await
}

#[tauri::command]
pub async fn stats_timeline(
    state: State<'_, NativeState>,
    range: Option<String>,
) -> Result<Vec<TokenUsageTimelinePoint>, ClientError> {
    let path = range_path("/api/stats/timeline", range);
    authenticated_api(&state).await?.get(&path).await
}

#[tauri::command]
pub async fn stats_requests(
    state: State<'_, NativeState>,
    limit: Option<usize>,
) -> Result<Vec<RequestLogSummary>, ClientError> {
    let path = match limit {
        Some(limit) => format!("/api/stats/requests?limit={limit}"),
        None => "/api/stats/requests".to_string(),
    };
    authenticated_api(&state).await?.get(&path).await
}

#[tauri::command]
pub async fn stats_models(
    state: State<'_, NativeState>,
    range: Option<String>,
) -> Result<Vec<ModelStatsSummary>, ClientError> {
    let path = range_path("/api/stats/models", range);
    authenticated_api(&state).await?.get(&path).await
}

#[tauri::command]
pub async fn stats_providers(
    state: State<'_, NativeState>,
    range: Option<String>,
) -> Result<Vec<ProviderStatsSummary>, ClientError> {
    let path = range_path("/api/stats/providers", range);
    authenticated_api(&state).await?.get(&path).await
}

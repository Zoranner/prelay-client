use prelay_protocol::{
    stats::UserLeaderboardEntry, ActivitySummary, ModelStatsSummary, ProviderStatsSummary,
    StatsOverview, TokenUsageTimelinePoint,
};
use tauri::State;

use crate::{identity::registration::authenticated_api, relay::client::ClientError, NativeState};

fn query_path(path: &str, params: Vec<(&str, Option<String>)>) -> String {
    let query = params
        .into_iter()
        .filter_map(|(key, value)| value.map(|value| format!("{key}={value}")))
        .collect::<Vec<_>>();
    if query.is_empty() {
        path.to_string()
    } else {
        format!("{path}?{}", query.join("&"))
    }
}

#[tauri::command]
pub async fn stats_overview(
    state: State<'_, NativeState>,
    range: Option<String>,
    scope: Option<String>,
) -> Result<StatsOverview, ClientError> {
    let path = query_path(
        "/api/stats/overview",
        vec![("range", range), ("scope", scope)],
    );
    authenticated_api(&state).await?.get(&path).await
}

#[tauri::command]
pub async fn stats_timeline(
    state: State<'_, NativeState>,
    range: Option<String>,
    granularity: Option<String>,
    scope: Option<String>,
) -> Result<Vec<TokenUsageTimelinePoint>, ClientError> {
    let path = query_path(
        "/api/stats/timeline",
        vec![
            ("range", range),
            ("granularity", granularity),
            ("scope", scope),
        ],
    );
    authenticated_api(&state).await?.get(&path).await
}

#[tauri::command]
pub async fn stats_activities(
    state: State<'_, NativeState>,
    limit: Option<usize>,
) -> Result<Vec<ActivitySummary>, ClientError> {
    let path = match limit {
        Some(limit) => format!("/api/stats/activities?limit={limit}"),
        None => "/api/stats/activities".to_string(),
    };
    authenticated_api(&state).await?.get(&path).await
}

#[tauri::command]
pub async fn stats_models(
    state: State<'_, NativeState>,
    range: Option<String>,
    scope: Option<String>,
) -> Result<Vec<ModelStatsSummary>, ClientError> {
    let path = query_path(
        "/api/stats/models",
        vec![("range", range), ("scope", scope)],
    );
    authenticated_api(&state).await?.get(&path).await
}

#[tauri::command]
pub async fn stats_providers(
    state: State<'_, NativeState>,
    range: Option<String>,
    scope: Option<String>,
) -> Result<Vec<ProviderStatsSummary>, ClientError> {
    let path = query_path(
        "/api/stats/providers",
        vec![("range", range), ("scope", scope)],
    );
    authenticated_api(&state).await?.get(&path).await
}

#[tauri::command]
pub async fn stats_leaderboard(
    state: State<'_, NativeState>,
    range: Option<String>,
    metric: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<UserLeaderboardEntry>, ClientError> {
    let mut query = Vec::new();
    if let Some(range) = range {
        query.push(format!("range={range}"));
    }
    if let Some(metric) = metric {
        query.push(format!("metric={metric}"));
    }
    if let Some(limit) = limit {
        query.push(format!("limit={limit}"));
    }
    let path = if query.is_empty() {
        "/api/stats/leaderboard".to_string()
    } else {
        format!("/api/stats/leaderboard?{}", query.join("&"))
    };
    authenticated_api(&state).await?.get(&path).await
}

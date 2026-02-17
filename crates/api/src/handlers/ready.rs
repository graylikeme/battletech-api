use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::state::AppState;

pub async fn ready_handler(State(state): State<AppState>) -> Response {
    // 1. Check DB connectivity
    if let Err(e) = sqlx::query("SELECT 1").execute(&state.pool).await {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "error": format!("db ping failed: {e}") })),
        )
            .into_response();
    }

    // 2. Check schema version against expected
    let row: Result<(i32,), _> = sqlx::query_as(
        "SELECT schema_version FROM dataset_metadata ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&state.pool)
    .await;

    match row {
        Ok((sv,)) if sv == state.dataset_version.parse::<i32>().unwrap_or(1) => {
            (StatusCode::OK, Json(json!({ "status": "ready" }))).into_response()
        }
        Ok((sv,)) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": format!(
                    "schema_version mismatch: got {sv}, expected {}",
                    state.dataset_version
                )
            })),
        )
            .into_response(),
        Err(_) => {
            // No metadata row yet — treat as ready (empty DB, migrations run)
            (StatusCode::OK, Json(json!({ "status": "ready", "note": "no metadata row" })))
                .into_response()
        }
    }
}

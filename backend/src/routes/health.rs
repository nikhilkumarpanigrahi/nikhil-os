//! Liveness + readiness probes.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::error::{AppError, Result};
use crate::state::AppState;

/// Liveness — process is up. No dependencies touched.
pub async fn healthz() -> StatusCode {
    StatusCode::OK
}

/// Readiness — database reachable.
pub async fn readyz(State(state): State<AppState>) -> Result<Json<serde_json::Value>> {
    sqlx::query("SELECT 1")
        .execute(&state.pool)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(serde_json::json!({ "status": "ready" })))
}

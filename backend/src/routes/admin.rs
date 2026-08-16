//! Admin surface — login (rate-limited), JWT-protected inbox + stats,
//! and the self-contained panel at /admin.

use std::net::{IpAddr, SocketAddr};

use axum::body::Body;
use axum::extract::{ConnectInfo, Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::Response;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::{issue_token, verify_password, AdminAuth};
use crate::error::{AppError, Result};
use crate::services::audit;
use crate::services::inbox;
use crate::state::AppState;

const ADMIN_HTML: &str = include_str!("../admin/index.html");
const VALID_STATUSES: &[&str] = &["new", "read", "replied", "archived"];

fn client_ip(headers: &HeaderMap, peer: IpAddr) -> IpAddr {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(str::trim)
        .and_then(|s| s.parse().ok())
        .unwrap_or(peer)
}

/// GET /admin — the panel itself (static, unauthenticated page; its API calls
/// carry the JWT).
pub async fn panel() -> Response {
    let mut resp = Response::new(Body::from(ADMIN_HTML));
    *resp.status_mut() = StatusCode::OK;
    resp.headers_mut().insert(
        "content-type",
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    resp.headers_mut()
        .insert("cache-control", HeaderValue::from_static("no-store"));
    resp
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_in: usize,
}

pub async fn login(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>> {
    let ip = client_ip(&headers, peer.ip());
    if state.login_limiter.check_key(&ip).is_err() {
        return Err(AppError::RateLimited);
    }

    if !verify_password(&req.password, &state.config.admin_password_hash) {
        audit::record(&state.pool, "admin_login_failed", None).await;
        return Err(AppError::Unauthorized);
    }

    let ttl = state.config.admin_jwt_ttl_secs;
    let token = issue_token(&state.config.admin_jwt_secret, ttl).map_err(|_| AppError::Internal)?;
    audit::record(&state.pool, "admin_login_ok", None).await;

    Ok(Json(LoginResponse {
        token,
        expires_in: ttl,
    }))
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    status: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

pub async fn list_messages(
    _auth: AdminAuth,
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<serde_json::Value>> {
    let limit = params.limit.unwrap_or(50).clamp(1, 100);
    let offset = params.offset.unwrap_or(0).max(0);

    let status = match params.status.as_deref() {
        None | Some("") => None,
        Some(s) if VALID_STATUSES.contains(&s) => Some(s),
        Some(_) => return Err(AppError::Validation("invalid status filter".into())),
    };

    let messages = inbox::list_messages(&state.pool, status, limit, offset).await?;
    Ok(Json(serde_json::json!({
        "messages": messages,
        "count": messages.len(),
        "limit": limit,
        "offset": offset,
    })))
}

#[derive(Debug, Deserialize)]
pub struct PatchBody {
    pub status: String,
}

pub async fn patch_message(
    _auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<PatchBody>,
) -> Result<Json<serde_json::Value>> {
    if !VALID_STATUSES.contains(&body.status.as_str()) {
        return Err(AppError::Validation(
            "status must be new|read|replied|archived".into(),
        ));
    }

    let ok = inbox::set_message_status(&state.pool, id, &body.status).await?;
    if !ok {
        return Err(AppError::NotFound);
    }

    audit::record(
        &state.pool,
        "message_status",
        Some(serde_json::json!({ "id": id, "status": body.status })),
    )
    .await;

    Ok(Json(serde_json::json!({ "id": id, "status": body.status })))
}

pub async fn stats(
    _auth: AdminAuth,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let (counts, daily) = inbox::stats(&state.pool).await?;
    Ok(Json(serde_json::json!({
        "counts": counts,
        "daily": daily,
        "knowledge_etag": crate::knowledge::etag(),
    })))
}

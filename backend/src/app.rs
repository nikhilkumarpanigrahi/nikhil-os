//! Router construction. Middleware order matters:
//! RequestId → Trace → CORS → body limit → routes (rate limiting is per-handler).

use std::time::Duration;

use axum::extract::DefaultBodyLimit;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderName, HeaderValue, Method};
use axum::routing::{get, patch, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;

use crate::routes;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    let origins: Vec<HeaderValue> = state
        .config
        .cors_allowed_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION])
        .max_age(Duration::from_secs(3600));

    Router::new()
        .route("/healthz", get(routes::health::healthz))
        .route("/readyz", get(routes::health::readyz))
        .route("/api/v1/profile", get(routes::knowledge::profile))
        .route("/api/v1/projects", get(routes::knowledge::projects))
        .route("/api/v1/skills", get(routes::knowledge::skills))
        .route("/api/v1/experience", get(routes::knowledge::experience))
        .route("/api/v1/claims", get(routes::knowledge::claims))
        .route("/api/v1/contact", post(routes::contact::submit))
        .route("/admin/api/login", post(routes::admin::login))
        .route("/admin/api/messages", get(routes::admin::list_messages))
        .route(
            "/admin/api/messages/{id}",
            patch(routes::admin::patch_message),
        )
        .route("/admin/api/stats", get(routes::admin::stats))
        .route("/admin", get(routes::admin::panel))
        .route("/admin/{*path}", get(routes::admin::panel))
        .layer(DefaultBodyLimit::max(16 * 1024))
        .layer(TraceLayer::new_for_http())
        .layer(PropagateRequestIdLayer::new(HeaderName::from_static(
            "x-request-id",
        )))
        .layer(SetRequestIdLayer::new(
            HeaderName::from_static("x-request-id"),
            MakeRequestUuid,
        ))
        .layer(cors)
        .with_state(state)
}

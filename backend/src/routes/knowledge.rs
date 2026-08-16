//! Live knowledge API — the same canonical profile the WASM core embeds,
//! served over REST with ETag + short cache lifetimes.

use axum::body::Body;
use axum::http::{HeaderValue, StatusCode};
use axum::response::Response;

use crate::knowledge;

fn cached_json(value: &serde_json::Value) -> Response {
    let body = serde_json::to_vec(value).expect("knowledge data always serializes");
    let mut resp = Response::new(Body::from(body));
    *resp.status_mut() = StatusCode::OK;
    let headers = resp.headers_mut();
    headers.insert("content-type", HeaderValue::from_static("application/json"));
    headers.insert(
        "etag",
        HeaderValue::from_str(knowledge::etag()).expect("etag is a valid header"),
    );
    headers.insert(
        "cache-control",
        HeaderValue::from_static("public, max-age=300"),
    );
    resp
}

pub async fn profile() -> Response {
    cached_json(&serde_json::to_value(knowledge::load()).expect("profile serializes"))
}

pub async fn projects() -> Response {
    cached_json(&serde_json::json!({ "projects": knowledge::load().projects }))
}

pub async fn skills() -> Response {
    cached_json(&serde_json::json!({ "skills": knowledge::load().skills }))
}

pub async fn experience() -> Response {
    cached_json(&serde_json::json!({ "experience": knowledge::load().experience }))
}

pub async fn claims() -> Response {
    cached_json(&serde_json::json!({ "claims": knowledge::load().claims }))
}

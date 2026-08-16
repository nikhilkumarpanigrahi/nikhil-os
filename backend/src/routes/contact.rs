//! POST /api/v1/contact — the visitor-facing entry point.
//!
//! Hardened: honeypot anti-spam, strict validation, per-IP rate limiting,
//! and an async notification to the owner. Never leaks internal errors.

use std::net::{IpAddr, SocketAddr};

use axum::extract::{ConnectInfo, State};
use axum::http::header::USER_AGENT;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::services::inbox::{insert_message, NewMessage};
use crate::state::AppState;

const TOPICS: &[&str] = &[
    "general",
    "collaboration",
    "opportunity",
    "feedback",
    "recruiting",
];
const MAX_BODY: usize = 2000;

#[derive(Debug, Deserialize)]
pub struct ContactRequest {
    pub name: String,
    pub email: String,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub topic: String,
    pub body: String,
    /// Honeypot. Hidden from humans; auto-filling bots trip on it.
    #[serde(default)]
    pub website: String,
}

/// The real client IP: trust the first X-Forwarded-For entry only when present
/// (Caddy sets it and is the only ingress), else the TCP peer.
fn client_ip(headers: &HeaderMap, peer: IpAddr) -> IpAddr {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(str::trim)
        .and_then(|s| s.parse().ok())
        .unwrap_or(peer)
}

fn validate(req: &ContactRequest) -> std::result::Result<(), String> {
    let name = req.name.trim();
    if !(2..=80).contains(&name.chars().count()) {
        return Err("name must be between 2 and 80 characters".into());
    }
    if !is_valid_email(&req.email) {
        return Err("a valid email address is required".into());
    }
    let body = req.body.trim();
    if body.chars().count() < 10 {
        return Err("message must be at least 10 characters".into());
    }
    if body.chars().count() > MAX_BODY {
        return Err(format!("message must be at most {MAX_BODY} characters"));
    }
    if req.subject.chars().count() > 120 {
        return Err("subject must be at most 120 characters".into());
    }
    if !req.topic.is_empty() && !TOPICS.contains(&req.topic.as_str()) {
        return Err("unknown topic".into());
    }
    Ok(())
}

fn is_valid_email(email: &str) -> bool {
    let email = email.trim();
    if email.is_empty() || email.chars().count() > 254 || email.chars().any(char::is_whitespace) {
        return false;
    }
    // Exactly one @, then strict-but-minimal local + domain parts:
    //  - local: 1..=64 chars, no leading/trailing/consecutive dots
    //  - domain: 2..=253 chars, fully dotted with non-empty labels made only of
    //    letters/digits/hyphens (no leading/trailing hyphen)
    let mut parts = email.split('@');
    let (Some(local), Some(domain)) = (parts.next(), parts.next()) else {
        return false;
    };
    if parts.next().is_some() {
        return false;
    }

    let valid_local = (1..=64).contains(&local.chars().count())
        && !local.starts_with('.')
        && !local.ends_with('.')
        && !local.contains("..");

    let valid_domain = (2..=253).contains(&domain.chars().count())
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.contains("..")
        && domain.split('.').all(|label| {
            !label.is_empty()
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        });

    valid_local && valid_domain
}

pub async fn submit(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<ContactRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let ip = client_ip(&headers, peer.ip());

    // Honeypot: silently pretend success and store nothing, so bots don't learn
    // that the field exists.
    if !req.website.is_empty() {
        tracing::warn!(ip = %ip, "contact honeypot triggered; discarding message");
        return Ok((
            StatusCode::CREATED,
            Json(serde_json::json!({ "id": Uuid::new_v4(), "status": "new" })),
        ));
    }

    validate(&req).map_err(AppError::Validation)?;

    // Per-IP rate limit.
    if state.contact_limiter.check_key(&ip).is_err() {
        tracing::warn!(ip = %ip, "contact rate limit hit");
        return Err(AppError::RateLimited);
    }

    let user_agent = headers
        .get(USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.chars().take(200).collect::<String>());
    let origin = headers
        .get("origin")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.chars().take(200).collect::<String>());

    let msg = NewMessage {
        name: req.name.trim().to_string(),
        email: req.email.trim().to_string(),
        subject: req.subject.trim().to_string(),
        topic: if req.topic.is_empty() {
            "general".to_string()
        } else {
            req.topic
        },
        body: req.body.trim().to_string(),
        origin,
        user_agent,
        ip: Some(sqlx::types::ipnetwork::IpNetwork::from(ip)),
    };

    let saved = insert_message(&state.pool, &msg).await?;

    // Notify the owner off the request path — a slow/broken sender must never
    // affect the visitor's response.
    {
        let notifier = state.notifier.clone();
        let name = saved.name.clone();
        let topic = saved.topic.clone();
        let body = saved.body.clone();
        tokio::spawn(async move {
            let preview = body.chars().take(160).collect::<String>();
            let title = format!("New message — {topic}");
            let text = format!("*{name}* ({})\n\n{preview}", saved.email);
            if let Err(e) = notifier.send(&title, &text).await {
                tracing::error!(error = %e, "failed to send contact notification");
            }
        });
    }

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": saved.id, "status": saved.status })),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req() -> ContactRequest {
        ContactRequest {
            name: "Ada".into(),
            email: "ada@example.com".into(),
            subject: String::new(),
            topic: "general".into(),
            body: "This is a sufficiently long message body.".into(),
            website: String::new(),
        }
    }

    #[test]
    fn valid_request_passes() {
        assert!(validate(&req()).is_ok());
    }

    #[test]
    fn rejects_short_body() {
        let mut r = req();
        r.body = "short".into();
        assert!(validate(&r).is_err());
    }

    #[test]
    fn rejects_bad_email() {
        for bad in ["", "nope", "a@b", "a @b.com", "a@.com", "@x.com"] {
            let mut r = req();
            r.email = bad.into();
            assert!(validate(&r).is_err(), "should reject {bad:?}");
        }
    }

    #[test]
    fn rejects_unknown_topic() {
        let mut r = req();
        r.topic = "nonsense".into();
        assert!(validate(&r).is_err());
    }
}

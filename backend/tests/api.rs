//! Integration tests — drive the real router against a real Postgres.
//!
//! Local: `DATABASE_URL=postgres://nikhilos:nikhilos_dev_pw@localhost:5432/nikhilos_test cargo test`
//! CI:    the backend-ci workflow provides a Postgres service container.
//!
//! Each test uses unique email/IP markers instead of truncating tables, so the
//! suite is safe to run concurrently.

use std::net::SocketAddr;

use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHasher};
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use nikhil_os_backend::app::build_router;
use nikhil_os_backend::config::{Config, LogFormat};
use nikhil_os_backend::db;
use nikhil_os_backend::services::notify::{LogNotifier, NotificationSender};
use nikhil_os_backend::state::AppState;
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;

fn test_config(contact_limit: u32) -> Config {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(b"admin123", &salt)
        .expect("hash")
        .to_string();

    Config {
        database_url: String::new(),
        bind_addr: "0.0.0.0:0".into(),
        cors_allowed_origins: vec![
            "https://nikhilkumarpanigrahi.github.io".into(),
            "http://localhost:5173".into(),
        ],
        admin_password_hash: hash,
        admin_jwt_secret: "test-secret-that-is-long-enough".into(),
        admin_jwt_ttl_secs: 900,
        telegram_bot_token: None,
        telegram_chat_id: None,
        contact_rate_limit_per_minute: contact_limit,
        login_rate_limit_per_minute: 100,
        log_format: LogFormat::Json,
    }
}

async fn test_pool() -> PgPool {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://nikhilos:nikhilos_dev_pw@localhost:5432/nikhilos_test".into()
    });
    let pool = db::connect(&url).await.expect("connect to test database");
    db::migrate(&pool).await.expect("migrate test database");
    pool
}

async fn test_app(contact_limit: u32) -> (axum::Router, PgPool) {
    let pool = test_pool().await;
    let config = test_config(contact_limit);
    let notifier: Box<dyn NotificationSender + Send + Sync> = Box::new(LogNotifier);
    let state = AppState::new(pool.clone(), config, notifier);
    (build_router(state), pool)
}

fn req(method: &str, uri: &str, body: Option<serde_json::Value>, ip: &str) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if body.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    let payload = body.map(|v| v.to_string()).unwrap_or_default();
    let mut r = builder.body(Body::from(payload)).expect("request");
    r.extensions_mut().insert(ConnectInfo(
        format!("{ip}:1234").parse::<SocketAddr>().unwrap(),
    ));
    r
}

async fn send(app: &axum::Router, r: Request<Body>) -> (StatusCode, serde_json::Value) {
    let resp = app.clone().oneshot(r).await.expect("response");
    let status = resp.status();
    let bytes = resp.into_body().collect().await.expect("body").to_bytes();
    let value = serde_json::from_slice(&bytes).unwrap_or(json!({}));
    (status, value)
}

fn contact_payload(email: &str) -> serde_json::Value {
    json!({
        "name": "Ada Lovelace",
        "email": email,
        "topic": "collaboration",
        "body": "This is a real, sufficiently long message from a visitor.",
        "subject": "Let's build something",
    })
}

#[tokio::test]
async fn health_and_ready() {
    let (app, _pool) = test_app(10).await;
    let (s, body) = send(&app, req("GET", "/healthz", None, "10.0.0.10")).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body, json!({}));

    let (s, body) = send(&app, req("GET", "/readyz", None, "10.0.0.10")).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["status"], "ready");
}

#[tokio::test]
async fn knowledge_api_serves_canonical_profile() {
    let (app, _pool) = test_app(10).await;

    let (s, body) = send(&app, req("GET", "/api/v1/profile", None, "10.0.0.11")).await;
    assert_eq!(s, StatusCode::OK);
    assert!(!body["person"]["name"]
        .as_str()
        .unwrap_or_default()
        .is_empty());
    assert_eq!(body["skills"].as_array().unwrap().len(), 23);
    assert!(body["projects"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["id"] == "nikhil-os"));

    let (s, body) = send(&app, req("GET", "/api/v1/skills", None, "10.0.0.11")).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["skills"].as_array().unwrap().len(), 23);

    let (s, _) = send(&app, req("GET", "/api/v1/claims", None, "10.0.0.11")).await;
    assert_eq!(s, StatusCode::OK);
}

#[tokio::test]
async fn contact_stores_message_and_returns_201() {
    let (app, pool) = test_app(10).await;
    let email = format!("visitor-{}@example.com", uuid::Uuid::new_v4());

    let (s, body) = send(
        &app,
        req(
            "POST",
            "/api/v1/contact",
            Some(contact_payload(&email)),
            "10.0.0.12",
        ),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED);
    assert_eq!(body["status"], "new");

    let row = sqlx::query!("SELECT email, status FROM messages WHERE email = $1", email)
        .fetch_one(&pool)
        .await
        .expect("message persisted");
    assert_eq!(row.status, "new");
}

#[tokio::test]
async fn contact_honeypot_pretends_success_but_stores_nothing() {
    let (app, pool) = test_app(10).await;
    let email = format!("bot-{}@example.com", uuid::Uuid::new_v4());
    let mut payload = contact_payload(&email);
    payload["website"] = json!("http://spam.example.com");

    let (s, _) = send(
        &app,
        req("POST", "/api/v1/contact", Some(payload), "10.0.0.13"),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED);

    let n = sqlx::query!("SELECT COUNT(*) AS n FROM messages WHERE email = $1", email)
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(n.n.unwrap_or(0), 0);
}

#[tokio::test]
async fn contact_rejects_invalid_input() {
    let (app, _pool) = test_app(10).await;
    let mut payload = contact_payload("x@example.com");
    payload["body"] = json!("too short");

    let (s, body) = send(
        &app,
        req("POST", "/api/v1/contact", Some(payload), "10.0.0.14"),
    )
    .await;
    assert_eq!(s, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_error");
}

#[tokio::test]
async fn contact_is_rate_limited_per_ip() {
    // Dedicated app with a limit of 2/min, hit 3 times from the same IP.
    let (app, _pool) = test_app(2).await;
    let email = format!("burst-{}@example.com", uuid::Uuid::new_v4());

    for _ in 0..2 {
        let (s, _) = send(
            &app,
            req(
                "POST",
                "/api/v1/contact",
                Some(contact_payload(&email)),
                "10.99.0.1",
            ),
        )
        .await;
        assert_eq!(s, StatusCode::CREATED);
    }
    let (s, body) = send(
        &app,
        req(
            "POST",
            "/api/v1/contact",
            Some(contact_payload(&email)),
            "10.99.0.1",
        ),
    )
    .await;
    assert_eq!(s, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["error"]["code"], "rate_limited");

    // A different IP is unaffected.
    let (s, _) = send(
        &app,
        req(
            "POST",
            "/api/v1/contact",
            Some(contact_payload(&email)),
            "10.99.0.2",
        ),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED);
}

#[tokio::test]
async fn admin_requires_auth_and_full_flow_works() {
    let (app, pool) = test_app(10).await;

    // Unauthenticated → 401.
    let (s, _) = send(&app, req("GET", "/admin/api/messages", None, "10.0.0.20")).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);

    // Wrong password → 401.
    let (s, _) = send(
        &app,
        req(
            "POST",
            "/admin/api/login",
            Some(json!({ "password": "wrong" })),
            "10.0.0.20",
        ),
    )
    .await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);

    // Correct password → token.
    let (s, body) = send(
        &app,
        req(
            "POST",
            "/admin/api/login",
            Some(json!({ "password": "admin123" })),
            "10.0.0.20",
        ),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let token = body["token"].as_str().expect("token").to_string();

    // Insert one message first (via the public API).
    let email = format!("adminflow-{}@example.com", uuid::Uuid::new_v4());
    send(
        &app,
        req(
            "POST",
            "/api/v1/contact",
            Some(contact_payload(&email)),
            "10.0.0.21",
        ),
    )
    .await;

    // Authenticated list.
    let mut list = req("GET", "/admin/api/messages?status=new", None, "10.0.0.20");
    list.headers_mut().insert(
        header::AUTHORIZATION,
        format!("Bearer {token}").parse().unwrap(),
    );
    let (s, body) = send(&app, list).await;
    assert_eq!(s, StatusCode::OK);
    // Don't assert a clean table — other concurrent tests / prior runs leave
    // messages behind. Find the one this flow just created by its unique email.
    let mine = body["messages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["email"] == email)
        .expect("our message is listed");

    let id = mine["id"].as_str().unwrap().to_string();

    // Mark replied.
    let mut patch = req(
        "PATCH",
        &format!("/admin/api/messages/{id}"),
        Some(json!({ "status": "replied" })),
        "10.0.0.20",
    );
    patch.headers_mut().insert(
        header::AUTHORIZATION,
        format!("Bearer {token}").parse().unwrap(),
    );
    let (s, body) = send(&app, patch).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["status"], "replied");

    let row = sqlx::query!(
        "SELECT status FROM messages WHERE id = $1",
        id.parse::<uuid::Uuid>().unwrap()
    )
    .fetch_one(&pool)
    .await
    .expect("row");
    assert_eq!(row.status, "replied");

    // Stats.
    let mut stats = req("GET", "/admin/api/stats", None, "10.0.0.20");
    stats.headers_mut().insert(
        header::AUTHORIZATION,
        format!("Bearer {token}").parse().unwrap(),
    );
    let (s, body) = send(&app, stats).await;
    assert_eq!(s, StatusCode::OK);
    assert!(body["counts"]["total"].as_i64().unwrap_or(-1) >= 1);
    assert!(body["daily"].is_array());

    // Audit trail was written.
    let n = sqlx::query!("SELECT COUNT(*) AS n FROM admin_events")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(
        n.n.unwrap_or(0) >= 3,
        "login_failed + login_ok + message_status"
    );
}

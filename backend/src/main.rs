use std::net::SocketAddr;

use nikhil_os_backend::app::build_router;
use nikhil_os_backend::config::{Config, LogFormat};
use nikhil_os_backend::db;
use nikhil_os_backend::services::notify::build_sender;
use nikhil_os_backend::state::AppState;

#[tokio::main]
async fn main() {
    let config = Config::from_env();
    init_tracing(&config);
    tracing::info!("starting nikhil-os backend");

    let pool = db::connect(&config.database_url)
        .await
        .unwrap_or_else(|e| panic!("failed to connect to postgres: {e}"));
    db::migrate(&pool)
        .await
        .unwrap_or_else(|e| panic!("failed to run migrations: {e}"));

    let notifier = build_sender(&config);
    let state = AppState::new(pool, config.clone(), notifier);
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .unwrap_or_else(|e| panic!("failed to bind {}: {e}", config.bind_addr));
    tracing::info!(addr = %config.bind_addr, "nikhil-os backend listening");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .expect("server error");
}

fn init_tracing(config: &Config) {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,tower_http=info"));

    let builder = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false);
    match config.log_format {
        LogFormat::Json => builder.json().init(),
        LogFormat::Pretty => builder.init(),
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("install ctrl-c handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("shutdown signal received; draining connections");
}

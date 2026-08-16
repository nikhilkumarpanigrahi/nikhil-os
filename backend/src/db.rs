//! PostgreSQL connection pool + embedded migrations.

use sqlx::postgres::PgPoolOptions;
use sqlx::{migrate, PgPool};

/// Creates a connection pool. `max_connections` stays modest — the free-tier
/// Postgres container has little RAM to spare.
pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .connect(database_url)
        .await
}

/// Applies embedded migrations (`migrations/*.sql`). Idempotent.
pub async fn migrate(pool: &PgPool) -> Result<(), migrate::MigrateError> {
    migrate!("./migrations").run(pool).await
}

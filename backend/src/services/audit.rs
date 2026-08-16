//! Audit trail for admin actions (login attempts, inbox reads, status changes).

use sqlx::PgPool;

pub async fn record(pool: &PgPool, kind: &str, detail: Option<serde_json::Value>) {
    if let Err(e) = sqlx::query!(
        "INSERT INTO admin_events (kind, detail) VALUES ($1, $2)",
        kind,
        detail,
    )
    .execute(pool)
    .await
    {
        // Audit failures must never take down a request path.
        tracing::warn!(error = %e, "failed to write audit event");
    }
}

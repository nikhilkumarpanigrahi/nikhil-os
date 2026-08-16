//! Message persistence + admin operations over the `messages` table.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::types::ipnetwork::IpNetwork;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Message {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub subject: String,
    pub body: String,
    pub topic: String,
    pub origin: Option<String>,
    pub user_agent: Option<String>,
    pub ip: Option<IpNetwork>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}

pub struct NewMessage {
    pub name: String,
    pub email: String,
    pub subject: String,
    pub topic: String,
    pub body: String,
    pub origin: Option<String>,
    pub user_agent: Option<String>,
    pub ip: Option<IpNetwork>,
}

pub async fn insert_message(pool: &PgPool, msg: &NewMessage) -> Result<Message, sqlx::Error> {
    sqlx::query_as!(
        Message,
        r#"
        INSERT INTO messages (name, email, subject, body, topic, origin, user_agent, ip)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING
            id, name, email, subject, body, topic, origin, user_agent, ip, status, created_at, read_at
        "#,
        msg.name,
        msg.email,
        msg.subject,
        msg.body,
        msg.topic,
        msg.origin,
        msg.user_agent,
        msg.ip,
    )
    .fetch_one(pool)
    .await
}

/// Lists messages newest-first with optional status filter and pagination.
/// Built dynamically, so it uses the runtime `query_as` API (macro-safe).
pub async fn list_messages(
    pool: &PgPool,
    status: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<Message>, sqlx::Error> {
    const BASE: &str =
        "SELECT id, name, email, subject, body, topic, origin, user_agent, ip, status, created_at, read_at \
         FROM messages";

    let has_filter = status.is_some();
    // Placeholder numbering: status=$1 when present, then LIMIT/OFFSET.
    let limit_idx = if has_filter { 2 } else { 1 };
    let offset_idx = limit_idx + 1;

    let sql = format!(
        "{base} {filter} ORDER BY created_at DESC LIMIT ${limit} OFFSET ${offset}",
        base = BASE,
        filter = if has_filter { "WHERE status = $1" } else { "" },
        limit = limit_idx,
        offset = offset_idx,
    );

    let mut q = sqlx::query_as::<_, Message>(&sql);
    if let Some(s) = status {
        q = q.bind(s);
    }
    q.bind(limit).bind(offset).fetch_all(pool).await
}

/// Returns false when the id doesn't exist.
pub async fn set_message_status(
    pool: &PgPool,
    id: Uuid,
    status: &str,
) -> Result<bool, sqlx::Error> {
    let res = sqlx::query!(
        "UPDATE messages
         SET status = $1, read_at = CASE WHEN $2 THEN COALESCE(read_at, now()) ELSE read_at END
         WHERE id = $3",
        status,
        status == "read",
        id,
    )
    .execute(pool)
    .await?;
    Ok(res.rows_affected() == 1)
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Stats {
    pub new: i64,
    pub read: i64,
    pub replied: i64,
    pub archived: i64,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DailyVolume {
    pub day: chrono::NaiveDate,
    pub count: i64,
}

pub async fn stats(pool: &PgPool) -> Result<(Stats, Vec<DailyVolume>), sqlx::Error> {
    let row = sqlx::query!(
        "SELECT
            COUNT(*) FILTER (WHERE status = 'new')     AS new_count,
            COUNT(*) FILTER (WHERE status = 'read')    AS read_count,
            COUNT(*) FILTER (WHERE status = 'replied') AS replied_count,
            COUNT(*) FILTER (WHERE status = 'archived') AS archived_count,
            COUNT(*)                                    AS total
         FROM messages"
    )
    .fetch_one(pool)
    .await?;

    let s = Stats {
        new: row.new_count.unwrap_or(0),
        read: row.read_count.unwrap_or(0),
        replied: row.replied_count.unwrap_or(0),
        archived: row.archived_count.unwrap_or(0),
        total: row.total.unwrap_or(0),
    };

    let rows = sqlx::query!(
        "SELECT date_trunc('day', created_at)::date AS day, COUNT(*) AS n
         FROM messages
         GROUP BY 1
         ORDER BY 1 DESC
         LIMIT 14"
    )
    .fetch_all(pool)
    .await?;

    let daily = rows
        .into_iter()
        .filter_map(|r| {
            Some(DailyVolume {
                day: r.day?,
                count: r.n.unwrap_or(0),
            })
        })
        .collect();

    Ok((s, daily))
}

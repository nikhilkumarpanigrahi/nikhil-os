//! Configuration — 12-factor, everything from the environment.
//! See `.env.example` for the full documented contract.

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    pub cors_allowed_origins: Vec<String>,
    pub admin_password_hash: String,
    pub admin_jwt_secret: String,
    pub admin_jwt_ttl_secs: usize,
    pub telegram_bot_token: Option<String>,
    pub telegram_chat_id: Option<String>,
    pub contact_rate_limit_per_minute: u32,
    pub login_rate_limit_per_minute: u32,
    pub log_format: LogFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    Pretty,
    Json,
}

impl Config {
    /// Loads config from the environment, failing fast with a clear message
    /// when a REQUIRED variable is missing (never run with half a config).
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        for key in ["DATABASE_URL", "ADMIN_PASSWORD_HASH", "ADMIN_JWT_SECRET"] {
            if env::var(key).is_err() {
                panic!(
                    "missing required environment variable: {key}\n\
                     see backend/.env.example for the full configuration contract"
                );
            }
        }

        let cors = env::var("CORS_ALLOWED_ORIGINS").unwrap_or_else(|_| {
            "http://localhost:5173,http://localhost:4173,https://nikhilkumarpanigrahi.github.io"
                .to_string()
        });

        Config {
            database_url: env::var("DATABASE_URL").unwrap(),
            bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            cors_allowed_origins: cors
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            admin_password_hash: env::var("ADMIN_PASSWORD_HASH").unwrap(),
            admin_jwt_secret: env::var("ADMIN_JWT_SECRET").unwrap(),
            admin_jwt_ttl_secs: env::var("ADMIN_JWT_TTL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(15 * 60),
            telegram_bot_token: env::var("TELEGRAM_BOT_TOKEN")
                .ok()
                .filter(|s| !s.is_empty()),
            telegram_chat_id: env::var("TELEGRAM_CHAT_ID").ok().filter(|s| !s.is_empty()),
            contact_rate_limit_per_minute: env::var("CONTACT_RATE_LIMIT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10)
                .clamp(1, 60),
            login_rate_limit_per_minute: env::var("LOGIN_RATE_LIMIT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(20)
                .clamp(1, 120),
            log_format: match env::var("LOG_FORMAT").as_deref() {
                Ok("json") => LogFormat::Json,
                _ => LogFormat::Pretty,
            },
        }
    }
}

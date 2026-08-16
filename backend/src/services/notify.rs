//! Outbound alerts, provider-abstracted behind a trait.
//!
//! Telegram is the primary sender (free, instant push to the owner's phone).
//! When no token is configured the notifier degrades to structured logging so
//! the service still behaves predictably in dev.

use async_trait::async_trait;

use crate::config::Config;

#[derive(Debug, thiserror::Error)]
pub enum NotifyError {
    #[error("telegram api error: {0}")]
    Telegram(String),
}

#[async_trait]
pub trait NotificationSender: Send + Sync {
    async fn send(&self, title: &str, body: &str) -> Result<(), NotifyError>;
}

pub struct TelegramNotifier {
    bot_token: String,
    chat_id: String,
    client: reqwest::Client,
}

impl TelegramNotifier {
    pub fn new(bot_token: String, chat_id: String) -> Self {
        Self {
            bot_token,
            chat_id,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl NotificationSender for TelegramNotifier {
    async fn send(&self, title: &str, body: &str) -> Result<(), NotifyError> {
        let text = format!("*{title}*\n{body}");
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);

        let resp = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "chat_id": self.chat_id,
                "text": text,
                "parse_mode": "Markdown",
                "disable_web_page_preview": true,
            }))
            .send()
            .await
            .map_err(|e| NotifyError::Telegram(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(NotifyError::Telegram(format!("{status}: {body}")));
        }
        Ok(())
    }
}

/// Fallback sender: records the alert in the structured log.
pub struct LogNotifier;

#[async_trait]
impl NotificationSender for LogNotifier {
    async fn send(&self, title: &str, body: &str) -> Result<(), NotifyError> {
        tracing::info!(
            notify.title = title,
            notify.body = body,
            "notification (no sender configured; logged only)"
        );
        Ok(())
    }
}

pub fn build_sender(config: &Config) -> Box<dyn NotificationSender + Send + Sync> {
    match (&config.telegram_bot_token, &config.telegram_chat_id) {
        (Some(token), Some(chat)) => {
            tracing::info!("notification sender: telegram");
            Box::new(TelegramNotifier::new(token.clone(), chat.clone()))
        }
        _ => {
            tracing::warn!(
                "no TELEGRAM_BOT_TOKEN/TELEGRAM_CHAT_ID set — alerts will only be logged"
            );
            Box::new(LogNotifier)
        }
    }
}

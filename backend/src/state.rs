//! Shared application state, injected into every handler via `State<AppState>`.

use std::net::IpAddr;
use std::num::NonZeroU32;
use std::sync::Arc;

use governor::clock::DefaultClock;
use governor::middleware::NoOpMiddleware;
use governor::state::keyed::DefaultKeyedStateStore;
use governor::{Quota, RateLimiter};
use sqlx::PgPool;

use crate::config::Config;
use crate::services::notify::NotificationSender;

/// Per-IP rate limiter (governor keyed state). Used for contact + admin login.
/// governor 0.8: `RateLimiter<K, S, C, MW>` where `K` is the key type and `S`
/// the keyed store — `DefaultKeyedStateStore<K>` is the DashMap-backed store.
pub type IpLimiter =
    RateLimiter<IpAddr, DefaultKeyedStateStore<IpAddr>, DefaultClock, NoOpMiddleware>;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
    pub notifier: Arc<dyn NotificationSender + Send + Sync>,
    pub contact_limiter: Arc<IpLimiter>,
    pub login_limiter: Arc<IpLimiter>,
}

fn ip_limiter(per_minute: u32) -> Arc<IpLimiter> {
    let quota = Quota::per_minute(NonZeroU32::new(per_minute).expect("rate limit must be >= 1"));
    Arc::new(RateLimiter::keyed(quota))
}

impl AppState {
    pub fn new(
        pool: PgPool,
        config: Config,
        notifier: Box<dyn NotificationSender + Send + Sync>,
    ) -> Self {
        let contact = ip_limiter(config.contact_rate_limit_per_minute);
        let login = ip_limiter(config.login_rate_limit_per_minute);
        Self {
            pool,
            config,
            notifier: Arc::from(notifier),
            contact_limiter: contact,
            login_limiter: login,
        }
    }
}

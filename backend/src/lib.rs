//! NIKHIL//OS backend — a modular monolith (per docs/02-ARCHITECTURE.md §6).
//!
//! Public API surface (all JSON, versioned):
//!   GET  /healthz                      liveness
//!   GET  /readyz                       readiness (DB ping)
//!   GET  /api/v1/{profile,projects,skills,experience,claims}
//!   POST /api/v1/contact               visitor → inbox
//!   POST /admin/api/login              admin JWT
//!   GET/PATCH /admin/api/messages      protected inbox
//!   GET  /admin/api/stats              protected counts + daily volume
//!   GET  /admin                        self-contained admin panel
//!
//! Library + thin binary split so integration tests can drive the router
//! directly via `tower::ServiceExt::oneshot`.

pub mod app;
pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod knowledge;
pub mod routes;
pub mod services;
pub mod state;

pub use app::build_router;
pub use config::Config;
pub use error::{AppError, Result};
pub use state::AppState;

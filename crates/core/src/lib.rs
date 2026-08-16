//! NIKHIL//OS shared core.
//!
//! A Unix-inspired simulated operating system, compiled to native code
//! (headless operation + tests, future Tauri desktop) and to WebAssembly
//! (Web Edition, via the `wasm` feature).
//!
//! The core has zero UI dependencies. All state is real and observable:
//! processes, scheduler, memory, filesystem (with live `/proc` and `/sys`),
//! IPC, syscalls, services, packages, and structured events.
//!
//! `unsafe` is forbidden: the core is intended to demonstrate security
//! boundaries, so it contains none of its own.

#![forbid(unsafe_code)]
// Public API surface is documented at the module level; every subsystem
// carries a `//!` doc block describing its contracts and invariants.

pub mod event;
pub mod filesystem;
pub mod ipc;
pub mod knowledge;
pub mod logging;
pub mod memory;
pub mod package;
pub mod permissions;
pub mod process;
pub mod scheduler;
pub mod service;
pub mod shell;
pub mod syscall;
pub mod sysinfo;
pub mod system;

/// Semantic version of the core.
pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(feature = "wasm")]
pub mod bridge;

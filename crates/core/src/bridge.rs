//! WebAssembly bridge between the UI and the core.
//!
//! The browser owns a single `System` instance in a thread-local, exactly as
//! a real kernel owns one machine. The React application calls these free
//! functions; nothing in the core is reachable otherwise.

use crate::system::System;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

thread_local! {
    static SYSTEM: RefCell<Option<System>> = const { RefCell::new(None) };
}

fn with_system<F, R>(f: F) -> R
where
    F: FnOnce(&mut System) -> R,
{
    SYSTEM.with(|cell| {
        let mut borrow = cell.borrow_mut();
        f(borrow
            .as_mut()
            .expect("NIKHIL//OS: call init() before using the system"))
    })
}

/// Create a fresh system instance. Safe to call more than once (resets state).
#[wasm_bindgen]
pub fn init() {
    SYSTEM.with(|cell| *cell.borrow_mut() = Some(System::new()));
}

/// Run the boot sequence and return its log.
#[wasm_bindgen]
pub fn boot() -> String {
    with_system(|system| system.boot().join("\n"))
}

/// Advance the simulated clock one tick.
#[wasm_bindgen]
pub fn tick() {
    with_system(|system| system.tick_kernel());
}

/// Execute one shell line and return its output.
#[wasm_bindgen]
pub fn run_command(input: &str) -> String {
    with_system(|system| system.run_command(input))
}

/// The shell prompt for the current directory.
#[wasm_bindgen]
pub fn prompt() -> String {
    with_system(|system| system.prompt())
}

/// Full runtime snapshot as JSON.
#[wasm_bindgen]
pub fn snapshot() -> String {
    with_system(|system| system.snapshot_json())
}

/// List a filesystem directory as JSON.
#[wasm_bindgen]
pub fn list_dir(path: &str) -> String {
    with_system(|system| system.fs_list_json(path))
}

/// Stat a filesystem path as JSON.
#[wasm_bindgen]
pub fn stat_path(path: &str) -> String {
    with_system(|system| system.fs_stat_json(path))
}

/// Read a file (works for virtual `/proc` and `/sys` too).
#[wasm_bindgen]
pub fn read_file(path: &str) -> String {
    with_system(|system| system.fs_read(path))
}

/// The canonical profile as JSON.
#[wasm_bindgen]
pub fn profile() -> String {
    with_system(|system| system.knowledge_json())
}

/// Recent structured events as JSON.
#[wasm_bindgen]
pub fn events(n: usize) -> String {
    with_system(|system| system.events_json(n))
}

/// Autocomplete suggestions for a partial word.
#[wasm_bindgen]
pub fn autocomplete(prefix: &str) -> Vec<String> {
    with_system(|system| system.autocomplete(prefix))
}

/// Core version string.
#[wasm_bindgen]
pub fn version() -> String {
    crate::CORE_VERSION.to_string()
}

//! Structured logging.
//!
//! A thin logger over the event bus: every log line is an observable event.
//! Shells, services, and the Developer Console read the same stream.

use crate::event::{EventBus, Severity};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

/// A single formatted log entry.
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub tick: u64,
    pub severity: Severity,
    pub source: String,
    pub message: String,
}

impl fmt::Display for LogEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{:0>6}] {} {}: {}",
            self.tick, self.severity, self.source, self.message
        )
    }
}

/// Structured logger shared across the system.
pub struct Logger {
    bus: Rc<EventBus>,
    tick: RefCell<u64>,
}

impl Logger {
    pub fn new(bus: Rc<EventBus>) -> Self {
        Self {
            bus,
            tick: RefCell::new(0),
        }
    }

    /// Record the current kernel tick so logs carry timestamps.
    pub fn set_tick(&self, tick: u64) {
        *self.tick.borrow_mut() = tick;
    }

    pub fn log(&self, severity: Severity, source: &str, message: impl AsRef<str>) {
        let tick = *self.tick.borrow();
        self.bus.emit(tick, severity, source, message);
    }

    pub fn debug(&self, source: &str, message: impl AsRef<str>) {
        self.log(Severity::Debug, source, message);
    }

    pub fn info(&self, source: &str, message: impl AsRef<str>) {
        self.log(Severity::Info, source, message);
    }

    pub fn warn(&self, source: &str, message: impl AsRef<str>) {
        self.log(Severity::Warning, source, message);
    }

    pub fn error(&self, source: &str, message: impl AsRef<str>) {
        self.log(Severity::Error, source, message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventBus;

    #[test]
    fn logger_emits_into_the_bus() {
        let bus = Rc::new(EventBus::new());
        let logger = Logger::new(Rc::clone(&bus));
        logger.set_tick(42);
        logger.info("kernel", "boot complete");
        let events = bus.recent(1);
        assert_eq!(events[0].tick, 42);
        assert_eq!(events[0].source, "kernel");
        assert_eq!(events[0].message, "boot complete");
    }
}

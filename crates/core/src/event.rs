//! Structured event bus.
//!
//! Every subsystem emits structured events. Developer Mode and System Monitor
//! consume these. Events are the project's observability contract: if a
//! behavior cannot emit an event, it is not observable, and it is not done.

use std::cell::RefCell;
use std::fmt;

/// Severity of a system event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Debug,
    Info,
    Warning,
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Severity::Debug => "DEBUG",
            Severity::Info => "INFO",
            Severity::Warning => "WARN",
            Severity::Error => "ERROR",
        };
        f.write_str(s)
    }
}

/// A single structured event emitted by a subsystem.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Event {
    pub sequence: u64,
    pub tick: u64,
    pub severity: Severity,
    /// Emitting subsystem, e.g. `process-manager`, `ai-core`, `scheduler`.
    pub source: String,
    /// Human-readable event message, e.g. `spawned pid=17`.
    pub message: String,
    /// Optional machine-readable details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.severity, self.source, self.message)
    }
}

/// Collects events emitted by the system.
#[derive(Default)]
pub struct EventBus {
    events: RefCell<Vec<Event>>,
    next_sequence: RefCell<u64>,
}

impl EventBus {
    pub fn new() -> Self {
        Self::default()
    }

    /// Emit an event at the given kernel tick.
    pub fn emit(&self, tick: u64, severity: Severity, source: &str, message: impl AsRef<str>) {
        let mut seq = self.next_sequence.borrow_mut();
        *seq += 1;
        let mut events = self.events.borrow_mut();
        events.push(Event {
            sequence: *seq,
            tick,
            severity,
            source: source.to_string(),
            message: message.as_ref().to_string(),
            data: None,
        });
        // Bound memory: keep the most recent events only.
        if events.len() > 10_000 {
            let drain = events.len() - 10_000;
            events.drain(0..drain);
        }
    }

    /// Emit an event with attached structured data.
    pub fn emit_with_data(
        &self,
        tick: u64,
        severity: Severity,
        source: &str,
        message: impl AsRef<str>,
        data: impl AsRef<str>,
    ) {
        self.emit(tick, severity, source, message);
        if let Some(last) = self.events.borrow_mut().last_mut() {
            last.data = Some(data.as_ref().to_string());
        }
    }

    /// Most recent `n` events, newest last.
    pub fn recent(&self, n: usize) -> Vec<Event> {
        let events = self.events.borrow();
        let len = events.len();
        let start = len.saturating_sub(n);
        events[start..].to_vec()
    }

    pub fn len(&self) -> usize {
        self.events.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Serialize the most recent events to JSON (for Developer Mode).
    pub fn recent_json(&self, n: usize) -> String {
        let events = self.recent(n);
        serde_json::to_string(&events).unwrap_or_else(|_| "[]".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_are_ordered_and_bounded() {
        let bus = EventBus::new();
        for i in 0..10_100 {
            bus.emit(i, Severity::Info, "test", format!("message {i}"));
        }
        assert_eq!(bus.len(), 10_000);
        let recent = bus.recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].sequence, 10_099);
        assert_eq!(recent[1].sequence, 10_100);
    }

    #[test]
    fn events_serialize_to_json() {
        let bus = EventBus::new();
        bus.emit(1, Severity::Info, "process-manager", "spawned pid=17");
        let json = bus.recent_json(1);
        assert!(json.contains("\"source\":\"process-manager\""));
        assert!(json.contains("spawned pid=17"));
    }
}

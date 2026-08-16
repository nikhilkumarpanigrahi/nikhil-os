//! Typed inter-process communication.
//!
//! All cross-component communication flows through the IPC bus with typed
//! messages: Terminal → Shell, Shell → Service, Service → Knowledge,
//! AI → Service. Services subscribe to the topics they serve; the bus
//! dispatches by topic and returns a typed reply.

use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;

/// A topic namespace for IPC routing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Topic {
    Shell,
    Filesystem,
    ServiceManager,
    PackageManager,
    ProcessManager,
    Knowledge,
    Ai,
    WindowManager,
}

impl Topic {
    pub fn as_str(&self) -> &'static str {
        match self {
            Topic::Shell => "shell",
            Topic::Filesystem => "filesystem",
            Topic::ServiceManager => "service-manager",
            Topic::PackageManager => "package-manager",
            Topic::ProcessManager => "process-manager",
            Topic::Knowledge => "knowledge",
            Topic::Ai => "ai",
            Topic::WindowManager => "window-manager",
        }
    }
}

/// A typed request routed over the bus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: u64,
    pub sender: String,
    pub topic: Topic,
    pub command: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

/// A typed reply.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reply {
    pub id: u64,
    pub ok: bool,
    pub data: serde_json::Value,
    #[serde(default)]
    pub error: Option<String>,
}

impl Reply {
    pub fn ok(id: u64, data: serde_json::Value) -> Self {
        Self {
            id,
            ok: true,
            data,
            error: None,
        }
    }
    pub fn err(id: u64, error: impl Into<String>) -> Self {
        Self {
            id,
            ok: false,
            data: serde_json::Value::Null,
            error: Some(error.into()),
        }
    }
}

/// A handler that answers messages on a topic.
pub trait MessageHandler {
    fn handle(&self, message: &Message) -> Reply;
}

/// The typed IPC bus.
#[derive(Default)]
pub struct IpcBus {
    next_id: RefCell<u64>,
    handlers: RefCell<HashMap<Topic, Box<dyn MessageHandler>>>,
    /// Ring buffer of recent message ids, for observability.
    recent: RefCell<Vec<Message>>,
}

impl IpcBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, topic: Topic, handler: Box<dyn MessageHandler>) {
        self.handlers.borrow_mut().insert(topic, handler);
    }

    /// Send a message and synchronously receive the reply.
    pub fn request(
        &self,
        sender: &str,
        topic: Topic,
        command: &str,
        payload: serde_json::Value,
    ) -> Reply {
        let id = {
            let mut n = self.next_id.borrow_mut();
            *n += 1;
            *n
        };
        let message = Message {
            id,
            sender: sender.to_string(),
            topic: topic.clone(),
            command: command.to_string(),
            payload,
        };
        {
            let mut recent = self.recent.borrow_mut();
            recent.push(message.clone());
            if recent.len() > 500 {
                recent.remove(0);
            }
        }
        let handlers = self.handlers.borrow();
        match handlers.get(&topic) {
            Some(h) => h.handle(&message),
            None => Reply::err(id, format!("no handler for topic {}", topic.as_str())),
        }
    }

    pub fn request_ok(&self, sender: &str, topic: Topic, command: &str) -> Reply {
        self.request(sender, topic, command, serde_json::Value::Null)
    }

    /// Recent messages, for Developer Mode.
    pub fn recent_json(&self, n: usize) -> String {
        let recent = self.recent.borrow();
        let len = recent.len();
        let start = len.saturating_sub(n);
        serde_json::to_string(&recent[start..]).unwrap_or_else(|_| "[]".to_string())
    }
}

/// A handler that answers with a closure over shared state.
pub struct FnHandler<F> {
    f: F,
}

impl<F> FnHandler<F>
where
    F: Fn(&Message) -> Reply,
{
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F> MessageHandler for FnHandler<F>
where
    F: Fn(&Message) -> Reply,
{
    fn handle(&self, message: &Message) -> Reply {
        (self.f)(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_typed_messages() {
        let bus = IpcBus::new();
        bus.register(
            Topic::Knowledge,
            Box::new(FnHandler::new(|m| {
                Reply::ok(m.id, serde_json::json!({ "echo": m.payload }))
            })),
        );
        let reply = bus.request(
            "shell",
            Topic::Knowledge,
            "query",
            serde_json::json!({ "q": "rust" }),
        );
        assert!(reply.ok);
        assert_eq!(reply.data["echo"]["q"], "rust");
        assert!(bus.recent_json(10).contains("rust"));
    }

    #[test]
    fn unknown_topic_returns_error() {
        let bus = IpcBus::new();
        let reply = bus.request("shell", Topic::Ai, "anything", serde_json::Value::Null);
        assert!(!reply.ok);
    }
}

use parking_lot::RwLock;
use serde_derive::Serialize;
use std::collections::VecDeque;
use std::sync::{Arc, OnceLock};
use tracing::Level;

/// Maximum number of logs to keep in memory
const MAX_LOGS: usize = 10000;

/// Log entry structure
#[derive(Clone, Serialize)]
pub struct LogEntry {
    pub timestamp: u64,
    pub level: String,
    pub message: String,
    pub target: Option<String>,
}

/// In-memory log storage
pub struct LogStore {
    logs: Arc<RwLock<VecDeque<LogEntry>>>,
}

impl LogStore {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(RwLock::new(VecDeque::with_capacity(MAX_LOGS))),
        }
    }

    /// Add a log entry
    pub fn add_log(&self, level: Level, message: String, target: Option<String>) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let level_str = match level {
            Level::TRACE => "trace",
            Level::DEBUG => "debug",
            Level::INFO => "info",
            Level::WARN => "warn",
            Level::ERROR => "error",
        }
        .to_string();

        let entry = LogEntry {
            timestamp,
            level: level_str,
            message,
            target,
        };

        let mut logs = self.logs.write();

        // Keep only the last MAX_LOGS entries
        if logs.len() >= MAX_LOGS {
            logs.pop_front();
        }

        logs.push_back(entry);
    }

    /// Get recent logs (up to limit)
    pub fn get_logs(&self, limit: usize) -> Vec<LogEntry> {
        let logs = self.logs.read();
        let start = if logs.len() > limit {
            logs.len() - limit
        } else {
            0
        };

        logs.iter().skip(start).cloned().collect()
    }

    /// Get all logs
    pub fn get_all_logs(&self) -> Vec<LogEntry> {
        self.logs.read().iter().cloned().collect()
    }

    /// Clear all logs
    pub fn clear(&self) {
        self.logs.write().clear();
    }

    /// Clone the store for sharing
    pub fn clone_store(&self) -> Self {
        Self {
            logs: self.logs.clone(),
        }
    }
}

impl Default for LogStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Custom tracing layer that stores logs in memory
pub struct InMemoryLayer {
    store: LogStore,
}

impl InMemoryLayer {
    pub fn new(store: LogStore) -> Self {
        Self { store }
    }
}

impl<S> tracing_subscriber::Layer<S> for InMemoryLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();
        let level = *metadata.level();
        let target = Some(metadata.target().to_string());

        // Extract the message from the event
        let mut message = String::new();
        event.record(&mut MessageVisitor(&mut message));

        self.store.add_log(level, message, target);
    }
}

/// Visitor to extract message from tracing event
struct MessageVisitor<'a>(&'a mut String);

impl<'a> tracing::field::Visit for MessageVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            *self.0 = format!("{:?}", value);
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            *self.0 = value.to_string();
        }
    }
}

/// Global log store instance
static LOG_STORE: OnceLock<LogStore> = OnceLock::new();

/// Initialize the global log store
pub fn init_log_store() -> LogStore {
    let store = LogStore::new();
    let _ = LOG_STORE.set(store.clone_store());
    store
}

/// Get the global log store
pub fn get_log_store() -> Option<LogStore> {
    LOG_STORE.get().map(|s| s.clone_store())
}

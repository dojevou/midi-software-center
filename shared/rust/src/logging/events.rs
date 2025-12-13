//! Structured log events for the application.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A structured log event that can be serialized and sent to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// Log level (error, warn, info, debug, trace)
    pub level: String,
    /// Event category
    pub category: LogCategory,
    /// Human-readable message
    pub message: String,
    /// Target module/component
    pub target: String,
    /// Additional structured fields
    pub fields: serde_json::Value,
    /// Span context if available
    pub span: Option<SpanInfo>,
}

/// Categories of log events.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LogCategory {
    Database,
    Midi,
    File,
    Network,
    System,
    Performance,
    Security,
    User,
    #[default]
    Unknown,
}

impl std::fmt::Display for LogCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database => write!(f, "database"),
            Self::Midi => write!(f, "midi"),
            Self::File => write!(f, "file"),
            Self::Network => write!(f, "network"),
            Self::System => write!(f, "system"),
            Self::Performance => write!(f, "performance"),
            Self::Security => write!(f, "security"),
            Self::User => write!(f, "user"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Information about the span context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanInfo {
    /// Span name
    pub name: String,
    /// Span ID
    pub id: String,
    /// Parent span ID if any
    pub parent_id: Option<String>,
}

impl LogEvent {
    /// Create a new log event.
    pub fn new(level: &str, category: LogCategory, message: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            level: level.to_string(),
            category,
            message: message.into(),
            target: String::new(),
            fields: serde_json::Value::Null,
            span: None,
        }
    }

    /// Add fields to the event.
    pub fn with_fields(mut self, fields: serde_json::Value) -> Self {
        self.fields = fields;
        self
    }

    /// Set the target.
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = target.into();
        self
    }
}

/// Builder for creating structured log events.
pub struct LogEventBuilder {
    event: LogEvent,
}

impl LogEventBuilder {
    pub fn info(category: LogCategory, message: impl Into<String>) -> Self {
        Self { event: LogEvent::new("info", category, message) }
    }

    pub fn warn(category: LogCategory, message: impl Into<String>) -> Self {
        Self { event: LogEvent::new("warn", category, message) }
    }

    pub fn error(category: LogCategory, message: impl Into<String>) -> Self {
        Self { event: LogEvent::new("error", category, message) }
    }

    pub fn debug(category: LogCategory, message: impl Into<String>) -> Self {
        Self { event: LogEvent::new("debug", category, message) }
    }

    pub fn field(mut self, key: &str, value: impl Serialize) -> Self {
        if self.event.fields.is_null() {
            self.event.fields = serde_json::json!({});
        }
        if let serde_json::Value::Object(ref mut map) = self.event.fields {
            map.insert(
                key.to_string(),
                serde_json::to_value(value).unwrap_or_default(),
            );
        }
        self
    }

    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.event.target = target.into();
        self
    }

    pub fn build(self) -> LogEvent {
        self.event
    }
}

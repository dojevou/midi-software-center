//! Logging context for carrying contextual information across spans.

use std::collections::HashMap;
use tracing::Span;

/// Contextual information that can be attached to log entries.
#[derive(Debug, Clone, Default)]
pub struct LogContext {
    /// Unique request/operation ID
    pub request_id: Option<String>,
    /// User session ID if applicable
    pub session_id: Option<String>,
    /// Current file being processed
    pub current_file: Option<String>,
    /// Current operation name
    pub operation: Option<String>,
    /// Additional key-value metadata
    pub metadata: HashMap<String, String>,
}

impl LogContext {
    /// Create a new empty context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a context with a request ID.
    pub fn with_request_id(request_id: impl Into<String>) -> Self {
        Self {
            request_id: Some(request_id.into()),
            ..Default::default()
        }
    }

    /// Add a metadata field.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set the current file.
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.current_file = Some(file.into());
        self
    }

    /// Set the operation name.
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }

    /// Generate a new unique request ID.
    pub fn generate_request_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros();
        format!("req-{:x}-{:04x}", timestamp, rand::random::<u16>())
    }

    /// Create a tracing span with this context.
    pub fn into_span(self, name: &'static str) -> Span {
        let span = tracing::info_span!(
            "operation",
            otel.name = name,
            request_id = tracing::field::Empty,
            session_id = tracing::field::Empty,
            file = tracing::field::Empty,
        );

        if let Some(ref id) = self.request_id {
            span.record("request_id", id.as_str());
        }
        if let Some(ref id) = self.session_id {
            span.record("session_id", id.as_str());
        }
        if let Some(ref file) = self.current_file {
            span.record("file", file.as_str());
        }

        span
    }
}

/// Guard for timing operations.
pub struct TimingGuard {
    operation: String,
    start: std::time::Instant,
    threshold_ms: Option<u64>,
}

impl TimingGuard {
    /// Create a new timing guard for an operation.
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            start: std::time::Instant::now(),
            threshold_ms: None,
        }
    }

    /// Log a warning if the operation exceeds this threshold.
    pub fn with_threshold(mut self, threshold_ms: u64) -> Self {
        self.threshold_ms = Some(threshold_ms);
        self
    }
}

impl Drop for TimingGuard {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        let duration_ms = duration.as_millis() as u64;

        if let Some(threshold) = self.threshold_ms {
            if duration_ms > threshold {
                tracing::warn!(
                    operation = %self.operation,
                    duration_ms = duration_ms,
                    threshold_ms = threshold,
                    "Operation exceeded time threshold"
                );
                return;
            }
        }

        tracing::debug!(
            operation = %self.operation,
            duration_ms = duration_ms,
            "Operation completed"
        );
    }
}

// Helper for generating random numbers without full rand crate
mod rand {
    use std::cell::Cell;
    use std::time::{SystemTime, UNIX_EPOCH};

    thread_local! {
        static SEED: Cell<u64> = Cell::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64
        );
    }

    pub fn random<T: FromRandom>() -> T {
        SEED.with(|seed| {
            let mut s = seed.get();
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
            seed.set(s);
            T::from_random(s)
        })
    }

    pub trait FromRandom {
        fn from_random(value: u64) -> Self;
    }

    impl FromRandom for u16 {
        fn from_random(value: u64) -> Self {
            value as u16
        }
    }
}

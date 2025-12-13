# TERMINAL 4: Logging Infrastructure

## Owner: Claude Instance #4
## Components: Centralized Logging with tracing crate, Log Aggregation, Structured Logging

---

## PART A: RUST LOGGING INFRASTRUCTURE

### A1. Add Dependencies to All Cargo.toml Files

Add to `daw/src-tauri/Cargo.toml`, `pipeline/src-tauri/Cargo.toml`, and `shared/rust/Cargo.toml`:

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json", "fmt"] }
tracing-appender = "0.2"
tracing-error = "0.2"
chrono = { version = "0.4", features = ["serde"] }
serde_json = "1.0"
```

### A2. Create Logging Module (`shared/rust/src/logging/mod.rs`)

```rust
//! Centralized logging infrastructure for the MIDI Software Center.
//!
//! Provides structured logging with multiple outputs (file, console, Tauri events),
//! performance tracing, and log aggregation capabilities.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{Level, Subscriber};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan, time::UtcTime},
    layer::SubscriberExt,
    registry::LookupSpan,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

pub mod context;
pub mod events;
pub mod metrics;

pub use context::LogContext;
pub use events::LogEvent;
pub use metrics::PerformanceMetrics;

static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Configuration for the logging system.
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Log directory path
    pub log_dir: PathBuf,
    /// Application name for log files
    pub app_name: String,
    /// Minimum log level for console output
    pub console_level: Level,
    /// Minimum log level for file output
    pub file_level: Level,
    /// Whether to include file/line info
    pub include_location: bool,
    /// Whether to use JSON format for file logs
    pub json_format: bool,
    /// Maximum log file size before rotation (in MB)
    pub max_file_size_mb: u64,
    /// Number of rotated log files to keep
    pub max_files: usize,
    /// Enable performance span timing
    pub enable_span_timing: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            log_dir: PathBuf::from("logs"),
            app_name: String::from("midi-software-center"),
            console_level: Level::INFO,
            file_level: Level::DEBUG,
            include_location: true,
            json_format: true,
            max_file_size_mb: 10,
            max_files: 5,
            enable_span_timing: true,
        }
    }
}

/// Initialize the logging system with the given configuration.
///
/// This should be called once at application startup. Subsequent calls are ignored.
pub fn init_logging(config: LogConfig) -> Result<(), LoggingError> {
    if INITIALIZED.swap(true, Ordering::SeqCst) {
        return Ok(()); // Already initialized
    }

    // Ensure log directory exists
    std::fs::create_dir_all(&config.log_dir).map_err(|e| LoggingError::IoError(e.to_string()))?;

    // Create file appender with daily rotation
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        &config.log_dir,
        format!("{}.log", config.app_name),
    );

    // Build the subscriber layers
    let registry = tracing_subscriber::registry();

    // Console layer - human-readable format
    let console_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(config.console_level.to_string()));

    let console_layer = fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(false)
        .with_file(config.include_location)
        .with_line_number(config.include_location)
        .with_ansi(true)
        .with_filter(console_filter);

    // File layer - JSON format for structured logging
    let file_filter = EnvFilter::new(config.file_level.to_string());

    let file_layer = if config.json_format {
        fmt::layer()
            .json()
            .with_timer(UtcTime::rfc_3339())
            .with_target(true)
            .with_current_span(true)
            .with_span_list(true)
            .with_file(config.include_location)
            .with_line_number(config.include_location)
            .with_writer(file_appender)
            .with_filter(file_filter)
            .boxed()
    } else {
        fmt::layer()
            .with_timer(UtcTime::rfc_3339())
            .with_target(true)
            .with_file(config.include_location)
            .with_line_number(config.include_location)
            .with_writer(file_appender)
            .with_filter(file_filter)
            .boxed()
    };

    // Optional span timing layer
    let span_layer = if config.enable_span_timing {
        Some(
            fmt::layer()
                .with_span_events(FmtSpan::CLOSE)
                .with_filter(EnvFilter::new("info")),
        )
    } else {
        None
    };

    // Initialize the subscriber
    registry
        .with(console_layer)
        .with(file_layer)
        .with(span_layer)
        .try_init()
        .map_err(|e| LoggingError::InitError(e.to_string()))?;

    tracing::info!(
        app = %config.app_name,
        log_dir = %config.log_dir.display(),
        "Logging system initialized"
    );

    Ok(())
}

/// Errors that can occur during logging operations.
#[derive(Debug, thiserror::Error)]
pub enum LoggingError {
    #[error("Failed to initialize logging: {0}")]
    InitError(String),
    #[error("IO error: {0}")]
    IoError(String),
}

/// Convenience macros for structured logging with context.
#[macro_export]
macro_rules! log_operation {
    ($level:expr, $operation:expr, $($field:tt)*) => {
        tracing::event!(
            $level,
            operation = $operation,
            $($field)*
        )
    };
}

#[macro_export]
macro_rules! log_database {
    ($level:expr, $query:expr, $duration_ms:expr) => {
        tracing::event!(
            $level,
            category = "database",
            query = $query,
            duration_ms = $duration_ms,
        )
    };
    ($level:expr, $query:expr, $duration_ms:expr, $($field:tt)*) => {
        tracing::event!(
            $level,
            category = "database",
            query = $query,
            duration_ms = $duration_ms,
            $($field)*
        )
    };
}

#[macro_export]
macro_rules! log_midi {
    ($level:expr, $event_type:expr, $($field:tt)*) => {
        tracing::event!(
            $level,
            category = "midi",
            event_type = $event_type,
            $($field)*
        )
    };
}

#[macro_export]
macro_rules! log_file {
    ($level:expr, $operation:expr, $path:expr) => {
        tracing::event!(
            $level,
            category = "file",
            operation = $operation,
            path = %$path,
        )
    };
    ($level:expr, $operation:expr, $path:expr, $($field:tt)*) => {
        tracing::event!(
            $level,
            category = "file",
            operation = $operation,
            path = %$path,
            $($field)*
        )
    };
}
```

### A3. Create Log Context Module (`shared/rust/src/logging/context.rs`)

```rust
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
```

### A4. Create Log Events Module (`shared/rust/src/logging/events.rs`)

```rust
//! Structured log events for the application.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    Unknown,
}

impl Default for LogCategory {
    fn default() -> Self {
        Self::Unknown
    }
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
        Self {
            event: LogEvent::new("info", category, message),
        }
    }

    pub fn warn(category: LogCategory, message: impl Into<String>) -> Self {
        Self {
            event: LogEvent::new("warn", category, message),
        }
    }

    pub fn error(category: LogCategory, message: impl Into<String>) -> Self {
        Self {
            event: LogEvent::new("error", category, message),
        }
    }

    pub fn debug(category: LogCategory, message: impl Into<String>) -> Self {
        Self {
            event: LogEvent::new("debug", category, message),
        }
    }

    pub fn field(mut self, key: &str, value: impl Serialize) -> Self {
        if self.event.fields.is_null() {
            self.event.fields = serde_json::json!({});
        }
        if let serde_json::Value::Object(ref mut map) = self.event.fields {
            map.insert(key.to_string(), serde_json::to_value(value).unwrap_or_default());
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
```

### A5. Create Performance Metrics Module (`shared/rust/src/logging/metrics.rs`)

```rust
//! Performance metrics collection and reporting.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Global metrics registry.
lazy_static::lazy_static! {
    static ref METRICS: RwLock<MetricsRegistry> = RwLock::new(MetricsRegistry::new());
}

/// Performance metrics for various operations.
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    /// Operation counts
    pub counters: HashMap<String, u64>,
    /// Operation timing statistics
    pub timings: HashMap<String, TimingStats>,
    /// Gauge values (current state)
    pub gauges: HashMap<String, f64>,
}

/// Timing statistics for an operation.
#[derive(Debug, Clone)]
pub struct TimingStats {
    pub count: u64,
    pub total_ms: u64,
    pub min_ms: u64,
    pub max_ms: u64,
    pub avg_ms: f64,
}

impl Default for TimingStats {
    fn default() -> Self {
        Self {
            count: 0,
            total_ms: 0,
            min_ms: u64::MAX,
            max_ms: 0,
            avg_ms: 0.0,
        }
    }
}

/// Internal metrics registry.
struct MetricsRegistry {
    counters: HashMap<String, AtomicU64>,
    timings: HashMap<String, RwLock<TimingAccumulator>>,
    gauges: HashMap<String, RwLock<f64>>,
}

struct TimingAccumulator {
    count: u64,
    total_ns: u64,
    min_ns: u64,
    max_ns: u64,
}

impl MetricsRegistry {
    fn new() -> Self {
        Self {
            counters: HashMap::new(),
            timings: HashMap::new(),
            gauges: HashMap::new(),
        }
    }
}

/// Increment a counter metric.
pub fn increment_counter(name: &str) {
    increment_counter_by(name, 1);
}

/// Increment a counter by a specific amount.
pub fn increment_counter_by(name: &str, amount: u64) {
    let registry = METRICS.read().unwrap();
    if let Some(counter) = registry.counters.get(name) {
        counter.fetch_add(amount, Ordering::Relaxed);
    } else {
        drop(registry);
        let mut registry = METRICS.write().unwrap();
        registry
            .counters
            .entry(name.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(amount, Ordering::Relaxed);
    }
}

/// Record a timing measurement.
pub fn record_timing(name: &str, duration: Duration) {
    let ns = duration.as_nanos() as u64;
    let registry = METRICS.read().unwrap();

    if let Some(timing) = registry.timings.get(name) {
        let mut acc = timing.write().unwrap();
        acc.count += 1;
        acc.total_ns += ns;
        acc.min_ns = acc.min_ns.min(ns);
        acc.max_ns = acc.max_ns.max(ns);
    } else {
        drop(registry);
        let mut registry = METRICS.write().unwrap();
        registry.timings.entry(name.to_string()).or_insert_with(|| {
            RwLock::new(TimingAccumulator {
                count: 1,
                total_ns: ns,
                min_ns: ns,
                max_ns: ns,
            })
        });
    }
}

/// Set a gauge value.
pub fn set_gauge(name: &str, value: f64) {
    let registry = METRICS.read().unwrap();

    if let Some(gauge) = registry.gauges.get(name) {
        *gauge.write().unwrap() = value;
    } else {
        drop(registry);
        let mut registry = METRICS.write().unwrap();
        registry
            .gauges
            .entry(name.to_string())
            .or_insert_with(|| RwLock::new(0.0));
        if let Some(gauge) = registry.gauges.get(name) {
            *gauge.write().unwrap() = value;
        }
    }
}

/// Get current performance metrics snapshot.
pub fn get_metrics() -> PerformanceMetrics {
    let registry = METRICS.read().unwrap();
    let mut metrics = PerformanceMetrics::default();

    // Counters
    for (name, counter) in &registry.counters {
        metrics
            .counters
            .insert(name.clone(), counter.load(Ordering::Relaxed));
    }

    // Timings
    for (name, timing) in &registry.timings {
        let acc = timing.read().unwrap();
        if acc.count > 0 {
            metrics.timings.insert(
                name.clone(),
                TimingStats {
                    count: acc.count,
                    total_ms: acc.total_ns / 1_000_000,
                    min_ms: acc.min_ns / 1_000_000,
                    max_ms: acc.max_ns / 1_000_000,
                    avg_ms: (acc.total_ns as f64 / acc.count as f64) / 1_000_000.0,
                },
            );
        }
    }

    // Gauges
    for (name, gauge) in &registry.gauges {
        metrics.gauges.insert(name.clone(), *gauge.read().unwrap());
    }

    metrics
}

/// Reset all metrics.
pub fn reset_metrics() {
    let mut registry = METRICS.write().unwrap();
    registry.counters.clear();
    registry.timings.clear();
    registry.gauges.clear();
}

/// Timer guard for automatic timing measurement.
pub struct Timer {
    name: String,
    start: Instant,
}

impl Timer {
    /// Start a new timer.
    pub fn start(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        record_timing(&self.name, self.start.elapsed());
    }
}

/// Macro for timing a block of code.
#[macro_export]
macro_rules! time_operation {
    ($name:expr, $block:expr) => {{
        let _timer = $crate::logging::metrics::Timer::start($name);
        $block
    }};
}
```

### A6. Add lib.rs Export

Add to `shared/rust/src/lib.rs`:

```rust
pub mod logging;

// Re-exports
pub use logging::{init_logging, LogConfig, LogContext, LogEvent, PerformanceMetrics};
```

---

## PART B: TAURI COMMANDS FOR LOG ACCESS

### B1. Create Log Commands (`daw/src-tauri/src/commands/logging.rs`)

```rust
use shared::logging::{get_metrics, LogConfig, LogEvent, PerformanceMetrics};
use std::path::PathBuf;
use tauri::State;
use tokio::fs;
use tokio::io::AsyncBufReadExt;

/// Get recent log entries from the log file.
#[tauri::command]
pub async fn get_recent_logs(
    limit: Option<usize>,
    level: Option<String>,
    category: Option<String>,
) -> Result<Vec<LogEvent>, String> {
    let limit = limit.unwrap_or(100);
    let log_path = get_log_path()?;

    let file = fs::File::open(&log_path)
        .await
        .map_err(|e| format!("Failed to open log file: {}", e))?;

    let reader = tokio::io::BufReader::new(file);
    let mut lines = reader.lines();
    let mut events: Vec<LogEvent> = Vec::new();

    while let Some(line) = lines
        .next_line()
        .await
        .map_err(|e| format!("Failed to read log: {}", e))?
    {
        if let Ok(event) = serde_json::from_str::<LogEvent>(&line) {
            // Filter by level if specified
            if let Some(ref lvl) = level {
                if event.level.to_lowercase() != lvl.to_lowercase() {
                    continue;
                }
            }

            // Filter by category if specified
            if let Some(ref cat) = category {
                if event.category.to_string() != *cat {
                    continue;
                }
            }

            events.push(event);
        }
    }

    // Return last N entries
    let start = events.len().saturating_sub(limit);
    Ok(events[start..].to_vec())
}

/// Get performance metrics.
#[tauri::command]
pub async fn get_performance_metrics() -> Result<PerformanceMetrics, String> {
    Ok(get_metrics())
}

/// Clear log file.
#[tauri::command]
pub async fn clear_logs() -> Result<(), String> {
    let log_path = get_log_path()?;
    fs::write(&log_path, "")
        .await
        .map_err(|e| format!("Failed to clear logs: {}", e))?;
    tracing::info!("Log file cleared");
    Ok(())
}

/// Export logs to a file.
#[tauri::command]
pub async fn export_logs(output_path: String) -> Result<(), String> {
    let log_path = get_log_path()?;
    fs::copy(&log_path, &output_path)
        .await
        .map_err(|e| format!("Failed to export logs: {}", e))?;
    tracing::info!(path = %output_path, "Logs exported");
    Ok(())
}

/// Get log file path.
fn get_log_path() -> Result<PathBuf, String> {
    let log_dir = dirs::data_local_dir()
        .ok_or_else(|| "Could not determine data directory".to_string())?
        .join("midi-software-center")
        .join("logs");

    Ok(log_dir.join("midi-software-center.log"))
}

/// Stream log events to the frontend via Tauri events.
#[tauri::command]
pub async fn subscribe_to_logs(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Emitter;

    // This would typically use a channel or subscriber to forward logs
    // For now, we'll just acknowledge the subscription
    app.emit("log-subscribed", ())
        .map_err(|e| format!("Failed to emit: {}", e))?;

    Ok(())
}
```

### B2. Register Commands in lib.rs

Add to `daw/src-tauri/src/lib.rs`:

```rust
mod commands;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // ... existing commands ...
            commands::logging::get_recent_logs,
            commands::logging::get_performance_metrics,
            commands::logging::clear_logs,
            commands::logging::export_logs,
            commands::logging::subscribe_to_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## PART C: FRONTEND LOG VIEWER

### C1. Create Log Viewer Store (`app/src/lib/stores/logStore.ts`)

```typescript
import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export interface LogEntry {
  timestamp: string;
  level: string;
  category: string;
  message: string;
  target: string;
  fields: Record<string, unknown>;
  span?: {
    name: string;
    id: string;
    parent_id?: string;
  };
}

export interface PerformanceMetrics {
  counters: Record<string, number>;
  timings: Record<string, {
    count: number;
    total_ms: number;
    min_ms: number;
    max_ms: number;
    avg_ms: number;
  }>;
  gauges: Record<string, number>;
}

export interface LogState {
  entries: LogEntry[];
  isStreaming: boolean;
  filter: {
    level: string | null;
    category: string | null;
    search: string;
  };
  metrics: PerformanceMetrics | null;
}

const initialState: LogState = {
  entries: [],
  isStreaming: false,
  filter: {
    level: null,
    category: null,
    search: '',
  },
  metrics: null,
};

function createLogStore() {
  const { subscribe, set, update } = writable<LogState>(initialState);

  return {
    subscribe,

    async fetchLogs(limit = 100, level?: string, category?: string) {
      try {
        const entries = await invoke<LogEntry[]>('get_recent_logs', {
          limit,
          level,
          category,
        });
        update(state => ({ ...state, entries }));
      } catch (e) {
        console.error('Failed to fetch logs:', e);
      }
    },

    async fetchMetrics() {
      try {
        const metrics = await invoke<PerformanceMetrics>('get_performance_metrics');
        update(state => ({ ...state, metrics }));
      } catch (e) {
        console.error('Failed to fetch metrics:', e);
      }
    },

    async clearLogs() {
      try {
        await invoke('clear_logs');
        update(state => ({ ...state, entries: [] }));
      } catch (e) {
        console.error('Failed to clear logs:', e);
      }
    },

    async exportLogs(outputPath: string) {
      try {
        await invoke('export_logs', { outputPath });
      } catch (e) {
        console.error('Failed to export logs:', e);
        throw e;
      }
    },

    setFilter(filter: Partial<LogState['filter']>) {
      update(state => ({
        ...state,
        filter: { ...state.filter, ...filter },
      }));
    },

    addEntry(entry: LogEntry) {
      update(state => ({
        ...state,
        entries: [...state.entries.slice(-999), entry],
      }));
    },

    async startStreaming() {
      try {
        await invoke('subscribe_to_logs');
        const unlisten = await listen<LogEntry>('log-entry', (event) => {
          this.addEntry(event.payload);
        });
        update(state => ({ ...state, isStreaming: true }));
        return unlisten;
      } catch (e) {
        console.error('Failed to start log streaming:', e);
      }
    },
  };
}

export const logStore = createLogStore();

export const filteredLogs = derived(logStore, $log => {
  let entries = $log.entries;

  if ($log.filter.level) {
    entries = entries.filter(e => e.level.toLowerCase() === $log.filter.level?.toLowerCase());
  }

  if ($log.filter.category) {
    entries = entries.filter(e => e.category === $log.filter.category);
  }

  if ($log.filter.search) {
    const search = $log.filter.search.toLowerCase();
    entries = entries.filter(e =>
      e.message.toLowerCase().includes(search) ||
      e.target.toLowerCase().includes(search)
    );
  }

  return entries;
});
```

### C2. Create Log Viewer Component (`app/src/lib/components/LogViewer.svelte`)

```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { logStore, filteredLogs } from '$lib/stores/logStore';
  import type { LogEntry } from '$lib/stores/logStore';

  export let maxHeight = '400px';
  export let autoScroll = true;

  let containerRef: HTMLDivElement;
  let unlisten: (() => void) | undefined;

  const levels = ['error', 'warn', 'info', 'debug', 'trace'];
  const categories = ['database', 'midi', 'file', 'network', 'system', 'performance'];

  function getLevelColor(level: string): string {
    switch (level.toLowerCase()) {
      case 'error': return 'var(--error)';
      case 'warn': return 'var(--warning)';
      case 'info': return 'var(--info)';
      case 'debug': return 'var(--text-muted)';
      case 'trace': return 'var(--text-muted)';
      default: return 'var(--text-secondary)';
    }
  }

  function formatTimestamp(ts: string): string {
    return new Date(ts).toLocaleTimeString();
  }

  function scrollToBottom() {
    if (autoScroll && containerRef) {
      containerRef.scrollTop = containerRef.scrollHeight;
    }
  }

  $: if ($filteredLogs.length && autoScroll) {
    requestAnimationFrame(scrollToBottom);
  }

  onMount(async () => {
    await logStore.fetchLogs();
    unlisten = await logStore.startStreaming();
  });

  onDestroy(() => {
    unlisten?.();
  });
</script>

<div class="log-viewer">
  <div class="log-controls">
    <select
      value={$logStore.filter.level || ''}
      on:change={(e) => logStore.setFilter({ level: e.currentTarget.value || null })}
      aria-label="Filter by level"
    >
      <option value="">All Levels</option>
      {#each levels as level}
        <option value={level}>{level.toUpperCase()}</option>
      {/each}
    </select>

    <select
      value={$logStore.filter.category || ''}
      on:change={(e) => logStore.setFilter({ category: e.currentTarget.value || null })}
      aria-label="Filter by category"
    >
      <option value="">All Categories</option>
      {#each categories as category}
        <option value={category}>{category}</option>
      {/each}
    </select>

    <input
      type="text"
      placeholder="Search logs..."
      value={$logStore.filter.search}
      on:input={(e) => logStore.setFilter({ search: e.currentTarget.value })}
      aria-label="Search logs"
    />

    <label class="auto-scroll">
      <input type="checkbox" bind:checked={autoScroll} />
      Auto-scroll
    </label>

    <button on:click={() => logStore.clearLogs()}>Clear</button>
    <button on:click={() => logStore.fetchLogs()}>Refresh</button>
  </div>

  <div
    bind:this={containerRef}
    class="log-entries"
    style="max-height: {maxHeight}"
    role="log"
    aria-live="polite"
  >
    {#each $filteredLogs as entry (entry.timestamp + entry.message)}
      <div class="log-entry" class:error={entry.level === 'error'} class:warn={entry.level === 'warn'}>
        <span class="log-time">{formatTimestamp(entry.timestamp)}</span>
        <span class="log-level" style="color: {getLevelColor(entry.level)}">
          [{entry.level.toUpperCase()}]
        </span>
        <span class="log-category">[{entry.category}]</span>
        <span class="log-message">{entry.message}</span>
        {#if entry.target}
          <span class="log-target">{entry.target}</span>
        {/if}
      </div>
    {/each}

    {#if $filteredLogs.length === 0}
      <div class="log-empty">No log entries</div>
    {/if}
  </div>

  <div class="log-footer">
    <span>{$filteredLogs.length} entries</span>
    {#if $logStore.isStreaming}
      <span class="streaming-indicator">● Streaming</span>
    {/if}
  </div>
</div>

<style>
  .log-viewer {
    display: flex;
    flex-direction: column;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--border-radius);
    overflow: hidden;
  }

  .log-controls {
    display: flex;
    gap: 8px;
    padding: 8px;
    border-bottom: 1px solid var(--border);
    flex-wrap: wrap;
  }

  .log-controls select,
  .log-controls input[type="text"] {
    padding: 4px 8px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--border-radius-sm);
    color: var(--text-primary);
    font-size: var(--font-size-sm);
  }

  .log-controls input[type="text"] {
    flex: 1;
    min-width: 150px;
  }

  .auto-scroll {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }

  .log-controls button {
    padding: 4px 12px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--border-radius-sm);
    color: var(--text-secondary);
    cursor: pointer;
    font-size: var(--font-size-sm);
  }

  .log-controls button:hover {
    background: var(--bg-hover);
  }

  .log-entries {
    flex: 1;
    overflow-y: auto;
    font-family: monospace;
    font-size: 12px;
  }

  .log-entry {
    display: flex;
    gap: 8px;
    padding: 4px 8px;
    border-bottom: 1px solid var(--border);
    white-space: nowrap;
  }

  .log-entry:hover {
    background: var(--bg-hover);
  }

  .log-entry.error {
    background: rgba(220, 53, 69, 0.1);
  }

  .log-entry.warn {
    background: rgba(255, 193, 7, 0.1);
  }

  .log-time {
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .log-level {
    font-weight: 600;
    flex-shrink: 0;
    width: 60px;
  }

  .log-category {
    color: var(--accent);
    flex-shrink: 0;
    width: 100px;
  }

  .log-message {
    color: var(--text-primary);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .log-target {
    color: var(--text-muted);
    font-size: 10px;
    flex-shrink: 0;
  }

  .log-empty {
    padding: 24px;
    text-align: center;
    color: var(--text-muted);
  }

  .log-footer {
    display: flex;
    justify-content: space-between;
    padding: 4px 8px;
    border-top: 1px solid var(--border);
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }

  .streaming-indicator {
    color: var(--success);
    animation: pulse 2s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
</style>
```

---

## TESTING CHECKLIST

### Rust Logging
- [ ] Log files are created in correct directory
- [ ] Console output shows colored log levels
- [ ] File logs are in JSON format
- [ ] Log rotation works correctly
- [ ] Performance metrics are recorded
- [ ] Log macros work correctly
- [ ] Timing guards measure operations
- [ ] Context propagates through spans

### Frontend Integration
- [ ] Logs can be fetched from backend
- [ ] Log filtering works by level
- [ ] Log filtering works by category
- [ ] Log search works
- [ ] Log streaming shows new entries
- [ ] Performance metrics display correctly
- [ ] Clear logs works
- [ ] Export logs works

---

## FILES TO CREATE/MODIFY

| File | Action |
|------|--------|
| `shared/rust/src/logging/mod.rs` | CREATE |
| `shared/rust/src/logging/context.rs` | CREATE |
| `shared/rust/src/logging/events.rs` | CREATE |
| `shared/rust/src/logging/metrics.rs` | CREATE |
| `shared/rust/src/lib.rs` | MODIFY - Add logging module |
| `shared/rust/Cargo.toml` | MODIFY - Add tracing deps |
| `daw/src-tauri/src/commands/logging.rs` | CREATE |
| `daw/src-tauri/src/commands/mod.rs` | MODIFY - Add logging module |
| `daw/src-tauri/src/lib.rs` | MODIFY - Register commands |
| `daw/src-tauri/Cargo.toml` | MODIFY - Add deps |
| `app/src/lib/stores/logStore.ts` | CREATE |
| `app/src/lib/components/LogViewer.svelte` | CREATE |

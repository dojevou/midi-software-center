//! Centralized logging infrastructure for the MIDI Software Center.
//!
//! Provides structured logging with multiple outputs (file, console, Tauri events),
//! performance tracing, and log aggregation capabilities.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
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

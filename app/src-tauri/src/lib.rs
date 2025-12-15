/// MIDI Software Center - Unified Application
///
/// Combines Pipeline (batch processing, database) and DAW (real-time playback)
/// into a single desktop application.
///
/// ## Structure
///
/// - `core::midi` - MIDI parsing and types
/// - `core::analysis` - Musical analysis (BPM, key detection, etc.)
/// - `db::models` - Database model types
/// - `db::repositories` - Database access layer
/// - `health` - Service health monitoring
/// - `logging` - Structured logging

// Core modules (migrated from shared/rust)
pub mod core;
pub mod db;
pub mod health;
pub mod logging;

// App-specific modules
pub mod shared;

// Re-export top-level modules for convenience
pub use core::analysis;
pub use core::midi;
pub use db::{models, repositories};
pub use health::{HealthChecker, HealthStatus, ServiceHealth, SystemHealth};
pub use logging::{init_logging, LogConfig, LogContext, LogEvent, PerformanceMetrics};

// Re-export the command registry for easy access
pub use shared::TauriCommands;

// Re-export Pipeline library (package: midi-pipeline, lib: midi_pipeline)
pub use midi_pipeline;

// Re-export DAW library (package: midi-software-center-daw, lib: midi_software_center_daw)
pub use midi_software_center_daw;

// Re-export commonly used types from both libraries
pub use midi_pipeline::{AppError as PipelineError, AppResult as PipelineResult};
pub use midi_software_center_daw::{AppError as DAWError, AppResult as DAWResult};

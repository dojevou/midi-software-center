// Clippy lint configuration
#![allow(clippy::too_many_arguments)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::ptr_arg)]

/// MIDI Software Center - Unified Application
///
/// Combines Pipeline (batch processing, database) and DAW (real-time playback)
/// into a single desktop application.
///
/// ## Structure
///
/// - `core::midi` - MIDI parsing and types
/// - `core::analysis` - Musical analysis (BPM, key detection, etc.)
/// - `core::hash` - Hashing utilities (BLAKE3)
/// - `core::naming` - File naming and sanitization
/// - `core::normalization` - Filename normalization
/// - `core::performance` - Performance utilities
/// - `core::pipeline` - Batch processing pipeline
/// - `core::splitting` - Track splitting
/// - `db::models` - Database model types
/// - `db::repositories` - Database access layer
/// - `io` - I/O operations (decompressor, etc.)
/// - `database` - Database connection management
/// - `commands` - Tauri commands (pipeline submodule)
/// - `windows` - Window management
/// - `health` - Service health monitoring
/// - `logging` - Structured logging

// Use mimalloc as global allocator for better performance
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// Core modules
pub mod core;
pub mod db;
pub mod health;
pub mod logging;

// Pipeline modules (migrated from pipeline/src-tauri)
pub mod commands;
pub mod database;
pub mod error;
pub mod io;
pub mod windows;

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

// Re-export database connection
pub use database::window_state::{
    DatabaseWindowState, PaginationInfo, SearchFilters, SearchResult, SortField, SortOrder,
    ViewMode,
};
pub use database::Database;

// Re-export error types
pub use error::{AppError, AppResult, TauriResult};

// Re-export pipeline models
pub use db::models::{
    bigdecimal_to_f64, f64_to_bigdecimal, MusicalMetadata, NewFile, NewMusicalMetadata,
    PipelineFile, PipelineSearchFilters, PipelineSearchResults,
};

// Re-export DAW library (package: midi-software-center-daw, lib: midi_software_center_daw)
pub use midi_software_center_daw;
pub use midi_software_center_daw::{AppError as DAWError, AppResult as DAWResult};

/// Application state shared across all Tauri commands
#[derive(Clone)]
pub struct AppState {
    pub database: Database,
}

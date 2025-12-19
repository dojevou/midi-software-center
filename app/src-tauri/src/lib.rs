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
/// ### Core Modules
/// - `core::midi` - MIDI parsing and types (analysis_parser for pipeline, playback_parser for DAW)
/// - `core::analysis` - Musical analysis (BPM, key detection, etc.)
/// - `core::hash` - Hashing utilities (BLAKE3)
/// - `core::naming` - File naming and sanitization
/// - `core::normalization` - Filename normalization
/// - `core::performance` - Performance utilities
/// - `core::pipeline` - Batch processing pipeline
/// - `core::splitting` - Track splitting
/// - `core::sequencer` - DAW sequencer engine (real-time)
/// - `core::compatibility` - DAW compatibility helpers
///
/// ### DAW Modules (real-time MIDI)
/// - `hardware` - MIDI hardware integration
/// - `midi` - Real-time MIDI I/O
/// - `midi_clock` - MIDI clock sync
/// - `editors` - MIDI editors (piano roll, etc.)
/// - `sequencer` - High-level sequencer module
/// - `scripting` - Lua scripting for automation
/// - `settings` - DAW settings
/// - `undo_redo` - Undo/redo system
/// - `browsers` - Content browsers
/// - `notation` - Music notation
/// - `profiling` - Performance profiling
/// - `automation` - Automation lanes
/// - `command_palette` - Command palette UI
///
/// ### Database & I/O
/// - `db::models` - Database model types
/// - `db::repositories` - Database access layer
/// - `io` - I/O operations (decompressor, etc.)
/// - `database` - Database connection management
///
/// ### Commands & Windows
/// - `commands` - Tauri commands (pipeline + daw submodules)
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
pub mod services;

// Pipeline modules (migrated from pipeline/src-tauri)
pub mod commands;
pub mod database;
pub mod error;
pub mod io;
pub mod windows;

// App-specific modules
pub mod shared;

// DAW modules (migrated from daw/src-tauri)
pub mod automation;
pub mod browsers;
pub mod command_palette;
pub mod editors;
pub mod hardware;
pub mod midi_clock;
pub mod midi_io;
pub mod notation;
pub mod profiling;
pub mod scripting;
pub mod sequencer;
pub mod settings;
pub mod undo_redo;

// DAW models (migrated from daw/src-tauri)
pub mod daw_models;

// DAW processing (DSP effects)
pub mod daw;

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

/// Application state shared across all Tauri commands
#[derive(Clone)]
pub struct AppState {
    pub database: Database,
    pub vip3_analytics: std::sync::Arc<services::VIP3AnalyticsService>,
    pub meilisearch: Option<std::sync::Arc<services::MeilisearchClient>>,
}

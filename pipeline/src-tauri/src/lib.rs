   /// MIDI Library Pipeline Processor
   ///
   /// Core library for MIDI file processing, analysis, and management.

pub mod commands;
pub mod core;
pub mod db;
pub mod io;

// Database connection module
pub mod database;

// Error handling module
pub mod error;

// Window management system
pub mod windows;

// Re-export commonly used types
pub use database::Database;
pub use database::window_state::{
    DatabaseWindowState, SearchFilters, SearchResult, PaginationInfo, ViewMode,
    SortField, SortOrder,
};
pub use error::{AppError, AppResult, TauriResult};
pub use db::models::{File, MusicalMetadata};

/// Application state shared across all Tauri commands
#[derive(Clone)]
pub struct AppState {
    pub database: Database,
}

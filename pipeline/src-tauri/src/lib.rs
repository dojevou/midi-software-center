// Clippy lint configuration for Pipeline crate
#![allow(clippy::too_many_arguments)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::ptr_arg)]

/// MIDI Library Pipeline Processor
///
/// Core library for MIDI file processing, analysis, and management.
// Use mimalloc as global allocator for better performance
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

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
pub use database::window_state::{
    DatabaseWindowState, PaginationInfo, SearchFilters, SearchResult, SortField, SortOrder,
    ViewMode,
};
pub use database::Database;
pub use db::models::{File, MusicalMetadata};
pub use error::{AppError, AppResult, TauriResult};

/// Application state shared across all Tauri commands
#[derive(Clone)]
pub struct AppState {
    pub database: Database,
}

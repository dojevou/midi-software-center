//! MIDI Library Pipeline Processor
//!
//! Core library for MIDI file processing, analysis, and management.

pub mod commands;
pub mod core;
pub mod db;
pub mod io;

// Database connection module
pub mod database;

// Error handling module
pub mod error;

use std::sync::Arc;
use tokio::sync::RwLock;

// Re-export commonly used types
pub use database::Database;
pub use error::{AppError, AppResult, TauriResult};
pub use db::models::{File, MusicalMetadata, SearchFilters};

/// Application state shared across all Tauri commands
pub struct AppState {
    pub database: Database,
}

//! Types for track splitting operations

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

use crate::core::splitting::SplitError;

/// Minimum confidence threshold for BPM/key detection results
pub const CONFIDENCE_THRESHOLD: f64 = 0.5;

/// Errors that can occur during split and import operations
#[derive(Error, Debug)]
pub enum SplitCommandError {
    #[error("File not found in database: {0}")]
    FileNotFound(i64),

    #[error("File not found on disk: {0}")]
    FileNotFoundOnDisk(String),

    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to split tracks: {0}")]
    SplitError(#[from] SplitError),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Failed to create output directory: {0}")]
    DirectoryCreationError(String),

    #[error("Transaction failed: {0}")]
    TransactionError(String),
}

// Convert to user-friendly string for Tauri commands
impl From<SplitCommandError> for String {
    fn from(err: SplitCommandError) -> String {
        err.to_string()
    }
}

/// Result of a successful split operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitResult {
    /// IDs of the newly created split files in the database
    pub split_file_ids: Vec<i64>,

    /// Number of tracks that were split
    pub tracks_split: usize,

    /// Directory where split files were written
    pub output_dir: PathBuf,
}

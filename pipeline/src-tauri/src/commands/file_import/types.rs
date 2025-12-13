//! Types for file import operations

use crate::core::analysis::auto_tagger::Tag;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Progress event for real-time UI updates
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ImportProgress {
    pub current: usize,
    pub total: usize,
    pub current_file: String,
    pub rate: f64,
}

/// Summary of import operation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSummary {
    pub total_files: usize,
    pub imported: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
    pub duration_secs: f64,
    pub rate: f64,
}

impl ImportSummary {
    /// Create an empty summary (no files to process)
    pub fn empty() -> Self {
        Self {
            total_files: 0,
            imported: 0,
            skipped: 0,
            errors: vec![],
            duration_secs: 0.0,
            rate: 0.0,
        }
    }

    /// Create summary when all files are duplicates
    pub fn all_duplicates(total: usize, duplicates: usize, elapsed_secs: f64) -> Self {
        Self {
            total_files: total,
            imported: 0,
            skipped: duplicates,
            errors: vec![],
            duration_secs: elapsed_secs,
            rate: 0.0,
        }
    }
}

/// File metadata returned from database
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct FileMetadata {
    pub id: i64,
    pub filename: String,
    pub original_filename: String,
    pub filepath: String,
    #[sqlx(rename = "content_hash_hex")]
    pub content_hash: String,
    pub file_size_bytes: i64,
    pub bpm: Option<f64>,
    pub key_signature: Option<String>,
}

/// Result of the deduplication process
#[derive(Debug)]
pub struct DeduplicationResult {
    pub files_to_process: Vec<PathBuf>,
    pub duplicates_found: usize,
    pub hash_errors: usize,
    pub elapsed_secs: f64,
}

/// Intermediate structure for batch processing
#[derive(Debug, Clone)]
pub struct ProcessedFile {
    pub filename: String,
    pub original_filename: String,
    pub filepath: String,
    pub parent_folder: Option<String>,
    pub content_hash: Vec<u8>,
    pub file_size_bytes: i64,
    pub category: Option<String>,
    pub bpm: Option<f64>,
    pub key_signature: Option<String>,
    pub tags: Vec<Tag>,
    // Filename-based metadata
    pub filename_bpm: Option<f32>,
    pub filename_key: Option<String>,
    pub filename_genres: Vec<String>,
    pub structure_tags: Vec<String>,
    pub track_number: Option<u32>,
    // MIDI text metadata
    pub track_names: Vec<String>,
    pub copyright: Option<String>,
    pub instrument_names_text: Vec<String>,
    pub markers: Vec<String>,
    pub lyrics: Vec<String>,
}

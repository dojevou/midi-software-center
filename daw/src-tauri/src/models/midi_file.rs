//! MIDI file database models
//!
//! Trusty Module: Pure data structures for MIDI file records.
//! Updated to match actual database schema with proper JOINs.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Main file record - matches the actual database schema
/// Uses proper JOINs to musical_metadata and file_categories tables
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MidiFile {
    pub id: i64,

    // File metadata (from files table)
    pub filename: String,
    pub filepath: String,
    pub file_size_bytes: i64,
    #[sqlx(default)]
    pub content_hash: Vec<u8>,

    // Multi-track info
    #[sqlx(default)]
    pub is_multi_track: bool,
    pub parent_file_id: Option<i64>,
    pub track_number: Option<i16>,
    pub total_tracks: Option<i16>,

    // Context from folders
    pub manufacturer: Option<String>,
    pub collection_name: Option<String>,
    #[sqlx(default)]
    pub folder_tags: Vec<String>,
    pub parent_folder: Option<String>,

    // Musical metadata (from musical_metadata table via JOIN)
    pub bpm: Option<f64>,  // numeric(6,2) in DB
    pub key_signature: Option<String>,

    // Time signature (formatted as "4/4" from numerator/denominator)
    pub time_signature: Option<String>,

    // Duration and notes
    pub duration_seconds: Option<f64>,  // numeric(10,3) in DB, can be NULL
    pub total_notes: i32,  // note_count in musical_metadata, defaulted to 0 if NULL

    // Track count from files table
    pub num_tracks: i16,

    // Categories (from file_categories table via subquery)
    pub primary_category: Option<String>,

    // Timestamps
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub analyzed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Lightweight file details for search results
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FileDetails {
    pub id: i64,
    #[serde(rename = "file_name")]
    pub filename: String,
    #[serde(rename = "file_path")]
    pub filepath: String,
    #[serde(rename = "file_size")]
    pub file_size_bytes: i64,
    pub bpm: Option<f64>,
    #[serde(rename = "key")]
    pub key_signature: Option<String>,
    pub time_signature: Option<String>,
    pub duration_seconds: Option<f64>,
    pub total_notes: Option<i32>,
    #[serde(rename = "category")]
    pub primary_category: Option<String>,
    pub parent_folder: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[sqlx(default)]
    pub is_favorite: bool,
    // Additional fields for compatibility with frontend
    #[sqlx(default)]
    #[serde(default)]
    pub tags: Vec<String>,
    pub manufacturer: Option<String>,
    #[serde(rename = "collection")]
    pub collection_name: Option<String>,
    #[serde(default)]
    pub track_count: i16,
    #[serde(default)]
    pub has_notes: bool,
    pub has_drums: Option<bool>,
    #[sqlx(default)]
    #[serde(default, skip_serializing)]
    pub content_hash: Vec<u8>,
}

impl MidiFile {
    /// Helper to format time signature from numerator and denominator
    pub fn format_time_signature(numerator: Option<i16>, denominator: Option<i16>) -> Option<String> {
        match (numerator, denominator) {
            (Some(num), Some(den)) => Some(format!("{}/{}", num, den)),
            _ => None,
        }
    }
}

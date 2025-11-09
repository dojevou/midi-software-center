   /// Database models aligned with actual schema
   ///
   /// These models match the database schema from 001_initial_schema.sql
   /// Database: midi_library on port 5433

use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::types::BigDecimal;
use sqlx::FromRow;
use uuid::Uuid;

// =============================================================================
// FILES TABLE
// =============================================================================

/// File record from database (aligned with schema)
// TODO: Fix BigDecimal serde support - temporarily disabled
#[derive(Debug, Clone, FromRow)]
pub struct File {
    pub id: i64,

    // File identification
    pub filename: String,
    pub filepath: String,
    pub original_filename: String,
    pub content_hash: Vec<u8>,
    pub file_size_bytes: i64,

    // MIDI format
    pub format: Option<i16>,
    pub num_tracks: i16,
    pub ticks_per_quarter_note: Option<i32>,

    // Duration
    pub duration_seconds: Option<BigDecimal>,
    pub duration_ticks: Option<i64>,

    // Multi-track handling
    pub is_multi_track: Option<bool>,
    pub parent_file_id: Option<i64>,
    pub track_number: Option<i16>,
    pub total_tracks: Option<i16>,

    // Extracted context
    pub manufacturer: Option<String>,
    pub collection_name: Option<String>,
    pub folder_tags: Option<Vec<String>>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub analyzed_at: Option<DateTime<Utc>>,

    // Processing
    pub import_batch_id: Option<Uuid>,
}

/// New file for insertion
#[derive(Debug, Clone)]
pub struct NewFile {
    pub filename: String,
    pub filepath: String,
    pub original_filename: String,
    pub content_hash: Vec<u8>,
    pub file_size_bytes: i64,
    pub format: Option<i16>,
    pub num_tracks: i16,
    pub ticks_per_quarter_note: Option<i32>,
    pub duration_seconds: Option<BigDecimal>,
    pub duration_ticks: Option<i64>,
    pub manufacturer: Option<String>,
    pub collection_name: Option<String>,
    pub folder_tags: Option<Vec<String>>,
    pub import_batch_id: Option<Uuid>,
}

// =============================================================================
// MUSICAL_METADATA TABLE
// =============================================================================

/// Musical metadata from database (aligned with schema)
// TODO: Fix BigDecimal serde support - temporarily disabled
#[derive(Debug, Clone, FromRow)]
pub struct MusicalMetadata {
    pub file_id: i64,

    // Tempo
    pub bpm: Option<BigDecimal>,
    pub bpm_confidence: Option<f32>,
    pub has_tempo_changes: Option<bool>,
    pub tempo_changes: Option<serde_json::Value>,

    // Key signature (enum type in database)
    pub key_signature: Option<String>,  // We'll handle the enum as String
    pub key_confidence: Option<f32>,
    pub has_key_changes: Option<bool>,
    pub key_changes: Option<serde_json::Value>,

    // Time signature
    pub time_signature_numerator: Option<i16>,
    pub time_signature_denominator: Option<i16>,
    pub has_time_signature_changes: Option<bool>,
    pub time_signature_changes: Option<serde_json::Value>,

    // Note statistics
    pub total_notes: i32,
    pub unique_pitches: Option<i32>,
    pub pitch_range_min: Option<i16>,
    pub pitch_range_max: Option<i16>,
    pub avg_velocity: Option<BigDecimal>,

    // Density metrics
    pub note_density: Option<BigDecimal>,
    pub polyphony_max: Option<i16>,
    pub polyphony_avg: Option<BigDecimal>,

    // Musical characteristics
    pub is_monophonic: Option<bool>,
    pub is_polyphonic: Option<bool>,
    pub is_percussive: Option<bool>,

    // Chord analysis
    pub has_chords: Option<bool>,
    pub chord_complexity: Option<f32>,

    // Melody analysis
    pub has_melody: Option<bool>,
    pub melodic_range: Option<i16>,

    pub created_at: DateTime<Utc>,
}

/// New musical metadata for insertion
#[derive(Debug, Clone)]
pub struct NewMusicalMetadata {
    pub file_id: i64,
    pub bpm: Option<BigDecimal>,
    pub bpm_confidence: Option<f32>,
    pub key_signature: Option<String>,
    pub key_confidence: Option<f32>,
    pub time_signature_numerator: Option<i16>,
    pub time_signature_denominator: Option<i16>,
    pub total_notes: i32,
    pub unique_pitches: Option<i32>,
    pub pitch_range_min: Option<i16>,
    pub pitch_range_max: Option<i16>,
    pub avg_velocity: Option<BigDecimal>,
    pub note_density: Option<BigDecimal>,
    pub polyphony_max: Option<i16>,
    pub polyphony_avg: Option<BigDecimal>,
    pub is_percussive: Option<bool>,
}

// =============================================================================
// SEARCH & QUERY MODELS
// =============================================================================

/// Search filters from frontend
#[derive(Debug, Clone, Deserialize)]
pub struct SearchFilters {
    pub category: Option<String>,
    pub min_bpm: Option<f64>,
    pub max_bpm: Option<f64>,
    pub key_signatures: Option<Vec<String>>,
    pub min_duration: Option<f64>,
    pub max_duration: Option<f64>,
}

/// Search result combining file and metadata
// TODO: Fix BigDecimal serde support - temporarily disabled
#[derive(Debug, Clone)]
pub struct FileSearchResult {
    pub id: i64,
    pub filename: String,
    pub filepath: String,
    pub bpm: Option<f64>,
    pub key_signature: Option<String>,
    pub duration_seconds: Option<f64>,
    pub total_notes: i32,
    pub category: Option<String>,
}

/// Paginated search results
// TODO: Fix BigDecimal serde support - temporarily disabled
#[derive(Debug, Clone)]
pub struct SearchResults {
    pub files: Vec<FileSearchResult>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

/// Detailed file view with metadata
// TODO: Fix BigDecimal serde support - temporarily disabled
#[derive(Debug, Clone)]
pub struct FileWithMetadata {
    pub file: File,
    pub metadata: Option<MusicalMetadata>,
}

// =============================================================================
// TYPE CONVERSION HELPERS
// =============================================================================

use num_traits::ToPrimitive;

/// Convert BigDecimal to f64
pub fn bigdecimal_to_f64(bd: Option<BigDecimal>) -> Option<f64> {
    bd.and_then(|b| b.to_f64())
}

/// Convert f64 to BigDecimal
pub fn f64_to_bigdecimal(val: Option<f64>) -> Option<BigDecimal> {
    use num_traits::FromPrimitive;
    val.and_then(BigDecimal::from_f64)
}

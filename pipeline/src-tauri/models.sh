cat > src/db/models.rs << 'EOF'
//! Database models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use uuid::Uuid;
use num_traits::cast::{ToPrimitive, FromPrimitive};

/// File record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub id: i64,
    pub original_path: String,
    pub current_path: String,
    pub original_filename: String,
    pub new_filename: String,
    pub content_hash: Vec<u8>,
    pub file_size_bytes: i64,
    pub file_modified: DateTime<Utc>,
    pub is_multi_track: Option<bool>,
    pub parent_file_id: Option<i64>,
    pub track_number: Option<i16>,
    pub total_tracks: Option<i16>,
    pub manufacturer: Option<String>,
    pub collection_name: Option<String>,
    pub folder_tags: Option<Vec<String>>,
    pub category: Option<String>,  // Handle custom type as String
    pub subcategory: Option<String>,
    pub auto_tags: Option<Vec<String>>,
    pub user_tags: Option<Vec<String>>,
    pub analyzed_at: Option<DateTime<Utc>>,
    pub import_batch_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New file insert data
#[derive(Debug, Clone)]
pub struct NewFile {
    pub original_path: String,
    pub current_path: String,
    pub original_filename: String,
    pub new_filename: String,
    pub content_hash: Vec<u8>,
    pub file_size: i64,
    pub file_modified: DateTime<Utc>,
    pub manufacturer: Option<String>,
    pub collection_name: Option<String>,
    pub folder_tags: Option<Vec<String>>,
    pub import_batch_id: Option<Uuid>,
}

/// Musical metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicalMetadata {
    pub file_id: i64,
    pub midi_format: i16,
    pub num_tracks: i16,
    pub ticks_per_quarter_note: i32,
    pub duration_seconds: f64,
    pub total_ticks: i64,
    pub detected_bpm: Option<f64>,
    pub bpm_confidence: Option<f64>,
    pub detected_key: Option<String>,  // Handle custom type as String
    pub key_confidence: Option<f64>,
    pub time_signature_numerator: Option<i16>,
    pub time_signature_denominator: Option<i16>,
    pub total_notes: i32,
    pub unique_pitches: i16,
    pub pitch_range_low: Option<i16>,
    pub pitch_range_high: Option<i16>,
    pub avg_velocity: Option<f64>,
    pub avg_note_duration_ms: Option<f64>,
    pub complexity_score: Option<f64>,
    pub polyphony_max: Option<i16>,
    pub polyphony_avg: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New musical metadata insert data
#[derive(Debug, Clone)]
pub struct NewMusicalMetadata {
    pub file_id: i64,
    pub midi_format: i16,
    pub num_tracks: i16,
    pub ticks_per_quarter_note: i32,
    pub duration_seconds: f64,
    pub total_ticks: i64,
    pub detected_bpm: Option<f64>,
    pub bpm_confidence: Option<f64>,
    pub detected_key: Option<String>,
    pub key_confidence: Option<f64>,
    pub time_signature_numerator: Option<i16>,
    pub time_signature_denominator: Option<i16>,
    pub total_notes: i32,
    pub unique_pitches: i16,
    pub pitch_range_low: Option<i16>,
    pub pitch_range_high: Option<i16>,
    pub avg_velocity: Option<f64>,
    pub avg_note_duration_ms: Option<f64>,
    pub complexity_score: Option<f64>,
    pub polyphony_max: Option<i16>,
    pub polyphony_avg: Option<f64>,
}

// Search-related models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchResult {
    pub id: i64,
    pub file_name: String,
    pub file_extension: String,
    pub detected_bpm: Option<f64>,
    pub category: String,
    pub duration_seconds: Option<f64>,
    pub query: String,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub category: Option<String>,
    pub min_bpm: Option<f64>,
    pub max_bpm: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub files: Vec<FileSearchResult>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDetails {
    pub id: i64,
    pub original_path: String,
    pub current_path: String,
    pub file_name: String,
    pub file_extension: String,
    pub file_size_bytes: i64,
    pub content_hash: String,
    pub category: Option<String>,
    pub imported_at: DateTime<Utc>,
    pub metadata: Option<MusicalMetadata>,
}

// Helper function to convert BigDecimal to f64
pub fn bigdecimal_to_f64(bd: Option<BigDecimal>) -> Option<f64> {
    bd.and_then(|b| b.to_f64())
}

// Helper function to convert f64 to BigDecimal
pub fn f64_to_bigdecimal(val: Option<f64>) -> Option<BigDecimal> {
    val.and_then(|v| BigDecimal::from_f64(v))
}
EOF

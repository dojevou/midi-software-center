//! VIP3 Browser type definitions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

/// VIP3 browser filter state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Vip3Filters {
    /// Selected folder IDs
    #[serde(default)]
    pub folder_ids: Vec<i64>,

    /// Selected instrument IDs (from existing tags)
    #[serde(default)]
    pub instrument_ids: Vec<i32>,

    /// Selected timbre IDs
    #[serde(default)]
    pub timbre_ids: Vec<i16>,

    /// Selected style IDs
    #[serde(default)]
    pub style_ids: Vec<i16>,

    /// Selected articulation IDs
    #[serde(default)]
    pub articulation_ids: Vec<i16>,

    /// Selected BPM range IDs
    #[serde(default)]
    pub bpm_range_ids: Vec<i16>,

    /// Selected key IDs
    #[serde(default)]
    pub key_ids: Vec<i16>,

    /// Selected MIDI channel (1-16)
    pub channel: Option<i16>,

    /// Free-text search query
    pub search_query: Option<String>,

    /// Minimum rating filter (1-5)
    pub min_rating: Option<i16>,

    /// Only show favorites
    #[serde(default)]
    pub favorites_only: bool,

    /// Custom tag IDs
    #[serde(default)]
    pub tag_ids: Vec<i32>,
}

/// Sort configuration for VIP3 browser
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vip3Sort {
    pub field: String, // "filename", "bpm", "duration", "rating", "created_at"
    pub order: String, // "asc" or "desc"
}

impl Default for Vip3Sort {
    fn default() -> Self {
        Self { field: "filename".to_string(), order: "asc".to_string() }
    }
}

/// VIP3 search result item
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Vip3FileResult {
    pub id: i64,
    pub filename: String,
    pub filepath: String,
    pub bpm: Option<f64>,
    pub key_signature: Option<String>,
    pub duration_ms: Option<i32>,
    pub note_count: Option<i32>,
    pub channel: Option<i16>,
    pub rating: Option<i16>,
    pub favorite: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
}

/// Paginated VIP3 search results
#[derive(Debug, Serialize)]
pub struct Vip3SearchResults {
    pub files: Vec<Vip3FileResult>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

/// Category filter option with count
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct FilterOption {
    pub id: i32,
    pub name: String,
    pub file_count: i64,
}

/// All filter counts for VIP3 sidebar
#[derive(Debug, Serialize)]
pub struct Vip3FilterCounts {
    pub timbres: Vec<FilterOption>,
    pub styles: Vec<FilterOption>,
    pub articulations: Vec<FilterOption>,
    pub bpm_ranges: Vec<FilterOption>,
    pub keys: Vec<FilterOption>,
    pub instruments: Vec<FilterOption>,
}

/// Saved search record
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct SavedSearchResponse {
    pub id: i64,
    pub name: String,
    #[sqlx(default)]
    pub description: Option<String>,
    pub filters: Value,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    #[sqlx(default)]
    pub icon: Option<String>,
    #[sqlx(default)]
    pub color: Option<String>,
    #[sqlx(default)]
    pub is_pinned: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
    pub use_count: Option<i32>,
}

/// Data for creating a saved search
#[derive(Debug, Clone, Deserialize)]
pub struct CreateSavedSearchRequest {
    pub name: String,
    pub description: Option<String>,
    pub filters: Vip3Filters,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

/// Collection record
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct CollectionResponse {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_smart: Option<bool>,
    pub file_count: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
    pub modified_at: Option<DateTime<Utc>>,
}

/// Data for creating a collection
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCollectionRequest {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

/// Response for file categories
#[derive(Debug, Serialize)]
pub struct FileCategoriesResponse {
    pub timbres: Vec<FilterOption>,
    pub styles: Vec<FilterOption>,
    pub articulations: Vec<FilterOption>,
}

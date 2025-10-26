//! Search and filter models
//!
//! Trusty Module: Pure data structures for search operations.

use serde::{Deserialize, Serialize};
use super::midi_file::FileDetails;

/**
 * Search filters
 *
 * Filters applied to search queries. All fields are optional.
 */
#[derive(Debug, Deserialize)]
pub struct SearchFilters {
    // BPM range filtering
    pub min_bpm: Option<f32>,
    pub max_bpm: Option<f32>,

    // Key and time signature
    pub key_signature: Option<String>,
    pub time_signature: Option<String>,

    // Category
    pub category: Option<String>,

    // Note count range
    pub min_notes: Option<i32>,
    pub max_notes: Option<i32>,

    // Duration range (seconds)
    pub min_duration: Option<f64>,
    pub max_duration: Option<f64>,

    // Instruments (array match)
    pub instruments: Option<Vec<String>>,

    // Text search
    pub search_text: Option<String>,

    // Sorting and pagination
    pub sort_by: Option<String>,
    pub sort_desc: Option<bool>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

/**
 * Search response
 *
 * Contains matching files and total count for pagination.
 */
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub files: Vec<FileDetails>,
    pub total: i32,
}

/**
 * Autocomplete suggestion
 *
 * Used for search bar autocomplete functionality.
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct Suggestion {
    pub value: String,
}

/**
 * Filter option
 *
 * Represents a single option in a filter dropdown with count.
 */
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct FilterOption {
    pub value: String,
    pub label: String,
    pub count: i64,
}

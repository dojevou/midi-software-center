//! Search query and filter models
//!
//! Provides structures for building complex search queries against
//! the MIDI file database with filtering, sorting, and pagination.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Search filters for querying MIDI files.
///
/// All fields are optional - only specified filters are applied.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilters {
    // === Tempo Filters ===
    /// Minimum BPM (inclusive)
    pub bpm_min: Option<f64>,

    /// Maximum BPM (inclusive)
    pub bpm_max: Option<f64>,

    // === Key Filters ===
    /// Exact key match (e.g., "C", "Am", "F#m")
    pub key: Option<String>,

    /// Multiple allowed keys
    pub keys: Option<Vec<String>>,

    // === Tag Filters ===
    /// Required tags (AND - all must match)
    pub tags: Option<Vec<String>>,

    /// Any tags (OR - at least one must match)
    pub tags_any: Option<Vec<String>>,

    /// Excluded tags (NOT - none can match)
    pub tags_exclude: Option<Vec<String>>,

    // === Instrument Filters ===
    /// Filter by instrument type
    pub instrument: Option<String>,

    /// Multiple instruments (OR)
    pub instruments: Option<Vec<String>>,

    // === Track Filters ===
    /// Minimum track count
    pub track_count_min: Option<i32>,

    /// Maximum track count
    pub track_count_max: Option<i32>,

    // === Duration Filters ===
    /// Minimum duration in seconds
    pub duration_min: Option<f64>,

    /// Maximum duration in seconds
    pub duration_max: Option<f64>,

    // === File Filters ===
    /// Minimum file size in bytes
    pub file_size_min: Option<i64>,

    /// Maximum file size in bytes
    pub file_size_max: Option<i64>,

    /// Filter by manufacturer/source
    pub manufacturer: Option<String>,

    /// Filter by collection name
    pub collection: Option<String>,

    // === Musical Characteristics ===
    /// Only drums/percussion files
    pub is_percussive: Option<bool>,

    /// Only files with chords
    pub has_chords: Option<bool>,

    /// Only files with melody
    pub has_melody: Option<bool>,

    // === Time Signature ===
    /// Time signature numerator (e.g., 4 for 4/4)
    pub time_sig_numerator: Option<i16>,

    /// Time signature denominator (e.g., 4 for 4/4)
    pub time_sig_denominator: Option<i16>,

    // === Analysis ===
    /// Minimum confidence score for analysis
    pub min_confidence: Option<f64>,

    /// Only files that have been analyzed
    pub analyzed_only: Option<bool>,
}

/// Sort direction for search results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    #[default]
    Desc,
}

/// Fields available for sorting.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortField {
    Filename,
    #[default]
    CreatedAt,
    UpdatedAt,
    FileSize,
    Bpm,
    Key,
    Duration,
    TrackCount,
    NoteCount,
    Relevance,
}

/// Complete search query with filters, sorting, and pagination.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Full-text search query
    pub query: Option<String>,

    /// Structured filters
    pub filters: SearchFilters,

    /// Maximum results to return (default: 50)
    pub limit: Option<i64>,

    /// Number of results to skip (for pagination)
    pub offset: Option<i64>,

    /// Field to sort by
    pub sort_by: Option<SortField>,

    /// Sort direction
    pub sort_order: Option<SortOrder>,
}

/// Search result with file and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct SearchResult {
    /// File ID
    pub id: i64,

    /// Filename
    pub filename: String,

    /// Full file path
    pub filepath: String,

    /// Detected BPM
    pub bpm: Option<f64>,

    /// Detected key signature
    pub key_signature: Option<String>,

    /// Auto-generated tags (JSONB)
    #[cfg_attr(feature = "database", sqlx(default))]
    pub tags: Value,

    /// Number of tracks
    pub track_count: i32,

    /// Duration in seconds
    pub duration_seconds: Option<f64>,

    /// Detected instruments (JSONB)
    #[cfg_attr(feature = "database", sqlx(default))]
    pub instruments: Value,

    /// Search relevance score (for full-text search)
    pub relevance_score: Option<f64>,
}

/// Paginated search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    /// Result items
    pub results: Vec<SearchResult>,

    /// Total count (before pagination)
    pub total_count: i64,

    /// Current offset
    pub offset: i64,

    /// Page size limit
    pub limit: i64,

    /// Whether there are more results
    pub has_more: bool,
}

/// Quick search result (minimal data for autocomplete).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct QuickSearchResult {
    pub id: i64,
    pub filename: String,
    pub bpm: Option<f64>,
    pub key_signature: Option<String>,
}

impl SearchFilters {
    /// Create empty filters.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by BPM range.
    #[must_use]
    pub fn with_bpm_range(mut self, min: f64, max: f64) -> Self {
        self.bpm_min = Some(min);
        self.bpm_max = Some(max);
        self
    }

    /// Filter by exact key.
    #[must_use]
    pub fn with_key(mut self, key: &str) -> Self {
        self.key = Some(key.to_string());
        self
    }

    /// Filter by tags (AND).
    #[must_use]
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Filter by instrument.
    #[must_use]
    pub fn with_instrument(mut self, instrument: &str) -> Self {
        self.instrument = Some(instrument.to_string());
        self
    }

    /// Filter by duration range.
    #[must_use]
    pub fn with_duration_range(mut self, min: f64, max: f64) -> Self {
        self.duration_min = Some(min);
        self.duration_max = Some(max);
        self
    }

    /// Filter drums only.
    #[must_use]
    pub fn drums_only(mut self) -> Self {
        self.is_percussive = Some(true);
        self
    }

    /// Filter melodic content only.
    #[must_use]
    pub fn melodic_only(mut self) -> Self {
        self.has_melody = Some(true);
        self.is_percussive = Some(false);
        self
    }

    /// Check if any filters are set.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bpm_min.is_none()
            && self.bpm_max.is_none()
            && self.key.is_none()
            && self.keys.is_none()
            && self.tags.is_none()
            && self.tags_any.is_none()
            && self.instrument.is_none()
            && self.instruments.is_none()
            && self.track_count_min.is_none()
            && self.track_count_max.is_none()
            && self.duration_min.is_none()
            && self.duration_max.is_none()
            && self.file_size_min.is_none()
            && self.file_size_max.is_none()
            && self.manufacturer.is_none()
            && self.collection.is_none()
            && self.is_percussive.is_none()
            && self.has_chords.is_none()
            && self.has_melody.is_none()
    }
}

impl SearchQuery {
    /// Create a new search query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a full-text search query.
    #[must_use]
    pub fn text(query: &str) -> Self {
        Self { query: Some(query.to_string()), ..Default::default() }
    }

    /// Set pagination.
    #[must_use]
    pub fn paginate(mut self, limit: i64, offset: i64) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }

    /// Set sorting.
    #[must_use]
    pub fn sort(mut self, field: SortField, order: SortOrder) -> Self {
        self.sort_by = Some(field);
        self.sort_order = Some(order);
        self
    }

    /// Apply filters.
    #[must_use]
    pub fn with_filters(mut self, filters: SearchFilters) -> Self {
        self.filters = filters;
        self
    }

    /// Get effective limit (default 50).
    #[must_use]
    pub fn effective_limit(&self) -> i64 {
        self.limit.unwrap_or(50).min(1000)
    }

    /// Get effective offset (default 0).
    #[must_use]
    pub fn effective_offset(&self) -> i64 {
        self.offset.unwrap_or(0)
    }
}

impl SearchResults {
    /// Create a new results container.
    #[must_use]
    pub fn new(results: Vec<SearchResult>, total_count: i64, offset: i64, limit: i64) -> Self {
        let has_more = offset + (results.len() as i64) < total_count;
        Self { results, total_count, offset, limit, has_more }
    }

    /// Calculate total pages.
    #[must_use]
    pub fn total_pages(&self) -> i64 {
        if self.limit == 0 {
            0
        } else {
            (self.total_count + self.limit - 1) / self.limit
        }
    }

    /// Calculate current page (1-indexed).
    #[must_use]
    pub fn current_page(&self) -> i64 {
        if self.limit == 0 {
            0
        } else {
            (self.offset / self.limit) + 1
        }
    }
}

impl SearchResult {
    /// Get tags as a Vec of strings.
    pub fn tags_as_vec(&self) -> Vec<String> {
        match &self.tags {
            Value::Array(arr) => arr.iter().filter_map(|v| v.as_str().map(String::from)).collect(),
            _ => Vec::new(),
        }
    }

    /// Get instruments as a Vec of strings.
    pub fn instruments_as_vec(&self) -> Vec<String> {
        match &self.instruments {
            Value::Array(arr) => arr.iter().filter_map(|v| v.as_str().map(String::from)).collect(),
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_filters() {
        let filters = SearchFilters::new()
            .with_bpm_range(118.0, 122.0)
            .with_key("Am")
            .with_tags(vec!["drums".to_string(), "rock".to_string()])
            .drums_only();

        assert_eq!(filters.bpm_min, Some(118.0));
        assert_eq!(filters.bpm_max, Some(122.0));
        assert_eq!(filters.key, Some("Am".to_string()));
        assert_eq!(filters.is_percussive, Some(true));
        assert!(!filters.is_empty());
    }

    #[test]
    fn test_empty_filters() {
        let filters = SearchFilters::new();
        assert!(filters.is_empty());
    }

    #[test]
    fn test_search_query() {
        let query = SearchQuery::text("jazz piano")
            .paginate(20, 40)
            .sort(SortField::Bpm, SortOrder::Asc)
            .with_filters(SearchFilters::new().with_bpm_range(100.0, 140.0));

        assert_eq!(query.query, Some("jazz piano".to_string()));
        assert_eq!(query.effective_limit(), 20);
        assert_eq!(query.effective_offset(), 40);
        assert_eq!(query.sort_by, Some(SortField::Bpm));
    }

    #[test]
    fn test_search_results() {
        let results = SearchResults::new(vec![], 100, 40, 20);
        assert_eq!(results.total_pages(), 5);
        assert_eq!(results.current_page(), 3);
        assert!(results.has_more);
    }

    #[test]
    fn test_tags_extraction() {
        let result = SearchResult {
            id: 1,
            filename: "test.mid".to_string(),
            filepath: "/path/test.mid".to_string(),
            bpm: Some(120.0),
            key_signature: Some("Am".to_string()),
            tags: serde_json::json!(["drums", "rock", "loop"]),
            track_count: 4,
            duration_seconds: Some(30.0),
            instruments: serde_json::json!(["piano", "bass"]),
            relevance_score: Some(0.95),
        };

        let tags = result.tags_as_vec();
        assert_eq!(tags.len(), 3);
        assert!(tags.contains(&"drums".to_string()));

        let instruments = result.instruments_as_vec();
        assert_eq!(instruments.len(), 2);
        assert!(instruments.contains(&"piano".to_string()));
    }
}

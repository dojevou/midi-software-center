//! Saved Search and Recent Search database models.
//!
//! Allows users to save and recall complex filter configurations:
//! - Saved searches store filter state as JSON
//! - Recent searches track search history for quick access

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Saved search record for storing filter configurations.
///
/// Users can save complex filter combinations for quick recall.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct SavedSearch {
    /// Primary key (BIGSERIAL)
    pub id: i64,

    /// User-defined name for this search
    pub name: String,

    /// Filter configuration as JSON
    #[cfg_attr(feature = "database", sqlx(default))]
    pub filters: Value,

    /// Sort field (e.g., "filename", "bpm", "created_at")
    pub sort_by: Option<String>,

    /// Sort order ("asc" or "desc")
    pub sort_order: Option<String>,

    /// When this search was created
    pub created_at: Option<DateTime<Utc>>,
}

/// Data required to create a saved search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSavedSearch {
    pub name: String,
    pub filters: Value,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

/// Optional fields for updating a saved search.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateSavedSearch {
    pub name: Option<String>,
    pub filters: Option<Value>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

/// Recent search record for tracking search history.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct RecentSearch {
    /// Primary key (BIGSERIAL)
    pub id: i64,

    /// The search query text
    pub query: String,

    /// Optional filter state as JSON
    #[cfg_attr(feature = "database", sqlx(default))]
    pub filters: Option<Value>,

    /// When this search was performed
    pub searched_at: Option<DateTime<Utc>>,
}

/// Data required to create a recent search entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRecentSearch {
    pub query: String,
    pub filters: Option<Value>,
}

/// VIP3-style browser filter state.
///
/// This structure matches the filters stored in saved_searches.filters JSONB.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BrowserFilters {
    /// Selected folder IDs
    #[serde(default)]
    pub folder_ids: Vec<i64>,

    /// Selected instrument IDs
    #[serde(default)]
    pub instrument_ids: Vec<i16>,

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

    /// Minimum rating filter (0-5)
    pub min_rating: Option<i16>,

    /// Only show favorites
    #[serde(default)]
    pub favorites_only: bool,

    /// Custom tag IDs
    #[serde(default)]
    pub tag_ids: Vec<i32>,
}

impl CreateSavedSearch {
    /// Create a new saved search with name and filters.
    #[must_use]
    pub fn new(name: &str, filters: BrowserFilters) -> Self {
        Self {
            name: name.to_string(),
            filters: serde_json::to_value(filters).unwrap_or(Value::Object(Default::default())),
            sort_by: None,
            sort_order: None,
        }
    }

    /// Set sort configuration.
    #[must_use]
    pub fn with_sort(mut self, field: &str, order: &str) -> Self {
        self.sort_by = Some(field.to_string());
        self.sort_order = Some(order.to_string());
        self
    }
}

impl SavedSearch {
    /// Parse the filters JSON into a BrowserFilters struct.
    #[must_use]
    pub fn parse_filters(&self) -> Option<BrowserFilters> {
        serde_json::from_value(self.filters.clone()).ok()
    }

    /// Check if this search has any active filters.
    #[must_use]
    pub fn has_filters(&self) -> bool {
        if let Some(filters) = self.parse_filters() {
            !filters.folder_ids.is_empty()
                || !filters.instrument_ids.is_empty()
                || !filters.timbre_ids.is_empty()
                || !filters.style_ids.is_empty()
                || !filters.articulation_ids.is_empty()
                || !filters.bpm_range_ids.is_empty()
                || !filters.key_ids.is_empty()
                || filters.channel.is_some()
                || filters.search_query.is_some()
                || filters.min_rating.is_some()
                || filters.favorites_only
                || !filters.tag_ids.is_empty()
        } else {
            false
        }
    }

    /// Get a summary of active filters for display.
    #[must_use]
    pub fn filter_summary(&self) -> String {
        if let Some(filters) = self.parse_filters() {
            let mut parts = Vec::new();

            if !filters.instrument_ids.is_empty() {
                parts.push(format!("{} instruments", filters.instrument_ids.len()));
            }
            if !filters.timbre_ids.is_empty() {
                parts.push(format!("{} timbres", filters.timbre_ids.len()));
            }
            if !filters.style_ids.is_empty() {
                parts.push(format!("{} styles", filters.style_ids.len()));
            }
            if !filters.articulation_ids.is_empty() {
                parts.push(format!("{} articulations", filters.articulation_ids.len()));
            }
            if !filters.bpm_range_ids.is_empty() {
                parts.push(format!("{} BPM ranges", filters.bpm_range_ids.len()));
            }
            if !filters.key_ids.is_empty() {
                parts.push(format!("{} keys", filters.key_ids.len()));
            }
            if filters.favorites_only {
                parts.push("favorites".to_string());
            }
            if let Some(rating) = filters.min_rating {
                parts.push(format!("{}+ stars", rating));
            }

            if parts.is_empty() {
                "No filters".to_string()
            } else {
                parts.join(", ")
            }
        } else {
            "Invalid filters".to_string()
        }
    }
}

impl BrowserFilters {
    /// Check if all filters are empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.folder_ids.is_empty()
            && self.instrument_ids.is_empty()
            && self.timbre_ids.is_empty()
            && self.style_ids.is_empty()
            && self.articulation_ids.is_empty()
            && self.bpm_range_ids.is_empty()
            && self.key_ids.is_empty()
            && self.channel.is_none()
            && self.search_query.is_none()
            && self.min_rating.is_none()
            && !self.favorites_only
            && self.tag_ids.is_empty()
    }

    /// Count the number of active filter categories.
    #[must_use]
    pub fn active_filter_count(&self) -> usize {
        let mut count = 0;
        if !self.folder_ids.is_empty() { count += 1; }
        if !self.instrument_ids.is_empty() { count += 1; }
        if !self.timbre_ids.is_empty() { count += 1; }
        if !self.style_ids.is_empty() { count += 1; }
        if !self.articulation_ids.is_empty() { count += 1; }
        if !self.bpm_range_ids.is_empty() { count += 1; }
        if !self.key_ids.is_empty() { count += 1; }
        if self.channel.is_some() { count += 1; }
        if self.search_query.is_some() { count += 1; }
        if self.min_rating.is_some() { count += 1; }
        if self.favorites_only { count += 1; }
        if !self.tag_ids.is_empty() { count += 1; }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_saved_search() {
        let filters = BrowserFilters {
            instrument_ids: vec![1, 2],
            bpm_range_ids: vec![4],
            favorites_only: true,
            ..Default::default()
        };

        let search = CreateSavedSearch::new("My Drums", filters)
            .with_sort("bpm", "asc");

        assert_eq!(search.name, "My Drums");
        assert_eq!(search.sort_by, Some("bpm".to_string()));
        assert_eq!(search.sort_order, Some("asc".to_string()));
    }

    #[test]
    fn test_browser_filters_empty() {
        let filters = BrowserFilters::default();
        assert!(filters.is_empty());
        assert_eq!(filters.active_filter_count(), 0);
    }

    #[test]
    fn test_browser_filters_active() {
        let filters = BrowserFilters {
            instrument_ids: vec![1, 2],
            timbre_ids: vec![3],
            favorites_only: true,
            ..Default::default()
        };

        assert!(!filters.is_empty());
        assert_eq!(filters.active_filter_count(), 3);
    }

    #[test]
    fn test_saved_search_parse_filters() {
        let filters = BrowserFilters {
            instrument_ids: vec![1],
            bpm_range_ids: vec![4, 5],
            ..Default::default()
        };

        let search = SavedSearch {
            id: 1,
            name: "Test".to_string(),
            filters: serde_json::to_value(&filters).unwrap(),
            sort_by: None,
            sort_order: None,
            created_at: None,
        };

        let parsed = search.parse_filters().unwrap();
        assert_eq!(parsed.instrument_ids, vec![1]);
        assert_eq!(parsed.bpm_range_ids, vec![4, 5]);
        assert!(search.has_filters());
    }

    #[test]
    fn test_filter_summary() {
        let filters = BrowserFilters {
            instrument_ids: vec![1, 2, 3],
            timbre_ids: vec![1],
            favorites_only: true,
            min_rating: Some(4),
            ..Default::default()
        };

        let search = SavedSearch {
            id: 1,
            name: "Test".to_string(),
            filters: serde_json::to_value(&filters).unwrap(),
            sort_by: None,
            sort_order: None,
            created_at: None,
        };

        let summary = search.filter_summary();
        assert!(summary.contains("3 instruments"));
        assert!(summary.contains("1 timbres"));
        assert!(summary.contains("favorites"));
        assert!(summary.contains("4+ stars"));
    }
}

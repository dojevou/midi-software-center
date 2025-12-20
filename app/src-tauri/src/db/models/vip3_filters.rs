use serde::{Deserialize, Serialize};

/// Filter selections for VIP3 browser search
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Vip3Filters {
    /// Selected folder IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder_ids: Option<Vec<i64>>,

    /// Selected instrument tag IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instrument_ids: Option<Vec<i64>>,

    /// Selected timbre category IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timbre_ids: Option<Vec<i32>>,

    /// Selected style category IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style_ids: Option<Vec<i32>>,

    /// Selected articulation category IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub articulation_ids: Option<Vec<i32>>,

    /// Selected BPM range IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bpm_range_ids: Option<Vec<i32>>,

    /// Selected key signature IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_ids: Option<Vec<i32>>,

    /// Selected MIDI channel (1-16)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<u8>,

    /// Free-text search query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_query: Option<String>,

    /// Filter to favorites only
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favorites_only: Option<bool>,

    /// Selected tag IDs (general tags, not instruments)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_ids: Option<Vec<i32>>,

    /// Minimum rating (1-5)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_rating: Option<i32>,

    /// Result limit (pagination)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,

    /// Result offset (pagination)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
}

impl Vip3Filters {
    /// Check if any filter is active
    pub fn has_any_filter(&self) -> bool {
        self.folder_ids.is_some()
            || self.instrument_ids.is_some()
            || self.timbre_ids.is_some()
            || self.style_ids.is_some()
            || self.articulation_ids.is_some()
            || self.bpm_range_ids.is_some()
            || self.key_ids.is_some()
            || self.channel.is_some()
            || self.search_query.is_some()
            || self.favorites_only.is_some()
            || self.tag_ids.is_some()
            || self.min_rating.is_some()
    }

    /// Get effective limit (default 100, max 500)
    pub fn effective_limit(&self) -> i32 {
        self.limit.unwrap_or(100).min(500)
    }

    /// Get effective offset (default 0)
    pub fn effective_offset(&self) -> i32 {
        self.offset.unwrap_or(0).max(0)
    }
}

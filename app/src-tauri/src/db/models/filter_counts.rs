use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Filter counts for VIP3 browser
/// Each HashMap maps filter value ID to count of matching files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCounts {
    /// folder_id → count of files in that folder
    pub folder_counts: HashMap<i64, usize>,

    /// tag_id → count of files with that instrument tag
    pub instrument_counts: HashMap<i64, usize>,

    /// timbre_id → count of files with that timbre
    pub timbre_counts: HashMap<i32, usize>,

    /// style_id → count of files with that style
    pub style_counts: HashMap<i32, usize>,

    /// articulation_id → count of files with that articulation
    pub articulation_counts: HashMap<i32, usize>,

    /// bpm_range_id → count of files in that BPM range
    pub bpm_range_counts: HashMap<i32, usize>,

    /// key_id → count of files in that key signature
    pub key_counts: HashMap<i32, usize>,

    /// channel → count of files using that MIDI channel
    pub channel_counts: HashMap<u8, usize>,

    /// Total number of files matching current filters
    pub total_matches: usize,
}

impl FilterCounts {
    pub fn new() -> Self {
        Self {
            folder_counts: HashMap::new(),
            instrument_counts: HashMap::new(),
            timbre_counts: HashMap::new(),
            style_counts: HashMap::new(),
            articulation_counts: HashMap::new(),
            bpm_range_counts: HashMap::new(),
            key_counts: HashMap::new(),
            channel_counts: HashMap::new(),
            total_matches: 0,
        }
    }
}

impl Default for FilterCounts {
    fn default() -> Self {
        Self::new()
    }
}

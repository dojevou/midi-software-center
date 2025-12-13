//! BPM Range database models for VIP3 filtering.
//!
//! BPM ranges provide quick tempo filtering:
//! - 0-60, 60-80, 80-100, 100-120, 120-140, 140-160, 160-180, 180+
//! - Pre-populated with 8 standard ranges.

use serde::{Deserialize, Serialize};

/// BPM range record for tempo-based filtering.
///
/// Used in VIP3-style filtering to quickly filter files by tempo range.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct BpmRange {
    /// Primary key (SMALLSERIAL)
    pub id: i16,

    /// Display label (e.g., "80-100", "120-140", "180+")
    pub label: String,

    /// Minimum BPM (inclusive)
    pub min_bpm: i16,

    /// Maximum BPM (exclusive, except for last range)
    pub max_bpm: i16,

    /// Display order in UI
    pub sort_order: Option<i16>,
}

/// BPM range with file count for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct BpmRangeWithCount {
    pub id: i16,
    pub label: String,
    pub min_bpm: i16,
    pub max_bpm: i16,
    pub sort_order: Option<i16>,
    pub file_count: i64,
}

impl BpmRange {
    /// Check if a BPM value falls within this range.
    #[must_use]
    pub fn contains(&self, bpm: f64) -> bool {
        let bpm_int = bpm as i16;
        bpm_int >= self.min_bpm && bpm_int < self.max_bpm
    }

    /// Get the midpoint BPM of this range.
    #[must_use]
    pub fn midpoint(&self) -> i16 {
        (self.min_bpm + self.max_bpm) / 2
    }

    /// Check if this is the "slow" range (0-60).
    #[must_use]
    pub fn is_slow(&self) -> bool {
        self.max_bpm <= 60
    }

    /// Check if this is a "fast" range (160+).
    #[must_use]
    pub fn is_fast(&self) -> bool {
        self.min_bpm >= 160
    }

    /// Get a tempo descriptor for the range.
    #[must_use]
    pub fn tempo_descriptor(&self) -> &'static str {
        match self.min_bpm {
            0..=59 => "Very Slow",
            60..=79 => "Slow",
            80..=99 => "Moderate",
            100..=119 => "Medium",
            120..=139 => "Upbeat",
            140..=159 => "Fast",
            160..=179 => "Very Fast",
            _ => "Extreme",
        }
    }
}

/// Pre-defined BPM ranges.
pub const SYSTEM_BPM_RANGES: &[(&str, i16, i16, i16)] = &[
    ("0-60", 0, 60, 1),
    ("60-80", 60, 80, 2),
    ("80-100", 80, 100, 3),
    ("100-120", 100, 120, 4),
    ("120-140", 120, 140, 5),
    ("140-160", 140, 160, 6),
    ("160-180", 160, 180, 7),
    ("180+", 180, 999, 8),
];

/// Find the appropriate BPM range ID for a given BPM value.
#[must_use]
pub fn bpm_to_range_index(bpm: f64) -> Option<usize> {
    let bpm_int = bpm as i16;
    SYSTEM_BPM_RANGES
        .iter()
        .position(|(_, min, max, _)| bpm_int >= *min && bpm_int < *max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bpm_range_contains() {
        let range = BpmRange {
            id: 4,
            label: "100-120".to_string(),
            min_bpm: 100,
            max_bpm: 120,
            sort_order: Some(4),
        };

        assert!(range.contains(100.0));
        assert!(range.contains(110.5));
        assert!(range.contains(119.9));
        assert!(!range.contains(120.0));
        assert!(!range.contains(99.9));
    }

    #[test]
    fn test_bpm_range_midpoint() {
        let range = BpmRange {
            id: 4,
            label: "100-120".to_string(),
            min_bpm: 100,
            max_bpm: 120,
            sort_order: Some(4),
        };

        assert_eq!(range.midpoint(), 110);
    }

    #[test]
    fn test_bpm_range_descriptors() {
        let slow = BpmRange {
            id: 1,
            label: "0-60".to_string(),
            min_bpm: 0,
            max_bpm: 60,
            sort_order: Some(1),
        };
        assert!(slow.is_slow());
        assert!(!slow.is_fast());
        assert_eq!(slow.tempo_descriptor(), "Very Slow");

        let fast = BpmRange {
            id: 7,
            label: "160-180".to_string(),
            min_bpm: 160,
            max_bpm: 180,
            sort_order: Some(7),
        };
        assert!(!fast.is_slow());
        assert!(fast.is_fast());
        assert_eq!(fast.tempo_descriptor(), "Very Fast");
    }

    #[test]
    fn test_bpm_to_range_index() {
        assert_eq!(bpm_to_range_index(50.0), Some(0)); // 0-60
        assert_eq!(bpm_to_range_index(75.0), Some(1)); // 60-80
        assert_eq!(bpm_to_range_index(120.0), Some(4)); // 120-140
        assert_eq!(bpm_to_range_index(200.0), Some(7)); // 180+
    }

    #[test]
    fn test_system_bpm_ranges_count() {
        assert_eq!(SYSTEM_BPM_RANGES.len(), 8);
    }
}

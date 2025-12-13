//! Timbre (sound character) database models for VIP3 filtering.
//!
//! Timbres describe the sonic character of MIDI files:
//! - Aggressive, Airy, Bright, Clean, Dark, Dirty, etc.
//! - Pre-populated with 21 system timbres, users can add custom ones.

use serde::{Deserialize, Serialize};

/// Timbre record representing a sound character category.
///
/// Used in VIP3-style filtering to categorize MIDI files by sonic quality.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct Timbre {
    /// Primary key (SMALLSERIAL)
    pub id: i16,

    /// Timbre name (unique, e.g., "Warm", "Bright", "Dark")
    pub name: String,

    /// Display order in UI
    pub sort_order: Option<i16>,

    /// Whether this is a system-defined timbre (cannot be deleted)
    pub is_system: bool,

    /// Cached count of files using this timbre
    pub file_count: Option<i32>,
}

/// Data required to create a new timbre.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTimbre {
    pub name: String,
    pub sort_order: Option<i16>,
    pub is_system: bool,
}

/// Optional fields for updating a timbre.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateTimbre {
    pub name: Option<String>,
    pub sort_order: Option<i16>,
}

/// Timbre with usage statistics for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct TimbreWithCount {
    pub id: i16,
    pub name: String,
    pub sort_order: Option<i16>,
    pub is_system: bool,
    pub file_count: i64,
}

/// File-timbre relationship (many-to-many).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct FileTimbre {
    pub midi_file_id: i64,
    pub timbre_id: i16,
}

impl CreateTimbre {
    /// Create a new custom timbre.
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), sort_order: None, is_system: false }
    }

    /// Create a system timbre with sort order.
    #[must_use]
    pub fn system(name: &str, sort_order: i16) -> Self {
        Self { name: name.to_string(), sort_order: Some(sort_order), is_system: true }
    }

    /// Set sort order.
    #[must_use]
    pub fn with_sort_order(mut self, order: i16) -> Self {
        self.sort_order = Some(order);
        self
    }
}

impl Timbre {
    /// Check if this timbre can be deleted (non-system only).
    #[must_use]
    pub fn can_delete(&self) -> bool {
        !self.is_system
    }

    /// Check if this timbre has any files.
    #[must_use]
    pub fn has_files(&self) -> bool {
        self.file_count.is_some_and(|c| c > 0)
    }
}

/// Pre-defined system timbres.
pub const SYSTEM_TIMBRES: &[(&str, i16)] = &[
    ("Aggressive", 1),
    ("Airy", 2),
    ("Bright", 3),
    ("Clean", 4),
    ("Dark", 5),
    ("Dirty", 6),
    ("Distorted", 7),
    ("Fat", 8),
    ("Gritty", 9),
    ("Hard", 10),
    ("Metallic", 11),
    ("Muted", 12),
    ("Punchy", 13),
    ("Soft", 14),
    ("Thin", 15),
    ("Warm", 16),
    ("Wide", 17),
    ("Analog", 18),
    ("Digital", 19),
    ("Organic", 20),
    ("Synthetic", 21),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_timbre() {
        let timbre = CreateTimbre::new("Custom");
        assert_eq!(timbre.name, "Custom");
        assert!(!timbre.is_system);
        assert!(timbre.sort_order.is_none());
    }

    #[test]
    fn test_create_system_timbre() {
        let timbre = CreateTimbre::system("Warm", 16);
        assert_eq!(timbre.name, "Warm");
        assert!(timbre.is_system);
        assert_eq!(timbre.sort_order, Some(16));
    }

    #[test]
    fn test_timbre_can_delete() {
        let system_timbre = Timbre {
            id: 1,
            name: "Warm".to_string(),
            sort_order: Some(16),
            is_system: true,
            file_count: Some(100),
        };
        assert!(!system_timbre.can_delete());

        let custom_timbre = Timbre {
            id: 100,
            name: "My Timbre".to_string(),
            sort_order: None,
            is_system: false,
            file_count: Some(0),
        };
        assert!(custom_timbre.can_delete());
    }

    #[test]
    fn test_system_timbres_count() {
        assert_eq!(SYSTEM_TIMBRES.len(), 21);
    }
}

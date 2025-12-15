//! Articulation database models for VIP3 filtering.
//!
//! Articulations describe how notes are played or the pattern type:
//! - Arpeggio, Chord, Fill, Loop, Melody, One-Shot, Phrase, Riff, etc.
//! - Pre-populated with 20 system articulations, users can add custom ones.

use serde::{Deserialize, Serialize};

/// Articulation record representing a playing style or pattern type.
///
/// Used in VIP3-style filtering to categorize MIDI files by articulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct Articulation {
    /// Primary key (SMALLSERIAL)
    pub id: i16,

    /// Articulation name (unique, e.g., "Loop", "One-Shot", "Arpeggio")
    pub name: String,

    /// Display order in UI
    pub sort_order: Option<i16>,

    /// Whether this is a system-defined articulation (cannot be deleted)
    pub is_system: bool,

    /// Cached count of files using this articulation
    pub file_count: Option<i32>,
}

/// Data required to create a new articulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateArticulation {
    pub name: String,
    pub sort_order: Option<i16>,
    pub is_system: bool,
}

/// Optional fields for updating an articulation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateArticulation {
    pub name: Option<String>,
    pub sort_order: Option<i16>,
}

/// Articulation with usage statistics for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct ArticulationWithCount {
    pub id: i16,
    pub name: String,
    pub sort_order: Option<i16>,
    pub is_system: bool,
    pub file_count: i64,
}

/// File-articulation relationship (many-to-many).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct FileArticulation {
    pub midi_file_id: i64,
    pub articulation_id: i16,
}

impl CreateArticulation {
    /// Create a new custom articulation.
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), sort_order: None, is_system: false }
    }

    /// Create a system articulation with sort order.
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

impl Articulation {
    /// Check if this articulation can be deleted (non-system only).
    #[must_use]
    pub fn can_delete(&self) -> bool {
        !self.is_system
    }

    /// Check if this articulation has any files.
    #[must_use]
    pub fn has_files(&self) -> bool {
        self.file_count.is_some_and(|c| c > 0)
    }

    /// Check if this is a loop-type articulation.
    #[must_use]
    pub fn is_loop_type(&self) -> bool {
        matches!(
            self.name.to_lowercase().as_str(),
            "loop" | "groove" | "pattern" | "sequence"
        )
    }

    /// Check if this is a one-shot type articulation.
    #[must_use]
    pub fn is_oneshot_type(&self) -> bool {
        matches!(
            self.name.to_lowercase().as_str(),
            "one-shot" | "stab" | "hit"
        )
    }
}

/// Pre-defined system articulations.
pub const SYSTEM_ARTICULATIONS: &[(&str, i16)] = &[
    ("Arpeggio", 1),
    ("Chord", 2),
    ("Fill", 3),
    ("Loop", 4),
    ("Melody", 5),
    ("One-Shot", 6),
    ("Phrase", 7),
    ("Riff", 8),
    ("Stab", 9),
    ("Sustain", 10),
    ("Sequence", 11),
    ("Pattern", 12),
    ("Groove", 13),
    ("Break", 14),
    ("Intro", 15),
    ("Outro", 16),
    ("Verse", 17),
    ("Chorus", 18),
    ("Bridge", 19),
    ("Drop", 20),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_articulation() {
        let articulation = CreateArticulation::new("Custom");
        assert_eq!(articulation.name, "Custom");
        assert!(!articulation.is_system);
        assert!(articulation.sort_order.is_none());
    }

    #[test]
    fn test_create_system_articulation() {
        let articulation = CreateArticulation::system("Loop", 4);
        assert_eq!(articulation.name, "Loop");
        assert!(articulation.is_system);
        assert_eq!(articulation.sort_order, Some(4));
    }

    #[test]
    fn test_articulation_can_delete() {
        let system = Articulation {
            id: 1,
            name: "Loop".to_string(),
            sort_order: Some(4),
            is_system: true,
            file_count: Some(1000),
        };
        assert!(!system.can_delete());

        let custom = Articulation {
            id: 100,
            name: "My Pattern".to_string(),
            sort_order: None,
            is_system: false,
            file_count: Some(0),
        };
        assert!(custom.can_delete());
    }

    #[test]
    fn test_articulation_type_checks() {
        let loop_art = Articulation {
            id: 4,
            name: "Loop".to_string(),
            sort_order: Some(4),
            is_system: true,
            file_count: None,
        };
        assert!(loop_art.is_loop_type());
        assert!(!loop_art.is_oneshot_type());

        let oneshot = Articulation {
            id: 6,
            name: "One-Shot".to_string(),
            sort_order: Some(6),
            is_system: true,
            file_count: None,
        };
        assert!(!oneshot.is_loop_type());
        assert!(oneshot.is_oneshot_type());
    }

    #[test]
    fn test_system_articulations_count() {
        assert_eq!(SYSTEM_ARTICULATIONS.len(), 20);
    }
}

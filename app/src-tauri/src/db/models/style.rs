//! Style/Genre database models for VIP3 filtering.
//!
//! Styles represent musical genres and categories:
//! - Ambient, Cinematic, Dance, EDM, Funk, Hip-Hop, House, Jazz, etc.
//! - Pre-populated with 24 system styles, users can add custom ones.

use serde::{Deserialize, Serialize};

/// Style record representing a musical genre or category.
///
/// Used in VIP3-style filtering to categorize MIDI files by genre.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct Style {
    /// Primary key (SMALLSERIAL)
    pub id: i16,

    /// Style name (unique, e.g., "Hip-Hop", "Jazz", "Ambient")
    pub name: String,

    /// Display order in UI
    pub sort_order: Option<i16>,

    /// Whether this is a system-defined style (cannot be deleted)
    pub is_system: bool,

    /// Cached count of files using this style
    pub file_count: Option<i32>,
}

/// Data required to create a new style.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStyle {
    pub name: String,
    pub sort_order: Option<i16>,
    pub is_system: bool,
}

/// Optional fields for updating a style.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateStyle {
    pub name: Option<String>,
    pub sort_order: Option<i16>,
}

/// Style with usage statistics for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct StyleWithCount {
    pub id: i16,
    pub name: String,
    pub sort_order: Option<i16>,
    pub is_system: bool,
    pub file_count: i64,
}

/// File-style relationship (many-to-many).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct FileStyle {
    pub midi_file_id: i64,
    pub style_id: i16,
}

impl CreateStyle {
    /// Create a new custom style.
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), sort_order: None, is_system: false }
    }

    /// Create a system style with sort order.
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

impl Style {
    /// Check if this style can be deleted (non-system only).
    #[must_use]
    pub fn can_delete(&self) -> bool {
        !self.is_system
    }

    /// Check if this style has any files.
    #[must_use]
    pub fn has_files(&self) -> bool {
        self.file_count.is_some_and(|c| c > 0)
    }
}

/// Pre-defined system styles.
pub const SYSTEM_STYLES: &[(&str, i16)] = &[
    ("Ambient", 1),
    ("Cinematic", 2),
    ("Dance", 3),
    ("EDM", 4),
    ("Funk", 5),
    ("Hip-Hop", 6),
    ("House", 7),
    ("Jazz", 8),
    ("Latin", 9),
    ("Lo-Fi", 10),
    ("Metal", 11),
    ("Pop", 12),
    ("R&B", 13),
    ("Reggae", 14),
    ("Rock", 15),
    ("Soul", 16),
    ("Techno", 17),
    ("Trap", 18),
    ("Orchestral", 19),
    ("Electronic", 20),
    ("Acoustic", 21),
    ("World", 22),
    ("Experimental", 23),
    ("Chillout", 24),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_style() {
        let style = CreateStyle::new("Custom Genre");
        assert_eq!(style.name, "Custom Genre");
        assert!(!style.is_system);
        assert!(style.sort_order.is_none());
    }

    #[test]
    fn test_create_system_style() {
        let style = CreateStyle::system("Hip-Hop", 6);
        assert_eq!(style.name, "Hip-Hop");
        assert!(style.is_system);
        assert_eq!(style.sort_order, Some(6));
    }

    #[test]
    fn test_style_can_delete() {
        let system_style = Style {
            id: 1,
            name: "Jazz".to_string(),
            sort_order: Some(8),
            is_system: true,
            file_count: Some(500),
        };
        assert!(!system_style.can_delete());

        let custom_style = Style {
            id: 100,
            name: "My Genre".to_string(),
            sort_order: None,
            is_system: false,
            file_count: Some(0),
        };
        assert!(custom_style.can_delete());
    }

    #[test]
    fn test_system_styles_count() {
        assert_eq!(SYSTEM_STYLES.len(), 24);
    }
}

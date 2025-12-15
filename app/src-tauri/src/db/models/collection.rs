//! Collection/Playlist database models for user organization.
//!
//! Collections allow users to organize MIDI files into playlists:
//! - User-created collections with custom names, descriptions, icons, and colors
//! - Files can belong to multiple collections
//! - Files within a collection can be custom-ordered

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Collection record for organizing MIDI files into playlists.
///
/// Similar to playlists in music apps or folders in file browsers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct Collection {
    /// Primary key (BIGSERIAL)
    pub id: i64,

    /// Collection name
    pub name: String,

    /// Optional description
    pub description: Option<String>,

    /// Optional icon name/emoji (e.g., "🎹", "🥁", "folder")
    pub icon: Option<String>,

    /// Optional hex color (e.g., "#FF5733")
    pub color: Option<String>,

    /// When this collection was created
    pub created_at: Option<DateTime<Utc>>,

    /// When this collection was last modified
    pub modified_at: Option<DateTime<Utc>>,
}

/// Data required to create a new collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCollection {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

/// Optional fields for updating a collection.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateCollection {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

/// Collection with file count for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct CollectionWithCount {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub file_count: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub modified_at: Option<DateTime<Utc>>,
}

/// Collection-file relationship (many-to-many).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct CollectionFile {
    pub collection_id: i64,
    pub midi_file_id: i64,
    pub sort_order: Option<i32>,
    pub added_at: Option<DateTime<Utc>>,
}

/// Data for adding a file to a collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddToCollection {
    pub collection_id: i64,
    pub midi_file_id: i64,
    pub sort_order: Option<i32>,
}

impl CreateCollection {
    /// Create a new collection with just a name.
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), description: None, icon: None, color: None }
    }

    /// Set description.
    #[must_use]
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Set icon.
    #[must_use]
    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    /// Set color.
    #[must_use]
    pub fn with_color(mut self, color: &str) -> Self {
        self.color = Some(color.to_string());
        self
    }
}

impl Collection {
    /// Check if this collection has a custom icon.
    #[must_use]
    pub fn has_icon(&self) -> bool {
        self.icon.is_some()
    }

    /// Check if this collection has a custom color.
    #[must_use]
    pub fn has_color(&self) -> bool {
        self.color.is_some()
    }

    /// Get display icon or default.
    #[must_use]
    pub fn display_icon(&self) -> &str {
        self.icon.as_deref().unwrap_or("📁")
    }

    /// Get display color or default.
    #[must_use]
    pub fn display_color(&self) -> &str {
        self.color.as_deref().unwrap_or("#808080")
    }
}

/// Pre-defined collection icons.
pub const COLLECTION_ICONS: &[&str] = &[
    "📁", "📂", "🎵", "🎶", "🎹", "🎸", "🥁", "🎺", "🎻", "🎷", "🎤", "🎧", "🎼", "🔊", "💿", "📀",
    "⭐", "❤️", "🔥", "💎",
];

/// Pre-defined collection colors.
pub const COLLECTION_COLORS: &[&str] = &[
    "#FF5733", // Red-orange
    "#33FF57", // Green
    "#3357FF", // Blue
    "#FF33F5", // Pink
    "#FFD700", // Gold
    "#00CED1", // Cyan
    "#9932CC", // Purple
    "#FF6347", // Tomato
    "#20B2AA", // Light sea green
    "#FF69B4", // Hot pink
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_collection() {
        let collection = CreateCollection::new("My Drums")
            .with_description("Favorite drum patterns")
            .with_icon("🥁")
            .with_color("#FF5733");

        assert_eq!(collection.name, "My Drums");
        assert_eq!(
            collection.description,
            Some("Favorite drum patterns".to_string())
        );
        assert_eq!(collection.icon, Some("🥁".to_string()));
        assert_eq!(collection.color, Some("#FF5733".to_string()));
    }

    #[test]
    fn test_collection_display() {
        let with_icon = Collection {
            id: 1,
            name: "Test".to_string(),
            description: None,
            icon: Some("🎹".to_string()),
            color: Some("#FF0000".to_string()),
            created_at: None,
            modified_at: None,
        };
        assert_eq!(with_icon.display_icon(), "🎹");
        assert_eq!(with_icon.display_color(), "#FF0000");
        assert!(with_icon.has_icon());
        assert!(with_icon.has_color());

        let without = Collection {
            id: 2,
            name: "Test2".to_string(),
            description: None,
            icon: None,
            color: None,
            created_at: None,
            modified_at: None,
        };
        assert_eq!(without.display_icon(), "📁");
        assert_eq!(without.display_color(), "#808080");
        assert!(!without.has_icon());
        assert!(!without.has_color());
    }

    #[test]
    fn test_collection_icons_and_colors() {
        // Verify icons and colors are populated (const arrays are known at compile time)
        assert!(COLLECTION_ICONS.len() >= 10);
        assert!(COLLECTION_COLORS.len() >= 5);

        // All colors should be valid hex
        for color in COLLECTION_COLORS {
            assert!(color.starts_with('#'));
            assert_eq!(color.len(), 7);
        }
    }
}

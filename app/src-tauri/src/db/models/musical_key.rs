//! Musical Key database models for VIP3 filtering.
//!
//! Musical keys for key-based filtering:
//! - All 12 major keys (C, C#, D, D#, E, F, F#, G, G#, A, A#, B)
//! - All 12 minor keys (Cm, C#m, Dm, D#m, Em, Fm, F#m, Gm, G#m, Am, A#m, Bm)
//! - Pre-populated with 24 standard keys.

use serde::{Deserialize, Serialize};

/// Musical key record for key-based filtering.
///
/// Used in VIP3-style filtering to quickly filter files by musical key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct MusicalKey {
    /// Primary key (SMALLSERIAL)
    pub id: i16,

    /// Key name (e.g., "C", "Am", "F#m")
    pub name: String,

    /// Root note (e.g., "C", "C#", "D")
    pub root_note: String,

    /// Mode: "Major" or "Minor"
    pub mode: String,

    /// Display order in UI (typically by circle of fifths)
    pub sort_order: Option<i16>,
}

/// Musical key with file count for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct MusicalKeyWithCount {
    pub id: i16,
    pub name: String,
    pub root_note: String,
    pub mode: String,
    pub sort_order: Option<i16>,
    pub file_count: i64,
}

impl MusicalKey {
    /// Check if this is a major key.
    #[must_use]
    pub fn is_major(&self) -> bool {
        self.mode == "Major"
    }

    /// Check if this is a minor key.
    #[must_use]
    pub fn is_minor(&self) -> bool {
        self.mode == "Minor"
    }

    /// Get the relative key (major -> relative minor, minor -> relative major).
    #[must_use]
    pub fn relative_key(&self) -> Option<&'static str> {
        match self.name.as_str() {
            // Major to relative minor (down 3 semitones)
            "C" => Some("Am"),
            "C#" => Some("A#m"),
            "D" => Some("Bm"),
            "D#" => Some("Cm"),
            "E" => Some("C#m"),
            "F" => Some("Dm"),
            "F#" => Some("D#m"),
            "G" => Some("Em"),
            "G#" => Some("Fm"),
            "A" => Some("F#m"),
            "A#" => Some("Gm"),
            "B" => Some("G#m"),
            // Minor to relative major (up 3 semitones)
            "Am" => Some("C"),
            "A#m" => Some("C#"),
            "Bm" => Some("D"),
            "Cm" => Some("D#"),
            "C#m" => Some("E"),
            "Dm" => Some("F"),
            "D#m" => Some("F#"),
            "Em" => Some("G"),
            "Fm" => Some("G#"),
            "F#m" => Some("A"),
            "Gm" => Some("A#"),
            "G#m" => Some("B"),
            _ => None,
        }
    }

    /// Get the parallel key (same root, opposite mode).
    #[must_use]
    pub fn parallel_key(&self) -> String {
        if self.is_major() {
            format!("{}m", self.root_note)
        } else {
            self.root_note.clone()
        }
    }

    /// Get the semitone offset from C (0-11).
    #[must_use]
    pub fn semitone_offset(&self) -> u8 {
        match self.root_note.as_str() {
            "C" => 0,
            "C#" | "Db" => 1,
            "D" => 2,
            "D#" | "Eb" => 3,
            "E" => 4,
            "F" => 5,
            "F#" | "Gb" => 6,
            "G" => 7,
            "G#" | "Ab" => 8,
            "A" => 9,
            "A#" | "Bb" => 10,
            "B" => 11,
            _ => 0,
        }
    }

    /// Check if two keys are harmonically compatible (same or relative keys).
    #[must_use]
    pub fn is_compatible_with(&self, other: &MusicalKey) -> bool {
        self.name == other.name
            || self.relative_key() == Some(other.name.as_str())
            || other.relative_key() == Some(self.name.as_str())
    }
}

/// Pre-defined musical keys (major and minor).
pub const SYSTEM_MUSICAL_KEYS: &[(&str, &str, &str, i16)] = &[
    ("C", "C", "Major", 1),
    ("Cm", "C", "Minor", 2),
    ("C#", "C#", "Major", 3),
    ("C#m", "C#", "Minor", 4),
    ("D", "D", "Major", 5),
    ("Dm", "D", "Minor", 6),
    ("D#", "D#", "Major", 7),
    ("D#m", "D#", "Minor", 8),
    ("E", "E", "Major", 9),
    ("Em", "E", "Minor", 10),
    ("F", "F", "Major", 11),
    ("Fm", "F", "Minor", 12),
    ("F#", "F#", "Major", 13),
    ("F#m", "F#", "Minor", 14),
    ("G", "G", "Major", 15),
    ("Gm", "G", "Minor", 16),
    ("G#", "G#", "Major", 17),
    ("G#m", "G#", "Minor", 18),
    ("A", "A", "Major", 19),
    ("Am", "A", "Minor", 20),
    ("A#", "A#", "Major", 21),
    ("A#m", "A#", "Minor", 22),
    ("B", "B", "Major", 23),
    ("Bm", "B", "Minor", 24),
];

/// Parse a key string into (root_note, mode).
#[must_use]
pub fn parse_key_string(key: &str) -> Option<(String, String)> {
    let key = key.trim();
    if key.is_empty() {
        return None;
    }

    // Check for explicit minor indicators
    let is_minor = key.ends_with('m')
        || key.to_lowercase().ends_with("min")
        || key.to_lowercase().ends_with("minor");

    // Extract root note
    let root =
        if key.len() >= 2 && (key.chars().nth(1) == Some('#') || key.chars().nth(1) == Some('b')) {
            key[..2].to_string()
        } else {
            key.chars().next()?.to_string()
        };

    let mode = if is_minor { "Minor" } else { "Major" };
    Some((root, mode.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_key(name: &str, root: &str, mode: &str) -> MusicalKey {
        MusicalKey {
            id: 1,
            name: name.to_string(),
            root_note: root.to_string(),
            mode: mode.to_string(),
            sort_order: Some(1),
        }
    }

    #[test]
    fn test_is_major_minor() {
        let c_major = make_key("C", "C", "Major");
        assert!(c_major.is_major());
        assert!(!c_major.is_minor());

        let a_minor = make_key("Am", "A", "Minor");
        assert!(!a_minor.is_major());
        assert!(a_minor.is_minor());
    }

    #[test]
    fn test_relative_keys() {
        let c_major = make_key("C", "C", "Major");
        assert_eq!(c_major.relative_key(), Some("Am"));

        let a_minor = make_key("Am", "A", "Minor");
        assert_eq!(a_minor.relative_key(), Some("C"));

        let f_sharp_minor = make_key("F#m", "F#", "Minor");
        assert_eq!(f_sharp_minor.relative_key(), Some("A"));
    }

    #[test]
    fn test_parallel_keys() {
        let c_major = make_key("C", "C", "Major");
        assert_eq!(c_major.parallel_key(), "Cm");

        let a_minor = make_key("Am", "A", "Minor");
        assert_eq!(a_minor.parallel_key(), "A");
    }

    #[test]
    fn test_semitone_offset() {
        let c = make_key("C", "C", "Major");
        assert_eq!(c.semitone_offset(), 0);

        let f_sharp = make_key("F#", "F#", "Major");
        assert_eq!(f_sharp.semitone_offset(), 6);

        let b = make_key("B", "B", "Major");
        assert_eq!(b.semitone_offset(), 11);
    }

    #[test]
    fn test_key_compatibility() {
        let c_major = make_key("C", "C", "Major");
        let a_minor = make_key("Am", "A", "Minor");
        let g_major = make_key("G", "G", "Major");

        assert!(c_major.is_compatible_with(&a_minor)); // Relative keys
        assert!(a_minor.is_compatible_with(&c_major));
        assert!(!c_major.is_compatible_with(&g_major)); // Not relative
    }

    #[test]
    fn test_parse_key_string() {
        assert_eq!(
            parse_key_string("C"),
            Some(("C".to_string(), "Major".to_string()))
        );
        assert_eq!(
            parse_key_string("Am"),
            Some(("A".to_string(), "Minor".to_string()))
        );
        assert_eq!(
            parse_key_string("F#m"),
            Some(("F#".to_string(), "Minor".to_string()))
        );
        assert_eq!(
            parse_key_string("Bb"),
            Some(("Bb".to_string(), "Major".to_string()))
        );
        assert_eq!(parse_key_string(""), None);
    }

    #[test]
    fn test_system_musical_keys_count() {
        assert_eq!(SYSTEM_MUSICAL_KEYS.len(), 24);
    }
}

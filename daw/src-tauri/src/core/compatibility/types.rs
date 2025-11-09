   /// Compatibility types - Music theory data structures
   ///
   /// Trusty Module: Pure data types for compatibility calculations.

use serde::{Deserialize, Serialize};

/// Musical key
///
/// Represents the 12 chromatic pitches using semitone numbering (0-11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    C = 0,
    CSharp = 1,
    D = 2,
    DSharp = 3,
    E = 4,
    F = 5,
    FSharp = 6,
    G = 7,
    GSharp = 8,
    A = 9,
    ASharp = 10,
    B = 11,
}

impl Key {
    /// Parse key from string (e.g., "C", "C#", "Db")
    ///
    /// Supports both sharp and flat notation (enharmonic equivalents).
    pub fn from_str(s: &str) -> Option<Self> {
        let normalized = s.to_uppercase().replace(" ", "");

        match normalized.as_str() {
            "C" => Some(Key::C),
            "C#" | "DB" => Some(Key::CSharp),
            "D" => Some(Key::D),
            "D#" | "EB" => Some(Key::DSharp),
            "E" => Some(Key::E),
            "F" => Some(Key::F),
            "F#" | "GB" => Some(Key::FSharp),
            "G" => Some(Key::G),
            "G#" | "AB" => Some(Key::GSharp),
            "A" => Some(Key::A),
            "A#" | "BB" => Some(Key::ASharp),
            "B" => Some(Key::B),
            _ => None,
        }
    }

    /// Get semitone value (0-11)
    pub fn semitone(&self) -> i32 {
        *self as i32
    }

    /// Get key name as string
    pub fn name(&self) -> &'static str {
        match self {
            Key::C => "C",
            Key::CSharp => "C#",
            Key::D => "D",
            Key::DSharp => "D#",
            Key::E => "E",
            Key::F => "F",
            Key::FSharp => "F#",
            Key::G => "G",
            Key::GSharp => "G#",
            Key::A => "A",
            Key::ASharp => "A#",
            Key::B => "B",
        }
    }
}

/// Musical mode (major or minor)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Major,
    Minor,
}

impl Mode {
    /// Parse mode from string
    ///
    /// Detects 'm' or 'min' for minor, defaults to major.
    pub fn from_str(s: &str) -> Self {
        let lower = s.to_lowercase();
        if lower.contains('m') && !lower.contains("maj") {
            Mode::Minor
        } else {
            Mode::Major
        }
    }
}

/// Complete key signature (key + mode)
///
/// Represents the tonality of a piece (e.g., "C Major", "Am").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeySignature {
    pub key: Key,
    pub mode: Mode,
}

impl KeySignature {
    /// Parse from string (e.g., "C", "Am", "F# Major")
    pub fn from_str(s: &str) -> Option<Self> {
        let mode = Mode::from_str(s);

        // Extract key name (first part before mode indicator)
        let key_part = s
            .split_whitespace()
            .next()
            .unwrap_or(s)
            .trim_end_matches('m');

        let key = Key::from_str(key_part)?;

        Some(KeySignature { key, mode })
    }

    /// Get human-readable name (e.g., "C Major", "Am")
    pub fn name(&self) -> String {
        match self.mode {
            Mode::Major => format!("{} Major", self.key.name()),
            Mode::Minor => format!("{}m", self.key.name()),
        }
    }
}

/// Compatibility score with detailed breakdown
///
/// All scores are 0-100 (percentage compatibility).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityScore {
    pub total_score: f32,      // Overall compatibility (0-100)
    pub key_score: f32,         // Key/harmony compatibility (0-100)
    pub bpm_score: f32,         // Tempo compatibility (0-100)
    pub category_score: f32,    // Category/style compatibility (0-100)
    pub explanation: String,    // Human-readable explanation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_from_str() {
        assert_eq!(Key::from_str("C").unwrap(), Key::C);
        assert_eq!(Key::from_str("C#").unwrap(), Key::CSharp);
        assert_eq!(Key::from_str("Db").unwrap(), Key::CSharp); // Enharmonic
        assert_eq!(Key::from_str("G").unwrap(), Key::G);
        assert!(Key::from_str("invalid").is_none());
    }

    #[test]
    fn test_key_semitone() {
        assert_eq!(Key::C.semitone(), 0);
        assert_eq!(Key::CSharp.semitone(), 1);
        assert_eq!(Key::B.semitone(), 11);
    }

    #[test]
    fn test_mode_from_str() {
        assert_eq!(Mode::from_str("Major"), Mode::Major);
        assert_eq!(Mode::from_str("Minor"), Mode::Minor);
        assert_eq!(Mode::from_str("m"), Mode::Minor);
        assert_eq!(Mode::from_str(""), Mode::Major); // Default
    }

    #[test]
    fn test_key_signature_from_str() {
        let c_maj = KeySignature::from_str("C").unwrap();
        assert_eq!(c_maj.key, Key::C);
        assert_eq!(c_maj.mode, Mode::Major);

        let a_min = KeySignature::from_str("Am").unwrap();
        assert_eq!(a_min.key, Key::A);
        assert_eq!(a_min.mode, Mode::Minor);

        let g_maj = KeySignature::from_str("G Major").unwrap();
        assert_eq!(g_maj.key, Key::G);
        assert_eq!(g_maj.mode, Mode::Major);
    }
}

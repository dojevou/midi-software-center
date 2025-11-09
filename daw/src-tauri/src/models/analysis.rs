   /// Analysis and compatibility models
   ///
   /// Trusty Module: Pure data structures for musical compatibility analysis.

use serde::Serialize;
use std::str::FromStr;

/**
 * Compatible file match
 *
 * Represents a file that is musically compatible with a reference file.
 */
#[derive(Debug, Serialize)]
pub struct CompatibleFile {
    pub id: i32,
    pub file_name: String,
    pub compatibility_score: i32,         // 0-100
    pub key_match: bool,
    pub bpm_difference: Option<f32>,
    pub time_signature_match: bool,
    pub suggested_bpm_multiplier: Option<f32>,
    pub category: Option<String>,
}

/**
 * Key signature for compatibility analysis
 *
 * Represents musical keys in semitone notation.
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
    A,
    ASharp,
    B,
}

impl Key {
    /// Parse key from string (e.g., "C", "C#", "Cm", "C#m")
    ///
    /// Handles both sharp and flat notations.
    pub fn from_string(s: &str) -> Option<Self> {
        let key_part = s.split('m').next()?;

        match key_part {
            "C" => Some(Key::C),
            "C#" | "Db" => Some(Key::CSharp),
            "D" => Some(Key::D),
            "D#" | "Eb" => Some(Key::DSharp),
            "E" => Some(Key::E),
            "F" => Some(Key::F),
            "F#" | "Gb" => Some(Key::FSharp),
            "G" => Some(Key::G),
            "G#" | "Ab" => Some(Key::GSharp),
            "A" => Some(Key::A),
            "A#" | "Bb" => Some(Key::ASharp),
            "B" => Some(Key::B),
            _ => None,
        }
    }

    /// Get semitone value (0-11)
    pub fn semitone(&self) -> i32 {
        match self {
            Key::C => 0,
            Key::CSharp => 1,
            Key::D => 2,
            Key::DSharp => 3,
            Key::E => 4,
            Key::F => 5,
            Key::FSharp => 6,
            Key::G => 7,
            Key::GSharp => 8,
            Key::A => 9,
            Key::ASharp => 10,
            Key::B => 11,
        }
    }

    /// Calculate distance between two keys (shortest path on circle of fifths)
    pub fn distance(&self, other: &Key) -> i32 {
        let diff = (other.semitone() - self.semitone()).abs();
        // Return shortest distance around the circle (max 6 semitones)
        diff.min(12 - diff)
    }
}

impl FromStr for Key {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s).ok_or_else(|| format!("Invalid key: {}", s))
    }
}

/**
 * Mode (major or minor)
 *
 * Musical mode for key signature analysis.
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Major,
    Minor,
}

impl Mode {
    /// Parse mode from string
    ///
    /// Detects "m" suffix for minor, defaults to major.
    pub fn from_string(s: &str) -> Self {
        if s.ends_with('m') && !s.ends_with("Maj") {
            Mode::Minor
        } else {
            Mode::Major
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_parsing() {
        assert_eq!(Key::from_string("C"), Some(Key::C));
        assert_eq!(Key::from_string("C#"), Some(Key::CSharp));
        assert_eq!(Key::from_string("Db"), Some(Key::CSharp));
        assert_eq!(Key::from_string("Cm"), Some(Key::C));
        assert_eq!(Key::from_string("C#m"), Some(Key::CSharp));
    }

    #[test]
    fn test_key_distance() {
        let c = Key::C;
        let g = Key::G;
        let f = Key::F;

        assert_eq!(c.distance(&g), 5); // C to G is 5 semitones
        assert_eq!(c.distance(&f), 5); // C to F is 5 semitones (going backwards)
        assert_eq!(c.distance(&c), 0); // Same key
    }

    #[test]
    fn test_mode_parsing() {
        assert_eq!(Mode::from_string("C"), Mode::Major);
        assert_eq!(Mode::from_string("Cm"), Mode::Minor);
        assert_eq!(Mode::from_string("CMaj"), Mode::Major);
    }
}

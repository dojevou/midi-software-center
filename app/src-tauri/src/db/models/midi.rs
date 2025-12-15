//! MIDI metadata database models
//!
//! Represents the `musical_metadata` table containing musical properties
//! extracted from MIDI files during analysis.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Musical metadata for a MIDI file.
///
/// Matches the `musical_metadata` database table.
/// Stores tempo, key, time signature, note statistics, and musical characteristics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct MidiMetadata {
    /// Foreign key to files table (PRIMARY KEY)
    pub file_id: i64,

    // === Tempo ===
    /// Detected BPM (20-300 range)
    pub bpm: Option<rust_decimal::Decimal>,

    /// BPM detection confidence (0.0-1.0)
    pub bpm_confidence: Option<f32>,

    /// Whether the file has tempo changes
    pub has_tempo_changes: Option<bool>,

    /// Tempo change events (JSONB array)
    #[cfg_attr(feature = "database", sqlx(default))]
    pub tempo_changes: Option<Value>,

    // === Key Signature ===
    /// Detected key signature (e.g., "C", "Am", "F#m")
    pub key_signature: Option<String>,

    /// Key detection confidence (0.0-1.0)
    pub key_confidence: Option<f32>,

    /// Whether the file has key changes
    pub has_key_changes: Option<bool>,

    /// Key change events (JSONB array)
    #[cfg_attr(feature = "database", sqlx(default))]
    pub key_changes: Option<Value>,

    // === Time Signature ===
    /// Time signature numerator (e.g., 4 for 4/4)
    pub time_signature_numerator: Option<i16>,

    /// Time signature denominator (e.g., 4 for 4/4)
    pub time_signature_denominator: Option<i16>,

    /// Whether the file has time signature changes
    pub has_time_signature_changes: Option<bool>,

    /// Time signature change events (JSONB array)
    #[cfg_attr(feature = "database", sqlx(default))]
    pub time_signature_changes: Option<Value>,

    // === Note Statistics ===
    /// Total number of note events
    pub total_notes: i32,

    /// Number of unique pitches used
    pub unique_pitches: Option<i32>,

    /// Lowest MIDI note (0-127)
    pub pitch_range_min: Option<i16>,

    /// Highest MIDI note (0-127)
    pub pitch_range_max: Option<i16>,

    /// Average note velocity (0-127)
    pub avg_velocity: Option<rust_decimal::Decimal>,

    // === Density Metrics ===
    /// Notes per second
    pub note_density: Option<rust_decimal::Decimal>,

    /// Maximum simultaneous notes
    pub polyphony_max: Option<i16>,

    /// Average simultaneous notes
    pub polyphony_avg: Option<rust_decimal::Decimal>,

    // === Musical Characteristics ===
    /// Single voice (one note at a time)
    pub is_monophonic: Option<bool>,

    /// Multiple simultaneous notes
    pub is_polyphonic: Option<bool>,

    /// Contains drum/percussion notes (channel 10)
    pub is_percussive: Option<bool>,

    // === Chord Analysis ===
    /// Contains chord progressions
    pub has_chords: Option<bool>,

    /// Chord progression complexity (0.0-1.0)
    pub chord_complexity: Option<f32>,

    // === Melody Analysis ===
    /// Contains melodic content
    pub has_melody: Option<bool>,

    /// Melodic pitch range in semitones
    pub melodic_range: Option<i16>,

    /// When this metadata was created
    pub created_at: Option<DateTime<Utc>>,
}

/// Data required to create musical metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMidiMetadata {
    pub file_id: i64,
    pub bpm: Option<rust_decimal::Decimal>,
    pub bpm_confidence: Option<f32>,
    pub has_tempo_changes: Option<bool>,
    pub tempo_changes: Option<Value>,
    pub key_signature: Option<String>,
    pub key_confidence: Option<f32>,
    pub time_signature_numerator: Option<i16>,
    pub time_signature_denominator: Option<i16>,
    pub total_notes: i32,
    pub unique_pitches: Option<i32>,
    pub pitch_range_min: Option<i16>,
    pub pitch_range_max: Option<i16>,
    pub avg_velocity: Option<rust_decimal::Decimal>,
    pub note_density: Option<rust_decimal::Decimal>,
    pub polyphony_max: Option<i16>,
    pub polyphony_avg: Option<rust_decimal::Decimal>,
    pub is_monophonic: Option<bool>,
    pub is_polyphonic: Option<bool>,
    pub is_percussive: Option<bool>,
}

/// Optional fields for updating musical metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateMidiMetadata {
    pub bpm: Option<rust_decimal::Decimal>,
    pub bpm_confidence: Option<f32>,
    pub key_signature: Option<String>,
    pub key_confidence: Option<f32>,
    pub has_chords: Option<bool>,
    pub chord_complexity: Option<f32>,
    pub has_melody: Option<bool>,
    pub melodic_range: Option<i16>,
}

/// Instrument information detected in a MIDI file.
///
/// Matches the `file_instruments` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct FileInstrument {
    pub id: i64,
    pub file_id: i64,

    /// MIDI channel (0-15)
    pub channel: i16,

    /// MIDI program number (0-127)
    pub program_number: i16,

    /// General MIDI program name
    pub program_name: Option<String>,

    /// Instrument family (e.g., "strings", "brass")
    pub instrument_family: Option<String>,

    /// Specific instrument type
    pub instrument_type: Option<String>,

    /// Number of notes played by this instrument
    pub note_count: Option<i32>,

    /// Whether this is the primary instrument
    pub is_primary: Option<bool>,

    /// Average velocity for this instrument
    pub avg_velocity: Option<rust_decimal::Decimal>,

    /// Lowest note played
    pub pitch_range_low: Option<i16>,

    /// Highest note played
    pub pitch_range_high: Option<i16>,

    pub created_at: Option<DateTime<Utc>>,
}

/// Data required to create an instrument record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFileInstrument {
    pub file_id: i64,
    pub channel: i16,
    pub program_number: i16,
    pub program_name: Option<String>,
    pub instrument_family: Option<String>,
    pub instrument_type: Option<String>,
    pub note_count: Option<i32>,
    pub is_primary: Option<bool>,
}

/// MIDI track information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct MidiTrack {
    pub id: i64,
    pub file_id: i64,
    pub track_number: i16,
    pub track_name: Option<String>,
    pub channel: Option<i16>,
    pub instrument_name: Option<String>,
    pub note_count: i32,
    pub created_at: Option<DateTime<Utc>>,
}

impl MidiMetadata {
    /// Get time signature as a string (e.g., "4/4").
    #[must_use]
    pub fn time_signature_string(&self) -> String {
        let num = self.time_signature_numerator.unwrap_or(4);
        let denom = self.time_signature_denominator.unwrap_or(4);
        format!("{num}/{denom}")
    }

    /// Get pitch range in semitones.
    #[must_use]
    pub fn pitch_range(&self) -> Option<i16> {
        match (self.pitch_range_min, self.pitch_range_max) {
            (Some(min), Some(max)) => Some(max - min),
            _ => None,
        }
    }

    /// Check if this is a drums file.
    #[must_use]
    pub fn is_drums(&self) -> bool {
        self.is_percussive.unwrap_or(false)
    }

    /// Check if this has tempo information.
    #[must_use]
    pub fn has_tempo(&self) -> bool {
        self.bpm.is_some()
    }

    /// Check if this has key information.
    #[must_use]
    pub fn has_key(&self) -> bool {
        self.key_signature.is_some()
    }
}

impl CreateMidiMetadata {
    /// Create a new metadata record with minimal required fields.
    #[must_use]
    pub fn new(file_id: i64, total_notes: i32) -> Self {
        Self {
            file_id,
            bpm: None,
            bpm_confidence: None,
            has_tempo_changes: None,
            tempo_changes: None,
            key_signature: None,
            key_confidence: None,
            time_signature_numerator: Some(4),
            time_signature_denominator: Some(4),
            total_notes,
            unique_pitches: None,
            pitch_range_min: None,
            pitch_range_max: None,
            avg_velocity: None,
            note_density: None,
            polyphony_max: None,
            polyphony_avg: None,
            is_monophonic: None,
            is_polyphonic: None,
            is_percussive: None,
        }
    }

    /// Set tempo information.
    #[must_use]
    pub fn with_tempo(mut self, bpm: rust_decimal::Decimal, confidence: f32) -> Self {
        self.bpm = Some(bpm);
        self.bpm_confidence = Some(confidence);
        self
    }

    /// Set key information.
    #[must_use]
    pub fn with_key(mut self, key: &str, confidence: f32) -> Self {
        self.key_signature = Some(key.to_string());
        self.key_confidence = Some(confidence);
        self
    }

    /// Set time signature.
    #[must_use]
    pub fn with_time_signature(mut self, numerator: i16, denominator: i16) -> Self {
        self.time_signature_numerator = Some(numerator);
        self.time_signature_denominator = Some(denominator);
        self
    }

    /// Set note statistics.
    #[must_use]
    pub fn with_note_stats(mut self, unique_pitches: i32, min_pitch: i16, max_pitch: i16) -> Self {
        self.unique_pitches = Some(unique_pitches);
        self.pitch_range_min = Some(min_pitch);
        self.pitch_range_max = Some(max_pitch);
        self
    }

    /// Mark as percussive/drums.
    #[must_use]
    pub fn as_percussive(mut self) -> Self {
        self.is_percussive = Some(true);
        self
    }
}

impl FileInstrument {
    /// Get the pitch range in semitones.
    #[must_use]
    pub fn pitch_range(&self) -> Option<i16> {
        match (self.pitch_range_low, self.pitch_range_high) {
            (Some(low), Some(high)) => Some(high - low),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_metadata() {
        let metadata = CreateMidiMetadata::new(1, 100)
            .with_tempo(dec!(120.0), 0.95)
            .with_key("Am", 0.87)
            .with_time_signature(4, 4)
            .with_note_stats(24, 36, 84);

        assert_eq!(metadata.file_id, 1);
        assert_eq!(metadata.total_notes, 100);
        assert_eq!(metadata.bpm, Some(dec!(120.0)));
        assert_eq!(metadata.key_signature, Some("Am".to_string()));
        assert_eq!(metadata.unique_pitches, Some(24));
    }

    #[test]
    fn test_time_signature_string() {
        let metadata = MidiMetadata {
            file_id: 1,
            bpm: Some(dec!(120.0)),
            bpm_confidence: Some(0.95),
            has_tempo_changes: Some(false),
            tempo_changes: None,
            key_signature: Some("C".to_string()),
            key_confidence: Some(0.9),
            has_key_changes: Some(false),
            key_changes: None,
            time_signature_numerator: Some(6),
            time_signature_denominator: Some(8),
            has_time_signature_changes: Some(false),
            time_signature_changes: None,
            total_notes: 100,
            unique_pitches: Some(24),
            pitch_range_min: Some(36),
            pitch_range_max: Some(84),
            avg_velocity: Some(dec!(80.5)),
            note_density: Some(dec!(4.5)),
            polyphony_max: Some(4),
            polyphony_avg: Some(dec!(2.1)),
            is_monophonic: Some(false),
            is_polyphonic: Some(true),
            is_percussive: Some(false),
            has_chords: Some(true),
            chord_complexity: Some(0.6),
            has_melody: Some(true),
            melodic_range: Some(48),
            created_at: None,
        };

        assert_eq!(metadata.time_signature_string(), "6/8");
        assert_eq!(metadata.pitch_range(), Some(48));
        assert!(!metadata.is_drums());
        assert!(metadata.has_tempo());
        assert!(metadata.has_key());
    }

    #[test]
    fn test_percussive() {
        let metadata = CreateMidiMetadata::new(1, 50).as_percussive();
        assert_eq!(metadata.is_percussive, Some(true));
    }
}

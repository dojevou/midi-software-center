//! DAW/Sequencer models
//!
//! Models for the real-time DAW component including projects, tracks,
//! and playback state.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Sequencer project containing tracks and settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct SequencerProject {
    /// Project ID (UUID for uniqueness)
    pub id: Uuid,

    /// Project name
    pub name: String,

    /// Project BPM
    pub bpm: f64,

    /// Time signature as string (e.g., "4/4")
    pub time_signature: String,

    /// Project length in measures
    pub length_measures: i32,

    /// Additional project settings (JSONB)
    #[cfg_attr(feature = "database", sqlx(default))]
    pub data: Value,

    /// When the project was created
    pub created_at: Option<DateTime<Utc>>,

    /// When the project was last updated
    pub updated_at: Option<DateTime<Utc>>,
}

/// Individual track within a sequencer project.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct SequencerTrack {
    /// Track ID (UUID)
    pub id: Uuid,

    /// Parent project ID
    pub project_id: Uuid,

    /// Track display name
    pub name: String,

    /// Track index (ordering)
    pub track_index: i32,

    /// MIDI channel (0-15)
    pub channel: Option<i32>,

    /// Instrument name
    pub instrument: Option<String>,

    /// Volume level (0.0-1.0)
    pub volume: Option<f64>,

    /// Pan position (-1.0 to 1.0)
    pub pan: Option<f64>,

    /// Whether track is muted
    pub muted: bool,

    /// Whether track is solo'd
    pub solo: bool,

    /// Track color (CSS color string)
    pub color: Option<String>,

    /// Track data including notes and automation (JSONB)
    #[cfg_attr(feature = "database", sqlx(default))]
    pub data: Value,

    /// When the track was created
    pub created_at: Option<DateTime<Utc>>,

    /// When the track was last updated
    pub updated_at: Option<DateTime<Utc>>,
}

/// Data required to create a new project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSequencerProject {
    pub name: String,
    pub bpm: f64,
    pub time_signature: String,
    pub length_measures: i32,
    pub data: Option<Value>,
}

/// Data required to create a new track.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSequencerTrack {
    pub project_id: Uuid,
    pub name: String,
    pub track_index: i32,
    pub channel: Option<i32>,
    pub instrument: Option<String>,
    pub volume: Option<f64>,
    pub pan: Option<f64>,
    pub muted: bool,
    pub solo: bool,
    pub color: Option<String>,
    pub data: Value,
}

/// Optional fields for updating a track.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateSequencerTrack {
    pub name: Option<String>,
    pub track_index: Option<i32>,
    pub channel: Option<i32>,
    pub instrument: Option<String>,
    pub volume: Option<f64>,
    pub pan: Option<f64>,
    pub muted: Option<bool>,
    pub solo: Option<bool>,
    pub color: Option<String>,
    pub data: Option<Value>,
}

/// MIDI note event in the sequencer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequencerNote {
    /// Note start time in ticks
    pub start_tick: i64,

    /// Note duration in ticks
    pub duration_ticks: i64,

    /// MIDI pitch (0-127)
    pub pitch: u8,

    /// MIDI velocity (0-127)
    pub velocity: u8,

    /// MIDI channel (0-15)
    pub channel: u8,
}

/// Automation point for a parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationPoint {
    /// Time in ticks
    pub tick: i64,

    /// Parameter value (0.0-1.0 normalized)
    pub value: f64,

    /// Curve type (linear, exponential, etc.)
    pub curve: Option<String>,
}

/// Automation lane for a specific parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationLane {
    /// Parameter name (e.g., "volume", "pan", "cc1")
    pub parameter: String,

    /// Automation points
    pub points: Vec<AutomationPoint>,
}

/// Current playback state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackState {
    /// Whether playback is active
    pub is_playing: bool,

    /// Whether recording is active
    pub is_recording: bool,

    /// Current position in ticks
    pub position_ticks: i64,

    /// Current position in seconds
    pub position_seconds: f64,

    /// Loop start position (if looping)
    pub loop_start: Option<i64>,

    /// Loop end position (if looping)
    pub loop_end: Option<i64>,

    /// Whether looping is enabled
    pub loop_enabled: bool,

    /// Current BPM (may differ from project if tempo automation)
    pub current_bpm: f64,
}

impl SequencerProject {
    /// Create a new project with default settings.
    #[must_use]
    pub fn new(name: &str) -> CreateSequencerProject {
        CreateSequencerProject {
            name: name.to_string(),
            bpm: 120.0,
            time_signature: "4/4".to_string(),
            length_measures: 16,
            data: None,
        }
    }

    /// Get time signature as a tuple (numerator, denominator).
    #[must_use]
    pub fn time_sig_tuple(&self) -> (i32, i32) {
        let parts: Vec<&str> = self.time_signature.split('/').collect();
        if parts.len() == 2 {
            let num = parts[0].parse().unwrap_or(4);
            let denom = parts[1].parse().unwrap_or(4);
            (num, denom)
        } else {
            (4, 4)
        }
    }

    /// Calculate project duration in seconds.
    #[must_use]
    pub fn duration_seconds(&self) -> f64 {
        let (num, denom) = self.time_sig_tuple();
        let beats_per_measure = num as f64 * (4.0 / denom as f64);
        let total_beats = beats_per_measure * self.length_measures as f64;
        (total_beats / self.bpm) * 60.0
    }
}

impl SequencerTrack {
    /// Check if track is audible (not muted, or solo is active somewhere).
    #[must_use]
    pub fn is_audible(&self) -> bool {
        !self.muted
    }

    /// Get effective volume (0.0-1.0).
    #[must_use]
    pub fn effective_volume(&self) -> f64 {
        self.volume.unwrap_or(1.0)
    }

    /// Get effective pan (-1.0 to 1.0).
    #[must_use]
    pub fn effective_pan(&self) -> f64 {
        self.pan.unwrap_or(0.0)
    }
}

impl CreateSequencerTrack {
    /// Create a new track with minimal settings.
    #[must_use]
    pub fn new(project_id: Uuid, name: &str, index: i32) -> Self {
        Self {
            project_id,
            name: name.to_string(),
            track_index: index,
            channel: None,
            instrument: None,
            volume: Some(1.0),
            pan: Some(0.0),
            muted: false,
            solo: false,
            color: None,
            data: Value::Object(serde_json::Map::new()),
        }
    }

    /// Set MIDI channel.
    #[must_use]
    pub fn with_channel(mut self, channel: i32) -> Self {
        self.channel = Some(channel);
        self
    }

    /// Set instrument.
    #[must_use]
    pub fn with_instrument(mut self, instrument: &str) -> Self {
        self.instrument = Some(instrument.to_string());
        self
    }

    /// Set track color.
    #[must_use]
    pub fn with_color(mut self, color: &str) -> Self {
        self.color = Some(color.to_string());
        self
    }
}

impl SequencerNote {
    /// Create a new note.
    #[must_use]
    pub fn new(start_tick: i64, duration_ticks: i64, pitch: u8, velocity: u8) -> Self {
        Self { start_tick, duration_ticks, pitch, velocity, channel: 0 }
    }

    /// Get note end tick.
    #[must_use]
    pub fn end_tick(&self) -> i64 {
        self.start_tick + self.duration_ticks
    }

    /// Check if note overlaps with a time range.
    #[must_use]
    pub fn overlaps(&self, start: i64, end: i64) -> bool {
        self.start_tick < end && self.end_tick() > start
    }
}

impl PlaybackState {
    /// Create a stopped state.
    #[must_use]
    pub fn stopped() -> Self {
        Self {
            is_playing: false,
            is_recording: false,
            position_ticks: 0,
            position_seconds: 0.0,
            loop_start: None,
            loop_end: None,
            loop_enabled: false,
            current_bpm: 120.0,
        }
    }

    /// Create a playing state at position.
    #[must_use]
    pub fn playing_at(position_ticks: i64, bpm: f64, tpqn: i32) -> Self {
        let position_seconds = (position_ticks as f64 / tpqn as f64) * (60.0 / bpm);
        Self {
            is_playing: true,
            is_recording: false,
            position_ticks,
            position_seconds,
            loop_start: None,
            loop_end: None,
            loop_enabled: false,
            current_bpm: bpm,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_project() {
        let project = SequencerProject::new("My Project");
        assert_eq!(project.name, "My Project");
        assert_eq!(project.bpm, 120.0);
        assert_eq!(project.time_signature, "4/4");
    }

    #[test]
    fn test_project_duration() {
        let project = SequencerProject {
            id: Uuid::new_v4(),
            name: "Test".to_string(),
            bpm: 120.0,
            time_signature: "4/4".to_string(),
            length_measures: 8,
            data: Value::Null,
            created_at: None,
            updated_at: None,
        };

        // 8 measures * 4 beats = 32 beats at 120 BPM = 16 seconds
        assert!((project.duration_seconds() - 16.0).abs() < 0.01);
    }

    #[test]
    fn test_create_track() {
        let project_id = Uuid::new_v4();
        let track = CreateSequencerTrack::new(project_id, "Drums", 0)
            .with_channel(10)
            .with_instrument("Standard Kit")
            .with_color("#FF0000");

        assert_eq!(track.name, "Drums");
        assert_eq!(track.channel, Some(10));
        assert_eq!(track.instrument, Some("Standard Kit".to_string()));
        assert_eq!(track.color, Some("#FF0000".to_string()));
    }

    #[test]
    fn test_note_overlap() {
        let note = SequencerNote::new(100, 50, 60, 100);
        assert!(note.overlaps(75, 125));
        assert!(note.overlaps(125, 175));
        assert!(!note.overlaps(0, 50));
        assert!(!note.overlaps(200, 250));
    }

    #[test]
    fn test_playback_state() {
        let state = PlaybackState::stopped();
        assert!(!state.is_playing);
        assert_eq!(state.position_ticks, 0);

        let playing = PlaybackState::playing_at(480, 120.0, 480);
        assert!(playing.is_playing);
        assert!((playing.position_seconds - 0.5).abs() < 0.01);
    }
}

//! Sequencer models
//!
//! Trusty Module: Pure data structures for sequencer state and tracks.

use serde::{Deserialize, Serialize};
use super::midi::MidiEvent;

/**
 * Sequencer track
 *
 * Represents a single track in the sequencer with playback properties.
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: i32,
    pub name: String,
    pub file_id: i32,
    pub channel: u8,
    pub muted: bool,
    pub solo: bool,
    pub volume: u8,      // 0-127
    pub pan: u8,         // 0-127 (64 = center)
    pub color: String,   // Hex color

    // Internal data (not serialized to frontend)
    #[serde(skip)]
    pub events: Vec<MidiEvent>,
}

/**
 * Track properties for updates
 *
 * Partial update structure for modifying track properties.
 */
#[derive(Debug, Deserialize)]
pub struct TrackProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solo: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pan: Option<u8>,
}

/**
 * Playback position
 *
 * Current position in the sequencer timeline.
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackPosition {
    pub current_tick: u64,
    pub current_bar: u32,
    pub current_beat: u32,
}

/**
 * Sequencer state
 *
 * Complete state of the sequencer engine.
 * This is internal state - not all fields are serialized to frontend.
 */
#[derive(Debug)]
pub struct SequencerState {
    pub is_playing: bool,
    pub tempo: f32,
    pub position: u64,
    pub tracks: Vec<Track>,
    pub next_track_id: i32,
}

impl SequencerState {
    /// Create new sequencer state with defaults
    pub fn new() -> Self {
        Self {
            is_playing: false,
            tempo: 120.0,
            position: 0,
            tracks: Vec::new(),
            next_track_id: 1,
        }
    }
}

impl Default for SequencerState {
    fn default() -> Self {
        Self::new()
    }
}

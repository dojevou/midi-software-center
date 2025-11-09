   /// MIDI event and device models
   ///
   /// Trusty Module: Pure data structures for MIDI hardware and events.

use serde::{Deserialize, Serialize};

/**
 * MIDI device information
 *
 * Represents a connected MIDI output device.
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiDevice {
    pub name: String,
    pub manufacturer: Option<String>,
}

/**
 * MIDI event type
 *
 * Supported MIDI message types for playback and recording.
 */
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MidiEventType {
    NoteOn,
    NoteOff,
    ControlChange,
    ProgramChange,
    PitchBend,
    Aftertouch,
}

/**
 * MIDI event
 *
 * Represents a single MIDI message with timing and data.
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiEvent {
    pub event_type: MidiEventType,
    pub tick: u64,
    pub channel: u8,

    // Optional fields depending on event type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub velocity: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controller: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program: Option<u8>,
}

/**
 * MIDI note (simplified for piano roll)
 *
 * Simplified representation of a note event with duration.
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiNote {
    pub pitch: u8,
    pub velocity: u8,
    pub start_tick: u64,
    pub duration_ticks: u64,
}

/**
 * MIDI pattern
 *
 * Complete pattern with events and timing information.
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct MidiPattern {
    pub events: Vec<MidiEvent>,
    pub ticks_per_quarter_note: u16,
    pub total_ticks: u64,
}

/**
 * MIDI connection status
 *
 * Represents the current state of MIDI hardware connection.
 */
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

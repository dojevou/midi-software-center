use serde::{Deserialize, Serialize};

/// Represents a complete MIDI file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiFile {
    pub header: Header,
    pub tracks: Vec<Track>,
}

/// MIDI header chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub format: u16, // 0, 1, or 2
    pub num_tracks: u16,
    pub ticks_per_quarter_note: u16,
}

/// A single MIDI track
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub events: Vec<TimedEvent>,
}

/// Event with delta time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimedEvent {
    pub delta_ticks: u32,
    pub event: Event,
}

/// MIDI events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    // Channel events
    NoteOn {
        channel: u8,
        note: u8,
        velocity: u8,
    },
    NoteOff {
        channel: u8,
        note: u8,
        velocity: u8,
    },
    Aftertouch {
        channel: u8,
        note: u8,
        pressure: u8,
    },
    ControlChange {
        channel: u8,
        controller: u8,
        value: u8,
    },
    ProgramChange {
        channel: u8,
        program: u8,
    },
    ChannelAftertouch {
        channel: u8,
        pressure: u8,
    },
    PitchBend {
        channel: u8,
        value: i16,
    },

    // Meta events
    TempoChange {
        microseconds_per_quarter: u32,
    },
    TimeSignature {
        numerator: u8,
        denominator: u8,
        clocks_per_click: u8,
        thirty_seconds_per_quarter: u8,
    },
    KeySignature {
        sharps_flats: i8,
        is_minor: bool,
    },
    Text {
        text_type: TextType,
        text: String,
    },
    EndOfTrack,

    // SysEx
    SysEx {
        data: Vec<u8>,
    },

    // Unknown/unsupported
    Unknown {
        status: u8,
        data: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextType {
    Text,
    Copyright,
    TrackName,
    InstrumentName,
    Lyric,
    Marker,
    CuePoint,
}

impl MidiFile {
    /// Calculate total duration in seconds
    pub fn duration_seconds(&self, _default_tempo_bpm: f64) -> f64 {
        let mut total_ticks = 0u64;
        let mut current_tempo_us_per_qn = 500_000u32; // Default: 120 BPM

        for track in &self.tracks {
            let mut track_ticks = 0u64;

            for timed_event in &track.events {
                track_ticks += timed_event.delta_ticks as u64;

                // Update tempo if we encounter a tempo change
                if let Event::TempoChange {
                    microseconds_per_quarter,
                } = timed_event.event
                {
                    current_tempo_us_per_qn = microseconds_per_quarter;
                }
            }

            total_ticks = total_ticks.max(track_ticks);
        }

        // Convert ticks to seconds
        let seconds_per_tick = (current_tempo_us_per_qn as f64 / 1_000_000.0)
            / self.header.ticks_per_quarter_note as f64;
        total_ticks as f64 * seconds_per_tick
    }

    /// Count total notes across all tracks
    pub fn total_notes(&self) -> usize {
        self.tracks
            .iter()
            .flat_map(|track| &track.events)
            .filter(|event| matches!(event.event, Event::NoteOn { velocity, .. } if velocity > 0))
            .count()
    }

    /// Get all unique MIDI channels used
    pub fn channels_used(&self) -> Vec<u8> {
        let mut channels = std::collections::HashSet::new();

        for track in &self.tracks {
            for timed_event in &track.events {
                if let Some(channel) = timed_event.event.channel() {
                    channels.insert(channel);
                }
            }
        }

        let mut result: Vec<u8> = channels.into_iter().collect();
        result.sort();
        result
    }
}

impl Event {
    /// Get the MIDI channel for channel events, None for meta/sysex
    pub fn channel(&self) -> Option<u8> {
        match self {
            Event::NoteOn { channel, .. }
            | Event::NoteOff { channel, .. }
            | Event::Aftertouch { channel, .. }
            | Event::ControlChange { channel, .. }
            | Event::ProgramChange { channel, .. }
            | Event::ChannelAftertouch { channel, .. }
            | Event::PitchBend { channel, .. } => Some(*channel),
            _ => None,
        }
    }

    /// Check if this is a note event
    pub fn is_note(&self) -> bool {
        matches!(self, Event::NoteOn { .. } | Event::NoteOff { .. })
    }
}

use crate::models::midi::{MidiEvent, MidiEventType};
/// MIDI file loading and parsing
///
/// Trusty Module: Pure functions for loading and parsing MIDI files.
/// Uses midly crate for efficient MIDI parsing.
use midly::{MidiMessage as MidlyMessage, Smf, Timing, TrackEventKind};
use std::fs;
use std::path::Path;
use tracing::debug;

use super::parser::{parse_midi, ParseError};

/// Load and parse a MIDI file from disk
///
/// Returns all MIDI events with absolute tick positions.
///
/// # Arguments
/// * `filepath` - Path to the MIDI file
///
/// # Returns
/// Result containing vector of parsed MIDI events with timing information
pub fn load_midi_file(filepath: &str) -> Result<LoadedMidiFile, String> {
    let path = Path::new(filepath);

    if !path.exists() {
        return Err(format!("MIDI file not found: {}", filepath));
    }

    // Read file bytes
    let bytes =
        fs::read(path).map_err(|e| format!("Failed to read MIDI file {}: {}", filepath, e))?;

    // Parse MIDI file
    let smf =
        Smf::parse(&bytes).map_err(|e| format!("Failed to parse MIDI file {}: {}", filepath, e))?;

    // Extract timing information
    let ticks_per_quarter = match smf.header.timing {
        Timing::Metrical(tpq) => tpq.as_int() as u32,
        Timing::Timecode(fps, sub) => {
            // Convert timecode to ticks per quarter note (approximate)
            let ticks_per_second = fps.as_f32() * sub as f32;
            (ticks_per_second * 0.5) as u32 // Assume 120 BPM default
        },
    };

    let format_num = match smf.header.format {
        midly::Format::SingleTrack => 0,
        midly::Format::Parallel => 1,
        midly::Format::Sequential => 2,
    };

    debug!(
        "Loaded MIDI file: {} (format {}, {} tracks, {} ticks/quarter)",
        filepath,
        format_num,
        smf.tracks.len(),
        ticks_per_quarter
    );

    // Parse events from all tracks
    let mut all_events = Vec::new();
    for (track_idx, track) in smf.tracks.iter().enumerate() {
        let track_events = parse_track_events(track, track_idx as u8, ticks_per_quarter)?;
        all_events.extend(track_events);
    }

    // Sort events by tick
    all_events.sort_by_key(|e| e.tick);

    Ok(LoadedMidiFile {
        events: all_events,
        ticks_per_quarter,
        _num_tracks: smf.tracks.len() as u16,
        _format: format_num,
    })
}

/// Loaded MIDI file with metadata
#[derive(Debug, Clone)]
pub struct LoadedMidiFile {
    pub events: Vec<MidiEvent>,
    pub ticks_per_quarter: u32,
    pub _num_tracks: u16,
    pub _format: u16,
}

impl LoadedMidiFile {
    /// Get ticks per quarter note for timing calculations
    pub fn ticks_per_quarter_note(&self) -> u32 {
        self.ticks_per_quarter
    }
}

/// Load and parse a MIDI file from raw bytes using the custom parser
///
/// This is useful for parsing MIDI data from memory (e.g., network, clipboard).
/// Uses our custom parser instead of midly for pure byte-level parsing.
///
/// # Arguments
/// * `data` - Raw MIDI file bytes
///
/// # Returns
/// Result containing LoadedMidiFile or error
pub fn load_midi_from_bytes(data: &[u8]) -> Result<LoadedMidiFile, String> {
    let pattern = parse_midi(data).map_err(|e: ParseError| format!("Parse error: {}", e))?;

    Ok(LoadedMidiFile {
        events: pattern.events,
        ticks_per_quarter: pattern.ticks_per_quarter_note as u32,
        _num_tracks: 1, // Custom parser merges all tracks
        _format: 0,
    })
}

/// Parse events from a single MIDI track
fn parse_track_events(
    track: &midly::Track,
    default_channel: u8,
    ticks_per_quarter: u32,
) -> Result<Vec<MidiEvent>, String> {
    let mut events = Vec::new();
    let mut absolute_tick: u64 = 0;
    let mut current_channel = default_channel;

    for event in track.iter() {
        // Update absolute tick position
        absolute_tick += event.delta.as_int() as u64;

        match event.kind {
            TrackEventKind::Midi { channel, message } => {
                current_channel = channel.as_int();

                if let Some(midi_event) =
                    convert_midi_message(message, current_channel, absolute_tick, ticks_per_quarter)
                {
                    events.push(midi_event);
                }
            },
            TrackEventKind::Meta(_) => {
                // Skip meta events for now (tempo, time signature, etc.)
                // These could be parsed in future for more accurate playback
            },
            TrackEventKind::SysEx(_) => {
                // Skip SysEx events
            },
            TrackEventKind::Escape(_) => {
                // Skip escape events
            },
        }
    }

    debug!(
        "Parsed {} events from track (channel {})",
        events.len(),
        current_channel
    );
    Ok(events)
}

/// Convert midly MIDI message to our MidiEvent format
fn convert_midi_message(
    message: MidlyMessage,
    channel: u8,
    tick: u64,
    __ticks_per_quarter: u32,
) -> Option<MidiEvent> {
    match message {
        MidlyMessage::NoteOff { key, vel } => Some(MidiEvent {
            event_type: MidiEventType::NoteOff,
            tick,
            channel,
            note: Some(key.as_int()),
            velocity: Some(vel.as_int()),
            controller: None,
            value: None,
            program: None,
        }),
        MidlyMessage::NoteOn { key, vel } => {
            // Note: velocity 0 should be treated as Note Off
            let event_type = if vel.as_int() == 0 {
                MidiEventType::NoteOff
            } else {
                MidiEventType::NoteOn
            };

            Some(MidiEvent {
                event_type,
                tick,
                channel,
                note: Some(key.as_int()),
                velocity: Some(vel.as_int()),
                controller: None,
                value: None,
                program: None,
            })
        },
        MidlyMessage::Aftertouch { key, vel } => Some(MidiEvent {
            event_type: MidiEventType::Aftertouch,
            tick,
            channel,
            note: Some(key.as_int()),
            value: Some(vel.as_int()),
            velocity: None,
            controller: None,
            program: None,
        }),
        MidlyMessage::Controller { controller, value } => Some(MidiEvent {
            event_type: MidiEventType::ControlChange,
            tick,
            channel,
            controller: Some(controller.as_int()),
            value: Some(value.as_int()),
            note: None,
            velocity: None,
            program: None,
        }),
        MidlyMessage::ProgramChange { program } => Some(MidiEvent {
            event_type: MidiEventType::ProgramChange,
            tick,
            channel,
            program: Some(program.as_int()),
            note: None,
            velocity: None,
            controller: None,
            value: None,
        }),
        MidlyMessage::ChannelAftertouch { vel } => Some(MidiEvent {
            event_type: MidiEventType::Aftertouch,
            tick,
            channel,
            value: Some(vel.as_int()),
            note: None,
            velocity: None,
            controller: None,
            program: None,
        }),
        MidlyMessage::PitchBend { bend } => {
            // Convert 14-bit pitch bend to two 7-bit values
            let bend_value = bend.as_int() as u16;
            Some(MidiEvent {
                event_type: MidiEventType::PitchBend,
                tick,
                channel,
                value: Some((bend_value & 0x7F) as u8), // LSB
                velocity: Some(((bend_value >> 7) & 0x7F) as u8), // MSB
                note: None,
                controller: None,
                program: None,
            })
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_nonexistent_file() {
        let result = load_midi_file("/nonexistent/path/file.mid");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_convert_note_on() {
        let message = MidlyMessage::NoteOn { key: 60.into(), vel: 100.into() };

        let event = convert_midi_message(message, 0, 0, 480).unwrap();
        assert_eq!(event.event_type, MidiEventType::NoteOn);
        assert_eq!(event.note, Some(60));
        assert_eq!(event.velocity, Some(100));
    }

    #[test]
    fn test_convert_note_on_zero_velocity() {
        let message = MidlyMessage::NoteOn { key: 60.into(), vel: 0.into() };

        let event = convert_midi_message(message, 0, 0, 480).unwrap();
        // Zero velocity Note On should become Note Off
        assert_eq!(event.event_type, MidiEventType::NoteOff);
    }

    #[test]
    fn test_convert_control_change() {
        let message = MidlyMessage::Controller {
            controller: 7.into(), // Volume
            value: 100.into(),
        };

        let event = convert_midi_message(message, 0, 0, 480).unwrap();
        assert_eq!(event.event_type, MidiEventType::ControlChange);
        assert_eq!(event.controller, Some(7));
        assert_eq!(event.value, Some(100));
    }
}

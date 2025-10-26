//! MIDI File Writer - Trusty Module
//!
//! Pure functions for writing MIDI files from event data structures.
//! NO I/O - caller receives bytes and writes to file.

use crate::models::midi::{MidiEvent, MidiEventType};

/// Write MIDI file from events
///
/// Pure function that generates MIDI file bytes from event data.
/// Takes events, timing resolution, and tempo as input.
///
/// # Arguments
///
/// * `events` - Slice of MIDI events to write
/// * `ticks_per_quarter` - Timing resolution (typically 480 or 960)
/// * `tempo_bpm` - Tempo in beats per minute (e.g., 120.0)
///
/// # Returns
///
/// * `Ok(Vec<u8>)` - Complete MIDI file as bytes
/// * `Err(String)` - Error message if generation fails
///
pub fn write_midi_file(
    events: &[MidiEvent],
    ticks_per_quarter: u16,
    tempo_bpm: f32,
) -> Result<Vec<u8>, String> {
    let mut data = Vec::new();

    // Write header (format 1, 1 track, ticks per quarter note)
    write_header(&mut data, 1, 1, ticks_per_quarter);

    // Write track with all events
    write_track(&mut data, events, tempo_bpm, ticks_per_quarter);

    Ok(data)
}

/// Write MIDI header chunk (MThd)
fn write_header(data: &mut Vec<u8>, format: u16, tracks: u16, tpqn: u16) {
    // Chunk type: "MThd"
    data.extend_from_slice(b"MThd");

    // Chunk length (always 6 for header)
    data.extend_from_slice(&6u32.to_be_bytes());

    // Format (0 = single track, 1 = multiple tracks, 2 = multiple sequences)
    data.extend_from_slice(&format.to_be_bytes());

    // Number of tracks
    data.extend_from_slice(&tracks.to_be_bytes());

    // Ticks per quarter note
    data.extend_from_slice(&tpqn.to_be_bytes());
}

/// Write MIDI track chunk (MTrk)
fn write_track(
    data: &mut Vec<u8>,
    events: &[MidiEvent],
    tempo_bpm: f32,
    _tpqn: u16,
) {
    let mut track_data = Vec::new();

    // Write tempo meta event at the start
    write_tempo_event(&mut track_data, tempo_bpm);

    // Write all MIDI events with delta times
    let mut last_tick = 0u64;
    for event in events {
        let delta = event.tick.saturating_sub(last_tick);
        write_variable_length(&mut track_data, delta);
        write_event(&mut track_data, event);
        last_tick = event.tick;
    }

    // End of track meta event
    write_variable_length(&mut track_data, 0);
    track_data.extend_from_slice(&[0xFF, 0x2F, 0x00]);

    // Write track header
    data.extend_from_slice(b"MTrk");
    data.extend_from_slice(&(track_data.len() as u32).to_be_bytes());
    data.extend_from_slice(&track_data);
}

/// Write tempo meta event
///
/// MIDI tempo is stored as microseconds per quarter note.
/// Conversion: microseconds_per_quarter = 60,000,000 / BPM
fn write_tempo_event(data: &mut Vec<u8>, bpm: f32) {
    let microseconds_per_quarter = (60_000_000.0 / bpm) as u32;

    write_variable_length(data, 0); // Delta time = 0 (at start)
    data.push(0xFF); // Meta event
    data.push(0x51); // Tempo meta event type
    data.push(0x03); // Length = 3 bytes

    // Write 3-byte tempo value (big-endian, skip first byte of u32)
    data.extend_from_slice(&microseconds_per_quarter.to_be_bytes()[1..4]);
}

/// Write single MIDI event
fn write_event(data: &mut Vec<u8>, event: &MidiEvent) {
    match event.event_type {
        MidiEventType::NoteOn => {
            data.push(0x90 | (event.channel & 0x0F));
            data.push(event.note.unwrap_or(0) & 0x7F);
            data.push(event.velocity.unwrap_or(100) & 0x7F);
        }
        MidiEventType::NoteOff => {
            data.push(0x80 | (event.channel & 0x0F));
            data.push(event.note.unwrap_or(0) & 0x7F);
            data.push(0x00); // Note off velocity is always 0
        }
        MidiEventType::ControlChange => {
            data.push(0xB0 | (event.channel & 0x0F));
            data.push(event.controller.unwrap_or(0) & 0x7F);
            data.push(event.value.unwrap_or(0) & 0x7F);
        }
        MidiEventType::ProgramChange => {
            data.push(0xC0 | (event.channel & 0x0F));
            data.push(event.program.unwrap_or(0) & 0x7F);
        }
        MidiEventType::PitchBend => {
            data.push(0xE0 | (event.channel & 0x0F));
            // Pitch bend is a 14-bit value (0-16383, center = 8192)
            // For now, we'll write a neutral pitch bend
            data.push(0x00); // LSB
            data.push(0x40); // MSB (64 = center)
        }
        MidiEventType::Aftertouch => {
            data.push(0xD0 | (event.channel & 0x0F));
            data.push(0x00); // Pressure value
        }
    }
}

/// Write variable-length quantity (MIDI standard encoding)
///
/// MIDI uses variable-length quantities to save space.
/// - Values 0-127: single byte (0xxxxxxx)
/// - Larger values: multiple bytes (1xxxxxxx 1xxxxxxx ... 0xxxxxxx)
/// - Most significant bit = 1 means "more bytes follow"
/// - Each byte contributes 7 bits to the value
///
/// Examples:
/// - 0 → [0x00]
/// - 127 → [0x7F]
/// - 128 → [0x81, 0x00]
/// - 8192 → [0xC0, 0x00]
fn write_variable_length(data: &mut Vec<u8>, mut value: u64) {
    let mut bytes = Vec::new();

    // Write least significant 7 bits (without continuation bit)
    bytes.push((value & 0x7F) as u8);
    value >>= 7;

    // Write remaining 7-bit groups (with continuation bit set)
    while value > 0 {
        bytes.push(((value & 0x7F) | 0x80) as u8);
        value >>= 7;
    }

    // Reverse to get big-endian order
    bytes.reverse();
    data.extend_from_slice(&bytes);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_variable_length_small() {
        let mut data = Vec::new();
        write_variable_length(&mut data, 0);
        assert_eq!(data, vec![0x00]);

        let mut data = Vec::new();
        write_variable_length(&mut data, 127);
        assert_eq!(data, vec![0x7F]);
    }

    #[test]
    fn test_write_variable_length_medium() {
        let mut data = Vec::new();
        write_variable_length(&mut data, 128);
        assert_eq!(data, vec![0x81, 0x00]);

        let mut data = Vec::new();
        write_variable_length(&mut data, 255);
        assert_eq!(data, vec![0x81, 0x7F]);
    }

    #[test]
    fn test_write_variable_length_large() {
        let mut data = Vec::new();
        write_variable_length(&mut data, 8192);
        assert_eq!(data, vec![0xC0, 0x00]);
    }

    #[test]
    fn test_write_simple_midi() {
        let events = vec![
            MidiEvent {
                event_type: MidiEventType::NoteOn,
                tick: 0,
                channel: 0,
                note: Some(60),
                velocity: Some(100),
                controller: None,
                value: None,
                program: None,
            },
            MidiEvent {
                event_type: MidiEventType::NoteOff,
                tick: 480,
                channel: 0,
                note: Some(60),
                velocity: Some(0),
                controller: None,
                value: None,
                program: None,
            },
        ];

        let result = write_midi_file(&events, 480, 120.0);
        assert!(result.is_ok());

        let data = result.unwrap();
        // Verify header
        assert_eq!(&data[0..4], b"MThd");
        // Verify track chunk exists
        assert!(data.windows(4).any(|w| w == b"MTrk"));
    }

    #[test]
    fn test_write_empty_events() {
        let events = vec![];
        let result = write_midi_file(&events, 480, 120.0);

        assert!(result.is_ok());
        let data = result.unwrap();

        // Should still have valid MIDI structure
        assert_eq!(&data[0..4], b"MThd");
        assert!(data.windows(4).any(|w| w == b"MTrk"));
    }

    #[test]
    fn test_write_control_change() {
        let events = vec![
            MidiEvent {
                event_type: MidiEventType::ControlChange,
                tick: 0,
                channel: 0,
                note: None,
                velocity: None,
                controller: Some(7), // Volume
                value: Some(100),
                program: None,
            },
        ];

        let result = write_midi_file(&events, 480, 120.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_multiple_channels() {
        let events = vec![
            MidiEvent {
                event_type: MidiEventType::NoteOn,
                tick: 0,
                channel: 0,
                note: Some(60),
                velocity: Some(100),
                controller: None,
                value: None,
                program: None,
            },
            MidiEvent {
                event_type: MidiEventType::NoteOn,
                tick: 0,
                channel: 9, // Typically drums
                note: Some(36), // Bass drum
                velocity: Some(120),
                controller: None,
                value: None,
                program: None,
            },
        ];

        let result = write_midi_file(&events, 480, 120.0);
        assert!(result.is_ok());
    }
}

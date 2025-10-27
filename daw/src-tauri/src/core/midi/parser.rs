//! MIDI File Parser - Trusty Module
//!
//! Pure functions for parsing MIDI files into data structures.
//! NO I/O - caller reads file and passes bytes.

use crate::models::midi::{MidiEvent, MidiEventType, MidiPattern};

/// Parse error types
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid MIDI file format: {0}")]
    InvalidFormat(String),

    #[error("Unsupported MIDI format: {0}")]
    UnsupportedFormat(String),

    #[error("Incomplete data at position {0}")]
    IncompleteData(usize),

    #[error("Invalid track data: {0}")]
    InvalidTrack(String),
}

/// Parse complete MIDI file
pub fn parse_midi(data: &[u8]) -> Result<MidiPattern, ParseError> {
    if data.is_empty() {
        return Err(ParseError::IncompleteData(0));
    }

    let mut reader = MidiReader::new(data);

    // Parse header
    let header = parse_header(&mut reader)?;

    // Parse tracks
    let mut all_events = Vec::new();

    for _ in 0..header.num_tracks {
        let track_events = parse_track(&mut reader)?;
        all_events.extend(track_events);
    }

    // Sort events by tick
    all_events.sort_by_key(|e| e.tick);

    let total_ticks = all_events.last().map(|e| e.tick).unwrap_or(0);

    Ok(MidiPattern {
        events: all_events,
        ticks_per_quarter_note: header.ticks_per_quarter_note,
        total_ticks,
    })
}

/// MIDI file header
#[derive(Debug)]
struct MidiHeader {
    num_tracks: u16,
    ticks_per_quarter_note: u16,
}

/// Parse MIDI header (MThd chunk)
fn parse_header(reader: &mut MidiReader) -> Result<MidiHeader, ParseError> {
    // Read "MThd"
    let chunk_type = reader.read_bytes(4)?;
    if chunk_type != b"MThd" {
        return Err(ParseError::InvalidFormat("Expected MThd header".to_string()));
    }

    // Read header length (should be 6)
    let length = reader.read_u32()?;
    if length != 6 {
        return Err(ParseError::InvalidFormat(format!(
            "Invalid header length: {}",
            length
        )));
    }

    // Read format (0, 1, or 2)
    let format = reader.read_u16()?;
    if format > 2 {
        return Err(ParseError::UnsupportedFormat(format!(
            "MIDI format {}",
            format
        )));
    }

    // Read number of tracks
    let num_tracks = reader.read_u16()?;

    // Read ticks per quarter note
    let ticks_per_quarter_note = reader.read_u16()?;

    Ok(MidiHeader {
        num_tracks,
        ticks_per_quarter_note,
    })
}

/// Parse a single track (MTrk chunk)
fn parse_track(reader: &mut MidiReader) -> Result<Vec<MidiEvent>, ParseError> {
    // Read "MTrk"
    let chunk_type = reader.read_bytes(4)?;
    if chunk_type != b"MTrk" {
        return Err(ParseError::InvalidTrack("Expected MTrk header".to_string()));
    }

    // Read track length
    let track_length = reader.read_u32()? as usize;
    let track_end = reader.position() + track_length;

    let mut events = Vec::new();
    let mut current_tick: u64 = 0;
    let mut running_status: Option<u8> = None;

    while reader.position() < track_end {
        // Read delta time (variable length)
        let delta_time = reader.read_variable_length()?;
        current_tick += delta_time;

        // Peek at next byte to determine if we need running status
        let status_byte = reader.peek_u8()?;

        let status = if status_byte & 0x80 == 0 {
            // Running status - reuse previous status
            running_status.ok_or(ParseError::InvalidTrack(
                "No running status available".to_string(),
            ))?
        } else {
            let s = reader.read_u8()?;
            if s != 0xFF && s != 0xF0 && s != 0xF7 {
                // Not a meta event or sysex, save as running status
                running_status = Some(s);
            }
            s
        };

        // Parse event based on status
        if status == 0xFF {
            // Meta event - skip it
            let _meta_type = reader.read_u8()?;
            let length = reader.read_variable_length()?;
            reader.skip(length as usize)?;
        } else if status == 0xF0 || status == 0xF7 {
            // SysEx event - skip it
            let length = reader.read_variable_length()?;
            reader.skip(length as usize)?;
        } else {
            // Channel event
            let command = status & 0xF0;
            let channel = status & 0x0F;

            let event = match command {
                0x90 => {
                    // Note On
                    let note = reader.read_u8()?;
                    let velocity = reader.read_u8()?;
                    Some(MidiEvent {
                        event_type: MidiEventType::NoteOn,
                        tick: current_tick,
                        channel,
                        note: Some(note),
                        velocity: Some(velocity),
                        controller: None,
                        value: None,
                        program: None,
                    })
                }
                0x80 => {
                    // Note Off
                    let note = reader.read_u8()?;
                    let _velocity = reader.read_u8()?;
                    Some(MidiEvent {
                        event_type: MidiEventType::NoteOff,
                        tick: current_tick,
                        channel,
                        note: Some(note),
                        velocity: Some(0),
                        controller: None,
                        value: None,
                        program: None,
                    })
                }
                0xB0 => {
                    // Control Change
                    let controller = reader.read_u8()?;
                    let value = reader.read_u8()?;
                    Some(MidiEvent {
                        event_type: MidiEventType::ControlChange,
                        tick: current_tick,
                        channel,
                        note: None,
                        velocity: None,
                        controller: Some(controller),
                        value: Some(value),
                        program: None,
                    })
                }
                0xC0 => {
                    // Program Change
                    let program = reader.read_u8()?;
                    Some(MidiEvent {
                        event_type: MidiEventType::ProgramChange,
                        tick: current_tick,
                        channel,
                        note: None,
                        velocity: None,
                        controller: None,
                        value: None,
                        program: Some(program),
                    })
                }
                0xE0 => {
                    // Pitch Bend
                    let _lsb = reader.read_u8()?;
                    let _msb = reader.read_u8()?;
                    Some(MidiEvent {
                        event_type: MidiEventType::PitchBend,
                        tick: current_tick,
                        channel,
                        note: None,
                        velocity: None,
                        controller: None,
                        value: None,
                        program: None,
                    })
                }
                0xD0 => {
                    // Aftertouch
                    let _value = reader.read_u8()?;
                    Some(MidiEvent {
                        event_type: MidiEventType::Aftertouch,
                        tick: current_tick,
                        channel,
                        note: None,
                        velocity: None,
                        controller: None,
                        value: None,
                        program: None,
                    })
                }
                _ => None,
            };

            if let Some(e) = event {
                events.push(e);
            }
        }
    }

    Ok(events)
}

/// Helper for reading MIDI binary data
struct MidiReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> MidiReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn position(&self) -> usize {
        self.pos
    }

    fn read_u8(&mut self) -> Result<u8, ParseError> {
        if self.pos >= self.data.len() {
            return Err(ParseError::IncompleteData(self.pos));
        }
        let value = self.data[self.pos];
        self.pos += 1;
        Ok(value)
    }

    fn peek_u8(&self) -> Result<u8, ParseError> {
        if self.pos >= self.data.len() {
            return Err(ParseError::IncompleteData(self.pos));
        }
        Ok(self.data[self.pos])
    }

    fn read_u16(&mut self) -> Result<u16, ParseError> {
        let b1 = self.read_u8()? as u16;
        let b2 = self.read_u8()? as u16;
        Ok((b1 << 8) | b2)
    }

    fn read_u32(&mut self) -> Result<u32, ParseError> {
        let b1 = self.read_u8()? as u32;
        let b2 = self.read_u8()? as u32;
        let b3 = self.read_u8()? as u32;
        let b4 = self.read_u8()? as u32;
        Ok((b1 << 24) | (b2 << 16) | (b3 << 8) | b4)
    }

    fn read_bytes(&mut self, count: usize) -> Result<&'a [u8], ParseError> {
        if self.pos + count > self.data.len() {
            return Err(ParseError::IncompleteData(self.pos));
        }
        let bytes = &self.data[self.pos..self.pos + count];
        self.pos += count;
        Ok(bytes)
    }

    fn read_variable_length(&mut self) -> Result<u64, ParseError> {
        let mut value: u64;
        let mut byte = self.read_u8()?;

        value = (byte & 0x7F) as u64;

        while byte & 0x80 != 0 {
            byte = self.read_u8()?;
            value = (value << 7) | ((byte & 0x7F) as u64);
        }

        Ok(value)
    }

    fn skip(&mut self, count: usize) -> Result<(), ParseError> {
        if self.pos + count > self.data.len() {
            return Err(ParseError::IncompleteData(self.pos));
        }
        self.pos += count;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // MidiReader Tests
    // ============================================================================

    #[test]
    fn test_midi_reader_read_u8_success() {
        let data = vec![0x4D, 0x54, 0x68];
        let mut reader = MidiReader::new(&data);

        assert_eq!(reader.read_u8().unwrap(), 0x4D);
        assert_eq!(reader.position(), 1);
        assert_eq!(reader.read_u8().unwrap(), 0x54);
        assert_eq!(reader.position(), 2);
    }

    #[test]
    fn test_midi_reader_read_u8_incomplete() {
        let data = vec![0x4D];
        let mut reader = MidiReader::new(&data);

        reader.read_u8().unwrap();
        let result = reader.read_u8();
        assert!(matches!(result, Err(ParseError::IncompleteData(1))));
    }

    #[test]
    fn test_midi_reader_peek_u8() {
        let data = vec![0x4D, 0x54];
        let reader = MidiReader::new(&data);

        // Peek doesn't advance position
        assert_eq!(reader.peek_u8().unwrap(), 0x4D);
        assert_eq!(reader.position(), 0);
        assert_eq!(reader.peek_u8().unwrap(), 0x4D);
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn test_midi_reader_read_u16_big_endian() {
        let data = vec![0x12, 0x34, 0xAB, 0xCD];
        let mut reader = MidiReader::new(&data);

        assert_eq!(reader.read_u16().unwrap(), 0x1234);
        assert_eq!(reader.read_u16().unwrap(), 0xABCD);
    }

    #[test]
    fn test_midi_reader_read_u32_big_endian() {
        let data = vec![0x12, 0x34, 0x56, 0x78];
        let mut reader = MidiReader::new(&data);

        assert_eq!(reader.read_u32().unwrap(), 0x12345678);
    }

    #[test]
    fn test_midi_reader_read_bytes() {
        let data = vec![0x4D, 0x54, 0x68, 0x64];
        let mut reader = MidiReader::new(&data);

        let bytes = reader.read_bytes(4).unwrap();
        assert_eq!(bytes, b"MThd");
        assert_eq!(reader.position(), 4);
    }

    #[test]
    fn test_midi_reader_read_variable_length_one_byte() {
        let data = vec![0x00, 0x40, 0x7F];
        let mut reader = MidiReader::new(&data);

        assert_eq!(reader.read_variable_length().unwrap(), 0);
        assert_eq!(reader.read_variable_length().unwrap(), 64);
        assert_eq!(reader.read_variable_length().unwrap(), 127);
    }

    #[test]
    fn test_midi_reader_read_variable_length_two_bytes() {
        let data = vec![0x81, 0x00, 0xFF, 0x7F];
        let mut reader = MidiReader::new(&data);

        // 0x81 0x00 = (0x01 << 7) | 0x00 = 128
        assert_eq!(reader.read_variable_length().unwrap(), 128);
        // 0xFF 0x7F = (0x7F << 7) | 0x7F = 16383
        assert_eq!(reader.read_variable_length().unwrap(), 16383);
    }

    #[test]
    fn test_midi_reader_read_variable_length_three_bytes() {
        let data = vec![0x81, 0x80, 0x00];
        let mut reader = MidiReader::new(&data);

        // 0x81 0x80 0x00 = (0x01 << 14) | (0x00 << 7) | 0x00 = 16384
        assert_eq!(reader.read_variable_length().unwrap(), 16384);
    }

    #[test]
    fn test_midi_reader_read_variable_length_four_bytes() {
        let data = vec![0x81, 0x80, 0x80, 0x00];
        let mut reader = MidiReader::new(&data);

        // 0x81 0x80 0x80 0x00 = (0x01 << 21) | (0x00 << 14) | (0x00 << 7) | 0x00 = 2097152
        assert_eq!(reader.read_variable_length().unwrap(), 2097152);
    }

    #[test]
    fn test_midi_reader_skip() {
        let data = vec![0x00, 0x01, 0x02, 0x03, 0x04];
        let mut reader = MidiReader::new(&data);

        reader.skip(2).unwrap();
        assert_eq!(reader.position(), 2);
        assert_eq!(reader.read_u8().unwrap(), 0x02);
    }

    #[test]
    fn test_midi_reader_skip_incomplete() {
        let data = vec![0x00, 0x01];
        let mut reader = MidiReader::new(&data);

        let result = reader.skip(10);
        assert!(matches!(result, Err(ParseError::IncompleteData(0))));
    }

    // ============================================================================
    // Header Parsing Tests
    // ============================================================================

    fn make_valid_header(format: u16, num_tracks: u16, ticks: u16) -> Vec<u8> {
        let mut data = vec![];
        data.extend_from_slice(b"MThd"); // Chunk type
        data.extend_from_slice(&6u32.to_be_bytes()); // Length
        data.extend_from_slice(&format.to_be_bytes()); // Format
        data.extend_from_slice(&num_tracks.to_be_bytes()); // Tracks
        data.extend_from_slice(&ticks.to_be_bytes()); // TPQN
        data
    }

    #[test]
    fn test_parse_header_format_0() {
        let data = make_valid_header(0, 1, 480);
        let mut reader = MidiReader::new(&data);

        let header = parse_header(&mut reader).unwrap();
        assert_eq!(header.num_tracks, 1);
        assert_eq!(header.ticks_per_quarter_note, 480);
    }

    #[test]
    fn test_parse_header_format_1() {
        let data = make_valid_header(1, 4, 960);
        let mut reader = MidiReader::new(&data);

        let header = parse_header(&mut reader).unwrap();
        assert_eq!(header.num_tracks, 4);
        assert_eq!(header.ticks_per_quarter_note, 960);
    }

    #[test]
    fn test_parse_header_format_2() {
        let data = make_valid_header(2, 8, 240);
        let mut reader = MidiReader::new(&data);

        let header = parse_header(&mut reader).unwrap();
        assert_eq!(header.num_tracks, 8);
        assert_eq!(header.ticks_per_quarter_note, 240);
    }

    #[test]
    fn test_parse_header_invalid_chunk_type() {
        let mut data = vec![];
        data.extend_from_slice(b"XXXX"); // Invalid
        data.extend_from_slice(&6u32.to_be_bytes());

        let mut reader = MidiReader::new(&data);
        let result = parse_header(&mut reader);

        assert!(matches!(result, Err(ParseError::InvalidFormat(_))));
        if let Err(ParseError::InvalidFormat(msg)) = result {
            assert!(msg.contains("MThd"));
        }
    }

    #[test]
    fn test_parse_header_invalid_length() {
        let mut data = vec![];
        data.extend_from_slice(b"MThd");
        data.extend_from_slice(&10u32.to_be_bytes()); // Should be 6
        data.extend_from_slice(&1u16.to_be_bytes());
        data.extend_from_slice(&1u16.to_be_bytes());
        data.extend_from_slice(&480u16.to_be_bytes());

        let mut reader = MidiReader::new(&data);
        let result = parse_header(&mut reader);

        assert!(matches!(result, Err(ParseError::InvalidFormat(_))));
    }

    #[test]
    fn test_parse_header_unsupported_format() {
        let data = make_valid_header(3, 1, 480);
        let mut reader = MidiReader::new(&data);

        let result = parse_header(&mut reader);
        assert!(matches!(result, Err(ParseError::UnsupportedFormat(_))));
    }

    #[test]
    fn test_parse_header_incomplete_data() {
        let data = vec![0x4D, 0x54]; // Only "MT"
        let mut reader = MidiReader::new(&data);

        let result = parse_header(&mut reader);
        assert!(matches!(result, Err(ParseError::IncompleteData(_))));
    }

    // ============================================================================
    // Track Parsing Tests
    // ============================================================================

    fn make_track_header(length: u32) -> Vec<u8> {
        let mut data = vec![];
        data.extend_from_slice(b"MTrk");
        data.extend_from_slice(&length.to_be_bytes());
        data
    }

    #[test]
    fn test_parse_track_single_note_on() {
        let mut data = make_track_header(4);
        data.extend_from_slice(&[0x00, 0x90, 0x3C, 0x64]); // Delta=0, NoteOn C4, vel=100

        let mut reader = MidiReader::new(&data);
        let events = parse_track(&mut reader).unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, MidiEventType::NoteOn);
        assert_eq!(events[0].tick, 0);
        assert_eq!(events[0].channel, 0);
        assert_eq!(events[0].note, Some(0x3C));
        assert_eq!(events[0].velocity, Some(0x64));
    }

    #[test]
    fn test_parse_track_note_off() {
        let mut data = make_track_header(4);
        data.extend_from_slice(&[0x00, 0x80, 0x3C, 0x00]); // NoteOff C4

        let mut reader = MidiReader::new(&data);
        let events = parse_track(&mut reader).unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, MidiEventType::NoteOff);
        assert_eq!(events[0].note, Some(0x3C));
        assert_eq!(events[0].velocity, Some(0));
    }

    #[test]
    fn test_parse_track_control_change() {
        let mut data = make_track_header(4);
        data.extend_from_slice(&[0x00, 0xB0, 0x07, 0x7F]); // CC#7 (volume) = 127

        let mut reader = MidiReader::new(&data);
        let events = parse_track(&mut reader).unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, MidiEventType::ControlChange);
        assert_eq!(events[0].controller, Some(0x07));
        assert_eq!(events[0].value, Some(0x7F));
    }

    #[test]
    fn test_parse_track_program_change() {
        let mut data = make_track_header(3);
        data.extend_from_slice(&[0x00, 0xC0, 0x05]); // Program #5

        let mut reader = MidiReader::new(&data);
        let events = parse_track(&mut reader).unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, MidiEventType::ProgramChange);
        assert_eq!(events[0].program, Some(0x05));
    }

    #[test]
    fn test_parse_track_pitch_bend() {
        let mut data = make_track_header(4);
        data.extend_from_slice(&[0x00, 0xE0, 0x00, 0x40]); // Pitch bend

        let mut reader = MidiReader::new(&data);
        let events = parse_track(&mut reader).unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, MidiEventType::PitchBend);
    }

    #[test]
    fn test_parse_track_aftertouch() {
        let mut data = make_track_header(3);
        data.extend_from_slice(&[0x00, 0xD0, 0x40]); // Aftertouch

        let mut reader = MidiReader::new(&data);
        let events = parse_track(&mut reader).unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, MidiEventType::Aftertouch);
    }

    #[test]
    fn test_parse_track_running_status() {
        let mut data = make_track_header(10);
        // NoteOn C4, then NoteOn D4 using running status
        data.extend_from_slice(&[0x00, 0x90, 0x3C, 0x64]); // Delta=0, NoteOn C4
        data.extend_from_slice(&[0x10, 0x3E, 0x64]); // Delta=16, D4 (running status)
        data.extend_from_slice(&[0x10, 0x40, 0x64]); // Delta=16, E4 (running status)

        let mut reader = MidiReader::new(&data);
        let events = parse_track(&mut reader).unwrap();

        assert_eq!(events.len(), 3);
        assert_eq!(events[0].tick, 0);
        assert_eq!(events[0].note, Some(0x3C));
        assert_eq!(events[1].tick, 16);
        assert_eq!(events[1].note, Some(0x3E));
        assert_eq!(events[2].tick, 32);
        assert_eq!(events[2].note, Some(0x40));
    }

    #[test]
    fn test_parse_track_meta_event_skipped() {
        let mut data = make_track_header(9);
        // Meta event (tempo): FF 51 03 07 A1 20
        data.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20]);
        // NoteOn after meta
        data.extend_from_slice(&[0x00, 0x90, 0x3C, 0x64]);

        let mut reader = MidiReader::new(&data);
        let events = parse_track(&mut reader).unwrap();

        // Meta event should be skipped, only NoteOn should appear
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, MidiEventType::NoteOn);
    }

    #[test]
    fn test_parse_track_sysex_event_skipped() {
        let mut data = make_track_header(8);
        // SysEx: F0 03 43 12 00
        data.extend_from_slice(&[0x00, 0xF0, 0x03, 0x43, 0x12, 0x00]);
        // NoteOn after SysEx
        data.extend_from_slice(&[0x00, 0x90, 0x3C, 0x64]);

        let mut reader = MidiReader::new(&data);
        let events = parse_track(&mut reader).unwrap();

        // SysEx should be skipped, only NoteOn should appear
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, MidiEventType::NoteOn);
    }

    #[test]
    fn test_parse_track_delta_time_accumulation() {
        let mut data = make_track_header(8);
        data.extend_from_slice(&[0x00, 0x90, 0x3C, 0x64]); // Delta=0, tick=0
        data.extend_from_slice(&[0x10, 0x80, 0x3C, 0x00]); // Delta=16, tick=16

        let mut reader = MidiReader::new(&data);
        let events = parse_track(&mut reader).unwrap();

        assert_eq!(events[0].tick, 0);
        assert_eq!(events[1].tick, 16);
    }

    #[test]
    fn test_parse_track_invalid_header() {
        let mut data = vec![];
        data.extend_from_slice(b"XXXX");
        data.extend_from_slice(&0u32.to_be_bytes());

        let mut reader = MidiReader::new(&data);
        let result = parse_track(&mut reader);

        assert!(matches!(result, Err(ParseError::InvalidTrack(_))));
    }

    #[test]
    fn test_parse_track_missing_running_status() {
        let mut data = make_track_header(3);
        // Data byte without status byte and no running status
        data.extend_from_slice(&[0x00, 0x3C, 0x64]); // Invalid: no status

        let mut reader = MidiReader::new(&data);
        let result = parse_track(&mut reader);

        assert!(matches!(result, Err(ParseError::InvalidTrack(_))));
    }

    #[test]
    fn test_parse_track_multiple_channels() {
        let mut data = make_track_header(12);
        data.extend_from_slice(&[0x00, 0x90, 0x3C, 0x64]); // Ch 0
        data.extend_from_slice(&[0x00, 0x91, 0x3E, 0x64]); // Ch 1
        data.extend_from_slice(&[0x00, 0x9F, 0x40, 0x64]); // Ch 15

        let mut reader = MidiReader::new(&data);
        let events = parse_track(&mut reader).unwrap();

        assert_eq!(events.len(), 3);
        assert_eq!(events[0].channel, 0);
        assert_eq!(events[1].channel, 1);
        assert_eq!(events[2].channel, 15);
    }

    // ============================================================================
    // Full File Parsing Tests
    // ============================================================================

    fn make_simple_midi_file() -> Vec<u8> {
        let mut data = vec![];

        // Header: Format 1, 1 track, 480 TPQN
        data.extend_from_slice(&make_valid_header(1, 1, 480));

        // Track with 2 notes
        let mut track = vec![];
        track.extend_from_slice(&[0x00, 0x90, 0x3C, 0x64]); // NoteOn C4 at tick 0
        track.extend_from_slice(&[0x60, 0x80, 0x3C, 0x00]); // NoteOff C4 at tick 96

        data.extend_from_slice(&make_track_header(track.len() as u32));
        data.extend_from_slice(&track);

        data
    }

    #[test]
    fn test_parse_midi_simple_file() {
        let data = make_simple_midi_file();
        let pattern = parse_midi(&data).unwrap();

        assert_eq!(pattern.ticks_per_quarter_note, 480);
        assert_eq!(pattern.events.len(), 2);
        assert_eq!(pattern.total_ticks, 96);
        assert_eq!(pattern.events[0].event_type, MidiEventType::NoteOn);
        assert_eq!(pattern.events[1].event_type, MidiEventType::NoteOff);
    }

    #[test]
    fn test_parse_midi_multi_track() {
        let mut data = vec![];

        // Header: Format 1, 2 tracks, 480 TPQN
        data.extend_from_slice(&make_valid_header(1, 2, 480));

        // Track 1
        let mut track1 = vec![];
        track1.extend_from_slice(&[0x00, 0x90, 0x3C, 0x64]); // Tick 0
        data.extend_from_slice(&make_track_header(track1.len() as u32));
        data.extend_from_slice(&track1);

        // Track 2
        let mut track2 = vec![];
        track2.extend_from_slice(&[0x10, 0x90, 0x40, 0x64]); // Tick 16
        data.extend_from_slice(&make_track_header(track2.len() as u32));
        data.extend_from_slice(&track2);

        let pattern = parse_midi(&data).unwrap();

        assert_eq!(pattern.events.len(), 2);
        // Events should be sorted by tick
        assert_eq!(pattern.events[0].tick, 0);
        assert_eq!(pattern.events[1].tick, 16);
    }

    #[test]
    fn test_parse_midi_events_sorted_by_tick() {
        let mut data = vec![];
        data.extend_from_slice(&make_valid_header(1, 2, 480));

        // Track 1 - event at tick 100
        let mut track1 = vec![];
        track1.extend_from_slice(&[0x64, 0x90, 0x3C, 0x64]); // Tick 100
        data.extend_from_slice(&make_track_header(track1.len() as u32));
        data.extend_from_slice(&track1);

        // Track 2 - event at tick 50
        let mut track2 = vec![];
        track2.extend_from_slice(&[0x32, 0x90, 0x40, 0x64]); // Tick 50
        data.extend_from_slice(&make_track_header(track2.len() as u32));
        data.extend_from_slice(&track2);

        let pattern = parse_midi(&data).unwrap();

        // Events should be sorted: tick 50 before tick 100
        assert_eq!(pattern.events[0].tick, 50);
        assert_eq!(pattern.events[0].note, Some(0x40));
        assert_eq!(pattern.events[1].tick, 100);
        assert_eq!(pattern.events[1].note, Some(0x3C));
    }

    #[test]
    fn test_parse_midi_empty_file() {
        let data = vec![];
        let result = parse_midi(&data);

        assert!(matches!(result, Err(ParseError::IncompleteData(0))));
    }

    #[test]
    fn test_parse_midi_incomplete_header() {
        let data = vec![0x4D, 0x54]; // Just "MT"
        let result = parse_midi(&data);

        assert!(matches!(result, Err(ParseError::IncompleteData(_))));
    }

    #[test]
    fn test_parse_midi_total_ticks_from_last_event() {
        let mut data = vec![];
        data.extend_from_slice(&make_valid_header(1, 1, 480));

        let mut track = vec![];
        track.extend_from_slice(&[0x00, 0x90, 0x3C, 0x64]); // Tick 0
        track.extend_from_slice(&[0x81, 0x00, 0x80, 0x3C, 0x00]); // Tick 128 (VLQ)

        data.extend_from_slice(&make_track_header(track.len() as u32));
        data.extend_from_slice(&track);

        let pattern = parse_midi(&data).unwrap();
        assert_eq!(pattern.total_ticks, 128);
    }

    #[test]
    fn test_parse_midi_empty_tracks() {
        let mut data = vec![];
        data.extend_from_slice(&make_valid_header(1, 1, 480));

        // Empty track
        data.extend_from_slice(&make_track_header(0));

        let pattern = parse_midi(&data).unwrap();
        assert_eq!(pattern.events.len(), 0);
        assert_eq!(pattern.total_ticks, 0); // No events, so total_ticks = 0
    }

    // ============================================================================
    // Edge Cases
    // ============================================================================

    #[test]
    fn test_parse_track_with_only_meta_events() {
        let mut data = make_track_header(7);
        // Only meta events, no channel events
        data.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20]); // Tempo

        let mut reader = MidiReader::new(&data);
        let events = parse_track(&mut reader).unwrap();

        // All meta events skipped
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_parse_track_large_delta_time() {
        let mut data = make_track_header(6);
        // Large VLQ delta: 0x82 0x80 0x00 = (2 << 14) | (0 << 7) | 0 = 32768
        data.extend_from_slice(&[0x82, 0x80, 0x00, 0x90, 0x3C, 0x64]);

        let mut reader = MidiReader::new(&data);
        let events = parse_track(&mut reader).unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].tick, 32768);
    }

    #[test]
    fn test_parse_track_unknown_channel_event_ignored() {
        let mut data = make_track_header(7);
        // 0xA0 = Polyphonic Key Pressure (not supported)
        data.extend_from_slice(&[0x00, 0xA0, 0x3C, 0x40]);
        // Valid NoteOn after
        data.extend_from_slice(&[0x00, 0x90, 0x3C, 0x64]);

        let mut reader = MidiReader::new(&data);
        let events = parse_track(&mut reader).unwrap();

        // Unknown event ignored, only NoteOn should appear
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, MidiEventType::NoteOn);
    }

    #[test]
    fn test_midi_reader_position_tracking() {
        let data = vec![0x00, 0x01, 0x02, 0x03, 0x04];
        let mut reader = MidiReader::new(&data);

        assert_eq!(reader.position(), 0);
        reader.read_u8().unwrap();
        assert_eq!(reader.position(), 1);
        reader.read_u16().unwrap();
        assert_eq!(reader.position(), 3);
        reader.skip(2).unwrap();
        assert_eq!(reader.position(), 5);
    }
}

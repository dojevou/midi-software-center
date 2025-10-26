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

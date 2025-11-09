use super::error::{MidiParseError, Result};
use super::types::*;

/// Parse a MIDI file from raw bytes
///
/// This is the main entry point for MIDI parsing. It accepts raw file bytes
/// and returns a structured MidiFile or an error.
///
/// # Examples
/// ```ignore
/// use midi_library_shared::core::midi::parse_midi_file;
///
/// let data = std::fs::read("song.mid").unwrap();
/// let midi_file = parse_midi_file(&data)?;
/// println!("Format: {}, Tracks: {}", midi_file.header.format, midi_file.header.num_tracks);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn parse_midi_file(data: &[u8]) -> Result<MidiFile> {
    if data.len() < 14 {
        return Err(MidiParseError::IncompleteData {
            expected: 14,
            actual: data.len(),
        });
    }

    // Parse header chunk
    let header = parse_header(&data[0..14])?;

    // Parse tracks
    let mut tracks = Vec::with_capacity(header.num_tracks as usize);
    let mut pos = 14;

    for track_num in 0..header.num_tracks {
        let (track, bytes_read) = parse_track(&data[pos..]).map_err(|e| match e {
            MidiParseError::InvalidTrack { position, reason } => MidiParseError::InvalidTrack {
                position: pos + position,
                reason: format!("Track {}: {}", track_num, reason),
            },
            e => e,
        })?;

        tracks.push(track);
        pos += bytes_read;
    }

    Ok(MidiFile { header, tracks })
}

/// Parse MIDI header chunk (MThd)
fn parse_header(data: &[u8]) -> Result<Header> {
    // Check magic number "MThd"
    if &data[0..4] != b"MThd" {
        return Err(MidiParseError::InvalidHeader(format!(
            "Expected 'MThd', got {:?}",
            &data[0..4]
        )));
    }

    // Check header length (must be 6)
    let length = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    if length != 6 {
        return Err(MidiParseError::InvalidHeader(format!(
            "Expected header length 6, got {}",
            length
        )));
    }

    let format = u16::from_be_bytes([data[8], data[9]]);
    let num_tracks = u16::from_be_bytes([data[10], data[11]]);
    let ticks_per_quarter_note = u16::from_be_bytes([data[12], data[13]]);

    // Validate format
    if format > 2 {
        return Err(MidiParseError::UnsupportedFormat(format));
    }

    Ok(Header {
        format,
        num_tracks,
        ticks_per_quarter_note,
    })
}

/// Parse a single MIDI track (MTrk)
/// Returns (Track, bytes_consumed)
fn parse_track(data: &[u8]) -> Result<(Track, usize)> {
    if data.len() < 8 {
        return Err(MidiParseError::InvalidTrack {
            position: 0,
            reason: "Track too short".to_string(),
        });
    }

    // Check magic number "MTrk"
    if &data[0..4] != b"MTrk" {
        return Err(MidiParseError::InvalidTrack {
            position: 0,
            reason: format!("Expected 'MTrk', got {:?}", &data[0..4]),
        });
    }

    let track_length = u32::from_be_bytes([data[4], data[5], data[6], data[7]]) as usize;

    if data.len() < 8 + track_length {
        return Err(MidiParseError::InvalidTrack {
            position: 0,
            reason: format!(
                "Track data incomplete: expected {} bytes, got {}",
                track_length,
                data.len() - 8
            ),
        });
    }

    let track_data = &data[8..8 + track_length];
    let events = parse_track_events(track_data)?;

    Ok((Track { events }, 8 + track_length))
}

/// Parse all events within a track
fn parse_track_events(data: &[u8]) -> Result<Vec<TimedEvent>> {
    let mut events = Vec::new();
    let mut pos = 0;
    let mut running_status: Option<u8> = None;

    while pos < data.len() {
        // Parse delta time (variable-length quantity)
        let (delta_ticks, delta_bytes) =
            read_var_len(&data[pos..]).ok_or(MidiParseError::InvalidVarLen(pos))?;
        pos += delta_bytes;

        // Parse event
        let (event, event_bytes, new_running_status) = parse_event(&data[pos..], running_status)
            .map_err(|e| match e {
                MidiParseError::InvalidEvent { position, reason } => MidiParseError::InvalidEvent {
                    position: pos + position,
                    reason,
                },
                e => e,
            })?;

        pos += event_bytes;
        running_status = new_running_status;

        events.push(TimedEvent { delta_ticks, event });

        // End of track?
        if matches!(
            events.last(),
            Some(TimedEvent {
                event: Event::EndOfTrack,
                ..
            })
        ) {
            break;
        }
    }

    Ok(events)
}

/// Parse a single MIDI event
/// Returns (Event, bytes_consumed, new_running_status)
fn parse_event(data: &[u8], running_status: Option<u8>) -> Result<(Event, usize, Option<u8>)> {
    if data.is_empty() {
        return Err(MidiParseError::InvalidEvent {
            position: 0,
            reason: "No data for event".to_string(),
        });
    }

    let mut status = data[0];
    let mut pos = 1;

    // Handle running status (reuse previous status byte if data byte encountered)
    if status < 0x80 {
        if let Some(rs) = running_status {
            status = rs;
            pos = 0; // Don't consume the byte, it's data
        } else {
            return Err(MidiParseError::InvalidEvent {
                position: 0,
                reason: "Data byte without running status".to_string(),
            });
        }
    }

    let event_type = status & 0xF0;
    let channel = status & 0x0F;

    match event_type {
        0x80 => {
            // Note Off
            if data.len() < pos + 2 {
                return Err(MidiParseError::IncompleteData {
                    expected: pos + 2,
                    actual: data.len(),
                });
            }
            Ok((
                Event::NoteOff {
                    channel,
                    note: data[pos],
                    velocity: data[pos + 1],
                },
                pos + 2,
                Some(status),
            ))
        }
        0x90 => {
            // Note On
            if data.len() < pos + 2 {
                return Err(MidiParseError::IncompleteData {
                    expected: pos + 2,
                    actual: data.len(),
                });
            }
            Ok((
                Event::NoteOn {
                    channel,
                    note: data[pos],
                    velocity: data[pos + 1],
                },
                pos + 2,
                Some(status),
            ))
        }
        0xA0 => {
            // Polyphonic Aftertouch
            if data.len() < pos + 2 {
                return Err(MidiParseError::IncompleteData {
                    expected: pos + 2,
                    actual: data.len(),
                });
            }
            Ok((
                Event::Aftertouch {
                    channel,
                    note: data[pos],
                    pressure: data[pos + 1],
                },
                pos + 2,
                Some(status),
            ))
        }
        0xB0 => {
            // Control Change
            if data.len() < pos + 2 {
                return Err(MidiParseError::IncompleteData {
                    expected: pos + 2,
                    actual: data.len(),
                });
            }
            Ok((
                Event::ControlChange {
                    channel,
                    controller: data[pos],
                    value: data[pos + 1],
                },
                pos + 2,
                Some(status),
            ))
        }
        0xC0 => {
            // Program Change
            if data.len() < pos + 1 {
                return Err(MidiParseError::IncompleteData {
                    expected: pos + 1,
                    actual: data.len(),
                });
            }
            Ok((
                Event::ProgramChange {
                    channel,
                    program: data[pos],
                },
                pos + 1,
                Some(status),
            ))
        }
        0xD0 => {
            // Channel Aftertouch
            if data.len() < pos + 1 {
                return Err(MidiParseError::IncompleteData {
                    expected: pos + 1,
                    actual: data.len(),
                });
            }
            Ok((
                Event::ChannelAftertouch {
                    channel,
                    pressure: data[pos],
                },
                pos + 1,
                Some(status),
            ))
        }
        0xE0 => {
            // Pitch Bend
            if data.len() < pos + 2 {
                return Err(MidiParseError::IncompleteData {
                    expected: pos + 2,
                    actual: data.len(),
                });
            }
            let lsb = data[pos] as i16;
            let msb = data[pos + 1] as i16;
            let value = ((msb << 7) | lsb) - 8192; // Center at 0
            Ok((Event::PitchBend { channel, value }, pos + 2, Some(status)))
        }
        0xF0 => {
            // System/Meta events
            parse_meta_or_sysex(&data[pos - 1..])
        }
        _ => Err(MidiParseError::InvalidEvent {
            position: 0,
            reason: format!("Unknown event type: 0x{:02X}", status),
        }),
    }
}

/// Parse meta events and SysEx
fn parse_meta_or_sysex(data: &[u8]) -> Result<(Event, usize, Option<u8>)> {
    let status = data[0];

    match status {
        0xFF => {
            // Meta event
            if data.len() < 2 {
                return Err(MidiParseError::IncompleteData {
                    expected: 2,
                    actual: data.len(),
                });
            }

            let meta_type = data[1];
            let (length, len_bytes) =
                read_var_len(&data[2..]).ok_or(MidiParseError::InvalidVarLen(2))?;

            let data_start = 2 + len_bytes;
            let data_end = data_start + length as usize;

            if data.len() < data_end {
                return Err(MidiParseError::IncompleteData {
                    expected: data_end,
                    actual: data.len(),
                });
            }

            let event_data = &data[data_start..data_end];

            let event = match meta_type {
                0x2F => Event::EndOfTrack,
                0x51 => {
                    if event_data.len() != 3 {
                        return Err(MidiParseError::InvalidEvent {
                            position: 0,
                            reason: "Tempo event must be 3 bytes".to_string(),
                        });
                    }
                    let microseconds_per_quarter =
                        u32::from_be_bytes([0, event_data[0], event_data[1], event_data[2]]);
                    Event::TempoChange {
                        microseconds_per_quarter,
                    }
                }
                0x58 => {
                    if event_data.len() != 4 {
                        return Err(MidiParseError::InvalidEvent {
                            position: 0,
                            reason: "Time signature event must be 4 bytes".to_string(),
                        });
                    }
                    Event::TimeSignature {
                        numerator: event_data[0],
                        denominator: event_data[1],
                        clocks_per_click: event_data[2],
                        thirty_seconds_per_quarter: event_data[3],
                    }
                }
                0x59 => {
                    if event_data.len() != 2 {
                        return Err(MidiParseError::InvalidEvent {
                            position: 0,
                            reason: "Key signature event must be 2 bytes".to_string(),
                        });
                    }
                    Event::KeySignature {
                        sharps_flats: event_data[0] as i8,
                        is_minor: event_data[1] != 0,
                    }
                }
                0x01..=0x0F => {
                    // Text events
                    let text = String::from_utf8(event_data.to_vec())?;
                    let text_type = match meta_type {
                        0x01 => TextType::Text,
                        0x02 => TextType::Copyright,
                        0x03 => TextType::TrackName,
                        0x04 => TextType::InstrumentName,
                        0x05 => TextType::Lyric,
                        0x06 => TextType::Marker,
                        0x07 => TextType::CuePoint,
                        _ => TextType::Text,
                    };
                    Event::Text { text_type, text }
                }
                _ => Event::Unknown {
                    status,
                    data: event_data.to_vec(),
                },
            };

            Ok((event, data_end, None)) // Meta events don't have running status
        }
        0xF0 | 0xF7 => {
            // SysEx
            let (length, len_bytes) =
                read_var_len(&data[1..]).ok_or(MidiParseError::InvalidVarLen(1))?;

            let data_start = 1 + len_bytes;
            let data_end = data_start + length as usize;

            if data.len() < data_end {
                return Err(MidiParseError::IncompleteData {
                    expected: data_end,
                    actual: data.len(),
                });
            }

            Ok((
                Event::SysEx {
                    data: data[data_start..data_end].to_vec(),
                },
                data_end,
                None, // SysEx doesn't have running status
            ))
        }
        _ => Err(MidiParseError::InvalidEvent {
            position: 0,
            reason: format!("Unknown system event: 0x{:02X}", status),
        }),
    }
}

/// Read a MIDI variable-length quantity
/// Returns (value, bytes_consumed) or None if invalid
fn read_var_len(data: &[u8]) -> Option<(u32, usize)> {
    let mut value = 0u32;
    let mut bytes_read = 0;

    for (i, &byte) in data.iter().enumerate() {
        if i >= 4 {
            // Variable length can be at most 4 bytes
            return None;
        }

        value = (value << 7) | (byte & 0x7F) as u32;
        bytes_read += 1;

        // If high bit is clear, we're done
        if byte & 0x80 == 0 {
            return Some((value, bytes_read));
        }
    }

    None // Ran out of data before finding end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_var_len() {
        // Single byte
        assert_eq!(read_var_len(&[0x00]), Some((0, 1)));
        assert_eq!(read_var_len(&[0x7F]), Some((127, 1)));

        // Two bytes
        assert_eq!(read_var_len(&[0x81, 0x00]), Some((128, 2)));
        assert_eq!(read_var_len(&[0xFF, 0x7F]), Some((16383, 2)));

        // Invalid (no terminating byte)
        assert_eq!(read_var_len(&[0x81, 0x82, 0x83, 0x84]), None);
    }

    #[test]
    fn test_parse_header() {
        let data = [
            b'M', b'T', b'h', b'd', // Magic
            0, 0, 0, 6, // Length
            0, 1, // Format 1
            0, 3, // 3 tracks
            0, 96, // 96 ticks per quarter note
        ];

        let header = parse_header(&data).unwrap();
        assert_eq!(header.format, 1);
        assert_eq!(header.num_tracks, 3);
        assert_eq!(header.ticks_per_quarter_note, 96);
    }

    #[test]
    fn test_parse_invalid_header_magic() {
        let data = [
            b'M', b'T', b'h', b'X', // Wrong magic
            0, 0, 0, 6, 0, 1, 0, 3, 0, 96,
        ];

        assert!(parse_header(&data).is_err());
    }

    #[test]
    fn test_parse_note_on() {
        // Delta time: 0, Note On channel 0, note 60, velocity 100
        let data = [0x00, 0x90, 0x3C, 0x64, 0x00, 0xFF, 0x2F, 0x00]; // Add End of Track

        let events = parse_track_events(&data).unwrap();
        assert_eq!(events.len(), 2); // NoteOn + EndOfTrack
        assert_eq!(events[0].delta_ticks, 0);

        match &events[0].event {
            Event::NoteOn {
                channel,
                note,
                velocity,
            } => {
                assert_eq!(*channel, 0);
                assert_eq!(*note, 60);
                assert_eq!(*velocity, 100);
            }
            _ => panic!("Expected NoteOn event"),
        }
    }

    #[test]
    fn test_parse_minimal_file() {
        let data = vec![
            // Header
            b'M', b'T', b'h', b'd', 0, 0, 0, 6, 0, 0, // Format 0
            0, 1, // 1 track
            0, 96, // 96 TPPQN
            // Track
            b'M', b'T', b'r', b'k', 0, 0, 0, 4, 0x00, 0xFF, 0x2F, 0x00, // End of track
        ];

        let midi = parse_midi_file(&data).unwrap();
        assert_eq!(midi.header.format, 0);
        assert_eq!(midi.header.num_tracks, 1);
        assert_eq!(midi.tracks.len(), 1);
    }

    #[test]
    fn test_total_notes() {
        let data = vec![
            // Header
            b'M', b'T', b'h', b'd', 0, 0, 0, 6, 0, 0, // Format 0
            0, 1, // 1 track
            0, 96, // 96 TPPQN
            // Track
            b'M', b'T', b'r', b'k', 0, 0, 0, 12, // Note On
            0x00, 0x90, 0x3C, 0x64, // Note On
            0x00, 0x90, 0x40, 0x64, // End of track
            0x00, 0xFF, 0x2F, 0x00,
        ];

        let midi = parse_midi_file(&data).unwrap();
        assert_eq!(midi.total_notes(), 2);
    }

    #[test]
    fn test_channels_used() {
        let data = vec![
            // Header
            b'M', b'T', b'h', b'd', 0, 0, 0, 6, 0, 0, // Format 0
            0, 1, // 1 track
            0, 96, // 96 TPPQN
            // Track
            b'M', b'T', b'r', b'k', 0, 0, 0, 16, // Note On channel 0
            0x00, 0x90, 0x3C, 0x64, // Note On channel 1
            0x00, 0x91, 0x40, 0x64, // Note On channel 9 (drums)
            0x00, 0x99, 0x24, 0x64, // End of track
            0x00, 0xFF, 0x2F, 0x00,
        ];

        let midi = parse_midi_file(&data).unwrap();
        let channels = midi.channels_used();
        assert_eq!(channels, vec![0, 1, 9]);
    }

    // ============================================================================
    // Helper Functions for Building MIDI Test Data
    // ============================================================================

    fn encode_vlq(mut value: u32) -> Vec<u8> {
        if value == 0 {
            return vec![0];
        }

        let mut bytes = Vec::new();
        let mut buffer = value & 0x7F;
        value >>= 7;

        while value > 0 {
            buffer <<= 8;
            buffer |= (value & 0x7F) | 0x80;
            value >>= 7;
        }

        loop {
            bytes.push((buffer & 0xFF) as u8);
            if buffer & 0x80 == 0 {
                break;
            }
            buffer >>= 8;
        }

        bytes
    }

    #[allow(dead_code)]
    fn minimal_track_with_eot() -> Vec<u8> {
        vec![
            0x00, 0xFF, 0x2F, 0x00, // Delta=0, EndOfTrack
        ]
    }

    // ============================================================================
    // Variable-Length Quantity Tests (Extended)
    // ============================================================================

    #[test]
    fn test_read_var_len_3_bytes() {
        // 3-byte VLQ: values 0x4000 - 0x1FFFFF
        let data = &[0x81, 0x80, 0x00]; // 0x4000 (16384)
        assert_eq!(read_var_len(data), Some((16384, 3)));

        let data = &[0xFF, 0xFF, 0x7F]; // Maximum 3-byte VLQ
        assert_eq!(read_var_len(data), Some((2097151, 3)));
    }

    #[test]
    fn test_read_var_len_4_bytes() {
        // 4-byte VLQ: values 0x200000 - 0x0FFFFFFF
        let data = &[0x81, 0x80, 0x80, 0x00]; // 0x200000 (2097152)
        assert_eq!(read_var_len(data), Some((2097152, 4)));

        let data = &[0xFF, 0xFF, 0xFF, 0x7F]; // Maximum VLQ (0x0FFFFFFF)
        assert_eq!(read_var_len(data), Some((268435455, 4)));
    }

    #[test]
    fn test_read_var_len_edge_values() {
        assert_eq!(read_var_len(&[0x00]), Some((0, 1)));
        assert_eq!(read_var_len(&[0x7F]), Some((127, 1)));
        assert_eq!(read_var_len(&[0x81, 0x00]), Some((128, 2)));
        assert_eq!(read_var_len(&[0xFF, 0x7F]), Some((16383, 2)));
    }

    #[test]
    fn test_read_var_len_too_long() {
        // More than 4 bytes should fail
        let data = &[0x81, 0x82, 0x83, 0x84, 0x85];
        assert_eq!(read_var_len(data), None);
    }

    // ============================================================================
    // Header Parsing Tests (Extended)
    // ============================================================================

    #[test]
    fn test_parse_header_format_0() {
        let data = [
            b'M', b'T', b'h', b'd',
            0, 0, 0, 6,
            0, 0,  // Format 0
            0, 1,  // 1 track
            0, 96,
        ];
        let header = parse_header(&data).unwrap();
        assert_eq!(header.format, 0);
        assert_eq!(header.num_tracks, 1);
    }

    #[test]
    fn test_parse_header_format_2() {
        let data = [
            b'M', b'T', b'h', b'd',
            0, 0, 0, 6,
            0, 2,  // Format 2
            0, 5,  // 5 patterns
            1, 0xE0,  // 480 TPPQN
        ];
        let header = parse_header(&data).unwrap();
        assert_eq!(header.format, 2);
        assert_eq!(header.num_tracks, 5);
        assert_eq!(header.ticks_per_quarter_note, 480);
    }

    #[test]
    fn test_parse_header_wrong_length() {
        let data = [
            b'M', b'T', b'h', b'd',
            0, 0, 0, 8,  // Wrong length (should be 6)
            0, 1, 0, 3, 0, 96,
        ];
        assert!(parse_header(&data).is_err());
    }

    #[test]
    fn test_parse_header_unsupported_format() {
        let data = [
            b'M', b'T', b'h', b'd',
            0, 0, 0, 6,
            0, 3,  // Format 3 (unsupported)
            0, 1, 0, 96,
        ];
        let result = parse_header(&data);
        assert!(matches!(result, Err(MidiParseError::UnsupportedFormat(3))));
    }

    #[test]
    fn test_parse_header_various_tppqn() {
        // Test different ticks-per-quarter-note values
        for tppqn in [96, 192, 384, 480, 960] {
            let data = [
                b'M', b'T', b'h', b'd', 0, 0, 0, 6,
                0, 1, 0, 1,
                (tppqn >> 8) as u8, (tppqn & 0xFF) as u8,
            ];
            let header = parse_header(&data).unwrap();
            assert_eq!(header.ticks_per_quarter_note, tppqn);
        }
    }

    // ============================================================================
    // Channel Event Tests
    // ============================================================================

    #[test]
    fn test_parse_note_off() {
        let data = vec![
            0x00, 0x80, 0x3C, 0x40,  // Delta=0, NoteOff channel 0, note 60, velocity 64
            0x00, 0xFF, 0x2F, 0x00,  // EndOfTrack
        ];
        let events = parse_track_events(&data).unwrap();
        assert_eq!(events.len(), 2);

        match &events[0].event {
            Event::NoteOff { channel, note, velocity } => {
                assert_eq!(*channel, 0);
                assert_eq!(*note, 60);
                assert_eq!(*velocity, 64);
            }
            _ => panic!("Expected NoteOff event"),
        }
    }

    #[test]
    fn test_parse_aftertouch() {
        let data = vec![
            0x00, 0xA0, 0x3C, 0x50,  // Delta=0, Aftertouch channel 0, note 60, pressure 80
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::Aftertouch { channel, note, pressure } => {
                assert_eq!(*channel, 0);
                assert_eq!(*note, 60);
                assert_eq!(*pressure, 80);
            }
            _ => panic!("Expected Aftertouch event"),
        }
    }

    #[test]
    fn test_parse_control_change() {
        let data = vec![
            0x00, 0xB0, 0x07, 0x64,  // Delta=0, CC channel 0, controller 7 (volume), value 100
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::ControlChange { channel, controller, value } => {
                assert_eq!(*channel, 0);
                assert_eq!(*controller, 7);
                assert_eq!(*value, 100);
            }
            _ => panic!("Expected ControlChange event"),
        }
    }

    #[test]
    fn test_parse_program_change() {
        let data = vec![
            0x00, 0xC0, 0x19,  // Delta=0, ProgramChange channel 0, program 25
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::ProgramChange { channel, program } => {
                assert_eq!(*channel, 0);
                assert_eq!(*program, 25);
            }
            _ => panic!("Expected ProgramChange event"),
        }
    }

    #[test]
    fn test_parse_channel_aftertouch() {
        let data = vec![
            0x00, 0xD0, 0x40,  // Delta=0, ChannelAftertouch channel 0, pressure 64
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::ChannelAftertouch { channel, pressure } => {
                assert_eq!(*channel, 0);
                assert_eq!(*pressure, 64);
            }
            _ => panic!("Expected ChannelAftertouch event"),
        }
    }

    #[test]
    fn test_parse_pitch_bend_center() {
        // Center position: LSB=0, MSB=64 → value=0
        let data = vec![
            0x00, 0xE0, 0x00, 0x40,  // Delta=0, PitchBend channel 0, center
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::PitchBend { channel, value } => {
                assert_eq!(*channel, 0);
                assert_eq!(*value, 0);  // Centered
            }
            _ => panic!("Expected PitchBend event"),
        }
    }

    #[test]
    fn test_parse_pitch_bend_max_up() {
        // Maximum up: LSB=127, MSB=127 → value=8191
        let data = vec![
            0x00, 0xE0, 0x7F, 0x7F,  // Delta=0, PitchBend channel 0, max up
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::PitchBend { channel, value } => {
                assert_eq!(*channel, 0);
                assert_eq!(*value, 8191);  // Max up
            }
            _ => panic!("Expected PitchBend event"),
        }
    }

    #[test]
    fn test_parse_pitch_bend_max_down() {
        // Maximum down: LSB=0, MSB=0 → value=-8192
        let data = vec![
            0x00, 0xE0, 0x00, 0x00,  // Delta=0, PitchBend channel 0, max down
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::PitchBend { channel, value } => {
                assert_eq!(*channel, 0);
                assert_eq!(*value, -8192);  // Max down
            }
            _ => panic!("Expected PitchBend event"),
        }
    }

    #[test]
    fn test_parse_all_16_channels() {
        // Test NoteOn on all 16 MIDI channels (0-15)
        for channel in 0..16 {
            let data = vec![
                0x00, 0x90 | channel, 0x3C, 0x64,
                0x00, 0xFF, 0x2F, 0x00,
            ];
            let events = parse_track_events(&data).unwrap();

            match &events[0].event {
                Event::NoteOn { channel: ch, .. } => {
                    assert_eq!(*ch, channel);
                }
                _ => panic!("Expected NoteOn event"),
            }
        }
    }

    // ============================================================================
    // Meta Event Tests
    // ============================================================================

    #[test]
    fn test_parse_end_of_track() {
        let data = vec![0x00, 0xFF, 0x2F, 0x00];  // Delta=0, EndOfTrack
        let events = parse_track_events(&data).unwrap();

        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].event, Event::EndOfTrack));
    }

    #[test]
    fn test_parse_tempo_change() {
        // 500,000 microseconds/quarter note = 120 BPM
        let data = vec![
            0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20,  // Tempo event
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::TempoChange { microseconds_per_quarter } => {
                assert_eq!(*microseconds_per_quarter, 500000);
            }
            _ => panic!("Expected TempoChange event"),
        }
    }

    #[test]
    fn test_parse_time_signature() {
        // 4/4 time
        let data = vec![
            0x00, 0xFF, 0x58, 0x04,
            0x04,  // Numerator (4)
            0x02,  // Denominator (2^2 = 4)
            0x18,  // Clocks per click (24)
            0x08,  // 32nds per quarter (8)
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::TimeSignature { numerator, denominator, clocks_per_click, thirty_seconds_per_quarter } => {
                assert_eq!(*numerator, 4);
                assert_eq!(*denominator, 2);  // 2^2 = 4
                assert_eq!(*clocks_per_click, 24);
                assert_eq!(*thirty_seconds_per_quarter, 8);
            }
            _ => panic!("Expected TimeSignature event"),
        }
    }

    #[test]
    fn test_parse_key_signature_sharps() {
        // D major (2 sharps)
        let data = vec![
            0x00, 0xFF, 0x59, 0x02,
            0x02,  // 2 sharps
            0x00,  // Major
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::KeySignature { sharps_flats, is_minor } => {
                assert_eq!(*sharps_flats, 2);
                assert!(!(*is_minor));
            }
            _ => panic!("Expected KeySignature event"),
        }
    }

    #[test]
    fn test_parse_key_signature_flats() {
        // B-flat major (2 flats), represented as -2
        let data = vec![
            0x00, 0xFF, 0x59, 0x02,
            0xFE,  // -2 (2 flats) as two's complement
            0x00,  // Major
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::KeySignature { sharps_flats, is_minor } => {
                assert_eq!(*sharps_flats, -2);
                assert!(!(*is_minor));
            }
            _ => panic!("Expected KeySignature event"),
        }
    }

    #[test]
    fn test_parse_text_event_track_name() {
        let text = "Piano";
        let data = vec![
            0x00, 0xFF, 0x03, text.len() as u8,
        ]
        .into_iter()
        .chain(text.bytes())
        .chain([0x00, 0xFF, 0x2F, 0x00].iter().copied())
        .collect::<Vec<u8>>();

        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::Text { text_type, text: t } => {
                assert!(matches!(text_type, TextType::TrackName));
                assert_eq!(t, text);
            }
            _ => panic!("Expected Text event"),
        }
    }

    #[test]
    fn test_parse_text_event_lyric() {
        let lyric = "Hello world";
        let data = vec![
            0x00, 0xFF, 0x05, lyric.len() as u8,
        ]
        .into_iter()
        .chain(lyric.bytes())
        .chain([0x00, 0xFF, 0x2F, 0x00].iter().copied())
        .collect::<Vec<u8>>();

        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::Text { text_type, text: t } => {
                assert!(matches!(text_type, TextType::Lyric));
                assert_eq!(t, lyric);
            }
            _ => panic!("Expected Text event"),
        }
    }

    #[test]
    fn test_parse_text_empty() {
        let data = vec![
            0x00, 0xFF, 0x01, 0x00,  // Text event with 0 length
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::Text { text, .. } => {
                assert_eq!(text, "");
            }
            _ => panic!("Expected Text event"),
        }
    }

    #[test]
    fn test_parse_text_invalid_utf8() {
        let data = vec![
            0x00, 0xFF, 0x01, 0x03,
            0xFF, 0xFE, 0xFD,  // Invalid UTF-8
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let result = parse_track_events(&data);
        assert!(matches!(result, Err(MidiParseError::Utf8(_))));
    }

    #[test]
    fn test_parse_unknown_meta_event() {
        let data = vec![
            0x00, 0xFF, 0x7E, 0x02,  // Unknown meta type 0x7E
            0x12, 0x34,  // Some data
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::Unknown { status, data: d } => {
                assert_eq!(*status, 0xFF);
                assert_eq!(d, &vec![0x12, 0x34]);
            }
            _ => panic!("Expected Unknown event"),
        }
    }

    // ============================================================================
    // SysEx Event Tests
    // ============================================================================

    #[test]
    fn test_parse_sysex_f0() {
        let data = vec![
            0x00, 0xF0, 0x05,  // Delta=0, SysEx, length=5
            0x43, 0x12, 0x00, 0x01, 0xF7,  // SysEx data
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::SysEx { data: d } => {
                assert_eq!(d, &vec![0x43, 0x12, 0x00, 0x01, 0xF7]);
            }
            _ => panic!("Expected SysEx event"),
        }
    }

    #[test]
    fn test_parse_sysex_f7() {
        let data = vec![
            0x00, 0xF7, 0x03,  // Delta=0, SysEx escape, length=3
            0x01, 0x02, 0x03,
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::SysEx { data: d } => {
                assert_eq!(d, &vec![0x01, 0x02, 0x03]);
            }
            _ => panic!("Expected SysEx event"),
        }
    }

    #[test]
    fn test_parse_sysex_empty() {
        let data = vec![
            0x00, 0xF0, 0x00,  // Delta=0, SysEx, length=0
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::SysEx { data: d } => {
                assert!(d.is_empty());
            }
            _ => panic!("Expected SysEx event"),
        }
    }

    // ============================================================================
    // Running Status Tests
    // ============================================================================

    #[test]
    fn test_running_status_continuation() {
        let data = vec![
            0x00, 0x90, 0x3C, 0x64,  // NoteOn C4 with status byte
            0x00, 0x3E, 0x64,        // NoteOn D4 without status (running status)
            0x00, 0x40, 0x64,        // NoteOn E4 without status (running status)
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        // Should have 3 NoteOn events + EndOfTrack
        assert_eq!(events.len(), 4);
        for event in events.iter().take(3) {
            assert!(matches!(event.event, Event::NoteOn { .. }));
        }
    }

    #[test]
    fn test_running_status_cleared_by_meta() {
        let data = vec![
            0x00, 0x90, 0x3C, 0x64,  // NoteOn with status
            0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20,  // Tempo (clears running status)
            0x00, 0x90, 0x3E, 0x64,  // NoteOn - needs status byte again
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();
        assert_eq!(events.len(), 4);  // 2 NoteOns + Tempo + EndOfTrack
    }

    #[test]
    fn test_running_status_error_without_prior() {
        let data = vec![
            0x00, 0x3C, 0x64,  // Data byte without prior status
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let result = parse_track_events(&data);
        assert!(matches!(result, Err(MidiParseError::InvalidEvent { .. })));
    }

    // ============================================================================
    // Track Parsing Tests
    // ============================================================================

    #[test]
    fn test_parse_empty_track() {
        let data = vec![
            b'M', b'T', b'r', b'k',
            0, 0, 0, 4,  // Length = 4
            0x00, 0xFF, 0x2F, 0x00,  // Just EndOfTrack
        ];
        let (track, bytes_consumed) = parse_track(&data).unwrap();
        assert_eq!(track.events.len(), 1);
        assert_eq!(bytes_consumed, 12);  // 8 header + 4 data
    }

    #[test]
    fn test_parse_track_invalid_magic() {
        let data = vec![
            b'M', b'T', b'r', b'X',  // Wrong magic
            0, 0, 0, 4,
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let result = parse_track(&data);
        assert!(matches!(result, Err(MidiParseError::InvalidTrack { .. })));
    }

    #[test]
    fn test_parse_track_too_short() {
        let data = vec![b'M', b'T', b'r', b'k', 0, 0];  // Only 6 bytes
        let result = parse_track(&data);
        assert!(matches!(result, Err(MidiParseError::InvalidTrack { .. })));
    }

    #[test]
    fn test_parse_track_incomplete_data() {
        let data = vec![
            b'M', b'T', b'r', b'k',
            0, 0, 0, 10,  // Claims 10 bytes
            0x00, 0xFF, 0x2F, 0x00,  // Only 4 bytes provided
        ];
        let result = parse_track(&data);
        assert!(matches!(result, Err(MidiParseError::InvalidTrack { .. })));
    }

    // ============================================================================
    // File Parsing Tests
    // ============================================================================

    #[test]
    fn test_parse_file_too_short() {
        let data = vec![b'M', b'T', b'h', b'd', 0, 0, 0, 6, 0, 0];  // Only 10 bytes
        let result = parse_midi_file(&data);
        assert!(matches!(result, Err(MidiParseError::IncompleteData { .. })));
    }

    #[test]
    fn test_parse_format_1_multiple_tracks() {
        let data = vec![
            // Header
            b'M', b'T', b'h', b'd', 0, 0, 0, 6,
            0, 1,  // Format 1
            0, 2,  // 2 tracks
            0, 96,
            // Track 1
            b'M', b'T', b'r', b'k', 0, 0, 0, 4,
            0x00, 0xFF, 0x2F, 0x00,
            // Track 2
            b'M', b'T', b'r', b'k', 0, 0, 0, 4,
            0x00, 0xFF, 0x2F, 0x00,
        ];

        let midi = parse_midi_file(&data).unwrap();
        assert_eq!(midi.header.format, 1);
        assert_eq!(midi.tracks.len(), 2);
    }

    // ============================================================================
    // Error Condition Tests
    // ============================================================================

    #[test]
    fn test_incomplete_note_on() {
        let data = vec![0x00, 0x90, 0x3C];  // Missing velocity byte
        let result = parse_track_events(&data);
        assert!(matches!(result, Err(MidiParseError::IncompleteData { .. })));
    }

    #[test]
    fn test_incomplete_control_change() {
        let data = vec![0x00, 0xB0, 0x07];  // Missing value byte
        let result = parse_track_events(&data);
        assert!(matches!(result, Err(MidiParseError::IncompleteData { .. })));
    }

    #[test]
    fn test_incomplete_pitch_bend() {
        let data = vec![0x00, 0xE0, 0x00];  // Missing MSB
        let result = parse_track_events(&data);
        assert!(matches!(result, Err(MidiParseError::IncompleteData { .. })));
    }

    #[test]
    fn test_invalid_tempo_length() {
        let data = vec![
            0x00, 0xFF, 0x51, 0x02,  // Tempo with wrong length (should be 3)
            0x07, 0xA1,
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let result = parse_track_events(&data);
        assert!(matches!(result, Err(MidiParseError::InvalidEvent { .. })));
    }

    #[test]
    fn test_invalid_time_signature_length() {
        let data = vec![
            0x00, 0xFF, 0x58, 0x03,  // TimeSignature with wrong length (should be 4)
            0x04, 0x02, 0x18,
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let result = parse_track_events(&data);
        assert!(matches!(result, Err(MidiParseError::InvalidEvent { .. })));
    }

    #[test]
    fn test_invalid_key_signature_length() {
        let data = vec![
            0x00, 0xFF, 0x59, 0x01,  // KeySignature with wrong length (should be 2)
            0x02,
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let result = parse_track_events(&data);
        assert!(matches!(result, Err(MidiParseError::InvalidEvent { .. })));
    }

    // ============================================================================
    // Edge Cases Tests
    // ============================================================================

    #[test]
    fn test_zero_delta_ticks() {
        // Multiple events with delta=0 (simultaneous)
        let data = vec![
            0x00, 0x90, 0x3C, 0x64,  // Delta=0, NoteOn
            0x00, 0x90, 0x40, 0x64,  // Delta=0, NoteOn (simultaneous)
            0x00, 0x90, 0x43, 0x64,  // Delta=0, NoteOn (simultaneous)
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        assert_eq!(events.len(), 4);
        assert_eq!(events[0].delta_ticks, 0);
        assert_eq!(events[1].delta_ticks, 0);
        assert_eq!(events[2].delta_ticks, 0);
    }

    #[test]
    fn test_large_delta_ticks() {
        // Large delta time using VLQ
        let large_delta = encode_vlq(100000);
        let data = large_delta
            .into_iter()
            .chain([0x90, 0x3C, 0x64])
            .chain([0x00, 0xFF, 0x2F, 0x00])
            .collect::<Vec<u8>>();

        let events = parse_track_events(&data).unwrap();
        assert_eq!(events[0].delta_ticks, 100000);
    }

    #[test]
    fn test_max_note_velocity_values() {
        let data = vec![
            0x00, 0x90, 0x7F, 0x7F,  // Note 127, velocity 127 (maximum values)
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::NoteOn { note, velocity, .. } => {
                assert_eq!(*note, 127);
                assert_eq!(*velocity, 127);
            }
            _ => panic!("Expected NoteOn event"),
        }
    }
}

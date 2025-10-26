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
}

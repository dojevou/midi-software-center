
/// Track Splitter - TRUSTY MODULE
///
/// Pure logic for splitting multi-track MIDI files into individual single-track files.
///
/// This module operates on byte arrays (no I/O) and provides functions to:
/// - Parse multi-track MIDI files (Format 1)
/// - Split into separate Format 0 (single-track) MIDI files
/// - Extract metadata (track name, channel, instrument, note count)
/// - Handle tempo tracks and edge cases
///
/// # Archetype: TRUSTY MODULE
/// - ✅ Pure functions, no side effects
/// - ✅ No I/O operations
/// - ✅ Operates on byte slices
/// - ✅ Comprehensive error handling
/// - ✅ Well-tested
use midly::{Format, Header, MetaMessage, Smf, Track, TrackEvent, TrackEventKind};
use thiserror::Error;

/// Error types for track splitting operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum SplitError {
    /// Failed to parse MIDI data
    #[error("Failed to parse MIDI data: {0}")]
    ParseError(String),

    /// Failed to write MIDI data
    #[error("Failed to write MIDI data: {0}")]
    WriteError(String),

    /// No tracks to split (empty or single tempo track)
    #[error("No tracks to split - file contains only tempo track or is empty")]
    NoTracksToSplit,
}

/// Information about a split track
#[derive(Debug, Clone, PartialEq)]
pub struct SplitTrack {
    /// Original track number (0-indexed)
    pub track_number: usize,

    /// Track name from meta events (if present)
    pub track_name: Option<String>,

    /// Primary MIDI channel used by this track (0-15)
    pub channel: Option<u8>,

    /// General MIDI instrument name
    pub instrument: Option<String>,

    /// Number of note-on events in this track
    pub note_count: usize,

    /// Complete Format 0 MIDI file as bytes
    pub midi_bytes: Vec<u8>,
}

/// Split multi-track MIDI file into individual single-track files.
///
/// Parses a MIDI file and creates separate Format 0 (single-track) MIDI files
/// for each music track. Skips tempo-only tracks (Track 0 in Format 1 files).
/// Preserves tempo, time signature, and key signature from the original file.
///
/// # Arguments
///
/// * `original_midi_bytes` - Complete MIDI file as byte slice
///
/// # Returns
///
/// Vector of `SplitTrack` structs, one for each music track found.
/// Returns error if parsing fails or no music tracks exist.
///
/// # Examples
///
/// ```
/// use pipeline::core::splitting::track_splitter::split_tracks;
///
/// // Parse multi-track MIDI
/// let midi_bytes = include_bytes!("test_data/multitrack.mid");
/// let tracks = split_tracks(midi_bytes)?;
///
/// for track in tracks {
///     println!("Track {}: {} notes", track.track_number, track.note_count);
///     if let Some(name) = track.track_name {
///         println!("  Name: {}", name);
///     }
/// }
/// # Ok::<(), pipeline::core::splitting::track_splitter::SplitError>(())
/// ```
pub fn split_tracks(original_midi_bytes: &[u8]) -> Result<Vec<SplitTrack>, SplitError> {
    // Parse the original MIDI file
    let smf = Smf::parse(original_midi_bytes)
        .map_err(|e| SplitError::ParseError(format!("midly parse error: {}", e)))?;

    // Check format - if already Format 0, return as-is
    if smf.header.format == Format::SingleTrack {
        let track = &smf.tracks[0];
        let note_count = count_notes(track);

        // Only return if it has notes
        if note_count == 0 {
            return Err(SplitError::NoTracksToSplit);
        }

        return Ok(vec![SplitTrack {
            track_number: 0,
            track_name: extract_track_name(track),
            channel: extract_primary_channel(track),
            instrument: extract_instrument(track),
            note_count,
            midi_bytes: original_midi_bytes.to_vec(),
        }]);
    }

    // Process Format 1 (parallel tracks) or Format 2 (sequential)
    let mut split_tracks = Vec::new();

    for (idx, track) in smf.tracks.iter().enumerate() {
        // Skip tempo-only tracks (usually Track 0)
        if is_tempo_track(track) {
            continue;
        }

        let note_count = count_notes(track);

        // Skip tracks with no notes
        if note_count == 0 {
            continue;
        }

        // Create Format 0 MIDI file for this track
        let midi_bytes = create_single_track_midi(&smf, track, idx)?;

        split_tracks.push(SplitTrack {
            track_number: idx,
            track_name: extract_track_name(track),
            channel: extract_primary_channel(track),
            instrument: extract_instrument(track),
            note_count,
            midi_bytes,
        });
    }

    if split_tracks.is_empty() {
        return Err(SplitError::NoTracksToSplit);
    }

    Ok(split_tracks)
}

/// Check if a track is a tempo-only track.
///
/// Tempo tracks contain only meta events (tempo, time signature, key signature)
/// and no note events. Common in Format 1 MIDI files as Track 0.
///
/// # Arguments
///
/// * `track` - MIDI track to analyze
///
/// # Returns
///
/// `true` if track contains only meta events, `false` otherwise
pub fn is_tempo_track(track: &Track) -> bool {
    let mut has_meta_events = false;
    let mut has_note_events = false;

    for event in track.iter() {
        match event.kind {
            TrackEventKind::Meta(_) => has_meta_events = true,
            TrackEventKind::Midi { message, .. } => {
                // Check for note-on or note-off
                use midly::MidiMessage;
                match message {
                    MidiMessage::NoteOn { .. } | MidiMessage::NoteOff { .. } => {
                        has_note_events = true;
                        break;
                    },
                    _ => {},
                }
            },
            _ => {},
        }
    }

    has_meta_events && !has_note_events
}

/// Create a Format 0 (single-track) MIDI file from a single track.
///
/// Merges tempo/time signature/key signature events from Track 0 (if Format 1)
/// with the music events from the specified track. Creates a valid Format 0 MIDI file.
///
/// # Arguments
///
/// * `original` - Original parsed MIDI file
/// * `track` - Track to extract
/// * `track_idx` - Index of the track (for reference)
///
/// # Returns
///
/// Complete Format 0 MIDI file as bytes
pub fn create_single_track_midi(
    original: &Smf,
    track: &Track,
    track_idx: usize,
) -> Result<Vec<u8>, SplitError> {
    // Create new Format 0 header with same timing
    let new_header = Header { format: Format::SingleTrack, timing: original.header.timing };

    // Build new track by merging tempo events from Track 0 (if exists) with this track
    let mut new_track_events = Vec::new();

    // If Format 1 and this isn't Track 0, copy tempo/meta events from Track 0
    if original.header.format == Format::Parallel && track_idx > 0 && !original.tracks.is_empty() {
        let track_0 = &original.tracks[0];
        for event in track_0.iter() {
            match event.kind {
                TrackEventKind::Meta(MetaMessage::Tempo(_))
                | TrackEventKind::Meta(MetaMessage::TimeSignature(..))
                | TrackEventKind::Meta(MetaMessage::KeySignature(..)) => {
                    new_track_events.push(*event);
                },
                _ => {},
            }
        }
    }

    // Add all events from the target track
    new_track_events.extend(track.iter().cloned());

    // Ensure track ends with End of Track
    let has_end_of_track = new_track_events
        .iter()
        .any(|e| matches!(e.kind, TrackEventKind::Meta(MetaMessage::EndOfTrack)));

    if !has_end_of_track {
        new_track_events.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
        });
    }

    // Create new SMF with single track
    let new_smf = Smf { header: new_header, tracks: vec![new_track_events] };

    // Write to bytes
    let mut bytes = Vec::new();
    new_smf
        .write_std(&mut bytes)
        .map_err(|e| SplitError::WriteError(format!("midly write error: {}", e)))?;

    Ok(bytes)
}

/// Extract track name from meta events.
///
/// Searches for TrackName or InstrumentName meta events.
///
/// # Arguments
///
/// * `track` - MIDI track to analyze
///
/// # Returns
///
/// Track name if found, `None` otherwise
pub fn extract_track_name(track: &Track) -> Option<String> {
    for event in track.iter() {
        if let TrackEventKind::Meta(MetaMessage::TrackName(name) | MetaMessage::InstrumentName(name)) = &event.kind {
            // Convert bytes to string
            if let Ok(name_str) = String::from_utf8(name.to_vec()) {
                let trimmed = name_str.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }
    None
}

/// Extract the primary MIDI channel used by this track.
///
/// Analyzes all MIDI messages and returns the most frequently used channel.
///
/// # Arguments
///
/// * `track` - MIDI track to analyze
///
/// # Returns
///
/// Most frequently used channel (0-15), or `None` if no MIDI messages found
pub fn extract_primary_channel(track: &Track) -> Option<u8> {
    let mut channel_counts = [0u32; 16];

    for event in track.iter() {
        if let TrackEventKind::Midi { channel, .. } = event.kind {
            channel_counts[channel.as_int() as usize] += 1;
        }
    }

    // Find channel with highest count
    let max_channel = channel_counts.iter().enumerate().max_by_key(|(_, &count)| count)?;

    if max_channel.1 > &0 {
        Some(max_channel.0 as u8)
    } else {
        None
    }
}

/// Extract instrument name from Program Change events.
///
/// Searches for the first Program Change event and maps to General MIDI instrument name.
///
/// # Arguments
///
/// * `track` - MIDI track to analyze
///
/// # Returns
///
/// General MIDI instrument name, or `None` if no Program Change found
pub fn extract_instrument(track: &Track) -> Option<String> {
    for event in track.iter() {
        if let TrackEventKind::Midi { message, .. } = &event.kind {
            use midly::MidiMessage;
            if let MidiMessage::ProgramChange { program } = message {
                return Some(get_instrument_name(program.as_int()));
            }
        }
    }
    None
}

/// Count note-on events in a track.
///
/// # Arguments
///
/// * `track` - MIDI track to analyze
///
/// # Returns
///
/// Number of note-on events with velocity > 0
pub fn count_notes(track: &Track) -> usize {
    let mut count = 0;

    for event in track.iter() {
        if let TrackEventKind::Midi { message, .. } = &event.kind {
            use midly::MidiMessage;
            if let MidiMessage::NoteOn { vel, .. } = message {
                if vel.as_int() > 0 {
                    count += 1;
                }
            }
        }
    }

    count
}

/// Get General MIDI instrument name from program number.
///
/// Maps GM program numbers (0-127) to standard instrument names.
///
/// # Arguments
///
/// * `program` - GM program number (0-127)
///
/// # Returns
///
/// General MIDI instrument name
///
/// # Examples
///
/// ```
/// use pipeline::core::splitting::track_splitter::get_instrument_name;
///
/// assert_eq!(get_instrument_name(0), "Acoustic Grand Piano");
/// assert_eq!(get_instrument_name(25), "Acoustic Guitar (nylon)");
/// assert_eq!(get_instrument_name(127), "Gunshot");
/// ```
pub fn get_instrument_name(program: u8) -> String {
    match program {
        // Piano (0-7)
        0 => "Acoustic Grand Piano",
        1 => "Bright Acoustic Piano",
        2 => "Electric Grand Piano",
        3 => "Honky-tonk Piano",
        4 => "Electric Piano 1",
        5 => "Electric Piano 2",
        6 => "Harpsichord",
        7 => "Clavinet",

        // Chromatic Percussion (8-15)
        8 => "Celesta",
        9 => "Glockenspiel",
        10 => "Music Box",
        11 => "Vibraphone",
        12 => "Marimba",
        13 => "Xylophone",
        14 => "Tubular Bells",
        15 => "Dulcimer",

        // Organ (16-23)
        16 => "Drawbar Organ",
        17 => "Percussive Organ",
        18 => "Rock Organ",
        19 => "Church Organ",
        20 => "Reed Organ",
        21 => "Accordion",
        22 => "Harmonica",
        23 => "Tango Accordion",

        // Guitar (24-31)
        24 => "Acoustic Guitar (nylon)",
        25 => "Acoustic Guitar (steel)",
        26 => "Electric Guitar (jazz)",
        27 => "Electric Guitar (clean)",
        28 => "Electric Guitar (muted)",
        29 => "Overdriven Guitar",
        30 => "Distortion Guitar",
        31 => "Guitar Harmonics",

        // Bass (32-39)
        32 => "Acoustic Bass",
        33 => "Electric Bass (finger)",
        34 => "Electric Bass (pick)",
        35 => "Fretless Bass",
        36 => "Slap Bass 1",
        37 => "Slap Bass 2",
        38 => "Synth Bass 1",
        39 => "Synth Bass 2",

        // Strings (40-47)
        40 => "Violin",
        41 => "Viola",
        42 => "Cello",
        43 => "Contrabass",
        44 => "Tremolo Strings",
        45 => "Pizzicato Strings",
        46 => "Orchestral Harp",
        47 => "Timpani",

        // Ensemble (48-55)
        48 => "String Ensemble 1",
        49 => "String Ensemble 2",
        50 => "Synth Strings 1",
        51 => "Synth Strings 2",
        52 => "Choir Aahs",
        53 => "Voice Oohs",
        54 => "Synth Voice",
        55 => "Orchestra Hit",

        // Brass (56-63)
        56 => "Trumpet",
        57 => "Trombone",
        58 => "Tuba",
        59 => "Muted Trumpet",
        60 => "French Horn",
        61 => "Brass Section",
        62 => "Synth Brass 1",
        63 => "Synth Brass 2",

        // Reed (64-71)
        64 => "Soprano Sax",
        65 => "Alto Sax",
        66 => "Tenor Sax",
        67 => "Baritone Sax",
        68 => "Oboe",
        69 => "English Horn",
        70 => "Bassoon",
        71 => "Clarinet",

        // Pipe (72-79)
        72 => "Piccolo",
        73 => "Flute",
        74 => "Recorder",
        75 => "Pan Flute",
        76 => "Blown Bottle",
        77 => "Shakuhachi",
        78 => "Whistle",
        79 => "Ocarina",

        // Synth Lead (80-87)
        80 => "Lead 1 (square)",
        81 => "Lead 2 (sawtooth)",
        82 => "Lead 3 (calliope)",
        83 => "Lead 4 (chiff)",
        84 => "Lead 5 (charang)",
        85 => "Lead 6 (voice)",
        86 => "Lead 7 (fifths)",
        87 => "Lead 8 (bass + lead)",

        // Synth Pad (88-95)
        88 => "Pad 1 (new age)",
        89 => "Pad 2 (warm)",
        90 => "Pad 3 (polysynth)",
        91 => "Pad 4 (choir)",
        92 => "Pad 5 (bowed)",
        93 => "Pad 6 (metallic)",
        94 => "Pad 7 (halo)",
        95 => "Pad 8 (sweep)",

        // Synth Effects (96-103)
        96 => "FX 1 (rain)",
        97 => "FX 2 (soundtrack)",
        98 => "FX 3 (crystal)",
        99 => "FX 4 (atmosphere)",
        100 => "FX 5 (brightness)",
        101 => "FX 6 (goblins)",
        102 => "FX 7 (echoes)",
        103 => "FX 8 (sci-fi)",

        // Ethnic (104-111)
        104 => "Sitar",
        105 => "Banjo",
        106 => "Shamisen",
        107 => "Koto",
        108 => "Kalimba",
        109 => "Bag pipe",
        110 => "Fiddle",
        111 => "Shanai",

        // Percussive (112-119)
        112 => "Tinkle Bell",
        113 => "Agogo",
        114 => "Steel Drums",
        115 => "Woodblock",
        116 => "Taiko Drum",
        117 => "Melodic Tom",
        118 => "Synth Drum",
        119 => "Reverse Cymbal",

        // Sound Effects (120-127)
        120 => "Guitar Fret Noise",
        121 => "Breath Noise",
        122 => "Seashore",
        123 => "Bird Tweet",
        124 => "Telephone Ring",
        125 => "Helicopter",
        126 => "Applause",
        127 => "Gunshot",

        // Fallback (should never happen with u8)
        _ => "Unknown Instrument",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use midly::{MetaMessage, MidiMessage, TrackEvent, TrackEventKind};

    // Helper: Create test track with events
    fn create_test_track(events: Vec<TrackEvent>) -> Track {
        events
    }

    #[test]
    fn test_get_instrument_name_piano() {
        assert_eq!(get_instrument_name(0), "Acoustic Grand Piano");
        assert_eq!(get_instrument_name(1), "Bright Acoustic Piano");
        assert_eq!(get_instrument_name(7), "Clavinet");
    }

    #[test]
    fn test_get_instrument_name_guitar() {
        assert_eq!(get_instrument_name(24), "Acoustic Guitar (nylon)");
        assert_eq!(get_instrument_name(25), "Acoustic Guitar (steel)");
        assert_eq!(get_instrument_name(30), "Distortion Guitar");
    }

    #[test]
    fn test_get_instrument_name_strings() {
        assert_eq!(get_instrument_name(40), "Violin");
        assert_eq!(get_instrument_name(42), "Cello");
        assert_eq!(get_instrument_name(46), "Orchestral Harp");
    }

    #[test]
    fn test_get_instrument_name_brass() {
        assert_eq!(get_instrument_name(56), "Trumpet");
        assert_eq!(get_instrument_name(57), "Trombone");
        assert_eq!(get_instrument_name(60), "French Horn");
    }

    #[test]
    fn test_get_instrument_name_effects() {
        assert_eq!(get_instrument_name(120), "Guitar Fret Noise");
        assert_eq!(get_instrument_name(122), "Seashore");
        assert_eq!(get_instrument_name(127), "Gunshot");
    }

    #[test]
    fn test_count_notes_empty_track() {
        let track = create_test_track(vec![]);
        assert_eq!(count_notes(&track), 0);
    }

    #[test]
    fn test_count_notes_with_notes() {
        let track = create_test_track(vec![
            TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Midi {
                    channel: 0.into(),
                    message: MidiMessage::NoteOn { key: 60.into(), vel: 64.into() },
                },
            },
            TrackEvent {
                delta: 10.into(),
                kind: TrackEventKind::Midi {
                    channel: 0.into(),
                    message: MidiMessage::NoteOn { key: 64.into(), vel: 80.into() },
                },
            },
            TrackEvent {
                delta: 10.into(),
                kind: TrackEventKind::Midi {
                    channel: 0.into(),
                    message: MidiMessage::NoteOff { key: 60.into(), vel: 0.into() },
                },
            },
        ]);

        assert_eq!(count_notes(&track), 2);
    }

    #[test]
    fn test_count_notes_ignores_zero_velocity() {
        let track = create_test_track(vec![
            TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Midi {
                    channel: 0.into(),
                    message: MidiMessage::NoteOn { key: 60.into(), vel: 64.into() },
                },
            },
            TrackEvent {
                delta: 10.into(),
                kind: TrackEventKind::Midi {
                    channel: 0.into(),
                    message: MidiMessage::NoteOn {
                        key: 64.into(),
                        vel: 0.into(), // Zero velocity = note off
                    },
                },
            },
        ]);

        assert_eq!(count_notes(&track), 1);
    }

    #[test]
    fn test_is_tempo_track_true() {
        let track = create_test_track(vec![
            TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Meta(MetaMessage::Tempo(500000.into())),
            },
            TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Meta(MetaMessage::TimeSignature(4, 2, 24, 8)),
            },
            TrackEvent { delta: 0.into(), kind: TrackEventKind::Meta(MetaMessage::EndOfTrack) },
        ]);

        assert!(is_tempo_track(&track));
    }

    #[test]
    fn test_is_tempo_track_false_with_notes() {
        let track = create_test_track(vec![
            TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Meta(MetaMessage::Tempo(500000.into())),
            },
            TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Midi {
                    channel: 0.into(),
                    message: MidiMessage::NoteOn { key: 60.into(), vel: 64.into() },
                },
            },
        ]);

        assert!(!is_tempo_track(&track));
    }

    #[test]
    fn test_is_tempo_track_false_empty() {
        let track = create_test_track(vec![]);
        assert!(!is_tempo_track(&track));
    }

    #[test]
    fn test_extract_track_name_found() {
        let track = create_test_track(vec![TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::TrackName(b"Piano Track")),
        }]);

        assert_eq!(extract_track_name(&track), Some("Piano Track".to_string()));
    }

    #[test]
    fn test_extract_track_name_instrument_name() {
        let track = create_test_track(vec![TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::InstrumentName(b"Grand Piano")),
        }]);

        assert_eq!(extract_track_name(&track), Some("Grand Piano".to_string()));
    }

    #[test]
    fn test_extract_track_name_not_found() {
        let track = create_test_track(vec![TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::Tempo(500000.into())),
        }]);

        assert_eq!(extract_track_name(&track), None);
    }

    #[test]
    fn test_extract_track_name_empty_string() {
        let track = create_test_track(vec![TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::TrackName(b"   ")),
        }]);

        assert_eq!(extract_track_name(&track), None);
    }

    #[test]
    fn test_extract_primary_channel_single_channel() {
        let track = create_test_track(vec![
            TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Midi {
                    channel: 5.into(),
                    message: MidiMessage::NoteOn { key: 60.into(), vel: 64.into() },
                },
            },
            TrackEvent {
                delta: 10.into(),
                kind: TrackEventKind::Midi {
                    channel: 5.into(),
                    message: MidiMessage::NoteOff { key: 60.into(), vel: 0.into() },
                },
            },
        ]);

        assert_eq!(extract_primary_channel(&track), Some(5));
    }

    #[test]
    fn test_extract_primary_channel_multiple_channels() {
        let track = create_test_track(vec![
            TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Midi {
                    channel: 0.into(),
                    message: MidiMessage::NoteOn { key: 60.into(), vel: 64.into() },
                },
            },
            TrackEvent {
                delta: 10.into(),
                kind: TrackEventKind::Midi {
                    channel: 1.into(),
                    message: MidiMessage::NoteOn { key: 64.into(), vel: 64.into() },
                },
            },
            TrackEvent {
                delta: 10.into(),
                kind: TrackEventKind::Midi {
                    channel: 1.into(),
                    message: MidiMessage::NoteOn { key: 67.into(), vel: 64.into() },
                },
            },
        ]);

        // Channel 1 has more events
        assert_eq!(extract_primary_channel(&track), Some(1));
    }

    #[test]
    fn test_extract_primary_channel_no_midi() {
        let track = create_test_track(vec![TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::Tempo(500000.into())),
        }]);

        assert_eq!(extract_primary_channel(&track), None);
    }

    #[test]
    fn test_extract_instrument_found() {
        let track = create_test_track(vec![TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Midi {
                channel: 0.into(),
                message: MidiMessage::ProgramChange { program: 0.into() },
            },
        }]);

        assert_eq!(
            extract_instrument(&track),
            Some("Acoustic Grand Piano".to_string())
        );
    }

    #[test]
    fn test_extract_instrument_not_found() {
        let track = create_test_track(vec![TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Midi {
                channel: 0.into(),
                message: MidiMessage::NoteOn { key: 60.into(), vel: 64.into() },
            },
        }]);

        assert_eq!(extract_instrument(&track), None);
    }

    #[test]
    fn test_split_error_display() {
        let err = SplitError::ParseError("test error".to_string());
        assert_eq!(err.to_string(), "Failed to parse MIDI data: test error");

        let err = SplitError::NoTracksToSplit;
        assert_eq!(
            err.to_string(),
            "No tracks to split - file contains only tempo track or is empty"
        );
    }

    #[test]
    fn test_split_track_struct() {
        let track = SplitTrack {
            track_number: 1,
            track_name: Some("Piano".to_string()),
            channel: Some(0),
            instrument: Some("Acoustic Grand Piano".to_string()),
            note_count: 42,
            midi_bytes: vec![0x4d, 0x54, 0x68, 0x64],
        };

        assert_eq!(track.track_number, 1);
        assert_eq!(track.track_name, Some("Piano".to_string()));
        assert_eq!(track.channel, Some(0));
        assert_eq!(track.note_count, 42);
        assert_eq!(track.midi_bytes.len(), 4);
    }
}

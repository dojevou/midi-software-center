/// Chord Analysis Module
///
/// Detects chord progressions, types, and complexity from MIDI files
/// Uses pitch class analysis for chord identification
use crate::core::midi::types::{Event, MidiFile};
use std::collections::HashSet;

/// Result of chord analysis
#[derive(Debug, Clone)]
pub struct ChordAnalysis {
    pub progression: Vec<String>,
    pub types: Vec<String>,
    pub has_sevenths: bool,
    pub has_extended: bool,
    pub change_rate: Option<f32>,
    pub complexity_score: f32,
}

/// Represents a detected chord at a specific time
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct DetectedChord {
    tick: u32,
    notes: Vec<u8>, // MIDI note numbers
    name: String,
    chord_type: String,
    is_seventh: bool,
    is_extended: bool,
}

/// Main chord analysis function
pub fn analyze_chords(midi_file: &MidiFile, ticks_per_quarter: u32) -> ChordAnalysis {
    // Extract all note events with timing
    let note_events = extract_note_events(midi_file);

    if note_events.is_empty() {
        return ChordAnalysis {
            progression: Vec::new(),
            types: Vec::new(),
            has_sevenths: false,
            has_extended: false,
            change_rate: None,
            complexity_score: 0.0,
        };
    }

    // Group notes into time windows (e.g., every quarter note)
    let window_size = ticks_per_quarter / 2; // Half-note windows
    let chords = detect_chords_in_windows(&note_events, window_size);

    // Calculate metrics
    let progression: Vec<String> = chords.iter().map(|c| c.name.clone()).collect();
    let types: Vec<String> = chords.iter().map(|c| c.chord_type.clone()).collect();
    let has_sevenths = chords.iter().any(|c| c.is_seventh);
    let has_extended = chords.iter().any(|c| c.is_extended);

    // Calculate chord change rate (chords per measure, assuming 4/4 time)
    let ticks_per_measure = ticks_per_quarter * 4;
    let total_ticks = note_events.last().map(|(t, _, _)| *t).unwrap_or(0);
    let total_measures = if total_ticks > 0 {
        (total_ticks as f32 / ticks_per_measure as f32).max(1.0)
    } else {
        1.0
    };
    let change_rate = if chords.len() > 1 {
        Some(chords.len() as f32 / total_measures)
    } else {
        None
    };

    // Calculate complexity score
    let complexity_score = calculate_complexity(&chords);

    ChordAnalysis { progression, types, has_sevenths, has_extended, change_rate, complexity_score }
}

/// Extract all note-on events with timing and channel
fn extract_note_events(midi_file: &MidiFile) -> Vec<(u32, u8, u8)> {
    let mut events = Vec::new();
    let mut cumulative_tick = 0u32;

    for track in &midi_file.tracks {
        let mut track_tick = 0u32;

        for event in &track.events {
            track_tick += event.delta_ticks;

            match &event.event {
                Event::NoteOn { channel, note, velocity } if *velocity > 0 => {
                    events.push((track_tick, *note, *channel));
                },
                _ => {},
            }
        }

        cumulative_tick = cumulative_tick.max(track_tick);
    }

    events.sort_by_key(|(tick, _, _)| *tick);
    events
}

/// Group notes into time windows and detect chords
fn detect_chords_in_windows(note_events: &[(u32, u8, u8)], window_size: u32) -> Vec<DetectedChord> {
    let mut chords: Vec<DetectedChord> = Vec::new();
    let mut current_window_start = 0u32;
    let mut current_notes = Vec::new();

    for (tick, note, channel) in note_events {
        // Skip drum channel (channel 9/10 in 0-indexed/1-indexed)
        if *channel == 9 {
            continue;
        }

        // Check if we need to process current window
        while *tick >= current_window_start + window_size {
            if current_notes.len() >= 3 {
                if let Some(chord) = identify_chord(&current_notes) {
                    // Only add if different from last chord (avoid duplicates)
                    if chords.is_empty() || chords.last().unwrap().name != chord.name {
                        chords.push(chord);
                    }
                }
            }

            current_notes.clear();
            current_window_start += window_size;
        }

        current_notes.push(*note);
    }

    // Process final window
    if current_notes.len() >= 3 {
        if let Some(chord) = identify_chord(&current_notes) {
            if chords.is_empty() || chords.last().unwrap().name != chord.name {
                chords.push(chord);
            }
        }
    }

    chords
}

/// Identify chord from a collection of MIDI notes
fn identify_chord(notes: &[u8]) -> Option<DetectedChord> {
    if notes.len() < 3 {
        return None;
    }

    // Remove duplicates and sort
    let unique_notes: Vec<u8> = {
        let set: HashSet<u8> = notes.iter().map(|n| n % 12).collect();
        let mut vec: Vec<u8> = set.into_iter().collect();
        vec.sort();
        vec
    };

    if unique_notes.len() < 3 {
        return None;
    }

    // Try to identify the chord using music theory
    match identify_chord_type(&unique_notes) {
        Some((root, chord_type, is_seventh, is_extended)) => {
            let name = format!("{}{}", root, chord_type);
            Some(DetectedChord {
                tick: 0,
                notes: notes.to_vec(),
                name,
                chord_type,
                is_seventh,
                is_extended,
            })
        },
        None => None,
    }
}

/// Identify chord type from pitch classes
fn identify_chord_type(pitch_classes: &[u8]) -> Option<(String, String, bool, bool)> {
    if pitch_classes.len() < 3 {
        return None;
    }

    // Get root note (assume lowest pitch is root for simplicity)
    let root = pitch_classes[0];
    let root_name = note_name_from_pitch(root);

    // Calculate intervals from root
    let intervals: Vec<u8> = pitch_classes.iter().map(|&p| (p + 12 - root) % 12).collect();

    // Identify chord quality based on intervals
    let (chord_type, is_seventh, is_extended) = match intervals.as_slice() {
        // Triads
        [0, 4, 7] => ("".to_string(), false, false), // Major
        [0, 3, 7] => ("m".to_string(), false, false), // Minor
        [0, 3, 6] => ("dim".to_string(), false, false), // Diminished
        [0, 4, 8] => ("aug".to_string(), false, false), // Augmented

        // Seventh chords
        [0, 4, 7, 11] => ("maj7".to_string(), true, false), // Major 7th
        [0, 3, 7, 10] => ("m7".to_string(), true, false),   // Minor 7th
        [0, 4, 7, 10] => ("7".to_string(), true, false),    // Dominant 7th
        [0, 3, 6, 9] => ("dim7".to_string(), true, false),  // Diminished 7th
        [0, 3, 6, 10] => ("m7b5".to_string(), true, false), // Half-diminished 7th

        // Extended chords (9ths, 11ths, 13ths)
        [0, 4, 7, 11, 2] | [0, 2, 4, 7, 11] => ("maj9".to_string(), true, true),
        [0, 3, 7, 10, 2] | [0, 2, 3, 7, 10] => ("m9".to_string(), true, true),
        [0, 4, 7, 10, 2] | [0, 2, 4, 7, 10] => ("9".to_string(), true, true),

        // Fallback: determine quality from thirds
        _ => {
            let has_major_third = intervals.contains(&4);
            let has_minor_third = intervals.contains(&3);
            let has_seventh = intervals.contains(&10) || intervals.contains(&11);
            let has_ninth = intervals.contains(&2);
            let has_eleventh = intervals.contains(&5);
            let has_thirteenth = intervals.contains(&9);

            let quality = if has_major_third {
                "".to_string()
            } else if has_minor_third {
                "m".to_string()
            } else {
                return None;
            };

            let is_extended = has_ninth || has_eleventh || has_thirteenth;
            (quality, has_seventh, is_extended)
        },
    };

    Some((root_name, chord_type, is_seventh, is_extended))
}

/// Convert MIDI pitch (0-11) to note name
fn note_name_from_pitch(pitch: u8) -> String {
    let names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    names[(pitch % 12) as usize].to_string()
}

/// Calculate complexity score based on chord types
fn calculate_complexity(chords: &[DetectedChord]) -> f32 {
    if chords.is_empty() {
        return 0.0;
    }

    let mut total_score = 0.0;

    for chord in chords {
        let score = if chord.is_extended {
            1.0 // Extended chords (9ths, 11ths, 13ths)
        } else if chord.is_seventh {
            0.6 // Seventh chords
        } else {
            0.3 // Triads
        };
        total_score += score;
    }

    // Normalize to 0-1 range
    (total_score / chords.len() as f32).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identify_major_chord() {
        let notes = vec![0, 4, 7]; // C major
        let result = identify_chord_type(&notes);
        assert!(result.is_some());
        let (root, chord_type, is_seventh, is_extended) = result.unwrap();
        assert_eq!(root, "C");
        assert_eq!(chord_type, "");
        assert!(!is_seventh);
        assert!(!is_extended);
    }

    #[test]
    fn test_identify_minor_chord() {
        let notes = vec![0, 3, 7]; // C minor
        let result = identify_chord_type(&notes);
        assert!(result.is_some());
        let (root, chord_type, is_seventh, is_extended) = result.unwrap();
        assert_eq!(root, "C");
        assert_eq!(chord_type, "m");
        assert!(!is_seventh);
        assert!(!is_extended);
    }

    #[test]
    fn test_identify_seventh_chord() {
        let notes = vec![0, 4, 7, 10]; // C dominant 7th
        let result = identify_chord_type(&notes);
        assert!(result.is_some());
        let (root, chord_type, is_seventh, is_extended) = result.unwrap();
        assert_eq!(root, "C");
        assert_eq!(chord_type, "7");
        assert!(is_seventh);
        assert!(!is_extended);
    }

    #[test]
    fn test_complexity_calculation() {
        let chords = vec![
            DetectedChord {
                tick: 0,
                notes: vec![60, 64, 67],
                name: "C".to_string(),
                chord_type: "".to_string(),
                is_seventh: false,
                is_extended: false,
            },
            DetectedChord {
                tick: 480,
                notes: vec![60, 64, 67, 70],
                name: "C7".to_string(),
                chord_type: "7".to_string(),
                is_seventh: true,
                is_extended: false,
            },
        ];

        let score = calculate_complexity(&chords);
        assert!(score > 0.4 && score < 0.5); // Average of 0.3 and 0.6
    }
}

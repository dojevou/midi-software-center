//! MIDI meta event extraction (tempo, key, time signature changes)

use midi_library_shared::core::midi::types::{Event, MidiFile};

/// Extract time signature from MIDI file
pub fn extract_time_signature(midi_file: &MidiFile) -> (Option<i16>, Option<i16>) {
    for track in &midi_file.tracks {
        for timed_event in &track.events {
            if let Event::TimeSignature { numerator, denominator, .. } = &timed_event.event {
                let denom_value = 2i16.pow(*denominator as u32);
                return (Some(*numerator as i16), Some(denom_value));
            }
        }
    }
    (Some(4), Some(4))
}

/// Calculate total number of ticks in MIDI file
pub fn calculate_total_ticks(midi_file: &MidiFile) -> i32 {
    let mut max_ticks = 0u32;
    for track in &midi_file.tracks {
        let mut track_ticks = 0u32;
        for timed_event in &track.events {
            track_ticks += timed_event.delta_ticks;
        }
        max_ticks = max_ticks.max(track_ticks);
    }
    max_ticks as i32
}

/// Calculate duration in seconds
pub fn calculate_duration_seconds(midi_file: &MidiFile, bpm: f64) -> Option<f64> {
    let total_ticks = calculate_total_ticks(midi_file) as f64;
    let ticks_per_quarter = midi_file.header.ticks_per_quarter_note as f64;

    if total_ticks > 0.0 && ticks_per_quarter > 0.0 && bpm > 0.0 {
        let quarters = total_ticks / ticks_per_quarter;
        let minutes = quarters / bpm;
        Some(minutes * 60.0)
    } else {
        None
    }
}

/// Extract tempo changes from MIDI meta events
pub fn extract_tempo_changes(midi_file: &MidiFile) -> Option<String> {
    let mut tempo_changes = Vec::new();

    for track in &midi_file.tracks {
        let mut track_tick = 0u32;
        for timed_event in &track.events {
            track_tick += timed_event.delta_ticks;

            if let Event::TempoChange { microseconds_per_quarter } = &timed_event.event {
                let bpm = 60_000_000.0 / *microseconds_per_quarter as f64;
                tempo_changes.push(serde_json::json!({
                    "tick": track_tick,
                    "bpm": (bpm * 100.0).round() / 100.0
                }));
            }
        }
    }

    if tempo_changes.is_empty() {
        None
    } else {
        serde_json::to_string(&tempo_changes).ok()
    }
}

/// Extract key signature changes from MIDI meta events
pub fn extract_key_changes(midi_file: &MidiFile) -> Option<String> {
    let mut key_changes = Vec::new();

    for track in &midi_file.tracks {
        let mut track_tick = 0u32;
        for timed_event in &track.events {
            track_tick += timed_event.delta_ticks;

            if let Event::KeySignature { sharps_flats, is_minor } = &timed_event.event {
                let key_name = get_key_name(*sharps_flats, *is_minor);
                key_changes.push(serde_json::json!({
                    "tick": track_tick,
                    "key": key_name
                }));
            }
        }
    }

    if key_changes.is_empty() {
        None
    } else {
        serde_json::to_string(&key_changes).ok()
    }
}

/// Extract time signature changes from MIDI meta events
pub fn extract_time_signature_changes(midi_file: &MidiFile) -> Option<String> {
    let mut time_sig_changes = Vec::new();

    for track in &midi_file.tracks {
        let mut track_tick = 0u32;
        for timed_event in &track.events {
            track_tick += timed_event.delta_ticks;

            if let Event::TimeSignature { numerator, denominator, .. } = &timed_event.event {
                let denom_value = 2i32.pow(*denominator as u32);
                time_sig_changes.push(serde_json::json!({
                    "tick": track_tick,
                    "numerator": numerator,
                    "denominator": denom_value
                }));
            }
        }
    }

    if time_sig_changes.is_empty() {
        None
    } else {
        serde_json::to_string(&time_sig_changes).ok()
    }
}

/// Convert sharps/flats to key name
pub fn get_key_name(sharps_flats: i8, is_minor: bool) -> String {
    let major_keys = [
        "C", "G", "D", "A", "E", "B", "F#", "C#", "F", "Bb", "Eb", "Ab", "Db", "Gb", "Cb",
    ];
    let minor_keys = [
        "Am", "Em", "Bm", "F#m", "C#m", "G#m", "D#m", "A#m", "Dm", "Gm", "Cm", "Fm", "Bbm", "Ebm",
        "Abm",
    ];

    let index = if sharps_flats >= 0 {
        sharps_flats as usize
    } else {
        (8 - sharps_flats - 1) as usize
    };

    if is_minor {
        minor_keys.get(index).unwrap_or(&"Unknown").to_string()
    } else {
        major_keys.get(index).unwrap_or(&"Unknown").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use midi_library_shared::core::midi::types::{Header, TimedEvent, Track};

    #[test]
    fn test_get_key_name() {
        assert_eq!(get_key_name(0, false), "C");
        assert_eq!(get_key_name(1, false), "G");
        assert_eq!(get_key_name(-1, false), "F");
        assert_eq!(get_key_name(0, true), "Am");
    }

    #[test]
    fn test_extract_tempo_changes() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track {
                events: vec![TimedEvent {
                    delta_ticks: 0,
                    event: Event::TempoChange { microseconds_per_quarter: 500_000 },
                }],
            }],
        };

        let result = extract_tempo_changes(&midi);
        assert!(result.is_some());
        assert!(result.unwrap().contains("120"));
    }
}

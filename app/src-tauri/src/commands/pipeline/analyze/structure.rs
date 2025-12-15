//! Musical structure and form analysis

use crate::core::midi::types::{Event, MidiFile};
use std::collections::HashMap;

/// Analyze musical structure and form
pub fn analyze_structure(midi_file: &MidiFile) -> Option<String> {
    let tpq = midi_file.header.ticks_per_quarter_note as u32;
    let segment_size = tpq * 16; // 4 measures in 4/4 time

    let mut max_tick = 0u32;
    for track in &midi_file.tracks {
        let mut current_tick = 0u32;
        for event in &track.events {
            current_tick += event.delta_ticks;
        }
        max_tick = max_tick.max(current_tick);
    }

    if max_tick < segment_size {
        return None;
    }

    let num_segments = (max_tick / segment_size) as usize;
    let mut segment_hashes: Vec<u64> = vec![0; num_segments];

    for track in &midi_file.tracks {
        let mut current_tick = 0u32;
        for event in &track.events {
            current_tick += event.delta_ticks;

            if let Event::NoteOn { note, velocity, .. } = &event.event {
                if *velocity > 0 {
                    let segment_idx = (current_tick / segment_size) as usize;
                    if segment_idx < num_segments {
                        let position = current_tick % segment_size;
                        let hash = ((*note as u64) << 32) | position as u64;
                        segment_hashes[segment_idx] ^= hash;
                    }
                }
            }
        }
    }

    let mut pattern_map: HashMap<u64, Vec<usize>> = HashMap::new();
    for (idx, &hash) in segment_hashes.iter().enumerate() {
        if hash != 0 {
            pattern_map.entry(hash).or_default().push(idx);
        }
    }

    let mut repeated_sections: Vec<(usize, usize)> = Vec::new();
    let mut pattern_id = 0;
    for (_hash, positions) in pattern_map.iter() {
        if positions.len() >= 2 {
            repeated_sections.push((pattern_id, positions.len()));
            pattern_id += 1;
        }
    }

    let total_repeated: usize = repeated_sections.iter().map(|(_, count)| count).sum();
    let repetition_percentage = if num_segments > 0 {
        (total_repeated as f64 / num_segments as f64) * 100.0
    } else {
        0.0
    };

    let num_unique = pattern_map.len();
    let estimated_form = if num_unique <= 2 {
        "Simple (AA or AB)"
    } else if num_unique <= 4 {
        "Song Form (AABA or ABAB)"
    } else if num_unique <= 6 {
        "Complex (ABABCB or similar)"
    } else {
        "Through-composed"
    };

    let result = serde_json::json!({
        "num_segments": num_segments,
        "num_unique_patterns": num_unique,
        "num_repeated_patterns": repeated_sections.len(),
        "repetition_percentage": (repetition_percentage * 100.0).round() / 100.0,
        "estimated_form": estimated_form,
        "has_repetition": repetition_percentage > 20.0,
        "is_through_composed": num_unique > 6,
    });

    serde_json::to_string(&result).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::midi::types::{Header, TimedEvent, Track};

    #[test]
    fn test_analyze_structure() {
        let tpq = 480;
        let measure_ticks = (tpq * 4) as u32;

        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: tpq },
            tracks: vec![Track {
                events: vec![
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                    },
                    TimedEvent {
                        delta_ticks: measure_ticks * 4,
                        event: Event::NoteOff { channel: 0, note: 60, velocity: 64 },
                    },
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                    },
                    TimedEvent {
                        delta_ticks: measure_ticks * 4,
                        event: Event::NoteOff { channel: 0, note: 60, velocity: 64 },
                    },
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::NoteOn { channel: 0, note: 64, velocity: 100 },
                    },
                    TimedEvent {
                        delta_ticks: measure_ticks * 4,
                        event: Event::NoteOff { channel: 0, note: 64, velocity: 64 },
                    },
                ],
            }],
        };

        let result = analyze_structure(&midi);
        assert!(result.is_some());

        let parsed: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert!(parsed["num_segments"].as_u64().unwrap() >= 3);
    }

    #[test]
    fn test_analyze_structure_too_short() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track {
                events: vec![
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                    },
                    TimedEvent {
                        delta_ticks: 100,
                        event: Event::NoteOff { channel: 0, note: 60, velocity: 64 },
                    },
                ],
            }],
        };

        assert!(analyze_structure(&midi).is_none());
    }
}

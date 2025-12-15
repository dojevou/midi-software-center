//! Articulation and performance characteristics analysis

use crate::core::midi::types::{Event, MidiFile};
use std::collections::{HashMap, HashSet};

/// Analyze articulation and performance characteristics
pub fn analyze_articulation(midi_file: &MidiFile, tempo_us_per_qn: u32) -> Option<String> {
    let tpq = midi_file.header.ticks_per_quarter_note as f64;
    let us_per_tick = tempo_us_per_qn as f64 / tpq;
    let ms_per_tick = us_per_tick / 1000.0;

    let mut note_events: HashMap<(u8, u8), (u32, u8)> = HashMap::new();
    let mut note_durations: Vec<f64> = Vec::new();
    let mut note_velocities: Vec<u8> = Vec::new();
    let mut timing_deviations: Vec<f64> = Vec::new();
    let mut legato_count = 0;
    let mut staccato_count = 0;
    let mut total_notes = 0;

    for track in &midi_file.tracks {
        let mut current_tick = 0u32;
        let mut active_notes: HashSet<(u8, u8)> = HashSet::new();

        for timed_event in &track.events {
            current_tick += timed_event.delta_ticks;

            match &timed_event.event {
                Event::NoteOn { channel, note, velocity } if *velocity > 0 => {
                    if !active_notes.is_empty() {
                        legato_count += 1;
                    }

                    active_notes.insert((*channel, *note));
                    note_events.insert((*channel, *note), (current_tick, *velocity));
                    note_velocities.push(*velocity);

                    let grid_size = (tpq / 4.0) as u32;
                    let deviation = (current_tick % grid_size) as f64;
                    let normalized_deviation = if deviation > grid_size as f64 / 2.0 {
                        grid_size as f64 - deviation
                    } else {
                        deviation
                    };
                    timing_deviations.push(normalized_deviation * ms_per_tick);

                    total_notes += 1;
                },
                Event::NoteOff { channel, note, .. }
                | Event::NoteOn { channel, note, velocity: 0 } => {
                    if let Some((start_tick, _)) = note_events.remove(&(*channel, *note)) {
                        let duration_ticks = current_tick.saturating_sub(start_tick);
                        let duration_ms = duration_ticks as f64 * ms_per_tick;
                        note_durations.push(duration_ms);

                        if duration_ms < 100.0 {
                            staccato_count += 1;
                        }
                    }
                    active_notes.remove(&(*channel, *note));
                },
                _ => {},
            }
        }
    }

    if total_notes == 0 {
        return None;
    }

    let legato_percentage = (legato_count as f64 / total_notes as f64) * 100.0;
    let staccato_percentage = (staccato_count as f64 / total_notes as f64) * 100.0;
    let avg_note_duration = note_durations.iter().sum::<f64>() / note_durations.len().max(1) as f64;
    let avg_timing_deviation =
        timing_deviations.iter().sum::<f64>() / timing_deviations.len().max(1) as f64;

    let min_velocity = *note_velocities.iter().min().unwrap_or(&0);
    let max_velocity = *note_velocities.iter().max().unwrap_or(&127);
    let dynamic_range = max_velocity.saturating_sub(min_velocity);

    let avg_velocity = note_velocities.iter().map(|&v| v as f64).sum::<f64>()
        / note_velocities.len().max(1) as f64;
    let velocity_variance = note_velocities
        .iter()
        .map(|&v| {
            let diff = v as f64 - avg_velocity;
            diff * diff
        })
        .sum::<f64>()
        / note_velocities.len().max(1) as f64;

    let result = serde_json::json!({
        "legato_percentage": (legato_percentage * 100.0).round() / 100.0,
        "staccato_percentage": (staccato_percentage * 100.0).round() / 100.0,
        "avg_note_duration_ms": (avg_note_duration * 100.0).round() / 100.0,
        "timing_deviation_ms": (avg_timing_deviation * 100.0).round() / 100.0,
        "dynamic_range": dynamic_range,
        "velocity_variance": (velocity_variance * 100.0).round() / 100.0,
        "is_humanized": avg_timing_deviation > 2.0,
        "is_legato": legato_percentage > 30.0,
        "is_staccato": staccato_percentage > 50.0,
    });

    serde_json::to_string(&result).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::midi::types::{Header, TimedEvent, Track};

    #[test]
    fn test_analyze_articulation() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track {
                events: vec![
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                    },
                    TimedEvent {
                        delta_ticks: 240,
                        event: Event::NoteOff { channel: 0, note: 60, velocity: 64 },
                    },
                ],
            }],
        };

        let result = analyze_articulation(&midi, 500_000);
        assert!(result.is_some());
    }

    #[test]
    fn test_analyze_articulation_empty() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track { events: vec![] }],
        };

        assert!(analyze_articulation(&midi, 500_000).is_none());
    }
}

//! MIDI controller (CC) message analysis

use crate::core::midi::types::{Event, MidiFile};
use std::collections::HashMap;

/// Analyze MIDI controller (CC) messages
pub fn analyze_controllers(midi_file: &MidiFile) -> Option<String> {
    let mut controller_data: HashMap<u8, (u32, u8, u8, u64)> = HashMap::new();
    let priority_controllers = [1, 2, 7, 10, 11, 64];

    for track in &midi_file.tracks {
        for timed_event in &track.events {
            if let Event::ControlChange { controller, value, .. } = &timed_event.event {
                let entry = controller_data.entry(*controller).or_insert((0, 255, 0, 0));
                entry.0 += 1;
                entry.1 = entry.1.min(*value);
                entry.2 = entry.2.max(*value);
                entry.3 += *value as u64;
            }
        }
    }

    if controller_data.is_empty() {
        return None;
    }

    let mut controllers = Vec::new();

    for &cc in &priority_controllers {
        if let Some(&(count, min, max, sum)) = controller_data.get(&cc) {
            let avg = sum as f64 / count as f64;
            controllers.push(serde_json::json!({
                "cc": cc,
                "name": get_cc_name(cc),
                "count": count,
                "min": min,
                "max": max,
                "avg": (avg * 100.0).round() / 100.0
            }));
        }
    }

    for (&cc, &(count, min, max, sum)) in &controller_data {
        if !priority_controllers.contains(&cc) && count > 10 {
            let avg = sum as f64 / count as f64;
            controllers.push(serde_json::json!({
                "cc": cc,
                "name": get_cc_name(cc),
                "count": count,
                "min": min,
                "max": max,
                "avg": (avg * 100.0).round() / 100.0
            }));
        }
    }

    if controllers.is_empty() {
        None
    } else {
        serde_json::to_string(&controllers).ok()
    }
}

/// Get human-readable name for CC number
pub fn get_cc_name(cc: u8) -> &'static str {
    match cc {
        0 => "Bank Select",
        1 => "Modulation Wheel",
        2 => "Breath Controller",
        4 => "Foot Controller",
        5 => "Portamento Time",
        6 => "Data Entry",
        7 => "Channel Volume",
        8 => "Balance",
        10 => "Pan",
        11 => "Expression",
        64 => "Sustain Pedal",
        65 => "Portamento",
        66 => "Sostenuto",
        67 => "Soft Pedal",
        68 => "Legato Footswitch",
        69 => "Hold 2",
        71 => "Resonance",
        72 => "Release Time",
        73 => "Attack Time",
        74 => "Cutoff Frequency",
        84 => "Portamento Control",
        91 => "Reverb",
        92 => "Tremolo",
        93 => "Chorus",
        94 => "Detune",
        95 => "Phaser",
        _ => "Other",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::midi::types::{Header, TimedEvent, Track};

    #[test]
    fn test_get_cc_name() {
        assert_eq!(get_cc_name(1), "Modulation Wheel");
        assert_eq!(get_cc_name(7), "Channel Volume");
        assert_eq!(get_cc_name(64), "Sustain Pedal");
        assert_eq!(get_cc_name(99), "Other");
    }

    #[test]
    fn test_analyze_controllers() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track {
                events: vec![
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::ControlChange { channel: 0, controller: 7, value: 100 },
                    },
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::ControlChange { channel: 0, controller: 1, value: 50 },
                    },
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::ControlChange { channel: 0, controller: 1, value: 70 },
                    },
                ],
            }],
        };

        let result = analyze_controllers(&midi);
        assert!(result.is_some());
    }

    #[test]
    fn test_analyze_controllers_empty() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track {
                events: vec![TimedEvent {
                    delta_ticks: 0,
                    event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                }],
            }],
        };

        assert!(analyze_controllers(&midi).is_none());
    }
}

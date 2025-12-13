//! MIDI feature detection helpers

use super::gm_instruments::program_to_instrument_name;
use midi_library_shared::core::midi::types::{Event, MidiFile, TextType};

/// Detect if MIDI file contains pitch bend events
pub fn detect_pitch_bend(midi_file: &MidiFile) -> bool {
    for track in &midi_file.tracks {
        for timed_event in &track.events {
            if matches!(&timed_event.event, Event::PitchBend { .. }) {
                return true;
            }
        }
    }
    false
}

/// Detect if MIDI file contains control change messages
pub fn detect_cc_messages(midi_file: &MidiFile) -> bool {
    for track in &midi_file.tracks {
        for timed_event in &track.events {
            if matches!(&timed_event.event, Event::ControlChange { .. }) {
                return true;
            }
        }
    }
    false
}

/// Extract instrument names from MIDI file
pub fn extract_instrument_names(midi_file: &MidiFile) -> Vec<String> {
    let mut instruments = Vec::new();

    for track in &midi_file.tracks {
        for timed_event in &track.events {
            match &timed_event.event {
                Event::Text { text_type, text } => {
                    if matches!(text_type, TextType::InstrumentName | TextType::TrackName)
                        && !instruments.contains(text)
                    {
                        instruments.push(text.clone());
                    }
                }
                Event::ProgramChange { program, .. } => {
                    if let Some(instrument_name) = program_to_instrument_name(*program) {
                        if !instruments.contains(&instrument_name) {
                            instruments.push(instrument_name);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    instruments
}

#[cfg(test)]
mod tests {
    use super::*;
    use midi_library_shared::core::midi::types::{Header, TimedEvent, Track};

    #[test]
    fn test_detect_pitch_bend() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track {
                events: vec![TimedEvent {
                    delta_ticks: 0,
                    event: Event::PitchBend { channel: 0, value: 8192 },
                }],
            }],
        };

        assert!(detect_pitch_bend(&midi));
    }

    #[test]
    fn test_detect_cc_messages() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track {
                events: vec![TimedEvent {
                    delta_ticks: 0,
                    event: Event::ControlChange { channel: 0, controller: 7, value: 100 },
                }],
            }],
        };

        assert!(detect_cc_messages(&midi));
    }
}

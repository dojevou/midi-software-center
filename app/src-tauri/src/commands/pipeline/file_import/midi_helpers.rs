//! MIDI extraction helpers for file import

use super::gm_instruments::program_to_instrument_name;
use crate::core::midi::types::{Event, MidiFile, TextType};

/// Extract instrument names from MIDI file for tag extraction
pub fn extract_instrument_names(midi: &MidiFile) -> Vec<String> {
    let mut instruments = Vec::new();

    for track in &midi.tracks {
        for timed_event in &track.events {
            match &timed_event.event {
                Event::Text { text_type, text } => {
                    if matches!(text_type, TextType::InstrumentName | TextType::TrackName) {
                        instruments.push(text.clone());
                    }
                },
                Event::ProgramChange { program, .. } => {
                    if let Some(instrument_name) = program_to_instrument_name(*program) {
                        instruments.push(instrument_name);
                    }
                },
                _ => {},
            }
        }
    }

    instruments
}

/// Extract time signature from MIDI file events
/// Returns format like "4-4" for 4/4 time, or None if not found
pub fn extract_time_signature(midi: &MidiFile) -> Option<String> {
    for track in &midi.tracks {
        for timed_event in &track.events {
            if let Event::TimeSignature { numerator, denominator, .. } = &timed_event.event {
                let denom_value = 2_u8.pow(*denominator as u32);
                return Some(format!("{}-{}", numerator, denom_value));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::midi::types::Header;

    fn empty_midi() -> MidiFile {
        MidiFile {
            header: Header { format: 0, num_tracks: 0, ticks_per_quarter_note: 480 },
            tracks: vec![],
        }
    }

    #[test]
    fn test_extract_time_signature_none() {
        assert!(extract_time_signature(&empty_midi()).is_none());
    }

    #[test]
    fn test_extract_instruments_empty() {
        assert!(extract_instrument_names(&empty_midi()).is_empty());
    }
}

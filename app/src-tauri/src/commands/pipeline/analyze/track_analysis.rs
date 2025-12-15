//! Track-level instrument analysis

use super::gm_instruments::get_instrument_info;
use super::types::TrackInstrument;
use crate::core::midi::types::{Event, MidiFile};
use std::collections::HashMap;

/// Analyze tracks and extract per-channel instrument information
pub fn analyze_tracks(midi_file: &MidiFile) -> Vec<TrackInstrument> {
    let mut channel_data: HashMap<u8, TrackInstrument> = HashMap::new();

    for track in &midi_file.tracks {
        let mut current_programs: HashMap<u8, u8> = HashMap::new();

        for timed_event in &track.events {
            match &timed_event.event {
                Event::ProgramChange { channel, program } => {
                    current_programs.insert(*channel, *program);
                },
                Event::NoteOn { channel, note, velocity } if *velocity > 0 => {
                    let program = current_programs.get(channel).copied().unwrap_or(0);

                    let entry = channel_data.entry(*channel).or_insert_with(|| {
                        let (name, family, inst_type) = get_instrument_info(program);
                        TrackInstrument {
                            channel: *channel as i16,
                            program_number: program as i16,
                            program_name: name,
                            instrument_family: family,
                            instrument_type: inst_type,
                            note_count: 0,
                            avg_velocity: None,
                            pitch_range_low: None,
                            pitch_range_high: None,
                            is_primary: false,
                        }
                    });

                    entry.note_count += 1;

                    entry.pitch_range_low =
                        Some(entry.pitch_range_low.map_or(*note as i16, |l| l.min(*note as i16)));
                    entry.pitch_range_high =
                        Some(entry.pitch_range_high.map_or(*note as i16, |h| h.max(*note as i16)));

                    if let Some(avg) = entry.avg_velocity {
                        entry.avg_velocity = Some(
                            (avg * (entry.note_count - 1) as f64 + *velocity as f64)
                                / entry.note_count as f64,
                        );
                    } else {
                        entry.avg_velocity = Some(*velocity as f64);
                    }
                },
                _ => {},
            }
        }
    }

    let mut instruments: Vec<TrackInstrument> = channel_data.into_values().collect();

    if let Some(max_notes) = instruments.iter().map(|i| i.note_count).max() {
        for inst in &mut instruments {
            if inst.note_count == max_notes {
                inst.is_primary = true;
                break;
            }
        }
    }

    instruments
}

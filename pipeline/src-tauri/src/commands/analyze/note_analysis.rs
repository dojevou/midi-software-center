//! Note statistics analysis for MIDI files

use super::types::NoteStats;
use midi_library_shared::core::midi::types::{Event, MidiFile};
use std::collections::{HashMap, HashSet};

/// Analyze notes in MIDI file
pub fn analyze_notes(midi_file: &MidiFile) -> NoteStats {
    let mut note_count = 0;
    let mut unique_pitch_set = HashSet::new();
    let mut min_pitch = 127u8;
    let mut max_pitch = 0u8;
    let mut min_velocity = 127u8;
    let mut max_velocity = 0u8;
    let mut velocity_sum = 0u32;
    let mut active_notes_per_tick: HashMap<u32, usize> = HashMap::new();
    let mut max_tick = 0u32;
    let mut percussive_note_count = 0;

    for track in &midi_file.tracks {
        let mut current_tick = 0u32;
        let mut active_notes = HashSet::new();

        for timed_event in &track.events {
            current_tick += timed_event.delta_ticks;
            max_tick = max_tick.max(current_tick);

            match &timed_event.event {
                Event::NoteOn { note, velocity, channel } if *velocity > 0 => {
                    note_count += 1;
                    unique_pitch_set.insert(*note);
                    min_pitch = min_pitch.min(*note);
                    max_pitch = max_pitch.max(*note);
                    min_velocity = min_velocity.min(*velocity);
                    max_velocity = max_velocity.max(*velocity);
                    velocity_sum += *velocity as u32;

                    if *channel == 9 {
                        percussive_note_count += 1;
                    }

                    active_notes.insert(*note);
                    active_notes_per_tick.insert(current_tick, active_notes.len());
                },
                Event::NoteOff { note, .. } | Event::NoteOn { note, velocity: 0, .. } => {
                    active_notes.remove(note);
                },
                _ => {},
            }
        }
    }

    if note_count == 0 {
        return NoteStats::default();
    }

    let unique_pitches = Some(unique_pitch_set.len() as i32);
    let avg_velocity = Some(velocity_sum as f64 / note_count as f64);
    let polyphony_max = active_notes_per_tick.values().max().copied().map(|v| v as i16);

    let polyphony_avg = if !active_notes_per_tick.is_empty() {
        let sum: usize = active_notes_per_tick.values().sum();
        Some(sum as f64 / active_notes_per_tick.len() as f64)
    } else {
        None
    };

    let is_monophonic = polyphony_max == Some(1);
    let is_polyphonic = polyphony_max.is_some_and(|p| p > 1);

    let is_percussive = {
        let drum_ratio = percussive_note_count as f64 / note_count as f64;
        let in_drum_range = min_pitch >= 27 && max_pitch <= 87;
        drum_ratio > 0.5 || in_drum_range
    };

    let duration_seconds = if max_tick > 0 {
        let ticks_per_beat = midi_file.header.ticks_per_quarter_note as f64;
        let seconds_per_beat = 0.5;
        Some((max_tick as f64 / ticks_per_beat) * seconds_per_beat)
    } else {
        None
    };

    let note_density = duration_seconds.and_then(|d| {
        if d > 0.0 {
            Some(note_count as f64 / d)
        } else {
            None
        }
    });

    let semitones = max_pitch.saturating_sub(min_pitch) as i16;

    NoteStats {
        note_count,
        unique_pitches,
        pitch_range_low: Some(min_pitch as i16),
        pitch_range_high: Some(max_pitch as i16),
        pitch_range_semitones: Some(semitones),
        avg_velocity,
        velocity_range_low: Some(min_velocity as i16),
        velocity_range_high: Some(max_velocity as i16),
        note_density,
        polyphony_max,
        polyphony_avg,
        is_monophonic,
        is_polyphonic,
        is_percussive,
    }
}

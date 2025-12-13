//! Complexity scoring for MIDI files

use super::meta_events::calculate_total_ticks;
use super::types::NoteStats;
use midi_library_shared::core::midi::types::MidiFile;

/// Calculate complexity score based on various factors
pub fn calculate_complexity_score(note_stats: &NoteStats, midi_file: &MidiFile) -> Option<f64> {
    if note_stats.note_count == 0 {
        return Some(0.0);
    }

    let mut score = 0.0;

    // Factor 1: Note density (notes per second)
    let duration_est = calculate_total_ticks(midi_file) as f64
        / (midi_file.header.ticks_per_quarter_note as f64 * 2.0);
    if duration_est > 0.0 {
        let note_density = note_stats.note_count as f64 / duration_est;
        score += (note_density / 10.0).min(30.0);
    }

    // Factor 2: Pitch range
    if let Some(semitones) = note_stats.pitch_range_semitones {
        score += (semitones as f64 / 2.0).min(20.0);
    }

    // Factor 3: Polyphony
    if let Some(polyphony) = note_stats.polyphony_max {
        score += (polyphony as f64 * 5.0).min(25.0);
    }

    // Factor 4: Track count
    let track_count = midi_file.tracks.len() as f64;
    score += (track_count * 2.0).min(15.0);

    // Factor 5: Velocity variation
    if let (Some(low), Some(high)) = (note_stats.velocity_range_low, note_stats.velocity_range_high)
    {
        let velocity_range = (high - low) as f64;
        score += (velocity_range / 10.0).min(10.0);
    }

    Some(score.min(100.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use midi_library_shared::core::midi::types::Header;

    #[test]
    fn test_complexity_score_empty() {
        let note_stats = NoteStats::default();
        let midi_file = MidiFile {
            header: Header { format: 0, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![],
        };

        let score = calculate_complexity_score(&note_stats, &midi_file);
        assert_eq!(score, Some(0.0));
    }
}

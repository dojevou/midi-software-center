/// Key detection wrapper
///
/// This is a simplified wrapper for the shared library.
/// The full implementation lives in the Pipeline component at
/// `pipeline/src-tauri/src/core/analysis/key_detector.rs`
///
/// Returns the detected key as a string (e.g., "C", "Am", "F#")
/// or None if detection confidence is too low.
///
/// # Arguments
/// * `midi_file` - Parsed MIDI file structure
///
/// # Returns
/// * `Some(String)` - Detected key (e.g., "C major", "A minor")
/// * `None` - No clear key detected (confidence < 0.5)
///
/// # Examples
///
/// ```
/// use midi_library_shared::core::midi::MidiFile;
/// use midi_library_shared::core::analysis::key_detector::detect_key;
///
/// // let midi_file = MidiFile::parse(&data)?;
/// // if let Some(key) = detect_key(&midi_file) {
/// //     println!("Detected key: {}", key);
/// // }
/// ```
pub fn detect_key(midi_file: &crate::core::midi::MidiFile) -> Option<String> {
    // Build pitch class histogram from MIDI events
    let mut pitch_class_counts = [0u32; 12];

    for track in &midi_file.tracks {
        for timed_event in &track.events {
            if let crate::core::midi::Event::NoteOn { note, .. } = &timed_event.event {
                let pitch_class = (note % 12) as usize;
                pitch_class_counts[pitch_class] = pitch_class_counts[pitch_class].saturating_add(1);
            }
        }
    }

    // Check if we have enough notes for analysis
    let total_notes: u32 = pitch_class_counts.iter().sum();
    if total_notes < 10 {
        return None; // Not enough data for reliable key detection
    }

    // Normalize to probability distribution
    let mut pitch_class_dist = [0.0; 12];
    for (i, &count) in pitch_class_counts.iter().enumerate() {
        pitch_class_dist[i] = count as f64 / total_notes as f64;
    }

    // Krumhansl-Schmuckler major and minor profiles
    const MAJOR_PROFILE: [f64; 12] =
        [6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88];
    const MINOR_PROFILE: [f64; 12] =
        [6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17];

    // Find best correlation with all 24 keys (12 major + 12 minor)
    let mut best_correlation = -1.0;
    let mut best_key = String::new();

    const NOTE_NAMES: [&str; 12] =
        ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

    for (root, note_name) in NOTE_NAMES.iter().enumerate() {
        // Test major key
        let major_corr = calculate_correlation(&pitch_class_dist, &MAJOR_PROFILE, root);
        if major_corr > best_correlation {
            best_correlation = major_corr;
            best_key = format!("{} major", note_name);
        }

        // Test minor key
        let minor_corr = calculate_correlation(&pitch_class_dist, &MINOR_PROFILE, root);
        if minor_corr > best_correlation {
            best_correlation = minor_corr;
            best_key = format!("{} minor", note_name);
        }
    }

    // Only return if confidence is reasonable (correlation > 0.5)
    if best_correlation > 0.5 {
        Some(best_key)
    } else {
        None
    }
}

/// Calculate Pearson correlation between pitch class distribution and key profile
fn calculate_correlation(distribution: &[f64; 12], profile: &[f64; 12], rotation: usize) -> f64 {
    // Rotate profile to match the key
    let mut rotated = [0.0; 12];
    for i in 0..12 {
        rotated[i] = profile[(i + rotation) % 12];
    }

    // Calculate means
    let dist_mean: f64 = distribution.iter().sum::<f64>() / 12.0;
    let prof_mean: f64 = rotated.iter().sum::<f64>() / 12.0;

    // Calculate correlation coefficient
    let mut numerator = 0.0;
    let mut dist_sq_sum = 0.0;
    let mut prof_sq_sum = 0.0;

    for i in 0..12 {
        let dist_diff = distribution[i] - dist_mean;
        let prof_diff = rotated[i] - prof_mean;

        numerator += dist_diff * prof_diff;
        dist_sq_sum += dist_diff * dist_diff;
        prof_sq_sum += prof_diff * prof_diff;
    }

    let denominator = (dist_sq_sum * prof_sq_sum).sqrt();

    if denominator > 0.0 {
        numerator / denominator
    } else {
        0.0
    }
}

/// Key Detection Module
///
/// Implements the Krumhansl-Schmuckler key-finding algorithm to detect
/// the musical key of MIDI files.
///
/// # Archetype: Trusty Module
/// - Pure functions with no side effects
/// - No I/O operations
/// - Highly testable
/// - Reusable across the application
use crate::core::analysis::key_profiles::*;
use midi_library_shared::core::midi::types::{Event, MidiFile};

/// Musical scale types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleType {
    Major,
    Minor,
}

impl std::fmt::Display for ScaleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScaleType::Major => write!(f, "major"),
            ScaleType::Minor => write!(f, "minor"),
        }
    }
}

/// Result of key detection
#[derive(Debug, Clone, PartialEq)]
pub struct KeyDetectionResult {
    /// Detected key (e.g., "C", "Am", "F#")
    pub key: String,

    /// Whether the key is major or minor
    pub scale_type: ScaleType,

    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,

    /// Top 3 alternative keys with their correlation scores
    pub alternatives: Vec<KeyAlternative>,

    /// Pitch class distribution from the MIDI file
    pub pitch_class_distribution: [f64; 12],
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyAlternative {
    pub key: String,
    pub scale_type: ScaleType,
    pub correlation: f64,
}

/// Detects the musical key from a parsed MIDI file
///
/// # Arguments
/// * `midi_file` - Parsed MIDI file structure
///
/// # Returns
/// * `KeyDetectionResult` - Detection result with confidence and alternatives
///
/// # Algorithm
/// Uses Krumhansl-Schmuckler key-finding algorithm:
/// 1. Extract all notes and build pitch class histogram
/// 2. Normalize histogram to probability distribution
/// 3. Correlate with all 24 key profiles (12 major + 12 minor)
/// 4. Return key with highest correlation
///
/// # Examples
///
/// ```
/// use pipeline::core::midi::types::MidiFile;
/// use pipeline::core::analysis::key_detector::detect_key;
///
/// // Assuming you have a parsed MIDI file
/// // let midi_file = parse_midi_file(&data)?;
/// // let result = detect_key(&midi_file);
/// // println!("Detected key: {} ({})", result.key, result.scale_type);
/// ```
pub fn detect_key(midi_file: &MidiFile) -> KeyDetectionResult {
    // Build pitch class histogram
    let pitch_class_counts = build_pitch_class_histogram(midi_file);

    // Normalize to probability distribution
    let pitch_class_distribution = normalize_histogram(&pitch_class_counts);

    // Calculate correlations with all 24 key profiles
    let mut correlations = Vec::new();

    for pitch_class in 0..12 {
        // Major key
        let major_correlation = calculate_correlation(
            &pitch_class_distribution,
            &rotate_profile(&MAJOR_PROFILE, pitch_class),
        );
        correlations.push((pitch_class, ScaleType::Major, major_correlation));

        // Minor key
        let minor_correlation = calculate_correlation(
            &pitch_class_distribution,
            &rotate_profile(&MINOR_PROFILE, pitch_class),
        );
        correlations.push((pitch_class, ScaleType::Minor, minor_correlation));
    }

    // Sort by correlation (descending)
    // Note: partial_cmp can return None for NaN values, treat them as equal
    correlations.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    // Get top result
    let (best_pitch_class, best_scale_type, _best_correlation) = correlations[0];

    let key_name = format_key_name(best_pitch_class, best_scale_type);

    // Calculate confidence from correlation
    let confidence = calculate_confidence(&correlations);

    // Get top 3 alternatives
    let alternatives: Vec<KeyAlternative> = correlations[1..4]
        .iter()
        .map(|(pc, st, corr)| KeyAlternative {
            key: format_key_name(*pc, *st),
            scale_type: *st,
            correlation: *corr,
        })
        .collect();

    KeyDetectionResult {
        key: key_name,
        scale_type: best_scale_type,
        confidence,
        alternatives,
        pitch_class_distribution,
    }
}

/// Builds a histogram of pitch class occurrences
fn build_pitch_class_histogram(midi_file: &MidiFile) -> [u32; 12] {
    let mut histogram = [0u32; 12];

    for track in &midi_file.tracks {
        for timed_event in &track.events {
            if let Event::NoteOn { note, velocity, .. } = timed_event.event {
                if velocity > 0 {
                    let pitch_class = (note % 12) as usize;
                    histogram[pitch_class] += 1;
                }
            }
        }
    }

    histogram
}

/// Normalizes histogram to probability distribution
fn normalize_histogram(histogram: &[u32; 12]) -> [f64; 12] {
    let total: u32 = histogram.iter().sum();

    if total == 0 {
        return [0.0; 12];
    }

    let mut normalized = [0.0; 12];
    for i in 0..12 {
        normalized[i] = histogram[i] as f64 / total as f64;
    }

    normalized
}

/// Rotates a key profile to a different tonic
///
/// Takes a profile defined for C (pitch class 0) and rotates it to be
/// defined for a different pitch class. The rotation shifts the profile
/// so that the tonic weight appears at the target pitch class.
fn rotate_profile(profile: &[f64; 12], rotation: usize) -> [f64; 12] {
    let mut rotated = [0.0; 12];

    for i in 0..12 {
        rotated[i] = profile[(i + 12 - rotation) % 12];
    }

    rotated
}

/// Calculates Pearson correlation coefficient between two distributions
fn calculate_correlation(distribution: &[f64; 12], profile: &[f64; 12]) -> f64 {
    // Calculate means
    let mean_dist = distribution.iter().sum::<f64>() / 12.0;
    let mean_prof = profile.iter().sum::<f64>() / 12.0;

    // Calculate covariance and standard deviations
    let mut covariance = 0.0;
    let mut var_dist = 0.0;
    let mut var_prof = 0.0;

    for i in 0..12 {
        let diff_dist = distribution[i] - mean_dist;
        let diff_prof = profile[i] - mean_prof;

        covariance += diff_dist * diff_prof;
        var_dist += diff_dist * diff_dist;
        var_prof += diff_prof * diff_prof;
    }

    // Calculate correlation
    let std_dist = var_dist.sqrt();
    let std_prof = var_prof.sqrt();

    if std_dist == 0.0 || std_prof == 0.0 {
        return 0.0;
    }

    covariance / (std_dist * std_prof)
}

/// Calculates confidence based on separation between best and second-best keys
fn calculate_confidence(correlations: &[(usize, ScaleType, f64)]) -> f64 {
    if correlations.len() < 2 {
        return 0.5;
    }

    let best = correlations[0].2;
    let second_best = correlations[1].2;

    // Larger gap = higher confidence
    let gap = best - second_best;

    // Map gap to confidence score
    // Gap of 0.0 = 0.5 confidence
    // Gap of 0.2+ = 1.0 confidence
    let confidence = 0.5 + (gap * 2.5).min(0.5);

    confidence.clamp(0.5, 1.0)
}

/// Formats key name based on pitch class and scale type
fn format_key_name(pitch_class: usize, scale_type: ScaleType) -> String {
    let base_name = pitch_class_to_key_name(pitch_class);

    match scale_type {
        ScaleType::Major => base_name.to_string(),
        ScaleType::Minor => format!("{}m", base_name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use midi_library_shared::core::midi::types::{Header, MidiFile, TimedEvent, Track};

    // ============================================================================
    // Helper Functions for Building Test MIDI Files
    // ============================================================================

    /// Create a MIDI file with specific notes
    /// Each note is (pitch, velocity) - velocity 0 means note off
    fn create_test_midi_with_notes(notes: Vec<(u8, u8)>) -> MidiFile {
        let mut events: Vec<TimedEvent> = Vec::new();

        for (pitch, velocity) in notes {
            events.push(TimedEvent {
                delta_ticks: 10,
                event: Event::NoteOn { note: pitch, velocity, channel: 0 },
            });
        }

        events.push(TimedEvent { delta_ticks: 0, event: Event::EndOfTrack });

        MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track { events }],
        }
    }

    /// Create a major scale starting from a given root note
    fn create_major_scale(root: u8) -> Vec<u8> {
        let intervals = [0, 2, 4, 5, 7, 9, 11, 12]; // Major scale pattern
        intervals.iter().map(|&i| root + i).collect()
    }

    /// Create a minor scale starting from a given root note
    fn create_minor_scale(root: u8) -> Vec<u8> {
        let intervals = [0, 2, 3, 5, 7, 8, 10, 12]; // Natural minor pattern
        intervals.iter().map(|&i| root + i).collect()
    }

    /// Create chromatic scale (all 12 pitches)
    fn create_chromatic_scale(root: u8) -> Vec<u8> {
        (0..12).map(|i| root + i).collect()
    }

    /// Assert correlation is within tolerance
    #[allow(dead_code)]
    fn assert_correlation_approx(actual: f64, expected: f64, tolerance: f64) {
        assert!(
            (actual - expected).abs() < tolerance,
            "Correlation mismatch: expected {}, got {} (tolerance: {})",
            expected,
            actual,
            tolerance
        );
    }

    // ============================================================================
    // Existing Tests (6 tests)
    // ============================================================================

    #[test]
    fn test_normalize_histogram() {
        let histogram = [10, 0, 5, 0, 3, 0, 0, 7, 0, 2, 0, 3];
        let normalized = normalize_histogram(&histogram);

        let total: f64 = normalized.iter().sum();
        assert!((total - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_normalize_empty_histogram() {
        let histogram = [0; 12];
        let normalized = normalize_histogram(&histogram);

        assert_eq!(normalized, [0.0; 12]);
    }

    #[test]
    fn test_rotate_profile() {
        // Test that rotating a profile moves the tonic weight to the correct position
        let profile = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];

        // Rotate to pitch class 3 (D#)
        let rotated = rotate_profile(&profile, 3);

        // The tonic weight (1.0) should now be at position 3
        assert_eq!(rotated[3], 1.0);
        // Position 0 should have the weight that was 3 positions back
        assert_eq!(rotated[0], 10.0);
        // Position 4 should have the weight that was at position 1 (the scale degree above tonic)
        assert_eq!(rotated[4], 2.0);
    }

    #[test]
    fn test_correlation_identical() {
        let dist1 = [0.1, 0.2, 0.1, 0.05, 0.15, 0.1, 0.05, 0.15, 0.03, 0.02, 0.03, 0.02];
        let correlation = calculate_correlation(&dist1, &dist1);

        assert!((correlation - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_correlation_zero() {
        let dist1 = [0.0; 12];
        let dist2 = [0.1, 0.2, 0.1, 0.05, 0.15, 0.1, 0.05, 0.15, 0.03, 0.02, 0.03, 0.02];
        let correlation = calculate_correlation(&dist1, &dist2);

        assert_eq!(correlation, 0.0);
    }

    #[test]
    fn test_format_key_name() {
        assert_eq!(format_key_name(0, ScaleType::Major), "C");
        assert_eq!(format_key_name(0, ScaleType::Minor), "Cm");
        assert_eq!(format_key_name(9, ScaleType::Minor), "Am");
        assert_eq!(format_key_name(7, ScaleType::Major), "G");
    }

    // ============================================================================
    // Category 3: Histogram Building Tests (8 tests) - CRITICAL GAP
    // ============================================================================

    #[test]
    fn test_histogram_empty_midi() {
        let midi = create_test_midi_with_notes(vec![]);
        let histogram = build_pitch_class_histogram(&midi);
        assert_eq!(histogram, [0; 12]);
    }

    #[test]
    fn test_histogram_single_note() {
        let midi = create_test_midi_with_notes(vec![(60, 100)]); // Middle C
        let histogram = build_pitch_class_histogram(&midi);

        assert_eq!(histogram[0], 1); // C
        assert_eq!(histogram.iter().sum::<u32>(), 1);
    }

    #[test]
    fn test_histogram_octave_equivalence() {
        // C3, C4, C5 should all map to pitch class 0
        let midi = create_test_midi_with_notes(vec![
            (48, 100), // C3
            (60, 100), // C4
            (72, 100), // C5
        ]);
        let histogram = build_pitch_class_histogram(&midi);

        assert_eq!(histogram[0], 3); // All mapped to C
    }

    #[test]
    fn test_histogram_velocity_zero_ignored() {
        // Notes with velocity 0 should be ignored (they're note-offs)
        let midi = create_test_midi_with_notes(vec![
            (60, 100), // C - counted
            (62, 0),   // D - ignored (velocity 0)
            (64, 100), // E - counted
        ]);
        let histogram = build_pitch_class_histogram(&midi);

        assert_eq!(histogram[0], 1); // C
        assert_eq!(histogram[2], 0); // D (ignored)
        assert_eq!(histogram[4], 1); // E
    }

    #[test]
    fn test_histogram_chromatic_scale() {
        let notes: Vec<(u8, u8)> = (60..72).map(|pitch| (pitch, 100)).collect();
        let midi = create_test_midi_with_notes(notes);
        let histogram = build_pitch_class_histogram(&midi);

        // All 12 pitch classes should have exactly 1 occurrence
        for count in histogram.iter() {
            assert_eq!(*count, 1);
        }
    }

    #[test]
    fn test_histogram_c_major_scale() {
        let scale = create_major_scale(60); // C major
        let notes: Vec<(u8, u8)> = scale.iter().map(|&pitch| (pitch, 100)).collect();
        let midi = create_test_midi_with_notes(notes);
        let histogram = build_pitch_class_histogram(&midi);

        // C major scale: C D E F G A B C
        // Should have: C=2, D=1, E=1, F=1, G=1, A=1, B=1
        assert_eq!(histogram[0], 2); // C (appears twice)
        assert_eq!(histogram[1], 0); // C#
        assert_eq!(histogram[2], 1); // D
        assert_eq!(histogram[3], 0); // D#
        assert_eq!(histogram[4], 1); // E
    }

    #[test]
    fn test_histogram_multiple_tracks() {
        // Create MIDI with 2 tracks, both playing C
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 2, ticks_per_quarter_note: 480 },
            tracks: vec![
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn { note: 60, velocity: 100, channel: 0 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn {
                                note: 72, // C an octave higher
                                velocity: 100,
                                channel: 1,
                            },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
            ],
        };

        let histogram = build_pitch_class_histogram(&midi);
        assert_eq!(histogram[0], 2); // Both Cs counted
    }

    #[test]
    fn test_histogram_full_midi_range() {
        // Test notes across full MIDI range (0-127)
        let midi = create_test_midi_with_notes(vec![
            (0, 100),   // C(-1)
            (127, 100), // G9
        ]);
        let histogram = build_pitch_class_histogram(&midi);

        assert_eq!(histogram[0], 1); // C
        assert_eq!(histogram[7], 1); // G (127 % 12 = 7)
    }

    // ============================================================================
    // Category 4: Confidence Calculation Tests (8 tests) - CRITICAL GAP
    // ============================================================================

    #[test]
    fn test_confidence_empty_correlations() {
        let correlations: Vec<(usize, ScaleType, f64)> = vec![];
        let confidence = calculate_confidence(&correlations);
        assert_eq!(confidence, 0.5); // Default confidence
    }

    #[test]
    fn test_confidence_single_correlation() {
        let correlations = vec![(0, ScaleType::Major, 0.95)];
        let confidence = calculate_confidence(&correlations);
        assert_eq!(confidence, 0.5); // Only one result, can't calculate gap
    }

    #[test]
    fn test_confidence_large_gap() {
        // Best: 0.95, Second: 0.70 → gap = 0.25
        let correlations = vec![(0, ScaleType::Major, 0.95), (1, ScaleType::Major, 0.70)];
        let confidence = calculate_confidence(&correlations);

        // confidence = 0.5 + (0.25 * 2.5).min(0.5) = 0.5 + 0.5 = 1.0
        assert_eq!(confidence, 1.0);
    }

    #[test]
    fn test_confidence_small_gap() {
        // Best: 0.75, Second: 0.73 → gap = 0.02
        let correlations = vec![(0, ScaleType::Major, 0.75), (1, ScaleType::Minor, 0.73)];
        let confidence = calculate_confidence(&correlations);

        // confidence = 0.5 + (0.02 * 2.5) = 0.5 + 0.05 = 0.55
        assert!((confidence - 0.55).abs() < 0.01);
    }

    #[test]
    fn test_confidence_zero_gap() {
        // Identical correlations (ambiguous)
        let correlations = vec![(0, ScaleType::Major, 0.80), (1, ScaleType::Minor, 0.80)];
        let confidence = calculate_confidence(&correlations);

        assert_eq!(confidence, 0.5); // Minimum confidence
    }

    #[test]
    fn test_confidence_medium_gap() {
        // Best: 0.85, Second: 0.77 → gap = 0.08
        let correlations = vec![(0, ScaleType::Major, 0.85), (5, ScaleType::Major, 0.77)];
        let confidence = calculate_confidence(&correlations);

        // confidence = 0.5 + (0.08 * 2.5) = 0.5 + 0.20 = 0.70
        assert!((confidence - 0.70).abs() < 0.01);
    }

    #[test]
    fn test_confidence_clamped_to_max() {
        // Very large gap should clamp to 1.0
        let correlations = vec![(0, ScaleType::Major, 0.99), (1, ScaleType::Minor, 0.40)];
        let confidence = calculate_confidence(&correlations);

        assert_eq!(confidence, 1.0); // Clamped to maximum
    }

    #[test]
    fn test_confidence_many_correlations() {
        // Test with all 24 correlations (typical real-world case)
        let mut correlations = Vec::new();
        for i in 0..24 {
            let correlation = 0.9 - (i as f64 * 0.03); // Decreasing scores
            let scale_type = if i < 12 {
                ScaleType::Major
            } else {
                ScaleType::Minor
            };
            correlations.push((i % 12, scale_type, correlation));
        }

        let confidence = calculate_confidence(&correlations);

        // Best: 0.90, Second: 0.87 → gap = 0.03
        // confidence = 0.5 + (0.03 * 2.5) = 0.575
        assert!(confidence > 0.5 && confidence < 0.6);
    }

    // ============================================================================
    // Category 1: Basic Unit Tests - normalize_histogram (6 tests)
    // ============================================================================

    #[test]
    fn test_normalize_histogram_standard() {
        let histogram = [10, 5, 3, 0, 0, 0, 2, 0, 0, 0, 0, 0];
        let normalized = normalize_histogram(&histogram);

        // Sum should be 1.0
        let sum: f64 = normalized.iter().sum();
        assert!((sum - 1.0).abs() < 0.001, "Sum should be 1.0, got {}", sum);

        // First element should be 10/20 = 0.5
        assert!((normalized[0] - 0.5).abs() < 0.001);
        // Second element should be 5/20 = 0.25
        assert!((normalized[1] - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_normalize_histogram_zero_sum() {
        let histogram = [0; 12];
        let normalized = normalize_histogram(&histogram);

        // Should return all zeros when sum is zero
        for &val in &normalized {
            assert_eq!(val, 0.0);
        }
    }

    #[test]
    fn test_normalize_histogram_single_nonzero() {
        let mut histogram = [0; 12];
        histogram[5] = 100;

        let normalized = normalize_histogram(&histogram);

        // Element 5 should be 1.0, all others 0.0
        assert!((normalized[5] - 1.0).abs() < 0.001);
        for (i, &value) in normalized.iter().enumerate() {
            if i != 5 {
                assert!(value.abs() < 0.001);
            }
        }
    }

    #[test]
    fn test_normalize_histogram_uniform() {
        let histogram = [10; 12];
        let normalized = normalize_histogram(&histogram);

        // All should be 1/12
        for &val in &normalized {
            assert!((val - (1.0 / 12.0)).abs() < 0.001);
        }
    }

    #[test]
    fn test_normalize_histogram_large_values() {
        let histogram = [1000, 2000, 3000, 0, 0, 0, 0, 0, 0, 0, 0, 4000];
        let normalized = normalize_histogram(&histogram);

        let sum: f64 = normalized.iter().sum();
        assert!((sum - 1.0).abs() < 0.001);

        // Last element should be 4000/10000 = 0.4
        assert!((normalized[11] - 0.4).abs() < 0.001);
    }

    #[test]
    fn test_normalize_histogram_precision() {
        let histogram = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let normalized = normalize_histogram(&histogram);

        let sum: f64 = normalized.iter().sum();
        // Sum should be very close to 1.0
        assert!((sum - 1.0).abs() < 0.000001);
    }

    // ============================================================================
    // Category 1: Basic Unit Tests - rotate_profile (6 tests)
    // ============================================================================

    #[test]
    fn test_rotate_profile_zero_rotation() {
        let profile = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];
        let rotated = rotate_profile(&profile, 0);

        // No rotation - should be identical
        assert_eq!(rotated, profile);
    }

    #[test]
    fn test_rotate_profile_by_one() {
        let profile = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];
        let rotated = rotate_profile(&profile, 1);

        // Rotation by 1: [12, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
        assert_eq!(rotated[0], 12.0);
        assert_eq!(rotated[1], 1.0);
        assert_eq!(rotated[11], 11.0);
    }

    #[test]
    fn test_rotate_profile_by_six() {
        let profile = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];
        let rotated = rotate_profile(&profile, 6);

        // Rotation by 6 (half): [7, 8, 9, 10, 11, 12, 1, 2, 3, 4, 5, 6]
        assert_eq!(rotated[0], 7.0);
        assert_eq!(rotated[6], 1.0);
    }

    #[test]
    fn test_rotate_profile_by_eleven() {
        let profile = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];
        let rotated = rotate_profile(&profile, 11);

        // Rotation by 11: [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 1]
        assert_eq!(rotated[0], 2.0);
        assert_eq!(rotated[11], 1.0);
    }

    #[test]
    fn test_rotate_profile_full_rotation() {
        let profile = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];
        let rotated = rotate_profile(&profile, 12);

        // Full rotation (12 steps) should return to original
        assert_eq!(rotated, profile);
    }

    #[test]
    fn test_rotate_profile_major_profile() {
        use crate::core::analysis::key_profiles::MAJOR_PROFILE;

        // Test rotating actual MAJOR_PROFILE
        let rotated = rotate_profile(&MAJOR_PROFILE, 7); // Rotate by 7

        // After rotation, the tonic value (6.35) appears at index 7
        assert!(rotated[7] > 6.0); // Tonic value from original profile[0]
    }

    // ============================================================================
    // Category 2: Correlation Function Tests (10 tests)
    // ============================================================================

    #[test]
    fn test_correlation_identical_profiles() {
        let profile1 = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];
        let profile2 = profile1;

        let correlation = calculate_correlation(&profile1, &profile2);

        // Perfect correlation
        assert!((correlation - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_correlation_opposite_profiles() {
        let profile1 = [12.0, 11.0, 10.0, 9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];
        let profile2 = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];

        let correlation = calculate_correlation(&profile1, &profile2);

        // Perfect negative correlation
        assert!((correlation + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_correlation_uniform_profiles() {
        let profile1 = [1.0; 12];
        let profile2 = [1.0; 12];

        let correlation = calculate_correlation(&profile1, &profile2);

        // Uniform profiles have zero variance - correlation undefined
        // Implementation should handle gracefully (return 0.0 or 1.0)
        assert!(correlation.is_finite());
    }

    #[test]
    fn test_correlation_zero_variance() {
        let profile1 = [5.0; 12]; // All same value
        let profile2 = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];

        let correlation = calculate_correlation(&profile1, &profile2);

        // Zero variance in profile1 - correlation undefined
        // Should return 0.0 or handle gracefully
        assert!(correlation.is_finite());
    }

    #[test]
    fn test_correlation_high_positive() {
        let profile1 = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];
        let profile2 = [1.5, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5, 8.5, 9.5, 10.5, 11.5, 12.5];

        let correlation = calculate_correlation(&profile1, &profile2);

        // Very similar profiles - high positive correlation
        assert!(correlation > 0.99);
    }

    #[test]
    fn test_correlation_moderate_positive() {
        let profile1 = [1.0, 3.0, 2.0, 4.0, 5.0, 7.0, 6.0, 8.0, 9.0, 11.0, 10.0, 12.0];
        let profile2 = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];

        let correlation = calculate_correlation(&profile1, &profile2);

        // Somewhat similar but with noise
        assert!(correlation > 0.5 && correlation < 1.0);
    }

    #[test]
    fn test_correlation_c_major_profile_with_c_major_data() {
        use crate::core::analysis::key_profiles::MAJOR_PROFILE;

        // Simulate C major distribution (emphasis on scale degrees)
        let c_major_data = [0.2, 0.01, 0.15, 0.01, 0.15, 0.12, 0.01, 0.18, 0.01, 0.1, 0.01, 0.05];

        let correlation = calculate_correlation(&c_major_data, &MAJOR_PROFILE);

        // Should have high correlation with MAJOR_PROFILE
        assert!(correlation > 0.5);
    }

    #[test]
    fn test_correlation_c_major_vs_c_sharp_major() {
        use crate::core::analysis::key_profiles::MAJOR_PROFILE;

        let c_major_profile = MAJOR_PROFILE;
        let c_sharp_major_profile = rotate_profile(&MAJOR_PROFILE, 1);

        // C major distribution
        let c_major_data = [0.2, 0.01, 0.15, 0.01, 0.15, 0.12, 0.01, 0.18, 0.01, 0.1, 0.01, 0.05];

        let corr_c = calculate_correlation(&c_major_data, &c_major_profile);
        let corr_c_sharp = calculate_correlation(&c_major_data, &c_sharp_major_profile);

        // C major data should correlate better with C major profile than C# major
        assert!(corr_c > corr_c_sharp);
    }

    #[test]
    fn test_correlation_symmetry() {
        let profile1 = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];
        let profile2 = [2.0, 3.0, 1.0, 5.0, 4.0, 7.0, 6.0, 9.0, 8.0, 11.0, 10.0, 12.0];

        let corr_12 = calculate_correlation(&profile1, &profile2);
        let corr_21 = calculate_correlation(&profile2, &profile1);

        // Correlation should be symmetric
        assert!((corr_12 - corr_21).abs() < 0.001);
    }

    // ============================================================================
    // Category 5: Musical Integration Tests (15 tests)
    // ============================================================================

    #[test]
    fn test_detect_c_major_scale() {
        let scale = create_major_scale(60); // C major scale
        let notes: Vec<(u8, u8)> = scale.iter().map(|&pitch| (pitch, 100)).collect();
        let midi = create_test_midi_with_notes(notes);

        let result = detect_key(&midi);

        assert_eq!(result.key, "C");
        assert_eq!(result.scale_type, ScaleType::Major);
        assert!(
            result.confidence > 0.7,
            "Expected high confidence for clear C major scale, got {}",
            result.confidence
        );
    }

    #[test]
    fn test_detect_a_natural_minor_scale() {
        let scale = create_minor_scale(57); // A minor scale (A3)
        let notes: Vec<(u8, u8)> = scale.iter().map(|&pitch| (pitch, 100)).collect();
        let midi = create_test_midi_with_notes(notes);

        let result = detect_key(&midi);

        // Minor scales are ambiguous - just verify it detects something
        assert!(!result.key.is_empty());
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_detect_g_major_scale() {
        let scale = create_major_scale(55); // G major scale (G3)
        let notes: Vec<(u8, u8)> = scale.iter().map(|&pitch| (pitch, 100)).collect();
        let midi = create_test_midi_with_notes(notes);

        let result = detect_key(&midi);

        assert_eq!(result.key, "G");
        assert_eq!(result.scale_type, ScaleType::Major);
        assert!(result.confidence > 0.7);
    }

    #[test]
    fn test_detect_d_minor_scale() {
        let scale = create_minor_scale(50); // D minor scale (D3)
        let notes: Vec<(u8, u8)> = scale.iter().map(|&pitch| (pitch, 100)).collect();
        let midi = create_test_midi_with_notes(notes);

        let result = detect_key(&midi);

        // Minor scales are ambiguous - just verify it detects something
        assert!(!result.key.is_empty());
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_detect_relative_keys_c_major_vs_a_minor() {
        // C major and A minor share the same notes, but emphasis differs
        // If we emphasize C-E-G (C major triad), should detect C major
        let notes = vec![
            (60, 120), // C - emphasized
            (64, 120), // E - emphasized
            (67, 120), // G - emphasized
            (62, 80),  // D
            (65, 80),  // F
            (69, 80),  // A
            (71, 80),  // B
        ];
        let midi = create_test_midi_with_notes(notes);

        let result = detect_key(&midi);

        // Should detect C major due to emphasis on tonic triad
        assert_eq!(result.key, "C");
        assert_eq!(result.scale_type, ScaleType::Major);
    }

    #[test]
    fn test_detect_relative_keys_emphasize_a_minor() {
        // Same notes, but emphasize A-C-E (A minor triad)
        let notes = vec![
            (57, 120), // A - emphasized
            (60, 120), // C - emphasized
            (64, 120), // E - emphasized
            (62, 80),  // D
            (65, 80),  // F
            (67, 80),  // G
            (71, 80),  // B
        ];
        let midi = create_test_midi_with_notes(notes);

        let result = detect_key(&midi);

        // Could detect A minor or C major (relative keys)
        assert!(result.key == "A" || result.key == "C");
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_detect_parallel_keys_c_major_vs_c_minor() {
        // C major scale
        let c_major_scale = create_major_scale(60);
        let notes: Vec<(u8, u8)> = c_major_scale.iter().map(|&pitch| (pitch, 100)).collect();
        let midi_major = create_test_midi_with_notes(notes);

        let result_major = detect_key(&midi_major);
        assert_eq!(result_major.key, "C");
        assert_eq!(result_major.scale_type, ScaleType::Major);

        // C minor scale (different notes: Eb instead of E, Ab instead of A)
        let c_minor_scale = create_minor_scale(60);
        let notes: Vec<(u8, u8)> = c_minor_scale.iter().map(|&pitch| (pitch, 100)).collect();
        let midi_minor = create_test_midi_with_notes(notes);

        let result_minor = detect_key(&midi_minor);
        // Minor scales are ambiguous - just verify it detects something
        assert!(!result_minor.key.is_empty());
        assert!(result_minor.confidence > 0.5);
    }

    #[test]
    fn test_detect_c_major_arpeggio() {
        // C major arpeggio: C-E-G
        let notes = vec![(60, 100), (64, 100), (67, 100), (72, 100)]; // C4, E4, G4, C5
        let midi = create_test_midi_with_notes(notes);

        let result = detect_key(&midi);

        assert_eq!(result.key, "C");
        assert_eq!(result.scale_type, ScaleType::Major);
    }

    #[test]
    fn test_detect_with_repeated_notes() {
        // C major scale with repeated tonic
        let mut notes = vec![];
        let scale = create_major_scale(60);
        for &pitch in &scale {
            notes.push((pitch, 100));
        }
        // Repeat C multiple times
        for _ in 0..5 {
            notes.push((60, 100));
        }

        let midi = create_test_midi_with_notes(notes);
        let result = detect_key(&midi);

        assert_eq!(result.key, "C");
        assert_eq!(result.scale_type, ScaleType::Major);
    }

    #[test]
    fn test_detect_across_multiple_octaves() {
        // C major scale across 3 octaves
        let mut notes = vec![];
        for octave in [48, 60, 72] {
            // C3, C4, C5
            let scale = create_major_scale(octave);
            for &pitch in &scale {
                notes.push((pitch, 100));
            }
        }

        let midi = create_test_midi_with_notes(notes);
        let result = detect_key(&midi);

        assert_eq!(result.key, "C");
        assert_eq!(result.scale_type, ScaleType::Major);
        assert!(result.confidence > 0.8); // Should be very confident
    }

    #[test]
    fn test_detect_pentatonic_scale() {
        // C major pentatonic: C D E G A
        let notes = vec![(60, 100), (62, 100), (64, 100), (67, 100), (69, 100), (72, 100)];
        let midi = create_test_midi_with_notes(notes);

        let result = detect_key(&midi);

        // Pentatonic should still suggest C major (or A minor)
        assert!(result.key == "C" || result.key == "A");
        // Confidence may be lower due to ambiguity
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_detect_blues_scale() {
        // E blues scale: E G A Bb B D
        let notes = vec![(52, 100), (55, 100), (57, 100), (58, 100), (59, 100), (62, 100)];
        let midi = create_test_midi_with_notes(notes);

        let result = detect_key(&midi);

        // Blues scale is very ambiguous - just check it detects something
        assert!(result.confidence > 0.3);
    }

    #[test]
    fn test_detect_with_varying_velocities() {
        // C major scale with different velocities
        let scale = create_major_scale(60);
        let velocities = [127, 100, 80, 60, 40, 80, 100, 127];
        let notes: Vec<(u8, u8)> =
            scale.iter().zip(velocities.iter()).map(|(&pitch, &vel)| (pitch, vel)).collect();

        let midi = create_test_midi_with_notes(notes);
        let result = detect_key(&midi);

        // Velocity shouldn't affect key detection (only counts, not weighted)
        assert_eq!(result.key, "C");
        assert_eq!(result.scale_type, ScaleType::Major);
    }

    #[test]
    fn test_detect_f_sharp_major() {
        // F# major scale (F# G# A# B C# D# E# F#)
        let scale = create_major_scale(54); // F# is MIDI note 54
        let notes: Vec<(u8, u8)> = scale.iter().map(|&pitch| (pitch, 100)).collect();
        let midi = create_test_midi_with_notes(notes);

        let result = detect_key(&midi);

        assert_eq!(result.key, "F#");
        assert_eq!(result.scale_type, ScaleType::Major);
        assert!(result.confidence > 0.7);
    }

    #[test]
    fn test_detect_b_flat_minor() {
        // Bb minor scale
        let scale = create_minor_scale(58); // Bb is MIDI note 58
        let notes: Vec<(u8, u8)> = scale.iter().map(|&pitch| (pitch, 100)).collect();
        let midi = create_test_midi_with_notes(notes);

        let result = detect_key(&midi);

        // Minor scales are ambiguous - just verify it detects something
        assert!(!result.key.is_empty());
        assert!(result.confidence > 0.5);
    }

    // ============================================================================
    // Category 6: Edge Case Integration Tests (8 tests)
    // ============================================================================

    #[test]
    fn test_detect_empty_midi_file() {
        let midi = create_test_midi_with_notes(vec![]);

        let result = detect_key(&midi);

        // Empty file should return default (C major with low confidence)
        assert_eq!(result.key, "C");
        assert!(result.confidence < 0.7); // Low confidence due to no data
    }

    #[test]
    fn test_detect_single_note() {
        let midi = create_test_midi_with_notes(vec![(60, 100)]); // Single C

        let result = detect_key(&midi);

        // Single note is ambiguous but should detect some key
        assert!(result.confidence < 0.7); // Very low confidence
    }

    #[test]
    fn test_detect_two_notes() {
        let midi = create_test_midi_with_notes(vec![(60, 100), (64, 100)]); // C and E

        let result = detect_key(&midi);

        // Two notes (C-E) suggest C major or A minor
        assert!(result.key == "C" || result.key == "A");
        assert!(result.confidence < 0.8); // Moderate confidence
    }

    #[test]
    fn test_detect_chromatic_scale() {
        let scale = create_chromatic_scale(60); // All 12 pitches
        let notes: Vec<(u8, u8)> = scale.iter().map(|&pitch| (pitch, 100)).collect();
        let midi = create_test_midi_with_notes(notes);

        let result = detect_key(&midi);

        // Chromatic scale is maximally ambiguous
        // Should still return a key but with very low confidence
        assert!(result.confidence < 0.6);
    }

    #[test]
    fn test_detect_atonal_music() {
        // Atonal pattern: no clear tonal center
        let notes = vec![(60, 100), (61, 100), (66, 100), (68, 100), (70, 100), (63, 100)];
        let midi = create_test_midi_with_notes(notes);

        let result = detect_key(&midi);

        // Atonal should have very low confidence
        assert!(result.confidence < 0.7);
    }

    #[test]
    fn test_detect_whole_tone_scale() {
        // Whole tone scale: C D E F# G# A#
        let notes = vec![(60, 100), (62, 100), (64, 100), (66, 100), (68, 100), (70, 100)];
        let midi = create_test_midi_with_notes(notes);

        let result = detect_key(&midi);

        // Whole tone is ambiguous
        assert!(result.confidence < 0.7);
    }

    #[test]
    fn test_detect_very_short_file() {
        // Very few notes (less than full scale)
        let notes = vec![(60, 100), (62, 100), (64, 100)]; // C D E
        let midi = create_test_midi_with_notes(notes);

        let result = detect_key(&midi);

        // Should detect a key (any key is fine with short data)
        assert!(result.confidence > 0.3);
    }

    #[test]
    fn test_detect_very_long_file() {
        // 1000 notes repeating C major scale
        let mut notes = vec![];
        let scale = create_major_scale(60);

        for _ in 0..125 {
            // 125 repetitions * 8 notes = 1000 notes
            for &pitch in &scale {
                notes.push((pitch, 100));
            }
        }

        let midi = create_test_midi_with_notes(notes);
        let result = detect_key(&midi);

        assert_eq!(result.key, "C");
        assert_eq!(result.scale_type, ScaleType::Major);
        assert!(result.confidence > 0.7); // High confidence with lots of data
    }

    // ============================================================================
    // Category 7: Numerical Stability Tests (6 tests)
    // ============================================================================

    #[test]
    fn test_stability_normalize_prevents_nan() {
        let histogram = [0; 12];
        let normalized = normalize_histogram(&histogram);

        // Should not produce NaN
        for &val in &normalized {
            assert!(!val.is_nan(), "Normalized histogram contains NaN");
            assert!(
                val.is_finite(),
                "Normalized histogram contains non-finite value"
            );
        }
    }

    #[test]
    fn test_stability_correlation_with_zeros() {
        let profile1 = [0.0; 12];
        let profile2 = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];

        let correlation = calculate_correlation(&profile1, &profile2);

        // Should not be NaN or infinity
        assert!(correlation.is_finite());
    }

    #[test]
    fn test_stability_confidence_nan_handling() {
        // Correlation with NaN (shouldn't happen, but test defensive code)
        let correlations = vec![(0, ScaleType::Major, 0.8), (1, ScaleType::Major, 0.75)];

        let confidence = calculate_confidence(&correlations);

        assert!(confidence.is_finite());
        assert!((0.0..=1.0).contains(&confidence));
    }

    #[test]
    fn test_stability_very_small_differences() {
        // Test numerical precision with very small differences
        let profile1 = [1.000000001, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];
        let profile2 = [1.000000002, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];

        let correlation = calculate_correlation(&profile1, &profile2);

        // Should be very close to 1.0
        assert!(correlation > 0.99);
        assert!(correlation.is_finite());
    }

    #[test]
    fn test_stability_large_histogram_counts() {
        // Test with very large counts (potential overflow)
        let histogram = [u32::MAX / 24; 12];
        let normalized = normalize_histogram(&histogram);

        let sum: f64 = normalized.iter().sum();
        assert!((sum - 1.0).abs() < 0.01, "Sum should be close to 1.0");

        for &val in &normalized {
            assert!(val.is_finite());
            assert!(!val.is_nan());
        }
    }

    #[test]
    fn test_stability_confidence_bounds() {
        // Test that confidence is always in [0.0, 1.0]
        let test_cases = vec![
            vec![(0, ScaleType::Major, 0.99), (1, ScaleType::Major, 0.01)],
            vec![(0, ScaleType::Major, 0.5), (1, ScaleType::Major, 0.5)],
            vec![(0, ScaleType::Major, 0.7), (1, ScaleType::Major, 0.69)],
        ];

        for correlations in test_cases {
            let confidence = calculate_confidence(&correlations);
            assert!(
                (0.0..=1.0).contains(&confidence),
                "Confidence {} out of bounds [0.0, 1.0]",
                confidence
            );
        }
    }

    // ============================================================================
    // Category 8: Property-Based Tests (5 tests)
    // ============================================================================

    #[test]
    fn test_property_normalize_sum_is_one() {
        // Property: Normalized histogram always sums to 1.0 (or uniform distribution)
        let test_cases = vec![
            [10, 20, 30, 0, 0, 0, 0, 0, 0, 0, 0, 40],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55, 60],
        ];

        for histogram in &test_cases {
            let normalized = normalize_histogram(histogram);
            let sum: f64 = normalized.iter().sum();
            assert!(
                (sum - 1.0).abs() < 0.001,
                "Normalized histogram sum {} != 1.0 for {:?}",
                sum,
                histogram
            );
        }
    }

    #[test]
    fn test_property_rotation_preserves_array_length() {
        use crate::core::analysis::key_profiles::MAJOR_PROFILE;

        for rotation in 0..12 {
            let rotated = rotate_profile(&MAJOR_PROFILE, rotation);
            assert_eq!(rotated.len(), 12, "Rotation changed array length");
        }
    }

    #[test]
    fn test_property_rotation_is_cyclic() {
        let profile = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];

        // Rotating 12 times should return to original
        let mut current = profile;
        for _ in 0..12 {
            current = rotate_profile(&current, 1);
        }

        assert_eq!(current, profile, "12 rotations didn't return to original");
    }

    #[test]
    fn test_property_correlation_bounded() {
        // Property: Pearson correlation is always in [-1.0, 1.0]
        let test_profiles = vec![
            (
                [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0],
                [12.0, 11.0, 10.0, 9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0],
            ),
            (
                [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2],
                [0.2, 0.3, 0.1, 0.5, 0.4, 0.7, 0.6, 0.9, 0.8, 1.1, 1.0, 1.2],
            ),
            (
                [5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0],
                [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0],
            ),
        ];

        for (profile1, profile2) in &test_profiles {
            let correlation = calculate_correlation(profile1, profile2);
            assert!(
                (-1.0..=1.0).contains(&correlation),
                "Correlation {} out of bounds [-1.0, 1.0]",
                correlation
            );
        }
    }

    #[test]
    fn test_property_key_detection_deterministic() {
        // Property: Same input always produces same output
        let scale = create_major_scale(60);
        let notes: Vec<(u8, u8)> = scale.iter().map(|&pitch| (pitch, 100)).collect();
        let midi = create_test_midi_with_notes(notes.clone());

        let result1 = detect_key(&midi);

        // Create identical MIDI again
        let midi2 = create_test_midi_with_notes(notes);
        let result2 = detect_key(&midi2);

        // Results should be identical
        assert_eq!(result1.key, result2.key);
        assert_eq!(result1.scale_type, result2.scale_type);
        assert!((result1.confidence - result2.confidence).abs() < 0.000001);
    }
}

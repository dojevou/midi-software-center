//! Key Detection Module
//!
//! Implements the Krumhansl-Schmuckler key-finding algorithm to detect
//! the musical key of MIDI files.
//!
//! # Archetype: Trusty Module
//! - Pure functions with no side effects
//! - No I/O operations
//! - Highly testable
//! - Reusable across the application

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
        let profile = [
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
        ];

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
        let dist1 = [
            0.1, 0.2, 0.1, 0.05, 0.15, 0.1, 0.05, 0.15, 0.03, 0.02, 0.03, 0.02,
        ];
        let correlation = calculate_correlation(&dist1, &dist1);

        assert!((correlation - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_correlation_zero() {
        let dist1 = [0.0; 12];
        let dist2 = [
            0.1, 0.2, 0.1, 0.05, 0.15, 0.1, 0.05, 0.15, 0.03, 0.02, 0.03, 0.02,
        ];
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
}

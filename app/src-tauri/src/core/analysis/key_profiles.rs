//! Key profiles for musical key detection algorithms.
//!
//! This module provides pre-defined pitch class profiles for various key detection
//! algorithms and musical genres, including the classic Krumhansl-Schmuckler
//! profiles and genre-specific adaptations for electronic music analysis.

use serde::{Deserialize, Serialize};

/// Represents a musical key profile with weights for each pitch class.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyProfile {
    /// Name of the profile (e.g., "Krumhansl Major", "Techno")
    pub name: String,

    /// Weights for each pitch class (0=C, 1=C#, ..., 11=B)
    pub weights: [f64; 12],

    /// Optional description of the profile's characteristics
    pub description: Option<String>,

    /// Profile type (major, minor, genre-specific, etc.)
    pub profile_type: ProfileType,
}

/// Type of key profile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileType {
    /// Major key profile
    Major,
    /// Minor key profile
    Minor,
    /// Blues/rock profile
    Blues,
    /// Jazz profile
    Jazz,
    /// Electronic dance music profile
    Electronic,
    /// Classical/romantic profile
    Classical,
    /// Pop music profile
    Pop,
    /// Custom profile
    Custom,
}

/// Standard Krumhansl-Kessler major key profile (1982)
/// Probe-tone ratings for major keys, indexed by pitch class relative to tonic.
pub const KRUMHANSL_MAJOR_PROFILE: [f64; 12] = [
    6.35, // Tonic (I) - C in C major
    2.23, // Minor 2nd - C# in C major
    3.48, // Major 2nd - D in C major
    2.33, // Minor 3rd - D# in C major
    4.38, // Major 3rd - E in C major
    4.09, // Perfect 4th - F in C major
    2.52, // Tritone - F# in C major
    5.19, // Perfect 5th - G in C major
    2.39, // Minor 6th - G# in C major
    3.66, // Major 6th - A in C major
    2.29, // Minor 7th - A# in C major
    2.88, // Major 7th - B in C major
];

/// Standard Krumhansl-Kessler minor key profile (1982)
pub const KRUMHANSL_MINOR_PROFILE: [f64; 12] = [
    6.33, // Tonic (i) - A in A minor
    2.68, // Minor 2nd - A# in A minor
    3.52, // Major 2nd - B in A minor
    5.38, // Minor 3rd - C in A minor (characteristic minor 3rd)
    2.60, // Major 3rd - C# in A minor
    3.53, // Perfect 4th - D in A minor
    2.54, // Tritone - D# in A minor
    4.75, // Perfect 5th - E in A minor
    3.98, // Minor 6th - F in A minor
    2.69, // Major 6th - F# in A minor
    3.34, // Minor 7th - G in A minor
    3.17, // Major 7th - G# in A minor
];

/// Temperley major profile (2001)
/// Alternative profile emphasizing leading tone resolution.
pub const TEMPERLEY_MAJOR_PROFILE: [f64; 12] = [
    5.0, // Tonic
    2.0, // Minor 2nd
    3.5, // Major 2nd
    2.0, // Minor 3rd
    4.5, // Major 3rd
    4.0, // Perfect 4th
    2.0, // Tritone
    4.5, // Perfect 5th
    2.0, // Minor 6th
    3.5, // Major 6th
    1.5, // Minor 7th
    4.0, // Major 7th
];

/// Temperley minor profile (2001)
pub const TEMPERLEY_MINOR_PROFILE: [f64; 12] = [
    5.0, // Tonic
    2.0, // Minor 2nd
    3.5, // Major 2nd
    4.5, // Minor 3rd
    2.0, // Major 3rd
    4.0, // Perfect 4th
    2.0, // Tritone
    4.5, // Perfect 5th
    3.5, // Minor 6th
    2.0, // Major 6th
    1.5, // Minor 7th
    4.0, // Major 7th
];

/// Simple binary major profile
/// For rapid classification when accuracy is less critical.
pub const SIMPLE_MAJOR_PROFILE: [f64; 12] = [
    1.0, // C - Tonic
    0.0, // C#
    1.0, // D - Major 2nd
    0.0, // D#
    1.0, // E - Major 3rd
    1.0, // F - Perfect 4th
    0.0, // F# - Tritone
    1.0, // G - Perfect 5th
    0.0, // G#
    1.0, // A - Major 6th
    0.0, // A#
    1.0, // B - Major 7th
];

/// Simple binary minor profile
pub const SIMPLE_MINOR_PROFILE: [f64; 12] = [
    1.0, // C - Tonic
    0.0, // C#
    1.0, // D - Major 2nd
    1.0, // D# - Minor 3rd
    0.0, // E
    1.0, // F - Perfect 4th
    0.0, // F#
    1.0, // G - Perfect 5th
    1.0, // G# - Minor 6th
    0.0, // A
    1.0, // A# - Minor 7th
    0.0, // B
];

// ============================================================================
// GENRE-SPECIFIC PROFILES
// ============================================================================

/// Blues/Rock profile
/// Emphasizes blue notes (b3, b5, b7) characteristic of blues and rock music.
pub const BLUES_PROFILE: [f64; 12] = [
    6.0, // Tonic
    2.0, // Minor 2nd
    3.0, // Major 2nd
    5.0, // Minor 3rd (blue note)
    3.5, // Major 3rd
    4.0, // Perfect 4th
    4.5, // Tritone (blue note)
    5.0, // Perfect 5th
    2.0, // Minor 6th
    3.5, // Major 6th
    5.0, // Minor 7th (blue note)
    2.5, // Major 7th
];

/// Jazz major profile
/// More chromatic, emphasizes extended harmonies (9ths, 11ths, 13ths).
pub const JAZZ_MAJOR_PROFILE: [f64; 12] = [
    5.5, // Tonic
    3.0, // Minor 2nd (b9)
    4.0, // Major 2nd (9)
    2.5, // Minor 3rd
    4.5, // Major 3rd
    4.0, // Perfect 4th (11)
    3.5, // Tritone (#11/b5)
    5.0, // Perfect 5th
    3.0, // Minor 6th (b13)
    4.0, // Major 6th (13)
    3.0, // Minor 7th (b7)
    4.0, // Major 7th
];

/// Jazz minor profile (melodic minor/harmonic minor)
pub const JAZZ_MINOR_PROFILE: [f64; 12] = [
    5.5, // Tonic
    3.0, // Minor 2nd
    4.0, // Major 2nd
    5.0, // Minor 3rd
    3.0, // Major 3rd
    4.0, // Perfect 4th
    4.0, // Tritone (#11 in melodic minor)
    5.0, // Perfect 5th
    4.0, // Minor 6th
    3.0, // Major 6th
    4.5, // Minor 7th (dominant 7th in harmonic minor)
    3.5, // Major 7th
];

/// Electronic/EDM profile (Techno, House, Trance)
/// Simpler harmonic content, emphasizes root, fifth, and minor/major third.
pub const EDM_PROFILE: [f64; 12] = [
    6.5, // Tonic (strong emphasis on root)
    1.5, // Minor 2nd (rare in EDM)
    3.0, // Major 2nd (used in melodies)
    2.0, // Minor 3rd (common in minor keys)
    4.0, // Major 3rd (common in major keys)
    3.5, // Perfect 4th (common)
    2.0, // Tritone (rare, dissonant)
    5.5, // Perfect 5th (very strong in EDM)
    2.0, // Minor 6th
    3.0, // Major 6th
    2.0, // Minor 7th
    3.0, // Major 7th
];

/// Drum & Bass/Jungle profile
/// Complex harmonic structures, rapid chord changes, minor-oriented.
pub const DRUM_AND_BASS_PROFILE: [f64; 12] = [
    6.0, // Tonic
    2.5, // Minor 2nd
    3.5, // Major 2nd
    5.5, // Minor 3rd (very characteristic)
    2.5, // Major 3rd
    4.0, // Perfect 4th
    3.0, // Tritone (used for tension)
    5.0, // Perfect 5th
    4.0, // Minor 6th (common in chord extensions)
    2.5, // Major 6th
    4.5, // Minor 7th (dominant 7th chords)
    2.5, // Major 7th
];

/// Trance/Progressive House profile
/// Epic, major-oriented with occasional minor modulations.
pub const TRANCE_PROFILE: [f64; 12] = [
    6.0, // Tonic
    2.0, // Minor 2nd
    3.5, // Major 2nd
    3.0, // Minor 3rd (for emotional modulation)
    4.5, // Major 3rd (characteristic)
    4.0, // Perfect 4th
    2.5, // Tritone
    5.5, // Perfect 5th (strong arpeggios)
    2.5, // Minor 6th
    3.5, // Major 6th
    3.0, // Minor 7th
    3.5, // Major 7th (for epic feel)
];

/// Hip-Hop/Trap profile
/// Simple, repetitive harmonic patterns, minor-oriented.
pub const HIPHOP_PROFILE: [f64; 12] = [
    6.0, // Tonic
    2.0, // Minor 2nd
    3.0, // Major 2nd
    5.0, // Minor 3rd (characteristic)
    2.5, // Major 3rd
    4.0, // Perfect 4th
    3.0, // Tritone (used in trap)
    5.0, // Perfect 5th
    3.5, // Minor 6th
    2.5, // Major 6th
    4.0, // Minor 7th
    2.0, // Major 7th
];

/// Ambient/Chillout profile
/// Extended chords, suspended tones, ambiguous tonality.
pub const AMBIENT_PROFILE: [f64; 12] = [
    5.5, // Tonic
    3.0, // Minor 2nd (cluster tones)
    3.5, // Major 2nd
    4.0, // Minor 3rd
    4.0, // Major 3rd
    4.5, // Perfect 4th (suspended chords)
    3.5, // Tritone (for ambiguous feel)
    5.0, // Perfect 5th
    3.5, // Minor 6th
    3.5, // Major 6th
    4.0, // Minor 7th
    3.5, // Major 7th
];

/// Classical/Romantic profile
/// Based on analysis of classical repertoire.
pub const CLASSICAL_PROFILE: [f64; 12] = [
    6.2, // Tonic
    2.3, // Minor 2nd
    3.6, // Major 2nd
    2.4, // Minor 3rd
    4.4, // Major 3rd
    4.1, // Perfect 4th
    2.6, // Tritone
    5.2, // Perfect 5th
    2.4, // Minor 6th
    3.7, // Major 6th
    2.3, // Minor 7th
    2.9, // Major 7th
];

/// Pop music profile (2000s-present)
/// Simple, catchy melodies, major bias.
pub const POP_PROFILE: [f64; 12] = [
    6.0, // Tonic
    2.0, // Minor 2nd
    3.5, // Major 2nd
    2.5, // Minor 3rd
    4.2, // Major 3rd
    4.0, // Perfect 4th
    2.5, // Tritone
    5.0, // Perfect 5th
    2.5, // Minor 6th
    3.5, // Major 6th
    3.0, // Minor 7th
    3.0, // Major 7th
];

// ============================================================================
// PROFILE MANAGEMENT FUNCTIONS
// ============================================================================

/// Rotate a profile array by the specified number of semitones.
///
/// This is used to test different key centers by rotating the profile
/// so that each pitch class can become the tonic.
///
/// # Arguments
/// * `profile` - The base profile array (length 12)
/// * `rotation` - Number of semitones to rotate (0-11)
///
/// # Returns
/// Rotated profile array
pub fn rotate_profile(profile: &[f64; 12], rotation: usize) -> [f64; 12] {
    let mut rotated = [0.0; 12];
    for i in 0..12 {
        rotated[i] = profile[(i + rotation) % 12];
    }
    rotated
}

/// Get the appropriate profile for a given genre and scale type.
///
/// Returns a tuple of (major_profile, minor_profile) for the genre.
/// If the genre is not recognized, returns the standard Krumhansl profiles.
pub fn get_profiles_for_genre(genre: &str) -> (&'static [f64; 12], &'static [f64; 12]) {
    match genre.to_lowercase().as_str() {
        "techno" | "house" | "edm" | "electronic" | "dance" => {
            (&EDM_PROFILE, &EDM_PROFILE) // Same profile for both in EDM
        },
        "drum and bass" | "dnb" | "jungle" => (&DRUM_AND_BASS_PROFILE, &DRUM_AND_BASS_PROFILE),
        "trance" | "progressive" | "progressive house" => (&TRANCE_PROFILE, &TRANCE_PROFILE),
        "hiphop" | "hip-hop" | "trap" | "rap" => (&HIPHOP_PROFILE, &HIPHOP_PROFILE),
        "ambient" | "chillout" | "downtempo" => (&AMBIENT_PROFILE, &AMBIENT_PROFILE),
        "blues" | "rock" | "rock and roll" => (&BLUES_PROFILE, &BLUES_PROFILE),
        "jazz" | "fusion" | "bebop" => (&JAZZ_MAJOR_PROFILE, &JAZZ_MINOR_PROFILE),
        "classical" | "romantic" | "baroque" => (&CLASSICAL_PROFILE, &CLASSICAL_PROFILE),
        "pop" | "popular" | "top 40" => (&POP_PROFILE, &POP_PROFILE),
        _ => {
            // Default to Krumhansl profiles for unknown genres
            (&KRUMHANSL_MAJOR_PROFILE, &KRUMHANSL_MINOR_PROFILE)
        },
    }
}

/// Get all available profile types as a vector of (name, major_profile, minor_profile).
pub fn get_all_profiles() -> Vec<(&'static str, &'static [f64; 12], &'static [f64; 12])> {
    vec![
        (
            "Krumhansl",
            &KRUMHANSL_MAJOR_PROFILE,
            &KRUMHANSL_MINOR_PROFILE,
        ),
        (
            "Temperley",
            &TEMPERLEY_MAJOR_PROFILE,
            &TEMPERLEY_MINOR_PROFILE,
        ),
        ("Simple", &SIMPLE_MAJOR_PROFILE, &SIMPLE_MINOR_PROFILE),
        ("EDM", &EDM_PROFILE, &EDM_PROFILE),
        (
            "Drum & Bass",
            &DRUM_AND_BASS_PROFILE,
            &DRUM_AND_BASS_PROFILE,
        ),
        ("Trance", &TRANCE_PROFILE, &TRANCE_PROFILE),
        ("Hip-Hop", &HIPHOP_PROFILE, &HIPHOP_PROFILE),
        ("Ambient", &AMBIENT_PROFILE, &AMBIENT_PROFILE),
        ("Blues", &BLUES_PROFILE, &BLUES_PROFILE),
        ("Jazz", &JAZZ_MAJOR_PROFILE, &JAZZ_MINOR_PROFILE),
        ("Classical", &CLASSICAL_PROFILE, &CLASSICAL_PROFILE),
        ("Pop", &POP_PROFILE, &POP_PROFILE),
    ]
}

/// Normalize a profile so that its values sum to 1.0.
///
/// This is useful for converting profile weights to probability distributions.
pub fn normalize_profile(profile: &[f64; 12]) -> [f64; 12] {
    let sum: f64 = profile.iter().sum();
    if sum.abs() < f64::EPSILON {
        // If sum is zero or near-zero, return uniform distribution
        return [1.0 / 12.0; 12];
    }

    let mut normalized = [0.0; 12];
    for i in 0..12 {
        normalized[i] = profile[i] / sum;
    }
    normalized
}

/// Create a weighted combination of multiple profiles.
///
/// Useful for creating custom profiles that blend characteristics
/// of multiple base profiles.
///
/// # Arguments
/// * `profiles` - Slice of profile arrays
/// * `weights` - Corresponding weights for each profile
///
/// # Returns
/// Combined profile array
pub fn combine_profiles(profiles: &[&[f64; 12]], weights: &[f64]) -> [f64; 12] {
    assert_eq!(
        profiles.len(),
        weights.len(),
        "Profiles and weights must have same length"
    );

    let mut combined = [0.0; 12];
    let weight_sum: f64 = weights.iter().sum();

    for i in 0..12 {
        for (profile_idx, &profile) in profiles.iter().enumerate() {
            combined[i] += profile[i] * weights[profile_idx];
        }
        if weight_sum.abs() > f64::EPSILON {
            combined[i] /= weight_sum;
        }
    }

    combined
}

/// Calculate the "profile strength" - how strongly a profile defines a key.
///
/// Returns a value between 0.0 and 1.0 where higher values indicate
/// profiles that give more unambiguous key predictions.
pub fn calculate_profile_strength(profile: &[f64; 12]) -> f64 {
    let normalized = normalize_profile(profile);

    // Calculate the difference between the strongest and weakest weights
    let max_weight = normalized.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let min_weight = normalized.iter().fold(f64::INFINITY, |a, &b| a.min(b));

    (max_weight - min_weight).clamp(0.0, 1.0)
}

/// Get the scale degree weights for a profile at a specific key.
///
/// Returns an array where index 0 is the tonic weight, index 1 is the minor 2nd, etc.
/// Rotated appropriately for the specified key.
pub fn get_scale_degree_weights(profile: &[f64; 12], tonic_pitch_class: usize) -> [f64; 12] {
    rotate_profile(profile, tonic_pitch_class)
}

/// Detect the most likely profile type for a given pitch distribution.
///
/// Compares the distribution against all known profiles and returns
/// the best match along with a confidence score.
pub fn detect_profile_type(pitch_distribution: &[f64; 12]) -> (ProfileType, f64) {
    // Define profile groups to test
    let profile_groups = [
        (ProfileType::Major, &KRUMHANSL_MAJOR_PROFILE),
        (ProfileType::Minor, &KRUMHANSL_MINOR_PROFILE),
        (ProfileType::Blues, &BLUES_PROFILE),
        (ProfileType::Jazz, &JAZZ_MAJOR_PROFILE),
        (ProfileType::Electronic, &EDM_PROFILE),
        (ProfileType::Classical, &CLASSICAL_PROFILE),
        (ProfileType::Pop, &POP_PROFILE),
    ];

    let mut best_match = ProfileType::Major;
    let mut best_correlation = f64::NEG_INFINITY;

    for (profile_type, profile) in profile_groups {
        let correlation = calculate_profile_correlation(pitch_distribution, profile);
        if correlation > best_correlation {
            best_correlation = correlation;
            best_match = profile_type;
        }
    }

    (best_match, best_correlation)
}

/// Calculate Pearson correlation between a pitch distribution and a profile.
///
/// Helper function used by `detect_profile_type`.
pub fn calculate_profile_correlation(distribution: &[f64; 12], profile: &[f64; 12]) -> f64 {
    let dist_mean: f64 = distribution.iter().sum::<f64>() / 12.0;
    let prof_mean: f64 = profile.iter().sum::<f64>() / 12.0;

    let mut numerator = 0.0;
    let mut dist_sq_sum = 0.0;
    let mut prof_sq_sum = 0.0;

    for i in 0..12 {
        let dist_diff = distribution[i] - dist_mean;
        let prof_diff = profile[i] - prof_mean;

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

/// Create a KeyProfile struct from raw weights.
pub fn create_key_profile(name: &str, weights: [f64; 12], profile_type: ProfileType) -> KeyProfile {
    KeyProfile {
        name: name.to_string(),
        weights,
        description: match profile_type {
            ProfileType::Major => Some("Standard major key profile".to_string()),
            ProfileType::Minor => Some("Standard minor key profile".to_string()),
            ProfileType::Blues => Some("Blues/rock profile emphasizing blue notes".to_string()),
            ProfileType::Jazz => Some("Jazz profile with extended harmonies".to_string()),
            ProfileType::Electronic => {
                Some("Electronic music profile with strong root/fifth".to_string())
            },
            ProfileType::Classical => Some("Classical/romantic era profile".to_string()),
            ProfileType::Pop => Some("Contemporary pop music profile".to_string()),
            ProfileType::Custom => Some("Custom user-defined profile".to_string()),
        },
        profile_type,
    }
}

/// Get the default Krumhansl key profile.
pub fn get_default_profile() -> KeyProfile {
    create_key_profile(
        "Krumhansl Major",
        KRUMHANSL_MAJOR_PROFILE,
        ProfileType::Major,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-10;

    /// Helper function to check if two floats are approximately equal
    fn approx_eq(a: f64, b: f64, epsilon: f64) -> bool {
        (a - b).abs() < epsilon
    }

    #[test]
    fn test_rotate_profile() {
        let profile = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];

        // Rotate by 0 (should be unchanged)
        let rotated0 = rotate_profile(&profile, 0);
        assert_eq!(rotated0, profile);

        // Rotate by 1
        let rotated1 = rotate_profile(&profile, 1);
        assert!(approx_eq(rotated1[0], profile[1], EPSILON));
        assert!(approx_eq(rotated1[11], profile[0], EPSILON));

        // Rotate by 12 (full circle)
        let rotated12 = rotate_profile(&profile, 12);
        assert_eq!(rotated12, profile);

        // Rotate by 7 (perfect fifth)
        let rotated7 = rotate_profile(&profile, 7);
        assert!(approx_eq(rotated7[0], profile[7], EPSILON));
    }

    #[test]
    fn test_normalize_profile() {
        let profile = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];
        let normalized = normalize_profile(&profile);

        // Check sum is 1.0
        let sum: f64 = normalized.iter().sum();
        assert!(approx_eq(sum, 1.0, EPSILON));

        // Check proportions are maintained
        for i in 1..12 {
            let ratio_original = profile[i] / profile[0];
            let ratio_normalized = normalized[i] / normalized[0];
            assert!(approx_eq(ratio_original, ratio_normalized, EPSILON));
        }
    }

    #[test]
    fn test_normalize_zero_profile() {
        let zero_profile = [0.0; 12];
        let normalized = normalize_profile(&zero_profile);

        // Should return uniform distribution
        for &val in &normalized {
            assert!(approx_eq(val, 1.0 / 12.0, EPSILON));
        }
    }

    #[test]
    fn test_combine_profiles() {
        let profile1 = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];
        let profile2 = [12.0, 11.0, 10.0, 9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];

        let combined = combine_profiles(&[&profile1, &profile2], &[0.5, 0.5]);

        // Equal weights should average
        for i in 0..12 {
            assert!(approx_eq(
                combined[i],
                (profile1[i] + profile2[i]) / 2.0,
                EPSILON
            ));
        }

        // Different weights
        let combined_weighted = combine_profiles(&[&profile1, &profile2], &[0.8, 0.2]);
        for i in 0..12 {
            let expected = profile1[i] * 0.8 + profile2[i] * 0.2;
            assert!(approx_eq(combined_weighted[i], expected, EPSILON));
        }
    }

    #[test]
    fn test_calculate_profile_strength() {
        // Strong profile (large differences)
        let strong_profile = [10.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let strength = calculate_profile_strength(&strong_profile);
        assert!(strength > 0.4); // Significant strength

        // Weak profile (no differences)
        let weak_profile = [1.0; 12];
        let strength = calculate_profile_strength(&weak_profile);
        assert!(approx_eq(strength, 0.0, EPSILON));
    }

    #[test]
    fn test_get_profiles_for_genre() {
        // Test known genres
        let (major, minor) = get_profiles_for_genre("techno");
        assert_eq!(major, &EDM_PROFILE);
        assert_eq!(minor, &EDM_PROFILE);

        let (major, minor) = get_profiles_for_genre("jazz");
        assert_eq!(major, &JAZZ_MAJOR_PROFILE);
        assert_eq!(minor, &JAZZ_MINOR_PROFILE);

        // Test case insensitivity
        let (major, _) = get_profiles_for_genre("TECHNO");
        assert_eq!(major, &EDM_PROFILE);

        // Test unknown genre (should return default)
        let (major, minor) = get_profiles_for_genre("unknown_genre");
        assert_eq!(major, &KRUMHANSL_MAJOR_PROFILE);
        assert_eq!(minor, &KRUMHANSL_MINOR_PROFILE);
    }

    #[test]
    fn test_profile_correlation() {
        // Create a perfect match (correlation with itself)
        let distribution = normalize_profile(&KRUMHANSL_MAJOR_PROFILE);
        let correlation = calculate_profile_correlation(&distribution, &KRUMHANSL_MAJOR_PROFILE);

        // Correlation with itself should be close to 1.0
        assert!(correlation > 0.99);

        // Test with random distribution
        let random_dist = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2];
        let normalized_random = normalize_profile(&random_dist);
        let correlation =
            calculate_profile_correlation(&normalized_random, &KRUMHANSL_MAJOR_PROFILE);

        // Should be between -1 and 1
        assert!((-1.0..=1.0).contains(&correlation));
    }

    #[test]
    fn test_detect_profile_type() {
        // Test with major profile distribution
        let major_dist = normalize_profile(&KRUMHANSL_MAJOR_PROFILE);
        let (profile_type, confidence) = detect_profile_type(&major_dist);
        assert_eq!(profile_type, ProfileType::Major);
        assert!(confidence > 0.9);

        // Test with minor profile distribution
        let minor_dist = normalize_profile(&KRUMHANSL_MINOR_PROFILE);
        let (profile_type, confidence) = detect_profile_type(&minor_dist);
        assert_eq!(profile_type, ProfileType::Minor);
        assert!(confidence > 0.9);
    }

    #[test]
    fn test_create_key_profile() {
        let profile = create_key_profile("Test Major", KRUMHANSL_MAJOR_PROFILE, ProfileType::Major);

        assert_eq!(profile.name, "Test Major");
        assert_eq!(profile.weights, KRUMHANSL_MAJOR_PROFILE);
        assert_eq!(profile.profile_type, ProfileType::Major);
        assert!(profile.description.is_some());
    }

    #[test]
    fn test_get_default_profile() {
        let profile = get_default_profile();
        assert_eq!(profile.name, "Krumhansl Major");
        assert_eq!(profile.profile_type, ProfileType::Major);
    }

    #[test]
    fn test_get_all_profiles() {
        let profiles = get_all_profiles();
        assert_eq!(profiles.len(), 12);

        // Check first profile is Krumhansl
        assert_eq!(profiles[0].0, "Krumhansl");
        assert_eq!(profiles[0].1, &KRUMHANSL_MAJOR_PROFILE);
        assert_eq!(profiles[0].2, &KRUMHANSL_MINOR_PROFILE);
    }

    #[test]
    fn test_profile_constants_valid() {
        // Ensure all profile constants have 12 elements and reasonable values
        let profiles: Vec<&[f64; 12]> = vec![
            &KRUMHANSL_MAJOR_PROFILE,
            &KRUMHANSL_MINOR_PROFILE,
            &TEMPERLEY_MAJOR_PROFILE,
            &TEMPERLEY_MINOR_PROFILE,
            &SIMPLE_MAJOR_PROFILE,
            &SIMPLE_MINOR_PROFILE,
            &BLUES_PROFILE,
            &JAZZ_MAJOR_PROFILE,
            &JAZZ_MINOR_PROFILE,
            &EDM_PROFILE,
            &DRUM_AND_BASS_PROFILE,
            &TRANCE_PROFILE,
            &HIPHOP_PROFILE,
            &AMBIENT_PROFILE,
            &CLASSICAL_PROFILE,
            &POP_PROFILE,
        ];

        for profile in profiles {
            assert_eq!(profile.len(), 12);
            for &val in profile {
                assert!(val >= 0.0, "Profile values should be non-negative");
                assert!(val <= 10.0, "Profile values should be reasonable (≤10)");
            }
        }
    }

    #[test]
    fn test_get_scale_degree_weights() {
        // For C major (rotation 0), scale degree weights should match original
        let weights = get_scale_degree_weights(&KRUMHANSL_MAJOR_PROFILE, 0);
        assert_eq!(weights, KRUMHANSL_MAJOR_PROFILE);

        // For G major (rotation 7), tonic weight should be at index 0
        let g_major_weights = get_scale_degree_weights(&KRUMHANSL_MAJOR_PROFILE, 7);
        assert!(approx_eq(
            g_major_weights[0],
            KRUMHANSL_MAJOR_PROFILE[7],
            EPSILON
        ));
    }
}

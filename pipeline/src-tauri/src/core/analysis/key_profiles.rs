/// Krumhansl-Schmuckler Key Profiles
///
/// These profiles represent the expected distribution of pitch classes
/// in major and minor keys, derived from music theory research.
/// Major key profile (Krumhansl & Kessler, 1982)
/// Indexed by pitch class: C, C#, D, D#, E, F, F#, G, G#, A, A#, B
pub const MAJOR_PROFILE: [f64; 12] = [
    6.35, // C  - Tonic (strongest)
    2.23, // C# - Minor 2nd
    3.48, // D  - Major 2nd
    2.33, // D# - Minor 3rd
    4.38, // E  - Major 3rd
    4.09, // F  - Perfect 4th
    2.52, // F# - Tritone
    5.19, // G  - Perfect 5th
    2.39, // G# - Minor 6th
    3.66, // A  - Major 6th
    2.29, // A# - Minor 7th
    2.88, // B  - Major 7th
];

/// Minor key profile (Krumhansl & Kessler, 1982)
pub const MINOR_PROFILE: [f64; 12] = [
    6.33, // C  - Tonic (strongest)
    2.68, // C# - Minor 2nd
    3.52, // D  - Major 2nd
    5.38, // D# - Minor 3rd (characteristic of minor)
    2.60, // E  - Major 3rd
    3.53, // F  - Perfect 4th
    2.54, // F# - Tritone
    4.75, // G  - Perfect 5th
    3.98, // G# - Minor 6th (characteristic of minor)
    2.69, // A  - Major 6th
    3.34, // A# - Minor 7th
    3.17, // B  - Major 7th
];

/// All possible key names in circle of fifths order
pub const KEY_NAMES: [&str; 12] = ["C", "G", "D", "A", "E", "B", "F#", "C#", "G#", "D#", "A#", "F"];

/// Maps pitch class to key name
pub fn pitch_class_to_key_name(pitch_class: usize) -> &'static str {
    match pitch_class {
        0 => "C",
        1 => "C#",
        2 => "D",
        3 => "D#",
        4 => "E",
        5 => "F",
        6 => "F#",
        7 => "G",
        8 => "G#",
        9 => "A",
        10 => "A#",
        11 => "B",
        _ => "UNKNOWN",
    }
}

/// Returns the minor key name for a given pitch class
pub fn pitch_class_to_minor_key_name(pitch_class: usize) -> String {
    format!("{}m", pitch_class_to_key_name(pitch_class))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Pitch Class to Key Name Tests =====

    #[test]
    fn test_pitch_class_to_key_name_all_classes() {
        assert_eq!(pitch_class_to_key_name(0), "C");
        assert_eq!(pitch_class_to_key_name(1), "C#");
        assert_eq!(pitch_class_to_key_name(2), "D");
        assert_eq!(pitch_class_to_key_name(3), "D#");
        assert_eq!(pitch_class_to_key_name(4), "E");
        assert_eq!(pitch_class_to_key_name(5), "F");
        assert_eq!(pitch_class_to_key_name(6), "F#");
        assert_eq!(pitch_class_to_key_name(7), "G");
        assert_eq!(pitch_class_to_key_name(8), "G#");
        assert_eq!(pitch_class_to_key_name(9), "A");
        assert_eq!(pitch_class_to_key_name(10), "A#");
        assert_eq!(pitch_class_to_key_name(11), "B");
    }

    #[test]
    fn test_pitch_class_to_key_name_out_of_range() {
        assert_eq!(pitch_class_to_key_name(12), "UNKNOWN");
        assert_eq!(pitch_class_to_key_name(100), "UNKNOWN");
        assert_eq!(pitch_class_to_key_name(usize::MAX), "UNKNOWN");
    }

    // ===== Minor Key Name Tests =====

    #[test]
    fn test_pitch_class_to_minor_key_name_all_classes() {
        assert_eq!(pitch_class_to_minor_key_name(0), "Cm");
        assert_eq!(pitch_class_to_minor_key_name(1), "C#m");
        assert_eq!(pitch_class_to_minor_key_name(2), "Dm");
        assert_eq!(pitch_class_to_minor_key_name(3), "D#m");
        assert_eq!(pitch_class_to_minor_key_name(4), "Em");
        assert_eq!(pitch_class_to_minor_key_name(5), "Fm");
        assert_eq!(pitch_class_to_minor_key_name(6), "F#m");
        assert_eq!(pitch_class_to_minor_key_name(7), "Gm");
        assert_eq!(pitch_class_to_minor_key_name(8), "G#m");
        assert_eq!(pitch_class_to_minor_key_name(9), "Am");
        assert_eq!(pitch_class_to_minor_key_name(10), "A#m");
        assert_eq!(pitch_class_to_minor_key_name(11), "Bm");
    }

    #[test]
    fn test_pitch_class_to_minor_key_name_out_of_range() {
        assert_eq!(pitch_class_to_minor_key_name(12), "UNKNOWNm");
        assert_eq!(pitch_class_to_minor_key_name(100), "UNKNOWNm");
    }

    // ===== Profile Constant Tests =====

    #[test]
    fn test_major_profile_length() {
        assert_eq!(MAJOR_PROFILE.len(), 12);
    }

    #[test]
    fn test_minor_profile_length() {
        assert_eq!(MINOR_PROFILE.len(), 12);
    }

    #[test]
    fn test_major_profile_tonic_is_strongest() {
        // In C major, C (index 0) should have the highest weight
        let tonic_weight = MAJOR_PROFILE[0];
        for (i, &weight) in MAJOR_PROFILE.iter().enumerate().skip(1) {
            assert!(
                tonic_weight > weight,
                "Tonic (C) weight {} should be stronger than index {} weight {}",
                tonic_weight,
                i,
                weight
            );
        }
    }

    #[test]
    fn test_minor_profile_tonic_is_strongest() {
        // In C minor, C (index 0) should have the highest weight
        let tonic_weight = MINOR_PROFILE[0];
        for (i, &weight) in MINOR_PROFILE.iter().enumerate().skip(1) {
            assert!(
                tonic_weight > weight,
                "Tonic (C) weight {} should be stronger than index {} weight {}",
                tonic_weight,
                i,
                weight
            );
        }
    }

    #[test]
    fn test_major_profile_known_values() {
        // Test a few key theoretical values from Krumhansl & Kessler (1982)
        assert_eq!(MAJOR_PROFILE[0], 6.35); // Tonic
        assert_eq!(MAJOR_PROFILE[4], 4.38); // Major third
        assert_eq!(MAJOR_PROFILE[7], 5.19); // Perfect fifth
    }

    #[test]
    fn test_minor_profile_known_values() {
        // Test a few key theoretical values from Krumhansl & Kessler (1982)
        assert_eq!(MINOR_PROFILE[0], 6.33); // Tonic
        assert_eq!(MINOR_PROFILE[3], 5.38); // Minor third (characteristic)
        assert_eq!(MINOR_PROFILE[7], 4.75); // Perfect fifth
    }

    #[test]
    fn test_major_profile_all_positive() {
        for (i, &weight) in MAJOR_PROFILE.iter().enumerate() {
            assert!(weight > 0.0, "Major profile index {} should be positive", i);
        }
    }

    #[test]
    fn test_minor_profile_all_positive() {
        for (i, &weight) in MINOR_PROFILE.iter().enumerate() {
            assert!(weight > 0.0, "Minor profile index {} should be positive", i);
        }
    }

    // ===== KEY_NAMES Constant Tests =====

    #[test]
    fn test_key_names_length() {
        assert_eq!(KEY_NAMES.len(), 12);
    }

    #[test]
    fn test_key_names_circle_of_fifths_order() {
        // Circle of fifths starting from C
        assert_eq!(KEY_NAMES[0], "C");
        assert_eq!(KEY_NAMES[1], "G"); // +7 semitones
        assert_eq!(KEY_NAMES[2], "D"); // +7 semitones
        assert_eq!(KEY_NAMES[3], "A"); // +7 semitones
        assert_eq!(KEY_NAMES[4], "E"); // +7 semitones
        assert_eq!(KEY_NAMES[5], "B"); // +7 semitones
        assert_eq!(KEY_NAMES[6], "F#"); // +7 semitones
    }

    #[test]
    fn test_key_names_no_duplicates() {
        let mut seen = std::collections::HashSet::new();
        for key in KEY_NAMES.iter() {
            assert!(seen.insert(key), "Duplicate key found: {}", key);
        }
    }

    #[test]
    fn test_key_names_all_valid() {
        let valid_keys = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        for key in KEY_NAMES.iter() {
            assert!(valid_keys.contains(key), "Invalid key name: {}", key);
        }
    }

    // ===== Integration Tests =====

    #[test]
    fn test_major_third_is_prominent_in_major() {
        // Major third (E in C major = index 4) should be more prominent than minor third
        let major_third = MAJOR_PROFILE[4]; // E
        let minor_third = MAJOR_PROFILE[3]; // D#/Eb
        assert!(
            major_third > minor_third,
            "Major third should be more prominent in major key"
        );
    }

    #[test]
    fn test_minor_third_is_prominent_in_minor() {
        // Minor third (D#/Eb in C minor = index 3) should be more prominent than major third
        let minor_third = MINOR_PROFILE[3]; // D#/Eb
        let major_third = MINOR_PROFILE[4]; // E
        assert!(
            minor_third > major_third,
            "Minor third should be more prominent in minor key"
        );
    }

    #[test]
    fn test_perfect_fifth_prominent_in_both() {
        // Perfect fifth (G in C = index 7) should be prominent in both major and minor
        let major_fifth = MAJOR_PROFILE[7];
        let minor_fifth = MINOR_PROFILE[7];

        // Fifth should be second strongest in major
        assert!(
            major_fifth > 5.0,
            "Perfect fifth should be prominent in major"
        );
        // Fifth should be strong in minor
        assert!(
            minor_fifth > 4.0,
            "Perfect fifth should be prominent in minor"
        );
    }
}

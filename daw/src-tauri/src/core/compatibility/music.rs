   /// Music theory utilities - Pure functions for musical analysis
   ///
   /// Trusty Module: All functions are pure - no I/O, fully deterministic.

use super::types::{Key, KeySignature, Mode};

/// Calculate distance between two keys in semitones
///
/// Returns the minimum distance around the circle of fifths (0-6).
/// Example: C to G is 5 semitones (perfect fifth).
///
/// # Arguments
/// * `key1` - First key
/// * `key2` - Second key
///
/// # Returns
/// Minimum distance (0-6 semitones)
pub fn key_distance(key1: Key, key2: Key) -> i32 {
    let diff = (key1.semitone() - key2.semitone()).abs();
    // Return minimum distance around circle of fifths
    diff.min(12 - diff)
}

/// Check if two key signatures are compatible
///
/// Keys are compatible if they:
/// - Are the same key
/// - Are relative major/minor (share the same notes)
/// - Are a perfect fifth apart (circle of fifths)
pub fn keys_compatible(ks1: &KeySignature, ks2: &KeySignature) -> bool {
    // Same key = perfect match
    if ks1.key == ks2.key && ks1.mode == ks2.mode {
        return true;
    }

    // Relative major/minor (e.g., C major and A minor)
    if is_relative_key(ks1, ks2) {
        return true;
    }

    // Perfect fifth apart (very compatible)
    let distance = key_distance(ks1.key, ks2.key);
    if distance == 5 {
        return true;
    }

    false
}

/// Check if two keys are relative (share the same notes)
///
/// Relative keys have different modes but share the same notes.
/// Example: C Major and A Minor are relative keys.
///
/// # Arguments
/// * `ks1` - First key signature
/// * `ks2` - Second key signature
///
/// # Returns
/// True if keys are relative (e.g., C Major and A Minor)
pub fn is_relative_key(ks1: &KeySignature, ks2: &KeySignature) -> bool {
    if ks1.mode == ks2.mode {
        return false;
    }

    // A minor is relative to C major (3 semitones down from major to minor)
    let major = if ks1.mode == Mode::Major { ks1 } else { ks2 };
    let minor = if ks1.mode == Mode::Minor { ks1 } else { ks2 };

    let diff = (major.key.semitone() - minor.key.semitone() + 12) % 12;
    diff == 3 // Minor is 3 semitones below its relative major
}

/// Get compatibility score for two key signatures (0-100)
///
/// Scoring:
/// - 100: Same key and mode (perfect match)
/// - 95: Relative major/minor (share notes)
/// - 85: Perfect fifth apart (circle of fifths)
/// - 70: Major/minor third apart
/// - 55: Whole tone apart
/// - 40: Semitone apart
/// - 20: Tritone (augmented fourth)
pub fn key_compatibility_score(ks1: &KeySignature, ks2: &KeySignature) -> f32 {
    // Perfect match
    if ks1.key == ks2.key && ks1.mode == ks2.mode {
        return 100.0;
    }

    // Relative keys (share notes)
    if is_relative_key(ks1, ks2) {
        return 95.0;
    }

    let distance = key_distance(ks1.key, ks2.key);

    // Perfect fifth (7 semitones) - very compatible
    if distance == 5 {
        return 85.0;
    }

    // Major/minor third (3-4 semitones) - compatible
    if distance == 3 || distance == 4 {
        return 70.0;
    }

    // Whole tone (2 semitones) - somewhat compatible
    if distance == 2 {
        return 55.0;
    }

    // Adjacent keys (1 semitone) - less compatible
    if distance == 1 {
        return 40.0;
    }

    // Tritone (6 semitones) - least compatible
    if distance == 6 {
        return 20.0;
    }

    50.0 // Default
}

/// Calculate BPM compatibility score (0-100)
///
/// Scores tempo similarity for DJ mixing and mashups.
///
/// # Arguments
/// * `bpm1` - First tempo in BPM
/// * `bpm2` - Second tempo in BPM
///
/// # Returns
/// Compatibility score (0-100)
pub fn bpm_compatibility_score(bpm1: f32, bpm2: f32) -> f32 {
    let diff = (bpm1 - bpm2).abs();

    // Perfect match
    if diff < 1.0 {
        return 100.0;
    }

    // Very close (within 5 BPM)
    if diff < 5.0 {
        return 95.0 - (diff * 1.0);
    }

    // Close (within 10 BPM)
    if diff < 10.0 {
        return 90.0 - (diff * 0.5);
    }

    // Within 20 BPM
    if diff < 20.0 {
        return 80.0 - (diff * 0.3);
    }

    // Within 40 BPM
    if diff < 40.0 {
        return 70.0 - (diff * 0.2);
    }

    // Too different
    30.0
}

/// Check if BPMs can be time-stretched to match
///
/// Returns true if the tempo ratio matches common musical ratios:
/// - 2:1 (double-time)
/// - 3:2 (sesquialtera)
/// - 4:3 (perfect fourth)
/// - And their inverses
///
/// # Arguments
/// * `bpm1` - First tempo
/// * `bpm2` - Second tempo
///
/// # Returns
/// True if tempos can be time-stretched with minimal artifacts
pub fn bpm_time_stretchable(bpm1: f32, bpm2: f32) -> bool {
    let ratio = bpm1 / bpm2;
    // Check if ratio is close to 2:1, 3:2, 4:3 (common musical ratios)
    let ratios = [2.0, 1.5, 1.333, 0.5, 0.667, 0.75];

    for target_ratio in ratios {
        if (ratio - target_ratio).abs() < 0.1 {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_distance_same() {
        assert_eq!(key_distance(Key::C, Key::C), 0);
        assert_eq!(key_distance(Key::A, Key::A), 0);
    }

    #[test]
    fn test_key_distance_fifth() {
        // C to G is a perfect fifth (5 semitones forward, 7 back)
        assert_eq!(key_distance(Key::C, Key::G), 5);
        // Going the other way should be the same
        assert_eq!(key_distance(Key::G, Key::C), 5);
    }

    #[test]
    fn test_key_distance_tritone() {
        // C to F# is a tritone (6 semitones - maximum distance)
        assert_eq!(key_distance(Key::C, Key::FSharp), 6);
    }

    #[test]
    fn test_key_distance_wraparound() {
        // A to C is 3 semitones forward (A=9, C=0: (9-0).abs().min(12-9) = 3)
        assert_eq!(key_distance(Key::A, Key::C), 3);
    }

    #[test]
    fn test_relative_keys() {
        let c_major = KeySignature {
            key: Key::C,
            mode: Mode::Major,
        };
        let a_minor = KeySignature {
            key: Key::A,
            mode: Mode::Minor,
        };

        assert!(is_relative_key(&c_major, &a_minor));
        assert!(is_relative_key(&a_minor, &c_major));
    }

    #[test]
    fn test_not_relative_keys() {
        let c_major = KeySignature {
            key: Key::C,
            mode: Mode::Major,
        };
        let c_minor = KeySignature {
            key: Key::C,
            mode: Mode::Minor,
        };

        assert!(!is_relative_key(&c_major, &c_minor));
    }

    #[test]
    fn test_keys_compatible() {
        let c_major = KeySignature {
            key: Key::C,
            mode: Mode::Major,
        };
        let g_major = KeySignature {
            key: Key::G,
            mode: Mode::Major,
        };

        assert!(keys_compatible(&c_major, &c_major)); // Same key
        assert!(keys_compatible(&c_major, &g_major)); // Perfect fifth
    }

    #[test]
    fn test_key_compatibility_perfect_match() {
        let key = KeySignature {
            key: Key::C,
            mode: Mode::Major,
        };

        assert_eq!(key_compatibility_score(&key, &key), 100.0);
    }

    #[test]
    fn test_key_compatibility_relative() {
        let c_major = KeySignature {
            key: Key::C,
            mode: Mode::Major,
        };
        let a_minor = KeySignature {
            key: Key::A,
            mode: Mode::Minor,
        };

        assert_eq!(key_compatibility_score(&c_major, &a_minor), 95.0);
    }

    #[test]
    fn test_key_compatibility_fifth() {
        let c_major = KeySignature {
            key: Key::C,
            mode: Mode::Major,
        };
        let g_major = KeySignature {
            key: Key::G,
            mode: Mode::Major,
        };

        assert_eq!(key_compatibility_score(&c_major, &g_major), 85.0);
    }

    #[test]
    fn test_bpm_compatibility_exact_match() {
        assert_eq!(bpm_compatibility_score(120.0, 120.0), 100.0);
    }

    #[test]
    fn test_bpm_compatibility_close() {
        let score = bpm_compatibility_score(120.0, 122.0);
        assert!(score > 90.0);
        assert!(score < 100.0);
    }

    #[test]
    fn test_bpm_compatibility_far() {
        let score = bpm_compatibility_score(120.0, 180.0);
        assert!(score < 50.0);
    }

    #[test]
    fn test_bpm_time_stretchable() {
        // Double tempo should be stretchable
        assert!(bpm_time_stretchable(120.0, 240.0));

        // 3:2 ratio should be stretchable
        assert!(bpm_time_stretchable(120.0, 180.0));

        // Random ratio should not be
        assert!(!bpm_time_stretchable(120.0, 137.0));
    }
}

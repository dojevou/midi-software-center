//! Krumhansl-Schmuckler Key Profiles
//!
//! These profiles represent the expected distribution of pitch classes
//! in major and minor keys, derived from music theory research.

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
pub const KEY_NAMES: [&str; 12] = [
    "C", "G", "D", "A", "E", "B", "F#", "C#", "G#", "D#", "A#", "F",
];

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

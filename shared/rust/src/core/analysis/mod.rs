pub mod auto_tagger;
/// Musical analysis modules
///
/// This module provides:
/// - BPM detection
/// - Key detection
/// - Auto-tagging
/// - Key profile data (genre-specific profiles for key detection)
pub mod bpm_detector;
pub mod key_detector;
pub mod key_profiles;

// Re-export main functions
pub use auto_tagger::generate_tags;
pub use bpm_detector::detect_bpm;
pub use key_detector::detect_key;

// Re-export key profile types and constants
pub use key_profiles::{
    calculate_profile_correlation,
    calculate_profile_strength,
    combine_profiles,
    create_key_profile,
    detect_profile_type,
    get_all_profiles,
    get_default_profile,
    get_profiles_for_genre,
    get_scale_degree_weights,
    normalize_profile,
    // Utility functions
    rotate_profile,
    // Profile type and structs
    KeyProfile,
    ProfileType,
    AMBIENT_PROFILE,
    // Genre-specific profiles
    BLUES_PROFILE,
    CLASSICAL_PROFILE,
    DRUM_AND_BASS_PROFILE,
    EDM_PROFILE,
    HIPHOP_PROFILE,
    JAZZ_MAJOR_PROFILE,
    JAZZ_MINOR_PROFILE,
    // Standard profiles
    KRUMHANSL_MAJOR_PROFILE,
    KRUMHANSL_MINOR_PROFILE,
    POP_PROFILE,
    SIMPLE_MAJOR_PROFILE,
    SIMPLE_MINOR_PROFILE,
    TEMPERLEY_MAJOR_PROFILE,
    TEMPERLEY_MINOR_PROFILE,
    TRANCE_PROFILE,
};

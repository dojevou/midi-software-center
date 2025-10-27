//! Analysis modules for MIDI file processing

pub mod auto_tagger;
pub mod bpm_detector;
pub mod key_detector;
pub mod key_profiles;

// Re-export main types
pub use auto_tagger::{AutoTagger, Tag};
pub use bpm_detector::{detect_bpm, BpmDetectionMethod, BpmDetectionResult, BpmMetadata};
pub use key_detector::{detect_key, KeyDetectionResult, ScaleType};

// Tests will be added during Phase 1 of test coverage initiative
// #[cfg(test)]
// mod tests;

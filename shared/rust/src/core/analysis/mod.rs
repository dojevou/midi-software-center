//! Musical analysis modules
//!
//! This module provides:
//! - BPM detection
//! - Key detection
//! - Auto-tagging
//! - Key profile data

pub mod bpm_detector;
pub mod key_detector;
pub mod key_profiles;
pub mod auto_tagger;

// Re-export main functions
pub use bpm_detector::detect_bpm;
pub use key_detector::detect_key;
pub use auto_tagger::generate_tags;

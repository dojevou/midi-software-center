   /// Compatibility module - Musical matching algorithms
   ///
   /// Trusty Module: Pure functions for calculating compatibility between MIDI files.
   /// NO I/O operations - all functions are deterministic and testable.

pub mod music;
pub mod scoring;
pub mod types;

// Re-export commonly used items
#[allow(unused_imports)]
pub use music::{
    bpm_compatibility_score, bpm_time_stretchable, is_relative_key, key_compatibility_score,
    key_distance, keys_compatible,
};
#[allow(unused_imports)]
pub use scoring::{calculate_compatibility, explain_compatibility};
#[allow(unused_imports)]
pub use types::{CompatibilityScore, Key, KeySignature, Mode};

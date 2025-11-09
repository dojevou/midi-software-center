   /// Filename and path normalization utilities.
   ///
   /// This module provides utilities for normalizing MIDI filenames and paths,
   /// ensuring consistency across the system.

pub mod filename;

// Re-export commonly used functions for convenience
pub use filename::{normalize_midi_filename, needs_normalization};

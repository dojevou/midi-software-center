
/// Filename and path normalization utilities.
///
/// This module provides utilities for normalizing MIDI filenames and paths,
/// ensuring consistency across the system.
pub mod filename;

// Re-export commonly used functions for convenience
pub use filename::{needs_normalization, normalize_midi_filename};

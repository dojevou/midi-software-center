
/// Splitting Module
///
/// Pure logic for splitting multi-track MIDI files into individual tracks.
///
/// # Archetype: TRUSTY MODULE
///
/// This module contains pure functions for:
/// - Parsing multi-track MIDI files
/// - Splitting into separate Format 0 (single-track) files
/// - Extracting track metadata
///
/// All functions operate on byte arrays with no I/O operations.
pub mod track_splitter;

// Re-export main types and functions
pub use track_splitter::{
    count_notes, create_single_track_midi, extract_instrument, extract_primary_channel,
    extract_track_name, get_instrument_name, is_tempo_track, split_tracks, SplitError, SplitTrack,
};

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
/// - Automatic repair of corrupted MIDI files
///
/// All functions operate on byte arrays with no I/O operations.
pub mod auto_repair;
pub mod track_splitter;

// Re-export main types and functions
pub use track_splitter::{
    count_notes, create_single_track_midi, extract_instrument, extract_primary_channel,
    extract_track_name, get_instrument_name, is_tempo_track, split_tracks, SplitError, SplitTrack,
};

// Re-export auto-repair types and functions
pub use auto_repair::{attempt_repair, split_tracks_with_repair, AutoRepairError, RepairResult};

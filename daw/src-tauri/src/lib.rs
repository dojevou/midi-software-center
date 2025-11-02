//! MIDI Library DAW Interface
//!
//! Core library for DAW functionality including MIDI playback, sequencing, and search.

pub mod commands;
pub mod core;
pub mod midi;
pub mod models;
pub mod sequencer;

// Re-export commonly used types
pub use models::{MidiFile, FileDetails, AppError, AppResult};

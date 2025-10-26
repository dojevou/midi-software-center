//! MIDI Library DAW Interface
//!
//! Core library for DAW functionality including MIDI playback, sequencing, and search.

pub mod core;
pub mod models;

// Re-export commonly used types
pub use models::{MidiFile, FileDetails, AppError, AppResult};

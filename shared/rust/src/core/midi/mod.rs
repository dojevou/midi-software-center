//! MIDI file parsing and types
//!
//! This module provides:
//! - MIDI file parsing
//! - MIDI data types
//! - Error handling

pub mod parser;
pub mod types;
pub mod error;

// Re-export commonly used items
pub use parser::parse_midi_file;
pub use types::{MidiFile, Event, Track};
pub use error::{MidiParseError, Result};


pub mod error;
/// MIDI file parsing and types
///
/// This module provides:
/// - MIDI file parsing
/// - MIDI data types
/// - Error handling
pub mod parser;
pub mod text_metadata;
pub mod types;

// Re-export commonly used items
pub use error::{MidiParseError, Result};
pub use parser::parse_midi_file;
pub use text_metadata::TextMetadata;
pub use types::{Event, MidiFile, Track};

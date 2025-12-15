pub mod error;
/// MIDI file parsing and types
///
/// This module provides:
/// - MIDI file parsing (analysis_parser for pipeline, playback_parser for DAW)
/// - MIDI data types
/// - Error handling
pub mod analysis_parser;
pub mod text_metadata;
pub mod types;

// DAW MIDI modules (real-time playback)
pub mod loader;
pub mod playback_parser;
pub mod playback_types;
pub mod validator;
pub mod writer;

// Re-export commonly used items (analysis/pipeline)
pub use analysis_parser::parse_midi_file;
pub use error::{MidiParseError, Result};
pub use text_metadata::TextMetadata;
pub use types::{Event, MidiFile, Track};

// Re-export DAW items (playback)
#[allow(unused_imports)]
pub use loader::{load_midi_file as load_midi, load_midi_from_bytes, LoadedMidiFile};
#[allow(unused_imports)]
pub use playback_parser::{parse_midi, ParseError};
#[allow(unused_imports)]
pub use playback_types::{MidiEventType, MidiMessage};
#[allow(unused_imports)]
pub use validator::{
    validate_channel, validate_control_value, validate_message, validate_note, validate_velocity,
};
#[allow(unused_imports)]
pub use writer::write_midi_file;

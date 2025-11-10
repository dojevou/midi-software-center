pub mod loader;
/// MIDI Core - Trusty Modules
///
/// Pure functions for MIDI event encoding, decoding, and validation.
/// NO I/O operations - all functions are deterministic and testable.
pub mod parser;
pub mod types;
pub mod validator;
pub mod writer;

#[allow(unused_imports)]
pub use loader::{load_midi_file, LoadedMidiFile};
#[allow(unused_imports)]
pub use parser::{parse_midi, ParseError};
#[allow(unused_imports)]
pub use types::{MidiEventType, MidiMessage};
#[allow(unused_imports)]
pub use validator::{
    validate_channel, validate_control_value, validate_message, validate_note, validate_velocity,
};
#[allow(unused_imports)]
pub use writer::write_midi_file;

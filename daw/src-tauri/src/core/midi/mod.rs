   /// MIDI Core - Trusty Modules
   ///
   /// Pure functions for MIDI event encoding, decoding, and validation.
   /// NO I/O operations - all functions are deterministic and testable.

pub mod parser;
pub mod types;
pub mod validator;
pub mod writer;
pub mod loader;

#[allow(unused_imports)]
pub use parser::{parse_midi, ParseError};
#[allow(unused_imports)]
pub use types::{MidiMessage, MidiEventType};
#[allow(unused_imports)]
pub use validator::{
    validate_channel, validate_note, validate_velocity,
    validate_control_value, validate_message
};
#[allow(unused_imports)]
pub use writer::write_midi_file;
#[allow(unused_imports)]
pub use loader::{load_midi_file, LoadedMidiFile};

   /// Data models for the DAW application
   ///
   /// These models match the TypeScript frontend types for proper serialization.
   /// Trusty Module: Type definitions only, no I/O operations.

pub mod midi_file;
pub mod search;
pub mod midi;
pub mod sequencer;
pub mod analysis;
pub mod error;

// Re-export commonly used types
#[allow(unused_imports)]
pub use midi_file::{MidiFile, FileDetails};
#[allow(unused_imports)]
pub use search::{SearchFilters, SearchResponse, Suggestion, FilterOption};
#[allow(unused_imports)]
pub use midi::{MidiDevice, MidiEventType, MidiEvent, MidiNote, MidiPattern, ConnectionStatus};
#[allow(unused_imports)]
pub use sequencer::{Track, TrackProperties, PlaybackPosition};
#[allow(unused_imports)]
pub use analysis::CompatibleFile;
#[allow(unused_imports)]
pub use error::AppError;

// Types used internally only
#[allow(unused_imports)]
pub use sequencer::SequencerState;
#[allow(unused_imports)]
pub use analysis::{Key, Mode};
#[allow(unused_imports)]
pub use error::AppResult;

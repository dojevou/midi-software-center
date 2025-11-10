pub mod analysis;
pub mod error;
pub mod midi;
/// Data models for the DAW application
///
/// These models match the TypeScript frontend types for proper serialization.
/// Trusty Module: Type definitions only, no I/O operations.
pub mod midi_file;
pub mod search;
pub mod sequencer;

// Re-export commonly used types
#[allow(unused_imports)]
pub use analysis::CompatibleFile;
#[allow(unused_imports)]
pub use error::AppError;
#[allow(unused_imports)]
pub use midi::{ConnectionStatus, MidiDevice, MidiEvent, MidiEventType, MidiNote, MidiPattern};
#[allow(unused_imports)]
pub use midi_file::{FileDetails, MidiFile};
#[allow(unused_imports)]
pub use search::{FilterOption, SearchFilters, SearchResponse, Suggestion};
#[allow(unused_imports)]
pub use sequencer::{PlaybackPosition, Track, TrackProperties};

// Types used internally only
#[allow(unused_imports)]
pub use analysis::{Key, Mode};
#[allow(unused_imports)]
pub use error::AppResult;
#[allow(unused_imports)]
pub use sequencer::SequencerState;

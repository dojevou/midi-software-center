//! Database model types

pub mod midi_file;
pub mod midi;
pub mod sequencer;
pub mod analysis;
pub mod search;
pub mod error;

// Re-export main types
pub use midi_file::File;
pub use midi::MidiMetadata;
pub use sequencer::SequencerTrack;
pub use analysis::AnalysisResult;
pub use search::SearchFilters;
pub use error::DbError;

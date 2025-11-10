
pub mod analysis;
pub mod error;
pub mod midi;
/// Database model types
pub mod midi_file;
pub mod search;
pub mod sequencer;

// Re-export main types
pub use analysis::AnalysisResult;
pub use error::DbError;
pub use midi::MidiMetadata;
pub use midi_file::File;
pub use search::SearchFilters;
pub use sequencer::SequencerTrack;

pub mod engine;
pub mod scheduler;
/// Sequencer implementation
///
/// Grown-up Scripts: Manage sequencer state, tracks, and playback.
/// Delegates to Trusty Modules for timing calculations.
pub mod track;

#[allow(unused_imports)]
pub use engine::{PlaybackState, SequencerEngine};
pub use scheduler::{EventScheduler, ScheduledEvent};
pub use track::TrackManager;

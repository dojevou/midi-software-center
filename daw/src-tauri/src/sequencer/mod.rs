   /// Sequencer implementation
   ///
   /// Grown-up Scripts: Manage sequencer state, tracks, and playback.
   /// Delegates to Trusty Modules for timing calculations.

pub mod track;
pub mod scheduler;
pub mod engine;

pub use track::TrackManager;
pub use scheduler::{EventScheduler, ScheduledEvent};
#[allow(unused_imports)]
pub use engine::{SequencerEngine, PlaybackState};

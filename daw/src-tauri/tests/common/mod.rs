
pub mod assertions;
pub mod builders;
/// Common test infrastructure for DAW command tests
/// Provides database mocks, MIDI device mocks, fixtures, builders, and assertions
pub mod database;
pub mod fixtures;
pub mod mocks;

pub use assertions::*;
pub use builders::{MidiFileBuilder, SequencerStateBuilder, TrackBuilder};
pub use database::TestDatabase;
pub use fixtures::{FileFixtures, TestFixtures};
pub use mocks::{EmittedEvent, MockAppHandle, MockMidiDevice, MockWindow};

// Re-export commonly used items
pub use sqlx::PgPool;
pub use std::sync::Arc;
pub use tokio::sync::Mutex;

//! Common test infrastructure for DAW command tests
//! Provides database mocks, MIDI device mocks, fixtures, builders, and assertions

pub mod database;
pub mod mocks;
pub mod builders;
pub mod assertions;
pub mod fixtures;

pub use database::TestDatabase;
pub use mocks::{MockWindow, MockAppHandle, MockMidiDevice, EmittedEvent};
pub use builders::{MidiFileBuilder, TrackBuilder, SequencerStateBuilder};
pub use assertions::*;
pub use fixtures::{TestFixtures, FileFixtures};

// Re-export commonly used items
pub use sqlx::PgPool;
pub use std::sync::Arc;
pub use tokio::sync::Mutex;

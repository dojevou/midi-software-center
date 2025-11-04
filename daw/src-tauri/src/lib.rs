//! MIDI Library DAW Interface
//!
//! Core library for DAW functionality including MIDI playback, sequencing, and search.

pub mod browsers;
pub mod command_palette;
pub mod commands;
pub mod core;
pub mod editors;
pub mod midi;
pub mod models;
pub mod sequencer;
pub mod settings;
pub mod windows;

// Re-export commonly used types
pub use models::{MidiFile, FileDetails, AppError, AppResult};

// Re-export command palette types
pub use command_palette::{CommandEntry, CommandCategory, CommandPalette, SearchResult};

// Re-export settings types
pub use settings::{
    AppSettings, AdvancedSettings, AudioSettings, DisplaySettings, GeneralSettings,
    ImportExportSettings, KeyboardSettings, LibrarySettings, MidiSettings, MixerSettings,
    PerformanceSettings, PlaybackSettings, PrivacySettings, RecordingSettings, SyncSettings,
    TrackSettings,
};

/// MIDI Library DAW Interface
///
/// Core library for DAW functionality including MIDI playback, sequencing, and search.
pub mod automation;
pub mod browsers;
pub mod command_palette;
pub mod commands;
pub mod core;
pub mod editors;
pub mod midi;
pub mod models;
pub mod profiling;
pub mod sequencer;
pub mod settings;
pub mod undo_redo;
pub mod windows;

// Re-export commonly used types
pub use models::{AppError, AppResult, FileDetails, MidiFile};

// Re-export command palette types
pub use command_palette::{CommandCategory, CommandEntry, CommandPalette, SearchResult};

// Re-export settings types
pub use settings::{
    AdvancedSettings, AppSettings, AudioSettings, DisplaySettings, GeneralSettings,
    ImportExportSettings, KeyboardSettings, LibrarySettings, MidiSettings, MixerSettings,
    PerformanceSettings, PlaybackSettings, PrivacySettings, RecordingSettings, SyncSettings,
    TrackSettings,
};

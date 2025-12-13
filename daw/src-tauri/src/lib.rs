// Clippy lint configuration for DAW crate
#![allow(clippy::too_many_arguments)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::manual_clamp)]
#![allow(clippy::ptr_arg)]

/// MIDI Library DAW Interface
///
/// Core library for DAW functionality including MIDI playback, sequencing, and search.
///
/// # Performance MIDI I/O
///
/// The `hardware` module provides high-performance MIDI I/O with multiple backends:
/// - **JACK**: ~3ms latency (Linux/macOS) - lock-free ringbuffers
/// - **ALSA Raw**: ~5ms latency (Linux) - direct kernel access
/// - **CoreMIDI**: ~5ms latency (macOS) - native framework
/// - **midir**: ~10-15ms latency - cross-platform fallback
///
/// Enable low-latency backends via Cargo features:
/// ```toml
/// [features]
/// pro-audio-linux = ["jack-backend", "alsa-backend"]
/// pro-audio-macos = ["coremidi-backend"]
/// ```
pub mod automation;
pub mod browsers;
pub mod command_palette;
pub mod commands;
pub mod core;
pub mod editors;
pub mod hardware;
pub mod midi;
pub mod midi_clock;
pub mod models;
pub mod notation;
pub mod profiling;
pub mod scripting;
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

// Re-export hardware types for high-performance MIDI I/O
pub use hardware::{
    MidiBackend, MidiBackendType, MidiManager, MidiPerformanceMetrics, MidiPort, PrecisionTimer,
    TimestampedMidiEvent,
};

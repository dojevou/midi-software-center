pub mod commands;
pub mod controller;
/// Undo/Redo System for DAW Editors
///
/// Generic undo/redo engine supporting all editor actions (Piano Roll, Velocity, Controller, Tempo, Track).
///
/// Architecture follows Three Archetypes Pattern:
/// - Command trait & CommandHistory: Trusty Module (pure logic, no I/O)
/// - Command implementations: Trusty Module (pure data transformations)
/// - Tauri command wrappers: Task-O-Matic (entry points)
///
/// This module provides a robust command pattern implementation with:
/// - Execute/Undo/Redo operations
/// - Command history management (undo/redo stacks)
/// - Memory limits and compression
/// - Serialization support for persistence
/// - Type-safe command implementations for all editor types
pub mod core;
pub mod performance;
pub mod piano_roll;
pub mod serialization;
pub mod tempo;
pub mod track;
pub mod velocity;

// Re-export commonly used types
pub use controller::{
    AddCCPointCommand, DeleteCCPointCommand, MoveCCPointCommand, SmoothCurveCommand,
};
pub use core::{Command, CommandHistory, UndoRedoError, UndoRedoResult};
pub use performance::{CommandCompressor, MemoryLimiter};
pub use piano_roll::{
    AddNoteCommand, DeleteNoteCommand, MoveNoteCommand, QuantizeNotesCommand, SetVelocityCommand,
    TransposeCommand,
};
pub use serialization::{CommandSerializer, SerializableCommand};
pub use tempo::{
    AddTempoMarkerCommand, RemoveTempoMarkerCommand, SetTempoCommand, SetTempoRampCommand,
};
pub use track::{
    AddTrackCommand, RemoveTrackCommand, RenameTrackCommand, SetTrackColorCommand,
    SetTrackVolumeCommand,
};
pub use velocity::{InterpolateVelocityCommand, ResetVelocityCommand, SetVelocityRangeCommand};

/// Maximum number of commands to keep in history (default)
pub const DEFAULT_MAX_DEPTH: usize = 100;

/// Maximum memory usage for command history in bytes (default: 10MB)
pub const DEFAULT_MAX_MEMORY: usize = 10 * 1024 * 1024;

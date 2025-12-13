/// MIDI Software Center - Unified Application
///
/// Combines Pipeline (batch processing, database) and DAW (real-time playback)
/// into a single desktop application.
// Shared modules for the unified app
pub mod shared;

// Re-export the command registry for easy access
pub use shared::TauriCommands;

// Re-export Pipeline library (package: midi-pipeline, lib: midi_pipeline)
pub use midi_pipeline;

// Re-export DAW library (package: midi-software-center-daw, lib: midi_software_center_daw)
pub use midi_software_center_daw;

// Re-export commonly used types from both libraries
pub use midi_pipeline::{AppError as PipelineError, AppResult as PipelineResult};
pub use midi_software_center_daw::{AppError as DAWError, AppResult as DAWResult};

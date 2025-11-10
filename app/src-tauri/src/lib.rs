/// MIDI Software Center - Unified Application
///
/// Combines Pipeline (batch processing, database) and DAW (real-time playback)
/// into a single desktop application.
// Re-export Pipeline library (package: midi-pipeline, lib: midi_pipeline)
pub use midi_pipeline;

// Re-export DAW library (package: midi-daw, lib: daw_lib)
pub use daw_lib;

// Re-export commonly used types from both libraries
pub use daw_lib::{AppError as DAWError, AppResult as DAWResult};
pub use midi_pipeline::{AppError as PipelineError, AppResult as PipelineResult};

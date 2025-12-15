//! Tauri commands for the MIDI Software Center
//!
//! This module contains all Tauri command handlers organized by feature:
//! - `pipeline` - Pipeline commands (file import, analysis, tagging, etc.)

pub mod pipeline;

// Re-export pipeline commands for convenience
pub use pipeline::*;

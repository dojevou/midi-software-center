
/// Tauri command handlers
///
/// All commands are Grown-up Scripts:
/// - Perform I/O (file system, database, network)
/// - Delegate business logic to Trusty Modules
/// - Handle errors and convert to frontend-friendly format
/// - Provide progress updates for long-running operations
pub mod analyze;
pub mod archive_import;
pub mod file_import;
pub mod files;
pub mod progress;
pub mod search;
pub mod split_file;
pub mod stats;
pub mod system;
pub mod tags;

// Re-export commonly used split types
pub use split_file::{split_and_import, SplitResult};

// Re-export analysis command
pub use analyze::start_analysis;

// Future command modules:
// pub mod playback;

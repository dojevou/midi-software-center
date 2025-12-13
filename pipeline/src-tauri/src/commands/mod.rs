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
pub mod health;
pub mod progress;
pub mod search;
pub mod split_file;
pub mod stats;
pub mod system;
pub mod tags;
pub mod vip3;

// Re-export commonly used split types and Tauri commands
pub use split_file::{split_and_import, split_file, split_file_batch, SplitResult};

// Re-export analysis command and types
pub use analyze::{
    analyze_single_file, batch_insert_analyzed_files, start_analysis, AnalyzedFile, FileRecord,
    TrackInstrument,
};

// Re-export health commands
pub use health::{
    check_meilisearch_health, check_postgres_health, check_system_health, get_cached_health,
    HealthState,
};

// Future command modules:
// pub mod playback;

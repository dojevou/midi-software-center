/// I/O operations module
///
/// Contains Grown-up Scripts that perform file I/O operations
pub mod decompressor;
pub mod error;

// Re-export error types
pub use error::{IoError, Result};

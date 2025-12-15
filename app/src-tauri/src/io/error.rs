/// I/O Error Types
///
/// Defines error types for the I/O layer using thiserror.
/// These errors cover file operations, archive extraction, and temporary file management.
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during I/O operations
#[derive(Error, Debug)]
pub enum IoError {
    /// Standard I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// ZIP archive error
    #[error("ZIP archive error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// Invalid file path (contains non-UTF8 characters)
    #[error("Invalid path (non-UTF8): {path:?}")]
    InvalidPath { path: PathBuf },

    /// Unsupported archive format
    #[error("Unsupported archive format: {path:?}")]
    UnsupportedFormat { path: PathBuf },

    /// Maximum extraction depth exceeded
    #[error("Maximum extraction depth ({max_depth}) exceeded at: {path:?}")]
    MaxDepthExceeded { max_depth: usize, path: PathBuf },

    /// Archive format not implemented yet
    #[error("Archive format {format:?} not yet implemented")]
    FormatNotImplemented { format: String },

    /// Lock poisoning error (from RwLock or Mutex)
    #[error("Lock poisoned")]
    LockPoisoned,

    /// Temporary directory creation failed
    #[error("Failed to create temporary directory")]
    TempDirCreation,

    /// Generic boxed error for compatibility
    #[error("Error: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// Result type for I/O operations
pub type Result<T> = std::result::Result<T, IoError>;

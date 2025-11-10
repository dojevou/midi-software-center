/// Error types for the DAW application
///
/// Trusty Module: Centralized error handling with proper type definitions.
use thiserror::Error;

/**
 * Application errors
 *
 * Unified error type for all DAW operations.
 * Implements proper error conversion and serialization.
 */
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("MIDI error: {0}")]
    Midi(String),

    #[error("File error: {0}")]
    File(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Sequencer error: {0}")]
    Sequencer(String),

    #[error("Connection error: {0}")]
    Connection(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/**
 * Result type for application
 *
 * Convenience type alias for operations that return AppError.
 */
pub type AppResult<T> = Result<T, AppError>;

//! Database error types
//!
//! Unified error handling for database operations using thiserror.

use thiserror::Error;

/// Database operation errors.
///
/// Covers connection, query, constraint, and data errors.
#[derive(Debug, Error)]
pub enum DbError {
    /// Database connection failed
    #[error("Database connection error: {0}")]
    Connection(String),

    /// Query execution failed
    #[error("Query error: {0}")]
    Query(String),

    /// Record not found
    #[error("Record not found: {0}")]
    NotFound(String),

    /// Duplicate record (unique constraint violation)
    #[error("Duplicate record: {0}")]
    Duplicate(String),

    /// Foreign key constraint violation
    #[error("Foreign key violation: {0}")]
    ForeignKeyViolation(String),

    /// Data validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Data serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Transaction error
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Migration error
    #[error("Migration error: {0}")]
    Migration(String),

    /// Pool exhausted
    #[error("Connection pool exhausted")]
    PoolExhausted,

    /// Timeout
    #[error("Database operation timed out: {0}")]
    Timeout(String),

    /// IO error (file operations)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Internal error
    #[error("Internal database error: {0}")]
    Internal(String),
}

/// Result type alias for database operations.
pub type DbResult<T> = Result<T, DbError>;

impl DbError {
    /// Create a connection error.
    #[must_use]
    pub fn connection(msg: impl Into<String>) -> Self {
        Self::Connection(msg.into())
    }

    /// Create a query error.
    #[must_use]
    pub fn query(msg: impl Into<String>) -> Self {
        Self::Query(msg.into())
    }

    /// Create a not found error.
    #[must_use]
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create a duplicate error.
    #[must_use]
    pub fn duplicate(msg: impl Into<String>) -> Self {
        Self::Duplicate(msg.into())
    }

    /// Create a validation error.
    #[must_use]
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create an internal error.
    #[must_use]
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// Check if this is a not found error.
    #[must_use]
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound(_))
    }

    /// Check if this is a duplicate error.
    #[must_use]
    pub fn is_duplicate(&self) -> bool {
        matches!(self, Self::Duplicate(_))
    }

    /// Check if this is a connection error.
    #[must_use]
    pub fn is_connection(&self) -> bool {
        matches!(self, Self::Connection(_))
    }

    /// Check if this is a transient error (retryable).
    #[must_use]
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            Self::Connection(_) | Self::PoolExhausted | Self::Timeout(_)
        )
    }
}

/// SQLx error conversion (when database feature is enabled)
#[cfg(feature = "database")]
impl From<sqlx::Error> for DbError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::NotFound("Row not found".to_string()),
            sqlx::Error::PoolTimedOut => Self::PoolExhausted,
            sqlx::Error::Database(db_err) => {
                let code = db_err.code().unwrap_or_default();
                let message = db_err.message().to_string();

                // PostgreSQL error codes
                match code.as_ref() {
                    "23505" => Self::Duplicate(message), // unique_violation
                    "23503" => Self::ForeignKeyViolation(message), // foreign_key_violation
                    "23502" => Self::Validation(format!("Not null violation: {message}")), // not_null_violation
                    "23514" => Self::Validation(format!("Check violation: {message}")), // check_violation
                    "42P01" => Self::Query(format!("Table not found: {message}")), // undefined_table
                    "42703" => Self::Query(format!("Column not found: {message}")), // undefined_column
                    "08000" | "08003" | "08006" => Self::Connection(message), // connection errors
                    "40001" => Self::Transaction(format!("Serialization failure: {message}")), // serialization_failure
                    "40P01" => Self::Transaction(format!("Deadlock detected: {message}")), // deadlock_detected
                    _ => Self::Query(message),
                }
            },
            sqlx::Error::Io(io_err) => Self::Io(io_err),
            sqlx::Error::Protocol(msg) => Self::Connection(msg),
            sqlx::Error::Configuration(err) => Self::Connection(err.to_string()),
            sqlx::Error::Tls(err) => Self::Connection(format!("TLS error: {err}")),
            _ => Self::Internal(err.to_string()),
        }
    }
}

/// Conversion to sqlx::Error for compatibility
#[cfg(feature = "database")]
impl From<DbError> for sqlx::Error {
    fn from(err: DbError) -> Self {
        sqlx::Error::Protocol(err.to_string())
    }
}

/// Error context extension trait.
pub trait ErrorContext<T> {
    /// Add context to an error.
    fn context(self, msg: &str) -> DbResult<T>;

    /// Add context with a closure.
    fn with_context<F: FnOnce() -> String>(self, f: F) -> DbResult<T>;
}

impl<T, E: Into<DbError>> ErrorContext<T> for Result<T, E> {
    fn context(self, msg: &str) -> DbResult<T> {
        self.map_err(|e| {
            let inner = e.into();
            DbError::Internal(format!("{msg}: {inner}"))
        })
    }

    fn with_context<F: FnOnce() -> String>(self, f: F) -> DbResult<T> {
        self.map_err(|e| {
            let inner = e.into();
            DbError::Internal(format!("{}: {inner}", f()))
        })
    }
}

/// Convert Option to DbResult with not found error.
pub trait OptionExt<T> {
    /// Convert None to NotFound error.
    fn ok_or_not_found(self, msg: &str) -> DbResult<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_not_found(self, msg: &str) -> DbResult<T> {
        self.ok_or_else(|| DbError::NotFound(msg.to_string()))
    }
}

// Note: Display is automatically implemented by thiserror via #[derive(Error)]
// The #[error("...")] attributes on each variant define the Display output.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = DbError::not_found("File with id 123");
        assert!(err.is_not_found());
        assert!(!err.is_duplicate());
        assert!(!err.is_transient());
    }

    #[test]
    fn test_duplicate_error() {
        let err = DbError::duplicate("Hash already exists");
        assert!(err.is_duplicate());
        assert!(!err.is_not_found());
    }

    #[test]
    fn test_transient_errors() {
        assert!(DbError::connection("Failed to connect").is_transient());
        assert!(DbError::PoolExhausted.is_transient());
        assert!(DbError::Timeout("Query timed out".to_string()).is_transient());
        assert!(!DbError::not_found("Not found").is_transient());
    }

    #[test]
    fn test_option_ext() {
        let some_value: Option<i32> = Some(42);
        let none_value: Option<i32> = None;

        assert_eq!(some_value.ok_or_not_found("value").unwrap(), 42);
        assert!(none_value.ok_or_not_found("value").is_err());
    }

    #[test]
    fn test_error_display() {
        let err = DbError::validation("BPM must be between 20 and 300");
        let msg = format!("{err}");
        assert!(msg.contains("Validation error"));
        assert!(msg.contains("BPM"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let db_err: DbError = io_err.into();
        assert!(matches!(db_err, DbError::Io(_)));
    }

    #[test]
    fn test_json_error_conversion() {
        let json_err: serde_json::Error = serde_json::from_str::<i32>("invalid").unwrap_err();
        let db_err: DbError = json_err.into();
        assert!(matches!(db_err, DbError::Json(_)));
    }
}


/// Error Handling Module - PURE FUNCTION ARCHETYPE
///
/// PURPOSE: Transform and convert error types for Tauri commands
/// ARCHETYPE: Pure Function (deterministic transformations, no I/O)
/// LOCATION: pipeline/src-tauri/src/error.rs
///
/// ✅ CAN: Transform errors (sqlx::Error → AppError)
/// ✅ CAN: Convert types (AppError → String)
/// ✅ SHOULD: Be deterministic
/// ❌ NO: I/O operations
/// ❌ NO: Side effects
/// ❌ NO: State
use std::fmt;

/// Application error types
///
/// Centralized error handling for all Tauri commands.
/// All variants can be converted to String for frontend consumption.
///
/// # Examples
///
/// ```rust
/// use error::AppError;
///
/// // From database error
/// let db_err = AppError::DatabaseError(sqlx_error);
///
/// // From validation
/// let val_err = AppError::ValidationError("Invalid BPM range".to_string());
///
/// // Convert to String for Tauri
/// let error_msg: String = app_err.into();
/// ```
#[derive(Debug)]
pub enum AppError {
    /// Database operation failed
    DatabaseError(sqlx::Error),

    /// Requested resource not found
    NotFound(String),

    /// Input validation failed
    ValidationError(String),

    /// File I/O operation failed
    IOError(std::io::Error),

    /// MIDI parsing or analysis error
    MidiError(String),

    /// Generic application error
    GeneralError(String),
}

// =============================================================================
// DISPLAY TRAIT - Pure transformation to string representation
// =============================================================================

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::DatabaseError(e) => write!(f, "Database error: {}", e),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::IOError(e) => write!(f, "I/O error: {}", e),
            AppError::MidiError(msg) => write!(f, "MIDI error: {}", msg),
            AppError::GeneralError(msg) => write!(f, "Error: {}", msg),
        }
    }
}

// =============================================================================
// ERROR TRAIT - Standard error interface
// =============================================================================

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::DatabaseError(e) => Some(e),
            AppError::IOError(e) => Some(e),
            _ => None,
        }
    }
}

// =============================================================================
// FROM TRAIT IMPLEMENTATIONS - Pure type conversions
// =============================================================================

/// Convert from sqlx::Error to AppError (pure transformation)
impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        AppError::DatabaseError(error)
    }
}

/// Convert from std::io::Error to AppError (pure transformation)
impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::IOError(error)
    }
}

/// Convert from String to AppError (pure transformation)
impl From<String> for AppError {
    fn from(error: String) -> Self {
        AppError::GeneralError(error)
    }
}

/// Convert from &str to AppError (pure transformation)
impl From<&str> for AppError {
    fn from(error: &str) -> Self {
        AppError::GeneralError(error.to_string())
    }
}

// =============================================================================
// TAURI CONVERSION - Pure transformation to String for frontend
// =============================================================================

/// Convert AppError to String for Tauri command return types
///
/// This is a pure transformation with no side effects.
/// Tauri requires errors to be String for IPC serialization.
///
/// # Examples
///
/// ```rust
/// #[tauri::command]
/// pub async fn my_command() -> Result<Data, String> {
///     let result = database_operation()
///         .await
///         .map_err(|e| AppError::from(e).into())?;
///     Ok(result)
/// }
/// ```
impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        error.to_string()
    }
}

// =============================================================================
// HELPER FUNCTIONS - Pure error creation utilities
// =============================================================================

impl AppError {
    /// Create a NotFound error (pure function)
    ///
    /// # Examples
    ///
    /// ```rust
    /// return Err(AppError::not_found("File with ID 123"));
    /// ```
    pub fn not_found(resource: &str) -> Self {
        AppError::NotFound(format!("{} not found", resource))
    }

    /// Create a ValidationError (pure function)
    ///
    /// # Examples
    ///
    /// ```rust
    /// return Err(AppError::validation("BPM must be between 20 and 300"));
    /// ```
    pub fn validation(message: &str) -> Self {
        AppError::ValidationError(message.to_string())
    }

    /// Create a MidiError (pure function)
    ///
    /// # Examples
    ///
    /// ```rust
    /// return Err(AppError::midi("Invalid MIDI header"));
    /// ```
    pub fn midi(message: &str) -> Self {
        AppError::MidiError(message.to_string())
    }

    /// Create a GeneralError (pure function)
    ///
    /// # Examples
    ///
    /// ```rust
    /// return Err(AppError::general("Something went wrong"));
    /// ```
    pub fn general(message: &str) -> Self {
        AppError::GeneralError(message.to_string())
    }
}

// =============================================================================
// RESULT TYPE ALIAS - Convenience type for commands
// =============================================================================

/// Standard Result type for Tauri commands
///
/// Uses String as error type for Tauri IPC compatibility.
///
/// # Examples
///
/// ```rust
/// #[tauri::command]
/// pub async fn get_file(id: i64) -> AppResult<File> {
///     let file = database.get_file(id)
///         .await
///         .map_err(AppError::from)?;
///
///     file.ok_or_else(|| AppError::not_found(&format!("File {}", id)))
/// }
/// ```
pub type AppResult<T> = Result<T, AppError>;

/// Tauri-compatible Result type (with String error)
///
/// For use in Tauri command return types.
///
/// # Examples
///
/// ```rust
/// #[tauri::command]
/// pub async fn search_files() -> TauriResult<Vec<File>> {
///     let files = database.search()
///         .await
///         .map_err(|e| AppError::from(e).into())?;
///     Ok(files)
/// }
/// ```
pub type TauriResult<T> = Result<T, String>;

// =============================================================================
// TESTS - Pure function testing
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_database_error() {
        let err = AppError::DatabaseError(sqlx::Error::RowNotFound);
        assert!(err.to_string().contains("Database error"));
    }

    #[test]
    fn test_display_not_found() {
        let err = AppError::NotFound("File 123".to_string());
        assert_eq!(err.to_string(), "Not found: File 123");
    }

    #[test]
    fn test_display_validation() {
        let err = AppError::ValidationError("Invalid input".to_string());
        assert_eq!(err.to_string(), "Validation error: Invalid input");
    }

    #[test]
    fn test_display_midi_error() {
        let err = AppError::MidiError("Bad header".to_string());
        assert_eq!(err.to_string(), "MIDI error: Bad header");
    }

    #[test]
    fn test_from_string() {
        let err = AppError::from("Test error");
        match err {
            AppError::GeneralError(msg) => assert_eq!(msg, "Test error"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_from_sqlx_error() {
        let sqlx_err = sqlx::Error::RowNotFound;
        let app_err = AppError::from(sqlx_err);
        match app_err {
            AppError::DatabaseError(_) => (),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_to_string_conversion() {
        let err = AppError::NotFound("Resource".to_string());
        let string_err: String = err.into();
        assert_eq!(string_err, "Not found: Resource");
    }

    #[test]
    fn test_not_found_helper() {
        let err = AppError::not_found("File 123");
        assert_eq!(err.to_string(), "Not found: File 123 not found");
    }

    #[test]
    fn test_validation_helper() {
        let err = AppError::validation("BPM out of range");
        assert_eq!(err.to_string(), "Validation error: BPM out of range");
    }

    #[test]
    fn test_midi_helper() {
        let err = AppError::midi("Invalid format");
        assert_eq!(err.to_string(), "MIDI error: Invalid format");
    }

    #[test]
    fn test_general_helper() {
        let err = AppError::general("Something wrong");
        assert_eq!(err.to_string(), "Error: Something wrong");
    }

    #[test]
    fn test_deterministic_conversion() {
        // Pure function - same input always produces same output
        let err1 = AppError::validation("Test");
        let err2 = AppError::validation("Test");
        assert_eq!(err1.to_string(), err2.to_string());
    }
}

// src-tauri/src/commands/files.rs
//
// ARCHETYPE: MANAGER (Grown-up Script)
// PURPOSE: Tauri commands for file operations with database I/O
//
// ✅ CAN: Perform database I/O (queries)
// ✅ CAN: Have side effects (database reads/writes)
// ✅ CAN: Be async
// ✅ SHOULD: Handle errors using AppError
// ❌ MUST NOT: Contain complex business logic
// ❌ MUST NOT: Have UI concerns
// ❌ SHOULD: Delegate complex logic to separate modules

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use tauri::State;

use crate::AppState;

// =============================================================================
// DATA STRUCTURES
// =============================================================================

/// MIDI file record with musical metadata
///
/// Combined data from files and musical_metadata tables.
/// Used for displaying file information in the UI.
///
/// # Archetype: Trusty Module (data structure)
///
/// This is a pure data container with no behavior.
#[derive(Debug, FromRow, Serialize)]
pub struct MidiFile {
    /// Unique file ID
    pub id: i64,

    /// Display filename (e.g., "my-song.mid")
    pub filename: String,

    /// Full path to file (e.g., "/library/bass/my-song.mid")
    pub filepath: String,

    /// Original filename before processing
    #[serde(rename = "originalFilename")]
    pub original_filename: String,

    /// Primary category (e.g., "BASS", "LEAD")
    pub category: String,

    /// Parent folder name (e.g., "bass", "drums", "leads")
    #[serde(rename = "parentFolder")]
    pub parent_folder: Option<String>,

    /// File size in bytes
    #[serde(rename = "fileSize")]
    pub file_size: i64,

    /// Detected BPM (nullable)
    pub bpm: Option<f64>,

    /// Detected key signature (nullable, e.g., "C", "Am")
    #[serde(rename = "key")]
    pub key_signature: Option<String>,

    /// Duration in seconds (nullable)
    #[serde(rename = "duration")]
    pub duration_seconds: Option<f64>,

    /// Timestamp when file was added to database
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,

    /// Timestamp when file was last updated
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// TAURI COMMANDS - MANAGER ARCHETYPE
// =============================================================================

/// Test database connection
///
/// Verifies that the database is reachable and responds to queries.
///
/// # Manager Archetype
/// - ✅ Performs I/O (database query)
/// - ✅ Has side effects (network call to database)
/// - ✅ Handles errors properly (converts to String)
/// - ❌ No complex business logic
///
/// # Returns
///
/// * `Result<bool, String>` - True if connected, error message if failed
///
/// # Frontend Usage
///
/// ```typescript
/// const connected = await invoke<boolean>('test_db_connection');
/// if (connected) {
///   console.log('Database is ready');
/// }
/// ```
#[tauri::command]
pub async fn test_db_connection(state: State<'_, AppState>) -> Result<bool, String> {
    state
        .database
        .test_connection()
        .await
        .map_err(|e| format!("Database connection failed: {}", e))
}

/// Get total count of files in database (implementation for tests and reuse)
///
/// Internal implementation that accepts &AppState for testing without Tauri context.
///
/// # Arguments
/// * `state` - Application state containing database connection
///
/// # Returns
/// * `Result<i64, String>` - Total file count or error message
pub async fn get_file_count_impl(state: &AppState) -> Result<i64, String> {
    let pool = state.database.pool().await;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files")
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("Failed to get file count: {}", e))?;

    Ok(count)
}

/// Get total count of files in database
///
/// Returns the number of MIDI files currently stored.
///
/// # Manager Archetype
/// - ✅ Performs I/O (database query)
/// - ✅ Has side effects (reads from database)
/// - ✅ Handles errors properly
/// - ❌ No complex business logic
///
/// # Returns
///
/// * `Result<i64, String>` - Total file count or error message
///
/// # Frontend Usage
///
/// ```typescript
/// const count = await invoke<number>('get_file_count');
/// console.log(`Library contains ${count} files`);
/// ```
#[tauri::command]
pub async fn get_file_count(state: State<'_, AppState>) -> Result<i64, String> {
    get_file_count_impl(&*state).await
}

/// Get file details by ID (implementation for tests and reuse)
///
/// Internal implementation that accepts &AppState for testing without Tauri context.
pub async fn get_file_details_impl(
    file_id: i64,
    state: &AppState,
) -> Result<MidiFile, String> {
    let pool = state.database.pool().await;
    let file = sqlx::query_as::<_, MidiFile>(
        r#"
        SELECT
            f.id,
            f.filename,
            f.filepath,
            f.original_filename,
            COALESCE(fc.primary_category::text, 'UNKNOWN') as category,
            f.parent_folder,
            f.file_size_bytes as file_size,
            CAST(f.duration_seconds AS DOUBLE PRECISION) as duration_seconds,
            f.created_at,
            f.updated_at,
            CAST(mm.bpm AS DOUBLE PRECISION) as bpm,
            mm.key_signature::text as key_signature
        FROM files f
        LEFT JOIN musical_metadata mm ON f.id = mm.file_id
        LEFT JOIN file_categories fc ON f.id = fc.file_id
        WHERE f.id = $1
        "#,
    )
    .bind(file_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| format!("Failed to fetch file details: {}", e))?
    .ok_or_else(|| format!("File with ID {} not found", file_id))?;

    Ok(file)
}

/// Get file details by ID
///
/// Retrieves complete information for a single MIDI file.
///
/// # Manager Archetype
/// - ✅ Performs I/O (database query)
/// - ✅ Has side effects (reads from database)
/// - ✅ Handles errors properly (including NotFound)
/// - ❌ No complex business logic
///
/// # Arguments
///
/// * `file_id` - Unique file ID to retrieve
///
/// # Returns
///
/// * `Result<MidiFile, String>` - File details or error message
///
/// # Errors
///
/// Returns error if file doesn't exist or query fails.
///
/// # Frontend Usage
///
/// ```typescript
/// const file = await invoke<MidiFile>('get_file_details', { fileId: 123 });
/// console.log(`File: ${file.filename}, BPM: ${file.bpm}`);
/// ```
#[tauri::command]
pub async fn get_file_details(
    file_id: i64,
    state: State<'_, AppState>,
) -> Result<MidiFile, String> {
    get_file_details_impl(file_id, &*state).await
}

/// Get file by ID (alias for get_file_details for frontend compatibility)
///
/// # Frontend Usage
///
/// ```typescript
/// const file = await invoke<MidiFile>('get_file', { fileId: 123 });
/// ```
#[tauri::command]
pub async fn get_file(
    file_id: i64,
    state: State<'_, AppState>,
) -> Result<MidiFile, String> {
    get_file_details(file_id, state).await
}

/// List files with pagination (implementation for tests and reuse)
///
/// Internal implementation that accepts &AppState for testing without Tauri context.
pub async fn list_files_impl(
    limit: Option<i64>,
    offset: Option<i64>,
    state: &AppState,
) -> Result<Vec<MidiFile>, String> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    let pool = state.database.pool().await;
    let files = sqlx::query_as::<_, MidiFile>(
        r#"
        SELECT
            f.id,
            f.filename,
            f.filepath,
            f.original_filename,
            COALESCE(fc.primary_category::text, 'UNKNOWN') as category,
            f.parent_folder,
            f.file_size_bytes as file_size,
            CAST(f.duration_seconds AS DOUBLE PRECISION) as duration_seconds,
            f.created_at,
            f.updated_at,
            CAST(mm.bpm AS DOUBLE PRECISION) as bpm,
            mm.key_signature::text as key_signature
        FROM files f
        LEFT JOIN musical_metadata mm ON f.id = mm.file_id
        LEFT JOIN file_categories fc ON f.id = fc.file_id
        ORDER BY f.created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to list files: {}", e))?;

    // Debug logging
    tracing::info!(
        "list_files: Returning {} files, first file parent_folder: {:?}",
        files.len(),
        files.first().map(|f| &f.parent_folder)
    );

    Ok(files)
}

/// List files with pagination
///
/// Returns a paginated list of files ordered by creation date (newest first).
///
/// # Manager Archetype
/// - ✅ Performs I/O (database query)
/// - ✅ Has side effects (reads from database)
/// - ✅ Handles errors properly
///
/// # Arguments
///
/// * `limit` - Maximum number of files to return (default: 50)
/// * `offset` - Number of files to skip (default: 0)
///
/// # Frontend Usage
///
/// ```typescript
/// const files = await invoke<MidiFile[]>('list_files', { limit: 50, offset: 0 });
/// ```
#[tauri::command]
pub async fn list_files(
    limit: Option<i64>,
    offset: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<MidiFile>, String> {
    list_files_impl(limit, offset, &*state).await
}

/// Get files by category
///
/// Returns all files in a specific category.
///
/// # Arguments
///
/// * `category` - Category name (e.g., "bass", "drums", "melody")
/// * `limit` - Maximum number of files to return (default: 50)
///
/// # Frontend Usage
///
/// ```typescript
/// const files = await invoke<MidiFile[]>('get_files_by_category', {
///   category: 'bass',
///   limit: 50
/// });
/// ```
#[tauri::command]
pub async fn get_files_by_category(
    category: String,
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<MidiFile>, String> {
    let limit = limit.unwrap_or(50);

    let pool = state.database.pool().await;
    let files = sqlx::query_as::<_, MidiFile>(
        r#"
        SELECT
            f.id,
            f.filename,
            f.filepath,
            f.original_filename,
            COALESCE(fc.primary_category::text, 'UNKNOWN') as category,
            f.parent_folder,
            f.file_size_bytes as file_size,
            CAST(f.duration_seconds AS DOUBLE PRECISION) as duration_seconds,
            f.created_at,
            f.updated_at,
            CAST(mm.bpm AS DOUBLE PRECISION) as bpm,
            mm.key_signature::text as key_signature
        FROM files f
        LEFT JOIN musical_metadata mm ON f.id = mm.file_id
        LEFT JOIN file_categories fc ON f.id = fc.file_id
        WHERE fc.primary_category::text = $1
        ORDER BY f.created_at DESC
        LIMIT $2
        "#,
    )
    .bind(category)
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get files by category: {}", e))?;

    Ok(files)
}

/// Get recently added files
///
/// Returns the most recently imported files.
///
/// # Arguments
///
/// * `limit` - Maximum number of files to return (default: 10)
///
/// # Frontend Usage
///
/// ```typescript
/// const files = await invoke<MidiFile[]>('get_recent_files', { limit: 10 });
/// ```
#[tauri::command]
pub async fn get_recent_files(
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<MidiFile>, String> {
    let limit = limit.unwrap_or(10);

    let pool = state.database.pool().await;
    let files = sqlx::query_as::<_, MidiFile>(
        r#"
        SELECT
            f.id,
            f.filename,
            f.filepath,
            f.original_filename,
            COALESCE(fc.primary_category::text, 'UNKNOWN') as category,
            f.parent_folder,
            f.file_size_bytes as file_size,
            CAST(f.duration_seconds AS DOUBLE PRECISION) as duration_seconds,
            f.created_at,
            f.updated_at,
            CAST(mm.bpm AS DOUBLE PRECISION) as bpm,
            mm.key_signature::text as key_signature
        FROM files f
        LEFT JOIN musical_metadata mm ON f.id = mm.file_id
        LEFT JOIN file_categories fc ON f.id = fc.file_id
        ORDER BY f.created_at DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get recent files: {}", e))?;

    Ok(files)
}

/// Delete a file
///
/// Removes a file from the database (cascading deletes related records).
///
/// # Arguments
///
/// * `file_id` - ID of the file to delete
///
/// # Frontend Usage
///
/// ```typescript
/// await invoke('delete_file', { fileId: 123 });
/// ```
#[tauri::command]
pub async fn delete_file(
    file_id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.database.pool().await;
    sqlx::query("DELETE FROM files WHERE id = $1")
        .bind(file_id)
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to delete file: {}", e))?;

    Ok(())
}

/// Update file tags
///
// update_file_tags moved to commands/tags.rs to use TagRepository

// =============================================================================
// TESTS - MANAGER ARCHETYPE TESTING
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that MidiFile struct has all required fields
    #[test]
    fn test_midi_file_struct() {
        let now = Utc::now();
        let file = MidiFile {
            id: 1,
            filename: "test.mid".to_string(),
            filepath: "/path/to/test.mid".to_string(),
            original_filename: "original_test.mid".to_string(),
            category: "DRUMS".to_string(),
            parent_folder: Some("drums".to_string()),
            file_size: 1024,
            bpm: Some(120.0),
            key_signature: Some("Cm".to_string()),
            duration_seconds: Some(180.0),
            created_at: now,
            updated_at: now,
        };

        assert_eq!(file.id, 1);
        assert_eq!(file.filename, "test.mid");
        assert_eq!(file.bpm, Some(120.0));
        assert_eq!(file.key_signature, Some("Cm".to_string()));
    }

    // NOTE: Advanced search functionality is in commands/search.rs

    // Integration tests require database connection and Tauri runtime
    // These tests should be run as part of E2E testing, not unit tests
    // For manual testing:
    // 1. Start database: docker-compose up -d
    // 2. Run the Tauri app and test commands from the frontend
}

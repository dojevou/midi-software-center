//! File Commands - Database queries for MIDI file records and metadata

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use tauri::State;

use crate::AppState;

/// MIDI file record with musical metadata from files and musical_metadata tables.
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

/// Test database connection.
#[tauri::command]
pub async fn test_db_connection(state: State<'_, AppState>) -> Result<bool, String> {
    state
        .database
        .test_connection()
        .await
        .map_err(|e| format!("Database connection failed: {}", e))
}

/// Get total count of files (internal implementation for tests).
pub async fn get_file_count_impl(state: &AppState) -> Result<i64, String> {
    let pool = state.database.pool().await;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files")
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("Failed to get file count: {}", e))?;

    Ok(count)
}

/// Get total count of files in database.
#[tauri::command]
pub async fn get_file_count(state: State<'_, AppState>) -> Result<i64, String> {
    get_file_count_impl(&state).await
}

/// Get file details by ID (internal implementation for tests).
pub async fn get_file_details_impl(file_id: i64, state: &AppState) -> Result<MidiFile, String> {
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

/// Get file details by ID.
#[tauri::command]
pub async fn get_file_details(
    file_id: i64,
    state: State<'_, AppState>,
) -> Result<MidiFile, String> {
    get_file_details_impl(file_id, &state).await
}

/// Get file by ID (alias for get_file_details).
#[tauri::command]
pub async fn get_file(file_id: i64, state: State<'_, AppState>) -> Result<MidiFile, String> {
    get_file_details(file_id, state).await
}

/// List files with pagination (internal implementation for tests).
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

/// List files with pagination, ordered by creation date (newest first).
#[tauri::command]
pub async fn list_files(
    limit: Option<i64>,
    offset: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<MidiFile>, String> {
    list_files_impl(limit, offset, &state).await
}

/// Get files by category.
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

/// Get recently added files.
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

/// Delete a file from the database (cascading deletes related records).
#[tauri::command]
pub async fn delete_file(file_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let pool = state.database.pool().await;
    sqlx::query("DELETE FROM files WHERE id = $1")
        .bind(file_id)
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to delete file: {}", e))?;

    Ok(())
}

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

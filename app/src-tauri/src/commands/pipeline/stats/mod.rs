//! Stats Commands - Database statistics and metrics queries

mod queries;

use crate::AppState;
use std::collections::HashMap;
use tauri::State;

// Re-export query implementations for tests and direct use
pub use queries::{
    get_category_stats as get_category_stats_impl, get_database_size as get_database_size_impl,
};

/// Get file count breakdown by category.
#[tauri::command]
pub async fn get_category_stats(
    state: State<'_, AppState>,
) -> Result<HashMap<String, i64>, String> {
    queries::get_category_stats(&state).await
}

/// Get file count breakdown by manufacturer.
#[tauri::command]
pub async fn get_manufacturer_stats(
    state: State<'_, AppState>,
) -> Result<HashMap<String, i64>, String> {
    queries::get_manufacturer_stats(&state).await
}

/// Get file count breakdown by key signature.
#[tauri::command]
pub async fn get_key_signature_stats(
    state: State<'_, AppState>,
) -> Result<HashMap<String, i64>, String> {
    queries::get_key_signature_stats(&state).await
}

/// Get count of recently added files (last 7 days).
#[tauri::command]
pub async fn get_recently_added_count(state: State<'_, AppState>) -> Result<i64, String> {
    queries::get_recently_added_count(&state).await
}

/// Get count of duplicate files (same content hash).
#[tauri::command]
pub async fn get_duplicate_count(state: State<'_, AppState>) -> Result<i64, String> {
    queries::get_duplicate_count(&state).await
}

/// Get database size as human-readable string (e.g., "125.4 MB").
#[tauri::command]
pub async fn get_database_size(state: State<'_, AppState>) -> Result<String, String> {
    queries::get_database_size(&state).await
}

/// Check database health status ("good", "warning", or "error").
#[tauri::command]
pub async fn check_database_health(state: State<'_, AppState>) -> Result<String, String> {
    queries::check_database_health(&state).await
}

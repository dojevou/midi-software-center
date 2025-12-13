//! VIP3 favorites commands

use super::search::search_files_vip3;
use super::types::{Vip3Filters, Vip3SearchResults, Vip3Sort};
use crate::AppState;
use tauri::State;

/// Toggle favorite status for a file.
#[tauri::command]
pub async fn toggle_favorite(file_id: i64, state: State<'_, AppState>) -> Result<bool, String> {
    let pool = state.database.pool().await;

    // Toggle and return new value
    let result: (bool,) = sqlx::query_as(
        r#"
        UPDATE files
        SET favorite = NOT COALESCE(favorite, FALSE),
            updated_at = NOW()
        WHERE id = $1
        RETURNING favorite
        "#,
    )
    .bind(file_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("Failed to toggle favorite: {}", e))?;

    Ok(result.0)
}

/// Set favorite status for a file.
#[tauri::command]
pub async fn set_favorite(
    file_id: i64,
    favorite: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.database.pool().await;

    sqlx::query("UPDATE files SET favorite = $1, updated_at = NOW() WHERE id = $2")
        .bind(favorite)
        .bind(file_id)
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to set favorite: {}", e))?;

    Ok(())
}

/// Get all favorite files.
#[tauri::command]
pub async fn get_favorites(
    page: Option<i32>,
    page_size: Option<i32>,
    state: State<'_, AppState>,
) -> Result<Vip3SearchResults, String> {
    let filters = Vip3Filters { favorites_only: true, ..Default::default() };

    search_files_vip3(filters, Some(Vip3Sort::default()), page, page_size, state).await
}

/// Get favorite count.
#[tauri::command]
pub async fn get_favorite_count(state: State<'_, AppState>) -> Result<i64, String> {
    let pool = state.database.pool().await;

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM files WHERE favorite = TRUE")
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("Failed to get favorite count: {}", e))?;

    Ok(count.0)
}

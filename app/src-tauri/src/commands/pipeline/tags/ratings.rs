//! File rating commands (1-5 stars)

use crate::AppState;
use tauri::State;

/// Set the rating for a file (1-5 stars, or None to clear).
#[tauri::command]
pub async fn set_file_rating(
    file_id: i64,
    rating: Option<i16>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if let Some(r) = rating {
        if !(1..=5).contains(&r) {
            return Err("Rating must be between 1 and 5".to_string());
        }
    }

    let pool = state.database.pool().await;

    sqlx::query("UPDATE files SET rating = $1, updated_at = NOW() WHERE id = $2")
        .bind(rating)
        .bind(file_id)
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to set file rating: {}", e))?;

    Ok(())
}

/// Get the rating for a file (1-5 or None if unrated).
#[tauri::command]
pub async fn get_file_rating(
    file_id: i64,
    state: State<'_, AppState>,
) -> Result<Option<i16>, String> {
    let pool = state.database.pool().await;

    let result: Option<(Option<i16>,)> = sqlx::query_as("SELECT rating FROM files WHERE id = $1")
        .bind(file_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| format!("Failed to get file rating: {}", e))?;

    Ok(result.and_then(|(r,)| r))
}

/// Get files by rating (1-5) with pagination.
#[tauri::command]
pub async fn get_files_by_rating(
    rating: i16,
    limit: Option<i32>,
    offset: Option<i32>,
    state: State<'_, AppState>,
) -> Result<Vec<i64>, String> {
    if !(1..=5).contains(&rating) {
        return Err("Rating must be between 1 and 5".to_string());
    }

    let pool = state.database.pool().await;
    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);

    let file_ids: Vec<(i64,)> = sqlx::query_as(
        "SELECT id FROM files WHERE rating = $1 ORDER BY updated_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(rating)
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get files by rating: {}", e))?;

    Ok(file_ids.into_iter().map(|(id,)| id).collect())
}

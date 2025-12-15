//! VIP3 saved searches commands

use super::types::{CreateSavedSearchRequest, SavedSearchResponse};
use crate::AppState;
use tauri::State;

/// Save a search configuration.
#[tauri::command]
pub async fn save_search(
    request: CreateSavedSearchRequest,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    let pool = state.database.pool().await;
    let filters_json = serde_json::to_value(&request.filters)
        .map_err(|e| format!("Failed to serialize filters: {}", e))?;

    let result: (i64,) = sqlx::query_as(
        r#"
        INSERT INTO saved_searches (name, description, filters, sort_by, sort_order, icon, color)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
    )
    .bind(&request.name)
    .bind(&request.description)
    .bind(&filters_json)
    .bind(&request.sort_by)
    .bind(&request.sort_order)
    .bind(&request.icon)
    .bind(&request.color)
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("Failed to save search: {}", e))?;

    Ok(result.0)
}

/// Get all saved searches
#[tauri::command]
pub async fn get_saved_searches(
    state: State<'_, AppState>,
) -> Result<Vec<SavedSearchResponse>, String> {
    let pool = state.database.pool().await;

    let searches: Vec<SavedSearchResponse> = sqlx::query_as(
        r#"
        SELECT id, name, description, filters, sort_by, sort_order, icon, color,
               is_pinned, created_at, last_used, use_count
        FROM saved_searches
        ORDER BY is_pinned DESC, last_used DESC NULLS LAST, created_at DESC
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get saved searches: {}", e))?;

    Ok(searches)
}

/// Load a saved search (increments use count)
#[tauri::command]
pub async fn load_saved_search(
    search_id: i64,
    state: State<'_, AppState>,
) -> Result<SavedSearchResponse, String> {
    let pool = state.database.pool().await;

    // Update use count and last used timestamp
    let search: SavedSearchResponse = sqlx::query_as(
        r#"
        UPDATE saved_searches
        SET use_count = COALESCE(use_count, 0) + 1, last_used = NOW()
        WHERE id = $1
        RETURNING id, name, description, filters, sort_by, sort_order, icon, color,
                  is_pinned, created_at, last_used, use_count
        "#,
    )
    .bind(search_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("Failed to load saved search: {}", e))?;

    Ok(search)
}

/// Delete a saved search
#[tauri::command]
pub async fn delete_saved_search(search_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let pool = state.database.pool().await;

    sqlx::query("DELETE FROM saved_searches WHERE id = $1")
        .bind(search_id)
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to delete saved search: {}", e))?;

    Ok(())
}

/// Toggle pinned status for a saved search
#[tauri::command]
pub async fn toggle_saved_search_pin(
    search_id: i64,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let pool = state.database.pool().await;

    let result: (bool,) = sqlx::query_as(
        r#"
        UPDATE saved_searches
        SET is_pinned = NOT COALESCE(is_pinned, FALSE)
        WHERE id = $1
        RETURNING is_pinned
        "#,
    )
    .bind(search_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("Failed to toggle pin: {}", e))?;

    Ok(result.0)
}

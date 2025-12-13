//! VIP3 lookup commands (get all filter options)

use super::types::FilterOption;
use crate::AppState;
use tauri::State;

/// Get all timbres (for dropdown/multiselect).
#[tauri::command]
pub async fn get_all_timbres(state: State<'_, AppState>) -> Result<Vec<FilterOption>, String> {
    let pool = state.database.pool().await;

    let timbres: Vec<FilterOption> = sqlx::query_as(
        r#"
        SELECT id::integer, name, file_count::bigint
        FROM timbres
        ORDER BY sort_order, name
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get timbres: {}", e))?;

    Ok(timbres)
}

/// Get all styles (for dropdown/multiselect)
#[tauri::command]
pub async fn get_all_styles(state: State<'_, AppState>) -> Result<Vec<FilterOption>, String> {
    let pool = state.database.pool().await;

    let styles: Vec<FilterOption> = sqlx::query_as(
        r#"
        SELECT id::integer, name, file_count::bigint
        FROM styles
        ORDER BY sort_order, name
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get styles: {}", e))?;

    Ok(styles)
}

/// Get all articulations (for dropdown/multiselect)
#[tauri::command]
pub async fn get_all_articulations(
    state: State<'_, AppState>,
) -> Result<Vec<FilterOption>, String> {
    let pool = state.database.pool().await;

    let articulations: Vec<FilterOption> = sqlx::query_as(
        r#"
        SELECT id::integer, name, file_count::bigint
        FROM articulations
        ORDER BY sort_order, name
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get articulations: {}", e))?;

    Ok(articulations)
}

/// Get all BPM ranges (for dropdown)
#[tauri::command]
pub async fn get_all_bpm_ranges(state: State<'_, AppState>) -> Result<Vec<FilterOption>, String> {
    let pool = state.database.pool().await;

    let bpm_ranges: Vec<FilterOption> = sqlx::query_as(
        r#"
        SELECT id::integer, label as name, file_count::bigint
        FROM bpm_ranges
        ORDER BY sort_order
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get BPM ranges: {}", e))?;

    Ok(bpm_ranges)
}

/// Get all musical keys (for dropdown)
#[tauri::command]
pub async fn get_all_musical_keys(state: State<'_, AppState>) -> Result<Vec<FilterOption>, String> {
    let pool = state.database.pool().await;

    let keys: Vec<FilterOption> = sqlx::query_as(
        r#"
        SELECT id::integer, name, file_count::bigint
        FROM musical_keys
        ORDER BY sort_order
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get musical keys: {}", e))?;

    Ok(keys)
}

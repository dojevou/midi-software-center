//! VIP3 file category commands (timbres, styles, articulations)

use super::types::{FileCategoriesResponse, FilterOption};
use crate::AppState;
use tauri::State;

/// Add a timbre to a file.
#[tauri::command]
pub async fn add_timbre_to_file(
    file_id: i64,
    timbre_id: i16,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.database.pool().await;

    sqlx::query(
        r#"
        INSERT INTO midi_file_timbres (file_id, timbre_id)
        VALUES ($1, $2)
        ON CONFLICT (file_id, timbre_id) DO NOTHING
        "#,
    )
    .bind(file_id)
    .bind(timbre_id)
    .execute(&pool)
    .await
    .map_err(|e| format!("Failed to add timbre: {}", e))?;

    Ok(())
}

/// Remove a timbre from a file
#[tauri::command]
pub async fn remove_timbre_from_file(
    file_id: i64,
    timbre_id: i16,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.database.pool().await;

    sqlx::query("DELETE FROM midi_file_timbres WHERE file_id = $1 AND timbre_id = $2")
        .bind(file_id)
        .bind(timbre_id)
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to remove timbre: {}", e))?;

    Ok(())
}

/// Add a style to a file
#[tauri::command]
pub async fn add_style_to_file(
    file_id: i64,
    style_id: i16,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.database.pool().await;

    sqlx::query(
        r#"
        INSERT INTO midi_file_styles (file_id, style_id)
        VALUES ($1, $2)
        ON CONFLICT (file_id, style_id) DO NOTHING
        "#,
    )
    .bind(file_id)
    .bind(style_id)
    .execute(&pool)
    .await
    .map_err(|e| format!("Failed to add style: {}", e))?;

    Ok(())
}

/// Remove a style from a file
#[tauri::command]
pub async fn remove_style_from_file(
    file_id: i64,
    style_id: i16,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.database.pool().await;

    sqlx::query("DELETE FROM midi_file_styles WHERE file_id = $1 AND style_id = $2")
        .bind(file_id)
        .bind(style_id)
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to remove style: {}", e))?;

    Ok(())
}

/// Add an articulation to a file
#[tauri::command]
pub async fn add_articulation_to_file(
    file_id: i64,
    articulation_id: i16,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.database.pool().await;

    sqlx::query(
        r#"
        INSERT INTO midi_file_articulations (file_id, articulation_id)
        VALUES ($1, $2)
        ON CONFLICT (file_id, articulation_id) DO NOTHING
        "#,
    )
    .bind(file_id)
    .bind(articulation_id)
    .execute(&pool)
    .await
    .map_err(|e| format!("Failed to add articulation: {}", e))?;

    Ok(())
}

/// Remove an articulation from a file
#[tauri::command]
pub async fn remove_articulation_from_file(
    file_id: i64,
    articulation_id: i16,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.database.pool().await;

    sqlx::query("DELETE FROM midi_file_articulations WHERE file_id = $1 AND articulation_id = $2")
        .bind(file_id)
        .bind(articulation_id)
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to remove articulation: {}", e))?;

    Ok(())
}

/// Get file's category assignments (timbres, styles, articulations)
#[tauri::command]
pub async fn get_file_categories(
    file_id: i64,
    state: State<'_, AppState>,
) -> Result<FileCategoriesResponse, String> {
    let pool = state.database.pool().await;

    let timbres: Vec<FilterOption> = sqlx::query_as(
        r#"
        SELECT t.id::integer, t.name, 0::bigint as file_count
        FROM timbres t
        INNER JOIN midi_file_timbres mft ON t.id = mft.timbre_id
        WHERE mft.file_id = $1
        ORDER BY t.sort_order
        "#,
    )
    .bind(file_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get file timbres: {}", e))?;

    let styles: Vec<FilterOption> = sqlx::query_as(
        r#"
        SELECT s.id::integer, s.name, 0::bigint as file_count
        FROM styles s
        INNER JOIN midi_file_styles mfs ON s.id = mfs.style_id
        WHERE mfs.file_id = $1
        ORDER BY s.sort_order
        "#,
    )
    .bind(file_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get file styles: {}", e))?;

    let articulations: Vec<FilterOption> = sqlx::query_as(
        r#"
        SELECT a.id::integer, a.name, 0::bigint as file_count
        FROM articulations a
        INNER JOIN midi_file_articulations mfa ON a.id = mfa.articulation_id
        WHERE mfa.file_id = $1
        ORDER BY a.sort_order
        "#,
    )
    .bind(file_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get file articulations: {}", e))?;

    Ok(FileCategoriesResponse { timbres, styles, articulations })
}

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

// ============================================================================
// Category List Fetching (for VIP3 Browser initialization)
// ============================================================================

/// Get all unique folders from the database
///
/// Folders are stored in the `parent_folder` column of the files table.
#[tauri::command]
pub async fn get_vip3_folders(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let pool = state.database.pool().await;

    let folders: Vec<String> = sqlx::query_scalar!(
        "SELECT DISTINCT parent_folder
         FROM files
         WHERE parent_folder IS NOT NULL
         ORDER BY parent_folder"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to fetch folders: {}", e))?
    .into_iter()
    .flatten()
    .collect();

    Ok(folders)
}

/// Get all unique instruments from tags table
///
/// Instruments are stored as tags with category='instrument'.
#[tauri::command]
pub async fn get_vip3_instruments(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let pool = state.database.pool().await;

    let instruments = sqlx::query_scalar!(
        "SELECT DISTINCT name
         FROM tags
         WHERE category = 'instrument'
         AND is_active = true
         ORDER BY name"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to fetch instruments: {}", e))?;

    Ok(instruments)
}

/// Get all timbres
///
/// Timbres are stored in the dedicated `timbres` table.
#[tauri::command]
pub async fn get_vip3_timbres(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let pool = state.database.pool().await;

    let timbres = sqlx::query_scalar!(
        "SELECT name
         FROM timbres
         ORDER BY sort_order, name"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to fetch timbres: {}", e))?;

    Ok(timbres)
}

/// Get all styles
///
/// Styles are stored in the dedicated `styles` table.
#[tauri::command]
pub async fn get_vip3_styles(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let pool = state.database.pool().await;

    let styles = sqlx::query_scalar!(
        "SELECT name
         FROM styles
         ORDER BY sort_order, name"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to fetch styles: {}", e))?;

    Ok(styles)
}

/// Get all articulations
///
/// Articulations are stored in the dedicated `articulations` table.
#[tauri::command]
pub async fn get_vip3_articulations(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let pool = state.database.pool().await;

    let articulations = sqlx::query_scalar!(
        "SELECT name
         FROM articulations
         ORDER BY sort_order, name"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to fetch articulations: {}", e))?;

    Ok(articulations)
}

/// Get all manufacturers
///
/// Manufacturers are stored in the `manufacturer` column of the files table.
#[tauri::command]
pub async fn get_vip3_manufacturers(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let pool = state.database.pool().await;

    let manufacturers: Vec<String> = sqlx::query_scalar!(
        "SELECT DISTINCT manufacturer
         FROM files
         WHERE manufacturer IS NOT NULL
         ORDER BY manufacturer"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to fetch manufacturers: {}", e))?
    .into_iter()
    .flatten()
    .collect();

    Ok(manufacturers)
}

// ============================================================================
// Combined Categories Fetching (for efficient VIP3 Browser initialization)
// ============================================================================

use serde::{Deserialize, Serialize};

/// All VIP3 categories in a single response
#[derive(Debug, Serialize, Deserialize)]
pub struct AllVip3Categories {
    pub folders: Vec<String>,
    pub instruments: Vec<String>,
    pub timbres: Vec<String>,
    pub styles: Vec<String>,
    pub articulations: Vec<String>,
    pub manufacturers: Vec<String>,
}

/// Get all VIP3 categories in a single call (more efficient than 6 separate calls)
///
/// This command fetches folders, instruments, timbres, styles, articulations, and manufacturers
/// in parallel for optimal performance during VIP3 Browser initialization.
#[tauri::command]
pub async fn get_all_vip3_categories(state: State<'_, AppState>) -> Result<AllVip3Categories, String> {
    let pool = state.database.pool().await;

    // Fetch all categories in parallel using tokio::try_join!
    let (folders_raw, instruments, timbres, styles, articulations, manufacturers_raw) = tokio::try_join!(
        sqlx::query_scalar!(
            "SELECT DISTINCT parent_folder FROM files WHERE parent_folder IS NOT NULL ORDER BY parent_folder"
        )
        .fetch_all(&pool),
        sqlx::query_scalar!(
            "SELECT DISTINCT name FROM tags WHERE category = 'instrument' AND is_active = true ORDER BY name"
        )
        .fetch_all(&pool),
        sqlx::query_scalar!("SELECT name FROM timbres ORDER BY sort_order, name")
            .fetch_all(&pool),
        sqlx::query_scalar!("SELECT name FROM styles ORDER BY sort_order, name")
            .fetch_all(&pool),
        sqlx::query_scalar!("SELECT name FROM articulations ORDER BY sort_order, name")
            .fetch_all(&pool),
        sqlx::query_scalar!(
            "SELECT DISTINCT manufacturer FROM files WHERE manufacturer IS NOT NULL ORDER BY manufacturer"
        )
        .fetch_all(&pool),
    )
    .map_err(|e| format!("Failed to fetch categories: {}", e))?;

    // Filter out None values from folders and manufacturers
    let folders: Vec<String> = folders_raw.into_iter().flatten().collect();
    let manufacturers: Vec<String> = manufacturers_raw.into_iter().flatten().collect();

    Ok(AllVip3Categories {
        folders,
        instruments,
        timbres,
        styles,
        articulations,
        manufacturers,
    })
}

/// Get VIP3 filter counts using DuckDB for fast aggregations (10-50ms vs 200-680ms)
///
/// Returns counts for all filter types: folders, BPM ranges, keys, instruments,
/// timbres, styles, articulations, channel counts, and multi-track files.
///
/// This uses DuckDB to query PostgreSQL directly, providing 10-20x faster
/// aggregation performance for GROUP BY queries.
#[tauri::command]
pub async fn get_vip3_analytics_counts(
    state: State<'_, AppState>,
) -> Result<crate::services::FilterCounts, String> {
    state
        .vip3_analytics
        .get_filter_counts()
        .await
        .map_err(|e| format!("Failed to get filter counts: {}", e))
}

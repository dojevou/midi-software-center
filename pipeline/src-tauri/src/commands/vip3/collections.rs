//! VIP3 collections commands

use super::search::{DEFAULT_PAGE, DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};
use super::types::{
    CollectionResponse, CreateCollectionRequest, Vip3FileResult, Vip3SearchResults,
};
use crate::AppState;
use tauri::State;

/// Create a new collection.
#[tauri::command]
pub async fn create_collection(
    request: CreateCollectionRequest,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    let pool = state.database.pool().await;

    let result: (i64,) = sqlx::query_as(
        r#"
        INSERT INTO collections (name, description, icon, color)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(&request.name)
    .bind(&request.description)
    .bind(&request.icon)
    .bind(&request.color)
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("Failed to create collection: {}", e))?;

    Ok(result.0)
}

/// Get all collections
#[tauri::command]
pub async fn get_collections(
    state: State<'_, AppState>,
) -> Result<Vec<CollectionResponse>, String> {
    let pool = state.database.pool().await;

    let collections: Vec<CollectionResponse> = sqlx::query_as(
        r#"
        SELECT id, name, description, icon, color, is_smart, file_count, created_at, modified_at
        FROM collections
        ORDER BY name
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get collections: {}", e))?;

    Ok(collections)
}

/// Get a single collection by ID
#[tauri::command]
pub async fn get_collection(
    collection_id: i64,
    state: State<'_, AppState>,
) -> Result<CollectionResponse, String> {
    let pool = state.database.pool().await;

    let collection: CollectionResponse = sqlx::query_as(
        r#"
        SELECT id, name, description, icon, color, is_smart, file_count, created_at, modified_at
        FROM collections
        WHERE id = $1
        "#,
    )
    .bind(collection_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("Failed to get collection: {}", e))?;

    Ok(collection)
}

/// Add a file to a collection
#[tauri::command]
pub async fn add_file_to_collection(
    collection_id: i64,
    file_id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.database.pool().await;

    // Get next sort order
    let max_order: Option<(Option<i32>,)> =
        sqlx::query_as("SELECT MAX(sort_order) FROM collection_files WHERE collection_id = $1")
            .bind(collection_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| format!("Failed to get max sort order: {}", e))?;

    let next_order = max_order.and_then(|(o,)| o).unwrap_or(0) + 1;

    sqlx::query(
        r#"
        INSERT INTO collection_files (collection_id, file_id, sort_order)
        VALUES ($1, $2, $3)
        ON CONFLICT (collection_id, file_id) DO NOTHING
        "#,
    )
    .bind(collection_id)
    .bind(file_id)
    .bind(next_order)
    .execute(&pool)
    .await
    .map_err(|e| format!("Failed to add file to collection: {}", e))?;

    Ok(())
}

/// Remove a file from a collection
#[tauri::command]
pub async fn remove_file_from_collection(
    collection_id: i64,
    file_id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.database.pool().await;

    sqlx::query("DELETE FROM collection_files WHERE collection_id = $1 AND file_id = $2")
        .bind(collection_id)
        .bind(file_id)
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to remove file from collection: {}", e))?;

    Ok(())
}

/// Get files in a collection
#[tauri::command]
pub async fn get_collection_files(
    collection_id: i64,
    page: Option<i32>,
    page_size: Option<i32>,
    state: State<'_, AppState>,
) -> Result<Vip3SearchResults, String> {
    let pool = state.database.pool().await;
    let page = page.unwrap_or(DEFAULT_PAGE).max(1);
    let page_size = page_size.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE);
    let offset = (page - 1) * page_size;

    // Get total count
    let total_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM collection_files WHERE collection_id = $1")
            .bind(collection_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| format!("Failed to count collection files: {}", e))?;

    // Get files
    let files: Vec<Vip3FileResult> = sqlx::query_as(
        r#"
        SELECT
            f.id,
            f.filename,
            f.filepath,
            f.bpm::float8 as bpm,
            f.key_signature::text as key_signature,
            f.duration_ms,
            f.note_count,
            f.channel,
            f.rating,
            f.favorite,
            f.created_at
        FROM files f
        INNER JOIN collection_files cf ON f.id = cf.file_id
        WHERE cf.collection_id = $1
        ORDER BY cf.sort_order, f.filename
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(collection_id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get collection files: {}", e))?;

    let total_pages = ((total_count.0 as f64) / (page_size as f64)).ceil() as i32;

    Ok(Vip3SearchResults { files, total_count: total_count.0, page, page_size, total_pages })
}

/// Delete a collection
#[tauri::command]
pub async fn delete_collection(
    collection_id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.database.pool().await;

    sqlx::query("DELETE FROM collections WHERE id = $1")
        .bind(collection_id)
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to delete collection: {}", e))?;

    Ok(())
}

/// Update collection details
#[tauri::command]
pub async fn update_collection(
    collection_id: i64,
    name: Option<String>,
    description: Option<String>,
    icon: Option<String>,
    color: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.database.pool().await;

    sqlx::query(
        r#"
        UPDATE collections SET
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            icon = COALESCE($4, icon),
            color = COALESCE($5, color),
            modified_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(collection_id)
    .bind(name)
    .bind(description)
    .bind(icon)
    .bind(color)
    .execute(&pool)
    .await
    .map_err(|e| format!("Failed to update collection: {}", e))?;

    Ok(())
}

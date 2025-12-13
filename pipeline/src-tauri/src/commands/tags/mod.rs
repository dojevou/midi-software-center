//! Tag Commands - Tauri commands for tag CRUD operations and file ratings

pub mod ratings;
mod types;

pub use ratings::{get_file_rating, get_files_by_rating, set_file_rating};
pub use types::TagResponse;

use crate::db::repositories::tag_repository::TagRepository;
use crate::AppState;
use tauri::State;

/// Get all tags for a specific file (implementation for tests and reuse)
pub async fn get_file_tags_impl(
    file_id: i64,
    state: &AppState,
) -> Result<Vec<TagResponse>, String> {
    let pool = state.database.pool().await;
    let repo = TagRepository::new(pool);

    let tags = repo
        .get_file_tags(file_id)
        .await
        .map_err(|e| format!("Failed to get file tags: {}", e))?;

    Ok(tags.into_iter().map(TagResponse::from).collect())
}

/// Get all tags for a specific file
#[tauri::command]
pub async fn get_file_tags(
    file_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<TagResponse>, String> {
    get_file_tags_impl(file_id, &state).await
}

/// Get popular tags with usage counts (implementation for tests and reuse)
pub async fn get_popular_tags_impl(
    limit: Option<i32>,
    state: &AppState,
) -> Result<Vec<TagResponse>, String> {
    let pool = state.database.pool().await;
    let repo = TagRepository::new(pool);

    let limit = limit.unwrap_or(50);

    let tags = repo
        .get_popular_tags(limit)
        .await
        .map_err(|e| format!("Failed to get popular tags: {}", e))?;

    Ok(tags.into_iter().map(TagResponse::from).collect())
}

/// Get popular tags with usage counts (for tag cloud).
#[tauri::command]
pub async fn get_popular_tags(
    limit: Option<i32>,
    state: State<'_, AppState>,
) -> Result<Vec<TagResponse>, String> {
    get_popular_tags_impl(limit, &state).await
}

/// Search tags by name prefix (implementation for tests and reuse)
pub async fn search_tags_impl(
    query: String,
    limit: Option<i32>,
    state: &AppState,
) -> Result<Vec<TagResponse>, String> {
    let pool = state.database.pool().await;
    let repo = TagRepository::new(pool);

    let limit = limit.unwrap_or(10);

    let tags = repo
        .search_tags(&query, limit)
        .await
        .map_err(|e| format!("Failed to search tags: {}", e))?;

    Ok(tags.into_iter().map(TagResponse::from).collect())
}

/// Search tags by name prefix (for autocomplete).
#[tauri::command]
pub async fn search_tags(
    query: String,
    limit: Option<i32>,
    state: State<'_, AppState>,
) -> Result<Vec<TagResponse>, String> {
    search_tags_impl(query, limit, &state).await
}

/// Get all unique tag categories
#[tauri::command]
pub async fn get_tag_categories(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let pool = state.database.pool().await;
    let repo = TagRepository::new(pool);

    repo.get_tag_categories()
        .await
        .map_err(|e| format!("Failed to get tag categories: {}", e))
}

/// Get tags by category
#[tauri::command]
pub async fn get_tags_by_category(
    category: String,
    state: State<'_, AppState>,
) -> Result<Vec<TagResponse>, String> {
    let pool = state.database.pool().await;
    let repo = TagRepository::new(pool);

    let tags = repo
        .get_tags_by_category(&category)
        .await
        .map_err(|e| format!("Failed to get tags by category: {}", e))?;

    Ok(tags.into_iter().map(TagResponse::from).collect())
}

/// Update tags for a file (replace all existing tags).
#[tauri::command]
pub async fn update_file_tags(
    file_id: i64,
    tag_names: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.database.pool().await;
    let repo = TagRepository::new(pool);

    let tag_data: Vec<(String, Option<String>)> = tag_names
        .into_iter()
        .map(|name| (name, None))
        .collect();

    let tag_ids = repo
        .get_or_create_tags_batch(&tag_data)
        .await
        .map_err(|e| format!("Failed to create tags: {}", e))?;

    repo.update_file_tags(file_id, &tag_ids)
        .await
        .map_err(|e| format!("Failed to update file tags: {}", e))?;

    Ok(())
}

/// Add tags to a file (implementation for tests and reuse)
pub async fn add_tags_to_file_impl(
    file_id: i64,
    tag_names: Vec<String>,
    state: &AppState,
) -> Result<(), String> {
    let pool = state.database.pool().await;
    let repo = TagRepository::new(pool);

    let tag_data: Vec<(String, Option<String>)> =
        tag_names.into_iter().map(|name| (name, None)).collect();

    let tag_ids = repo
        .get_or_create_tags_batch(&tag_data)
        .await
        .map_err(|e| format!("Failed to create tags: {}", e))?;

    repo.add_tags_to_file(file_id, &tag_ids)
        .await
        .map_err(|e| format!("Failed to add tags to file: {}", e))?;

    Ok(())
}

/// Add tags to a file (without removing existing tags)
#[tauri::command]
pub async fn add_tags_to_file(
    file_id: i64,
    tag_names: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    add_tags_to_file_impl(file_id, tag_names, &state).await
}

/// Remove a specific tag from a file
#[tauri::command]
pub async fn remove_tag_from_file(
    file_id: i64,
    tag_id: i32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.database.pool().await;
    let repo = TagRepository::new(pool);

    repo.remove_tag_from_file(file_id, tag_id)
        .await
        .map_err(|e| format!("Failed to remove tag from file: {}", e))?;

    Ok(())
}

/// Get files by tags (match_all=true for AND, false for OR logic).
#[tauri::command]
pub async fn get_files_by_tags(
    tag_names: Vec<String>,
    match_all: bool,
    state: State<'_, AppState>,
) -> Result<Vec<i64>, String> {
    let pool = state.database.pool().await;
    let repo = TagRepository::new(pool);

    repo.get_files_by_tags(&tag_names, match_all)
        .await
        .map_err(|e| format!("Failed to get files by tags: {}", e))
}

/// Get usage statistics for a tag
#[tauri::command]
pub async fn get_tag_stats(tag_id: i32, state: State<'_, AppState>) -> Result<i64, String> {
    let pool = state.database.pool().await;
    let repo = TagRepository::new(pool);

    repo.get_tag_file_count(tag_id)
        .await
        .map_err(|e| format!("Failed to get tag stats: {}", e))
}

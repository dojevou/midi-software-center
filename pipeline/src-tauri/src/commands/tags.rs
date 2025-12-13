//! Tag Commands - Tauri commands for tag CRUD operations and file ratings

use crate::db::repositories::tag_repository::{DbTag, TagRepository, TagWithCount};
use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

/// Tag for JSON serialization (frontend-friendly)
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TagResponse {
    pub id: i32,
    pub name: String,
    pub category: Option<String>,
    pub usage_count: i32,
}

impl From<DbTag> for TagResponse {
    fn from(db_tag: DbTag) -> Self {
        Self {
            id: db_tag.id,
            name: db_tag.name,
            category: db_tag.category,
            usage_count: db_tag.usage_count,
        }
    }
}

impl From<TagWithCount> for TagResponse {
    fn from(tag: TagWithCount) -> Self {
        Self { id: tag.id, name: tag.name, category: tag.category, usage_count: tag.usage_count }
    }
}

impl PartialEq<str> for TagResponse {
    fn eq(&self, other: &str) -> bool {
        self.name == other
    }
}

impl PartialEq<&str> for TagResponse {
    fn eq(&self, other: &&str) -> bool {
        self.name == *other
    }
}

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

    let categories = repo
        .get_tag_categories()
        .await
        .map_err(|e| format!("Failed to get tag categories: {}", e))?;

    Ok(categories)
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

    // Get or create tags and get their IDs
    let tag_data: Vec<(String, Option<String>)> = tag_names
        .into_iter()
        .map(|name| (name, None)) // No category for user-added tags
        .collect();

    let tag_ids = repo
        .get_or_create_tags_batch(&tag_data)
        .await
        .map_err(|e| format!("Failed to create tags: {}", e))?;

    // Update file tags
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

    // Get or create tags and get their IDs
    let tag_data: Vec<(String, Option<String>)> =
        tag_names.into_iter().map(|name| (name, None)).collect();

    let tag_ids = repo
        .get_or_create_tags_batch(&tag_data)
        .await
        .map_err(|e| format!("Failed to create tags: {}", e))?;

    // Add tags to file
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

    let file_ids = repo
        .get_files_by_tags(&tag_names, match_all)
        .await
        .map_err(|e| format!("Failed to get files by tags: {}", e))?;

    Ok(file_ids)
}

/// Get usage statistics for a tag
#[tauri::command]
pub async fn get_tag_stats(tag_id: i32, state: State<'_, AppState>) -> Result<i64, String> {
    let pool = state.database.pool().await;
    let repo = TagRepository::new(pool);

    let count = repo
        .get_tag_file_count(tag_id)
        .await
        .map_err(|e| format!("Failed to get tag stats: {}", e))?;

    Ok(count)
}

/// Set the rating for a file (1-5 stars, or None to clear).
#[tauri::command]
pub async fn set_file_rating(
    file_id: i64,
    rating: Option<i16>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Validate rating if provided
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

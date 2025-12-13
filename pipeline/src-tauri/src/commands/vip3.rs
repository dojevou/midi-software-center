//! VIP3 Browser Commands - Multi-filter file search, favorites, saved searches, collections

use crate::AppState;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use tauri::State;

// Pagination defaults
const DEFAULT_PAGE: i32 = 1;
const DEFAULT_PAGE_SIZE: i32 = 50;
const MAX_PAGE_SIZE: i32 = 500;

/// VIP3 browser filter state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Vip3Filters {
    /// Selected folder IDs
    #[serde(default)]
    pub folder_ids: Vec<i64>,

    /// Selected instrument IDs (from existing tags)
    #[serde(default)]
    pub instrument_ids: Vec<i32>,

    /// Selected timbre IDs
    #[serde(default)]
    pub timbre_ids: Vec<i16>,

    /// Selected style IDs
    #[serde(default)]
    pub style_ids: Vec<i16>,

    /// Selected articulation IDs
    #[serde(default)]
    pub articulation_ids: Vec<i16>,

    /// Selected BPM range IDs
    #[serde(default)]
    pub bpm_range_ids: Vec<i16>,

    /// Selected key IDs
    #[serde(default)]
    pub key_ids: Vec<i16>,

    /// Selected MIDI channel (1-16)
    pub channel: Option<i16>,

    /// Free-text search query
    pub search_query: Option<String>,

    /// Minimum rating filter (1-5)
    pub min_rating: Option<i16>,

    /// Only show favorites
    #[serde(default)]
    pub favorites_only: bool,

    /// Custom tag IDs
    #[serde(default)]
    pub tag_ids: Vec<i32>,
}

/// Sort configuration for VIP3 browser
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vip3Sort {
    pub field: String, // "filename", "bpm", "duration", "rating", "created_at"
    pub order: String, // "asc" or "desc"
}

impl Default for Vip3Sort {
    fn default() -> Self {
        Self { field: "filename".to_string(), order: "asc".to_string() }
    }
}

/// VIP3 search result item
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Vip3FileResult {
    pub id: i64,
    pub filename: String,
    pub filepath: String,
    pub bpm: Option<f64>,
    pub key_signature: Option<String>,
    pub duration_ms: Option<i32>,
    pub note_count: Option<i32>,
    pub channel: Option<i16>,
    pub rating: Option<i16>,
    pub favorite: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
}

/// Paginated VIP3 search results
#[derive(Debug, Serialize)]
pub struct Vip3SearchResults {
    pub files: Vec<Vip3FileResult>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

/// Category filter option with count
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct FilterOption {
    pub id: i32,
    pub name: String,
    pub file_count: i64,
}

/// All filter counts for VIP3 sidebar
#[derive(Debug, Serialize)]
pub struct Vip3FilterCounts {
    pub timbres: Vec<FilterOption>,
    pub styles: Vec<FilterOption>,
    pub articulations: Vec<FilterOption>,
    pub bpm_ranges: Vec<FilterOption>,
    pub keys: Vec<FilterOption>,
    pub instruments: Vec<FilterOption>,
}

/// Saved search record
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct SavedSearchResponse {
    pub id: i64,
    pub name: String,
    #[sqlx(default)]
    pub description: Option<String>,
    pub filters: Value,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    #[sqlx(default)]
    pub icon: Option<String>,
    #[sqlx(default)]
    pub color: Option<String>,
    #[sqlx(default)]
    pub is_pinned: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
    pub use_count: Option<i32>,
}

/// Data for creating a saved search
#[derive(Debug, Clone, Deserialize)]
pub struct CreateSavedSearchRequest {
    pub name: String,
    pub description: Option<String>,
    pub filters: Vip3Filters,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

/// Collection record
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct CollectionResponse {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_smart: Option<bool>,
    pub file_count: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
    pub modified_at: Option<DateTime<Utc>>,
}

/// Data for creating a collection
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCollectionRequest {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

/// Search files with VIP3-style multi-filter support (main browser query).
#[tauri::command]
pub async fn search_files_vip3(
    filters: Vip3Filters,
    sort: Option<Vip3Sort>,
    page: Option<i32>,
    page_size: Option<i32>,
    state: State<'_, AppState>,
) -> Result<Vip3SearchResults, String> {
    let pool = state.database.pool().await;
    let page = page.unwrap_or(DEFAULT_PAGE).max(1);
    let page_size = page_size.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE);
    let sort = sort.unwrap_or_default();
    let offset = (page - 1) * page_size;

    // Build dynamic WHERE clause
    let mut conditions = vec!["1=1".to_string()];
    let mut param_idx = 1;

    // Build conditions based on filters
    if !filters.folder_ids.is_empty() {
        conditions.push(format!("f.folder_id = ANY(${}::bigint[])", param_idx));
        param_idx += 1;
    }

    if !filters.timbre_ids.is_empty() {
        conditions.push(format!(
            "EXISTS (SELECT 1 FROM midi_file_timbres mft WHERE mft.file_id = f.id AND mft.timbre_id = ANY(${}::smallint[]))",
            param_idx
        ));
        param_idx += 1;
    }

    if !filters.style_ids.is_empty() {
        conditions.push(format!(
            "EXISTS (SELECT 1 FROM midi_file_styles mfs WHERE mfs.file_id = f.id AND mfs.style_id = ANY(${}::smallint[]))",
            param_idx
        ));
        param_idx += 1;
    }

    if !filters.articulation_ids.is_empty() {
        conditions.push(format!(
            "EXISTS (SELECT 1 FROM midi_file_articulations mfa WHERE mfa.file_id = f.id AND mfa.articulation_id = ANY(${}::smallint[]))",
            param_idx
        ));
        param_idx += 1;
    }

    if !filters.bpm_range_ids.is_empty() {
        conditions.push(format!("f.bpm_range_id = ANY(${}::smallint[])", param_idx));
        param_idx += 1;
    }

    if !filters.key_ids.is_empty() {
        conditions.push(format!("f.key_id = ANY(${}::smallint[])", param_idx));
        param_idx += 1;
    }

    if let Some(channel) = filters.channel {
        conditions.push(format!("f.channel = ${}", param_idx));
        param_idx += 1;
        let _ = channel; // Used in bind
    }

    if let Some(min_rating) = filters.min_rating {
        conditions.push(format!("f.rating >= ${}", param_idx));
        param_idx += 1;
        let _ = min_rating;
    }

    if filters.favorites_only {
        conditions.push("f.favorite = TRUE".to_string());
    }

    if let Some(ref query) = filters.search_query {
        if !query.is_empty() {
            conditions.push(format!(
                "(f.filename ILIKE '%' || ${} || '%' OR f.filepath ILIKE '%' || ${} || '%')",
                param_idx, param_idx
            ));
            param_idx += 1;
            let _ = query;
        }
    }

    if !filters.tag_ids.is_empty() {
        conditions.push(format!(
            "EXISTS (SELECT 1 FROM file_tags ft WHERE ft.file_id = f.id AND ft.tag_id = ANY(${}::integer[]))",
            param_idx
        ));
        // param_idx += 1; // Unused after this point
    }

    let where_clause = conditions.join(" AND ");

    // Build ORDER BY clause
    let order_clause = match (sort.field.as_str(), sort.order.as_str()) {
        ("filename", "asc") => "f.filename ASC",
        ("filename", "desc") => "f.filename DESC",
        ("bpm", "asc") => "f.bpm ASC NULLS LAST",
        ("bpm", "desc") => "f.bpm DESC NULLS LAST",
        ("duration", "asc") => "f.duration_ms ASC NULLS LAST",
        ("duration", "desc") => "f.duration_ms DESC NULLS LAST",
        ("rating", "asc") => "f.rating ASC NULLS LAST",
        ("rating", "desc") => "f.rating DESC NULLS FIRST",
        ("created_at", "asc") => "f.created_at ASC",
        ("created_at", "desc") => "f.created_at DESC",
        _ => "f.filename ASC",
    };

    // Execute count query
    let count_sql = format!("SELECT COUNT(*) FROM files f WHERE {}", where_clause);

    // Use raw query with dynamic binding
    let total_count: i64 = {
        let mut query = sqlx::query_scalar::<_, i64>(&count_sql);

        // Bind parameters in order
        if !filters.folder_ids.is_empty() {
            query = query.bind(&filters.folder_ids);
        }
        if !filters.timbre_ids.is_empty() {
            query = query.bind(&filters.timbre_ids);
        }
        if !filters.style_ids.is_empty() {
            query = query.bind(&filters.style_ids);
        }
        if !filters.articulation_ids.is_empty() {
            query = query.bind(&filters.articulation_ids);
        }
        if !filters.bpm_range_ids.is_empty() {
            query = query.bind(&filters.bpm_range_ids);
        }
        if !filters.key_ids.is_empty() {
            query = query.bind(&filters.key_ids);
        }
        if let Some(channel) = filters.channel {
            query = query.bind(channel);
        }
        if let Some(min_rating) = filters.min_rating {
            query = query.bind(min_rating);
        }
        if let Some(ref search_query) = filters.search_query {
            if !search_query.is_empty() {
                query = query.bind(search_query);
            }
        }
        if !filters.tag_ids.is_empty() {
            query = query.bind(&filters.tag_ids);
        }

        query.fetch_one(&pool).await.map_err(|e| format!("Count query error: {}", e))?
    };

    // Execute main query
    let files_sql = format!(
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
        WHERE {}
        ORDER BY {}
        LIMIT {} OFFSET {}
        "#,
        where_clause, order_clause, page_size, offset
    );

    let files: Vec<Vip3FileResult> = {
        let mut query = sqlx::query_as::<_, Vip3FileResult>(&files_sql);

        // Bind parameters in same order
        if !filters.folder_ids.is_empty() {
            query = query.bind(&filters.folder_ids);
        }
        if !filters.timbre_ids.is_empty() {
            query = query.bind(&filters.timbre_ids);
        }
        if !filters.style_ids.is_empty() {
            query = query.bind(&filters.style_ids);
        }
        if !filters.articulation_ids.is_empty() {
            query = query.bind(&filters.articulation_ids);
        }
        if !filters.bpm_range_ids.is_empty() {
            query = query.bind(&filters.bpm_range_ids);
        }
        if !filters.key_ids.is_empty() {
            query = query.bind(&filters.key_ids);
        }
        if let Some(channel) = filters.channel {
            query = query.bind(channel);
        }
        if let Some(min_rating) = filters.min_rating {
            query = query.bind(min_rating);
        }
        if let Some(ref search_query) = filters.search_query {
            if !search_query.is_empty() {
                query = query.bind(search_query);
            }
        }
        if !filters.tag_ids.is_empty() {
            query = query.bind(&filters.tag_ids);
        }

        query.fetch_all(&pool).await.map_err(|e| format!("Search query error: {}", e))?
    };

    let total_pages = ((total_count as f64) / (page_size as f64)).ceil() as i32;

    Ok(Vip3SearchResults { files, total_count, page, page_size, total_pages })
}

/// Get counts for all VIP3 filter categories (for sidebar).
#[tauri::command]
pub async fn get_vip3_filter_counts(
    state: State<'_, AppState>,
) -> Result<Vip3FilterCounts, String> {
    let pool = state.database.pool().await;

    // Get timbres with counts
    let timbres: Vec<FilterOption> = sqlx::query_as(
        r#"
        SELECT id::integer, name, file_count::bigint
        FROM timbres
        WHERE file_count > 0
        ORDER BY sort_order, name
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get timbres: {}", e))?;

    // Get styles with counts
    let styles: Vec<FilterOption> = sqlx::query_as(
        r#"
        SELECT id::integer, name, file_count::bigint
        FROM styles
        WHERE file_count > 0
        ORDER BY sort_order, name
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get styles: {}", e))?;

    // Get articulations with counts
    let articulations: Vec<FilterOption> = sqlx::query_as(
        r#"
        SELECT id::integer, name, file_count::bigint
        FROM articulations
        WHERE file_count > 0
        ORDER BY sort_order, name
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get articulations: {}", e))?;

    // Get BPM ranges with counts
    let bpm_ranges: Vec<FilterOption> = sqlx::query_as(
        r#"
        SELECT id::integer, label as name, file_count::bigint
        FROM bpm_ranges
        WHERE file_count > 0
        ORDER BY sort_order
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get BPM ranges: {}", e))?;

    // Get musical keys with counts
    let keys: Vec<FilterOption> = sqlx::query_as(
        r#"
        SELECT id::integer, name, file_count::bigint
        FROM musical_keys
        WHERE file_count > 0
        ORDER BY sort_order
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get keys: {}", e))?;

    // Get instruments (from existing tags table, category = 'instrument')
    let instruments: Vec<FilterOption> = sqlx::query_as(
        r#"
        SELECT id, name, usage_count::bigint as file_count
        FROM tags
        WHERE category = 'instrument' AND usage_count > 0
        ORDER BY usage_count DESC, name
        "#,
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default(); // Gracefully handle if tags table doesn't have instrument category

    Ok(Vip3FilterCounts { timbres, styles, articulations, bpm_ranges, keys, instruments })
}

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

/// Response for file categories
#[derive(Debug, Serialize)]
pub struct FileCategoriesResponse {
    pub timbres: Vec<FilterOption>,
    pub styles: Vec<FilterOption>,
    pub articulations: Vec<FilterOption>,
}

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

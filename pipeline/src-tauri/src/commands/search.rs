/// Search command handlers - GROWN-UP SCRIPT ARCHETYPE
///
/// PURPOSE: Advanced search functionality with filters and pagination
/// ARCHETYPE: Grown-up Script (I/O operations, reusable logic)
///
/// ✅ CAN: Perform database I/O
/// ✅ CAN: Have side effects (complex queries)
/// ✅ SHOULD: Handle errors properly
/// ❌ NO: Complex business logic (delegate to Trusty Modules)
use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

// =============================================================================
// DATA STRUCTURES
// =============================================================================

/// Search filters from frontend
#[derive(Debug, Clone, Deserialize)]
pub struct SearchFilters {
    pub category: Option<String>,
    pub min_bpm: Option<f64>,
    pub max_bpm: Option<f64>,
    pub key_signature: Option<String>,
}

/// Search result item (simplified for list view)
///
/// Note: NUMERIC columns are cast to float8 in SQL queries for simplicity
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct SearchResultItem {
    pub id: i64,
    pub filename: String,
    pub filepath: String,
    pub bpm: Option<f64>, // Cast from NUMERIC in SQL
    pub key_signature: Option<String>,
    pub duration_seconds: Option<f64>, // Cast from NUMERIC in SQL
    pub category: Option<String>,
}

/// Paginated search results
#[derive(Debug, Serialize)]
pub struct SearchResults {
    pub items: Vec<SearchResultItem>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Count search results for pagination
async fn count_search_results(
    query: &str,
    filters: &SearchFilters,
    pool: &sqlx::PgPool,
) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM files f
        LEFT JOIN musical_metadata mm ON f.id = mm.file_id
        LEFT JOIN file_categories fc ON f.id = fc.file_id
        WHERE
            ($1::text = '' OR f.filename ILIKE '%' || $1 || '%' OR f.filepath ILIKE '%' || $1 || '%')
            AND ($2::text IS NULL OR fc.primary_category::text = $2)
            AND ($3::float8 IS NULL OR mm.bpm >= $3)
            AND ($4::float8 IS NULL OR mm.bpm <= $4)
            AND ($5::text IS NULL OR mm.key_signature::text = $5)
        "#
    )
    .bind(query)
    .bind(&filters.category)
    .bind(filters.min_bpm)
    .bind(filters.max_bpm)
    .bind(&filters.key_signature)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

// =============================================================================
// TAURI COMMANDS
// =============================================================================

/// Search files with filters and pagination (implementation for tests and reuse)
pub async fn search_files_impl(
    query: String,
    filters: SearchFilters,
    page: i32,
    page_size: i32,
    state: &AppState,
) -> Result<SearchResults, String> {
    let pool = state.database.pool().await;

    // Validate pagination
    if page < 1 {
        return Err("Page must be >= 1".to_string());
    }
    if !(1..=100).contains(&page_size) {
        return Err("Page size must be between 1 and 100".to_string());
    }

    // Calculate offset
    let offset = (page - 1) * page_size;

    // Query with correct column names from schema
    let items = sqlx::query_as::<_, SearchResultItem>(
        r#"
        SELECT
            f.id,
            f.filename,
            f.filepath,
            mm.bpm::float8 as bpm,
            mm.key_signature::text as key_signature,
            f.duration_seconds::float8 as duration_seconds,
            fc.primary_category::text as category
        FROM files f
        LEFT JOIN musical_metadata mm ON f.id = mm.file_id
        LEFT JOIN file_categories fc ON f.id = fc.file_id
        WHERE
            ($1::text = '' OR f.filename ILIKE '%' || $1 || '%' OR f.filepath ILIKE '%' || $1 || '%')
            AND ($2::text IS NULL OR fc.primary_category::text = $2)
            AND ($3::float8 IS NULL OR mm.bpm >= $3)
            AND ($4::float8 IS NULL OR mm.bpm <= $4)
            AND ($5::text IS NULL OR mm.key_signature::text = $5)
        ORDER BY f.created_at DESC
        LIMIT $6 OFFSET $7
        "#
    )
    .bind(&query)
    .bind(&filters.category)
    .bind(filters.min_bpm)
    .bind(filters.max_bpm)
    .bind(&filters.key_signature)
    .bind(page_size as i64)
    .bind(offset as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Search error: {}", e))?;

    let total_count = count_search_results(&query, &filters, &pool)
        .await
        .map_err(|e| format!("Count error: {}", e))?;

    Ok(SearchResults {
        items,
        total_count,
        page,
        page_size,
        total_pages: ((total_count as f64) / (page_size as f64)).ceil() as i32,
    })
}

/// Search files with filters and pagination
///
/// # Manager Archetype
/// - ✅ Performs I/O (complex database query)
/// - ✅ Has side effects (reads from database)
/// - ✅ Handles errors properly
///
/// # Arguments
///
/// * `query` - Text search query (searches filename and filepath)
/// * `filters` - Search filters (category, BPM range, key)
/// * `page` - Page number (1-indexed)
/// * `page_size` - Items per page (1-100)
///
/// # Returns
///
/// Paginated search results with total count
#[tauri::command]
pub async fn pipeline_search_files(
    query: String,
    filters: SearchFilters,
    page: i32,
    page_size: i32,
    state: State<'_, AppState>,
) -> Result<SearchResults, String> {
    search_files_impl(query, filters, page, page_size, &state).await
}

/// Get all unique tags from database (implementation for tests and reuse)
pub async fn get_all_tags_impl(state: &AppState) -> Result<Vec<String>, String> {
    let tags: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT DISTINCT tag_name
        FROM file_tags
        ORDER BY tag_name ASC
        "#,
    )
    .fetch_all(&state.database.pool().await)
    .await
    .map_err(|e| format!("Failed to get tags: {}", e))?;

    Ok(tags.into_iter().map(|(tag,)| tag).collect())
}

/// Get all unique tags from database
///
/// Returns a list of all unique tag names used in the database.
///
/// # Frontend Usage
///
/// ```typescript
/// const tags = await invoke<string[]>('get_all_tags');
/// ```
#[tauri::command]
pub async fn get_all_tags(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    get_all_tags_impl(&state).await
}

/// Get files by tag
///
/// Returns all files that have a specific tag.
///
/// # Arguments
///
/// * `tag` - Tag name to filter by
///
/// # Frontend Usage
///
/// ```typescript
/// const files = await invoke<FileMetadata[]>('get_files_by_tag', { tag: 'ambient' });
/// ```
#[tauri::command]
pub async fn get_files_by_tag(
    tag: String,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResultItem>, String> {
    let files = sqlx::query_as::<_, SearchResultItem>(
        r#"
        SELECT
            f.id,
            f.filename,
            f.filepath,
            mm.bpm::float8 as bpm,
            mm.key_signature::text as key_signature,
            f.duration_seconds::float8 as duration_seconds,
            fc.primary_category::text as category
        FROM files f
        LEFT JOIN musical_metadata mm ON f.id = mm.file_id
        LEFT JOIN file_categories fc ON f.id = fc.file_id
        INNER JOIN file_tags ft ON f.id = ft.file_id
        WHERE ft.tag_name = $1
        ORDER BY f.created_at DESC
        "#,
    )
    .bind(tag)
    .fetch_all(&state.database.pool().await)
    .await
    .map_err(|e| format!("Failed to get files by tag: {}", e))?;

    Ok(files)
}

/// Get BPM range from database (implementation for tests and reuse)
pub async fn get_bpm_range_impl(state: &AppState) -> Result<BpmRange, String> {
    let pool = state.database.pool().await;
    let result: Option<(Option<f64>, Option<f64>)> = sqlx::query_as(
        r#"
        SELECT MIN(bpm)::float8, MAX(bpm)::float8
        FROM musical_metadata
        WHERE bpm IS NOT NULL
        "#,
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| format!("Failed to get BPM range: {}", e))?;

    match result {
        Some((Some(min), Some(max))) => Ok(BpmRange { min, max }),
        _ => Ok(BpmRange { min: 0.0, max: 300.0 }), // Default range if no data
    }
}

/// Get BPM range from database
///
/// Returns the minimum and maximum BPM values in the database.
///
/// # Frontend Usage
///
/// ```typescript
/// const range = await invoke<{min: number, max: number}>('get_bpm_range');
/// ```
#[tauri::command]
pub async fn get_bpm_range(state: State<'_, AppState>) -> Result<BpmRange, String> {
    get_bpm_range_impl(&state).await
}

/// Get all unique key signatures from database
///
/// Returns a list of all unique key signatures.
///
/// # Frontend Usage
///
/// ```typescript
/// const keys = await invoke<string[]>('get_all_keys');
/// ```
#[tauri::command]
pub async fn get_all_keys(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let keys: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT DISTINCT key_signature::text
        FROM musical_metadata
        WHERE key_signature IS NOT NULL
        ORDER BY key_signature ASC
        "#,
    )
    .fetch_all(&state.database.pool().await)
    .await
    .map_err(|e| format!("Failed to get keys: {}", e))?;

    Ok(keys.into_iter().map(|(key,)| key).collect())
}

/// BPM range response
#[derive(Debug, Serialize)]
pub struct BpmRange {
    pub min: f64,
    pub max: f64,
}

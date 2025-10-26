cat > src/commands/search.rs << 'EOF'
//! Search command handlers

use crate::db::models::{FileSearchResult, SearchFilters, SearchResults, bigdecimal_to_f64, f64_to_bigdecimal};
use crate::AppState;
use tauri::State;
use sqlx::types::BigDecimal;

/// Count search results for pagination
async fn count_search_results(
    query: &str,
    filters: &SearchFilters,
    pool: &sqlx::PgPool,
) -> Result<i64, sqlx::Error> {
    let count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM files f
        LEFT JOIN musical_metadata mm ON f.id = mm.file_id
        WHERE
            ($1::text IS NULL OR f.search_vector @@ plainto_tsquery('english', $1))
            AND ($2::text IS NULL OR f.category = $2::text)
            AND ($3::numeric IS NULL OR mm.detected_bpm >= $3)
            AND ($4::numeric IS NULL OR mm.detected_bpm <= $4)
        "#,
        if query.is_empty() { None } else { Some(query) },
        filters.category.as_deref(),
        f64_to_bigdecimal(filters.min_bpm),
        f64_to_bigdecimal(filters.max_bpm),
    )
    .fetch_one(pool)
    .await?;

    Ok(count)
}

/// Searches files by text query with filters and pagination
#[tauri::command]
pub async fn search_files(
    query: String,
    filters: SearchFilters,
    page: i32,
    page_size: i32,
    state: State<'_, AppState>,
) -> Result<SearchResults, String> {
    let pool = &state.db_pool;

    // Validate pagination
    if page < 1 {
        return Err("Page must be >= 1".to_string());
    }
    if page_size < 1 || page_size > 100 {
        return Err("Page size must be between 1 and 100".to_string());
    }

    // Build SQL query based on filters
    let offset = (page - 1) * page_size;

    // Use raw query to avoid macro issues with custom types
    let rows = sqlx::query(
        r#"
        SELECT
            f.id,
            f.new_filename as file_name,
            f.category,
            mm.detected_bpm,
            mm.detected_key,
            mm.duration_seconds
        FROM files f
        LEFT JOIN musical_metadata mm ON f.id = mm.file_id
        WHERE
            ($1::text IS NULL OR f.search_vector @@ plainto_tsquery('english', $1))
            AND ($2::text IS NULL OR f.category = $2::text)
            AND ($3::numeric IS NULL OR mm.detected_bpm >= $3)
            AND ($4::numeric IS NULL OR mm.detected_bpm <= $4)
        ORDER BY
            CASE WHEN $1::text IS NOT NULL
                THEN ts_rank(f.search_vector, plainto_tsquery('english', $1))
                ELSE 0
            END DESC,
            f.created_at DESC
        LIMIT $5 OFFSET $6
        "#
    )
    .bind(if query.is_empty() { None } else { Some(&query) })
    .bind(filters.category.as_deref())
    .bind(f64_to_bigdecimal(filters.min_bpm))
    .bind(f64_to_bigdecimal(filters.max_bpm))
    .bind(page_size as i64)
    .bind(offset as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Search error: {}", e))?;

    // Convert to proper result type
    let file_results: Vec<FileSearchResult> = rows.into_iter().map(|row| {
        FileSearchResult {
            id: row.get("id"),
            file_name: row.get::<String, _>("file_name"),
            file_extension: "mid".to_string(),
            detected_bpm: bigdecimal_to_f64(row.get::<Option<BigDecimal>, _>("detected_bpm")),
            category: row.get::<Option<String>, _>("category").unwrap_or_default(),
            duration_seconds: bigdecimal_to_f64(row.get::<Option<BigDecimal>, _>("duration_seconds")),
            query: query.clone(),
            offset: offset as i64,
        }
    }).collect();

    let total_count = count_search_results(&query, &filters, pool)
        .await
        .map_err(|e| format!("Count error: {}", e))?;

    Ok(SearchResults {
        files: file_results,
        total_count,
        page,
        page_size,
        total_pages: (total_count as f64 / page_size as f64).ceil() as i32,
    })
}

/// Get file by ID
#[tauri::command]
pub async fn get_file_details(
    file_id: i64,
    state: State<'_, AppState>,
) -> Result<Option<crate::db::models::FileDetails>, String> {
    let pool = &state.db_pool;
    
    // This would need a more complex query to join with metadata
    // For now, return None as placeholder
    Ok(None)
}
EOF

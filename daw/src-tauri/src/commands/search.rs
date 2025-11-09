   /// Search Tauri commands
   ///
   /// Thin wrappers that expose search functionality to the frontend.
   /// Queries the PostgreSQL database for MIDI files with filtering and sorting.
   /// Updated to use proper JOINs with actual database schema.

use crate::commands::AppState;
use crate::models::midi_file::FileDetails;
use crate::models::search::{SearchFilters, SearchResponse, Suggestion};
use sqlx::Row;
use tauri::State;
use tracing::{debug, error};

/// Search for MIDI files with filters
///
/// Supports filtering by:
/// - BPM range
/// - Key signature
/// - Time signature
/// - Category
/// - Note count range
/// - Duration range
/// - Full-text search in filename
#[tauri::command]
pub async fn search_files(
    filters: SearchFilters,
    state: State<'_, AppState>,
) -> Result<SearchResponse, String> {
    debug!("Searching files with filters: {:?}", filters);

    // Build base query with proper JOINs and type casts
    let mut query = String::from(
        r#"
        SELECT
            f.id,
            f.filename,
            f.filepath,
            f.file_size_bytes,
            f.parent_folder,
            f.created_at,
            f.content_hash,
            f.num_tracks,
            f.manufacturer,
            f.collection_name,
            COALESCE(f.folder_tags, ARRAY[]::TEXT[]) as folder_tags,
            mm.bpm::FLOAT8 as bpm,
            mm.key_signature::TEXT as key_signature,
            CASE
                WHEN mm.time_signature_numerator IS NOT NULL
                THEN mm.time_signature_numerator::TEXT || '/' || mm.time_signature_denominator::TEXT
                ELSE NULL
            END as time_signature,
            f.duration_seconds::FLOAT8 as duration_seconds,
            mm.total_notes,
            mm.is_percussive as has_drums,
            fc.primary_category::TEXT as primary_category,
            CASE WHEN fav.file_id IS NOT NULL THEN true ELSE false END as is_favorite
        FROM files f
        LEFT JOIN musical_metadata mm ON f.id = mm.file_id
        LEFT JOIN file_categories fc ON f.id = fc.file_id
        LEFT JOIN favorites fav ON f.id = fav.file_id
        WHERE 1=1
        "#
    );

    let mut conditions = Vec::new();

    // BPM filter
    if let Some(min_bpm) = filters.min_bpm {
        conditions.push(format!("mm.bpm >= {}", min_bpm));
    }
    if let Some(max_bpm) = filters.max_bpm {
        conditions.push(format!("mm.bpm <= {}", max_bpm));
    }

    // Key signature filter
    if let Some(ref key) = filters.key_signature {
        conditions.push(format!("mm.key_signature = '{}'", key.replace("'", "''")));
    }

    // Time signature filter (need to match formatted string)
    if let Some(ref time_sig) = filters.time_signature {
        let parts: Vec<&str> = time_sig.split('/').collect();
        if parts.len() == 2 {
            conditions.push(format!(
                "mm.time_signature_numerator = {} AND mm.time_signature_denominator = {}",
                parts[0], parts[1]
            ));
        }
    }

    // Category filter (check primary_category in file_categories table)
    if let Some(ref category) = filters.category {
        conditions.push(format!(
            "fc.primary_category::TEXT ILIKE '%{}%'",
            category.replace("'", "''")
        ));
    }

    // Note count range
    if let Some(min_notes) = filters.min_notes {
        conditions.push(format!("mm.total_notes >= {}", min_notes));
    }
    if let Some(max_notes) = filters.max_notes {
        conditions.push(format!("mm.total_notes <= {}", max_notes));
    }

    // Duration range
    if let Some(min_duration) = filters.min_duration {
        conditions.push(format!("f.duration_seconds >= {}", min_duration));
    }
    if let Some(max_duration) = filters.max_duration {
        conditions.push(format!("f.duration_seconds <= {}", max_duration));
    }

    // Text search in filename
    if let Some(ref search_text) = filters.search_text {
        if !search_text.is_empty() {
            conditions.push(format!("f.filename ILIKE '%{}%'", search_text.replace("'", "''")));
        }
    }

    // Add all conditions to query
    for condition in &conditions {
        query.push_str(&format!(" AND {}", condition));
    }

    // Add sorting - map frontend field names to actual DB columns
    let sort_by = match filters.sort_by.as_deref().unwrap_or("created_at") {
        "file_name" => "f.filename",
        "bpm" => "mm.bpm",
        "key_signature" => "mm.key_signature",
        "duration_seconds" => "f.duration_seconds",
        "note_count" => "mm.total_notes",
        "created_at" => "f.created_at",
        _ => "f.created_at",
    };

    let sort_order = if filters.sort_desc.unwrap_or(false) {
        "DESC"
    } else {
        "ASC"
    };
    query.push_str(&format!(" ORDER BY {} {}", sort_by, sort_order));

    // Add pagination
    let limit = filters.limit.unwrap_or(50).min(500); // Cap at 500
    let offset = filters.offset.unwrap_or(0);
    query.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

    debug!("Executing query: {}", query);

    // Execute query
    let rows = sqlx::query(&query)
        .fetch_all(state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?)
        .await
        .map_err(|e| {
            error!("Database query failed: {}", e);
            format!("Failed to search files: {}", e)
        })?;

    // Convert rows to FileDetails structs
    let files: Vec<FileDetails> = rows
        .iter()
        .map(|row| FileDetails {
            id: row.get("id"),
            filename: row.get("filename"),
            filepath: row.get("filepath"),
            file_size_bytes: row.get("file_size_bytes"),
            bpm: row.try_get("bpm").ok(),
            key_signature: row.try_get("key_signature").ok(),
            time_signature: row.try_get("time_signature").ok(),
            duration_seconds: row.try_get("duration_seconds").ok(),
            total_notes: row.try_get("total_notes").ok(),
            primary_category: row.try_get("primary_category").ok(),
            parent_folder: row.try_get("parent_folder").ok(),
            created_at: row.get("created_at"),
            is_favorite: row.try_get("is_favorite").unwrap_or(false),
            tags: row.try_get("folder_tags").unwrap_or_default(),
            manufacturer: row.try_get("manufacturer").ok(),
            collection_name: row.try_get("collection_name").ok(),
            track_count: row.try_get("num_tracks").unwrap_or(0),
            has_notes: row.try_get::<Option<i32>, _>("total_notes").ok().flatten().unwrap_or(0) > 0,
            has_drums: row.try_get("has_drums").ok(),
            content_hash: row.try_get("content_hash").unwrap_or_default(),
        })
        .collect();

    // Get total count (without pagination) - must include all JOINs for WHERE conditions
    let mut count_query = String::from(
        "SELECT COUNT(*) FROM files f \
         LEFT JOIN musical_metadata mm ON f.id = mm.file_id \
         LEFT JOIN file_categories fc ON f.id = fc.file_id \
         LEFT JOIN favorites fav ON f.id = fav.file_id \
         WHERE 1=1"
    );
    for condition in &conditions {
        count_query.push_str(&format!(" AND {}", condition));
    }

    let total: i64 = sqlx::query_scalar(&count_query)
        .fetch_one(state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?)
        .await
        .map_err(|e| {
            error!("Count query failed: {}", e);
            format!("Failed to count results: {}", e)
        })?;

    debug!("Found {} total results, returning {} files", total, files.len());

    Ok(SearchResponse {
        files,
        total: total as i32,
    })
}

/// Get detailed information about a specific file
#[tauri::command]
pub async fn get_file_details(
    file_id: i32,
    state: State<'_, AppState>,
) -> Result<FileDetails, String> {
    debug!("Getting details for file ID: {}", file_id);

    let file = sqlx::query_as!(
        FileDetails,
        r#"
        SELECT
            f.id as "id!",
            f.filename as "filename!",
            f.filepath as "filepath!",
            f.file_size_bytes as "file_size_bytes!",
            f.parent_folder as "parent_folder?",
            f.created_at as "created_at!",
            f.content_hash as "content_hash!",
            f.num_tracks as "track_count!",
            f.manufacturer as "manufacturer?",
            f.collection_name as "collection_name?",
            COALESCE(f.folder_tags, ARRAY[]::TEXT[]) as "tags!",
            mm.bpm::FLOAT8 as "bpm?",
            mm.key_signature::TEXT as "key_signature?",
            CASE
                WHEN mm.time_signature_numerator IS NOT NULL
                THEN mm.time_signature_numerator::TEXT || '/' || mm.time_signature_denominator::TEXT
                ELSE NULL
            END as "time_signature?",
            f.duration_seconds::FLOAT8 as "duration_seconds?",
            COALESCE(mm.total_notes, 0) > 0 as "has_notes!",
            mm.total_notes as "total_notes?",
            mm.is_percussive as "has_drums?",
            fc.primary_category::TEXT as "primary_category?",
            CASE WHEN fav.file_id IS NOT NULL THEN true ELSE false END as "is_favorite!"
        FROM files f
        LEFT JOIN musical_metadata mm ON f.id = mm.file_id
        LEFT JOIN file_categories fc ON f.id = fc.file_id
        LEFT JOIN favorites fav ON f.id = fav.file_id
        WHERE f.id = $1
        "#,
        file_id as i64
    )
    .fetch_optional(state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?)
    .await
    .map_err(|e| {
        error!("Database query failed: {}", e);
        format!("Failed to get file details: {}", e)
    })?;

    file.ok_or_else(|| format!("File with ID {} not found", file_id))
}

/// Get autocomplete suggestions for search
///
/// Provides suggestions for:
/// - Categories
/// - Key signatures
/// - Time signatures
#[tauri::command]
pub async fn get_search_suggestions(
    query: String,
    field: String,
    state: State<'_, AppState>,
) -> Result<Vec<Suggestion>, String> {
    debug!("Getting suggestions for field '{}' with query '{}'", field, query);

    let suggestions: Vec<Suggestion> = match field.as_str() {
        "category" => {
            let rows: Vec<(String,)> = sqlx::query_as(
                "SELECT DISTINCT primary_category::TEXT as category FROM file_categories
                 WHERE primary_category IS NOT NULL
                 ORDER BY category LIMIT 10"
            )
            .bind(format!("%{}%", query))
            .fetch_all(state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?)
            .await
            .map_err(|e| format!("Failed to get category suggestions: {}", e))?;

            rows.into_iter()
                .map(|(value,)| Suggestion { value })
                .collect()
        }
        "key_signature" => {
            let rows: Vec<(String,)> = sqlx::query_as(
                "SELECT DISTINCT key_signature::TEXT FROM musical_metadata
                 WHERE key_signature IS NOT NULL AND key_signature::TEXT ILIKE $1
                 ORDER BY key_signature LIMIT 10"
            )
            .bind(format!("%{}%", query))
            .fetch_all(state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?)
            .await
            .map_err(|e| format!("Failed to get key suggestions: {}", e))?;

            rows.into_iter()
                .map(|(value,)| Suggestion { value })
                .collect()
        }
        "time_signature" => {
            let rows: Vec<(String,)> = sqlx::query_as(
                "SELECT DISTINCT
                    time_signature_numerator::TEXT || '/' || time_signature_denominator::TEXT as time_sig
                 FROM musical_metadata
                 WHERE time_signature_numerator IS NOT NULL
                   AND time_signature_denominator IS NOT NULL
                 ORDER BY time_sig LIMIT 10"
            )
            .bind(format!("%{}%", query))
            .fetch_all(state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?)
            .await
            .map_err(|e| format!("Failed to get time signature suggestions: {}", e))?;

            rows.into_iter()
                .map(|(value,)| Suggestion { value })
                .collect()
        }
        _ => {
            return Err(format!("Unknown field for suggestions: {}", field));
        }
    };

    debug!("Returning {} suggestions", suggestions.len());
    Ok(suggestions)
}

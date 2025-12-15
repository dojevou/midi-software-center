//! VIP3 search commands

use super::types::{
    FilterOption, Vip3FileResult, Vip3FilterCounts, Vip3Filters, Vip3SearchResults, Vip3Sort,
};
use crate::AppState;
use tauri::State;

// Pagination defaults
pub const DEFAULT_PAGE: i32 = 1;
pub const DEFAULT_PAGE_SIZE: i32 = 50;
pub const MAX_PAGE_SIZE: i32 = 500;

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

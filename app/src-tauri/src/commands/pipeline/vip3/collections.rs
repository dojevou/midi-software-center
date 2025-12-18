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
        INSERT INTO collections (name, description, icon, color, is_smart, smart_filters)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(&request.name)
    .bind(&request.description)
    .bind(&request.icon)
    .bind(&request.color)
    .bind(&request.is_smart)
    .bind(&request.smart_filters)
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
        SELECT
            c.id,
            c.name,
            c.description,
            c.icon,
            c.color,
            c.is_smart,
            COUNT(cf.file_id)::int4 as file_count,
            c.created_at,
            c.modified_at
        FROM collections c
        LEFT JOIN collection_files cf ON c.id = cf.collection_id
        GROUP BY c.id, c.name, c.description, c.icon, c.color, c.is_smart, c.created_at, c.modified_at
        ORDER BY c.name
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
        SELECT
            c.id,
            c.name,
            c.description,
            c.icon,
            c.color,
            c.is_smart,
            COUNT(cf.file_id)::int4 as file_count,
            c.created_at,
            c.modified_at
        FROM collections c
        LEFT JOIN collection_files cf ON c.id = cf.collection_id
        WHERE c.id = $1
        GROUP BY c.id, c.name, c.description, c.icon, c.color, c.is_smart, c.created_at, c.modified_at
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
    smart_filters: Option<serde_json::Value>,
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
            smart_filters = COALESCE($6, smart_filters),
            modified_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(collection_id)
    .bind(name)
    .bind(description)
    .bind(icon)
    .bind(color)
    .bind(smart_filters)
    .execute(&pool)
    .await
    .map_err(|e| format!("Failed to update collection: {}", e))?;

    Ok(())
}

/// Batch add multiple files to a collection
#[tauri::command]
pub async fn batch_add_files_to_collection(
    collection_id: i64,
    file_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let pool = state.database.pool().await;

    // Get current max sort_order
    let max_order: Option<(Option<i32>,)> =
        sqlx::query_as("SELECT MAX(sort_order) FROM collection_files WHERE collection_id = $1")
            .bind(collection_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| format!("Failed to get max sort order: {}", e))?;

    let mut next_order = max_order.and_then(|(o,)| o).unwrap_or(0) + 1;
    let mut added = 0;

    for file_id in file_ids {
        let result = sqlx::query(
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
        .map_err(|e| format!("Failed to add file {} to collection: {}", file_id, e))?;

        if result.rows_affected() > 0 {
            added += 1;
            next_order += 1;
        }
    }

    Ok(added)
}

/// Batch remove multiple files from a collection
#[tauri::command]
pub async fn batch_remove_files_from_collection(
    collection_id: i64,
    file_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let pool = state.database.pool().await;

    let result = sqlx::query(
        r#"
        DELETE FROM collection_files
        WHERE collection_id = $1 AND file_id = ANY($2)
        "#,
    )
    .bind(collection_id)
    .bind(&file_ids)
    .execute(&pool)
    .await
    .map_err(|e| format!("Failed to remove files from collection: {}", e))?;

    Ok(result.rows_affected() as usize)
}

/// Clear all files from a collection
#[tauri::command]
pub async fn clear_collection(
    collection_id: i64,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let pool = state.database.pool().await;

    let result = sqlx::query("DELETE FROM collection_files WHERE collection_id = $1")
        .bind(collection_id)
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to clear collection: {}", e))?;

    // Update modified_at
    sqlx::query("UPDATE collections SET modified_at = NOW() WHERE id = $1")
        .bind(collection_id)
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to update collection timestamp: {}", e))?;

    Ok(result.rows_affected() as usize)
}

/// Reorder files in a collection
#[tauri::command]
pub async fn reorder_collection_files(
    collection_id: i64,
    file_ids_in_order: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.database.pool().await;

    // Update sort_order for each file
    for (index, file_id) in file_ids_in_order.iter().enumerate() {
        sqlx::query(
            r#"
            UPDATE collection_files
            SET sort_order = $1
            WHERE collection_id = $2 AND file_id = $3
            "#,
        )
        .bind(index as i32)
        .bind(collection_id)
        .bind(file_id)
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to reorder file {}: {}", file_id, e))?;
    }

    // Update modified_at
    sqlx::query("UPDATE collections SET modified_at = NOW() WHERE id = $1")
        .bind(collection_id)
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to update collection timestamp: {}", e))?;

    Ok(())
}

/// Get files for a smart collection (dynamically computed from filters)
#[tauri::command]
pub async fn get_smart_collection_files(
    collection_id: i64,
    page: Option<i32>,
    page_size: Option<i32>,
    state: State<'_, AppState>,
) -> Result<Vip3SearchResults, String> {
    let pool = state.database.pool().await;
    let page = page.unwrap_or(DEFAULT_PAGE).max(1);
    let page_size = page_size.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE);
    let offset = (page - 1) * page_size;

    // Get the collection's smart_filters
    let collection: (Option<bool>, Option<serde_json::Value>,) = sqlx::query_as(
        "SELECT is_smart, smart_filters FROM collections WHERE id = $1",
    )
    .bind(collection_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("Failed to get collection: {}", e))?;

    let (is_smart, smart_filters) = collection;

    // If not a smart collection, fall back to regular collection files
    if !is_smart.unwrap_or(false) {
        return get_collection_files(collection_id, Some(page), Some(page_size), state).await;
    }

    // Parse filters
    let filters: super::types::Vip3Filters = smart_filters
        .map(|v| serde_json::from_value(v).unwrap_or_default())
        .unwrap_or_default();

    // Build dynamic WHERE clause (same logic as search_files_vip3)
    let mut conditions = vec!["1=1".to_string()];
    let mut param_idx = 1;

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
        let _ = channel;
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
        }
    }

    if !filters.tag_ids.is_empty() {
        conditions.push(format!(
            "EXISTS (SELECT 1 FROM file_tags ft WHERE ft.file_id = f.id AND ft.tag_id = ANY(${}::integer[]))",
            param_idx
        ));
    }

    let where_clause = conditions.join(" AND ");

    // Execute count query
    let count_sql = format!("SELECT COUNT(*) FROM files f WHERE {}", where_clause);

    let total_count: i64 = {
        let mut query = sqlx::query_scalar::<_, i64>(&count_sql);

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
        if let Some(ref q) = filters.search_query {
            if !q.is_empty() {
                query = query.bind(q);
            }
        }
        if !filters.tag_ids.is_empty() {
            query = query.bind(&filters.tag_ids);
        }

        query.fetch_one(&pool).await.map_err(|e| format!("Failed to count files: {}", e))?
    };

    // Execute files query
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
        ORDER BY f.filename ASC
        LIMIT {} OFFSET {}
        "#,
        where_clause, page_size, offset
    );

    let files: Vec<Vip3FileResult> = {
        let mut query = sqlx::query_as::<_, Vip3FileResult>(&files_sql);

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
        if let Some(ref q) = filters.search_query {
            if !q.is_empty() {
                query = query.bind(q);
            }
        }
        if !filters.tag_ids.is_empty() {
            query = query.bind(&filters.tag_ids);
        }

        query.fetch_all(&pool).await.map_err(|e| format!("Failed to get files: {}", e))?
    };

    let total_pages = ((total_count as f64) / (page_size as f64)).ceil() as i32;

    Ok(Vip3SearchResults { files, total_count, page, page_size, total_pages })
}

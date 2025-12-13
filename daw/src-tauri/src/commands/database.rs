//! Database commands for DAW
#![allow(dead_code)] // Commands are called externally via Tauri IPC

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::{command, State};
use tracing::info;

use crate::commands::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MidiFile {
    pub id: i64,
    pub file_path: String,
    pub file_name: String,
    pub bpm: f32,
    pub key_signature: String,
    pub tags: Vec<String>,
    pub duration: f32,
    pub track_count: i32,
    pub file_size: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SearchFilters {
    pub query: Option<String>,
    pub bpm_min: Option<f32>,
    pub bpm_max: Option<f32>,
    pub key: Option<String>,
    pub tag: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResults {
    pub files: Vec<MidiFile>,
    pub total_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_files: i64,
    pub avg_bpm: f32,
    pub total_size: i64,
}

#[command]
pub async fn database_search(
    state: State<'_, AppState>,
    filters: SearchFilters,
) -> Result<SearchResults, String> {
    let pool = state.db_pool.as_ref().ok_or("Database not available".to_string())?;

    // Build WHERE clause dynamically
    let mut where_clause = String::new();

    if let Some(q) = &filters.query {
        where_clause.push_str(&format!(
            " AND (file_name ILIKE '%{}%' OR tags::text ILIKE '%{}%')",
            q.replace("'", "''"),
            q.replace("'", "''")
        ));
    }

    if let Some(min) = filters.bpm_min {
        where_clause.push_str(&format!(" AND bpm >= {}", min));
    }

    if let Some(max) = filters.bpm_max {
        where_clause.push_str(&format!(" AND bpm <= {}", max));
    }

    if let Some(k) = &filters.key {
        where_clause.push_str(&format!(" AND key_signature = '{}'", k.replace("'", "''")));
    }

    if let Some(t) = &filters.tag {
        where_clause.push_str(&format!(" AND tags ? '{}'", t.replace("'", "''")));
    }

    let limit = filters.limit.unwrap_or(50);
    let offset = filters.offset.unwrap_or(0);

    // Build count query
    let count_query = format!(
        "SELECT COUNT(*) as count FROM files WHERE 1=1{}",
        where_clause
    );

    // Fetch total count using sqlx::query with Row API
    let count_row = sqlx::query(&count_query)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Count query failed: {}", e))?;

    let total_count: i64 = count_row
        .try_get("count")
        .map_err(|e| format!("Failed to extract count: {}", e))?;

    // Build select query
    let select_query = format!(
        "SELECT id, file_path, file_name, bpm, key_signature, tags, duration, track_count, file_size, created_at, updated_at FROM files WHERE 1=1{} ORDER BY id LIMIT {} OFFSET {}",
        where_clause, limit, offset
    );

    // Fetch rows using sqlx::query with Row API and map to tuple
    let rows = sqlx::query(&select_query)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Search query failed: {}", e))?;

    let files = rows
        .into_iter()
        .map(|row| {
            let tags_bytes: Vec<u8> = row.try_get("tags").unwrap_or_default();
            MidiFile {
                id: row.try_get("id").unwrap_or(0),
                file_path: row.try_get("file_path").unwrap_or_default(),
                file_name: row.try_get("file_name").unwrap_or_default(),
                bpm: row.try_get("bpm").unwrap_or(120.0),
                key_signature: row
                    .try_get("key_signature")
                    .unwrap_or_else(|_| "C Major".to_string()),
                tags: bincode::deserialize(&tags_bytes).unwrap_or_default(),
                duration: row.try_get("duration").unwrap_or(0.0),
                track_count: row.try_get("track_count").unwrap_or(1),
                file_size: row.try_get("file_size").unwrap_or(0),
                created_at: row.try_get("created_at").unwrap_or_else(|_| Utc::now()),
                updated_at: row.try_get("updated_at").unwrap_or_else(|_| Utc::now()),
            }
        })
        .collect();

    Ok(SearchResults { files, total_count })
}

#[command]
pub async fn database_get_file_metadata(
    state: State<'_, AppState>,
    id: i64,
) -> Result<Option<MidiFile>, String> {
    let pool = state.db_pool.as_ref().ok_or("Database not available".to_string())?;

    let row = sqlx::query_as::<_, (i64, String, String, f32, String, Vec<u8>, f32, i32, i64, DateTime<Utc>, DateTime<Utc>)>(
        "SELECT id, file_path, file_name, bpm, key_signature, tags, duration, track_count, file_size, created_at, updated_at FROM files WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Fetch failed: {}", e))?;

    if let Some((
        id,
        file_path,
        file_name,
        bpm,
        key_signature,
        tags_bytes,
        duration,
        track_count,
        file_size,
        created_at,
        updated_at,
    )) = row
    {
        let tags = bincode::deserialize(&tags_bytes).unwrap_or_default();
        Ok(Some(MidiFile {
            id,
            file_path,
            file_name,
            bpm,
            key_signature,
            tags,
            duration,
            track_count,
            file_size,
            created_at,
            updated_at,
        }))
    } else {
        Ok(None)
    }
}

#[allow(clippy::too_many_arguments)]
#[command]
pub async fn database_add_file(
    state: State<'_, AppState>,
    file_path: String,
    file_name: String,
    bpm: Option<f32>,
    key_signature: Option<String>,
    tags: Option<Vec<String>>,
    duration: Option<f32>,
    track_count: Option<i32>,
    file_size: Option<i64>,
) -> Result<i64, String> {
    let pool = state.db_pool.as_ref().ok_or("Database not available".to_string())?;

    let tags_bytes = bincode::serialize(&tags.unwrap_or_default())
        .map_err(|e| format!("Serialize error: {}", e))?;

    let (id,): (i64,) = sqlx::query_as(
        "INSERT INTO files (file_path, file_name, bpm, key_signature, tags, duration, track_count, file_size) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id"
    )
    .bind(file_path)
    .bind(file_name)
    .bind(bpm.unwrap_or(120.0))
    .bind(key_signature.unwrap_or("C Major".to_string()))
    .bind(tags_bytes)
    .bind(duration.unwrap_or(0.0))
    .bind(track_count.unwrap_or(1))
    .bind(file_size.unwrap_or(0))
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Insert failed: {}", e))?;

    info!("Database: Added file ID {}", id);
    Ok(id)
}

#[command]
pub async fn database_remove_file(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let pool = state.db_pool.as_ref().ok_or("Database not available".to_string())?;

    let result = sqlx::query("DELETE FROM files WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| format!("Delete failed: {}", e))?;

    if result.rows_affected() > 0 {
        info!("Database: Removed file ID {}", id);
        Ok(())
    } else {
        Err("File not found".to_string())
    }
}

#[command]
pub async fn database_get_stats(state: State<'_, AppState>) -> Result<DatabaseStats, String> {
    let pool = state.db_pool.as_ref().ok_or("Database not available".to_string())?;

    let row = sqlx::query("SELECT COUNT(*) as total_files, AVG(bpm) as avg_bpm, SUM(file_size) as total_size FROM files")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Stats query failed: {}", e))?;

    let total_files: i64 = row
        .try_get("total_files")
        .map_err(|_| "Total files column missing".to_string())?;
    let avg_bpm: Option<f32> =
        row.try_get("avg_bpm").map_err(|_| "Avg BPM column missing".to_string())?;
    let total_size: Option<i64> =
        row.try_get("total_size").map_err(|_| "Total size column missing".to_string())?;

    Ok(DatabaseStats {
        total_files,
        avg_bpm: avg_bpm.unwrap_or(120.0),
        total_size: total_size.unwrap_or(0),
    })
}

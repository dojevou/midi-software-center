///! Meilisearch Commands - Full-text search with Meilisearch
///!
///! Provides high-performance, typo-tolerant search as an alternative/complement
///! to the existing PostgreSQL-based search.

use crate::services::meilisearch_client::{
    MeilisearchClient, MidiSearchDocument, SearchFilters, SearchResult,
};
use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, error, info};

/// Initialize the Meilisearch index with optimal settings.
///
/// Should be called once when the application starts or when
/// the user enables Meilisearch search.
#[tauri::command]
pub async fn meilisearch_initialize(state: State<'_, AppState>) -> Result<(), String> {
    info!("Initializing Meilisearch index...");

    let meili_client = state
        .meilisearch
        .as_ref()
        .ok_or("Meilisearch client not configured")?;

    meili_client.initialize_index().await?;

    info!("Meilisearch index initialized successfully");
    Ok(())
}

/// Perform a full-text search using Meilisearch.
///
/// # Arguments
/// * `query` - Search query string
/// * `limit` - Maximum number of results (default: 20)
/// * `offset` - Number of results to skip (for pagination)
#[tauri::command]
pub async fn meilisearch_search(
    query: String,
    limit: Option<usize>,
    offset: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    debug!("Meilisearch search: query='{}', limit={:?}, offset={:?}", query, limit, offset);

    let meili_client = state
        .meilisearch
        .as_ref()
        .ok_or("Meilisearch client not configured")?;

    let results = meili_client
        .search(&query, limit, offset)
        .await?;

    debug!("Found {} results", results.len());
    Ok(results)
}

/// Perform a faceted search with filters using Meilisearch.
///
/// # Arguments
/// * `query` - Optional search query string
/// * `filters` - Filter criteria (instruments, tags, BPM, etc.)
/// * `limit` - Maximum number of results
/// * `offset` - Number of results to skip
#[tauri::command]
pub async fn meilisearch_faceted_search(
    query: Option<String>,
    filters: SearchFilters,
    limit: Option<usize>,
    offset: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    debug!(
        "Meilisearch faceted search: query={:?}, filters={:?}, limit={:?}, offset={:?}",
        query, filters, limit, offset
    );

    let meili_client = state
        .meilisearch
        .as_ref()
        .ok_or("Meilisearch client not configured")?;

    let results = meili_client
        .faceted_search(query.as_deref(), &filters, limit, offset)
        .await?;

    debug!("Found {} results", results.len());
    Ok(results)
}

/// Index a single file in Meilisearch.
///
/// Called automatically after file import or when re-indexing.
///
/// # Arguments
/// * `document` - The MIDI file document to index
#[tauri::command]
pub async fn meilisearch_index_file(
    document: MidiSearchDocument,
    state: State<'_, AppState>,
) -> Result<(), String> {
    debug!("Indexing file: {} (id: {})", document.filename, document.id);

    let meili_client = state
        .meilisearch
        .as_ref()
        .ok_or("Meilisearch client not configured")?;

    meili_client.index_file(&document).await?;

    Ok(())
}

/// Index multiple files in batch (more efficient).
///
/// # Arguments
/// * `documents` - Vector of documents to index
#[tauri::command]
pub async fn meilisearch_index_files_batch(
    documents: Vec<MidiSearchDocument>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Batch indexing {} files", documents.len());

    let meili_client = state
        .meilisearch
        .as_ref()
        .ok_or("Meilisearch client not configured")?;

    meili_client.index_files_batch(&documents).await?;

    info!("Batch indexing completed");
    Ok(())
}

/// Delete a file from the Meilisearch index.
///
/// # Arguments
/// * `file_id` - The ID of the file to delete
#[tauri::command]
pub async fn meilisearch_delete_file(
    file_id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    debug!("Deleting file from Meilisearch: {}", file_id);

    let meili_client = state
        .meilisearch
        .as_ref()
        .ok_or("Meilisearch client not configured")?;

    meili_client.delete_file(file_id).await?;

    Ok(())
}

/// Delete multiple files from the index in batch.
///
/// # Arguments
/// * `file_ids` - Vector of file IDs to delete
#[tauri::command]
pub async fn meilisearch_delete_files_batch(
    file_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Batch deleting {} files from Meilisearch", file_ids.len());

    let meili_client = state
        .meilisearch
        .as_ref()
        .ok_or("Meilisearch client not configured")?;

    meili_client.delete_files_batch(&file_ids).await?;

    Ok(())
}

/// Clear all documents from the Meilisearch index.
///
/// **Use with caution!** This will remove all indexed files.
#[tauri::command]
pub async fn meilisearch_clear_index(state: State<'_, AppState>) -> Result<(), String> {
    info!("Clearing Meilisearch index (user requested)");

    let meili_client = state
        .meilisearch
        .as_ref()
        .ok_or("Meilisearch client not configured")?;

    meili_client.clear_index().await?;

    Ok(())
}

/// Get Meilisearch index statistics.
///
/// Returns information about the number of documents, indexing status, etc.
#[tauri::command]
pub async fn meilisearch_get_stats(
    state: State<'_, AppState>,
) -> Result<std::collections::HashMap<String, serde_json::Value>, String> {
    debug!("Getting Meilisearch stats");

    let meili_client = state
        .meilisearch
        .as_ref()
        .ok_or("Meilisearch client not configured")?;

    let stats = meili_client.get_stats().await?;

    debug!("Meilisearch stats: {:?}", stats);
    Ok(stats)
}

/// Rebuild the entire Meilisearch index from the database.
///
/// This is a long-running operation that should be run in the background.
/// It fetches all files from the database and re-indexes them in Meilisearch.
#[tauri::command]
pub async fn meilisearch_rebuild_index(state: State<'_, AppState>) -> Result<String, String> {
    info!("Starting Meilisearch index rebuild...");

    let meili_client = state
        .meilisearch
        .as_ref()
        .ok_or("Meilisearch client not configured")?;

    // Clear existing index
    meili_client.clear_index().await?;

    // Re-initialize with settings
    meili_client.initialize_index().await?;

    // Fetch all files from database
    let pool = state.database.pool().await;

    let files: Vec<(i64, String, String, String)> = sqlx::query_as(
        r#"
        SELECT
            f.id,
            f.filename,
            f.original_filename,
            f.filepath
        FROM files f
        WHERE f.deleted_at IS NULL
        ORDER BY f.id
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to fetch files: {}", e))?;

    let total_files = files.len();
    info!("Rebuilding index for {} files", total_files);

    // Process in batches of 1000
    let batch_size = 1000;
    let mut indexed = 0;

    for chunk in files.chunks(batch_size) {
        let mut documents = Vec::new();

        for (file_id, filename, original_filename, filepath) in chunk {
            // Fetch additional metadata
            let metadata: Option<(
                Option<rust_decimal::Decimal>,
                Option<String>,
                Option<i16>,
                Option<i16>,
            )> = sqlx::query_as(
                r#"
                SELECT
                    mm.bpm,
                    mm.key_signature,
                    mm.time_signature_numerator,
                    mm.time_signature_denominator
                FROM musical_metadata mm
                WHERE mm.file_id = $1
                "#
            )
            .bind(file_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| format!("Failed to fetch metadata: {}", e))?;

            // Fetch tags with their categories
            let tags: Vec<(String, Option<String>)> = sqlx::query_as(
                r#"
                SELECT t.name, t.category
                FROM tags t
                INNER JOIN file_tags ft ON t.id = ft.tag_id
                WHERE ft.file_id = $1
                "#
            )
            .bind(file_id)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("Failed to fetch tags: {}", e))?;

            // Separate tags by category
            let mut tag_names: Vec<String> = Vec::new();
            let mut instruments: Vec<String> = Vec::new();
            let mut timbres: Vec<String> = Vec::new();
            let mut styles: Vec<String> = Vec::new();
            let mut articulations: Vec<String> = Vec::new();

            for (name, category) in tags {
                tag_names.push(name.clone());
                match category.as_deref() {
                    Some("instrument") => instruments.push(name),
                    Some("timbre") => timbres.push(name),
                    Some("style") => styles.push(name),
                    Some("articulation") => articulations.push(name),
                    _ => {} // Other categories go into general tags only
                }
            }

            // Build document
            let mut document = MidiSearchDocument {
                id: *file_id,
                filename: filename.clone(),
                original_filename: original_filename.clone(),
                filepath: filepath.clone(),
                tags: if tag_names.is_empty() { None } else { Some(tag_names) },
                instruments: if instruments.is_empty() { None } else { Some(instruments) },
                bpm: None,
                key_signature: None,
                time_signature: None,
                manufacturer: None,
                collection_name: None,
                duration_seconds: None,
                num_tracks: 0,
                is_multi_track: None,
                is_percussive: None,
                timbres: if timbres.is_empty() { None } else { Some(timbres) },
                styles: if styles.is_empty() { None } else { Some(styles) },
                articulations: if articulations.is_empty() { None } else { Some(articulations) },
            };

            // Add metadata if available
            if let Some((bpm, key_sig, ts_num, ts_denom)) = metadata {
                document.bpm = bpm.map(|b| b.to_string().parse::<f64>().unwrap_or(0.0));
                document.key_signature = key_sig;

                if let (Some(num), Some(denom)) = (ts_num, ts_denom) {
                    document.time_signature = Some(format!("{}/{}", num, denom));
                }
            }

            documents.push(document);
        }

        // Index this batch
        meili_client.index_files_batch(&documents).await?;
        indexed += documents.len();

        info!("Indexed {}/{} files", indexed, total_files);
    }

    let message = format!("Successfully rebuilt index with {} files", indexed);
    info!("{}", message);
    Ok(message)
}

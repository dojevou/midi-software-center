//! VIP3 Bulk Retag Command - High-performance batch tagging using Rust + Rayon
//!
//! Processes all files in the database and assigns VIP3 categories based on
//! filename/filepath analysis using Aho-Corasick multi-pattern matching.

use crate::core::analysis::Vip3Extractor;
use crate::AppState;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tauri::State;
use tracing::info;

/// File record for bulk processing
#[derive(Debug, Clone)]
struct FileRecord {
    id: i64,
    filename: String,
    filepath: String,
}

/// Category ID lookup maps
struct CategoryMaps {
    timbres: HashMap<String, i16>,
    styles: HashMap<String, i16>,
    articulations: HashMap<String, i16>,
}

/// Load category ID maps from database
async fn load_category_maps(pool: &PgPool) -> Result<CategoryMaps, sqlx::Error> {
    // Load timbres
    let timbre_rows: Vec<(i16, String)> =
        sqlx::query_as("SELECT id, name FROM timbres").fetch_all(pool).await?;
    let timbres: HashMap<String, i16> =
        timbre_rows.into_iter().map(|(id, name)| (name, id)).collect();

    // Load styles
    let style_rows: Vec<(i16, String)> =
        sqlx::query_as("SELECT id, name FROM styles").fetch_all(pool).await?;
    let styles: HashMap<String, i16> =
        style_rows.into_iter().map(|(id, name)| (name, id)).collect();

    // Load articulations
    let articulation_rows: Vec<(i16, String)> =
        sqlx::query_as("SELECT id, name FROM articulations").fetch_all(pool).await?;
    let articulations: HashMap<String, i16> =
        articulation_rows.into_iter().map(|(id, name)| (name, id)).collect();

    Ok(CategoryMaps { timbres, styles, articulations })
}

/// Fetch files in batches for processing
async fn fetch_file_batch(
    pool: &PgPool,
    offset: i64,
    limit: i64,
) -> Result<Vec<FileRecord>, sqlx::Error> {
    let rows: Vec<(i64, String, String)> = sqlx::query_as(
        r#"
        SELECT id, filename, filepath
        FROM files
        ORDER BY id
        OFFSET $1 LIMIT $2
        "#,
    )
    .bind(offset)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, filename, filepath)| FileRecord { id, filename, filepath })
        .collect())
}

/// Batch insert timbre assignments
async fn insert_timbres(pool: &PgPool, assignments: &[(i64, i16)]) -> Result<u64, sqlx::Error> {
    if assignments.is_empty() {
        return Ok(0);
    }

    // Use unnest for bulk insert
    let file_ids: Vec<i64> = assignments.iter().map(|(f, _)| *f).collect();
    let timbre_ids: Vec<i16> = assignments.iter().map(|(_, t)| *t).collect();

    let result = sqlx::query(
        r#"
        INSERT INTO midi_file_timbres (file_id, timbre_id)
        SELECT * FROM UNNEST($1::bigint[], $2::smallint[])
        ON CONFLICT (file_id, timbre_id) DO NOTHING
        "#,
    )
    .bind(&file_ids)
    .bind(&timbre_ids)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Batch insert style assignments
async fn insert_styles(pool: &PgPool, assignments: &[(i64, i16)]) -> Result<u64, sqlx::Error> {
    if assignments.is_empty() {
        return Ok(0);
    }

    let file_ids: Vec<i64> = assignments.iter().map(|(f, _)| *f).collect();
    let style_ids: Vec<i16> = assignments.iter().map(|(_, s)| *s).collect();

    let result = sqlx::query(
        r#"
        INSERT INTO midi_file_styles (file_id, style_id)
        SELECT * FROM UNNEST($1::bigint[], $2::smallint[])
        ON CONFLICT (file_id, style_id) DO NOTHING
        "#,
    )
    .bind(&file_ids)
    .bind(&style_ids)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Batch insert articulation assignments
async fn insert_articulations(
    pool: &PgPool,
    assignments: &[(i64, i16)],
) -> Result<u64, sqlx::Error> {
    if assignments.is_empty() {
        return Ok(0);
    }

    let file_ids: Vec<i64> = assignments.iter().map(|(f, _)| *f).collect();
    let articulation_ids: Vec<i16> = assignments.iter().map(|(_, a)| *a).collect();

    let result = sqlx::query(
        r#"
        INSERT INTO midi_file_articulations (file_id, articulation_id)
        SELECT * FROM UNNEST($1::bigint[], $2::smallint[])
        ON CONFLICT (file_id, articulation_id) DO NOTHING
        "#,
    )
    .bind(&file_ids)
    .bind(&articulation_ids)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Response for bulk retag operation
#[derive(Debug, Clone, serde::Serialize)]
pub struct BulkRetagResponse {
    pub files_processed: usize,
    pub timbres_added: usize,
    pub styles_added: usize,
    pub articulations_added: usize,
    pub total_tags_added: usize,
    pub elapsed_seconds: f64,
    pub files_per_second: f64,
}

/// Bulk retag all files with VIP3 categories
///
/// This command processes all files in the database, extracting VIP3 categories
/// from filenames/paths using high-performance Aho-Corasick matching and Rayon
/// parallel processing.
#[tauri::command]
pub async fn bulk_retag_vip3(
    batch_size: Option<i64>,
    dry_run: Option<bool>,
    state: State<'_, AppState>,
) -> Result<BulkRetagResponse, String> {
    let start = Instant::now();
    let pool = state.database.pool().await;
    let batch_size = batch_size.unwrap_or(10_000);
    let dry_run = dry_run.unwrap_or(false);

    info!(
        "Starting VIP3 bulk retag (batch_size={}, dry_run={})",
        batch_size, dry_run
    );

    // Load category maps
    let category_maps = load_category_maps(&pool)
        .await
        .map_err(|e| format!("Failed to load category maps: {}", e))?;

    info!(
        "Loaded {} timbres, {} styles, {} articulations",
        category_maps.timbres.len(),
        category_maps.styles.len(),
        category_maps.articulations.len()
    );

    // Create extractor (builds Aho-Corasick automata)
    let extractor = Arc::new(Vip3Extractor::new());

    // Get total file count
    let total_files: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM files")
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("Failed to count files: {}", e))?;
    let total_files = total_files.0;

    info!("Processing {} files", total_files);

    let mut files_processed = 0usize;
    let mut timbres_added = 0usize;
    let mut styles_added = 0usize;
    let mut articulations_added = 0usize;
    let mut offset = 0i64;

    while offset < total_files {
        // Fetch batch
        let files = fetch_file_batch(&pool, offset, batch_size)
            .await
            .map_err(|e| format!("Failed to fetch files: {}", e))?;

        if files.is_empty() {
            break;
        }

        let batch_len = files.len();

        // Convert to format expected by extractor
        let file_tuples: Vec<(i64, String, String)> =
            files.iter().map(|f| (f.id, f.filename.clone(), f.filepath.clone())).collect();

        // Extract categories in parallel
        let results = extractor.extract_batch(&file_tuples);

        // Collect assignments
        let mut timbre_assignments: Vec<(i64, i16)> = Vec::new();
        let mut style_assignments: Vec<(i64, i16)> = Vec::new();
        let mut articulation_assignments: Vec<(i64, i16)> = Vec::new();

        for (file_id, categories) in &results {
            for timbre in &categories.timbres {
                if let Some(&timbre_id) = category_maps.timbres.get(timbre) {
                    timbre_assignments.push((*file_id, timbre_id));
                }
            }
            for style in &categories.styles {
                if let Some(&style_id) = category_maps.styles.get(style) {
                    style_assignments.push((*file_id, style_id));
                }
            }
            for articulation in &categories.articulations {
                if let Some(&articulation_id) = category_maps.articulations.get(articulation) {
                    articulation_assignments.push((*file_id, articulation_id));
                }
            }
        }

        // Insert if not dry run
        if !dry_run {
            let t = insert_timbres(&pool, &timbre_assignments)
                .await
                .map_err(|e| format!("Failed to insert timbres: {}", e))?;
            let s = insert_styles(&pool, &style_assignments)
                .await
                .map_err(|e| format!("Failed to insert styles: {}", e))?;
            let a = insert_articulations(&pool, &articulation_assignments)
                .await
                .map_err(|e| format!("Failed to insert articulations: {}", e))?;

            timbres_added += t as usize;
            styles_added += s as usize;
            articulations_added += a as usize;
        } else {
            timbres_added += timbre_assignments.len();
            styles_added += style_assignments.len();
            articulations_added += articulation_assignments.len();
        }

        files_processed += batch_len;
        offset += batch_size;

        // Progress log every 100k files
        if files_processed % 100_000 < batch_len {
            let elapsed = start.elapsed().as_secs_f64();
            let rate = files_processed as f64 / elapsed;
            info!(
                "Progress: {}/{} files ({:.1}%), {:.0} files/sec",
                files_processed,
                total_files,
                (files_processed as f64 / total_files as f64) * 100.0,
                rate
            );
        }
    }

    let elapsed = start.elapsed();
    let elapsed_seconds = elapsed.as_secs_f64();
    let files_per_second = files_processed as f64 / elapsed_seconds;

    info!(
        "Bulk retag complete: {} files, {} timbres, {} styles, {} articulations in {:.2}s ({:.0} files/sec)",
        files_processed, timbres_added, styles_added, articulations_added, elapsed_seconds, files_per_second
    );

    if dry_run {
        info!("[DRY RUN - No changes written to database]");
    }

    Ok(BulkRetagResponse {
        files_processed,
        timbres_added,
        styles_added,
        articulations_added,
        total_tags_added: timbres_added + styles_added + articulations_added,
        elapsed_seconds,
        files_per_second,
    })
}

/// Update file counts in category tables after bulk retag
#[tauri::command]
pub async fn update_vip3_counts(state: State<'_, AppState>) -> Result<(), String> {
    let pool = state.database.pool().await;

    info!("Updating VIP3 category counts...");

    // Update timbre counts
    sqlx::query(
        r#"
        UPDATE timbres t
        SET file_count = (
            SELECT COUNT(*) FROM midi_file_timbres mft WHERE mft.timbre_id = t.id
        )
        "#,
    )
    .execute(&pool)
    .await
    .map_err(|e| format!("Failed to update timbre counts: {}", e))?;

    // Update style counts
    sqlx::query(
        r#"
        UPDATE styles s
        SET file_count = (
            SELECT COUNT(*) FROM midi_file_styles mfs WHERE mfs.style_id = s.id
        )
        "#,
    )
    .execute(&pool)
    .await
    .map_err(|e| format!("Failed to update style counts: {}", e))?;

    // Update articulation counts
    sqlx::query(
        r#"
        UPDATE articulations a
        SET file_count = (
            SELECT COUNT(*) FROM midi_file_articulations mfa WHERE mfa.articulation_id = a.id
        )
        "#,
    )
    .execute(&pool)
    .await
    .map_err(|e| format!("Failed to update articulation counts: {}", e))?;

    info!("VIP3 category counts updated");

    Ok(())
}

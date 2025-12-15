//! File Import Commands - High-performance parallel MIDI file importing
//!
//! Optimizations: BLAKE3 hashing, parallel processing, batch DB inserts, dynamic concurrency

mod discovery;
mod gm_instruments;
mod midi_helpers;
mod types;

pub use types::{FileMetadata, ImportProgress, ImportSummary};

use crate::core::analysis::auto_tagger::AutoTagger;
use crate::core::analysis::bpm_detector::detect_bpm;
use crate::core::analysis::key_detector::detect_key;
use crate::core::analysis::FilenameMetadata;
use crate::core::hash::calculate_file_hash;
use crate::core::midi::parse_midi_file;
use crate::core::midi::text_metadata::TextMetadata;
use crate::core::naming::generator::generate_production_filename;
use crate::core::performance::concurrency::{
    calculate_optimal_concurrency, detect_system_resources,
};
use crate::database::batch_insert::BatchInserter;
use crate::AppState;
use discovery::{find_midi_files_recursive, find_midi_files_shallow, is_midi_file};
use midi_helpers::{extract_instrument_names, extract_time_signature};
use types::{DeduplicationResult, ProcessedFile};

use futures::stream::{self, StreamExt};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tauri::{Emitter, State, Window};
use tokio::sync::Mutex;

// Configuration constants
const HASH_CONCURRENCY: usize = 64;
const BATCH_INSERT_SIZE: usize = 1000;
const BATCH_FLUSH_THRESHOLD: usize = 100;
const DB_QUERY_CHUNK_SIZE: usize = 10000;

/// Import a single MIDI file (implementation for tests and reuse)
pub async fn import_single_file_impl(
    file_path: String,
    category: Option<String>,
    state: &AppState,
) -> Result<FileMetadata, String> {
    let path = Path::new(&file_path);

    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    if !is_midi_file(path) {
        return Err("Not a MIDI file".to_string());
    }

    let processed = process_single_file(path, category)
        .await
        .map_err(|e| format!("Failed to process file: {}", e))?;

    let pool = state.database.pool().await;
    let file_id = insert_single_file(&processed, &pool)
        .await
        .map_err(|e| format!("Failed to insert file: {}", e))?;

    fetch_file_metadata(file_id, &pool).await
}

/// Import a single MIDI file
#[tauri::command]
pub async fn import_single_file(
    file_path: String,
    category: Option<String>,
    state: State<'_, AppState>,
    window: Window,
) -> Result<FileMetadata, String> {
    let file = import_single_file_impl(file_path, category, &state).await?;

    let _ = window.emit(
        "import-progress",
        ImportProgress { current: 1, total: 1, current_file: file.filename.clone(), rate: 1.0 },
    );

    Ok(file)
}

/// Import all MIDI files from a directory (implementation for tests and reuse)
pub async fn import_directory_impl(
    directory_path: String,
    recursive: bool,
    category: Option<String>,
    state: &AppState,
) -> Result<ImportSummary, String> {
    let start_time = std::time::Instant::now();
    let path = Path::new(&directory_path);

    if !path.exists() {
        return Err(format!("Directory not found: {}", directory_path));
    }

    let files = collect_midi_files(path, recursive)?;
    let total = files.len();

    if total == 0 {
        return Ok(ImportSummary::empty());
    }

    let pool = state.database.pool().await;
    let dedup = deduplicate_files(&files, &pool).await?;

    log_deduplication_stats(total, &dedup);

    if dedup.files_to_process.is_empty() {
        return Ok(ImportSummary::all_duplicates(
            total,
            dedup.duplicates_found,
            dedup.elapsed_secs,
        ));
    }

    let resources = detect_system_resources();
    let concurrency_limit = calculate_optimal_concurrency(&resources);
    log_system_resources(&resources, concurrency_limit);

    let stats = process_files_parallel(
        dedup.files_to_process,
        category,
        dedup.duplicates_found,
        concurrency_limit,
        pool,
    )
    .await;

    let duration = start_time.elapsed().as_secs_f64();
    let rate = if duration > 0.0 {
        stats.imported as f64 / duration
    } else {
        0.0
    };

    Ok(ImportSummary {
        total_files: total,
        imported: stats.imported,
        skipped: stats.skipped,
        errors: stats.errors,
        duration_secs: duration,
        rate,
    })
}

/// Import all MIDI files from a directory
#[tauri::command]
pub async fn import_directory(
    directory_path: String,
    recursive: bool,
    category: Option<String>,
    state: State<'_, AppState>,
    _window: Window,
) -> Result<ImportSummary, String> {
    import_directory_impl(directory_path, recursive, category, &state).await
}

// ============================================================================
// Internal helpers
// ============================================================================

fn collect_midi_files(path: &Path, recursive: bool) -> Result<Vec<PathBuf>, String> {
    if recursive {
        find_midi_files_recursive(path)
    } else {
        find_midi_files_shallow(path)
    }
    .map_err(|e| format!("Error scanning directory: {}", e))
}

fn log_deduplication_stats(total: usize, dedup: &DeduplicationResult) {
    println!("🔍 Pre-scanning {} files for duplicates...", total);
    println!("✓ Deduplication complete in {:.2}s", dedup.elapsed_secs);
    println!("  Total files: {}", total);
    println!("  Duplicates skipped: {}", dedup.duplicates_found);
    println!("  Hash errors: {}", dedup.hash_errors);
    println!("  New files to process: {}", dedup.files_to_process.len());
}

fn log_system_resources(
    resources: &crate::core::performance::concurrency::SystemResources,
    concurrency: usize,
) {
    println!("🚀 System resources detected:");
    println!("  CPU cores: {}", resources.cpu_cores);
    println!(
        "  Available memory: {:.2} GB",
        resources.available_memory_gb
    );
    println!("  Optimal concurrency: {}", concurrency);
}

struct ProcessingStats {
    imported: usize,
    skipped: usize,
    errors: Vec<String>,
}

async fn process_files_parallel(
    files: Vec<PathBuf>,
    category: Option<String>,
    initial_skipped: usize,
    concurrency_limit: usize,
    pool: sqlx::PgPool,
) -> ProcessingStats {
    let imported = Arc::new(AtomicUsize::new(0));
    let skipped = Arc::new(AtomicUsize::new(initial_skipped));
    let errors = Arc::new(Mutex::new(Vec::new()));
    let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency_limit));
    let batch_inserter = Arc::new(BatchInserter::new(pool, BATCH_INSERT_SIZE));
    let processed_files = Arc::new(Mutex::new(Vec::new()));

    stream::iter(files)
        .map(|file_path| {
            let sem = Arc::clone(&semaphore);
            let category = category.clone();
            let imported = Arc::clone(&imported);
            let skipped = Arc::clone(&skipped);
            let errors = Arc::clone(&errors);
            let processed_files = Arc::clone(&processed_files);
            let batch_inserter = Arc::clone(&batch_inserter);

            async move {
                let _permit = match sem.acquire().await {
                    Ok(p) => p,
                    Err(e) => {
                        errors.lock().await.push(format!("Semaphore error: {}", e));
                        skipped.fetch_add(1, Ordering::SeqCst);
                        return;
                    },
                };

                match process_single_file(&file_path, category).await {
                    Ok(processed) => {
                        imported.fetch_add(1, Ordering::SeqCst);
                        let mut files = processed_files.lock().await;
                        files.push(processed);

                        if files.len() >= BATCH_FLUSH_THRESHOLD {
                            let batch: Vec<ProcessedFile> = files.drain(..).collect();
                            drop(files);
                            flush_batch(&batch, &batch_inserter, &errors, &skipped).await;
                        }
                    },
                    Err(e) => {
                        errors.lock().await.push(format!("{}: {}", file_path.display(), e));
                        skipped.fetch_add(1, Ordering::SeqCst);
                    },
                }
            }
        })
        .buffer_unordered(concurrency_limit)
        .collect::<Vec<_>>()
        .await;

    // Flush remaining
    let remaining = processed_files.lock().await;
    if !remaining.is_empty() {
        let batch: Vec<ProcessedFile> = remaining.iter().cloned().collect();
        drop(remaining);
        flush_batch(&batch, &batch_inserter, &errors, &skipped).await;
    }

    let error_list = errors.lock().await.clone();
    ProcessingStats {
        imported: imported.load(Ordering::SeqCst),
        skipped: skipped.load(Ordering::SeqCst),
        errors: error_list,
    }
}

async fn flush_batch(
    batch: &[ProcessedFile],
    inserter: &BatchInserter,
    errors: &Mutex<Vec<String>>,
    skipped: &AtomicUsize,
) {
    let records = to_file_records(batch);
    if let Err(e) = inserter.insert_files_batch(records).await {
        errors.lock().await.push(format!("Batch insert failed: {}", e));
        skipped.fetch_add(batch.len(), Ordering::SeqCst);
    }
}

async fn process_single_file(
    file_path: &Path,
    category: Option<String>,
) -> Result<ProcessedFile, Box<dyn std::error::Error + Send + Sync>> {
    let hash_bytes = calculate_file_hash(file_path)?;
    let content_hash: Vec<u8> = hash_bytes.to_vec();
    let file_bytes = tokio::fs::read(file_path).await?;
    let midi_file = parse_midi_file(&file_bytes)?;

    let parent_folder = file_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|s| s.to_string());

    let bpm_result = detect_bpm(&midi_file);
    let bpm = (bpm_result.confidence > 0.5).then_some(bpm_result.bpm);

    let key_result = detect_key(&midi_file);
    let key_signature = (key_result.confidence > 0.5).then(|| key_result.key.clone());

    let text_meta = TextMetadata::extract(&midi_file);

    let original_filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid filename")?
        .to_string();

    let filepath = file_path.to_str().ok_or("Invalid file path")?.to_string();

    let pack_name = parent_folder.clone().unwrap_or_else(|| "Unknown".to_string());
    let time_signature = extract_time_signature(&midi_file).unwrap_or_else(|| "4-4".to_string());
    let original_name_clean =
        original_filename.trim_end_matches(".mid").trim_end_matches(".MID").to_string();
    let detected_category = category.clone().unwrap_or_else(|| "MIDI".to_string());

    let filename = generate_production_filename(
        &detected_category,
        bpm.unwrap_or(120.0),
        &key_signature.clone().unwrap_or_else(|| "C".to_string()),
        "000000",
        &time_signature,
        &pack_name,
        &original_name_clean,
    );

    let file_size_bytes = tokio::fs::metadata(file_path).await?.len() as i64;
    let midi_instruments = extract_instrument_names(&midi_file);

    let auto_tagger =
        AutoTagger::new().map_err(|e| format!("Failed to initialize auto-tagger: {}", e))?;
    let tags = auto_tagger.extract_tags(
        &filepath,
        &filename,
        &midi_instruments,
        bpm,
        key_signature.as_deref(),
        Some(&midi_file),
    );

    let filename_meta = FilenameMetadata::extract_from_filename(&filename);

    Ok(ProcessedFile {
        filename,
        original_filename,
        filepath,
        parent_folder,
        content_hash,
        file_size_bytes,
        category,
        bpm,
        key_signature,
        tags,
        filename_bpm: filename_meta.bpm.map(|v| v as f32),
        filename_key: filename_meta.key,
        filename_genres: filename_meta.genres,
        structure_tags: filename_meta.structure_tags,
        track_number: filename_meta.track_number,
        track_names: text_meta.track_names,
        copyright: text_meta.copyright,
        instrument_names_text: text_meta.instrument_names,
        markers: text_meta.markers,
        lyrics: text_meta.lyrics,
    })
}

async fn fetch_file_metadata(file_id: i64, pool: &sqlx::PgPool) -> Result<FileMetadata, String> {
    sqlx::query_as::<_, FileMetadata>(
        r#"
        SELECT
            f.id, f.filename, f.original_filename, f.filepath,
            encode(f.content_hash, 'hex') as content_hash_hex,
            f.file_size_bytes, m.bpm, m.key_signature::text as key_signature
        FROM files f
        LEFT JOIN musical_metadata m ON f.id = m.file_id
        WHERE f.id = $1
        "#,
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Failed to retrieve file: {}", e))
}

async fn insert_single_file(
    file: &ProcessedFile,
    pool: &sqlx::PgPool,
) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
    let mut tx = pool.begin().await?;

    let metadata_source = match (&file.bpm, &file.filename_bpm) {
        (Some(_), Some(_)) => "both",
        (Some(_), None) => "analyzed",
        (None, Some(_)) => "filename",
        (None, None) => "none",
    };

    let file_id_opt = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO files (
            filename, original_filename, filepath, content_hash, file_size_bytes,
            num_tracks, filename_bpm, filename_key, filename_genres, structure_tags,
            track_number, metadata_source, track_names, copyright, instrument_names_text,
            markers, lyrics, created_at
        ) VALUES ($1, $2, $3, $4, $5, 1, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, NOW())
        ON CONFLICT (content_hash) DO NOTHING
        RETURNING id
        "#,
    )
    .bind(&file.filename)
    .bind(&file.original_filename)
    .bind(&file.filepath)
    .bind(&file.content_hash)
    .bind(file.file_size_bytes)
    .bind(file.filename_bpm)
    .bind(file.filename_key.as_ref())
    .bind(&file.filename_genres)
    .bind(&file.structure_tags)
    .bind(file.track_number.map(|n| n as i32))
    .bind(metadata_source)
    .bind(&file.track_names)
    .bind(file.copyright.as_ref())
    .bind(&file.instrument_names_text)
    .bind(&file.markers)
    .bind(&file.lyrics)
    .fetch_optional(&mut *tx)
    .await?;

    let file_id = match file_id_opt {
        Some(id) => id,
        None => {
            tx.rollback().await?;
            return Err("File already exists (duplicate hash)".into());
        },
    };

    insert_musical_metadata(&mut tx, file_id, file).await?;
    insert_category(&mut tx, file_id, &file.category).await?;
    insert_tags(&mut tx, file_id, &file.tags).await?;

    tx.commit().await?;
    Ok(file_id)
}

async fn insert_musical_metadata(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    file_id: i64,
    file: &ProcessedFile,
) -> Result<(), sqlx::Error> {
    if file.bpm.is_some() || file.key_signature.is_some() {
        sqlx::query(
            r#"
            INSERT INTO musical_metadata (file_id, bpm, key_signature, time_signature_numerator, time_signature_denominator)
            VALUES ($1, $2, $3::musical_key, 4, 4)
            ON CONFLICT (file_id) DO UPDATE SET bpm = EXCLUDED.bpm, key_signature = EXCLUDED.key_signature
            "#,
        )
        .bind(file_id)
        .bind(file.bpm)
        .bind(file.key_signature.as_deref())
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn insert_category(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    file_id: i64,
    category: &Option<String>,
) -> Result<(), sqlx::Error> {
    if let Some(ref category_name) = category {
        let category_id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO categories (name, created_at) VALUES ($1, NOW())
            ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
            RETURNING id
            "#,
        )
        .bind(category_name)
        .fetch_one(&mut **tx)
        .await?;

        sqlx::query("INSERT INTO file_categories (file_id, category_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(file_id)
            .bind(category_id)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

async fn insert_tags(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    file_id: i64,
    tags: &[crate::core::analysis::auto_tagger::Tag],
) -> Result<(), sqlx::Error> {
    for tag in tags {
        let tag_id = sqlx::query_scalar::<_, i32>(
            r#"
            INSERT INTO tags (name, category, usage_count, created_at) VALUES ($1, $2, 0, NOW())
            ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
            RETURNING id
            "#,
        )
        .bind(&tag.name)
        .bind(tag.category.as_deref())
        .fetch_one(&mut **tx)
        .await?;

        sqlx::query(
            "INSERT INTO file_tags (file_id, tag_id, added_at, added_by) VALUES ($1, $2, NOW(), 'system') ON CONFLICT DO NOTHING",
        )
        .bind(file_id)
        .bind(tag_id)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

fn to_file_records(files: &[ProcessedFile]) -> Vec<crate::database::batch_insert::FileRecord> {
    files
        .iter()
        .map(|f| {
            crate::database::batch_insert::FileRecord::new(
                f.filename.clone(),
                f.original_filename.clone(),
                f.filepath.clone(),
                f.parent_folder.clone(),
                hex::encode(&f.content_hash),
                f.file_size_bytes,
                f.category.clone(),
            )
        })
        .collect()
}

async fn deduplicate_files(
    files: &[PathBuf],
    pool: &sqlx::PgPool,
) -> Result<DeduplicationResult, String> {
    let total = files.len();
    let start = std::time::Instant::now();

    let hash_concurrency = std::cmp::min(total, HASH_CONCURRENCY);
    let semaphore = Arc::new(tokio::sync::Semaphore::new(hash_concurrency));

    let hash_results: Vec<(PathBuf, Option<Vec<u8>>)> = stream::iter(files.iter().cloned())
        .map(|path| {
            let sem = Arc::clone(&semaphore);
            async move {
                let _permit = sem.acquire().await.ok()?;
                let hash = calculate_file_hash(&path).ok().map(|h| h.to_vec());
                Some((path, hash))
            }
        })
        .buffer_unordered(hash_concurrency)
        .filter_map(|x| async { x })
        .collect()
        .await;

    let mut file_to_hash = HashMap::new();
    let mut hash_error_count = 0;

    for (path, hash_opt) in hash_results {
        match hash_opt {
            Some(hash) => {
                file_to_hash.insert(path, hash);
            },
            None => hash_error_count += 1,
        }
    }

    let hashes: Vec<Vec<u8>> = file_to_hash.values().cloned().collect();
    let existing: HashSet<Vec<u8>> = query_existing_hashes(&hashes, pool).await?;

    let files_to_process: Vec<PathBuf> = file_to_hash
        .into_iter()
        .filter(|(_, hash)| !existing.contains(hash))
        .map(|(path, _)| path)
        .collect();

    let duplicates_found = total - files_to_process.len() - hash_error_count;

    Ok(DeduplicationResult {
        files_to_process,
        duplicates_found,
        hash_errors: hash_error_count,
        elapsed_secs: start.elapsed().as_secs_f64(),
    })
}

async fn query_existing_hashes(
    hashes: &[Vec<u8>],
    pool: &sqlx::PgPool,
) -> Result<HashSet<Vec<u8>>, String> {
    if hashes.is_empty() {
        return Ok(HashSet::new());
    }

    let mut result = HashSet::new();
    for chunk in hashes.chunks(DB_QUERY_CHUNK_SIZE) {
        let found: Vec<Vec<u8>> =
            sqlx::query_scalar("SELECT content_hash FROM files WHERE content_hash = ANY($1)")
                .bind(chunk)
                .fetch_all(pool)
                .await
                .map_err(|e| format!("Database query failed: {}", e))?;
        result.extend(found);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_auto_tagging_import() {
        println!("\n🧪 Starting auto-tagging integration test...");

        let database_url = "postgresql://midiuser:145278963@localhost:5433/midi_library";
        let pool = sqlx::PgPool::connect(database_url)
            .await
            .expect("Failed to connect to database");

        let test_file_path =
            std::path::Path::new("/tmp/midi_test_import/Vengeance_Deep_House_Kick_128_C.mid");
        if !test_file_path.exists() {
            panic!("Test file not found: {:?}", test_file_path);
        }

        let processed =
            process_single_file(test_file_path, Some("test".to_string())).await.unwrap();

        println!("Tags extracted: {}", processed.tags.len());
        for tag in &processed.tags {
            match &tag.category {
                Some(cat) => println!("  - {}:{}", cat, tag.name),
                None => println!("  - {}", tag.name),
            }
        }

        let file_id = insert_single_file(&processed, &pool).await.unwrap();

        // Cleanup
        sqlx::query("DELETE FROM files WHERE id = $1")
            .bind(file_id)
            .execute(&pool)
            .await
            .expect("Failed to cleanup");
    }
}

// pipeline/src-tauri/src/core/pipeline/workers/import.rs
//! Stage 1: Import workers - Extract, hash, parse, dedupe, insert to DB

use crate::core::pipeline::queues::{FileRecord, PipelineQueues};
use crate::error::PipelineError;
use sqlx::PgPool;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{debug, info, warn};

/// Import worker configuration
pub struct ImportWorkerConfig {
    pub source_path: PathBuf,
    pub db_pool: PgPool,
    pub output_queue: Arc<PipelineQueues>,
    pub running: Arc<AtomicBool>,
    pub counter: Arc<AtomicU64>,
    pub worker_count: usize,
}

/// Import worker - handles Stage 1 processing
pub struct ImportWorker {
    _config: ImportWorkerConfig,
}

impl ImportWorker {
    /// Create new import worker
    pub fn new(config: ImportWorkerConfig) -> Self {
        Self { _config: config }
    }

    /// Spawn import worker threads
    pub async fn spawn_workers(config: ImportWorkerConfig) -> Result<(), PipelineError> {
        info!("Spawning {} import workers", config.worker_count);

        // Spawn worker tasks
        for worker_id in 0..config.worker_count {
            let source_path = config.source_path.clone();
            let db_pool = config.db_pool.clone();
            let output_queue = Arc::clone(&config.output_queue);
            let running = Arc::clone(&config.running);
            let counter = Arc::clone(&config.counter);

            tokio::spawn(async move {
                Self::worker_loop(
                    worker_id,
                    source_path,
                    db_pool,
                    output_queue,
                    running,
                    counter,
                )
                .await;
            });
        }

        Ok(())
    }

    /// Main worker loop - processes files from source
    async fn worker_loop(
        worker_id: usize,
        source_path: PathBuf,
        db_pool: PgPool,
        output_queue: Arc<PipelineQueues>,
        running: Arc<AtomicBool>,
        counter: Arc<AtomicU64>,
    ) {
        debug!("Import worker {} started", worker_id);

        // Walk directory using walkdir (single-threaded per worker for simplicity)
        use walkdir::WalkDir;

        for entry in WalkDir::new(&source_path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                // Skip *_splits directories created by split workers
                if let Some(name) = e.file_name().to_str() {
                    !name.ends_with("_splits")
                } else {
                    true
                }
            })
            .filter_map(|e| e.ok())
        {
            if !running.load(Ordering::Acquire) {
                break;
            }

            let path = entry.path();

            // Skip directories and non-MIDI files
            if !path.is_file() {
                continue;
            }

            let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if extension != "mid" && extension != "midi" {
                continue;
            }

            // Process file
            match Self::process_file(path.to_path_buf(), &db_pool).await {
                Ok(file_record) => {
                    // Push to sanitize queue
                    if output_queue.import_to_sanitize.push(file_record).is_err() {
                        warn!(
                            "Import worker {}: sanitize queue full, waiting...",
                            worker_id
                        );
                        sleep(Duration::from_millis(50)).await;
                    }
                    counter.fetch_add(1, Ordering::AcqRel);
                },
                Err(e) => {
                    warn!(
                        "Import worker {}: failed to process {:?}: {}",
                        worker_id, path, e
                    );
                },
            }
        }

        debug!("Import worker {} stopped", worker_id);
    }

    /// Process a single file through import stage
    async fn process_file(
        file_path: PathBuf,
        db_pool: &PgPool,
    ) -> Result<FileRecord, PipelineError> {
        use crate::core::hash::calculate_file_hash;
        use crate::core::midi::parse_midi_file;

        // Read file bytes
        let bytes = tokio::fs::read(&file_path).await?;

        // Calculate hash (pass file path)
        let hash = calculate_file_hash(&file_path)
            .map_err(|e| PipelineError::GeneralError(format!("Hash calculation failed: {}", e)))?;

        // Check for duplicate
        let existing: Option<i64> =
            sqlx::query_scalar("SELECT id FROM files WHERE content_hash = $1")
                .bind(hash)
                .fetch_optional(db_pool)
                .await?;

        if let Some(file_id) = existing {
            // Duplicate found, fetch record
            return Self::fetch_file_record(file_id, db_pool).await;
        }

        // Parse MIDI to detect multi-track
        let midi = parse_midi_file(&bytes).map_err(|e| PipelineError::MidiError(e.to_string()))?;
        let is_multi_track = midi.tracks.len() > 1;

        // Extract metadata for insert
        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.mid")
            .to_string();
        let filepath = file_path.to_string_lossy().to_string();
        let parent_folder = file_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|s| s.to_string());

        // Insert into database
        let file_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO files (filename, original_filename, filepath, parent_folder, content_hash, file_size_bytes)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#
        )
        .bind(&filename)
        .bind(&filename)
        .bind(&filepath)
        .bind(&parent_folder)
        .bind(hash)
        .bind(bytes.len() as i64)
        .fetch_one(db_pool)
        .await?;

        Ok(FileRecord {
            id: file_id,
            filepath: filepath.clone(),
            filename: filename.clone(),
            parent_folder,
            is_multi_track,
            analyzed: false,
        })
    }

    /// Fetch existing file record from database
    async fn fetch_file_record(
        file_id: i64,
        db_pool: &PgPool,
    ) -> Result<FileRecord, PipelineError> {
        use crate::core::midi::parse_midi_file;

        let record = sqlx::query_as::<_, (i64, String, String, Option<String>)>(
            "SELECT id, filepath, filename, parent_folder FROM files WHERE id = $1",
        )
        .bind(file_id)
        .fetch_one(db_pool)
        .await?;

        // Parse MIDI to detect multi-track
        let bytes = tokio::fs::read(&record.1).await?;
        let midi = parse_midi_file(&bytes).map_err(|e| PipelineError::MidiError(e.to_string()))?;
        let is_multi_track = midi.tracks.len() > 1;

        Ok(FileRecord {
            id: record.0,
            filepath: record.1,
            filename: record.2,
            parent_folder: record.3,
            is_multi_track,
            analyzed: false,
        })
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_import_worker_creation() {
        // TODO: Add tests for import worker
    }
}

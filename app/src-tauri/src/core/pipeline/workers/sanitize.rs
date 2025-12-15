// pipeline/src-tauri/src/core/pipeline/workers/sanitize.rs
//! Stage 2: Sanitize workers - Clean filenames (spaces→_, .midi→.mid, remove special chars)

use crate::core::pipeline::queues::{FileRecord, PipelineQueues};
use crate::error::PipelineError;
use sqlx::PgPool;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{debug, info};

/// Sanitize worker configuration
pub struct SanitizeWorkerConfig {
    pub db_pool: PgPool,
    pub input_queue: Arc<PipelineQueues>,
    pub output_queue: Arc<PipelineQueues>,
    pub running: Arc<AtomicBool>,
    pub counter: Arc<AtomicU64>,
    pub worker_count: usize,
}

/// Sanitize worker - handles Stage 2 processing
pub struct SanitizeWorker {
    _config: SanitizeWorkerConfig,
}

impl SanitizeWorker {
    pub fn new(config: SanitizeWorkerConfig) -> Self {
        Self { _config: config }
    }

    pub async fn spawn_workers(config: SanitizeWorkerConfig) -> Result<(), PipelineError> {
        info!("Spawning {} sanitize workers", config.worker_count);

        for worker_id in 0..config.worker_count {
            let db_pool = config.db_pool.clone();
            let input_queue = Arc::clone(&config.input_queue);
            let output_queue = Arc::clone(&config.output_queue);
            let running = Arc::clone(&config.running);
            let counter = Arc::clone(&config.counter);

            tokio::spawn(async move {
                Self::worker_loop(
                    worker_id,
                    db_pool,
                    input_queue,
                    output_queue,
                    running,
                    counter,
                )
                .await;
            });
        }

        Ok(())
    }

    async fn worker_loop(
        worker_id: usize,
        db_pool: PgPool,
        input_queue: Arc<PipelineQueues>,
        output_queue: Arc<PipelineQueues>,
        running: Arc<AtomicBool>,
        counter: Arc<AtomicU64>,
    ) {
        debug!("Sanitize worker {} started", worker_id);

        while running.load(Ordering::Acquire) {
            // Pop from import_to_sanitize queue
            if let Some(mut file_record) = input_queue.import_to_sanitize.pop() {
                // Process sanitization
                match Self::sanitize_file(&mut file_record, &db_pool).await {
                    Ok(_) => {
                        // Push to split queue
                        if output_queue.sanitize_to_split.push(file_record).is_err() {
                            debug!("Sanitize worker {}: split queue full", worker_id);
                            sleep(Duration::from_millis(10)).await;
                        }
                        counter.fetch_add(1, Ordering::AcqRel);
                    },
                    Err(e) => {
                        debug!(
                            "Sanitize worker {}: error sanitizing file {}: {}",
                            worker_id, file_record.id, e
                        );
                    },
                }
            } else {
                // No work available, sleep briefly
                sleep(Duration::from_millis(10)).await;
            }
        }

        debug!("Sanitize worker {} stopped", worker_id);
    }

    /// Sanitize filename and update database
    async fn sanitize_file(
        file_record: &mut FileRecord,
        db_pool: &PgPool,
    ) -> Result<(), PipelineError> {
        use std::path::Path;

        let old_path = Path::new(&file_record.filepath);
        let parent = old_path.parent().ok_or_else(|| {
            PipelineError::IOError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No parent directory",
            ))
        })?;

        // Sanitize filename
        let old_filename = &file_record.filename;
        let mut new_filename = old_filename.clone();
        let mut changed = false;

        // 1. Convert .midi → .mid
        if new_filename.ends_with(".midi") || new_filename.ends_with(".MIDI") {
            new_filename = new_filename[..new_filename.len() - 5].to_string() + ".mid";
            changed = true;
        }

        // 2. Replace spaces with underscores
        if new_filename.contains(' ') {
            new_filename = new_filename.replace(' ', "_");
            changed = true;
        }

        // 3. Remove special characters (keep only alphanumeric, dash, underscore, dot)
        let sanitized: String = new_filename
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                    c
                } else {
                    '_'
                }
            })
            .collect();

        if sanitized != new_filename {
            new_filename = sanitized;
            changed = true;
        }

        if !changed {
            return Ok(());
        }

        // Rename file on disk
        let new_path = parent.join(&new_filename);
        tokio::fs::rename(&old_path, &new_path).await?;

        // Update database
        let new_filepath = new_path.to_string_lossy().to_string();
        sqlx::query("UPDATE files SET filename = $1, filepath = $2 WHERE id = $3")
            .bind(&new_filename)
            .bind(&new_filepath)
            .bind(file_record.id)
            .execute(db_pool)
            .await?;

        // Update file record
        file_record.filename = new_filename;
        file_record.filepath = new_filepath;

        Ok(())
    }
}

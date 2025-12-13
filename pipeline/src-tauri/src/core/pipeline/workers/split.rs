// pipeline/src-tauri/src/core/pipeline/workers/split.rs
//! Stage 3: Split workers - Split multi-track MIDI files into individual tracks

use crate::core::pipeline::queues::{FileRecord, PipelineQueues};
use crate::error::PipelineError;
use sqlx::PgPool;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{debug, info};

/// Split worker configuration
pub struct SplitWorkerConfig {
    pub db_pool: PgPool,
    pub input_queue: Arc<PipelineQueues>,
    pub output_queue: Arc<PipelineQueues>,
    pub running: Arc<AtomicBool>,
    pub counter: Arc<AtomicU64>,
    pub worker_count: usize,
}

/// Split worker - handles Stage 3 processing
pub struct SplitWorker {
    _config: SplitWorkerConfig,
}

impl SplitWorker {
    pub fn new(config: SplitWorkerConfig) -> Self {
        Self { _config: config }
    }

    pub async fn spawn_workers(config: SplitWorkerConfig) -> Result<(), PipelineError> {
        info!("Spawning {} split workers", config.worker_count);

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
        debug!("Split worker {} started", worker_id);

        while running.load(Ordering::Acquire) {
            if let Some(file_record) = input_queue.sanitize_to_split.pop() {
                // Check if multi-track
                if !file_record.is_multi_track {
                    // Single track, pass through
                    if let Err(_) = output_queue.split_to_analyze.push(file_record) {
                        debug!("Split worker {}: analyze queue full", worker_id);
                        sleep(Duration::from_millis(10)).await;
                    }
                    counter.fetch_add(1, Ordering::AcqRel);
                    continue;
                }

                // Multi-track file - split it
                match Self::split_tracks(&file_record, &db_pool, &output_queue).await {
                    Ok(split_count) => {
                        debug!(
                            "Split worker {}: split file {} into {} tracks",
                            worker_id, file_record.id, split_count
                        );
                        counter.fetch_add(1, Ordering::AcqRel);
                    },
                    Err(e) => {
                        debug!(
                            "Split worker {}: error splitting file {}: {}",
                            worker_id, file_record.id, e
                        );
                        // Push original file to analyze queue on error
                        let _ = output_queue.split_to_analyze.push(file_record);
                    },
                }
            } else {
                sleep(Duration::from_millis(10)).await;
            }
        }

        debug!("Split worker {} stopped", worker_id);
    }

    /// Split multi-track file and push tracks to analyze queue
    async fn split_tracks(
        file_record: &FileRecord,
        db_pool: &PgPool,
        output_queue: &Arc<PipelineQueues>,
    ) -> Result<usize, PipelineError> {
        use crate::core::hash::calculate_file_hash;
        use crate::core::splitting::track_splitter::split_tracks;
        use std::path::Path;

        // Read MIDI file
        let path = Path::new(&file_record.filepath);
        let bytes = tokio::fs::read(path).await?;

        // Split tracks
        let split_result =
            split_tracks(&bytes).map_err(|e| PipelineError::MidiError(e.to_string()))?;

        if split_result.is_empty() {
            return Err(PipelineError::MidiError("No tracks to split".to_string()));
        }

        // Create output directory
        let output_dir = path
            .parent()
            .ok_or_else(|| {
                PipelineError::IOError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "No parent directory",
                ))
            })?
            .join(format!(
                "{}_splits",
                path.file_stem().unwrap().to_string_lossy()
            ));

        tokio::fs::create_dir_all(&output_dir).await?;

        let mut split_count = 0;

        // Write and insert each split track
        for split_track in split_result {
            let track_filename = format!("track_{:02}.mid", split_track.track_number);
            let track_path = output_dir.join(&track_filename);

            // Write track file
            tokio::fs::write(&track_path, &split_track.midi_bytes).await?;

            // Calculate hash for split track
            let track_hash = calculate_file_hash(&track_path).map_err(|e| {
                PipelineError::GeneralError(format!("Hash calculation failed: {}", e))
            })?;

            // Insert into database
            let track_filepath = track_path.to_string_lossy().to_string();
            let file_id: i64 = sqlx::query_scalar(
                r#"
                INSERT INTO files (filename, original_filename, filepath, parent_folder, content_hash, file_size_bytes)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING id
                "#
            )
            .bind(&track_filename)
            .bind(&track_filename)
            .bind(&track_filepath)
            .bind(file_record.parent_folder.as_ref())
            .bind(&track_hash)
            .bind(split_track.midi_bytes.len() as i64)
            .fetch_one(db_pool)
            .await?;

            // Create track_splits relationship
            sqlx::query(
                "INSERT INTO track_splits (parent_file_id, track_file_id, track_index) VALUES ($1, $2, $3)"
            )
            .bind(file_record.id)
            .bind(file_id)
            .bind(split_track.track_number as i32)
            .execute(db_pool)
            .await?;

            // Push to analyze queue
            let track_record = FileRecord {
                id: file_id,
                filepath: track_filepath,
                filename: track_filename,
                parent_folder: file_record.parent_folder.clone(),
                is_multi_track: false,
                analyzed: false,
            };

            let _ = output_queue.split_to_analyze.push(track_record);
            split_count += 1;
        }

        Ok(split_count)
    }
}

// pipeline/src-tauri/src/core/pipeline/workers/rename.rs
//! Stage 5: Rename workers - Generate metadata-based filenames (OPTIONAL - disabled by default)

use crate::core::pipeline::queues::{FileRecord, PipelineQueues};
use crate::error::PipelineError;
use sqlx::PgPool;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{debug, info};

/// Rename worker configuration
pub struct RenameWorkerConfig {
    pub db_pool: PgPool,
    pub input_queue: Arc<PipelineQueues>,
    pub output_queue: Arc<PipelineQueues>,
    pub running: Arc<AtomicBool>,
    pub counter: Arc<AtomicU64>,
    pub worker_count: usize,
}

/// Rename worker - handles Stage 5 processing (OPTIONAL)
pub struct RenameWorker {
    _config: RenameWorkerConfig,
}

impl RenameWorker {
    pub fn new(config: RenameWorkerConfig) -> Self {
        Self { _config: config }
    }

    pub async fn spawn_workers(config: RenameWorkerConfig) -> Result<(), PipelineError> {
        info!(
            "Spawning {} rename workers (OPTIONAL PHASE)",
            config.worker_count
        );

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
        debug!("Rename worker {} started (OPTIONAL)", worker_id);

        while running.load(Ordering::Acquire) {
            if let Some(mut file_record) = input_queue.analyze_to_rename.pop() {
                // Rename file with metadata
                match Self::rename_file(&mut file_record, &db_pool).await {
                    Ok(_) => {
                        if output_queue.rename_to_export.push(file_record).is_err() {
                            debug!("Rename worker {}: export queue full", worker_id);
                            sleep(Duration::from_millis(10)).await;
                        }
                        counter.fetch_add(1, Ordering::AcqRel);
                    },
                    Err(e) => {
                        debug!(
                            "Rename worker {}: error renaming file {}: {}",
                            worker_id, file_record.id, e
                        );
                        // Push to export queue anyway
                        let _ = output_queue.rename_to_export.push(file_record);
                    },
                }
            } else {
                sleep(Duration::from_millis(10)).await;
            }
        }

        debug!("Rename worker {} stopped", worker_id);
    }

    /// Strict character sanitization - only allows: a-z A-Z 0-9 _ - .
    /// Removes consecutive duplicates of special characters
    fn sanitize_strict(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let mut last_was_special = false;

        for c in s.chars() {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    result.push(c);
                    last_was_special = false;
                },
                '_' | '-' | '.' => {
                    if !last_was_special {
                        result.push(c);
                        last_was_special = true;
                    }
                },
                _ => {
                    // Replace invalid characters with underscore
                    if !last_was_special {
                        result.push('_');
                        last_was_special = true;
                    }
                },
            }
        }

        // Trim trailing special characters
        result.trim_end_matches(['_', '-', '.']).to_string()
    }

    /// Calculate bar length from MIDI metadata
    /// Formula: bars = (duration_seconds * bpm / 60) / time_signature_numerator
    fn calculate_bars(
        duration_seconds: Option<f64>,
        bpm: Option<f64>,
        time_sig_numerator: Option<i32>,
    ) -> Option<i32> {
        match (duration_seconds, bpm, time_sig_numerator) {
            (Some(dur), Some(tempo), Some(numerator))
                if dur > 0.0 && tempo > 0.0 && numerator > 0 =>
            {
                let beats = (dur * tempo) / 60.0;
                let bars = beats / (numerator as f64);
                Some(bars.round() as i32)
            },
            _ => None,
        }
    }

    /// Rename file with metadata-based naming: {bars}-{bpm}bpm_{key}_{folder}-{name}.mid
    async fn rename_file(
        file_record: &mut FileRecord,
        db_pool: &PgPool,
    ) -> Result<(), PipelineError> {
        use std::path::Path;

        // Load extended metadata including duration and time signature
        let metadata = sqlx::query_as::<
            _,
            (
                Option<f64>,    // tempo_bpm
                Option<String>, // key_signature
                Option<f64>,    // duration_seconds
                Option<i32>,    // time_signature_numerator
            ),
        >(
            "SELECT tempo_bpm, key_signature::text, duration_seconds, time_signature_numerator
             FROM musical_metadata WHERE file_id = $1",
        )
        .bind(file_record.id)
        .fetch_optional(db_pool)
        .await?;

        let (bpm, key, duration_seconds, time_sig_numerator) = match metadata {
            Some(m) => m,
            None => return Ok(()), // No metadata, skip rename
        };

        // Calculate bars
        let bars = Self::calculate_bars(duration_seconds, bpm, time_sig_numerator);

        // Build filename: {bars}-{bpm}bpm_{key}_{folder}-{name}
        let old_path = Path::new(&file_record.filepath);
        let stem = old_path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");

        let mut parts = Vec::new();

        // Part 1: Bar length (if available)
        if let Some(bar_count) = bars {
            parts.push(format!("{}", bar_count));
        }

        // Part 2: BPM (always 3 digits with leading zeros)
        if let Some(bpm_val) = bpm {
            parts.push(format!("{:03}bpm", bpm_val.round() as i32));
        }

        // Part 3: Key signature
        if let Some(key_val) = key {
            parts.push(Self::sanitize_strict(&key_val));
        }

        // Part 4: Parent folder name (sanitized)
        if let Some(ref folder) = file_record.parent_folder {
            parts.push(Self::sanitize_strict(folder));
        }

        // Part 5: Original filename (sanitized)
        parts.push(Self::sanitize_strict(stem));

        if parts.is_empty() {
            return Ok(()); // No metadata to add
        }

        // Join parts: bars-bpm_key_folder-name
        // Use hyphen between bars and bpm, underscore for rest, hyphen before final name
        let new_filename = if bars.is_some() && bpm.is_some() {
            let bars_bpm = format!("{}-{}", parts[0], parts[1]);
            let middle = parts[2..parts.len() - 1].join("_");
            let name = &parts[parts.len() - 1];
            if middle.is_empty() {
                format!("{}_{}.mid", bars_bpm, name)
            } else {
                format!("{}_{}_{}.mid", bars_bpm, middle, name)
            }
        } else {
            format!("{}.mid", parts.join("_"))
        };

        // Rename on disk
        let parent = old_path.parent().ok_or_else(|| {
            PipelineError::IOError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No parent directory",
            ))
        })?;
        let new_path = parent.join(&new_filename);

        // Only rename if filename changed
        if old_path != new_path {
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
        }

        Ok(())
    }
}

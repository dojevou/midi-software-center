// pipeline/src-tauri/src/core/pipeline/workers/analyze.rs
//! Stage 4: Analyze workers - BPM, key, drum detection from MIDI events

use crate::core::pipeline::queues::{FileRecord, PipelineQueues};
use crate::error::PipelineError;
use sqlx::PgPool;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{debug, info};

/// Analyze worker configuration
pub struct AnalyzeWorkerConfig {
    pub db_pool: PgPool,
    pub input_queue: Arc<PipelineQueues>,
    pub output_queue: Arc<PipelineQueues>,
    pub running: Arc<AtomicBool>,
    pub counter: Arc<AtomicU64>,
    pub worker_count: usize,
    pub enable_rename: bool, // Route to rename or export queue
}

/// Analyze worker - handles Stage 4 processing
pub struct AnalyzeWorker {
    _config: AnalyzeWorkerConfig,
}

impl AnalyzeWorker {
    pub fn new(config: AnalyzeWorkerConfig) -> Self {
        Self { _config: config }
    }

    pub async fn spawn_workers(config: AnalyzeWorkerConfig) -> Result<(), PipelineError> {
        info!("Spawning {} analyze workers", config.worker_count);

        for worker_id in 0..config.worker_count {
            let db_pool = config.db_pool.clone();
            let input_queue = Arc::clone(&config.input_queue);
            let output_queue = Arc::clone(&config.output_queue);
            let enable_rename = config.enable_rename;
            let running = Arc::clone(&config.running);
            let counter = Arc::clone(&config.counter);

            tokio::spawn(async move {
                Self::worker_loop(
                    worker_id,
                    db_pool,
                    input_queue,
                    output_queue,
                    enable_rename,
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
        enable_rename: bool,
        running: Arc<AtomicBool>,
        counter: Arc<AtomicU64>,
    ) {
        debug!("Analyze worker {} started", worker_id);

        while running.load(Ordering::Acquire) {
            if let Some(mut file_record) = input_queue.split_to_analyze.pop() {
                // Analyze file
                match Self::analyze_file(&file_record, &db_pool).await {
                    Ok(_) => {
                        file_record.analyzed = true;

                        // Route to next stage
                        let target_queue = if enable_rename {
                            &output_queue.analyze_to_rename
                        } else {
                            &output_queue.rename_to_export
                        };

                        if target_queue.push(file_record).is_err() {
                            debug!("Analyze worker {}: next queue full", worker_id);
                            sleep(Duration::from_millis(10)).await;
                        }

                        counter.fetch_add(1, Ordering::AcqRel);
                    },
                    Err(e) => {
                        debug!(
                            "Analyze worker {}: error analyzing file {}: {}",
                            worker_id, file_record.id, e
                        );
                    },
                }
            } else {
                sleep(Duration::from_millis(10)).await;
            }
        }

        debug!("Analyze worker {} stopped", worker_id);
    }

    /// Analyze MIDI file and store musical metadata
    async fn analyze_file(file_record: &FileRecord, db_pool: &PgPool) -> Result<(), PipelineError> {
        use crate::core::analysis::bpm_detector::detect_bpm;
        use crate::core::analysis::key_detector::detect_key;
        use crate::core::midi::parse_midi_file;
        use std::path::Path;

        // Read and parse MIDI file
        let path = Path::new(&file_record.filepath);
        let bytes = tokio::fs::read(path).await?;
        let midi = parse_midi_file(&bytes).map_err(|e| PipelineError::MidiError(e.to_string()))?;

        // Detect BPM (returns struct directly, not Option)
        let bpm_result = detect_bpm(&midi);
        let bpm = if bpm_result.confidence > 0.5 {
            Some(bpm_result.bpm)
        } else {
            None
        };
        let bpm_confidence = Some(bpm_result.confidence);

        // Detect key (returns struct directly, not Option)
        let key_result = detect_key(&midi);
        let key_signature = if key_result.confidence > 0.5 {
            Some(format!("{}{}", key_result.key, key_result.scale_type))
        } else {
            None
        };
        let key_confidence = Some(key_result.confidence);

        // Calculate basic stats from MIDI tracks
        let note_count: i32 = midi
            .tracks
            .iter()
            .flat_map(|track| track.events.iter())
            .filter(|e| matches!(e.event, crate::core::midi::types::Event::NoteOn { .. }))
            .count() as i32;

        // Calculate duration (use last event from longest track)
        let duration_ticks = midi
            .tracks
            .iter()
            .filter_map(|track| track.events.last().map(|e| e.delta_ticks))
            .max()
            .unwrap_or(0);

        let duration_seconds = if let Some(bpm_val) = bpm {
            let ticks_per_beat = midi.header.ticks_per_quarter_note as f64;
            let beats = duration_ticks as f64 / ticks_per_beat;
            Some((beats / bpm_val) * 60.0)
        } else {
            None
        };

        // Insert or update musical_metadata
        sqlx::query(
            r#"
            INSERT INTO musical_metadata (
                file_id, tempo_bpm, bpm_confidence, key_signature, key_confidence,
                duration_seconds, duration_ticks, note_count
            )
            VALUES ($1, $2, $3, $4::musical_key, $5, $6, $7, $8)
            ON CONFLICT (file_id) DO UPDATE SET
                tempo_bpm = EXCLUDED.tempo_bpm,
                bpm_confidence = EXCLUDED.bpm_confidence,
                key_signature = EXCLUDED.key_signature,
                key_confidence = EXCLUDED.key_confidence,
                duration_seconds = EXCLUDED.duration_seconds,
                duration_ticks = EXCLUDED.duration_ticks,
                note_count = EXCLUDED.note_count
            "#,
        )
        .bind(file_record.id)
        .bind(bpm)
        .bind(bpm_confidence)
        .bind(&key_signature)
        .bind(key_confidence)
        .bind(duration_seconds)
        .bind(duration_ticks as i32)
        .bind(note_count)
        .execute(db_pool)
        .await?;

        // Update analyzed_at timestamp
        sqlx::query("UPDATE files SET analyzed_at = NOW() WHERE id = $1")
            .bind(file_record.id)
            .execute(db_pool)
            .await?;

        Ok(())
    }
}

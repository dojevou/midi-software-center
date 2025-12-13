// pipeline/src-tauri/src/core/pipeline/workers/export.rs
//! Stage 6: Export workers - Copy to MPC One/Akai Force compatible structure (OPTIONAL)

use crate::core::pipeline::queues::{FileRecord, PipelineQueues};
use crate::error::PipelineError;
use sqlx::PgPool;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{debug, info};

/// Export worker configuration
pub struct ExportWorkerConfig {
    pub db_pool: PgPool,
    pub input_queue: Arc<PipelineQueues>,
    pub running: Arc<AtomicBool>,
    pub counter: Arc<AtomicU64>,
    pub worker_count: usize,
    pub export_path: PathBuf,
    pub export_format: String, // "mpc-one", "akai-force", or "both"
}

/// Export worker - handles Stage 6 processing (OPTIONAL)
pub struct ExportWorker {
    _config: ExportWorkerConfig,
}

/// MPC category for file organization
#[derive(Debug, Clone, PartialEq)]
pub enum MPCCategory {
    // Drums (most granular)
    DrumKicks,
    DrumSnares,
    DrumHats,
    DrumCymbals,
    DrumToms,
    DrumPerc,
    Drums,

    // Melodic
    Bass,
    Melody,
    Chords,
    Progressions,

    // Other
    FX,
    Loops,
}

impl MPCCategory {
    /// Get folder path for this category
    pub fn folder_path(&self) -> &str {
        match self {
            MPCCategory::DrumKicks => "MPC_Documents/SAMPLES/Drums/Kicks",
            MPCCategory::DrumSnares => "MPC_Documents/SAMPLES/Drums/Snares",
            MPCCategory::DrumHats => "MPC_Documents/SAMPLES/Drums/Hats",
            MPCCategory::DrumCymbals => "MPC_Documents/SAMPLES/Drums/Cymbals",
            MPCCategory::DrumToms => "MPC_Documents/SAMPLES/Drums/Toms",
            MPCCategory::DrumPerc => "MPC_Documents/SAMPLES/Drums/Percussion",
            MPCCategory::Drums => "MPC_Documents/SAMPLES/Drums",
            MPCCategory::Bass => "MPC_Documents/SAMPLES/Bass",
            MPCCategory::Melody => "MPC_Documents/SAMPLES/Melody",
            MPCCategory::Chords => "MPC_Documents/SAMPLES/Chords",
            MPCCategory::Progressions => "MPC_Documents/Progressions",
            MPCCategory::FX => "MPC_Documents/SAMPLES/FX",
            MPCCategory::Loops => "MPC_Documents/SAMPLES/Loops",
        }
    }
}

impl ExportWorker {
    pub fn new(config: ExportWorkerConfig) -> Self {
        Self { _config: config }
    }

    pub async fn spawn_workers(config: ExportWorkerConfig) -> Result<(), PipelineError> {
        info!(
            "Spawning {} export workers (OPTIONAL PHASE) to {}",
            config.worker_count,
            config.export_path.display()
        );

        for worker_id in 0..config.worker_count {
            let db_pool = config.db_pool.clone();
            let input_queue = Arc::clone(&config.input_queue);
            let export_path = config.export_path.clone();
            let export_format = config.export_format.clone();
            let running = Arc::clone(&config.running);
            let counter = Arc::clone(&config.counter);

            tokio::spawn(async move {
                Self::worker_loop(
                    worker_id,
                    db_pool,
                    input_queue,
                    export_path,
                    export_format,
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
        export_path: PathBuf,
        export_format: String,
        running: Arc<AtomicBool>,
        counter: Arc<AtomicU64>,
    ) {
        debug!(
            "Export worker {} started (OPTIONAL) - format: {}",
            worker_id, export_format
        );

        while running.load(Ordering::Acquire) {
            if let Some(file_record) = input_queue.rename_to_export.pop() {
                // Export file
                match Self::export_file(&file_record, &db_pool, &export_path).await {
                    Ok(_) => {
                        counter.fetch_add(1, Ordering::AcqRel);
                    },
                    Err(e) => {
                        debug!(
                            "Export worker {}: error exporting file {}: {}",
                            worker_id, file_record.id, e
                        );
                    },
                }
            } else {
                sleep(Duration::from_millis(10)).await;
            }
        }

        debug!("Export worker {} stopped", worker_id);
    }

    /// Export file to MPC/Force compatible structure with optimized I/O
    async fn export_file(
        file_record: &FileRecord,
        db_pool: &PgPool,
        export_path: &PathBuf,
    ) -> Result<(), PipelineError> {
        use std::path::Path;

        // Detect category using advanced multi-strategy algorithm
        let category = Self::detect_mpc_category(file_record, db_pool).await?;
        let category_path = export_path.join(category.folder_path());

        // Create category directory (only once per category)
        tokio::fs::create_dir_all(&category_path).await?;

        // Copy file with optimized async I/O
        let source = Path::new(&file_record.filepath);
        let dest = category_path.join(&file_record.filename);

        // Use tokio's async file copy for efficient I/O
        // This uses buffered I/O internally and doesn't block the executor
        tokio::fs::copy(source, &dest).await?;

        Ok(())
    }

    /// Advanced MPC category detection using multiple signals:
    /// 1. Auto-tags from database
    /// 2. MIDI note range analysis
    /// 3. Parent folder name
    /// 4. Filename patterns
    async fn detect_mpc_category(
        file_record: &FileRecord,
        db_pool: &PgPool,
    ) -> Result<MPCCategory, PipelineError> {
        // Load tags from database
        let tags: Vec<String> = sqlx::query_scalar("SELECT tag FROM file_tags WHERE file_id = $1")
            .bind(file_record.id)
            .fetch_all(db_pool)
            .await
            .unwrap_or_default();

        // Load MIDI note range from musical_metadata
        let note_range: Option<(i32, i32)> = sqlx::query_as(
            "SELECT lowest_note, highest_note FROM musical_metadata WHERE file_id = $1",
        )
        .bind(file_record.id)
        .fetch_optional(db_pool)
        .await?;

        // Strategy 1: Check auto-tags first (most reliable)
        let tags_lower: Vec<String> = tags.iter().map(|t| t.to_lowercase()).collect();

        // Drum detection (most specific first)
        if tags_lower.iter().any(|t| t.contains("kick") || t == "bd" || t == "bass-drum") {
            return Ok(MPCCategory::DrumKicks);
        }
        if tags_lower.iter().any(|t| t.contains("snare") || t == "sd") {
            return Ok(MPCCategory::DrumSnares);
        }
        if tags_lower
            .iter()
            .any(|t| t.contains("hihat") || t.contains("hi-hat") || t == "hh" || t.contains("hat"))
        {
            return Ok(MPCCategory::DrumHats);
        }
        if tags_lower.iter().any(|t| {
            t.contains("crash") || t.contains("ride") || t.contains("china") || t.contains("cymbal")
        }) {
            return Ok(MPCCategory::DrumCymbals);
        }
        if tags_lower.iter().any(|t| t.contains("tom")) {
            return Ok(MPCCategory::DrumToms);
        }
        if tags_lower.iter().any(|t| {
            t.contains("perc") || t.contains("shaker") || t.contains("conga") || t.contains("bongo")
        }) {
            return Ok(MPCCategory::DrumPerc);
        }
        if tags_lower
            .iter()
            .any(|t| t == "drums" || t == "drum-kit" || t.contains("drum-loop"))
        {
            return Ok(MPCCategory::Drums);
        }

        // Melodic/harmonic detection
        if tags_lower.iter().any(|t| t.contains("bass") || t == "808" || t == "sub-bass") {
            return Ok(MPCCategory::Bass);
        }
        if tags_lower
            .iter()
            .any(|t| t.contains("chord") || t.contains("progression") || t.contains("harmony"))
        {
            return Ok(MPCCategory::Chords);
        }
        if tags_lower
            .iter()
            .any(|t| t.contains("melody") || t.contains("lead") || t.contains("arp"))
        {
            return Ok(MPCCategory::Melody);
        }
        if tags_lower.iter().any(|t| {
            t.contains("fx") || t.contains("effect") || t.contains("riser") || t.contains("sweep")
        }) {
            return Ok(MPCCategory::FX);
        }
        if tags_lower.iter().any(|t| t.contains("loop") || t.contains("full-mix")) {
            return Ok(MPCCategory::Loops);
        }

        // Strategy 2: Use MIDI note range analysis
        if let Some((low, high)) = note_range {
            // Bass range: C1 (36) to E3 (52)
            if low >= 36 && high <= 52 {
                return Ok(MPCCategory::Bass);
            }

            // Drum range: typically C1 (36) to C4 (60) - GM drum map
            if low >= 35 && high <= 81 && (high - low) < 25 {
                // Narrow range in drum zone suggests drums
                return Ok(MPCCategory::Drums);
            }

            // High melody range: C5 (72) and above
            if low >= 60 && high >= 72 {
                return Ok(MPCCategory::Melody);
            }

            // Wide range suggests chords/progression
            if (high - low) > 36 {
                return Ok(MPCCategory::Chords);
            }
        }

        // Strategy 3: Parent folder analysis (fallback)
        if let Some(ref folder) = file_record.parent_folder {
            let folder_lower = folder.to_lowercase();

            if folder_lower.contains("kick") {
                return Ok(MPCCategory::DrumKicks);
            } else if folder_lower.contains("snare") {
                return Ok(MPCCategory::DrumSnares);
            } else if folder_lower.contains("hat") || folder_lower.contains("hihat") {
                return Ok(MPCCategory::DrumHats);
            } else if folder_lower.contains("cymbal") {
                return Ok(MPCCategory::DrumCymbals);
            } else if folder_lower.contains("tom") {
                return Ok(MPCCategory::DrumToms);
            } else if folder_lower.contains("perc") {
                return Ok(MPCCategory::DrumPerc);
            } else if folder_lower.contains("drum") {
                return Ok(MPCCategory::Drums);
            } else if folder_lower.contains("bass") || folder_lower.contains("808") {
                return Ok(MPCCategory::Bass);
            } else if folder_lower.contains("melody") || folder_lower.contains("lead") {
                return Ok(MPCCategory::Melody);
            } else if folder_lower.contains("chord") {
                return Ok(MPCCategory::Chords);
            } else if folder_lower.contains("fx") || folder_lower.contains("effect") {
                return Ok(MPCCategory::FX);
            } else if folder_lower.contains("loop") {
                return Ok(MPCCategory::Loops);
            }
        }

        // Strategy 4: Filename analysis (last resort)
        let filename_lower = file_record.filename.to_lowercase();
        if filename_lower.contains("kick") {
            return Ok(MPCCategory::DrumKicks);
        } else if filename_lower.contains("snare") {
            return Ok(MPCCategory::DrumSnares);
        } else if filename_lower.contains("bass") || filename_lower.contains("808") {
            return Ok(MPCCategory::Bass);
        } else if filename_lower.contains("chord") {
            return Ok(MPCCategory::Chords);
        } else if filename_lower.contains("melody") || filename_lower.contains("lead") {
            return Ok(MPCCategory::Melody);
        }

        // Default category: Melody (safest fallback for uncategorized melodic content)
        Ok(MPCCategory::Melody)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mpc_category_paths() {
        assert_eq!(
            MPCCategory::DrumKicks.folder_path(),
            "MPC_Documents/SAMPLES/Drums/Kicks"
        );
        assert_eq!(
            MPCCategory::Bass.folder_path(),
            "MPC_Documents/SAMPLES/Bass"
        );
        assert_eq!(
            MPCCategory::Progressions.folder_path(),
            "MPC_Documents/Progressions"
        );
    }
}

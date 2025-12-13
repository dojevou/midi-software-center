// pipeline/src-tauri/src/core/pipeline/orchestrator.rs
//! Pipelined parallel processing orchestrator
//!
//! Coordinates all pipeline stages running simultaneously via lock-free queues.
//! Expected speedup: 3.8x (4.9 hours → 1.3 hours for 4.3M files)

use super::queues::{PipelineQueues, QUEUE_CAPACITY};
use super::worker_pool::WorkerPool;
use crate::error::PipelineError;
use sqlx::PgPool;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info, warn};

/// Pipeline configuration
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Source directory or archive file
    pub source_path: PathBuf,

    /// Database connection pool
    pub db_pool: PgPool,

    /// Enable Phase 5 renaming (default: false, skip)
    pub enable_rename: bool,

    /// Enable Phase 6 export to external drive
    pub enable_export: bool,

    /// Export destination path (if export enabled)
    pub export_path: Option<PathBuf>,

    /// Export format: "mpc-one", "akai-force", or "both"
    pub export_format: Option<String>,

    /// Worker counts per stage
    pub import_workers: usize, // Default: 16
    pub sanitize_workers: usize, // Default: 32 (CPU-bound)
    pub split_workers: usize,    // Default: 16
    pub analyze_workers: usize,  // Default: 24 (CPU-intensive)
    pub rename_workers: usize,   // Default: 32 (fast)
    pub export_workers: usize,   // Default: 8 (I/O-bound)
}

impl PipelineConfig {
    /// Create default configuration
    pub fn new(source_path: PathBuf, db_pool: PgPool) -> Self {
        Self {
            source_path,
            db_pool,
            enable_rename: false, // Skip by default
            enable_export: false,
            export_path: None,
            export_format: None,
            import_workers: 16,
            sanitize_workers: 32,
            split_workers: 16,
            analyze_workers: 24,
            rename_workers: 32,
            export_workers: 8,
        }
    }

    /// Enable Phase 5 renaming
    pub fn with_rename(mut self) -> Self {
        self.enable_rename = true;
        self
    }

    /// Enable Phase 6 export
    pub fn with_export(mut self, path: PathBuf, format: String) -> Self {
        self.enable_export = true;
        self.export_path = Some(path);
        self.export_format = Some(format);
        self
    }
}

/// Pipeline orchestrator - coordinates all stages
pub struct PipelineOrchestrator {
    config: PipelineConfig,
    queues: Arc<PipelineQueues>,
    worker_pools: WorkerPools,
    progress: Arc<PipelineProgress>,
}

/// Worker pools for all stages
struct WorkerPools {
    import: WorkerPool,
    sanitize: WorkerPool,
    split: WorkerPool,
    analyze: WorkerPool,
    rename: Option<WorkerPool>, // Only if enabled
    export: Option<WorkerPool>, // Only if enabled
}

/// Progress tracking across all stages
#[derive(Default)]
pub struct PipelineProgress {
    // Phase 1: Import
    pub import_total: AtomicU64,
    pub import_completed: AtomicU64,

    // Phase 2: Sanitize
    pub sanitize_completed: AtomicU64,

    // Phase 3: Split
    pub split_completed: AtomicU64,

    // Phase 4: Analyze
    pub analyze_completed: AtomicU64,

    // Phase 5: Rename (optional)
    pub rename_completed: AtomicU64,

    // Phase 6: Export (optional)
    pub export_completed: AtomicU64,
}

impl PipelineProgress {
    /// Get overall completion percentage (0-100)
    pub fn overall_percentage(&self) -> f64 {
        let total = self.import_total.load(Ordering::Acquire) as f64;
        if total == 0.0 {
            return 0.0;
        }

        // Use the slowest stage as overall progress
        let import = self.import_completed.load(Ordering::Acquire) as f64;
        let sanitize = self.sanitize_completed.load(Ordering::Acquire) as f64;
        let split = self.split_completed.load(Ordering::Acquire) as f64;
        let analyze = self.analyze_completed.load(Ordering::Acquire) as f64;

        // Minimum progress across all stages
        let min_progress = import.min(sanitize).min(split).min(analyze);

        (min_progress / total * 100.0).min(100.0)
    }

    /// Get stage-specific percentage
    pub fn stage_percentage(&self, stage: &str) -> f64 {
        let total = self.import_total.load(Ordering::Acquire) as f64;
        if total == 0.0 {
            return 0.0;
        }

        let completed = match stage {
            "import" => self.import_completed.load(Ordering::Acquire),
            "sanitize" => self.sanitize_completed.load(Ordering::Acquire),
            "split" => self.split_completed.load(Ordering::Acquire),
            "analyze" => self.analyze_completed.load(Ordering::Acquire),
            "rename" => self.rename_completed.load(Ordering::Acquire),
            "export" => self.export_completed.load(Ordering::Acquire),
            _ => 0,
        } as f64;

        (completed / total * 100.0).min(100.0)
    }
}

impl PipelineOrchestrator {
    /// Create new pipeline orchestrator
    pub fn new(config: PipelineConfig) -> Self {
        let queues = Arc::new(PipelineQueues::with_capacity(QUEUE_CAPACITY));

        let worker_pools = WorkerPools {
            import: WorkerPool::new(config.import_workers),
            sanitize: WorkerPool::new(config.sanitize_workers),
            split: WorkerPool::new(config.split_workers),
            analyze: WorkerPool::new(config.analyze_workers),
            rename: if config.enable_rename {
                Some(WorkerPool::new(config.rename_workers))
            } else {
                None
            },
            export: if config.enable_export {
                Some(WorkerPool::new(config.export_workers))
            } else {
                None
            },
        };

        Self { config, queues, worker_pools, progress: Arc::new(PipelineProgress::default()) }
    }

    /// Run the entire pipeline
    pub async fn run(&mut self) -> Result<(), PipelineError> {
        info!("🚀 Starting pipelined parallel processing");
        info!("Source: {:?}", self.config.source_path);
        info!("Rename enabled: {}", self.config.enable_rename);
        info!("Export enabled: {}", self.config.enable_export);

        // Stage 0: Extract all archives first (recursive, in-place)
        self.extract_archives().await?;

        // Start all worker pools in parallel
        self.start_all_stages().await?;

        // Monitor progress until completion
        self.monitor_progress().await;

        info!("✅ Pipeline completed successfully");
        Ok(())
    }

    /// Stage 0: Extract all archives in source directory (recursive)
    async fn extract_archives(&self) -> Result<(), PipelineError> {
        use crate::io::decompressor::{extractor, formats};
        use walkdir::WalkDir;

        info!(
            "Stage 0: Extracting archives from {:?}",
            self.config.source_path
        );

        let config = extractor::ExtractionConfig::default();
        let mut total_extracted = 0;
        let mut total_midi_files = 0;

        // Find all archives in source directory
        for entry in WalkDir::new(&self.config.source_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Check if it's an archive
            if path.is_file() && formats::is_archive(path) {
                info!("Extracting archive: {:?}", path);

                // Extract to parent directory (in-place)
                if let Some(parent) = path.parent() {
                    match extractor::extract_archive(path, parent, &config) {
                        Ok(result) => {
                            total_extracted += result.archives_processed;
                            total_midi_files += result.midi_files.len();
                            info!(
                                "✓ Extracted {} MIDI files from {} archives",
                                result.midi_files.len(),
                                result.archives_processed
                            );
                            if !result.errors.is_empty() {
                                warn!("Extraction warnings: {:?}", result.errors);
                            }
                        },
                        Err(e) => {
                            warn!("Failed to extract {:?}: {}", path, e);
                        },
                    }
                }
            }
        }

        info!(
            "Stage 0 complete: {} archives extracted, {} MIDI files found",
            total_extracted, total_midi_files
        );

        Ok(())
    }

    /// Start all pipeline stages
    async fn start_all_stages(&mut self) -> Result<(), PipelineError> {
        // Stage 1: Import
        self.start_import_stage().await?;

        // Stage 2: Sanitize
        self.start_sanitize_stage().await?;

        // Stage 3: Split
        self.start_split_stage().await?;

        // Stage 4: Analyze
        self.start_analyze_stage().await?;

        // Stage 5: Rename (optional)
        if self.config.enable_rename {
            self.start_rename_stage().await?;
        }

        // Stage 6: Export (optional)
        if self.config.enable_export {
            self.start_export_stage().await?;
        }

        Ok(())
    }

    /// Stage 1: Import files to database
    async fn start_import_stage(&mut self) -> Result<(), PipelineError> {
        info!(
            "Stage 1: Starting import with {} workers",
            self.worker_pools.import.worker_count()
        );
        self.worker_pools.import.start();

        // Spawn import workers
        use crate::core::pipeline::workers::import::{ImportWorker, ImportWorkerConfig};

        let config = ImportWorkerConfig {
            source_path: self.config.source_path.clone(),
            db_pool: self.config.db_pool.clone(),
            output_queue: Arc::clone(&self.queues),
            running: self.worker_pools.import.running_flag(),
            counter: self.worker_pools.import.processed_counter(),
            worker_count: self.worker_pools.import.worker_count(),
        };

        ImportWorker::spawn_workers(config).await?;

        Ok(())
    }

    /// Stage 2: Sanitize filenames
    async fn start_sanitize_stage(&mut self) -> Result<(), PipelineError> {
        info!(
            "Stage 2: Starting sanitize with {} workers",
            self.worker_pools.sanitize.worker_count()
        );
        self.worker_pools.sanitize.start();

        // Spawn sanitize workers
        use crate::core::pipeline::workers::sanitize::{SanitizeWorker, SanitizeWorkerConfig};

        let config = SanitizeWorkerConfig {
            db_pool: self.config.db_pool.clone(),
            input_queue: Arc::clone(&self.queues),
            output_queue: Arc::clone(&self.queues),
            running: self.worker_pools.sanitize.running_flag(),
            counter: self.worker_pools.sanitize.processed_counter(),
            worker_count: self.worker_pools.sanitize.worker_count(),
        };

        SanitizeWorker::spawn_workers(config).await?;

        Ok(())
    }

    /// Stage 3: Split multi-track files
    async fn start_split_stage(&mut self) -> Result<(), PipelineError> {
        info!(
            "Stage 3: Starting split with {} workers",
            self.worker_pools.split.worker_count()
        );
        self.worker_pools.split.start();

        // Spawn split workers
        use crate::core::pipeline::workers::split::{SplitWorker, SplitWorkerConfig};

        let config = SplitWorkerConfig {
            db_pool: self.config.db_pool.clone(),
            input_queue: Arc::clone(&self.queues),
            output_queue: Arc::clone(&self.queues),
            running: self.worker_pools.split.running_flag(),
            counter: self.worker_pools.split.processed_counter(),
            worker_count: self.worker_pools.split.worker_count(),
        };

        SplitWorker::spawn_workers(config).await?;

        Ok(())
    }

    /// Stage 4: Analyze musical content
    async fn start_analyze_stage(&mut self) -> Result<(), PipelineError> {
        info!(
            "Stage 4: Starting analyze with {} workers",
            self.worker_pools.analyze.worker_count()
        );
        self.worker_pools.analyze.start();

        // Spawn analyze workers
        use crate::core::pipeline::workers::analyze::{AnalyzeWorker, AnalyzeWorkerConfig};

        let config = AnalyzeWorkerConfig {
            db_pool: self.config.db_pool.clone(),
            input_queue: Arc::clone(&self.queues),
            output_queue: Arc::clone(&self.queues),
            running: self.worker_pools.analyze.running_flag(),
            counter: self.worker_pools.analyze.processed_counter(),
            worker_count: self.worker_pools.analyze.worker_count(),
            enable_rename: self.config.enable_rename,
        };

        AnalyzeWorker::spawn_workers(config).await?;

        Ok(())
    }

    /// Stage 5: Rename files (optional)
    async fn start_rename_stage(&mut self) -> Result<(), PipelineError> {
        if let Some(ref mut pool) = self.worker_pools.rename {
            info!(
                "Stage 5: Starting rename with {} workers",
                pool.worker_count()
            );
            pool.start();

            // Spawn rename workers
            use crate::core::pipeline::workers::rename::{RenameWorker, RenameWorkerConfig};

            let config = RenameWorkerConfig {
                db_pool: self.config.db_pool.clone(),
                input_queue: Arc::clone(&self.queues),
                output_queue: Arc::clone(&self.queues),
                running: pool.running_flag(),
                counter: pool.processed_counter(),
                worker_count: pool.worker_count(),
            };

            RenameWorker::spawn_workers(config).await?;
        }

        Ok(())
    }

    /// Stage 6: Export to external drive (optional)
    async fn start_export_stage(&mut self) -> Result<(), PipelineError> {
        if let Some(ref mut pool) = self.worker_pools.export {
            info!(
                "Stage 6: Starting export with {} workers",
                pool.worker_count()
            );
            pool.start();

            // Spawn export workers
            use crate::core::pipeline::workers::export::{ExportWorker, ExportWorkerConfig};

            let export_path =
                self.config.export_path.clone().ok_or_else(|| {
                    PipelineError::Config("Export path not configured".to_string())
                })?;
            let export_format =
                self.config.export_format.clone().unwrap_or_else(|| "mpc-one".to_string());

            let config = ExportWorkerConfig {
                db_pool: self.config.db_pool.clone(),
                input_queue: Arc::clone(&self.queues),
                running: pool.running_flag(),
                counter: pool.processed_counter(),
                worker_count: pool.worker_count(),
                export_path,
                export_format,
            };

            ExportWorker::spawn_workers(config).await?;
        }

        Ok(())
    }

    /// Monitor progress until completion
    async fn monitor_progress(&self) {
        loop {
            sleep(Duration::from_secs(5)).await;

            let overall = self.progress.overall_percentage();
            let queued = self.queues.total_queued();

            info!("Progress: {:.1}% | Queued: {}", overall, queued);

            // Check if all stages complete and queues empty
            if overall >= 99.9 && self.queues.is_empty() {
                debug!("Pipeline complete - all queues empty");
                break;
            }
        }
    }

    /// Get current progress
    pub fn get_progress(&self) -> &PipelineProgress {
        &self.progress
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pipeline_config_defaults() {
        let pool = PgPool::connect_lazy("postgresql://test").unwrap();
        let config = PipelineConfig::new(PathBuf::from("/test"), pool);

        assert!(!config.enable_rename);
        assert!(!config.enable_export);
        assert_eq!(config.import_workers, 16);
        assert_eq!(config.sanitize_workers, 32);
    }

    #[tokio::test]
    async fn test_pipeline_config_with_rename() {
        let pool = PgPool::connect_lazy("postgresql://test").unwrap();
        let config = PipelineConfig::new(PathBuf::from("/test"), pool).with_rename();

        assert!(config.enable_rename);
    }

    #[tokio::test]
    async fn test_pipeline_config_with_export() {
        let pool = PgPool::connect_lazy("postgresql://test").unwrap();
        let config = PipelineConfig::new(PathBuf::from("/test"), pool)
            .with_export(PathBuf::from("/mpc"), "mpc-one".to_string());

        assert!(config.enable_export);
        assert_eq!(config.export_path, Some(PathBuf::from("/mpc")));
        assert_eq!(config.export_format, Some("mpc-one".to_string()));
    }

    #[test]
    fn test_progress_tracking() {
        let progress = PipelineProgress::default();

        // Set total
        progress.import_total.store(1000, Ordering::Release);

        // No progress yet
        assert_eq!(progress.overall_percentage(), 0.0);

        // Complete 500 import
        progress.import_completed.store(500, Ordering::Release);
        assert_eq!(progress.stage_percentage("import"), 50.0);

        // Complete all import
        progress.import_completed.store(1000, Ordering::Release);
        progress.sanitize_completed.store(1000, Ordering::Release);
        progress.split_completed.store(1000, Ordering::Release);
        progress.analyze_completed.store(1000, Ordering::Release);

        assert!(progress.overall_percentage() >= 99.0);
    }
}

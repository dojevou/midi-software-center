// pipeline/src-tauri/src/core/pipeline/mod.rs
//! Pipelined parallel processing architecture for MIDI pipeline
//!
//! This module implements a lock-free pipelined architecture where all phases
//! (Import, Sanitize, Split, Analyze, Rename, Export) run simultaneously on
//! different batches of files using MPMC queues for communication.

pub mod orchestrator;
pub mod queues;
pub mod worker_pool;
pub mod workers;

pub use orchestrator::{PipelineConfig, PipelineOrchestrator};
pub use queues::PipelineQueues;
pub use worker_pool::WorkerPool;
pub use workers::{
    AnalyzeWorker, ExportWorker, ImportWorker, RenameWorker, SanitizeWorker, SplitWorker,
};

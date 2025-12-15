// pipeline/src-tauri/src/bin/pipeline-orchestrator.rs
//! Pipeline Orchestrator CLI - Pipelined Parallel Processing for MIDI Files
//!
//! This binary runs the complete pipelined architecture with lock-free queues:
//! - Stage 1: Import (16 workers)
//! - Stage 2: Sanitize (32 workers)
//! - Stage 3: Split (16 workers)
//! - Stage 4: Analyze (24 workers)
//! - Stage 5: Rename (32 workers) - OPTIONAL
//! - Stage 6: Export (8 workers) - OPTIONAL
//!
//! Expected performance: 3.8x faster than sequential (4.9 hours → 1.3 hours for 4.3M files)

use clap::Parser;
use midi_app::core::pipeline::{PipelineConfig, PipelineOrchestrator};
use sqlx::PgPool;
use std::path::PathBuf;
use tracing::{error, info};

/// Pipeline Orchestrator - Pipelined Parallel MIDI Processing
#[derive(Parser, Debug)]
#[command(name = "pipeline-orchestrator")]
#[command(about = "Process MIDI files through pipelined parallel architecture", long_about = None)]
struct Args {
    /// Source directory or archive file containing MIDI files
    #[arg(short, long)]
    source: PathBuf,

    /// Database connection string
    #[arg(
        long,
        env = "DATABASE_URL",
        default_value = "postgresql://postgres:postgres@localhost:5432/midi_library"
    )]
    database_url: String,

    /// Enable Phase 5: Rename files with metadata (disabled by default for speed)
    #[arg(long, default_value_t = false)]
    enable_rename: bool,

    /// Export destination path (enables Phase 6)
    #[arg(long)]
    export_to: Option<PathBuf>,

    /// Export format: mpc-one, akai-force, or both
    #[arg(long, default_value = "mpc-one")]
    export_format: String,

    /// Custom worker counts: import,sanitize,split,analyze,rename,export
    #[arg(long)]
    workers: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    // Parse arguments
    let args = Args::parse();

    info!("🚀 MIDI Pipeline Orchestrator");
    info!("Source: {:?}", args.source);
    info!("Database: {}", args.database_url);
    info!("Rename enabled: {}", args.enable_rename);
    info!(
        "Export: {}",
        if let Some(ref path) = args.export_to {
            format!("enabled to {:?} ({})", path, args.export_format)
        } else {
            "disabled".to_string()
        }
    );

    // Validate source
    if !args.source.exists() {
        error!("Source path does not exist: {:?}", args.source);
        return Err("Source path not found".into());
    }

    // Connect to database
    info!("Connecting to database...");
    let pool = PgPool::connect(&args.database_url).await?;
    info!("Database connected ✓");

    // Create pipeline config
    let mut config = PipelineConfig::new(args.source, pool);

    // Apply custom worker counts if provided
    if let Some(workers_str) = args.workers {
        parse_worker_counts(&workers_str, &mut config)?;
    }

    // Enable optional stages
    if args.enable_rename {
        config = config.with_rename();
    }

    if let Some(export_path) = args.export_to {
        config = config.with_export(export_path, args.export_format);
    }

    // Create orchestrator
    let mut orchestrator = PipelineOrchestrator::new(config);

    // Run pipeline
    info!("Starting pipeline...");
    orchestrator.run().await?;

    info!("Pipeline completed successfully ✅");
    Ok(())
}

/// Parse custom worker counts from string: "16,32,16,24,32,8"
fn parse_worker_counts(
    workers_str: &str,
    config: &mut PipelineConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let counts: Vec<usize> = workers_str
        .split(',')
        .map(|s| s.trim().parse())
        .collect::<Result<Vec<_>, _>>()?;

    if counts.len() != 6 {
        return Err(
            "Worker counts must be 6 values: import,sanitize,split,analyze,rename,export".into(),
        );
    }

    config.import_workers = counts[0];
    config.sanitize_workers = counts[1];
    config.split_workers = counts[2];
    config.analyze_workers = counts[3];
    config.rename_workers = counts[4];
    config.export_workers = counts[5];

    info!("Custom worker counts:");
    info!("  Import: {}", counts[0]);
    info!("  Sanitize: {}", counts[1]);
    info!("  Split: {}", counts[2]);
    info!("  Analyze: {}", counts[3]);
    info!("  Rename: {}", counts[4]);
    info!("  Export: {}", counts[5]);

    Ok(())
}

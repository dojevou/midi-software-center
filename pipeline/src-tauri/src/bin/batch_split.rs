/// Batch Split - Split all multi-track MIDI files in parallel
///
/// Processes all multi-track files from the database and splits them into
/// individual single-track files. Handles parallel processing, progress tracking,
/// and database updates for the split tracks.
use anyhow::{Context, Result};
use blake3;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use midi_pipeline::core::splitting::{split_tracks_with_repair, RepairResult};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::fs;
use tokio::task;

#[derive(Parser, Debug)]
#[command(name = "batch_split")]
#[command(about = "Batch split all multi-track MIDI files")]
struct Args {
    /// Output directory for split files
    #[arg(short, long, default_value = "/tmp/midi_splits")]
    output_dir: PathBuf,

    /// Number of parallel workers
    #[arg(short, long, default_value_t = 24)]
    workers: usize,

    /// Database connection string
    #[arg(short = 'D', long, env = "DATABASE_URL")]
    database_url: String,

    /// Batch size for database queries
    #[arg(short = 'b', long, default_value_t = 100)]
    batch_size: i64,

    /// Test mode - only process first N files
    #[arg(short = 't', long)]
    test_limit: Option<i64>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct MultiTrackFile {
    id: i64,
    filepath: String,
    filename: String,
    num_tracks: i16,
}

#[derive(Debug)]
struct Stats {
    files_processed: AtomicU64,
    tracks_created: AtomicU64,
    files_skipped: AtomicU64,
    errors: AtomicU64,
    files_repaired: AtomicU64,
    files_corrupt: AtomicU64,
}

impl Stats {
    fn new() -> Self {
        Self {
            files_processed: AtomicU64::new(0),
            tracks_created: AtomicU64::new(0),
            files_skipped: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            files_repaired: AtomicU64::new(0),
            files_corrupt: AtomicU64::new(0),
        }
    }

    fn inc_processed(&self) {
        self.files_processed.fetch_add(1, Ordering::Relaxed);
    }

    fn inc_tracks(&self, count: u64) {
        self.tracks_created.fetch_add(count, Ordering::Relaxed);
    }

    #[allow(dead_code)]
    fn inc_skipped(&self) {
        self.files_skipped.fetch_add(1, Ordering::Relaxed);
    }

    fn inc_errors(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    fn inc_repaired(&self) {
        self.files_repaired.fetch_add(1, Ordering::Relaxed);
    }

    #[allow(dead_code)]
    fn inc_corrupt(&self) {
        self.files_corrupt.fetch_add(1, Ordering::Relaxed);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("🎵 BATCH MIDI SPLIT TOOL");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Output directory: {:?}", args.output_dir);
    println!("Workers: {}", args.workers);
    println!();

    // Create output directory
    if !args.output_dir.exists() {
        println!("📁 Creating output directory...");
        fs::create_dir_all(&args.output_dir).await?;
    }

    // Connect to database
    println!("🔌 Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(args.workers as u32 + 5)
        .connect(&args.database_url)
        .await?;
    println!("✅ Database connected");

    // Count multi-track files
    let count_query = if let Some(limit) = args.test_limit {
        format!(
            "SELECT COUNT(*) as count FROM (SELECT id FROM files WHERE num_tracks > 1 LIMIT {}) as limited",
            limit
        )
    } else {
        "SELECT COUNT(*) as count FROM files WHERE num_tracks > 1".to_string()
    };

    let count: i64 = sqlx::query_scalar(&count_query).fetch_one(&pool).await?;

    println!("📊 Found {} multi-track files to split", count);
    println!();

    if count == 0 {
        println!("✅ No files to split!");
        return Ok(());
    }

    // Create progress bar
    let progress = Arc::new(ProgressBar::new(count as u64));
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) - {msg}")?
            .progress_chars("█▓▒░ "),
    );

    // Initialize stats
    let stats = Arc::new(Stats::new());

    // Process files in batches
    let mut offset = 0i64;
    loop {
        let files = fetch_batch(&pool, offset, args.batch_size, args.test_limit).await?;
        if files.is_empty() {
            break;
        }

        // Process batch in parallel
        let mut tasks = Vec::new();
        for file in files {
            let pool = pool.clone();
            let output_dir = args.output_dir.clone();
            let stats = stats.clone();
            let progress = progress.clone();

            let task = task::spawn(async move {
                match process_file(&pool, &output_dir, file).await {
                    Ok((track_count, was_repaired)) => {
                        stats.inc_processed();
                        stats.inc_tracks(track_count as u64);
                        if was_repaired {
                            stats.inc_repaired();
                        }
                        progress.inc(1);
                    },
                    Err(e) => {
                        stats.inc_errors();
                        progress.inc(1);
                        eprintln!("Error: {}", e);
                    },
                }
            });
            tasks.push(task);
        }

        // Wait for batch to complete
        futures::future::join_all(tasks).await;

        offset += args.batch_size;
    }

    progress.finish_with_message("Complete!");
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ SPLIT COMPLETE WITH AUTO-REPAIR");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!(
        "Files processed: {}",
        stats.files_processed.load(Ordering::Relaxed)
    );
    println!(
        "Tracks created:  {}",
        stats.tracks_created.load(Ordering::Relaxed)
    );
    println!(
        "🔧 Files repaired: {}",
        stats.files_repaired.load(Ordering::Relaxed)
    );
    println!(
        "❌ Files corrupt:  {}",
        stats.files_corrupt.load(Ordering::Relaxed)
    );
    println!(
        "Files skipped:   {}",
        stats.files_skipped.load(Ordering::Relaxed)
    );
    println!("Errors:          {}", stats.errors.load(Ordering::Relaxed));
    println!();

    Ok(())
}

async fn fetch_batch(
    pool: &Pool<Postgres>,
    offset: i64,
    limit: i64,
    test_limit: Option<i64>,
) -> Result<Vec<MultiTrackFile>> {
    let _query = if let Some(test_limit) = test_limit {
        format!(
            "SELECT id, filepath, filename, num_tracks
             FROM files
             WHERE num_tracks > 1
             ORDER BY id
             LIMIT {} OFFSET {}",
            std::cmp::min(limit, test_limit - offset),
            offset
        )
    } else {
        format!(
            "SELECT id, filepath, filename, num_tracks
             FROM files
             WHERE num_tracks > 1
             ORDER BY id
             LIMIT {} OFFSET {}",
            limit, offset
        )
    };

    let files = sqlx::query_as!(
        MultiTrackFile,
        r#"
        SELECT id, filepath, filename, num_tracks
        FROM files
        WHERE num_tracks > 1
        ORDER BY id
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    Ok(files)
}

/// Result of processing a file: (track_count, was_repaired)
type ProcessResult = (usize, bool);

async fn process_file(
    pool: &Pool<Postgres>,
    output_dir: &Path,
    file: MultiTrackFile,
) -> Result<ProcessResult> {
    // Read MIDI file
    let midi_bytes = fs::read(&file.filepath)
        .await
        .context(format!("Failed to read file: {}", file.filepath))?;

    // Split tracks with automatic repair
    let (split_tracks, repair_result) = split_tracks_with_repair(&midi_bytes)
        .map_err(|e| anyhow::anyhow!("Failed to split {}: {}", file.filename, e))?;

    // Track repair status and log
    let was_repaired = match &repair_result {
        RepairResult::Valid => {
            // File was valid, no repair needed
            false
        },
        RepairResult::Repaired { fix_description, .. } => {
            println!("🔧 REPAIRED: {} - {}", file.filename, fix_description);
            true
        },
        RepairResult::Corrupt { reason } => {
            eprintln!("❌ CORRUPT: {} - {}", file.filename, reason);
            false
        },
    };

    let track_count = split_tracks.len();

    // Save each split track
    for (_idx, split_track) in split_tracks.iter().enumerate() {
        // Generate filename
        let base_name = Path::new(&file.filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("track");

        let track_name = split_track
            .track_name
            .as_ref()
            .map(|n| sanitize_filename(n))
            .unwrap_or_default();

        let output_filename = if track_name.is_empty() {
            format!("{}_{:02}.mid", base_name, split_track.track_number + 1)
        } else {
            format!(
                "{}_{:02}_{}.mid",
                base_name,
                split_track.track_number + 1,
                track_name
            )
        };

        // Include parent file ID in path to avoid collisions from different parent files
        let output_path = output_dir.join(format!("{}_{}", file.id, &output_filename));

        // Write file
        fs::write(&output_path, &split_track.midi_bytes).await?;

        // Calculate hash
        let hash = blake3::hash(&split_track.midi_bytes);

        // Insert split file into files table
        // Content-based deduplication via content_hash
        let insert_result: Result<Option<(i64,)>, sqlx::Error> = sqlx::query_as(
            r#"
            INSERT INTO files (
                filename, filepath, original_filename,
                content_hash, file_size_bytes, num_tracks
            ) VALUES ($1, $2, $3, $4, $5, 1)
            ON CONFLICT (content_hash) DO NOTHING
            RETURNING id
            "#,
        )
        .bind(&output_filename)
        .bind(output_path.to_str().unwrap_or(""))
        .bind(&output_filename)
        .bind(hash.as_bytes())
        .bind(split_track.midi_bytes.len() as i64)
        .fetch_optional(pool)
        .await;

        match insert_result {
            Ok(Some((split_file_id,))) => {
                // Insert into track_splits to link parent and split file
                let track_name = split_track.track_name.clone();
                let instrument = split_track.instrument.clone();
                let note_count = split_track.note_count as i32;

                let _ = sqlx::query!(
                    r#"
                    INSERT INTO track_splits (
                        parent_file_id, split_file_id, track_number,
                        track_name, instrument, note_count
                    ) VALUES ($1, $2, $3, $4, $5, $6)
                    ON CONFLICT (parent_file_id, split_file_id) DO NOTHING
                    "#,
                    file.id,
                    split_file_id,
                    split_track.track_number as i32,
                    track_name,
                    instrument,
                    note_count
                )
                .execute(pool)
                .await;
            },
            Ok(None) => {
                // Duplicate (content_hash conflict) - delete file
                let _ = fs::remove_file(&output_path).await;
            },
            Err(e) => {
                eprintln!("⚠️  Database error for {}: {}", output_filename, e);
            },
        }
    }

    Ok((track_count, was_repaired))
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

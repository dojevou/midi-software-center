// orchestrator.rs - Intelligent pipeline orchestrator
// Coordinates import, analysis, splitting, and renaming phases with optimal parallelization

// Use jemalloc for better multi-threaded performance
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use anyhow::{Context, Result};
use blake3;
use clap::Parser;
use crossbeam_channel::bounded;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use midi_library_shared::core::midi::parser::parse_midi_file;
use midi_pipeline::core::analysis::chord_analyzer::analyze_chords;
use midi_pipeline::core::analysis::{detect_bpm, detect_key};
use midi_pipeline::core::splitting::track_splitter::split_tracks;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;
use tracing::error;

#[derive(Parser, Debug)]
#[command(name = "orchestrator")]
#[command(
    about = "MIDI Pipeline Orchestrator - Parallel import, analysis, splitting, and renaming"
)]
struct Args {
    /// Source directory containing MIDI files
    #[arg(short, long)]
    source: PathBuf,

    /// Number of analysis workers (default: CPU count)
    #[arg(short = 'w', long, default_value_t = num_cpus::get())]
    workers: usize,

    /// Batch size for database inserts
    #[arg(short = 'b', long, default_value_t = 1000)]
    batch_size: usize,

    /// Skip import if files already exist
    #[arg(long)]
    skip_import: bool,

    /// Skip analysis phase
    #[arg(long)]
    skip_analysis: bool,

    /// Skip track splitting phase
    #[arg(long)]
    skip_split: bool,

    /// Skip file renaming phase
    #[arg(long)]
    skip_rename: bool,
}

#[derive(Debug, Clone)]
struct FileRecord {
    id: i64,
    filepath: String,
    filename: String,
}

#[derive(Debug, Clone)]
struct MultiTrackRecord {
    id: i64,
    filepath: String,
    filename: String,
    num_tracks: i32,
}

#[derive(Debug)]
struct Stats {
    files_imported: AtomicU64,
    files_analyzed: AtomicU64,
    files_split: AtomicU64,
    tracks_created: AtomicU64,
    files_renamed: AtomicU64,
    import_errors: AtomicU64,
    analysis_errors: AtomicU64,
    split_errors: AtomicU64,
    rename_errors: AtomicU64,
    start_time: Instant,
}

impl Stats {
    fn new() -> Self {
        Self {
            files_imported: AtomicU64::new(0),
            files_analyzed: AtomicU64::new(0),
            files_split: AtomicU64::new(0),
            tracks_created: AtomicU64::new(0),
            files_renamed: AtomicU64::new(0),
            import_errors: AtomicU64::new(0),
            analysis_errors: AtomicU64::new(0),
            split_errors: AtomicU64::new(0),
            rename_errors: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    fn import_file(&self) {
        self.files_imported.fetch_add(1, Ordering::Relaxed);
    }

    fn analyze_file(&self) {
        self.files_analyzed.fetch_add(1, Ordering::Relaxed);
    }

    fn split_file(&self) {
        self.files_split.fetch_add(1, Ordering::Relaxed);
    }

    fn create_track(&self, count: u64) {
        self.tracks_created.fetch_add(count, Ordering::Relaxed);
    }

    fn rename_file(&self) {
        self.files_renamed.fetch_add(1, Ordering::Relaxed);
    }

    fn import_error(&self) {
        self.import_errors.fetch_add(1, Ordering::Relaxed);
    }

    fn analysis_error(&self) {
        self.analysis_errors.fetch_add(1, Ordering::Relaxed);
    }

    fn split_error(&self) {
        self.split_errors.fetch_add(1, Ordering::Relaxed);
    }

    fn rename_error(&self) {
        self.rename_errors.fetch_add(1, Ordering::Relaxed);
    }

    fn get_import_count(&self) -> u64 {
        self.files_imported.load(Ordering::Relaxed)
    }

    fn get_analysis_count(&self) -> u64 {
        self.files_analyzed.load(Ordering::Relaxed)
    }

    fn get_split_count(&self) -> u64 {
        self.files_split.load(Ordering::Relaxed)
    }

    fn get_tracks_count(&self) -> u64 {
        self.tracks_created.load(Ordering::Relaxed)
    }

    fn get_renamed_count(&self) -> u64 {
        self.files_renamed.load(Ordering::Relaxed)
    }

    fn get_import_errors(&self) -> u64 {
        self.import_errors.load(Ordering::Relaxed)
    }

    fn get_analysis_errors(&self) -> u64 {
        self.analysis_errors.load(Ordering::Relaxed)
    }

    fn get_split_errors(&self) -> u64 {
        self.split_errors.load(Ordering::Relaxed)
    }

    fn get_rename_errors(&self) -> u64 {
        self.rename_errors.load(Ordering::Relaxed)
    }

    fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // Validate source directory
    if !args.source.exists() {
        anyhow::bail!("Source directory does not exist: {:?}", args.source);
    }

    println!("🎵 MIDI Pipeline Orchestrator");
    println!("==============================");
    println!("Source: {:?}", args.source);
    println!("Workers: {}", args.workers);
    println!("Batch size: {}", args.batch_size);
    println!();

    // Connect to database
    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL not set")?;

    // Optimize connection pool for parallel analysis
    // One connection per worker ensures no contention; kept warm and reused indefinitely
    let worker_connections = args.workers as u32;
    let pool = PgPoolOptions::new()
        // Max connections: one per worker + 2 for import/split phases
        .max_connections(worker_connections + 2)
        // Min connections: maintain pool at worker capacity for warm connections
        .min_connections(worker_connections)
        // Acquire timeout: allow 30s for connection checkout to avoid busy-wait
        .acquire_timeout(Duration::from_secs(30))
        // Idle timeout: None = never close idle connections (reuse indefinitely)
        .idle_timeout(None)
        // Max lifetime: None = reuse connections indefinitely (reduces overhead)
        .max_lifetime(None)
        .connect(&database_url)
        .await
        .context("Failed to connect to database")?;

    println!("✅ Connected to database");
    println!("📊 Connection pool: {} workers × 1 connection (+ 2 utility connections)", args.workers);
    println!("⚡ Pool config: keep-warm, indefinite reuse");
    println!();

    let stats = Arc::new(Stats::new());
    let shutdown = Arc::new(AtomicBool::new(false));
    let multi_progress = Arc::new(MultiProgress::new());

    // Phase 1: File Renaming (filesystem-level, no database)
    let rename_handle = if !args.skip_rename {
        Some(spawn_rename_filesystem_phase(
            args.source.clone(),
            args.workers,
            stats.clone(),
            shutdown.clone(),
            multi_progress.clone(),
        ))
    } else {
        println!("⏭️  Skipping rename phase");
        None
    };

    // Wait for renaming to complete
    if let Some(handle) = rename_handle {
        handle.await??;
        println!("✅ Rename phase complete");
    }

    // Phase 2: Import
    let import_handle = if !args.skip_import {
        Some(spawn_import_phase(
            args.source.clone(),
            pool.clone(),
            args.batch_size,
            stats.clone(),
            shutdown.clone(),
            multi_progress.clone(),
        ))
    } else {
        println!("⏭️  Skipping import phase");
        None
    };

    // Wait for import to complete
    if let Some(handle) = import_handle {
        handle.await??;
        println!("✅ Import phase complete");
    }

    // Phase 3: Track Splitting
    let split_handles = if !args.skip_split {
        spawn_split_phase(
            pool.clone(),
            args.workers,
            stats.clone(),
            shutdown.clone(),
            multi_progress.clone(),
        )
        .await?
    } else {
        println!("⏭️  Skipping split phase");
        Vec::new()
    };

    // Wait for splitting to complete
    for handle in split_handles {
        handle.await??;
    }

    if !args.skip_split {
        println!("✅ Split phase complete");
    }

    // Phase 4: Analysis (analyzes BOTH parent and split files)
    let analysis_handles = if !args.skip_analysis {
        tokio::time::sleep(Duration::from_secs(2)).await;
        spawn_analysis_phase(
            pool.clone(),
            args.workers,
            stats.clone(),
            shutdown.clone(),
            multi_progress.clone(),
        )
        .await?
    } else {
        println!("⏭️  Skipping analysis phase");
        Vec::new()
    };

    // Wait for analysis to complete
    for handle in analysis_handles {
        handle.await??;
    }

    if !args.skip_analysis {
        println!("✅ Analysis phase complete");
    }

    println!();
    println!("🎉 Pipeline Complete!");
    println!("======================");
    println!("Files imported:   {}", stats.get_import_count());
    println!("Files analyzed:   {}", stats.get_analysis_count());
    println!("Files split:      {}", stats.get_split_count());
    println!("Tracks created:   {}", stats.get_tracks_count());
    println!("Files renamed:    {}", stats.get_renamed_count());
    println!();
    println!("Import errors:    {}", stats.get_import_errors());
    println!("Analysis errors:  {}", stats.get_analysis_errors());
    println!("Split errors:     {}", stats.get_split_errors());
    println!("Rename errors:    {}", stats.get_rename_errors());
    println!();
    println!("Total time:       {:.1}s", stats.elapsed().as_secs_f64());
    println!();

    Ok(())
}

fn spawn_import_phase(
    source: PathBuf,
    pool: Pool<Postgres>,
    batch_size: usize,
    stats: Arc<Stats>,
    shutdown: Arc<AtomicBool>,
    multi_progress: Arc<MultiProgress>,
) -> JoinHandle<Result<()>> {
    tokio::spawn(async move {
        let pb = multi_progress.add(ProgressBar::new(0));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("=>-"),
        );
        pb.set_message("Importing files...");

        let files = scan_directory(&source)?;
        pb.set_length(files.len() as u64);

        for batch in files.chunks(batch_size) {
            if shutdown.load(Ordering::Relaxed) {
                break;
            }
            import_batch(&pool, batch, &stats, &pb).await?;
        }

        pb.finish_with_message("Import complete");
        Ok(())
    })
}

async fn spawn_analysis_phase(
    pool: Pool<Postgres>,
    worker_count: usize,
    stats: Arc<Stats>,
    shutdown: Arc<AtomicBool>,
    multi_progress: Arc<MultiProgress>,
) -> Result<Vec<JoinHandle<Result<()>>>> {
    let mut handles = Vec::new();
    let (tx, rx) = bounded::<FileRecord>(worker_count * 2);

    let fetcher_pool = pool.clone();
    let fetcher_shutdown = shutdown.clone();

    let fetcher_handle = tokio::spawn(async move {
        loop {
            if fetcher_shutdown.load(Ordering::Relaxed) {
                break;
            }

            let files = fetch_unanalyzed_files(&fetcher_pool, 100).await?;

            if files.is_empty() {
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }

            for file in files {
                if fetcher_shutdown.load(Ordering::Relaxed) {
                    break;
                }
                if tx.send(file).is_err() {
                    break;
                }
            }
        }
        drop(tx);
        Ok::<(), anyhow::Error>(())
    });

    handles.push(fetcher_handle);

    for worker_id in 0..worker_count {
        let worker_pool = pool.clone();
        let worker_rx = rx.clone();
        let worker_stats = stats.clone();
        let worker_shutdown = shutdown.clone();
        let worker_mp = multi_progress.clone();

        let handle = tokio::spawn(async move {
            let pb = worker_mp.add(ProgressBar::new(0));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template(&format!(
                        "[Worker {}] {{spinner:.green}} {{msg}}",
                        worker_id
                    ))
                    .unwrap(),
            );

            while let Ok(file) = worker_rx.recv() {
                if worker_shutdown.load(Ordering::Relaxed) {
                    break;
                }

                pb.set_message(format!("Analyzing {}", file.filename));

                if let Err(e) = analyze_file(&worker_pool, &file).await {
                    error!("Analysis failed for {}: {}", file.filename, e);
                    worker_stats.analysis_error();
                } else {
                    worker_stats.analyze_file();
                }
            }

            pb.finish_with_message("Worker finished");
            Ok(())
        });

        handles.push(handle);
    }

    Ok(handles)
}

async fn spawn_split_phase(
    pool: Pool<Postgres>,
    worker_count: usize,
    stats: Arc<Stats>,
    shutdown: Arc<AtomicBool>,
    multi_progress: Arc<MultiProgress>,
) -> Result<Vec<JoinHandle<Result<()>>>> {
    let mut handles = Vec::new();
    let (tx, rx) = bounded::<MultiTrackRecord>(worker_count * 2);

    let fetcher_pool = pool.clone();
    let fetcher_shutdown = shutdown.clone();

    let fetcher_handle = tokio::spawn(async move {
        loop {
            if fetcher_shutdown.load(Ordering::Relaxed) {
                break;
            }

            let files = fetch_multitrack_files(&fetcher_pool, 50).await?;

            if files.is_empty() {
                break; // No more multi-track files to process
            }

            for file in files {
                if fetcher_shutdown.load(Ordering::Relaxed) {
                    break;
                }
                if tx.send(file).is_err() {
                    break;
                }
            }
        }
        drop(tx);
        Ok::<(), anyhow::Error>(())
    });

    handles.push(fetcher_handle);

    for worker_id in 0..worker_count {
        let worker_pool = pool.clone();
        let worker_rx = rx.clone();
        let worker_stats = stats.clone();
        let worker_shutdown = shutdown.clone();
        let worker_mp = multi_progress.clone();

        let handle = tokio::spawn(async move {
            let pb = worker_mp.add(ProgressBar::new(0));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template(&format!("[Split {}] {{spinner:.cyan}} {{msg}}", worker_id))
                    .unwrap(),
            );

            while let Ok(file) = worker_rx.recv() {
                if worker_shutdown.load(Ordering::Relaxed) {
                    break;
                }

                pb.set_message(format!("Splitting {}", file.filename));

                match split_multitrack_file(&worker_pool, &file).await {
                    Ok(track_count) => {
                        worker_stats.split_file();
                        worker_stats.create_track(track_count);
                    },
                    Err(e) => {
                        error!("Split failed for {}: {}", file.filename, e);
                        worker_stats.split_error();
                    },
                }
            }

            pb.finish_with_message("Split worker finished");
            Ok(())
        });

        handles.push(handle);
    }

    Ok(handles)
}

fn spawn_rename_filesystem_phase(
    source: PathBuf,
    worker_count: usize,
    stats: Arc<Stats>,
    shutdown: Arc<AtomicBool>,
    multi_progress: Arc<MultiProgress>,
) -> JoinHandle<Result<()>> {
    tokio::spawn(async move {
        let pb = multi_progress.add(ProgressBar::new(0));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("=>-"),
        );
        pb.set_message("Scanning and renaming files...");

        // Scan directory for all MIDI files
        let files = scan_directory(&source)?;
        pb.set_length(files.len() as u64);

        // Rename files in parallel batches
        use rayon::prelude::*;

        files.par_iter().for_each(|filepath| {
            if shutdown.load(Ordering::Relaxed) {
                return;
            }

            match rename_file_filesystem(filepath) {
                Ok(renamed) => {
                    if renamed {
                        stats.rename_file();
                    }
                    pb.inc(1);
                },
                Err(e) => {
                    error!("Rename failed for {:?}: {}", filepath, e);
                    stats.rename_error();
                    pb.inc(1);
                },
            }
        });

        pb.finish_with_message("Rename complete");
        Ok(())
    })
}

async fn spawn_rename_phase(
    pool: Pool<Postgres>,
    worker_count: usize,
    stats: Arc<Stats>,
    shutdown: Arc<AtomicBool>,
    multi_progress: Arc<MultiProgress>,
) -> Result<Vec<JoinHandle<Result<()>>>> {
    let mut handles = Vec::new();
    let (tx, rx) = bounded::<FileRecord>(worker_count * 2);

    let fetcher_pool = pool.clone();
    let fetcher_shutdown = shutdown.clone();

    let fetcher_handle = tokio::spawn(async move {
        loop {
            if fetcher_shutdown.load(Ordering::Relaxed) {
                break;
            }

            let files = fetch_all_files(&fetcher_pool, 100).await?;

            if files.is_empty() {
                break; // No more files to process
            }

            for file in files {
                if fetcher_shutdown.load(Ordering::Relaxed) {
                    break;
                }
                if tx.send(file).is_err() {
                    break;
                }
            }
        }
        drop(tx);
        Ok::<(), anyhow::Error>(())
    });

    handles.push(fetcher_handle);

    for worker_id in 0..worker_count {
        let worker_pool = pool.clone();
        let worker_rx = rx.clone();
        let worker_stats = stats.clone();
        let worker_shutdown = shutdown.clone();
        let worker_mp = multi_progress.clone();

        let handle = tokio::spawn(async move {
            let pb = worker_mp.add(ProgressBar::new(0));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template(&format!(
                        "[Rename {}] {{spinner:.yellow}} {{msg}}",
                        worker_id
                    ))
                    .unwrap(),
            );

            while let Ok(file) = worker_rx.recv() {
                if worker_shutdown.load(Ordering::Relaxed) {
                    break;
                }

                pb.set_message(format!("Renaming {}", file.filename));

                if let Err(e) = rename_file(&worker_pool, &file).await {
                    error!("Rename failed for {}: {}", file.filename, e);
                    worker_stats.rename_error();
                } else {
                    worker_stats.rename_file();
                }
            }

            pb.finish_with_message("Rename worker finished");
            Ok(())
        });

        handles.push(handle);
    }

    Ok(handles)
}

fn scan_directory(path: &Path) -> Result<Vec<PathBuf>> {
    use jwalk::WalkDir;

    let files: Vec<PathBuf> = WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("mid") || e.eq_ignore_ascii_case("midi"))
                .unwrap_or(false)
        })
        .map(|entry| entry.path())
        .collect();

    Ok(files)
}

async fn import_batch(
    pool: &Pool<Postgres>,
    files: &[PathBuf],
    stats: &Stats,
    pb: &ProgressBar,
) -> Result<()> {
    for file in files {
        match import_single_file(pool, file).await {
            Ok(_) => {
                stats.import_file();
                pb.inc(1);
            },
            Err(e) => {
                error!("Import failed for {:?}: {}", file, e);
                stats.import_error();
            },
        }
    }
    Ok(())
}

async fn import_single_file(pool: &Pool<Postgres>, filepath: &Path) -> Result<i64> {
    let filepath_str = filepath.to_string_lossy().to_string();
    let filename = filepath
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.mid")
        .to_string();

    // Read file to get size and hash
    let file_bytes = std::fs::read(filepath)?;
    let file_size_bytes = file_bytes.len() as i64;

    // Calculate BLAKE3 hash for deduplication
    let content_hash = blake3::hash(&file_bytes).as_bytes().to_vec();

    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (filename, filepath, original_filename, content_hash, file_size_bytes)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (filepath) DO UPDATE SET
            filename = EXCLUDED.filename,
            original_filename = EXCLUDED.original_filename,
            content_hash = EXCLUDED.content_hash,
            file_size_bytes = EXCLUDED.file_size_bytes
         RETURNING id"
    )
    .bind(&filename)
    .bind(&filepath_str)
    .bind(&filename)  // original_filename = filename for direct files
    .bind(&content_hash)
    .bind(file_size_bytes)
    .fetch_one(pool)
    .await?;

    Ok(file_id)
}

async fn fetch_unanalyzed_files(pool: &Pool<Postgres>, limit: i64) -> Result<Vec<FileRecord>> {
    let files = sqlx::query_as!(
        FileRecord,
        "SELECT id, filepath, filename FROM files WHERE analyzed_at IS NULL ORDER BY id LIMIT $1",
        limit
    )
    .fetch_all(pool)
    .await?;

    Ok(files)
}

async fn fetch_multitrack_files(
    pool: &Pool<Postgres>,
    limit: i64,
) -> Result<Vec<MultiTrackRecord>> {
    // Query for files that have num_tracks > 1 and have been analyzed but not yet split
    let files = sqlx::query!(
        "SELECT id, filepath, filename, num_tracks
         FROM files
         WHERE num_tracks > 1
           AND analyzed_at IS NOT NULL
           AND (is_multi_track IS NULL OR is_multi_track = false)
         ORDER BY id
         LIMIT $1",
        limit
    )
    .fetch_all(pool)
    .await?;

    Ok(files
        .into_iter()
        .map(|f| MultiTrackRecord {
            id: f.id,
            filepath: f.filepath,
            filename: f.filename,
            num_tracks: f.num_tracks as i32,
        })
        .collect())
}

async fn fetch_all_files(pool: &Pool<Postgres>, limit: i64) -> Result<Vec<FileRecord>> {
    let files = sqlx::query_as!(
        FileRecord,
        "SELECT id, filepath, filename FROM files ORDER BY id LIMIT $1",
        limit
    )
    .fetch_all(pool)
    .await?;

    Ok(files)
}

async fn analyze_file(pool: &Pool<Postgres>, file: &FileRecord) -> Result<()> {
    let midi_data = std::fs::read(&file.filepath)?;
    let midi_file = parse_midi_file(&midi_data)?;

    let bpm_result = detect_bpm(&midi_file);
    let tempo_bpm = if bpm_result.confidence > 0.3 {
        Some(bpm_result.bpm)
    } else {
        None
    };
    let has_tempo_variation = !bpm_result.metadata.is_constant;

    let key_result = detect_key(&midi_file);

    let ticks_per_quarter = midi_file.header.ticks_per_quarter_note as u32;
    let chord_analysis = analyze_chords(&midi_file, ticks_per_quarter);
    let chord_progression = if !chord_analysis.progression.is_empty() {
        Some(serde_json::json!(chord_analysis.progression))
    } else {
        None
    };

    let duration_seconds = Some(midi_file.duration_seconds(120.0));
    let num_tracks = midi_file.tracks.len() as i32;

    sqlx::query(
        "INSERT INTO musical_metadata (
            file_id, bpm, bpm_confidence, has_tempo_variation,
            detected_key, key_confidence, duration_seconds,
            chord_progression, chord_types,
            has_seventh_chords, has_extended_chords,
            chord_change_rate, chord_complexity_score
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        ON CONFLICT (file_id) DO UPDATE SET
            bpm = EXCLUDED.bpm,
            bpm_confidence = EXCLUDED.bpm_confidence,
            has_tempo_variation = EXCLUDED.has_tempo_variation,
            detected_key = EXCLUDED.detected_key,
            key_confidence = EXCLUDED.key_confidence,
            duration_seconds = EXCLUDED.duration_seconds,
            chord_progression = EXCLUDED.chord_progression,
            chord_types = EXCLUDED.chord_types,
            has_seventh_chords = EXCLUDED.has_seventh_chords,
            has_extended_chords = EXCLUDED.has_extended_chords,
            chord_change_rate = EXCLUDED.chord_change_rate,
            chord_complexity_score = EXCLUDED.chord_complexity_score",
    )
    .bind(file.id)
    .bind(tempo_bpm)
    .bind(Some(bpm_result.confidence))
    .bind(has_tempo_variation)
    .bind(&key_result.key)
    .bind(Some(key_result.confidence))
    .bind(duration_seconds)
    .bind(&chord_progression)
    .bind(&chord_analysis.types)
    .bind(chord_analysis.has_sevenths)
    .bind(chord_analysis.has_extended)
    .bind(chord_analysis.change_rate)
    .bind(Some(chord_analysis.complexity_score))
    .execute(pool)
    .await?;

    sqlx::query(
        "UPDATE files
         SET analyzed_at = NOW(), num_tracks = $2
         WHERE id = $1",
    )
    .bind(file.id)
    .bind(num_tracks)
    .execute(pool)
    .await?;

    Ok(())
}

async fn split_multitrack_file(pool: &Pool<Postgres>, file: &MultiTrackRecord) -> Result<u64> {
    // Read file bytes
    let file_bytes = fs::read(&file.filepath)?;

    // Split tracks
    let split_tracks =
        split_tracks(&file_bytes).map_err(|e| anyhow::anyhow!("Track split error: {}", e))?;

    let parent_path = Path::new(&file.filepath);
    let parent_dir = parent_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Cannot get parent directory"))?;
    let parent_stem = parent_path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");

    let mut tracks_created = 0u64;

    for split_track in split_tracks {
        // Determine instrument suffix
        let instrument_suffix = split_track
            .instrument
            .as_ref()
            .map(|i| format!("_{}", sanitize_filename(i)))
            .unwrap_or_else(|| {
                split_track
                    .track_name
                    .as_ref()
                    .map(|n| format!("_{}", sanitize_filename(n)))
                    .unwrap_or_default()
            });

        // Build new filename: parent_track{N}{_instrument}.mid
        let new_filename = format!(
            "{}_track{}{}{}",
            parent_stem, split_track.track_number, instrument_suffix, ".mid"
        );

        let new_filepath = parent_dir.join(&new_filename);
        let new_filepath_str = new_filepath.to_string_lossy().to_string();

        // Write split track to disk
        fs::write(&new_filepath, &split_track.midi_bytes)?;

        // Calculate hash for new file
        let content_hash = blake3::hash(&split_track.midi_bytes).as_bytes().to_vec();
        let file_size_bytes = split_track.midi_bytes.len() as i64;

        // Insert split file into database
        let split_file_id: i64 = sqlx::query_scalar(
            "INSERT INTO files (filename, filepath, original_filename, content_hash, file_size_bytes)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (filepath) DO UPDATE SET
                filename = EXCLUDED.filename,
                content_hash = EXCLUDED.content_hash,
                file_size_bytes = EXCLUDED.file_size_bytes
             RETURNING id"
        )
        .bind(&new_filename)
        .bind(&new_filepath_str)
        .bind(&new_filename)
        .bind(&content_hash)
        .bind(file_size_bytes)
        .fetch_one(pool)
        .await?;

        // Create track_splits record
        sqlx::query(
            "INSERT INTO track_splits (parent_file_id, split_file_id, track_number, track_name, instrument, note_count)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (parent_file_id, split_file_id) DO UPDATE SET
                track_number = EXCLUDED.track_number,
                track_name = EXCLUDED.track_name,
                instrument = EXCLUDED.instrument,
                note_count = EXCLUDED.note_count"
        )
        .bind(file.id)
        .bind(split_file_id)
        .bind(split_track.track_number as i32)
        .bind(&split_track.track_name)
        .bind(&split_track.instrument)
        .bind(split_track.note_count as i32)
        .execute(pool)
        .await?;

        tracks_created += 1;
    }

    // Mark parent file as multi-track
    sqlx::query("UPDATE files SET is_multi_track = true WHERE id = $1")
        .bind(file.id)
        .execute(pool)
        .await?;

    Ok(tracks_created)
}

/// Rename a file on the filesystem (no database operations)
/// Returns true if file was renamed, false if no rename needed
fn rename_file_filesystem(filepath: &Path) -> Result<bool> {
    let filename = filepath
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;

    let mut needs_rename = false;
    let mut new_filename = filename.to_string();

    // Check if extension is .midi (should be .mid)
    if let Some(ext) = filepath.extension().and_then(|e| e.to_str()) {
        if ext.eq_ignore_ascii_case("midi") {
            new_filename = new_filename.replace(".midi", ".mid").replace(".MIDI", ".mid");
            needs_rename = true;
        }
    }

    // Replace spaces with underscores
    if new_filename.contains(' ') {
        new_filename = new_filename.replace(' ', "_");
        needs_rename = true;
    }

    if !needs_rename {
        return Ok(false); // No rename needed
    }

    // Build new filepath
    let parent_dir = filepath
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Cannot get parent directory"))?;
    let new_filepath = parent_dir.join(&new_filename);

    // Rename physical file
    std::fs::rename(filepath, &new_filepath)?;

    Ok(true)
}

async fn rename_file(pool: &Pool<Postgres>, file: &FileRecord) -> Result<()> {
    let filepath = Path::new(&file.filepath);
    let filename = &file.filename;

    let mut needs_rename = false;
    let mut new_filename = filename.clone();

    // Check if extension is .midi (should be .mid)
    if let Some(ext) = filepath.extension().and_then(|e| e.to_str()) {
        if ext.eq_ignore_ascii_case("midi") {
            new_filename = new_filename.replace(".midi", ".mid").replace(".MIDI", ".mid");
            needs_rename = true;
        }
    }

    // Replace spaces with underscores
    if new_filename.contains(' ') {
        new_filename = new_filename.replace(' ', "_");
        needs_rename = true;
    }

    if !needs_rename {
        return Ok(()); // No rename needed
    }

    // Build new filepath
    let parent_dir = filepath
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Cannot get parent directory"))?;
    let new_filepath = parent_dir.join(&new_filename);
    let new_filepath_str = new_filepath.to_string_lossy().to_string();

    // Rename physical file
    fs::rename(filepath, &new_filepath)?;

    // Update database
    sqlx::query(
        "UPDATE files
         SET filename = $1, filepath = $2
         WHERE id = $3",
    )
    .bind(&new_filename)
    .bind(&new_filepath_str)
    .bind(file.id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Sanitize filename component by removing problematic characters
fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            ' ' => '_',
            _ => c,
        })
        .collect()
}

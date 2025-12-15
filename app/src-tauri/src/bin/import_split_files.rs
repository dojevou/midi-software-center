use clap::Parser;
use dashmap::DashMap;
use midly::{Smf, Timing};
use rayon::prelude::*;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Import split MIDI files into database with parallel processing
#[derive(Parser, Debug)]
#[command(name = "import_split_files")]
#[command(about = "Import split MIDI files into database", long_about = None)]
struct Args {
    /// Directory containing split files
    #[arg(short, long, default_value = "/home/dojevou/tmp/midi_splits_fast")]
    input_dir: String,

    /// Number of parallel workers
    #[arg(short, long, default_value = "16")]
    workers: usize,

    /// Batch size for database inserts
    #[arg(short, long, default_value = "1000")]
    batch_size: usize,

    /// Database URL
    #[arg(
        short,
        long,
        default_value = "postgresql://midiuser:145278963@localhost:5433/midi_library"
    )]
    database_url: String,

    /// Skip existing files (check by hash)
    #[arg(long, default_value = "true")]
    skip_existing: bool,
}

#[derive(Debug, Clone)]
struct FileRecord {
    filepath: String,
    filename: String,
    original_filename: String,
    content_hash: Vec<u8>,
    file_size_bytes: i64,
    parent_file_id: Option<i64>,
    track_number: Option<i16>,
    format: i16,
    num_tracks: i16,
    ticks_per_quarter_note: i32,
    duration_ticks: i64,
}

struct ImportStats {
    total_files: AtomicU64,
    processed: AtomicU64,
    inserted: AtomicU64,
    skipped: AtomicU64,
    errors: AtomicU64,
}

impl ImportStats {
    fn new() -> Self {
        Self {
            total_files: AtomicU64::new(0),
            processed: AtomicU64::new(0),
            inserted: AtomicU64::new(0),
            skipped: AtomicU64::new(0),
            errors: AtomicU64::new(0),
        }
    }

    fn print_progress(&self, start_time: Instant) {
        let processed = self.processed.load(Ordering::Relaxed);
        let inserted = self.inserted.load(Ordering::Relaxed);
        let skipped = self.skipped.load(Ordering::Relaxed);
        let errors = self.errors.load(Ordering::Relaxed);
        let total = self.total_files.load(Ordering::Relaxed);
        let elapsed = start_time.elapsed().as_secs_f64();
        let rate = if elapsed > 0.0 {
            processed as f64 / elapsed
        } else {
            0.0
        };

        println!(
            "Progress: {}/{} ({:.1}%) | Inserted: {} | Skipped: {} | Errors: {} | Rate: {:.0} files/sec",
            processed,
            total,
            (processed as f64 / total as f64) * 100.0,
            inserted,
            skipped,
            errors,
            rate
        );
    }
}

/// Calculate BLAKE3 hash of a file (returns raw bytes)
fn hash_file(path: &Path) -> Result<Vec<u8>, std::io::Error> {
    let mut file = fs::File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0; 65536]; // 64KB buffer

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize().as_bytes().to_vec())
}

/// Extract parent file ID and track number from split filename
/// Format: {parent_id}_{original_name}_{track_num}_{instrument}.mid
fn extract_parent_info(filename: &str) -> (Option<i64>, Option<i16>) {
    // Example: 6614730_mikeshiver-feelings_bren-f_03_Bass pad.mid
    if let Some(caps) = regex::Regex::new(r"^(\d+)_").unwrap().captures(filename) {
        let parent_id = caps.get(1).and_then(|m| m.as_str().parse::<i64>().ok());

        // Try to extract track number (look for _XX_ pattern)
        let track_number = regex::Regex::new(r"_(\d{2})_")
            .unwrap()
            .captures(filename)
            .and_then(|caps| caps.get(1))
            .and_then(|m| m.as_str().parse::<i16>().ok());

        return (parent_id, track_number);
    }

    (None, None)
}

/// Parse MIDI file for basic metadata
fn parse_midi_basic(path: &Path) -> Result<(i16, i16, i32, i64), String> {
    let data = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    let smf = Smf::parse(&data).map_err(|e| format!("Failed to parse MIDI: {}", e))?;

    let midi_format = match smf.header.format {
        midly::Format::SingleTrack => 0,
        midly::Format::Parallel => 1,
        midly::Format::Sequential => 2,
    };

    let num_tracks = smf.tracks.len() as i16;
    let ticks_per_quarter = match smf.header.timing {
        Timing::Metrical(tpq) => tpq.as_int() as i32,
        Timing::Timecode(fps, sub) => {
            // Convert timecode to ticks per quarter note (approximate)
            let ticks_per_second = fps.as_f32() * sub as f32;
            (ticks_per_second * 0.5) as i32 // Assume 120 BPM default
        },
    };

    // Calculate duration in ticks
    let mut max_ticks = 0i64;
    for track in &smf.tracks {
        let mut track_ticks = 0i64;
        for event in track {
            track_ticks += event.delta.as_int() as i64;
        }
        if track_ticks > max_ticks {
            max_ticks = track_ticks;
        }
    }

    Ok((midi_format, num_tracks, ticks_per_quarter, max_ticks))
}

/// Find all MIDI files in directory
fn find_midi_files(root: &Path) -> Vec<PathBuf> {
    println!("🔍 Scanning for MIDI files in: {}", root.display());
    let start = Instant::now();

    let files: Vec<PathBuf> = walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| {
                    let ext_lower = ext.to_lowercase();
                    ext_lower == "mid" || ext_lower == "midi"
                })
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    println!(
        "✅ Found {} MIDI files in {:.2}s",
        files.len(),
        start.elapsed().as_secs_f64()
    );
    files
}

/// Process files in parallel and extract metadata
fn process_files(
    files: Vec<PathBuf>,
    existing_hashes: Arc<DashMap<Vec<u8>, ()>>,
    stats: Arc<ImportStats>,
    skip_existing: bool,
) -> Vec<FileRecord> {
    println!("⚙️ Processing {} files with rayon...", files.len());

    let records: Vec<FileRecord> = files
        .par_iter()
        .filter_map(|path| {
            stats.processed.fetch_add(1, Ordering::Relaxed);

            // Calculate hash
            let content_hash = match hash_file(path) {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("❌ Hash error for {}: {}", path.display(), e);
                    stats.errors.fetch_add(1, Ordering::Relaxed);
                    return None;
                },
            };

            // Skip if already exists
            if skip_existing && existing_hashes.contains_key(&content_hash) {
                stats.skipped.fetch_add(1, Ordering::Relaxed);
                return None;
            }

            // Get file size
            let file_size_bytes = match fs::metadata(path) {
                Ok(m) => m.len() as i64,
                Err(e) => {
                    eprintln!("❌ Metadata error for {}: {}", path.display(), e);
                    stats.errors.fetch_add(1, Ordering::Relaxed);
                    return None;
                },
            };

            // Parse MIDI
            let (format, num_tracks, ticks_per_quarter, duration_ticks) =
                match parse_midi_basic(path) {
                    Ok(data) => data,
                    Err(e) => {
                        eprintln!("❌ MIDI parse error for {}: {}", path.display(), e);
                        stats.errors.fetch_add(1, Ordering::Relaxed);
                        return None;
                    },
                };

            // Extract parent info
            let filename = path.file_name()?.to_str()?.to_string();
            let (parent_file_id, track_number) = extract_parent_info(&filename);

            Some(FileRecord {
                filepath: path.to_str()?.to_string(),
                filename: filename.clone(),
                original_filename: filename,
                content_hash,
                file_size_bytes,
                parent_file_id,
                track_number,
                format,
                num_tracks,
                ticks_per_quarter_note: ticks_per_quarter,
                duration_ticks,
            })
        })
        .collect();

    println!("✅ Processed {} valid records", records.len());
    records
}

/// Batch insert records into database
async fn batch_insert_records(
    pool: &PgPool,
    records: Vec<FileRecord>,
    batch_size: usize,
    stats: Arc<ImportStats>,
) -> Result<(), sqlx::Error> {
    println!(
        "💾 Inserting {} records in batches of {}...",
        records.len(),
        batch_size
    );
    let start = Instant::now();

    for (batch_num, chunk) in records.chunks(batch_size).enumerate() {
        let mut tx = pool.begin().await?;

        for record in chunk {
            let result = sqlx::query!(
                r#"
                INSERT INTO files (
                    filepath, filename, original_filename, content_hash, file_size_bytes,
                    parent_file_id, track_number, format, num_tracks,
                    ticks_per_quarter_note, duration_ticks
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                ON CONFLICT (filepath) DO NOTHING
                RETURNING id
                "#,
                record.filepath,
                record.filename,
                record.original_filename,
                record.content_hash,
                record.file_size_bytes,
                record.parent_file_id,
                record.track_number,
                record.format,
                record.num_tracks,
                record.ticks_per_quarter_note,
                record.duration_ticks
            )
            .fetch_optional(&mut *tx)
            .await?;

            if result.is_some() {
                stats.inserted.fetch_add(1, Ordering::Relaxed);
            } else {
                stats.skipped.fetch_add(1, Ordering::Relaxed);
            }
        }

        tx.commit().await?;

        if (batch_num + 1) % 10 == 0 {
            println!(
                "  Batch {}/{} | Inserted: {} | Elapsed: {:.1}s",
                batch_num + 1,
                records.len().div_ceil(batch_size),
                stats.inserted.load(Ordering::Relaxed),
                start.elapsed().as_secs_f64()
            );
        }
    }

    println!(
        "✅ Database insert complete in {:.2}s",
        start.elapsed().as_secs_f64()
    );
    Ok(())
}

/// Load existing file hashes from database
async fn load_existing_hashes(pool: &PgPool) -> Result<Arc<DashMap<Vec<u8>, ()>>, sqlx::Error> {
    println!("📂 Loading existing file hashes from database...");
    let start = Instant::now();

    let hashes = Arc::new(DashMap::new());

    let rows = sqlx::query!("SELECT content_hash FROM files WHERE content_hash IS NOT NULL")
        .fetch_all(pool)
        .await?;

    for row in rows {
        hashes.insert(row.content_hash, ());
    }

    println!(
        "✅ Loaded {} existing hashes in {:.2}s",
        hashes.len(),
        start.elapsed().as_secs_f64()
    );

    Ok(hashes)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🚀 Split File Import Tool (Rust Edition)");
    println!("═══════════════════════════════════════════");
    println!("Input directory: {}", args.input_dir);
    println!("Workers: {}", args.workers);
    println!("Batch size: {}", args.batch_size);
    println!("Skip existing: {}", args.skip_existing);
    println!();

    // Set rayon thread pool
    rayon::ThreadPoolBuilder::new()
        .num_threads(args.workers)
        .build_global()
        .unwrap();

    // Connect to database
    println!("🔌 Connecting to database...");
    let pool = PgPoolOptions::new().max_connections(20).connect(&args.database_url).await?;
    println!("✅ Database connected");

    // Load existing hashes
    let existing_hashes = if args.skip_existing {
        load_existing_hashes(&pool).await?
    } else {
        Arc::new(DashMap::new())
    };

    // Find all MIDI files
    let files = find_midi_files(Path::new(&args.input_dir));

    if files.is_empty() {
        println!("⚠️ No MIDI files found!");
        return Ok(());
    }

    // Initialize stats
    let stats = Arc::new(ImportStats::new());
    stats.total_files.store(files.len() as u64, Ordering::Relaxed);

    // Process files in parallel
    let start_time = Instant::now();
    let records = process_files(
        files,
        existing_hashes,
        Arc::clone(&stats),
        args.skip_existing,
    );

    // Print processing stats
    stats.print_progress(start_time);
    println!();

    if records.is_empty() {
        println!("ℹ️ No new files to insert (all already exist)");
        return Ok(());
    }

    // Batch insert into database
    batch_insert_records(&pool, records, args.batch_size, Arc::clone(&stats)).await?;

    // Final stats
    println!();
    println!("═══════════════════════════════════════════");
    println!("✅ Import Complete!");
    stats.print_progress(start_time);
    println!("Total time: {:.2}s", start_time.elapsed().as_secs_f64());
    println!("═══════════════════════════════════════════");

    Ok(())
}

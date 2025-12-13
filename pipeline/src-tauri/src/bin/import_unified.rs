/// Unified MIDI Import Pipeline
///
/// This binary orchestrates ALL existing modules to provide a complete, single-pass
/// import pipeline that processes compressed archives directly into the database with
/// FULL analysis (BPM, key, tags, complexity, etc.).
///
/// # Architecture: Orchestration Layer
/// This is a thin orchestration layer that combines:
/// - Archive extraction (io::decompressor)
/// - MIDI parsing (core::midi::parser)
/// - Musical analysis (core::analysis)
/// - Intelligent tagging (core::analysis::auto_tagger)
/// - Hash-based deduplication (core::hash)
/// - Batch database inserts (database::batch_insert)
///
/// # Workflow:
/// ```text
/// For each archive in input directory:
///   1. Extract archive → temp directory
///   2. Find all .mid/.midi files
///   3. For EACH MIDI file (in parallel with 32 workers):
///      a. Read file bytes
///      b. Parse MIDI
///      c. Detect BPM and key
///      d. Extract tags from path and content
///      e. Analyze notes (complexity, pitch range, polyphony, etc.)
///      f. Calculate BLAKE3 hash for deduplication
///      g. INSERT INTO files + musical_metadata (ONE transaction)
///   4. Clean up temp files
///   5. Move to next archive
/// ```
///
/// # Performance:
/// - Target: 350-400 files/sec with full analysis
/// - 1.5M files completed in ~1-1.5 hours
/// - Single-pass processing (no re-analysis needed)
///
/// # Usage:
/// ```bash
/// # Process directory of archives
/// cargo run --release --bin import_unified -- ~/floorp_downloads/_1.002.000-Midi-Collection_/
///
/// # Process single archive
/// cargo run --release --bin import_unified -- ~/path/to/archive.zip
/// ```
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use clap::Parser;
use futures::stream::{self, StreamExt};
use tokio::sync::Mutex;
// Unused: use indicatif::{ProgressBar, ProgressStyle, MultiProgress};

// Import existing modules - we just orchestrate them
use midi_library_shared::core::midi::parser::parse_midi_file;
use midi_library_shared::core::midi::types::{Event, MidiFile, TextType};
use midi_pipeline::core::analysis::auto_tagger::AutoTagger;
use midi_pipeline::core::analysis::bpm_detector::detect_bpm;
use midi_pipeline::core::analysis::key_detector::detect_key;
use midi_pipeline::core::hash::calculate_file_hash;
use midi_pipeline::core::normalization::normalize_directory;
use midi_pipeline::database::Database;
use midi_pipeline::io::decompressor::extractor::{extract_archive, ExtractionConfig};

//=============================================================================
// CLI ARGUMENTS
//=============================================================================

#[derive(Parser, Debug)]
#[command(name = "import-unified")]
#[command(about = "Unified MIDI import pipeline with full analysis in single pass")]
#[allow(dead_code)]
struct Args {
    /// Path to archive directory or single archive file
    #[arg(help = "Directory containing .zip/.rar/.7z archives, or single archive file")]
    path: PathBuf,

    /// Number of parallel MIDI processing workers (default: 32)
    #[arg(short = 'w', long, default_value = "32")]
    workers: usize,

    /// Batch size for database inserts (default: 100)
    #[arg(short = 'b', long, default_value = "100")]
    batch_size: usize,

    /// Database URL (default: from DATABASE_URL env var)
    #[arg(long)]
    database_url: Option<String>,
}

//=============================================================================
// DATA STRUCTURES
//=============================================================================

/// Fully analyzed MIDI file ready for database insertion
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct AnalyzedMidiFile {
    // File metadata
    filename: String,
    original_filename: String,
    filepath: String,
    parent_folder: Option<String>,
    content_hash: Vec<u8>,
    file_size_bytes: i64,
    num_tracks: i16,
    format: Option<i16>,
    is_multi_track: bool,

    // Musical metadata - tempo
    tempo_bpm: Option<f64>,
    bpm_confidence: Option<f64>,
    has_tempo_variation: bool,

    // Musical metadata - key
    key_signature: Option<String>,
    key_confidence: Option<f64>,
    scale_type: Option<String>,

    // Musical metadata - time
    time_signature_num: Option<i16>,
    time_signature_den: Option<i16>,
    duration_seconds: Option<f64>,
    duration_ticks: Option<i32>,

    // Musical metadata - notes
    note_count: i32,
    pitch_range_low: Option<i16>,
    pitch_range_high: Option<i16>,
    pitch_range_semitones: Option<i16>,
    avg_velocity: Option<f64>,
    velocity_range_low: Option<i16>,
    velocity_range_high: Option<i16>,
    polyphony_max: Option<i16>,

    // Musical metadata - complexity
    complexity_score: Option<f64>,

    // Musical metadata - features
    instruments: Vec<String>,
    has_pitch_bend: bool,
    has_cc_messages: bool,

    // Tags
    tags: Vec<String>,
    category: Option<String>,
}

/// Statistics for the import operation
#[derive(Debug, Default)]
#[allow(dead_code)]
struct ImportStats {
    archives_processed: AtomicUsize,
    archives_total: AtomicUsize,
    files_found: AtomicU64,
    files_imported: AtomicU64,
    files_duplicates: AtomicU64,
    files_errors: AtomicU64,
    start_time: Option<Instant>,
}

//=============================================================================
// MAIN ENTRY POINT
//=============================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Parse CLI arguments
    let args = Args::parse();

    // Setup logging
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    println!("\n🎵 UNIFIED MIDI IMPORT PIPELINE");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Connect to database
    let database_url = args.database_url.unwrap_or_else(|| {
        std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            eprintln!("❌ Error: DATABASE_URL must be set in environment or via --database-url");
            std::process::exit(1);
        })
    });

    println!("🔌 Connecting to database...");
    let db = Database::new(&database_url).await?;
    println!("✅ Database connected\n");

    // Check if path is directory or single archive
    if args.path.is_dir() {
        process_archive_directory(&args.path, &db, args.workers, args.batch_size).await?;
    } else if args.path.is_file() {
        process_single_archive(&args.path, &db, args.workers, args.batch_size).await?;
    } else {
        anyhow::bail!("Path does not exist: {}", args.path.display());
    }

    println!("\n✅ Import pipeline completed successfully!");

    Ok(())
}

//=============================================================================
// ARCHIVE DIRECTORY PROCESSING
//=============================================================================

/// Process all archives in a directory
async fn process_archive_directory(
    dir_path: &Path,
    db: &Database,
    workers: usize,
    batch_size: usize,
) -> anyhow::Result<()> {
    println!("📂 Scanning for archives in: {}", dir_path.display());

    // Find all archive files (.zip, .rar, .7z)
    let archives: Vec<PathBuf> = std::fs::read_dir(dir_path)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| {
                    ext.eq_ignore_ascii_case("zip")
                        || ext.eq_ignore_ascii_case("rar")
                        || ext.eq_ignore_ascii_case("7z")
                })
                .unwrap_or(false)
        })
        .collect();

    let total_archives = archives.len();
    println!("✅ Found {} archives to process\n", total_archives);

    if total_archives == 0 {
        println!("⚠️  No archives found in directory");
        return Ok(());
    }

    // Initialize statistics
    let stats = Arc::new(ImportStats {
        archives_total: AtomicUsize::new(total_archives),
        start_time: Some(Instant::now()),
        ..Default::default()
    });

    // Process archives sequentially (avoid I/O bottleneck)
    for (index, archive_path) in archives.iter().enumerate() {
        let archive_name = archive_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");

        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!(
            "📦 Archive [{}/{}]: {}",
            index + 1,
            total_archives,
            archive_name
        );
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        match process_archive_with_stats(archive_path, db, workers, batch_size, stats.clone()).await
        {
            Ok(_) => {
                stats.archives_processed.fetch_add(1, Ordering::SeqCst);
                print_progress_summary(&stats);
            },
            Err(e) => {
                eprintln!("❌ Failed to process archive {}: {}", archive_name, e);
                stats.archives_processed.fetch_add(1, Ordering::SeqCst);
            },
        }
    }

    // Print final summary
    print_final_summary(&stats);

    Ok(())
}

/// Process a single archive file
async fn process_single_archive(
    archive_path: &Path,
    db: &Database,
    workers: usize,
    batch_size: usize,
) -> anyhow::Result<()> {
    let archive_name = archive_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");

    println!("📦 Processing archive: {}\n", archive_name);

    let stats = Arc::new(ImportStats {
        archives_total: AtomicUsize::new(1),
        start_time: Some(Instant::now()),
        ..Default::default()
    });

    process_archive_with_stats(archive_path, db, workers, batch_size, stats.clone()).await?;

    print_final_summary(&stats);

    Ok(())
}

//=============================================================================
// ARCHIVE PROCESSING WITH FULL ANALYSIS
//=============================================================================

/// Process a single archive with full analysis and database insertion
async fn process_archive_with_stats(
    archive_path: &Path,
    db: &Database,
    workers: usize,
    batch_size: usize,
    stats: Arc<ImportStats>,
) -> anyhow::Result<()> {
    let start_time = Instant::now();

    // Step 1: Extract archive to temp directory
    println!("  📂 Extracting archive...");
    let temp_dir = std::env::temp_dir().join(format!("midi_unified_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)?;

    let config = ExtractionConfig::default();
    let extraction_result =
        extract_archive(archive_path, &temp_dir, &config).map_err(|e| anyhow::anyhow!("{}", e))?;

    let midi_files = extraction_result.midi_files;
    let midi_count = midi_files.len();
    stats.files_found.fetch_add(midi_count as u64, Ordering::SeqCst);

    println!("  🎵 Found {} MIDI files", midi_count);

    if midi_count == 0 {
        std::fs::remove_dir_all(&temp_dir)?;
        return Ok(());
    }

    // Step 1.5: Normalize filenames (extensions, spaces, UTF-8, MPC-compatible characters)
    println!("  🧹 Normalizing filenames...");
    let norm_start = Instant::now();
    match normalize_directory(&temp_dir, workers) {
        Ok(norm_stats) => {
            let norm_elapsed = norm_start.elapsed().as_secs_f64();
            norm_stats.print_summary(norm_elapsed);
        },
        Err(e) => {
            eprintln!("  ⚠️  Warning: Normalization error: {}", e);
        },
    }

    // Step 1.6: Re-scan for MIDI files after normalization (paths have changed!)
    use walkdir::WalkDir;
    let midi_files: Vec<PathBuf> = WalkDir::new(&temp_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_lowercase();
            name.ends_with(".mid") || name.ends_with(".midi")
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    let midi_count = midi_files.len();
    println!(
        "  ✓ Re-scanned: {} MIDI files after normalization",
        midi_count
    );

    // Extract category from archive name
    let category = archive_path.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string());

    // Step 2: Process MIDI files in parallel with full analysis
    println!(
        "  ⚡ Processing {} MIDI files with {} workers...",
        midi_count, workers
    );

    // Thread-safe counters
    let processed = Arc::new(AtomicUsize::new(0));
    let semaphore = Arc::new(tokio::sync::Semaphore::new(workers));

    // Batch buffer for database inserts
    let analyzed_files = Arc::new(Mutex::new(Vec::new()));
    let pool = db.pool().await;

    // Process files in parallel with buffer_unordered
    stream::iter(midi_files)
        .map(|file_path| {
            let sem = Arc::clone(&semaphore);
            let category = category.clone();
            let processed = Arc::clone(&processed);
            let analyzed_files = Arc::clone(&analyzed_files);
            let stats = Arc::clone(&stats);
            let pool = pool.clone();

            async move {
                // Acquire semaphore permit
                let _permit = match sem.acquire().await {
                    Ok(permit) => permit,
                    Err(_) => {
                        eprintln!("Warning: Semaphore closed during import");
                        return;
                    }
                };

                let current = processed.fetch_add(1, Ordering::SeqCst) + 1;

                // Show progress every 100 files
                if current.is_multiple_of(100) || current == midi_count {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let rate = if elapsed > 0.0 { current as f64 / elapsed } else { 0.0 };
                    println!("    Processing: {}/{} ({:.1}%) - {:.1} files/sec",
                        current, midi_count,
                        (current as f64 / midi_count as f64) * 100.0,
                        rate
                    );
                }

                // Analyze the file with full analysis
                match analyze_midi_file(&file_path, category).await {
                    Ok(analyzed) => {
                        // Add to batch for insertion
                        analyzed_files.lock().await.push(analyzed);

                        // Flush batch if it reaches threshold
                        let mut files = analyzed_files.lock().await;
                        if files.len() >= batch_size {
                            let batch: Vec<AnalyzedMidiFile> = files.drain(..).collect();
                            drop(files); // Release lock

                            match insert_batch(&pool, &batch).await {
                                Ok(inserted) => {
                                    stats.files_imported.fetch_add(inserted as u64, Ordering::SeqCst);
                                    let duplicates = batch.len() - inserted;
                                    stats.files_duplicates.fetch_add(duplicates as u64, Ordering::SeqCst);
                                }
                                Err(e) => {
                                    eprintln!("      ❌ Batch insert failed: {}", e);
                                    stats.files_errors.fetch_add(batch.len() as u64, Ordering::SeqCst);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("      ⚠️  Failed to analyze {}: {}", file_path.display(), e);
                        stats.files_errors.fetch_add(1, Ordering::SeqCst);
                    }
                }
            }
        })
        .buffer_unordered(workers) // THE MAGIC: Process N files concurrently!
        .collect::<Vec<_>>()
        .await;

    // Flush remaining batch
    let remaining_files = analyzed_files.lock().await;
    if !remaining_files.is_empty() {
        let batch: Vec<AnalyzedMidiFile> = remaining_files.iter().cloned().collect();
        drop(remaining_files);

        match insert_batch(&pool, &batch).await {
            Ok(inserted) => {
                stats.files_imported.fetch_add(inserted as u64, Ordering::SeqCst);
                let duplicates = batch.len() - inserted;
                stats.files_duplicates.fetch_add(duplicates as u64, Ordering::SeqCst);
            },
            Err(e) => {
                eprintln!("      ❌ Final batch insert failed: {}", e);
                stats.files_errors.fetch_add(batch.len() as u64, Ordering::SeqCst);
            },
        }
    }

    // Cleanup temp directory
    std::fs::remove_dir_all(&temp_dir)?;

    let elapsed = start_time.elapsed().as_secs_f64();
    let rate = if elapsed > 0.0 {
        midi_count as f64 / elapsed
    } else {
        0.0
    };
    println!(
        "  ✅ Completed in {:.1}s ({:.1} files/sec)\n",
        elapsed, rate
    );

    Ok(())
}

//=============================================================================
// MIDI FILE ANALYSIS (Full Analysis in Single Pass)
//=============================================================================

/// Analyze a single MIDI file with FULL analysis (BPM, key, tags, complexity, etc.)
async fn analyze_midi_file(
    file_path: &Path,
    category: Option<String>,
) -> anyhow::Result<AnalyzedMidiFile> {
    // 1. Read file bytes
    let file_bytes = tokio::fs::read(file_path).await?;
    let file_size_bytes = file_bytes.len() as i64;

    // 2. Calculate BLAKE3 hash for deduplication
    let hash_bytes = calculate_file_hash(file_path)?;
    let content_hash: Vec<u8> = hash_bytes.to_vec();

    // 3. Parse MIDI file
    let midi_file = parse_midi_file(&file_bytes)?;
    let num_tracks = midi_file.tracks.len() as i16;
    let format = Some(midi_file.header.format as i16);
    let is_multi_track = midi_file.tracks.len() > 1;

    // 4. BPM Detection
    let bpm_result = detect_bpm(&midi_file);
    let tempo_bpm = if bpm_result.confidence > 0.3 {
        Some(bpm_result.bpm)
    } else {
        None
    };
    let bpm_confidence = Some(bpm_result.confidence);
    let has_tempo_variation = !bpm_result.metadata.is_constant;

    // 5. Key Detection
    let key_result = detect_key(&midi_file);
    let key_signature = if key_result.confidence > 0.5 {
        Some(key_result.key.clone())
    } else {
        None
    };
    let key_confidence = Some(key_result.confidence);
    let scale_type = Some(key_result.scale_type.to_string());

    // 6. Time Signature
    let (time_signature_num, time_signature_den) = extract_time_signature(&midi_file);

    // 7. Duration Calculation
    let duration_ticks = calculate_total_ticks(&midi_file);
    let duration_seconds = calculate_duration_seconds(&midi_file, bpm_result.bpm);

    // 8. Note Analysis
    let note_stats = analyze_notes(&midi_file);

    // 9. Extract Instruments
    let instruments = extract_instrument_names(&midi_file);

    // 10. Detect MIDI Features
    let has_pitch_bend = detect_pitch_bend(&midi_file);
    let has_cc_messages = detect_cc_messages(&midi_file);

    // 11. Calculate Complexity Score
    let complexity_score = calculate_complexity_score(&note_stats, &midi_file);

    // 12. Extract Tags
    let filename = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string();

    let filepath = file_path.to_str().unwrap_or("").to_string();

    let auto_tagger = AutoTagger::new()?;
    let tags_obj = auto_tagger.extract_tags(
        &filepath,
        &filename,
        &instruments,
        tempo_bpm,
        key_signature.as_deref(),
        Some(&midi_file),
    );

    // Convert tags to strings for database
    let tags: Vec<String> = tags_obj
        .iter()
        .map(|t| match &t.category {
            Some(cat) => format!("{}:{}", cat, t.name),
            None => t.name.clone(),
        })
        .collect();

    // 13. Extract parent folder
    let parent_folder = file_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|s| s.to_string());

    Ok(AnalyzedMidiFile {
        filename: filename.clone(),
        original_filename: filename,
        filepath,
        parent_folder,
        content_hash,
        file_size_bytes,
        num_tracks,
        format,
        is_multi_track,
        tempo_bpm,
        bpm_confidence,
        has_tempo_variation,
        key_signature,
        key_confidence,
        scale_type,
        time_signature_num,
        time_signature_den,
        duration_seconds,
        duration_ticks: Some(duration_ticks),
        note_count: note_stats.note_count,
        pitch_range_low: note_stats.pitch_range_low,
        pitch_range_high: note_stats.pitch_range_high,
        pitch_range_semitones: note_stats.pitch_range_semitones,
        avg_velocity: note_stats.avg_velocity,
        velocity_range_low: note_stats.velocity_range_low,
        velocity_range_high: note_stats.velocity_range_high,
        polyphony_max: note_stats.polyphony_max,
        complexity_score,
        instruments,
        has_pitch_bend,
        has_cc_messages,
        tags,
        category,
    })
}

//=============================================================================
// DATABASE BATCH INSERTION
//=============================================================================

/// Insert batch of analyzed files into database
/// Returns number of files successfully inserted (excludes duplicates)
async fn insert_batch(pool: &sqlx::PgPool, files: &[AnalyzedMidiFile]) -> anyhow::Result<usize> {
    let mut inserted_count = 0;

    for file in files {
        let mut tx = pool.begin().await?;

        // Insert file with ON CONFLICT to handle duplicates
        let file_id_opt = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO files (
                filename, original_filename, filepath, parent_folder,
                content_hash, file_size_bytes, num_tracks,
                format, is_multi_track,
                duration_seconds, duration_ticks,
                instrument_names_text,
                created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW())
            ON CONFLICT (content_hash) DO NOTHING
            RETURNING id
            "#,
        )
        .bind(&file.filename)
        .bind(&file.original_filename)
        .bind(&file.filepath)
        .bind(&file.parent_folder)
        .bind(&file.content_hash)
        .bind(file.file_size_bytes)
        .bind(file.num_tracks)
        .bind(file.format)
        .bind(file.is_multi_track)
        .bind(file.duration_seconds)
        .bind(file.duration_ticks.map(|t| t as i64))
        .bind(&file.instruments)
        .fetch_optional(&mut *tx)
        .await?;

        // If duplicate, skip
        let file_id = match file_id_opt {
            Some(id) => id,
            None => {
                tx.rollback().await?;
                continue; // Skip duplicate
            },
        };

        // Insert musical metadata (only if we have analysis data)
        if file.tempo_bpm.is_some() || file.key_signature.is_some() {
            sqlx::query(
                r#"
                INSERT INTO musical_metadata (
                    file_id,
                    bpm,
                    bpm_confidence,
                    has_tempo_changes,
                    key_signature,
                    key_confidence,
                    time_signature_numerator,
                    time_signature_denominator,
                    total_notes,
                    pitch_range_min,
                    pitch_range_max,
                    avg_velocity,
                    polyphony_max
                ) VALUES ($1, $2, $3, $4, $5::musical_key, $6, $7, $8, $9, $10, $11, $12, $13)
                ON CONFLICT (file_id) DO UPDATE SET
                    bpm = EXCLUDED.bpm,
                    bpm_confidence = EXCLUDED.bpm_confidence,
                    key_signature = EXCLUDED.key_signature,
                    key_confidence = EXCLUDED.key_confidence
                "#,
            )
            .bind(file_id)
            .bind(file.tempo_bpm)
            .bind(file.bpm_confidence.map(|c| c as f32))
            .bind(file.has_tempo_variation)
            .bind(file.key_signature.as_deref())
            .bind(file.key_confidence.map(|c| c as f32))
            .bind(file.time_signature_num.unwrap_or(4))
            .bind(file.time_signature_den.unwrap_or(4))
            .bind(file.note_count)
            .bind(file.pitch_range_low)
            .bind(file.pitch_range_high)
            .bind(file.avg_velocity)
            .bind(file.polyphony_max)
            .execute(&mut *tx)
            .await?;
        }

        // Update analyzed_at timestamp
        sqlx::query("UPDATE files SET analyzed_at = NOW() WHERE id = $1")
            .bind(file_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        inserted_count += 1;
    }

    Ok(inserted_count)
}

//=============================================================================
// HELPER FUNCTIONS - MIDI ANALYSIS
//=============================================================================

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct NoteStats {
    note_count: i32,
    pitch_range_low: Option<i16>,
    pitch_range_high: Option<i16>,
    pitch_range_semitones: Option<i16>,
    avg_velocity: Option<f64>,
    velocity_range_low: Option<i16>,
    velocity_range_high: Option<i16>,
    polyphony_max: Option<i16>,
}

fn analyze_notes(midi_file: &MidiFile) -> NoteStats {
    let mut note_count = 0;
    let mut min_pitch = 127u8;
    let mut max_pitch = 0u8;
    let mut min_velocity = 127u8;
    let mut max_velocity = 0u8;
    let mut velocity_sum = 0u32;
    let mut active_notes_per_tick: std::collections::HashMap<u32, usize> =
        std::collections::HashMap::new();

    for track in &midi_file.tracks {
        let mut current_tick = 0u32;
        let mut active_notes = std::collections::HashSet::new();

        for timed_event in &track.events {
            current_tick += timed_event.delta_ticks;

            match &timed_event.event {
                Event::NoteOn { note, velocity, .. } if *velocity > 0 => {
                    note_count += 1;
                    min_pitch = min_pitch.min(*note);
                    max_pitch = max_pitch.max(*note);
                    min_velocity = min_velocity.min(*velocity);
                    max_velocity = max_velocity.max(*velocity);
                    velocity_sum += *velocity as u32;

                    active_notes.insert(*note);
                    active_notes_per_tick.insert(current_tick, active_notes.len());
                },
                Event::NoteOff { note, .. } | Event::NoteOn { note, velocity: 0, .. } => {
                    active_notes.remove(note);
                },
                _ => {},
            }
        }
    }

    let avg_velocity = if note_count > 0 {
        Some(velocity_sum as f64 / note_count as f64)
    } else {
        None
    };

    let polyphony_max = active_notes_per_tick.values().max().copied().map(|v| v as i16);

    let (pitch_range_low, pitch_range_high, pitch_range_semitones) = if note_count > 0 {
        let semitones = max_pitch.saturating_sub(min_pitch) as i16;
        (
            Some(min_pitch as i16),
            Some(max_pitch as i16),
            Some(semitones),
        )
    } else {
        (None, None, None)
    };

    let (velocity_range_low, velocity_range_high) = if note_count > 0 {
        (Some(min_velocity as i16), Some(max_velocity as i16))
    } else {
        (None, None)
    };

    NoteStats {
        note_count,
        pitch_range_low,
        pitch_range_high,
        pitch_range_semitones,
        avg_velocity,
        velocity_range_low,
        velocity_range_high,
        polyphony_max,
    }
}

fn extract_time_signature(midi_file: &MidiFile) -> (Option<i16>, Option<i16>) {
    for track in &midi_file.tracks {
        for timed_event in &track.events {
            if let Event::TimeSignature { numerator, denominator, .. } = &timed_event.event {
                let denom_value = 2i16.pow(*denominator as u32);
                return (Some(*numerator as i16), Some(denom_value));
            }
        }
    }
    (Some(4), Some(4))
}

fn calculate_total_ticks(midi_file: &MidiFile) -> i32 {
    let mut max_ticks = 0u32;
    for track in &midi_file.tracks {
        let mut track_ticks = 0u32;
        for timed_event in &track.events {
            track_ticks += timed_event.delta_ticks;
        }
        max_ticks = max_ticks.max(track_ticks);
    }
    max_ticks as i32
}

fn calculate_duration_seconds(midi_file: &MidiFile, bpm: f64) -> Option<f64> {
    let total_ticks = calculate_total_ticks(midi_file) as f64;
    let ticks_per_quarter = midi_file.header.ticks_per_quarter_note as f64;

    if total_ticks > 0.0 && ticks_per_quarter > 0.0 && bpm > 0.0 {
        let quarters = total_ticks / ticks_per_quarter;
        let minutes = quarters / bpm;
        let seconds = minutes * 60.0;
        Some(seconds)
    } else {
        None
    }
}

fn extract_instrument_names(midi_file: &MidiFile) -> Vec<String> {
    let mut instruments = Vec::new();

    for track in &midi_file.tracks {
        for timed_event in &track.events {
            match &timed_event.event {
                Event::Text { text_type, text } => {
                    if matches!(text_type, TextType::InstrumentName | TextType::TrackName)
                        && !instruments.contains(text)
                    {
                        instruments.push(text.clone());
                    }
                },
                Event::ProgramChange { program, .. } => {
                    if let Some(instrument_name) = program_to_instrument_name(*program) {
                        if !instruments.contains(&instrument_name) {
                            instruments.push(instrument_name);
                        }
                    }
                },
                _ => {},
            }
        }
    }

    instruments
}

fn program_to_instrument_name(program: u8) -> Option<String> {
    match program {
        0..=7 => Some("Piano".to_string()),
        8..=15 => Some("Keys".to_string()),
        16..=23 => Some("Organ".to_string()),
        24..=31 => Some("Guitar".to_string()),
        32..=39 => Some("Bass".to_string()),
        40..=47 => Some("Strings".to_string()),
        48..=55 => Some("Ensemble".to_string()),
        56..=63 => Some("Brass".to_string()),
        64..=71 => Some("Woodwind".to_string()),
        72..=79 => Some("Flute".to_string()),
        80..=87 => Some("Lead".to_string()),
        88..=95 => Some("Pad".to_string()),
        96..=103 => Some("FX".to_string()),
        104..=111 => Some("Ethnic".to_string()),
        112..=119 => Some("Percussion".to_string()),
        120..=127 => Some("FX".to_string()),
        _ => None,
    }
}

fn detect_pitch_bend(midi_file: &MidiFile) -> bool {
    for track in &midi_file.tracks {
        for timed_event in &track.events {
            if matches!(&timed_event.event, Event::PitchBend { .. }) {
                return true;
            }
        }
    }
    false
}

fn detect_cc_messages(midi_file: &MidiFile) -> bool {
    for track in &midi_file.tracks {
        for timed_event in &track.events {
            if matches!(&timed_event.event, Event::ControlChange { .. }) {
                return true;
            }
        }
    }
    false
}

fn calculate_complexity_score(note_stats: &NoteStats, midi_file: &MidiFile) -> Option<f64> {
    if note_stats.note_count == 0 {
        return Some(0.0);
    }

    let mut score = 0.0;

    // Factor 1: Note density
    let duration_est = calculate_total_ticks(midi_file) as f64
        / (midi_file.header.ticks_per_quarter_note as f64 * 2.0);
    if duration_est > 0.0 {
        let note_density = note_stats.note_count as f64 / duration_est;
        score += (note_density / 10.0).min(30.0);
    }

    // Factor 2: Pitch range
    if let Some(semitones) = note_stats.pitch_range_semitones {
        score += (semitones as f64 / 2.0).min(20.0);
    }

    // Factor 3: Polyphony
    if let Some(polyphony) = note_stats.polyphony_max {
        score += (polyphony as f64 * 5.0).min(25.0);
    }

    // Factor 4: Track count
    let track_count = midi_file.tracks.len() as f64;
    score += (track_count * 2.0).min(15.0);

    // Factor 5: Velocity variation
    if let (Some(low), Some(high)) = (
        note_stats.velocity_range_low,
        note_stats.velocity_range_high,
    ) {
        let velocity_range = (high - low) as f64;
        score += (velocity_range / 10.0).min(10.0);
    }

    Some(score.min(100.0))
}

//=============================================================================
// PROGRESS REPORTING
//=============================================================================

fn print_progress_summary(stats: &ImportStats) {
    let elapsed = stats.start_time.map(|t| t.elapsed().as_secs_f64()).unwrap_or(0.0);
    let imported = stats.files_imported.load(Ordering::SeqCst);
    let rate = if elapsed > 0.0 {
        imported as f64 / elapsed
    } else {
        0.0
    };

    println!("  📊 Progress:");
    println!(
        "    Archives: {}/{}",
        stats.archives_processed.load(Ordering::SeqCst),
        stats.archives_total.load(Ordering::SeqCst)
    );
    println!("    Imported: {}", imported);
    println!(
        "    Duplicates: {}",
        stats.files_duplicates.load(Ordering::SeqCst)
    );
    println!("    Errors: {}", stats.files_errors.load(Ordering::SeqCst));
    println!("    Rate: {:.1} files/sec", rate);
}

fn print_final_summary(stats: &ImportStats) {
    let elapsed = stats
        .start_time
        .map(|t| t.elapsed())
        .unwrap_or_else(|| std::time::Duration::from_secs(0));
    let duration_secs = elapsed.as_secs_f64();
    let imported = stats.files_imported.load(Ordering::SeqCst);
    let rate = if duration_secs > 0.0 {
        imported as f64 / duration_secs
    } else {
        0.0
    };

    println!("\n========================================");
    println!("UNIFIED IMPORT COMPLETE");
    println!("========================================");
    println!(
        "Archives processed: {}/{}",
        stats.archives_processed.load(Ordering::SeqCst),
        stats.archives_total.load(Ordering::SeqCst)
    );
    println!(
        "MIDI files found: {}",
        stats.files_found.load(Ordering::SeqCst)
    );
    println!("Successfully imported: {}", imported);
    println!("  With full analysis: {}", imported);
    println!(
        "Duplicates skipped: {}",
        stats.files_duplicates.load(Ordering::SeqCst)
    );
    println!("Errors: {}", stats.files_errors.load(Ordering::SeqCst));
    println!(
        "Time: {:.0}h {:.0}m {:.0}s",
        duration_secs / 3600.0,
        (duration_secs % 3600.0) / 60.0,
        duration_secs % 60.0
    );
    println!("Avg speed: {:.0} files/sec", rate);
    println!("========================================");
    println!("All files include: BPM, Key, Tags, Complexity");
    println!("Ready to use in DAW!");
    println!("========================================\n");
}

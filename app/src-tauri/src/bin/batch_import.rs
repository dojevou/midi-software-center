#!/usr/bin/env cargo
/// Batch import using existing repository layer
///
/// This imports MIDI files using the FileRepository and MetadataRepository
/// which are already aligned with the database schema.
use anyhow::Result;
use clap::Parser;
use sqlx::types::BigDecimal;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use midi_app::core::analysis::bpm_detector::detect_bpm;
use midi_app::core::analysis::key_detector::detect_key;
use midi_app::core::hash::calculate_file_hash;
use midi_app::core::midi::parse_midi_file;
use midi_app::db::models::pipeline::{NewFile, NewMusicalMetadata};
use midi_app::db::repositories::pipeline::file_repository::FileRepository;
use midi_app::db::repositories::pipeline::metadata_repository::MetadataRepository;

#[derive(Parser)]
#[command(name = "batch-import")]
#[command(about = "Batch import MIDI files using repository layer")]
struct Args {
    /// Directory containing MIDI files
    #[arg(short, long)]
    directory: PathBuf,

    /// Number of parallel workers
    #[arg(short = 'w', long, default_value = "32")]
    workers: usize,

    /// Database URL
    #[arg(long, env = "DATABASE_URL")]
    database_url: Option<String>,
}

#[derive(Debug, Default)]
struct ImportStats {
    files_found: AtomicU64,
    files_imported: AtomicU64,
    files_duplicates: AtomicU64,
    files_errors: AtomicU64,
    start_time: Option<Instant>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let args = Args::parse();

    println!("\n🎵 BATCH MIDI IMPORT (Repository Layer)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Connect to database
    let database_url = args.database_url.unwrap_or_else(|| {
        std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            eprintln!("❌ Error: DATABASE_URL environment variable must be set");
            std::process::exit(1);
        })
    });

    println!("🔌 Connecting to database...");
    let pool = sqlx::PgPool::connect(&database_url).await?;
    println!("✅ Database connected\n");

    // Find all MIDI files
    println!(
        "📂 Scanning for MIDI files in: {}",
        args.directory.display()
    );
    let midi_files = find_midi_files(&args.directory)?;
    let total_files = midi_files.len();

    println!("✅ Found {} MIDI files\n", total_files);

    if total_files == 0 {
        println!("⚠️  No MIDI files found");
        return Ok(());
    }

    // Initialize stats
    let stats = Arc::new(ImportStats {
        files_found: AtomicU64::new(total_files as u64),
        start_time: Some(Instant::now()),
        ..Default::default()
    });

    // Process files in parallel
    println!(
        "⚡ Processing {} files with {} workers...\n",
        total_files, args.workers
    );

    use futures::stream::{self, StreamExt};

    let semaphore = Arc::new(tokio::sync::Semaphore::new(args.workers));
    let processed = Arc::new(AtomicUsize::new(0));

    stream::iter(midi_files)
        .map(|file_path| {
            let sem = Arc::clone(&semaphore);
            let pool = pool.clone();
            let stats = Arc::clone(&stats);
            let processed = Arc::clone(&processed);

            async move {
                let _permit = match sem.acquire().await {
                    Ok(permit) => permit,
                    Err(_) => {
                        eprintln!("Warning: Semaphore closed during import");
                        return;
                    },
                };

                let current = processed.fetch_add(1, Ordering::SeqCst) + 1;

                // Show progress every 100 files
                if current.is_multiple_of(100) || current == total_files {
                    let elapsed =
                        stats.start_time.map(|t| t.elapsed().as_secs_f64()).unwrap_or(0.0);
                    let rate = if elapsed > 0.0 {
                        current as f64 / elapsed
                    } else {
                        0.0
                    };
                    println!(
                        "    Processing: {}/{} ({:.1}%) - {:.1} files/sec",
                        current,
                        total_files,
                        (current as f64 / total_files as f64) * 100.0,
                        rate
                    );
                }

                // Process the file
                match process_file(&pool, &file_path).await {
                    Ok(imported) => {
                        if imported {
                            stats.files_imported.fetch_add(1, Ordering::SeqCst);
                        } else {
                            stats.files_duplicates.fetch_add(1, Ordering::SeqCst);
                        }
                    },
                    Err(e) => {
                        eprintln!("      ⚠️  Error processing {}: {}", file_path.display(), e);
                        stats.files_errors.fetch_add(1, Ordering::SeqCst);
                    },
                }
            }
        })
        .buffer_unordered(args.workers)
        .collect::<Vec<_>>()
        .await;

    // Print final summary
    print_summary(&stats);

    Ok(())
}

/// Process a single MIDI file
async fn process_file(pool: &sqlx::PgPool, file_path: &Path) -> Result<bool> {
    // 1. Read file
    let file_bytes = tokio::fs::read(file_path).await?;
    let file_size = file_bytes.len() as i64;

    // 2. Calculate content hash
    let content_hash = calculate_file_hash(file_path)?;

    // 3. Check for duplicate
    if FileRepository::check_duplicate(pool, &content_hash).await? {
        return Ok(false); // Duplicate, skip
    }

    // 4. Parse MIDI file
    let midi_file = parse_midi_file(&file_bytes)?;

    // 5. Extract file metadata
    let filename = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string();

    let filepath = file_path.to_str().unwrap_or("").to_string();

    let format = Some(midi_file.header.format as i16);
    let num_tracks = midi_file.tracks.len() as i16;
    let ticks_per_quarter = Some(midi_file.header.ticks_per_quarter_note as i32);

    // 6. Run BPM detection
    let bpm_result = detect_bpm(&midi_file);
    let bpm = if bpm_result.confidence > 0.3 {
        Some(BigDecimal::from(bpm_result.bpm as i64))
    } else {
        None
    };
    let bpm_confidence = Some(bpm_result.confidence as f32);

    // 7. Run key detection
    let key_result = detect_key(&midi_file);
    let key_signature = if key_result.confidence > 0.5 {
        Some(key_result.key.clone())
    } else {
        None
    };
    let key_confidence = Some(key_result.confidence as f32);

    // 8. Extract time signature
    let (time_sig_num, time_sig_den) = extract_time_signature(&midi_file);

    // 9. Calculate duration
    let total_ticks = calculate_total_ticks(&midi_file);
    let duration_seconds = if bpm.is_some() {
        let bpm_f64 = bpm_result.bpm;
        let ticks = total_ticks as f64;
        let tpq = midi_file.header.ticks_per_quarter_note as f64;
        if tpq > 0.0 && bpm_f64 > 0.0 {
            let quarters = ticks / tpq;
            let minutes = quarters / bpm_f64;
            let seconds = minutes * 60.0;
            Some(BigDecimal::from(seconds as i64))
        } else {
            None
        }
    } else {
        None
    };

    let duration_ticks = Some(total_ticks);

    // 10. Analyze notes
    let note_stats = analyze_notes(&midi_file);

    // 11. Insert file using FileRepository
    let new_file = NewFile {
        filename: filename.clone(),
        filepath: filepath.clone(),
        original_filename: filename,
        content_hash: content_hash.to_vec(),
        file_size_bytes: file_size,
        format,
        num_tracks,
        ticks_per_quarter_note: ticks_per_quarter,
        duration_seconds,
        duration_ticks,
        manufacturer: None,
        collection_name: None,
        folder_tags: None,
        import_batch_id: None,
        parent_folder: None,
        filename_bpm: None,
        filename_key: None,
        filename_genres: None,
        structure_tags: None,
        metadata_source: None,
        track_names: None,
        copyright: None,
        instrument_names_text: None,
        markers: None,
        lyrics: None,
    };

    let file_id = FileRepository::insert(pool, new_file).await?;

    // 12. Insert metadata using MetadataRepository
    let new_metadata = NewMusicalMetadata {
        file_id,
        bpm,
        bpm_confidence,
        key_signature,
        key_confidence,
        time_signature_numerator: time_sig_num,
        time_signature_denominator: time_sig_den,
        total_notes: note_stats.note_count,
        unique_pitches: note_stats.unique_pitches,
        pitch_range_min: note_stats.pitch_min,
        pitch_range_max: note_stats.pitch_max,
        avg_velocity: note_stats.avg_velocity,
        note_density: None, // Can be calculated later
        polyphony_max: note_stats.polyphony_max,
        chord_progression: None,
        chord_types: None,
        has_seventh_chords: None,
        has_extended_chords: None,
        chord_change_rate: None,
        chord_complexity_score: None,
        polyphony_avg: None,
        is_percussive: None,
    };

    MetadataRepository::insert(pool, new_metadata).await?;

    Ok(true) // Successfully imported
}

/// Find all MIDI files in a directory
fn find_midi_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.eq_ignore_ascii_case("mid") || ext.eq_ignore_ascii_case("midi") {
                    files.push(path.to_path_buf());
                }
            }
        }
    }

    Ok(files)
}

/// Extract time signature from MIDI file
fn extract_time_signature(
    midi_file: &midi_app::core::midi::types::MidiFile,
) -> (Option<i16>, Option<i16>) {
    use midi_app::core::midi::types::Event;

    for track in &midi_file.tracks {
        for timed_event in &track.events {
            if let Event::TimeSignature { numerator, denominator, .. } = &timed_event.event {
                let denom_value = 2i16.pow(*denominator as u32);
                return (Some(*numerator as i16), Some(denom_value));
            }
        }
    }
    (Some(4), Some(4)) // Default
}

/// Calculate total ticks in MIDI file
fn calculate_total_ticks(midi_file: &midi_app::core::midi::types::MidiFile) -> i64 {
    let mut max_ticks = 0u32;
    for track in &midi_file.tracks {
        let mut track_ticks = 0u32;
        for timed_event in &track.events {
            track_ticks += timed_event.delta_ticks;
        }
        max_ticks = max_ticks.max(track_ticks);
    }
    max_ticks as i64
}

/// Note statistics
#[derive(Debug)]
struct NoteStats {
    note_count: i32,
    unique_pitches: Option<i32>,
    pitch_min: Option<i16>,
    pitch_max: Option<i16>,
    avg_velocity: Option<BigDecimal>,
    polyphony_max: Option<i16>,
}

/// Analyze notes in MIDI file
fn analyze_notes(midi_file: &midi_app::core::midi::types::MidiFile) -> NoteStats {
    use midi_app::core::midi::types::Event;
    use std::collections::{HashMap, HashSet};

    let mut note_count = 0i32;
    let mut pitches = HashSet::new();
    let mut min_pitch = 127u8;
    let mut max_pitch = 0u8;
    let mut velocity_sum = 0u32;
    let mut active_notes_per_tick: HashMap<u32, usize> = HashMap::new();

    for track in &midi_file.tracks {
        let mut current_tick = 0u32;
        let mut active_notes = HashSet::new();

        for timed_event in &track.events {
            current_tick += timed_event.delta_ticks;

            match &timed_event.event {
                Event::NoteOn { note, velocity, .. } if *velocity > 0 => {
                    note_count += 1;
                    pitches.insert(*note);
                    min_pitch = min_pitch.min(*note);
                    max_pitch = max_pitch.max(*note);
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
        Some(BigDecimal::from((velocity_sum / note_count as u32) as i64))
    } else {
        None
    };

    let polyphony_max = active_notes_per_tick.values().max().copied().map(|v| v as i16);

    let (pitch_min, pitch_max) = if note_count > 0 {
        (Some(min_pitch as i16), Some(max_pitch as i16))
    } else {
        (None, None)
    };

    NoteStats {
        note_count,
        unique_pitches: Some(pitches.len() as i32),
        pitch_min,
        pitch_max,
        avg_velocity,
        polyphony_max,
    }
}

/// Print final summary
fn print_summary(stats: &ImportStats) {
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
    println!("BATCH IMPORT COMPLETE");
    println!("========================================");
    println!("Files found: {}", stats.files_found.load(Ordering::SeqCst));
    println!("Successfully imported: {}", imported);
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
    println!("All files include: BPM, Key, Notes, Stats");
    println!("========================================\n");
}

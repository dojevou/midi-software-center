use futures::stream::{self, StreamExt};
/// MIDI Analysis CLI Tool
///
/// Standalone binary to analyze all imported MIDI files
///
/// Usage:
///   cargo run --bin analyze
///
/// Environment Variables:
///   DATABASE_URL - PostgreSQL connection string
///                  Default: postgresql://midiuser:145278963@localhost:5433/midi_library
use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

// Import from the main library
use midi_library_shared::core::midi::parser::parse_midi_file;
use midi_library_shared::core::midi::types::{Event, MidiFile, TextType};
use midi_pipeline::core::analysis::bpm_detector::detect_bpm;
use midi_pipeline::core::analysis::key_detector::detect_key;

#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
struct FileRecord {
    id: i64,
    filepath: String,
    filename: String,
}

#[derive(Debug, Clone)]
struct AnalyzedFile {
    file_id: i64,
    tempo_bpm: Option<f64>,
    bpm_confidence: Option<f64>,
    has_tempo_variation: bool,
    key_signature: Option<String>,
    key_confidence: Option<f64>,
    scale_type: Option<String>,
    time_signature_num: Option<i16>,
    time_signature_den: Option<i16>,
    duration_seconds: Option<f64>,
    duration_ticks: Option<i32>,
    note_count: i32,
    pitch_range_low: Option<i16>,
    pitch_range_high: Option<i16>,
    pitch_range_semitones: Option<i16>,
    avg_velocity: Option<f64>,
    velocity_range_low: Option<i16>,
    velocity_range_high: Option<i16>,
    polyphony_max: Option<i16>,
    complexity_score: Option<f64>,
    instruments: Vec<String>,
    has_pitch_bend: bool,
    has_cc_messages: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🎵 MIDI Analysis Tool");
    println!("====================\n");

    // Get database URL from environment
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    println!("📡 Connecting to database...");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await?;
    println!("✅ Connected to database\n");

    // Get total count of unanalyzed files
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files WHERE analyzed_at IS NULL")
        .fetch_one(&pool)
        .await?;

    println!("🔍 Found {} unanalyzed files\n", total);

    if total == 0 {
        println!("✅ All files are already analyzed!");
        return Ok(());
    }

    let start_time = std::time::Instant::now();

    // Configuration
    let concurrency_limit = 32;
    let batch_size = 1000;

    println!("🚀 Starting analysis:");
    println!("  Concurrency: {} workers", concurrency_limit);
    println!("  Batch size: {} files\n", batch_size);

    // Thread-safe counters
    let analyzed = Arc::new(AtomicUsize::new(0));
    let skipped = Arc::new(AtomicUsize::new(0));
    let errors = Arc::new(Mutex::new(Vec::new()));
    let current_index = Arc::new(AtomicUsize::new(0));

    // Semaphore to limit concurrency
    let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency_limit));

    // Batch buffer for database inserts
    let analyzed_files = Arc::new(Mutex::new(Vec::new()));

    let total_usize = total as usize;

    // Process files in batches
    let mut offset = 0i64;

    loop {
        // Fetch batch of unanalyzed files
        let files: Vec<FileRecord> = sqlx::query_as(
            "SELECT id, filepath, filename
             FROM files
             WHERE analyzed_at IS NULL
             ORDER BY id
             LIMIT $1 OFFSET $2",
        )
        .bind(batch_size)
        .bind(offset)
        .fetch_all(&pool)
        .await?;

        if files.is_empty() {
            break;
        }

        let pool_clone = pool.clone();

        // Process batch in parallel
        stream::iter(files)
            .map(|file_record| {
                let sem = Arc::clone(&semaphore);
                let analyzed = Arc::clone(&analyzed);
                let skipped = Arc::clone(&skipped);
                let errors = Arc::clone(&errors);
                let current_index = Arc::clone(&current_index);
                let analyzed_files = Arc::clone(&analyzed_files);
                let pool = pool_clone.clone();

                async move {
                    let _permit = match sem.acquire().await {
                        Ok(permit) => permit,
                        Err(_) => {
                            eprintln!("Warning: Semaphore closed during analysis");
                            return;
                        },
                    };

                    let current = current_index.fetch_add(1, Ordering::SeqCst) + 1;

                    // Print progress every 100 files
                    if current.is_multiple_of(100) || current == total_usize {
                        let elapsed = start_time.elapsed().as_secs_f64();
                        let rate = if elapsed > 0.0 {
                            current as f64 / elapsed
                        } else {
                            0.0
                        };
                        let remaining = total_usize - current;
                        let eta_seconds = if rate > 0.0 {
                            remaining as f64 / rate
                        } else {
                            0.0
                        };

                        println!(
                            "Analyzing: {}/{} ({:.1}%) - {:.1} files/sec - ETA: {}",
                            current,
                            total_usize,
                            (current as f64 / total_usize as f64) * 100.0,
                            rate,
                            format_duration(eta_seconds)
                        );
                    }

                    // Analyze the file
                    match analyze_single_file(&file_record).await {
                        Ok(analyzed_data) => {
                            analyzed_files.lock().await.push(analyzed_data);
                            analyzed.fetch_add(1, Ordering::SeqCst);

                            // Flush batch if threshold reached
                            let mut files = analyzed_files.lock().await;
                            if files.len() >= 100 {
                                let batch: Vec<AnalyzedFile> = files.drain(..).collect();
                                drop(files);

                                if let Err(e) = batch_insert_analyzed_files(&batch, &pool).await {
                                    errors.lock().await.push(format!("Batch insert failed: {}", e));
                                }
                            }
                        },
                        Err(e) => {
                            skipped.fetch_add(1, Ordering::SeqCst);
                            // Only log first 10 errors to avoid spam
                            let mut err_list = errors.lock().await;
                            if err_list.len() < 10 {
                                err_list.push(format!("{}: {}", file_record.filepath, e));
                            }
                        },
                    }
                }
            })
            .buffer_unordered(concurrency_limit)
            .collect::<Vec<_>>()
            .await;

        offset += batch_size;
    }

    // Flush remaining batch
    let remaining_files = analyzed_files.lock().await;
    if !remaining_files.is_empty() {
        let batch: Vec<AnalyzedFile> = remaining_files.iter().cloned().collect();
        drop(remaining_files);

        batch_insert_analyzed_files(&batch, &pool).await?;
    }

    // Print final statistics
    let duration = start_time.elapsed().as_secs_f64();
    let analyzed_count = analyzed.load(Ordering::SeqCst);
    let skipped_count = skipped.load(Ordering::SeqCst);
    let rate = if duration > 0.0 {
        analyzed_count as f64 / duration
    } else {
        0.0
    };

    println!("\n✅ Analysis complete!");
    println!("==================");
    println!("  Total files:    {}", total_usize);
    println!("  Analyzed:       {}", analyzed_count);
    println!("  Skipped:        {}", skipped_count);
    println!("  Duration:       {}", format_duration(duration));
    println!("  Average rate:   {:.1} files/sec", rate);

    let error_list = errors.lock().await;
    if !error_list.is_empty() {
        println!("\n⚠️  Errors encountered:");
        for (i, error) in error_list.iter().enumerate().take(10) {
            println!("  {}. {}", i + 1, error);
        }
        if error_list.len() > 10 {
            println!("  ... and {} more errors", error_list.len() - 10);
        }
    }

    Ok(())
}

// Helper function to format duration in human-readable format
fn format_duration(seconds: f64) -> String {
    if seconds < 60.0 {
        format!("{:.0}s", seconds)
    } else if seconds < 3600.0 {
        let minutes = (seconds / 60.0).floor();
        let secs = seconds % 60.0;
        format!("{}m {:.0}s", minutes, secs)
    } else {
        let hours = (seconds / 3600.0).floor();
        let minutes = ((seconds % 3600.0) / 60.0).floor();
        format!("{}h {}m", hours, minutes)
    }
}

// Copy of analyze_single_file from commands/analyze.rs
async fn analyze_single_file(
    file_record: &FileRecord,
) -> Result<AnalyzedFile, Box<dyn std::error::Error + Send + Sync>> {
    let file_bytes = tokio::fs::read(&file_record.filepath).await?;
    let midi_file = parse_midi_file(&file_bytes)?;

    let bpm_result = detect_bpm(&midi_file);
    let tempo_bpm = if bpm_result.confidence > 0.3 {
        Some(bpm_result.bpm)
    } else {
        None
    };
    let bpm_confidence = Some(bpm_result.confidence);
    let has_tempo_variation = !bpm_result.metadata.is_constant;

    let key_result = detect_key(&midi_file);
    let key_signature = if key_result.confidence > 0.5 {
        Some(key_result.key.clone())
    } else {
        None
    };
    let key_confidence = Some(key_result.confidence);
    let scale_type = Some(key_result.scale_type.to_string());

    let (time_signature_num, time_signature_den) = extract_time_signature(&midi_file);
    let duration_ticks = calculate_total_ticks(&midi_file);
    let duration_seconds = calculate_duration_seconds(&midi_file, bpm_result.bpm);
    let note_stats = analyze_notes(&midi_file);
    let instruments = extract_instrument_names(&midi_file);
    let has_pitch_bend = detect_pitch_bend(&midi_file);
    let has_cc_messages = detect_cc_messages(&midi_file);
    let complexity_score = calculate_complexity_score(&note_stats, &midi_file);

    Ok(AnalyzedFile {
        file_id: file_record.id,
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
    })
}

// Copy of batch_insert_analyzed_files from commands/analyze.rs
async fn batch_insert_analyzed_files(
    files: &[AnalyzedFile],
    pool: &sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if files.is_empty() {
        return Ok(());
    }

    let mut tx = pool.begin().await?;

    for file in files {
        sqlx::query(
            r#"
            INSERT INTO musical_metadata (
                file_id, tempo_bpm, bpm_confidence, has_tempo_variation,
                key_signature, key_confidence, scale_type,
                time_signature_num, time_signature_den,
                duration_seconds, duration_ticks, note_count,
                pitch_range_low, pitch_range_high, pitch_range_semitones,
                avg_velocity, velocity_range_low, velocity_range_high,
                polyphony_max, complexity_score, instruments,
                has_pitch_bend, has_cc_messages
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23)
            ON CONFLICT (file_id) DO UPDATE SET
                tempo_bpm = EXCLUDED.tempo_bpm,
                bpm_confidence = EXCLUDED.bpm_confidence,
                has_tempo_variation = EXCLUDED.has_tempo_variation,
                key_signature = EXCLUDED.key_signature,
                key_confidence = EXCLUDED.key_confidence,
                scale_type = EXCLUDED.scale_type,
                time_signature_num = EXCLUDED.time_signature_num,
                time_signature_den = EXCLUDED.time_signature_den,
                duration_seconds = EXCLUDED.duration_seconds,
                duration_ticks = EXCLUDED.duration_ticks,
                note_count = EXCLUDED.note_count,
                pitch_range_low = EXCLUDED.pitch_range_low,
                pitch_range_high = EXCLUDED.pitch_range_high,
                pitch_range_semitones = EXCLUDED.pitch_range_semitones,
                avg_velocity = EXCLUDED.avg_velocity,
                velocity_range_low = EXCLUDED.velocity_range_low,
                velocity_range_high = EXCLUDED.velocity_range_high,
                polyphony_max = EXCLUDED.polyphony_max,
                complexity_score = EXCLUDED.complexity_score,
                instruments = EXCLUDED.instruments,
                has_pitch_bend = EXCLUDED.has_pitch_bend,
                has_cc_messages = EXCLUDED.has_cc_messages
            "#
        )
        .bind(file.file_id)
        .bind(file.tempo_bpm)
        .bind(file.bpm_confidence)
        .bind(file.has_tempo_variation)
        .bind(&file.key_signature)
        .bind(file.key_confidence)
        .bind(&file.scale_type)
        .bind(file.time_signature_num)
        .bind(file.time_signature_den)
        .bind(file.duration_seconds)
        .bind(file.duration_ticks)
        .bind(file.note_count)
        .bind(file.pitch_range_low)
        .bind(file.pitch_range_high)
        .bind(file.pitch_range_semitones)
        .bind(file.avg_velocity)
        .bind(file.velocity_range_low)
        .bind(file.velocity_range_high)
        .bind(file.polyphony_max)
        .bind(file.complexity_score)
        .bind(&file.instruments)
        .bind(file.has_pitch_bend)
        .bind(file.has_cc_messages)
        .execute(&mut *tx)
        .await?;

        sqlx::query("UPDATE files SET analyzed_at = NOW() WHERE id = $1")
            .bind(file.file_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(())
}

// Helper analysis functions (copied from commands/analyze.rs)

#[derive(Debug, Clone)]
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
        Some(minutes * 60.0)
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
                        && !instruments.contains(text) {
                            instruments.push(text.clone());
                        }
                },
                Event::ProgramChange { program, .. } => {
                    if let Some(name) = program_to_instrument_name(*program) {
                        if !instruments.contains(&name) {
                            instruments.push(name);
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

    let duration_est = calculate_total_ticks(midi_file) as f64
        / (midi_file.header.ticks_per_quarter_note as f64 * 2.0);
    if duration_est > 0.0 {
        let note_density = note_stats.note_count as f64 / duration_est;
        score += (note_density / 10.0).min(30.0);
    }

    if let Some(semitones) = note_stats.pitch_range_semitones {
        score += (semitones as f64 / 2.0).min(20.0);
    }

    if let Some(polyphony) = note_stats.polyphony_max {
        score += (polyphony as f64 * 5.0).min(25.0);
    }

    let track_count = midi_file.tracks.len() as f64;
    score += (track_count * 2.0).min(15.0);

    if let (Some(low), Some(high)) = (
        note_stats.velocity_range_low,
        note_stats.velocity_range_high,
    ) {
        let velocity_range = (high - low) as f64;
        score += (velocity_range / 10.0).min(10.0);
    }

    Some(score.min(100.0))
}

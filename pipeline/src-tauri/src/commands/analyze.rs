//! Musical Analysis Commands - HIGH-PERFORMANCE PARALLEL IMPLEMENTATION
//!
//! Architecture: Grown-up Script
//! Purpose: Analyze all imported MIDI files using existing analysis modules
//!
//! This module processes 1.1M+ imported files by:
//! - Reading unanalyzed files from database in batches
//! - Parallel processing with buffer_unordered (32 workers)
//! - Running BPM detection, key detection, and auto-tagging
//! - Batch database inserts for musical_metadata
//! - Real-time progress updates
//!
//! Performance Target: 400-500 files/sec (complete 1.1M files in ~40-60 minutes)

use crate::AppState;
use midi_library_shared::core::midi::parser::parse_midi_file;
use midi_library_shared::core::midi::types::{Event, MidiFile, TextType};
use crate::core::analysis::bpm_detector::detect_bpm;
use crate::core::analysis::key_detector::detect_key;
// Unused: use crate::core::analysis::auto_tagger::{AutoTagger, Tag};

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Emitter, State, Window};
use futures::stream::{self, StreamExt};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Mutex;

//=============================================================================
// TYPE DEFINITIONS
//=============================================================================

/// Progress event for real-time UI updates
#[derive(Debug, Clone, Serialize)]
pub struct AnalysisProgress {
    pub current: usize,
    pub total: usize,
    pub current_file: String,
    pub rate: f64, // files per second
    pub eta_seconds: f64,
}

/// Summary of analysis operation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    pub total_files: usize,
    pub analyzed: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
    pub duration_secs: f64,
    pub rate: f64, // files per second
}

/// File record from database
#[derive(Debug, Clone, sqlx::FromRow)]
struct FileRecord {
    id: i64,
    filepath: String,
    filename: String,
}

/// Analyzed file data ready for database insertion
#[derive(Debug, Clone)]
struct AnalyzedFile {
    file_id: i64,

    // Tempo
    tempo_bpm: Option<f64>,
    bpm_confidence: Option<f64>,
    has_tempo_variation: bool,

    // Key
    key_signature: Option<String>,
    key_confidence: Option<f64>,
    scale_type: Option<String>,

    // Time signature
    time_signature_num: Option<i16>,
    time_signature_den: Option<i16>,

    // Duration
    duration_seconds: Option<f64>,
    duration_ticks: Option<i32>,

    // Note analysis
    note_count: i32,
    pitch_range_low: Option<i16>,
    pitch_range_high: Option<i16>,
    pitch_range_semitones: Option<i16>,

    // Velocity
    avg_velocity: Option<f64>,
    velocity_range_low: Option<i16>,
    velocity_range_high: Option<i16>,

    // Polyphony
    polyphony_max: Option<i16>,

    // Complexity
    complexity_score: Option<f64>,

    // Additional properties
    instruments: Vec<String>,
    has_pitch_bend: bool,
    has_cc_messages: bool,
}

//=============================================================================
// TAURI COMMANDS
//=============================================================================

/// Analyze all unanalyzed MIDI files (HIGH-PERFORMANCE PARALLEL VERSION)
///
/// This command:
/// 1. Reads unanalyzed files from database in batches
/// 2. Processes them in parallel with 32 workers
/// 3. Runs BPM detection, key detection, note analysis
/// 4. Batch inserts results into musical_metadata
/// 5. Updates files.analyzed_at timestamp
/// 6. Shows real-time progress
#[tauri::command]
pub async fn start_analysis(
    state: State<'_, AppState>,
    window: Window,
) -> Result<AnalysisSummary, String> {
    let start_time = std::time::Instant::now();
    let pool: sqlx::PgPool = state.database.pool().await;

    // Get total count of unanalyzed files
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM files WHERE analyzed_at IS NULL"
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("Failed to count unanalyzed files: {}", e))?;

    println!("🔍 Found {} unanalyzed files", total);

    if total == 0 {
        return Ok(AnalysisSummary {
            total_files: 0,
            analyzed: 0,
            skipped: 0,
            errors: vec![],
            duration_secs: 0.0,
            rate: 0.0,
        });
    }

    // Parallel processing configuration
    let concurrency_limit = 32; // Process 32 files concurrently
    let batch_size = 1000; // Fetch files in batches of 1000

    println!("🚀 Starting analysis:");
    println!("  Concurrency: {} workers", concurrency_limit);
    println!("  Batch size: {} files", batch_size);

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
             LIMIT $1 OFFSET $2"
        )
        .bind(batch_size)
        .bind(offset)
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("Failed to fetch files: {}", e))?;

        if files.is_empty() {
            break;
        }

        let batch_len = files.len();
        println!("📦 Processing batch: {} files (offset: {})", batch_len, offset);

        // Process batch in parallel
        stream::iter(files)
            .map(|file_record| {
                // Clone Arc pointers for each concurrent task
                let sem = Arc::clone(&semaphore);
                let analyzed = Arc::clone(&analyzed);
                let skipped = Arc::clone(&skipped);
                let errors = Arc::clone(&errors);
                let current_index = Arc::clone(&current_index);
                let analyzed_files = Arc::clone(&analyzed_files);
                let window = window.clone();

                    let pool = pool.clone();
                async move {
                    // Acquire semaphore permit (blocks if at limit)
                    let _permit = sem.acquire().await.unwrap();

                    let current = current_index.fetch_add(1, Ordering::SeqCst) + 1;

                    // Emit progress every 10 files
                    if current % 10 == 0 || current == total_usize {
                        let elapsed = start_time.elapsed().as_secs_f64();
                        let rate = if elapsed > 0.0 { current as f64 / elapsed } else { 0.0 };
                        let remaining = total_usize - current;
                        let eta_seconds = if rate > 0.0 { remaining as f64 / rate } else { 0.0 };

                        let _ = window.emit("analysis-progress", AnalysisProgress {
                            current,
                            total: total_usize,
                            current_file: file_record.filename.clone(),
                            rate,
                            eta_seconds,
                        });

                        // Print progress every 100 files
                        if current % 100 == 0 {
                            println!(
                                "Analyzing: {}/{} ({:.1}%) - {:.1} files/sec - ETA: {:.0}s",
                                current,
                                total_usize,
                                (current as f64 / total_usize as f64) * 100.0,
                                rate,
                                eta_seconds
                            );
                        }
                    }

                    // Analyze the file
                    match analyze_single_file(&file_record).await {
                        Ok(analyzed_data) => {
                            // Add to batch for insertion
                            analyzed_files.lock().await.push(analyzed_data);
                            analyzed.fetch_add(1, Ordering::SeqCst);

                            // Flush batch if it reaches threshold (100 files)
                            let mut files = analyzed_files.lock().await;
                            if files.len() >= 100 {
                                let batch: Vec<AnalyzedFile> = files.drain(..).collect();
                                drop(files); // Release lock

                                if let Err(e) = batch_insert_analyzed_files(&batch, &pool).await {
                                    errors.lock().await.push(format!("Batch insert failed: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            let error_msg = format!("{}: {}", file_record.filepath, e);
                            errors.lock().await.push(error_msg);
                            skipped.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                }
            })
            .buffer_unordered(concurrency_limit) // ← THE MAGIC: Process N files concurrently!
            .collect::<Vec<_>>()
            .await;

        offset += batch_size;
    }

    // Flush remaining batch
    let remaining_files = analyzed_files.lock().await;
    if !remaining_files.is_empty() {
        let batch: Vec<AnalyzedFile> = remaining_files.iter().cloned().collect();
        drop(remaining_files);

        if let Err(e) = batch_insert_analyzed_files(&batch, &pool).await {
            errors.lock().await.push(format!("Final batch insert failed: {}", e));
        }
    }

    // Calculate final statistics
    let duration = start_time.elapsed().as_secs_f64();
    let analyzed_count = analyzed.load(Ordering::SeqCst);
    let rate = if duration > 0.0 { analyzed_count as f64 / duration } else { 0.0 };

    println!("\n✅ Analysis complete!");
    println!("  Total files: {}", total_usize);
    println!("  Analyzed: {}", analyzed_count);
    println!("  Skipped: {}", skipped.load(Ordering::SeqCst));
    println!("  Duration: {:.1}s", duration);
    println!("  Rate: {:.1} files/sec", rate);

    // Extract errors before creating summary
    let error_list = errors.lock().await.clone();

    Ok(AnalysisSummary {
        total_files: total_usize,
        analyzed: analyzed_count,
        skipped: skipped.load(Ordering::SeqCst),
        errors: error_list,
        duration_secs: duration,
        rate,
    })
}

//=============================================================================
// CORE ANALYSIS LOGIC
//=============================================================================

/// Analyze a single MIDI file using all analysis modules
async fn analyze_single_file(
    file_record: &FileRecord,
) -> Result<AnalyzedFile, Box<dyn std::error::Error + Send + Sync>> {
    // 1. Read MIDI file from filesystem
    let file_bytes = tokio::fs::read(&file_record.filepath).await?;

    // 2. Parse MIDI file (Trusty Module)
    let midi_file = parse_midi_file(&file_bytes)?;

    // 3. BPM Detection (Trusty Module)
    let bpm_result = detect_bpm(&midi_file);
    let tempo_bpm = if bpm_result.confidence > 0.3 {
        Some(bpm_result.bpm)
    } else {
        None
    };
    let bpm_confidence = Some(bpm_result.confidence);
    let has_tempo_variation = !bpm_result.metadata.is_constant;

    // 4. Key Detection (Trusty Module)
    let key_result = detect_key(&midi_file);
    let key_signature = if key_result.confidence > 0.5 {
        Some(key_result.key.clone())
    } else {
        None
    };
    let key_confidence = Some(key_result.confidence);
    let scale_type = Some(key_result.scale_type.to_string());

    // 5. Extract time signature from MIDI events
    let (time_signature_num, time_signature_den) = extract_time_signature(&midi_file);

    // 6. Calculate duration
    let duration_ticks = calculate_total_ticks(&midi_file);
    let duration_seconds = calculate_duration_seconds(&midi_file, bpm_result.bpm);

    // 7. Note analysis
    let note_stats = analyze_notes(&midi_file);

    // 8. Extract instruments
    let instruments = extract_instrument_names(&midi_file);

    // 9. Detect MIDI features
    let has_pitch_bend = detect_pitch_bend(&midi_file);
    let has_cc_messages = detect_cc_messages(&midi_file);

    // 10. Calculate complexity score (simple heuristic)
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

/// Batch insert analyzed files into musical_metadata and update files.analyzed_at
async fn batch_insert_analyzed_files(
    files: &[AnalyzedFile],
    pool: &sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if files.is_empty() {
        return Ok(());
    }

    let mut tx = pool.begin().await?;

    for file in files {
        // Insert or update musical_metadata
        sqlx::query(
            r#"
            INSERT INTO musical_metadata (
                file_id,
                tempo_bpm,
                bpm_confidence,
                has_tempo_variation,
                key_signature,
                key_confidence,
                scale_type,
                time_signature_num,
                time_signature_den,
                duration_seconds,
                duration_ticks,
                note_count,
                pitch_range_low,
                pitch_range_high,
                pitch_range_semitones,
                avg_velocity,
                velocity_range_low,
                velocity_range_high,
                polyphony_max,
                complexity_score,
                instruments,
                has_pitch_bend,
                has_cc_messages
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

        // Update files.analyzed_at timestamp
        sqlx::query(
            "UPDATE files SET analyzed_at = NOW() WHERE id = $1"
        )
        .bind(file.file_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

//=============================================================================
// HELPER FUNCTIONS - MIDI ANALYSIS
//=============================================================================

/// Note statistics
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

/// Analyze notes in MIDI file
fn analyze_notes(midi_file: &MidiFile) -> NoteStats {
    let mut note_count = 0;
    let mut min_pitch = 127u8;
    let mut max_pitch = 0u8;
    let mut min_velocity = 127u8;
    let mut max_velocity = 0u8;
    let mut velocity_sum = 0u32;
    let mut active_notes_per_tick: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();

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
                }
                Event::NoteOff { note, .. } | Event::NoteOn { note, velocity: 0, .. } => {
                    active_notes.remove(note);
                }
                _ => {}
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
        (Some(min_pitch as i16), Some(max_pitch as i16), Some(semitones))
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

/// Extract time signature from MIDI file
fn extract_time_signature(midi_file: &MidiFile) -> (Option<i16>, Option<i16>) {
    for track in &midi_file.tracks {
        for timed_event in &track.events {
            if let Event::TimeSignature { numerator, denominator, .. } = &timed_event.event {
                // MIDI stores denominator as power of 2 (2 = quarter note, 3 = eighth note, etc.)
                let denom_value = 2i16.pow(*denominator as u32);
                return (Some(*numerator as i16), Some(denom_value));
            }
        }
    }

    // Default to 4/4 if not found
    (Some(4), Some(4))
}

/// Calculate total number of ticks in MIDI file
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

/// Calculate duration in seconds
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

/// Extract instrument names from MIDI file
fn extract_instrument_names(midi_file: &MidiFile) -> Vec<String> {
    let mut instruments = Vec::new();

    for track in &midi_file.tracks {
        for timed_event in &track.events {
            match &timed_event.event {
                Event::Text { text_type, text } => {
                    if matches!(text_type, TextType::InstrumentName | TextType::TrackName) {
                        if !instruments.contains(text) {
                            instruments.push(text.clone());
                        }
                    }
                }
                Event::ProgramChange { program, .. } => {
                    if let Some(instrument_name) = program_to_instrument_name(*program) {
                        if !instruments.contains(&instrument_name) {
                            instruments.push(instrument_name);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    instruments
}

/// Map MIDI General MIDI program number to instrument name
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

/// Detect if MIDI file contains pitch bend events
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

/// Detect if MIDI file contains control change messages
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

/// Calculate complexity score based on various factors
fn calculate_complexity_score(note_stats: &NoteStats, midi_file: &MidiFile) -> Option<f64> {
    if note_stats.note_count == 0 {
        return Some(0.0);
    }

    let mut score = 0.0;

    // Factor 1: Note density (notes per second)
    // Assume average 120 BPM for rough estimate
    let duration_est = calculate_total_ticks(midi_file) as f64 / (midi_file.header.ticks_per_quarter_note as f64 * 2.0);
    if duration_est > 0.0 {
        let note_density = note_stats.note_count as f64 / duration_est;
        score += (note_density / 10.0).min(30.0); // Max 30 points
    }

    // Factor 2: Pitch range (wider range = more complex)
    if let Some(semitones) = note_stats.pitch_range_semitones {
        score += (semitones as f64 / 2.0).min(20.0); // Max 20 points
    }

    // Factor 3: Polyphony (more simultaneous notes = more complex)
    if let Some(polyphony) = note_stats.polyphony_max {
        score += (polyphony as f64 * 5.0).min(25.0); // Max 25 points
    }

    // Factor 4: Track count
    let track_count = midi_file.tracks.len() as f64;
    score += (track_count * 2.0).min(15.0); // Max 15 points

    // Factor 5: Velocity variation
    if let (Some(low), Some(high)) = (note_stats.velocity_range_low, note_stats.velocity_range_high) {
        let velocity_range = (high - low) as f64;
        score += (velocity_range / 10.0).min(10.0); // Max 10 points
    }

    // Normalize to 0-100 scale
    Some(score.min(100.0))
}

//=============================================================================
// TESTS
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_to_instrument_name() {
        assert_eq!(program_to_instrument_name(0), Some("Piano".to_string()));
        assert_eq!(program_to_instrument_name(32), Some("Bass".to_string()));
        assert_eq!(program_to_instrument_name(80), Some("Lead".to_string()));
    }

    #[test]
    fn test_complexity_score_empty() {
        let note_stats = NoteStats {
            note_count: 0,
            pitch_range_low: None,
            pitch_range_high: None,
            pitch_range_semitones: None,
            avg_velocity: None,
            velocity_range_low: None,
            velocity_range_high: None,
            polyphony_max: None,
        };

        let midi_file = MidiFile {
            header: midi_library_shared::core::midi::types::Header {
                format: 0,
                num_tracks: 1,
                ticks_per_quarter_note: 480,
            },
            tracks: vec![],
        };

        let score = calculate_complexity_score(&note_stats, &midi_file);
        assert_eq!(score, Some(0.0));
    }
}

//! Musical Analysis Commands - Parallel MIDI file analysis pipeline
//!
//! Processes imported files with BPM detection, key detection, chord analysis,
//! and note statistics. Uses parallel processing with batch database inserts.

mod articulation;
mod complexity;
mod controllers;
mod db;
mod gm_instruments;
mod meta_events;
mod midi_helpers;
mod note_analysis;
mod structure;
mod track_analysis;
mod types;

pub use db::batch_insert_analyzed_files;
pub use gm_instruments::get_instrument_info;
pub use types::{AnalysisProgress, AnalysisSummary, AnalyzedFile, FileRecord, TrackInstrument};

use crate::core::analysis::bpm_detector::detect_bpm;
use crate::core::analysis::chord_analyzer::analyze_chords;
use crate::core::analysis::drum_analyzer::analyze_drum_midi;
use crate::core::analysis::key_detector::detect_key;
use crate::AppState;
use midi_library_shared::core::midi::parser::parse_midi_file;

use articulation::analyze_articulation;
use complexity::calculate_complexity_score;
use controllers::analyze_controllers;
use meta_events::{
    calculate_duration_seconds, calculate_total_ticks, extract_key_changes, extract_tempo_changes,
    extract_time_signature, extract_time_signature_changes,
};
use midi_helpers::{detect_cc_messages, detect_pitch_bend, extract_instrument_names};
use note_analysis::analyze_notes;
use structure::analyze_structure;
use track_analysis::analyze_tracks;

use futures::stream::{self, StreamExt};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tauri::{Emitter, State, Window};
use tokio::sync::Mutex;

// Analysis configuration
const CONCURRENCY_LIMIT: usize = 32;
const BATCH_SIZE: i64 = 1000;
const BPM_CONFIDENCE_THRESHOLD: f64 = 0.3;
const KEY_CONFIDENCE_THRESHOLD: f64 = 0.5;

/// Analyze all unanalyzed MIDI files with parallel processing.
#[tauri::command]
pub async fn start_analysis(
    state: State<'_, AppState>,
    window: Window,
) -> Result<AnalysisSummary, String> {
    let start_time = std::time::Instant::now();
    let pool: sqlx::PgPool = state.database.pool().await;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files WHERE analyzed_at IS NULL")
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

    println!("🚀 Starting analysis:");
    println!("  Concurrency: {} workers", CONCURRENCY_LIMIT);
    println!("  Batch size: {} files", BATCH_SIZE);

    let analyzed = Arc::new(AtomicUsize::new(0));
    let skipped = Arc::new(AtomicUsize::new(0));
    let errors = Arc::new(Mutex::new(Vec::new()));
    let current_index = Arc::new(AtomicUsize::new(0));

    let semaphore = Arc::new(tokio::sync::Semaphore::new(CONCURRENCY_LIMIT));
    let analyzed_files = Arc::new(Mutex::new(Vec::new()));

    let total_usize = total as usize;
    let mut offset = 0i64;

    loop {
        let files: Vec<FileRecord> = sqlx::query_as(
            "SELECT id, filepath, filename
             FROM files
             WHERE analyzed_at IS NULL
             ORDER BY id
             LIMIT $1 OFFSET $2",
        )
        .bind(BATCH_SIZE)
        .bind(offset)
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("Failed to fetch files: {}", e))?;

        if files.is_empty() {
            break;
        }

        let batch_len = files.len();
        println!("📦 Processing batch: {} files (offset: {})", batch_len, offset);

        stream::iter(files)
            .map(|file_record| {
                let sem = Arc::clone(&semaphore);
                let analyzed = Arc::clone(&analyzed);
                let skipped = Arc::clone(&skipped);
                let errors = Arc::clone(&errors);
                let current_index = Arc::clone(&current_index);
                let analyzed_files = Arc::clone(&analyzed_files);
                let window = window.clone();
                let pool = pool.clone();

                async move {
                    let _permit = match sem.acquire().await {
                        Ok(permit) => permit,
                        Err(e) => {
                            let error_msg =
                                format!("FATAL: Semaphore unavailable during analysis: {}", e);
                            eprintln!("ERROR: {}", error_msg);
                            errors.lock().await.push(error_msg);
                            skipped.fetch_add(1, Ordering::SeqCst);
                            return;
                        }
                    };

                    let current = current_index.fetch_add(1, Ordering::SeqCst) + 1;

                    if current.is_multiple_of(10) || current == total_usize {
                        let elapsed = start_time.elapsed().as_secs_f64();
                        let rate = if elapsed > 0.0 { current as f64 / elapsed } else { 0.0 };
                        let remaining = total_usize - current;
                        let eta_seconds = if rate > 0.0 { remaining as f64 / rate } else { 0.0 };

                        if let Err(e) = window.emit(
                            "analysis-progress",
                            AnalysisProgress {
                                current,
                                total: total_usize,
                                current_file: file_record.filename.clone(),
                                rate,
                                eta_seconds,
                            },
                        ) {
                            eprintln!(
                                "WARNING: Failed to emit analysis progress (file {}): {}",
                                file_record.filename, e
                            );
                        }

                        if current.is_multiple_of(100) {
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

                    match analyze_single_file(&file_record).await {
                        Ok(analyzed_data) => {
                            analyzed_files.lock().await.push(analyzed_data);
                            analyzed.fetch_add(1, Ordering::SeqCst);

                            let mut files = analyzed_files.lock().await;
                            if files.len() >= 100 {
                                let batch: Vec<AnalyzedFile> = files.drain(..).collect();
                                drop(files);

                                if let Err(e) = batch_insert_analyzed_files(&batch, &pool).await {
                                    let error_msg = format!("Batch insert failed: {}", e);
                                    eprintln!("ERROR: {}", error_msg);
                                    errors.lock().await.push(error_msg);
                                    skipped.fetch_add(batch.len(), Ordering::SeqCst);
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
            .buffer_unordered(CONCURRENCY_LIMIT)
            .collect::<Vec<_>>()
            .await;

        offset += BATCH_SIZE;
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

    let duration = start_time.elapsed().as_secs_f64();
    let analyzed_count = analyzed.load(Ordering::SeqCst);
    let rate = if duration > 0.0 { analyzed_count as f64 / duration } else { 0.0 };

    println!("\n✅ Analysis complete!");
    println!("  Total files: {}", total_usize);
    println!("  Analyzed: {}", analyzed_count);
    println!("  Skipped: {}", skipped.load(Ordering::SeqCst));
    println!("  Duration: {:.1}s", duration);
    println!("  Rate: {:.1} files/sec", rate);

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

/// Analyze a single MIDI file using all analysis modules.
pub async fn analyze_single_file(
    file_record: &FileRecord,
) -> Result<AnalyzedFile, Box<dyn std::error::Error + Send + Sync>> {
    let file_bytes = tokio::fs::read(&file_record.filepath).await?;
    let midi_file = parse_midi_file(&file_bytes)?;

    // BPM Detection
    let bpm_result = detect_bpm(&midi_file);
    let tempo_bpm = (bpm_result.confidence > BPM_CONFIDENCE_THRESHOLD).then_some(bpm_result.bpm);
    let bpm_confidence = Some(bpm_result.confidence);
    let has_tempo_variation = !bpm_result.metadata.is_constant;

    // Key Detection
    let key_result = detect_key(&midi_file);
    let key_signature =
        (key_result.confidence > KEY_CONFIDENCE_THRESHOLD).then(|| key_result.key.clone());
    let key_confidence = Some(key_result.confidence);
    let scale_type = Some(key_result.scale_type.to_string());

    // Time signature and duration
    let (time_signature_num, time_signature_den) = extract_time_signature(&midi_file);
    let duration_ticks = calculate_total_ticks(&midi_file);
    let duration_seconds = calculate_duration_seconds(&midi_file, bpm_result.bpm);

    // Note and track analysis
    let note_stats = analyze_notes(&midi_file);
    let track_instruments = analyze_tracks(&midi_file);
    let instruments = extract_instrument_names(&midi_file);

    // MIDI feature detection
    let has_pitch_bend = detect_pitch_bend(&midi_file);
    let has_cc_messages = detect_cc_messages(&midi_file);

    // Chord analysis
    let ticks_per_quarter = midi_file.header.ticks_per_quarter_note as u32;
    let chord_analysis = analyze_chords(&midi_file, ticks_per_quarter);
    let has_chords = !chord_analysis.progression.is_empty();
    let chord_progression = has_chords.then_some(chord_analysis.progression);
    let chord_types = (!chord_analysis.types.is_empty()).then_some(chord_analysis.types);

    // Drum analysis (if percussion file)
    let _drum_analysis = if note_stats.is_percussive {
        Some(analyze_drum_midi(&midi_file))
    } else {
        None
    };

    // Melody detection
    let has_melody = note_stats.is_monophonic
        || (note_stats.polyphony_avg.is_some_and(|p| p < 2.0) && note_stats.note_count > 10);
    let melodic_range = if has_melody { note_stats.pitch_range_semitones } else { None };

    // Extract variation timelines
    let tempo_changes = extract_tempo_changes(&midi_file);
    let key_changes = extract_key_changes(&midi_file);
    let time_signature_changes = extract_time_signature_changes(&midi_file);

    // Controller analysis
    let controller_data = analyze_controllers(&midi_file);

    // Articulation/Performance analysis
    let tempo_us_per_qn = (60_000_000.0 / bpm_result.bpm) as u32;
    let articulation_data = analyze_articulation(&midi_file, tempo_us_per_qn);

    // Structure/Form analysis
    let structure_data = analyze_structure(&midi_file);

    // Calculate complexity score
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
        unique_pitches: note_stats.unique_pitches,
        pitch_range_low: note_stats.pitch_range_low,
        pitch_range_high: note_stats.pitch_range_high,
        pitch_range_semitones: note_stats.pitch_range_semitones,
        avg_velocity: note_stats.avg_velocity,
        velocity_range_low: note_stats.velocity_range_low,
        velocity_range_high: note_stats.velocity_range_high,
        note_density: note_stats.note_density,
        polyphony_max: note_stats.polyphony_max,
        polyphony_avg: note_stats.polyphony_avg,
        is_monophonic: note_stats.is_monophonic,
        is_polyphonic: note_stats.is_polyphonic,
        is_percussive: note_stats.is_percussive,
        has_chords,
        chord_progression,
        chord_types,
        has_seventh_chords: chord_analysis.has_sevenths,
        has_extended_chords: chord_analysis.has_extended,
        chord_change_rate: chord_analysis.change_rate,
        chord_complexity_score: Some(chord_analysis.complexity_score),
        has_melody,
        melodic_range,
        tempo_changes,
        key_changes,
        time_signature_changes,
        controller_data,
        articulation_data,
        structure_data,
        complexity_score,
        instruments,
        track_instruments,
        has_pitch_bend,
        has_cc_messages,
    })
}

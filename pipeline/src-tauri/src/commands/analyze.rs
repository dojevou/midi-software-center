//! Musical Analysis Commands - Parallel MIDI file analysis pipeline
//!
//! Processes imported files with BPM detection, key detection, chord analysis,
//! and note statistics. Uses parallel processing with batch database inserts.

use crate::core::analysis::bpm_detector::detect_bpm;
use crate::core::analysis::chord_analyzer::analyze_chords;
use crate::core::analysis::drum_analyzer::analyze_drum_midi;
use crate::core::analysis::key_detector::detect_key;
use crate::AppState;
use midi_library_shared::core::midi::parser::parse_midi_file;
use midi_library_shared::core::midi::types::{Event, MidiFile, TextType};

use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tauri::{Emitter, State, Window};
use tokio::sync::Mutex;

extern crate serde_json;

// Analysis configuration
const CONCURRENCY_LIMIT: usize = 32;
const BATCH_SIZE: i64 = 1000;
const BPM_CONFIDENCE_THRESHOLD: f64 = 0.3;
const KEY_CONFIDENCE_THRESHOLD: f64 = 0.5;

/// Progress event for real-time UI updates
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct AnalysisProgress {
    pub current: usize,
    pub total: usize,
    pub current_file: String,
    pub rate: f64, // files per second
    pub eta_seconds: f64,
}

/// Summary of analysis operation results
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct FileRecord {
    pub id: i64,
    pub filepath: String,
    pub filename: String,
}

/// Analyzed file data ready for database insertion
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AnalyzedFile {
    pub file_id: i64,

    // Tempo
    pub tempo_bpm: Option<f64>,
    pub bpm_confidence: Option<f64>,
    pub has_tempo_variation: bool,

    // Key
    pub key_signature: Option<String>,
    pub key_confidence: Option<f64>,
    pub scale_type: Option<String>,

    // Time signature
    pub time_signature_num: Option<i16>,
    pub time_signature_den: Option<i16>,

    // Duration
    pub duration_seconds: Option<f64>,
    pub duration_ticks: Option<i32>,

    // Note analysis
    pub note_count: i32,
    pub unique_pitches: Option<i32>,
    pub pitch_range_low: Option<i16>,
    pub pitch_range_high: Option<i16>,
    pub pitch_range_semitones: Option<i16>,

    // Velocity
    pub avg_velocity: Option<f64>,
    pub velocity_range_low: Option<i16>,
    pub velocity_range_high: Option<i16>,

    // Note density
    pub note_density: Option<f64>,

    // Polyphony
    pub polyphony_max: Option<i16>,
    pub polyphony_avg: Option<f64>,

    // Characteristics
    pub is_monophonic: bool,
    pub is_polyphonic: bool,
    pub is_percussive: bool,

    // Chord analysis
    pub has_chords: bool,
    pub chord_progression: Option<Vec<String>>,
    pub chord_types: Option<Vec<String>>,
    pub has_seventh_chords: bool,
    pub has_extended_chords: bool,
    pub chord_change_rate: Option<f32>,
    pub chord_complexity_score: Option<f32>,

    // Melody
    pub has_melody: bool,
    pub melodic_range: Option<i16>,

    // Variation tracking (JSON timelines)
    pub tempo_changes: Option<String>,
    pub key_changes: Option<String>,
    pub time_signature_changes: Option<String>,

    // Controller analysis (JSON)
    pub controller_data: Option<String>,

    // Articulation/Performance analysis (JSON)
    pub articulation_data: Option<String>,

    // Structure/Form analysis (JSON)
    pub structure_data: Option<String>,

    // Complexity
    pub complexity_score: Option<f64>,

    // Additional properties
    pub instruments: Vec<String>,
    pub track_instruments: Vec<TrackInstrument>,
    pub has_pitch_bend: bool,
    pub has_cc_messages: bool,
}

/// Analyze all unanalyzed MIDI files with parallel processing.
#[tauri::command]
pub async fn start_analysis(
    state: State<'_, AppState>,
    window: Window,
) -> Result<AnalysisSummary, String> {
    let start_time = std::time::Instant::now();
    let pool: sqlx::PgPool = state.database.pool().await;

    // Get total count of unanalyzed files
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

    // Thread-safe counters
    let analyzed = Arc::new(AtomicUsize::new(0));
    let skipped = Arc::new(AtomicUsize::new(0));
    let errors = Arc::new(Mutex::new(Vec::new()));
    let current_index = Arc::new(AtomicUsize::new(0));

    let semaphore = Arc::new(tokio::sync::Semaphore::new(CONCURRENCY_LIMIT));

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
        .bind(BATCH_SIZE)
        .bind(offset)
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("Failed to fetch files: {}", e))?;

        if files.is_empty() {
            break;
        }

        let batch_len = files.len();
        println!(
            "📦 Processing batch: {} files (offset: {})",
            batch_len, offset
        );

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
                    let _permit = match sem.acquire().await {
                        Ok(permit) => permit,
                        Err(e) => {
                            let error_msg =
                                format!("FATAL: Semaphore unavailable during analysis: {}", e);
                            eprintln!("ERROR: {}", error_msg);

                            // Track this as an error
                            errors.lock().await.push(error_msg);

                            // Mark file as skipped
                            skipped.fetch_add(1, Ordering::SeqCst);
                            return;
                        },
                    };

                    let current = current_index.fetch_add(1, Ordering::SeqCst) + 1;

                    // Emit progress every 10 files
                    if current.is_multiple_of(10) || current == total_usize {
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
                            // Log but don't fail the operation
                            eprintln!(
                                "WARNING: Failed to emit analysis progress (file {}): {}",
                                file_record.filename, e
                            );
                        }

                        // Print progress every 100 files
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

                                // CRITICAL: Database batch insert - if this fails, analysis data is lost
                                if let Err(e) = batch_insert_analyzed_files(&batch, &pool).await {
                                    let error_msg = format!("Batch insert failed: {}", e);
                                    eprintln!("ERROR: {}", error_msg);

                                    // Record the error and mark files as skipped
                                    errors.lock().await.push(error_msg);
                                    skipped.fetch_add(batch.len(), Ordering::SeqCst);
                                }
                            }
                        },
                        Err(e) => {
                            let error_msg = format!("{}: {}", file_record.filepath, e);
                            errors.lock().await.push(error_msg);
                            skipped.fetch_add(1, Ordering::SeqCst);
                        },
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

    // Calculate final statistics
    let duration = start_time.elapsed().as_secs_f64();
    let analyzed_count = analyzed.load(Ordering::SeqCst);
    let rate = if duration > 0.0 {
        analyzed_count as f64 / duration
    } else {
        0.0
    };

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

    // 12. Melody detection (simple heuristic: monophonic content)
    let has_melody = note_stats.is_monophonic
        || (note_stats.polyphony_avg.is_some_and(|p| p < 2.0) && note_stats.note_count > 10);
    let melodic_range = if has_melody {
        note_stats.pitch_range_semitones
    } else {
        None
    };

    // 13. Extract variation timelines (tempo, key, time signature changes)
    let tempo_changes = extract_tempo_changes(&midi_file);
    let key_changes = extract_key_changes(&midi_file);
    let time_signature_changes = extract_time_signature_changes(&midi_file);

    // 14. Controller analysis (CC messages)
    let controller_data = analyze_controllers(&midi_file);

    // 15. Articulation/Performance analysis
    let tempo_us_per_qn = (60_000_000.0 / bpm_result.bpm) as u32;
    let articulation_data = analyze_articulation(&midi_file, tempo_us_per_qn);

    // 16. Structure/Form analysis
    let structure_data = analyze_structure(&midi_file);

    // 17. Calculate complexity score (simple heuristic)
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

/// Batch insert analyzed files into musical_metadata and update files.analyzed_at
pub async fn batch_insert_analyzed_files(
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
                bpm,
                bpm_confidence,
                has_tempo_changes,
                key_signature,
                key_confidence,
                time_signature_numerator,
                time_signature_denominator,
                total_notes,
                unique_pitches,
                pitch_range_min,
                pitch_range_max,
                avg_velocity,
                note_density,
                polyphony_max,
                polyphony_avg,
                is_monophonic,
                is_polyphonic,
                is_percussive,
                has_chords,
                chord_progression,
                chord_types,
                has_seventh_chords,
                has_extended_chords,
                chord_change_rate,
                chord_complexity_score,
                has_melody,
                melodic_range,
                tempo_changes,
                key_changes,
                time_signature_changes,
                controller_data,
                articulation_data,
                structure_data
            ) VALUES ($1, $2, $3, $4, $5::musical_key, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21::jsonb, $22, $23, $24, $25, $26, $27, $28, $29::jsonb, $30::jsonb, $31::jsonb, $32::jsonb, $33::jsonb, $34::jsonb)
            ON CONFLICT (file_id) DO UPDATE SET
                bpm = EXCLUDED.bpm,
                bpm_confidence = EXCLUDED.bpm_confidence,
                has_tempo_changes = EXCLUDED.has_tempo_changes,
                key_signature = EXCLUDED.key_signature,
                key_confidence = EXCLUDED.key_confidence,
                time_signature_numerator = EXCLUDED.time_signature_numerator,
                time_signature_denominator = EXCLUDED.time_signature_denominator,
                total_notes = EXCLUDED.total_notes,
                unique_pitches = EXCLUDED.unique_pitches,
                pitch_range_min = EXCLUDED.pitch_range_min,
                pitch_range_max = EXCLUDED.pitch_range_max,
                avg_velocity = EXCLUDED.avg_velocity,
                note_density = EXCLUDED.note_density,
                polyphony_max = EXCLUDED.polyphony_max,
                polyphony_avg = EXCLUDED.polyphony_avg,
                is_monophonic = EXCLUDED.is_monophonic,
                is_polyphonic = EXCLUDED.is_polyphonic,
                is_percussive = EXCLUDED.is_percussive,
                has_chords = EXCLUDED.has_chords,
                chord_progression = EXCLUDED.chord_progression,
                chord_types = EXCLUDED.chord_types,
                has_seventh_chords = EXCLUDED.has_seventh_chords,
                has_extended_chords = EXCLUDED.has_extended_chords,
                chord_change_rate = EXCLUDED.chord_change_rate,
                chord_complexity_score = EXCLUDED.chord_complexity_score,
                has_melody = EXCLUDED.has_melody,
                melodic_range = EXCLUDED.melodic_range,
                tempo_changes = EXCLUDED.tempo_changes,
                key_changes = EXCLUDED.key_changes,
                time_signature_changes = EXCLUDED.time_signature_changes,
                controller_data = EXCLUDED.controller_data,
                articulation_data = EXCLUDED.articulation_data,
                structure_data = EXCLUDED.structure_data
            "#
        )
        .bind(file.file_id)
        .bind(file.tempo_bpm)
        .bind(file.bpm_confidence)
        .bind(file.has_tempo_variation)
        .bind(&file.key_signature)
        .bind(file.key_confidence)
        .bind(file.time_signature_num)
        .bind(file.time_signature_den)
        .bind(file.note_count)
        .bind(file.unique_pitches)
        .bind(file.pitch_range_low)
        .bind(file.pitch_range_high)
        .bind(file.avg_velocity)
        .bind(file.note_density)
        .bind(file.polyphony_max)
        .bind(file.polyphony_avg)
        .bind(file.is_monophonic)
        .bind(file.is_polyphonic)
        .bind(file.is_percussive)
        .bind(file.has_chords)
        .bind(file.chord_progression.as_ref().and_then(|v| serde_json::to_string(v).ok()))
        .bind(&file.chord_types)
        .bind(file.has_seventh_chords)
        .bind(file.has_extended_chords)
        .bind(file.chord_change_rate)
        .bind(file.chord_complexity_score)
        .bind(file.has_melody)
        .bind(file.melodic_range)
        .bind(&file.tempo_changes)
        .bind(&file.key_changes)
        .bind(&file.time_signature_changes)
        .bind(&file.controller_data)
        .bind(&file.articulation_data)
        .bind(&file.structure_data)
        .execute(&mut *tx)
        .await?;

        // Insert track instruments into file_instruments table
        for inst in &file.track_instruments {
            sqlx::query(
                r#"
                INSERT INTO file_instruments (
                    file_id, channel, program_number, program_name,
                    instrument_family, instrument_type, note_count,
                    is_primary, avg_velocity, pitch_range_low, pitch_range_high
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                ON CONFLICT (file_id, channel, program_number) DO UPDATE SET
                    program_name = EXCLUDED.program_name,
                    instrument_family = EXCLUDED.instrument_family,
                    instrument_type = EXCLUDED.instrument_type,
                    note_count = EXCLUDED.note_count,
                    is_primary = EXCLUDED.is_primary,
                    avg_velocity = EXCLUDED.avg_velocity,
                    pitch_range_low = EXCLUDED.pitch_range_low,
                    pitch_range_high = EXCLUDED.pitch_range_high
                "#,
            )
            .bind(file.file_id)
            .bind(inst.channel)
            .bind(inst.program_number)
            .bind(&inst.program_name)
            .bind(&inst.instrument_family)
            .bind(&inst.instrument_type)
            .bind(inst.note_count)
            .bind(inst.is_primary)
            .bind(inst.avg_velocity)
            .bind(inst.pitch_range_low)
            .bind(inst.pitch_range_high)
            .execute(&mut *tx)
            .await?;
        }

        // Update files.analyzed_at timestamp
        sqlx::query("UPDATE files SET analyzed_at = NOW() WHERE id = $1")
            .bind(file.file_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    Ok(())
}

/// Note statistics
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct NoteStats {
    note_count: i32,
    unique_pitches: Option<i32>,
    pitch_range_low: Option<i16>,
    pitch_range_high: Option<i16>,
    pitch_range_semitones: Option<i16>,
    avg_velocity: Option<f64>,
    velocity_range_low: Option<i16>,
    velocity_range_high: Option<i16>,
    note_density: Option<f64>,
    polyphony_max: Option<i16>,
    polyphony_avg: Option<f64>,
    is_monophonic: bool,
    is_polyphonic: bool,
    is_percussive: bool,
}

/// Analyze notes in MIDI file
fn analyze_notes(midi_file: &MidiFile) -> NoteStats {
    let mut note_count = 0;
    let mut unique_pitch_set = std::collections::HashSet::new();
    let mut min_pitch = 127u8;
    let mut max_pitch = 0u8;
    let mut min_velocity = 127u8;
    let mut max_velocity = 0u8;
    let mut velocity_sum = 0u32;
    let mut active_notes_per_tick: std::collections::HashMap<u32, usize> =
        std::collections::HashMap::new();
    let mut max_tick = 0u32;
    let mut percussive_note_count = 0; // Notes on channel 10 (GM drums)

    for track in &midi_file.tracks {
        let mut current_tick = 0u32;
        let mut active_notes = std::collections::HashSet::new();

        for timed_event in &track.events {
            current_tick += timed_event.delta_ticks;
            max_tick = max_tick.max(current_tick);

            match &timed_event.event {
                Event::NoteOn { note, velocity, channel } if *velocity > 0 => {
                    note_count += 1;
                    unique_pitch_set.insert(*note);
                    min_pitch = min_pitch.min(*note);
                    max_pitch = max_pitch.max(*note);
                    min_velocity = min_velocity.min(*velocity);
                    max_velocity = max_velocity.max(*velocity);
                    velocity_sum += *velocity as u32;

                    // Channel 10 (index 9) is standard GM drums
                    if *channel == 9 {
                        percussive_note_count += 1;
                    }

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

    let unique_pitches = if note_count > 0 {
        Some(unique_pitch_set.len() as i32)
    } else {
        None
    };

    let avg_velocity = if note_count > 0 {
        Some(velocity_sum as f64 / note_count as f64)
    } else {
        None
    };

    let polyphony_max = active_notes_per_tick.values().max().copied().map(|v| v as i16);

    // Calculate average polyphony
    let polyphony_avg = if !active_notes_per_tick.is_empty() {
        let sum: usize = active_notes_per_tick.values().sum();
        Some(sum as f64 / active_notes_per_tick.len() as f64)
    } else {
        None
    };

    // Monophonic: max polyphony is 1
    let is_monophonic = polyphony_max == Some(1);

    // Polyphonic: max polyphony > 1
    let is_polyphonic = polyphony_max.is_some_and(|p| p > 1);

    // Percussive: >50% of notes on channel 10 OR pitch range in drum range (35-81)
    let is_percussive = if note_count > 0 {
        let drum_ratio = percussive_note_count as f64 / note_count as f64;
        let in_drum_range = min_pitch >= 27 && max_pitch <= 87; // GM drum range with buffer
        drum_ratio > 0.5 || in_drum_range
    } else {
        false
    };

    // Calculate note density (notes per second)
    let duration_seconds = if max_tick > 0 {
        // Estimate duration assuming 120 BPM if no tempo events
        let ticks_per_beat = midi_file.header.ticks_per_quarter_note as f64;
        let seconds_per_beat = 0.5; // 120 BPM = 0.5 seconds per beat
        Some((max_tick as f64 / ticks_per_beat) * seconds_per_beat)
    } else {
        None
    };

    let note_density = if let Some(duration) = duration_seconds {
        if duration > 0.0 {
            Some(note_count as f64 / duration)
        } else {
            None
        }
    } else {
        None
    };

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
        unique_pitches,
        pitch_range_low,
        pitch_range_high,
        pitch_range_semitones,
        avg_velocity,
        velocity_range_low,
        velocity_range_high,
        note_density,
        polyphony_max,
        polyphony_avg,
        is_monophonic,
        is_polyphonic,
        is_percussive,
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

/// Track-level instrument information
#[derive(Debug, Clone)]
pub struct TrackInstrument {
    pub channel: i16,
    pub program_number: i16,
    pub program_name: String,
    pub instrument_family: String,
    pub instrument_type: String,
    pub note_count: i32,
    pub avg_velocity: Option<f64>,
    pub pitch_range_low: Option<i16>,
    pub pitch_range_high: Option<i16>,
    pub is_primary: bool,
}

/// Analyze tracks and extract per-channel instrument information
fn analyze_tracks(midi_file: &MidiFile) -> Vec<TrackInstrument> {
    let mut channel_data: std::collections::HashMap<u8, TrackInstrument> =
        std::collections::HashMap::new();

    // Extract program changes and note events per channel
    for track in &midi_file.tracks {
        let mut current_programs: std::collections::HashMap<u8, u8> =
            std::collections::HashMap::new();

        for timed_event in &track.events {
            match &timed_event.event {
                Event::ProgramChange { channel, program } => {
                    current_programs.insert(*channel, *program);
                },
                Event::NoteOn { channel, note, velocity } if *velocity > 0 => {
                    let program = current_programs.get(channel).copied().unwrap_or(0);

                    let entry = channel_data.entry(*channel).or_insert_with(|| {
                        let (name, family, inst_type) = get_instrument_info(program);
                        TrackInstrument {
                            channel: *channel as i16,
                            program_number: program as i16,
                            program_name: name,
                            instrument_family: family,
                            instrument_type: inst_type,
                            note_count: 0,
                            avg_velocity: None,
                            pitch_range_low: None,
                            pitch_range_high: None,
                            is_primary: false,
                        }
                    });

                    entry.note_count += 1;

                    // Update pitch range
                    entry.pitch_range_low =
                        Some(entry.pitch_range_low.map_or(*note as i16, |l| l.min(*note as i16)));
                    entry.pitch_range_high =
                        Some(entry.pitch_range_high.map_or(*note as i16, |h| h.max(*note as i16)));

                    // Update average velocity (running average)
                    if let Some(avg) = entry.avg_velocity {
                        entry.avg_velocity = Some(
                            (avg * (entry.note_count - 1) as f64 + *velocity as f64)
                                / entry.note_count as f64,
                        );
                    } else {
                        entry.avg_velocity = Some(*velocity as f64);
                    }
                },
                _ => {},
            }
        }
    }

    // Convert to vec and mark primary instrument (most notes)
    let mut instruments: Vec<TrackInstrument> = channel_data.into_values().collect();
    if let Some(max_notes) = instruments.iter().map(|i| i.note_count).max() {
        for inst in &mut instruments {
            if inst.note_count == max_notes {
                inst.is_primary = true;
                break;
            }
        }
    }

    instruments
}

/// Extract tempo changes from MIDI meta events
fn extract_tempo_changes(midi_file: &MidiFile) -> Option<String> {
    let mut tempo_changes = Vec::new();
    let mut current_tick = 0u32;

    for track in &midi_file.tracks {
        let mut track_tick = 0u32;
        for timed_event in &track.events {
            track_tick += timed_event.delta_ticks;

            if let Event::TempoChange { microseconds_per_quarter } = &timed_event.event {
                let bpm = 60_000_000.0 / *microseconds_per_quarter as f64;
                tempo_changes.push(serde_json::json!({
                    "tick": track_tick,
                    "bpm": (bpm * 100.0).round() / 100.0 // Round to 2 decimals
                }));
            }
        }
        current_tick = current_tick.max(track_tick);
    }

    if tempo_changes.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&tempo_changes).unwrap_or_default())
    }
}

/// Extract key signature changes from MIDI meta events
fn extract_key_changes(midi_file: &MidiFile) -> Option<String> {
    let mut key_changes = Vec::new();
    let mut current_tick = 0u32;

    for track in &midi_file.tracks {
        let mut track_tick = 0u32;
        for timed_event in &track.events {
            track_tick += timed_event.delta_ticks;

            if let Event::KeySignature { sharps_flats, is_minor } = &timed_event.event {
                let key_name = get_key_name(*sharps_flats, *is_minor);
                key_changes.push(serde_json::json!({
                    "tick": track_tick,
                    "key": key_name
                }));
            }
        }
        current_tick = current_tick.max(track_tick);
    }

    if key_changes.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&key_changes).unwrap_or_default())
    }
}

/// Extract time signature changes from MIDI meta events
fn extract_time_signature_changes(midi_file: &MidiFile) -> Option<String> {
    let mut time_sig_changes = Vec::new();
    let mut current_tick = 0u32;

    for track in &midi_file.tracks {
        let mut track_tick = 0u32;
        for timed_event in &track.events {
            track_tick += timed_event.delta_ticks;

            if let Event::TimeSignature { numerator, denominator, .. } = &timed_event.event {
                let denom_value = 2i32.pow(*denominator as u32);
                time_sig_changes.push(serde_json::json!({
                    "tick": track_tick,
                    "numerator": numerator,
                    "denominator": denom_value
                }));
            }
        }
        current_tick = current_tick.max(track_tick);
    }

    if time_sig_changes.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&time_sig_changes).unwrap_or_default())
    }
}

/// Convert sharps/flats to key name
fn get_key_name(sharps_flats: i8, is_minor: bool) -> String {
    let major_keys = [
        "C", "G", "D", "A", "E", "B", "F#", "C#", "F", "Bb", "Eb", "Ab", "Db", "Gb", "Cb",
    ];
    let minor_keys = [
        "Am", "Em", "Bm", "F#m", "C#m", "G#m", "D#m", "A#m", "Dm", "Gm", "Cm", "Fm", "Bbm", "Ebm",
        "Abm",
    ];

    let index = if sharps_flats >= 0 {
        sharps_flats as usize
    } else {
        // For flats: -1 -> index 8 (F), -2 -> index 9 (Bb), etc.
        (8 - sharps_flats - 1) as usize
    };

    if is_minor {
        minor_keys.get(index).unwrap_or(&"Unknown").to_string()
    } else {
        major_keys.get(index).unwrap_or(&"Unknown").to_string()
    }
}

/// Controller statistics for a single CC number
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ControllerStats {
    cc_number: u8,
    count: u32,
    min_value: u8,
    max_value: u8,
    avg_value: f64,
}

/// Analyze MIDI controller (CC) messages
fn analyze_controllers(midi_file: &MidiFile) -> Option<String> {
    use std::collections::HashMap;

    // Track statistics for each controller number
    let mut controller_data: HashMap<u8, (u32, u8, u8, u64)> = HashMap::new(); // (count, min, max, sum)

    // High-priority controllers to track
    let priority_controllers = [1, 2, 7, 10, 11, 64]; // Modulation, Breath, Volume, Pan, Expression, Sustain

    for track in &midi_file.tracks {
        for timed_event in &track.events {
            if let Event::ControlChange { controller, value, .. } = &timed_event.event {
                let entry = controller_data.entry(*controller).or_insert((0, 255, 0, 0));
                entry.0 += 1; // count
                entry.1 = entry.1.min(*value); // min
                entry.2 = entry.2.max(*value); // max
                entry.3 += *value as u64; // sum for average
            }
        }
    }

    if controller_data.is_empty() {
        return None;
    }

    // Build JSON array of controller statistics
    let mut controllers = Vec::new();

    // First add priority controllers if present
    for &cc in &priority_controllers {
        if let Some(&(count, min, max, sum)) = controller_data.get(&cc) {
            let avg = sum as f64 / count as f64;
            controllers.push(serde_json::json!({
                "cc": cc,
                "name": get_cc_name(cc),
                "count": count,
                "min": min,
                "max": max,
                "avg": (avg * 100.0).round() / 100.0
            }));
        }
    }

    // Then add other controllers with significant usage (>10 events)
    for (&cc, &(count, min, max, sum)) in &controller_data {
        if !priority_controllers.contains(&cc) && count > 10 {
            let avg = sum as f64 / count as f64;
            controllers.push(serde_json::json!({
                "cc": cc,
                "name": get_cc_name(cc),
                "count": count,
                "min": min,
                "max": max,
                "avg": (avg * 100.0).round() / 100.0
            }));
        }
    }

    if controllers.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&controllers).unwrap_or_default())
    }
}

/// Get human-readable name for CC number
fn get_cc_name(cc: u8) -> &'static str {
    match cc {
        0 => "Bank Select",
        1 => "Modulation Wheel",
        2 => "Breath Controller",
        4 => "Foot Controller",
        5 => "Portamento Time",
        6 => "Data Entry",
        7 => "Channel Volume",
        8 => "Balance",
        10 => "Pan",
        11 => "Expression",
        64 => "Sustain Pedal",
        65 => "Portamento",
        66 => "Sostenuto",
        67 => "Soft Pedal",
        68 => "Legato Footswitch",
        69 => "Hold 2",
        71 => "Resonance",
        72 => "Release Time",
        73 => "Attack Time",
        74 => "Cutoff Frequency",
        84 => "Portamento Control",
        91 => "Reverb",
        92 => "Tremolo",
        93 => "Chorus",
        94 => "Detune",
        95 => "Phaser",
        _ => "Other",
    }
}

/// Articulation and performance characteristics
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ArticulationAnalysis {
    legato_percentage: f64,
    staccato_percentage: f64,
    avg_note_duration_ms: f64,
    timing_deviation_ms: f64,
    dynamic_range: u8,
    velocity_variance: f64,
}

/// Analyze articulation and performance characteristics
fn analyze_articulation(midi_file: &MidiFile, tempo_us_per_qn: u32) -> Option<String> {
    let tpq = midi_file.header.ticks_per_quarter_note as f64;
    let us_per_tick = tempo_us_per_qn as f64 / tpq;
    let ms_per_tick = us_per_tick / 1000.0;

    // Track note events per channel
    let mut note_events: std::collections::HashMap<(u8, u8), (u32, u8)> =
        std::collections::HashMap::new(); // (channel, pitch) -> (start_tick, velocity)
    let mut note_durations: Vec<f64> = Vec::new();
    let mut note_velocities: Vec<u8> = Vec::new();
    let mut timing_deviations: Vec<f64> = Vec::new();
    let mut legato_count = 0;
    let mut staccato_count = 0;
    let mut total_notes = 0;

    for track in &midi_file.tracks {
        let mut current_tick = 0u32;
        let mut active_notes: std::collections::HashSet<(u8, u8)> =
            std::collections::HashSet::new();

        for timed_event in &track.events {
            current_tick += timed_event.delta_ticks;

            match &timed_event.event {
                Event::NoteOn { channel, note, velocity } if *velocity > 0 => {
                    // Check for legato (note starts while others are active)
                    if !active_notes.is_empty() {
                        legato_count += 1;
                    }

                    active_notes.insert((*channel, *note));
                    note_events.insert((*channel, *note), (current_tick, *velocity));
                    note_velocities.push(*velocity);

                    // Calculate timing deviation from grid (16th note = tpq/4)
                    let grid_size = (tpq / 4.0) as u32; // 16th note grid
                    let deviation = (current_tick % grid_size) as f64;
                    let normalized_deviation = if deviation > grid_size as f64 / 2.0 {
                        grid_size as f64 - deviation
                    } else {
                        deviation
                    };
                    timing_deviations.push(normalized_deviation * ms_per_tick);

                    total_notes += 1;
                },
                Event::NoteOff { channel, note, .. }
                | Event::NoteOn { channel, note, velocity: 0 } => {
                    if let Some((start_tick, _)) = note_events.remove(&(*channel, *note)) {
                        let duration_ticks = current_tick.saturating_sub(start_tick);
                        let duration_ms = duration_ticks as f64 * ms_per_tick;
                        note_durations.push(duration_ms);

                        // Staccato detection: very short notes (<100ms)
                        if duration_ms < 100.0 {
                            staccato_count += 1;
                        }
                    }
                    active_notes.remove(&(*channel, *note));
                },
                _ => {},
            }
        }
    }

    if total_notes == 0 {
        return None;
    }

    // Calculate statistics
    let legato_percentage = (legato_count as f64 / total_notes as f64) * 100.0;
    let staccato_percentage = (staccato_count as f64 / total_notes as f64) * 100.0;

    let avg_note_duration = note_durations.iter().sum::<f64>() / note_durations.len().max(1) as f64;

    let avg_timing_deviation =
        timing_deviations.iter().sum::<f64>() / timing_deviations.len().max(1) as f64;

    let min_velocity = *note_velocities.iter().min().unwrap_or(&0);
    let max_velocity = *note_velocities.iter().max().unwrap_or(&127);
    let dynamic_range = max_velocity.saturating_sub(min_velocity);

    let avg_velocity = note_velocities.iter().map(|&v| v as f64).sum::<f64>()
        / note_velocities.len().max(1) as f64;
    let velocity_variance = note_velocities
        .iter()
        .map(|&v| {
            let diff = v as f64 - avg_velocity;
            diff * diff
        })
        .sum::<f64>()
        / note_velocities.len().max(1) as f64;

    let result = serde_json::json!({
        "legato_percentage": (legato_percentage * 100.0).round() / 100.0,
        "staccato_percentage": (staccato_percentage * 100.0).round() / 100.0,
        "avg_note_duration_ms": (avg_note_duration * 100.0).round() / 100.0,
        "timing_deviation_ms": (avg_timing_deviation * 100.0).round() / 100.0,
        "dynamic_range": dynamic_range,
        "velocity_variance": (velocity_variance * 100.0).round() / 100.0,
        "is_humanized": avg_timing_deviation > 2.0, // >2ms average deviation suggests human/humanized performance
        "is_legato": legato_percentage > 30.0,
        "is_staccato": staccato_percentage > 50.0,
    });

    Some(serde_json::to_string(&result).unwrap_or_default())
}

/// Analyze musical structure and form
fn analyze_structure(midi_file: &MidiFile) -> Option<String> {
    // Divide the file into segments (every 4 measures, assuming 4/4)
    let tpq = midi_file.header.ticks_per_quarter_note as u32;
    let segment_size = tpq * 16; // 4 measures in 4/4 time (16 quarter notes)

    // Find total duration in ticks
    let mut max_tick = 0u32;
    for track in &midi_file.tracks {
        let mut current_tick = 0u32;
        for event in &track.events {
            current_tick += event.delta_ticks;
        }
        max_tick = max_tick.max(current_tick);
    }

    if max_tick < segment_size {
        return None; // File too short for structure analysis
    }

    // Create segment hashes based on note patterns
    let num_segments = (max_tick / segment_size) as usize;
    let mut segment_hashes: Vec<u64> = vec![0; num_segments];

    for track in &midi_file.tracks {
        let mut current_tick = 0u32;
        for event in &track.events {
            current_tick += event.delta_ticks;

            if let Event::NoteOn { note, velocity, .. } = &event.event {
                if *velocity > 0 {
                    let segment_idx = (current_tick / segment_size) as usize;
                    if segment_idx < num_segments {
                        // Simple hash: combine note and position within segment
                        let position = current_tick % segment_size;
                        let hash = ((*note as u64) << 32) | position as u64;
                        segment_hashes[segment_idx] ^= hash; // XOR for simplicity
                    }
                }
            }
        }
    }

    // Find repeated patterns
    let mut pattern_map: std::collections::HashMap<u64, Vec<usize>> =
        std::collections::HashMap::new();
    for (idx, &hash) in segment_hashes.iter().enumerate() {
        if hash != 0 {
            pattern_map.entry(hash).or_default().push(idx);
        }
    }

    // Identify major repeated sections (appears 2+ times)
    let mut repeated_sections: Vec<(usize, usize)> = Vec::new(); // (pattern_id, count)
    let mut pattern_id = 0;
    for (_hash, positions) in pattern_map.iter() {
        if positions.len() >= 2 {
            repeated_sections.push((pattern_id, positions.len()));
            pattern_id += 1;
        }
    }

    // Calculate repetition percentage
    let total_repeated: usize = repeated_sections.iter().map(|(_, count)| count).sum();
    let repetition_percentage = if num_segments > 0 {
        (total_repeated as f64 / num_segments as f64) * 100.0
    } else {
        0.0
    };

    // Estimate form based on number of unique patterns
    let num_unique = pattern_map.len();
    let estimated_form = if num_unique <= 2 {
        "Simple (AA or AB)"
    } else if num_unique <= 4 {
        "Song Form (AABA or ABAB)"
    } else if num_unique <= 6 {
        "Complex (ABABCB or similar)"
    } else {
        "Through-composed"
    };

    let result = serde_json::json!({
        "num_segments": num_segments,
        "num_unique_patterns": num_unique,
        "num_repeated_patterns": repeated_sections.len(),
        "repetition_percentage": (repetition_percentage * 100.0).round() / 100.0,
        "estimated_form": estimated_form,
        "has_repetition": repetition_percentage > 20.0,
        "is_through_composed": num_unique > 6,
    });

    Some(serde_json::to_string(&result).unwrap_or_default())
}

/// Get instrument information from GM program number
fn get_instrument_info(program: u8) -> (String, String, String) {
    match program {
        0..=7 => (
            "Piano".to_string(),
            "Keyboard".to_string(),
            "Acoustic Piano".to_string(),
        ),
        8..=15 => (
            "Chromatic Percussion".to_string(),
            "Keyboard".to_string(),
            "Celesta/Glockenspiel".to_string(),
        ),
        16..=23 => (
            "Organ".to_string(),
            "Keyboard".to_string(),
            "Drawbar Organ".to_string(),
        ),
        24..=31 => (
            "Guitar".to_string(),
            "Strings".to_string(),
            "Acoustic Guitar".to_string(),
        ),
        32..=39 => (
            "Bass".to_string(),
            "Strings".to_string(),
            "Electric Bass".to_string(),
        ),
        40..=47 => (
            "Strings".to_string(),
            "Strings".to_string(),
            "Violin/Viola".to_string(),
        ),
        48..=55 => (
            "Ensemble".to_string(),
            "Ensemble".to_string(),
            "String Ensemble".to_string(),
        ),
        56..=63 => (
            "Brass".to_string(),
            "Brass".to_string(),
            "Trumpet/Trombone".to_string(),
        ),
        64..=71 => (
            "Reed".to_string(),
            "Reed".to_string(),
            "Saxophone".to_string(),
        ),
        72..=79 => (
            "Pipe".to_string(),
            "Pipe".to_string(),
            "Flute/Piccolo".to_string(),
        ),
        80..=87 => (
            "Synth Lead".to_string(),
            "Synth".to_string(),
            "Lead Synth".to_string(),
        ),
        88..=95 => (
            "Synth Pad".to_string(),
            "Synth".to_string(),
            "Pad Synth".to_string(),
        ),
        96..=103 => (
            "Synth Effects".to_string(),
            "Synth".to_string(),
            "FX Synth".to_string(),
        ),
        104..=111 => (
            "Ethnic".to_string(),
            "Ethnic".to_string(),
            "Sitar/Shamisen".to_string(),
        ),
        112..=119 => (
            "Percussive".to_string(),
            "Percussion".to_string(),
            "Timpani/Taiko".to_string(),
        ),
        120..=127 => (
            "Sound Effects".to_string(),
            "SFX".to_string(),
            "Sound Effect".to_string(),
        ),
        _ => (
            "Unknown".to_string(),
            "Unknown".to_string(),
            "Unknown".to_string(),
        ),
    }
}

/// Calculate complexity score based on various factors
fn calculate_complexity_score(note_stats: &NoteStats, midi_file: &MidiFile) -> Option<f64> {
    if note_stats.note_count == 0 {
        return Some(0.0);
    }

    let mut score = 0.0;

    // Factor 1: Note density (notes per second)
    // Assume average 120 BPM for rough estimate
    let duration_est = calculate_total_ticks(midi_file) as f64
        / (midi_file.header.ticks_per_quarter_note as f64 * 2.0);
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
    if let (Some(low), Some(high)) = (
        note_stats.velocity_range_low,
        note_stats.velocity_range_high,
    ) {
        let velocity_range = (high - low) as f64;
        score += (velocity_range / 10.0).min(10.0); // Max 10 points
    }

    // Normalize to 0-100 scale
    Some(score.min(100.0))
}

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
            unique_pitches: None,
            pitch_range_low: None,
            pitch_range_high: None,
            pitch_range_semitones: None,
            avg_velocity: None,
            velocity_range_low: None,
            velocity_range_high: None,
            note_density: None,
            polyphony_max: None,
            polyphony_avg: None,
            is_monophonic: false,
            is_polyphonic: false,
            is_percussive: false,
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

    #[test]
    fn test_extract_tempo_changes() {
        use midi_library_shared::core::midi::types::{Event, TimedEvent, Track};

        let midi_file = MidiFile {
            header: midi_library_shared::core::midi::types::Header {
                format: 1,
                num_tracks: 1,
                ticks_per_quarter_note: 480,
            },
            tracks: vec![Track {
                events: vec![
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::TempoChange {
                            microseconds_per_quarter: 500_000, // 120 BPM
                        },
                    },
                    TimedEvent {
                        delta_ticks: 1920,
                        event: Event::TempoChange {
                            microseconds_per_quarter: 600_000, // 100 BPM
                        },
                    },
                ],
            }],
        };

        let result = extract_tempo_changes(&midi_file);
        assert!(result.is_some());

        let json_str = result.unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["tick"], 0);
        assert_eq!(parsed[0]["bpm"], 120.0);
        assert_eq!(parsed[1]["tick"], 1920);
        assert_eq!(parsed[1]["bpm"], 100.0);
    }

    #[test]
    fn test_extract_key_changes() {
        use midi_library_shared::core::midi::types::{Event, TimedEvent, Track};

        let midi_file = MidiFile {
            header: midi_library_shared::core::midi::types::Header {
                format: 1,
                num_tracks: 1,
                ticks_per_quarter_note: 480,
            },
            tracks: vec![Track {
                events: vec![
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::KeySignature {
                            sharps_flats: 0, // C major
                            is_minor: false,
                        },
                    },
                    TimedEvent {
                        delta_ticks: 1920,
                        event: Event::KeySignature {
                            sharps_flats: 2, // D major
                            is_minor: false,
                        },
                    },
                ],
            }],
        };

        let result = extract_key_changes(&midi_file);
        assert!(result.is_some());

        let json_str = result.unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["tick"], 0);
        assert_eq!(parsed[0]["key"], "C");
        assert_eq!(parsed[1]["tick"], 1920);
        assert_eq!(parsed[1]["key"], "D");
    }

    #[test]
    fn test_extract_time_signature_changes() {
        use midi_library_shared::core::midi::types::{Event, TimedEvent, Track};

        let midi_file = MidiFile {
            header: midi_library_shared::core::midi::types::Header {
                format: 1,
                num_tracks: 1,
                ticks_per_quarter_note: 480,
            },
            tracks: vec![Track {
                events: vec![
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::TimeSignature {
                            numerator: 4,
                            denominator: 2, // 2^2 = 4, so 4/4 time
                            clocks_per_click: 24,
                            thirty_seconds_per_quarter: 8,
                        },
                    },
                    TimedEvent {
                        delta_ticks: 1920,
                        event: Event::TimeSignature {
                            numerator: 3,
                            denominator: 2, // 2^2 = 4, so 3/4 time
                            clocks_per_click: 24,
                            thirty_seconds_per_quarter: 8,
                        },
                    },
                ],
            }],
        };

        let result = extract_time_signature_changes(&midi_file);
        assert!(result.is_some());

        let json_str = result.unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["tick"], 0);
        assert_eq!(parsed[0]["numerator"], 4);
        assert_eq!(parsed[0]["denominator"], 4);
        assert_eq!(parsed[1]["tick"], 1920);
        assert_eq!(parsed[1]["numerator"], 3);
        assert_eq!(parsed[1]["denominator"], 4);
    }

    #[test]
    fn test_get_key_name() {
        // Major keys with sharps
        assert_eq!(get_key_name(0, false), "C");
        assert_eq!(get_key_name(1, false), "G");
        assert_eq!(get_key_name(2, false), "D");

        // Major keys with flats
        assert_eq!(get_key_name(-1, false), "F");
        assert_eq!(get_key_name(-2, false), "Bb");

        // Minor keys
        assert_eq!(get_key_name(0, true), "Am");
        assert_eq!(get_key_name(1, true), "Em");
        assert_eq!(get_key_name(-1, true), "Dm");
    }

    #[test]
    fn test_analyze_controllers() {
        use midi_library_shared::core::midi::types::{Event, TimedEvent, Track};

        let midi_file = MidiFile {
            header: midi_library_shared::core::midi::types::Header {
                format: 1,
                num_tracks: 1,
                ticks_per_quarter_note: 480,
            },
            tracks: vec![Track {
                events: vec![
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::ControlChange {
                            channel: 0,
                            controller: 7, // Volume
                            value: 100,
                        },
                    },
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::ControlChange {
                            channel: 0,
                            controller: 10, // Pan
                            value: 64,
                        },
                    },
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::ControlChange {
                            channel: 0,
                            controller: 1, // Modulation
                            value: 50,
                        },
                    },
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::ControlChange {
                            channel: 0,
                            controller: 1, // Modulation again
                            value: 70,
                        },
                    },
                ],
            }],
        };

        let result = analyze_controllers(&midi_file);
        assert!(result.is_some());

        let json_str = result.unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();

        // Should have 3 controllers (1, 7, 10)
        assert_eq!(parsed.len(), 3);

        // Find the modulation controller (CC1)
        let mod_controller = parsed.iter().find(|c| c["cc"] == 1).unwrap();
        assert_eq!(mod_controller["name"], "Modulation Wheel");
        assert_eq!(mod_controller["count"], 2);
        assert_eq!(mod_controller["min"], 50);
        assert_eq!(mod_controller["max"], 70);
        assert_eq!(mod_controller["avg"], 60.0);
    }

    #[test]
    fn test_analyze_controllers_empty() {
        use midi_library_shared::core::midi::types::{Event, TimedEvent, Track};

        let midi_file = MidiFile {
            header: midi_library_shared::core::midi::types::Header {
                format: 1,
                num_tracks: 1,
                ticks_per_quarter_note: 480,
            },
            tracks: vec![Track {
                events: vec![TimedEvent {
                    delta_ticks: 0,
                    event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                }],
            }],
        };

        let result = analyze_controllers(&midi_file);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_cc_name() {
        assert_eq!(get_cc_name(1), "Modulation Wheel");
        assert_eq!(get_cc_name(7), "Channel Volume");
        assert_eq!(get_cc_name(10), "Pan");
        assert_eq!(get_cc_name(64), "Sustain Pedal");
        assert_eq!(get_cc_name(99), "Other");
    }

    #[test]
    fn test_analyze_articulation() {
        use midi_library_shared::core::midi::types::{Event, TimedEvent, Track};

        let midi_file = MidiFile {
            header: midi_library_shared::core::midi::types::Header {
                format: 1,
                num_tracks: 1,
                ticks_per_quarter_note: 480,
            },
            tracks: vec![Track {
                events: vec![
                    // First note - normal duration
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                    },
                    TimedEvent {
                        delta_ticks: 240, // Half a quarter note
                        event: Event::NoteOff { channel: 0, note: 60, velocity: 64 },
                    },
                    // Second note - legato (overlaps conceptually)
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::NoteOn { channel: 0, note: 64, velocity: 80 },
                    },
                    TimedEvent {
                        delta_ticks: 120, // Staccato (short)
                        event: Event::NoteOff { channel: 0, note: 64, velocity: 64 },
                    },
                ],
            }],
        };

        let result = analyze_articulation(&midi_file, 500_000); // 120 BPM
        assert!(result.is_some());

        let json_str = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        // Verify structure
        assert!(parsed["legato_percentage"].is_number());
        assert!(parsed["staccato_percentage"].is_number());
        assert!(parsed["avg_note_duration_ms"].is_number());
        assert!(parsed["timing_deviation_ms"].is_number());
        assert!(parsed["dynamic_range"].is_number());
        assert!(parsed["velocity_variance"].is_number());
        assert!(parsed["is_humanized"].is_boolean());
        assert!(parsed["is_legato"].is_boolean());
        assert!(parsed["is_staccato"].is_boolean());

        // Dynamic range should be 100 - 80 = 20
        assert_eq!(parsed["dynamic_range"], 20);
    }

    #[test]
    fn test_analyze_articulation_empty() {
        use midi_library_shared::core::midi::types::Track;

        let midi_file = MidiFile {
            header: midi_library_shared::core::midi::types::Header {
                format: 1,
                num_tracks: 1,
                ticks_per_quarter_note: 480,
            },
            tracks: vec![Track { events: vec![] }],
        };

        let result = analyze_articulation(&midi_file, 500_000);
        assert!(result.is_none());
    }

    #[test]
    fn test_analyze_structure() {
        use midi_library_shared::core::midi::types::{Event, TimedEvent, Track};

        // Create a MIDI file with repeated patterns (simulate AABA form)
        let tpq = 480;
        let measure_ticks = (tpq * 4) as u32; // 4 beats per measure

        let midi_file = MidiFile {
            header: midi_library_shared::core::midi::types::Header {
                format: 1,
                num_tracks: 1,
                ticks_per_quarter_note: tpq,
            },
            tracks: vec![Track {
                events: vec![
                    // Section A (4 measures)
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                    },
                    TimedEvent {
                        delta_ticks: measure_ticks * 4,
                        event: Event::NoteOff { channel: 0, note: 60, velocity: 64 },
                    },
                    // Section A repeated (4 measures)
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                    },
                    TimedEvent {
                        delta_ticks: measure_ticks * 4,
                        event: Event::NoteOff { channel: 0, note: 60, velocity: 64 },
                    },
                    // Section B (4 measures) - different pattern
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::NoteOn { channel: 0, note: 64, velocity: 100 },
                    },
                    TimedEvent {
                        delta_ticks: measure_ticks * 4,
                        event: Event::NoteOff { channel: 0, note: 64, velocity: 64 },
                    },
                ],
            }],
        };

        let result = analyze_structure(&midi_file);
        assert!(result.is_some());

        let json_str = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        // Verify structure
        assert!(parsed["num_segments"].is_number());
        assert!(parsed["num_unique_patterns"].is_number());
        assert!(parsed["num_repeated_patterns"].is_number());
        assert!(parsed["repetition_percentage"].is_number());
        assert!(parsed["estimated_form"].is_string());
        assert!(parsed["has_repetition"].is_boolean());
        assert!(parsed["is_through_composed"].is_boolean());

        // Should have at least 3 segments (3 x 4 measures)
        assert!(parsed["num_segments"].as_u64().unwrap() >= 3);
    }

    #[test]
    fn test_analyze_structure_too_short() {
        use midi_library_shared::core::midi::types::{Event, TimedEvent, Track};

        let midi_file = MidiFile {
            header: midi_library_shared::core::midi::types::Header {
                format: 1,
                num_tracks: 1,
                ticks_per_quarter_note: 480,
            },
            tracks: vec![Track {
                events: vec![
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                    },
                    TimedEvent {
                        delta_ticks: 100, // Very short file
                        event: Event::NoteOff { channel: 0, note: 60, velocity: 64 },
                    },
                ],
            }],
        };

        let result = analyze_structure(&midi_file);
        assert!(result.is_none()); // Too short for structure analysis
    }
}

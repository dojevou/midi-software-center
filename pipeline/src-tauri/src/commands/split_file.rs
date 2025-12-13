//! Track Splitting Commands - Split multi-track MIDI files into individual tracks
//!
//! Handles database queries, file I/O, and transaction management.
//! Actual splitting logic delegated to track_splitter module.

use crate::core::analysis::bpm_detector::detect_bpm;
use crate::core::analysis::key_detector::detect_key;
use crate::core::hash::calculate_file_hash;
use crate::core::naming::generator::generate_production_layer_filename;
use crate::core::splitting::{split_tracks_with_repair, RepairResult, SplitError, SplitTrack};
use midi_library_shared::core::midi::parser::parse_midi_file;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Minimum confidence threshold for BPM/key detection results
const CONFIDENCE_THRESHOLD: f64 = 0.5;

/// Errors that can occur during split and import operations
#[derive(Error, Debug)]
pub enum SplitCommandError {
    #[error("File not found in database: {0}")]
    FileNotFound(i64),

    #[error("File not found on disk: {0}")]
    FileNotFoundOnDisk(String),

    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to split tracks: {0}")]
    SplitError(#[from] SplitError),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Failed to create output directory: {0}")]
    DirectoryCreationError(String),

    #[error("Transaction failed: {0}")]
    TransactionError(String),
}

// Convert to user-friendly string for Tauri commands
impl From<SplitCommandError> for String {
    fn from(err: SplitCommandError) -> String {
        err.to_string()
    }
}

/// Result of a successful split operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitResult {
    /// IDs of the newly created split files in the database
    pub split_file_ids: Vec<i64>,

    /// Number of tracks that were split
    pub tracks_split: usize,

    /// Directory where split files were written
    pub output_dir: PathBuf,
}

/// Split a multi-track MIDI file and import each track as a separate file.
///
/// Queries DB for file info, reads MIDI, splits tracks, writes outputs,
/// imports to DB with metadata, and creates parent-child relationships.
pub async fn split_and_import(
    file_id: i64,
    output_dir: PathBuf,
    pool: &sqlx::PgPool,
) -> Result<SplitResult, SplitCommandError> {
    // 1. Query database for parent file info with metadata for Production naming
    let parent_file = sqlx::query!(
        r#"
        SELECT f.id, f.filename, f.original_filename, f.filepath, f.parent_folder,
               m.bpm, m.key_signature::text as "key_signature?"
        FROM files f
        LEFT JOIN musical_metadata m ON f.id = m.file_id
        WHERE f.id = $1
        "#,
        file_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| SplitCommandError::DatabaseError(e.to_string()))?
    .ok_or(SplitCommandError::FileNotFound(file_id))?;

    // 2. Read original file from disk
    let file_path = Path::new(&parent_file.filepath);
    if !file_path.exists() {
        return Err(SplitCommandError::FileNotFoundOnDisk(
            parent_file.filepath.clone(),
        ));
    }

    let original_bytes = tokio::fs::read(file_path).await?;

    // 2b. Parse MIDI to extract time signature for Production naming
    let midi_data = parse_midi_file(&original_bytes)
        .map_err(|e| SplitCommandError::DatabaseError(format!("Failed to parse MIDI: {}", e)))?;

    // Extract time signature from events (default to 4-4)
    let time_signature =
        extract_time_signature_from_midi(&midi_data).unwrap_or_else(|| "4-4".to_string());

    // 3. Call Trusty Module to split tracks with automatic repair
    let (split_tracks, repair_result) = split_tracks_with_repair(&original_bytes)
        .map_err(|e| SplitCommandError::SplitError(SplitError::ParseError(e.to_string())))?;

    // Log repair if it occurred
    match &repair_result {
        RepairResult::Valid => {
            // File was valid, no repair needed
        },
        RepairResult::Repaired { fix_description, .. } => {
            eprintln!(
                "🔧 REPAIRED: {} - {}",
                parent_file.filename, fix_description
            );
        },
        RepairResult::Corrupt { reason } => {
            eprintln!("❌ CORRUPT: {} - {}", parent_file.filename, reason);
        },
    }

    if split_tracks.is_empty() {
        return Err(SplitCommandError::SplitError(SplitError::NoTracksToSplit));
    }

    // 4. Create output directory if it doesn't exist
    if !output_dir.exists() {
        tokio::fs::create_dir_all(&output_dir)
            .await
            .map_err(|e| SplitCommandError::DirectoryCreationError(e.to_string()))?;
    }

    // 5. Process each split track with Production naming
    let mut split_file_ids = Vec::new();

    // Extract metadata for Production template
    // Query musical_metadata for category if available
    // query_scalar with fetch_optional returns Result<Option<Option<String>>>
    // We unwrap the Result, then flatten the nested Options to get Option<String>,
    // then unwrap_or to get the final String value
    let category = sqlx::query_scalar::<_, Option<String>>(
        "SELECT category FROM musical_metadata WHERE file_id = $1"
    )
    .bind(parent_file.id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()  // Flattens Option<Option<Option<String>>> to Option<Option<String>>
    .flatten()  // Flattens Option<Option<String>> to Option<String>
    .unwrap_or_else(|| "MIDI".to_string());

    let pack_name = parent_file.parent_folder.clone().unwrap_or_else(|| "Unknown".to_string());
    let file_id_str = format!("{:06}", parent_file.id);

    // Convert Option<BigDecimal> to f64 for BPM
    let bpm = parent_file
        .bpm
        .as_ref()
        .and_then(|bd| bd.to_string().parse::<f64>().ok())
        .unwrap_or(120.0);

    let key_signature = parent_file.key_signature.clone().unwrap_or_else(|| "C".to_string());

    for (layer_idx, split_track) in split_tracks.iter().enumerate() {
        // Extract layer name from instrument or track name
        let layer_name = if let Some(ref instrument) = split_track.instrument {
            instrument.clone()
        } else if let Some(ref track_name) = split_track.track_name {
            track_name.clone()
        } else {
            format!("Track{:02}", split_track.track_number)
        };

        // Generate Production filename: {CATEGORY}_{TIMESIG}_{BPM}BPM_{KEY}_{ID}_{PACK}_{LAYER}_L{NUM}.mid
        let filename = generate_production_layer_filename(
            category.as_str(), // Convert String to &str for function parameter
            bpm,
            &key_signature,
            &file_id_str,
            &time_signature,
            &pack_name,
            &layer_name,
            layer_idx + 1, // 1-based layer numbering
        );

        // Full path for split file
        let split_path = output_dir.join(&filename);

        // Write MIDI bytes to disk
        tokio::fs::write(&split_path, &split_track.midi_bytes).await?;

        // Import split file to database with full metadata
        let split_file_id =
            import_split_track(&split_path, &filename, &split_track.midi_bytes, pool)
                .await
                .map_err(|e| SplitCommandError::DatabaseError(e.to_string()))?;

        // Create relationship in track_splits table
        insert_track_split_relationship(file_id, split_file_id, split_track, pool)
            .await
            .map_err(|e| SplitCommandError::TransactionError(e.to_string()))?;

        split_file_ids.push(split_file_id);
    }

    Ok(SplitResult { split_file_ids, tracks_split: split_tracks.len(), output_dir })
}

use crate::AppState;
use tauri::State;

/// Tauri command: Split a multi-track MIDI file into individual tracks.
#[tauri::command]
pub async fn split_file(
    state: State<'_, AppState>,
    file_id: i64,
    output_dir: String,
) -> Result<SplitResult, String> {
    let pool = state.database.pool().await;
    let output_path = PathBuf::from(output_dir);

    split_and_import(file_id, output_path, &pool).await.map_err(|e| e.to_string())
}

/// Tauri command: Split multiple MIDI files into individual tracks (batch).
#[tauri::command]
pub async fn split_file_batch(
    state: State<'_, AppState>,
    file_ids: Vec<i64>,
    output_dir: String,
) -> Result<Vec<Result<SplitResult, String>>, String> {
    let pool = state.database.pool().await;
    let output_path = PathBuf::from(output_dir);

    let mut results = Vec::with_capacity(file_ids.len());

    for file_id in file_ids {
        let result = split_and_import(file_id, output_path.clone(), &pool)
            .await
            .map_err(|e| e.to_string());
        results.push(result);
    }

    Ok(results)
}

/// Import a split track to the database with hash, metadata, BPM, and key detection.
async fn import_split_track(
    filepath: &Path,
    filename: &str,
    file_data: &[u8],
    pool: &sqlx::PgPool,
) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
    // Calculate hash for deduplication (BLAKE3)
    let hash_bytes = calculate_file_hash(filepath)?;
    let content_hash: Vec<u8> = hash_bytes.to_vec();

    // Parse MIDI for metadata
    let midi_data = parse_midi_file(file_data)?;

    let bpm_result = detect_bpm(&midi_data);
    let bpm = (bpm_result.confidence > CONFIDENCE_THRESHOLD).then_some(bpm_result.bpm);

    let key_result = detect_key(&midi_data);
    let key_signature =
        (key_result.confidence > CONFIDENCE_THRESHOLD).then(|| key_result.key.clone());

    // Get file size
    let file_size_bytes = file_data.len() as i64;
    let filepath_str = filepath.to_str().ok_or("Invalid file path")?;

    // Begin transaction
    let mut tx = pool.begin().await?;

    // Insert file record
    let file_id = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO files (
            filename,
            original_filename,
            filepath,
            content_hash,
            file_size_bytes,
            num_tracks,
            created_at
        ) VALUES ($1, $2, $3, $4, $5, 1, NOW())
        ON CONFLICT (content_hash) DO NOTHING
        RETURNING id
        "#,
    )
    .bind(filename)
    .bind(filename) // Original filename is same as filename for splits
    .bind(filepath_str)
    .bind(&content_hash)
    .bind(file_size_bytes)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or("File already exists (duplicate hash)")?;

    // Insert musical metadata if available
    if bpm.is_some() || key_signature.is_some() {
        sqlx::query(
            r#"
            INSERT INTO musical_metadata (
                file_id,
                bpm,
                key_signature,
                time_signature_numerator,
                time_signature_denominator
            ) VALUES ($1, $2, $3::musical_key, 4, 4)
            ON CONFLICT (file_id) DO UPDATE SET
                bpm = EXCLUDED.bpm,
                key_signature = EXCLUDED.key_signature
            "#,
        )
        .bind(file_id)
        .bind(bpm)
        .bind(key_signature.as_deref())
        .execute(&mut *tx)
        .await?;
    }

    // Commit transaction
    tx.commit().await?;

    Ok(file_id)
}

/// Insert parent-child relationship into track_splits table.
async fn insert_track_split_relationship(
    parent_file_id: i64,
    split_file_id: i64,
    split_track: &SplitTrack,
    pool: &sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query!(
        r#"
        INSERT INTO track_splits (
            parent_file_id,
            split_file_id,
            track_number,
            track_name,
            instrument,
            note_count,
            created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, NOW())
        "#,
        parent_file_id,
        split_file_id,
        split_track.track_number as i32,
        split_track.track_name.as_deref(),
        split_track.instrument.as_deref(),
        split_track.note_count as i32,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Generate filename for a split track: `{base}_track_{num:02}_{instrument}.mid`
pub fn generate_split_filename(base_filename: &str, split_track: &SplitTrack) -> String {
    let base = sanitize_filename(base_filename);
    let track_num = format!("{:02}", split_track.track_number);

    // Build suffix: prefer instrument, fall back to track name, then just number
    let suffix = if let Some(ref instrument) = split_track.instrument {
        sanitize_filename(instrument)
    } else if let Some(ref track_name) = split_track.track_name {
        sanitize_filename(track_name)
    } else {
        String::new()
    };

    if suffix.is_empty() {
        format!("{}_track_{}.mid", base, track_num)
    } else {
        format!("{}_track_{}_{}.mid", base, track_num, suffix)
    }
}

/// Sanitize string for use in filenames (replaces unsafe chars, collapses underscores).
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            // Replace spaces with underscores
            ' ' => '_',
            // Remove problematic characters
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            // Keep alphanumeric, underscore, hyphen, period, parentheses
            c if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '(' || c == ')' => c,
            // Replace everything else with underscore
            _ => '_',
        })
        .collect::<String>()
        // Collapse multiple underscores
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

/// Extract time signature from MIDI events (e.g., "4-4" for 4/4 time).
fn extract_time_signature_from_midi(
    midi: &midi_library_shared::core::midi::types::MidiFile,
) -> Option<String> {
    use midi_library_shared::core::midi::types::Event;

    // Search all tracks for TimeSignature event
    for track in &midi.tracks {
        for timed_event in &track.events {
            if let Event::TimeSignature { numerator, denominator, .. } = &timed_event.event {
                // Convert denominator from power-of-2 format (e.g., 2 = quarter note = 4)
                let denom_value = 2_u8.pow(*denominator as u32);
                return Some(format!("{}-{}", numerator, denom_value));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename_spaces() {
        assert_eq!(sanitize_filename("Piano Track"), "Piano_Track");
        assert_eq!(sanitize_filename("My Song Name"), "My_Song_Name");
    }

    #[test]
    fn test_sanitize_filename_special_chars() {
        assert_eq!(sanitize_filename("Track: 1"), "Track_1");
        assert_eq!(sanitize_filename("Bass/Guitar"), "Bass_Guitar");
        assert_eq!(sanitize_filename("Lead (Synth)"), "Lead_(Synth)");
        assert_eq!(sanitize_filename("File*Name?"), "File_Name");
        assert_eq!(sanitize_filename("Path\\To\\File"), "Path_To_File");
    }

    #[test]
    fn test_sanitize_filename_multiple_underscores() {
        assert_eq!(sanitize_filename("Track___1"), "Track_1");
        assert_eq!(sanitize_filename("__Piano__"), "Piano");
        assert_eq!(sanitize_filename("A___B___C"), "A_B_C");
    }

    #[test]
    fn test_sanitize_filename_edge_cases() {
        assert_eq!(sanitize_filename(""), "");
        assert_eq!(sanitize_filename("   "), "");
        assert_eq!(sanitize_filename("___"), "");
        assert_eq!(sanitize_filename("ValidName123"), "ValidName123");
    }

    #[test]
    fn test_sanitize_filename_unicode() {
        // Keep alphanumeric Unicode (includes accented characters)
        assert_eq!(sanitize_filename("Café"), "Café");
        assert_eq!(sanitize_filename("Track♪1"), "Track_1");
    }

    #[test]
    fn test_generate_split_filename_with_instrument() {
        let track = SplitTrack {
            track_number: 1,
            track_name: Some("Piano Part".to_string()),
            channel: Some(0),
            instrument: Some("Acoustic Grand Piano".to_string()),
            note_count: 100,
            midi_bytes: vec![],
        };

        let filename = generate_split_filename("my_song", &track);
        assert_eq!(filename, "my_song_track_01_Acoustic_Grand_Piano.mid");
    }

    #[test]
    fn test_generate_split_filename_with_track_name_only() {
        let track = SplitTrack {
            track_number: 2,
            track_name: Some("Bass Line".to_string()),
            channel: Some(1),
            instrument: None,
            note_count: 50,
            midi_bytes: vec![],
        };

        let filename = generate_split_filename("song", &track);
        assert_eq!(filename, "song_track_02_Bass_Line.mid");
    }

    #[test]
    fn test_generate_split_filename_no_metadata() {
        let track = SplitTrack {
            track_number: 0,
            track_name: None,
            channel: None,
            instrument: None,
            note_count: 10,
            midi_bytes: vec![],
        };

        let filename = generate_split_filename("minimal", &track);
        assert_eq!(filename, "minimal_track_00.mid");
    }

    #[test]
    fn test_generate_split_filename_sanitizes_base() {
        let track = SplitTrack {
            track_number: 5,
            track_name: None,
            channel: None,
            instrument: Some("Guitar".to_string()),
            note_count: 75,
            midi_bytes: vec![],
        };

        let filename = generate_split_filename("My/Bad\\Filename:1", &track);
        assert_eq!(filename, "My_Bad_Filename_1_track_05_Guitar.mid");
    }

    #[test]
    fn test_generate_split_filename_sanitizes_instrument() {
        let track = SplitTrack {
            track_number: 3,
            track_name: None,
            channel: None,
            instrument: Some("Electric Piano (DX7)".to_string()),
            note_count: 80,
            midi_bytes: vec![],
        };

        let filename = generate_split_filename("track", &track);
        assert_eq!(filename, "track_track_03_Electric_Piano_(DX7).mid");
    }

    #[test]
    fn test_generate_split_filename_high_track_numbers() {
        let track = SplitTrack {
            track_number: 99,
            track_name: None,
            channel: None,
            instrument: Some("Drums".to_string()),
            note_count: 200,
            midi_bytes: vec![],
        };

        let filename = generate_split_filename("orchestra", &track);
        assert_eq!(filename, "orchestra_track_99_Drums.mid");
    }
}

//! Core track splitting logic

use std::path::{Path, PathBuf};

use crate::core::naming::generator::generate_production_layer_filename;
use crate::core::splitting::{split_tracks_with_repair, RepairResult, SplitError};
use midi_library_shared::core::midi::parser::parse_midi_file;

use super::db::{import_split_track, insert_track_split_relationship};
use super::helpers::extract_time_signature_from_midi;
use super::types::{SplitCommandError, SplitResult};

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
        return Err(SplitCommandError::FileNotFoundOnDisk(parent_file.filepath.clone()));
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
        }
        RepairResult::Repaired { fix_description, .. } => {
            eprintln!("🔧 REPAIRED: {} - {}", parent_file.filename, fix_description);
        }
        RepairResult::Corrupt { reason } => {
            eprintln!("❌ CORRUPT: {} - {}", parent_file.filename, reason);
        }
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
    let category =
        sqlx::query_scalar::<_, Option<String>>(
            "SELECT category FROM musical_metadata WHERE file_id = $1",
        )
        .bind(parent_file.id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten() // Flattens Option<Option<Option<String>>> to Option<Option<String>>
        .flatten() // Flattens Option<Option<String>> to Option<String>
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
        let split_file_id = import_split_track(&split_path, &filename, &split_track.midi_bytes, pool)
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

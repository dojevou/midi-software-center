//! Database operations for track splitting

use std::path::Path;

use crate::core::analysis::bpm_detector::detect_bpm;
use crate::core::analysis::key_detector::detect_key;
use crate::core::hash::calculate_file_hash;
use crate::core::splitting::SplitTrack;
use midi_library_shared::core::midi::parser::parse_midi_file;

use super::types::CONFIDENCE_THRESHOLD;

/// Import a split track to the database with hash, metadata, BPM, and key detection.
pub async fn import_split_track(
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
pub async fn insert_track_split_relationship(
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

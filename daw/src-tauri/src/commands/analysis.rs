/// Analysis Tauri commands
///
/// Grown-up Script: I/O wrapper for musical analysis and compatibility matching.
/// Updated to use proper JOINs with actual database schema.
use crate::commands::AppState;
use crate::core::compatibility;
use crate::models::analysis::CompatibleFile;
use crate::models::midi_file::MidiFile;
use tauri::State;
use tracing::{debug, error};

/// Find files that are musically compatible with a given file
///
/// Returns files sorted by compatibility score (highest first).
/// Considers key signature, BPM, and time signature.
///
/// # Arguments
/// * `file_id` - The reference file to find compatible files for
/// * `max_results` - Maximum number of results to return (default: 20, max: 100)
#[tauri::command]
pub async fn find_compatible_files(
    file_id: i32,
    max_results: Option<i32>,
    state: State<'_, AppState>,
) -> Result<Vec<CompatibleFile>, String> {
    debug!("Finding compatible files for file ID: {}", file_id);

    let max = max_results.unwrap_or(20).min(100);

    // Get reference file with proper JOINs
    let ref_file = sqlx::query_as!(
        MidiFile,
        r#"
        SELECT
            f.id,
            f.filename,
            f.filepath,
            f.file_size_bytes,
            f.content_hash as "content_hash!",
            f.is_multi_track as "is_multi_track!",
            f.parent_file_id as "parent_file_id?",
            f.track_number as "track_number?",
            f.total_tracks as "total_tracks?",
            f.manufacturer as "manufacturer?",
            f.collection_name as "collection_name?",
            COALESCE(f.folder_tags, ARRAY[]::TEXT[]) as "folder_tags!",
            f.parent_folder as "parent_folder?",
            f.num_tracks,
            f.created_at as "created_at!",
            f.analyzed_at as "analyzed_at?",
            mm.bpm::FLOAT8 as "bpm?",
            mm.key_signature::TEXT as "key_signature?",
            CASE
                WHEN mm.time_signature_numerator IS NOT NULL
                THEN mm.time_signature_numerator::TEXT || '/' || mm.time_signature_denominator::TEXT
                ELSE NULL
            END as "time_signature?",
            f.duration_seconds::FLOAT8 as "duration_seconds?",
            COALESCE(mm.total_notes, 0) as "total_notes!",
            fc.primary_category::TEXT as "primary_category?"
        FROM files f
        LEFT JOIN musical_metadata mm ON f.id = mm.file_id
        LEFT JOIN file_categories fc ON f.id = fc.file_id
        WHERE f.id = $1
        "#,
        file_id as i64
    )
    .fetch_optional(state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?)
    .await
    .map_err(|e| {
        error!("Failed to get reference file: {}", e);
        format!("Failed to get reference file: {}", e)
    })?;

    let ref_file = ref_file.ok_or_else(|| format!("File with ID {} not found", file_id))?;

    debug!(
        "Reference file - BPM: {:?}, Key: {:?}, Time sig: {:?}",
        ref_file.bpm, ref_file.key_signature, ref_file.time_signature
    );

    // Get all other files with proper JOINs
    let candidate_files = sqlx::query_as!(
        MidiFile,
        r#"
        SELECT
            f.id,
            f.filename,
            f.filepath,
            f.file_size_bytes,
            f.content_hash as "content_hash!",
            f.is_multi_track as "is_multi_track!",
            f.parent_file_id as "parent_file_id?",
            f.track_number as "track_number?",
            f.total_tracks as "total_tracks?",
            f.manufacturer as "manufacturer?",
            f.collection_name as "collection_name?",
            COALESCE(f.folder_tags, ARRAY[]::TEXT[]) as "folder_tags!",
            f.parent_folder as "parent_folder?",
            f.num_tracks,
            f.created_at as "created_at!",
            f.analyzed_at as "analyzed_at?",
            mm.bpm::FLOAT8 as "bpm?",
            mm.key_signature::TEXT as "key_signature?",
            CASE
                WHEN mm.time_signature_numerator IS NOT NULL
                THEN mm.time_signature_numerator::TEXT || '/' || mm.time_signature_denominator::TEXT
                ELSE NULL
            END as "time_signature?",
            f.duration_seconds::FLOAT8 as "duration_seconds?",
            COALESCE(mm.total_notes, 0) as "total_notes!",
            fc.primary_category::TEXT as "primary_category?"
        FROM files f
        LEFT JOIN musical_metadata mm ON f.id = mm.file_id
        LEFT JOIN file_categories fc ON f.id = fc.file_id
        WHERE f.id != $1
        LIMIT 500
        "#,
        file_id as i64
    )
    .fetch_all(state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?)
    .await
    .map_err(|e| {
        error!("Failed to fetch candidate files: {}", e);
        format!("Failed to fetch files: {}", e)
    })?;

    let mut compatible_files: Vec<CompatibleFile> = candidate_files
        .iter()
        .map(|candidate| {
            // Use Trusty Module to calculate compatibility (pure function)
            let score = compatibility::calculate_compatibility(&ref_file, candidate);

            // Calculate BPM-based time stretch suggestion
            let suggested_bpm_multiplier =
                if let (Some(ref_bpm), Some(cand_bpm)) = (ref_file.bpm, candidate.bpm) {
                    let ratio = cand_bpm / ref_bpm;
                    // Suggest multiplier if it's a simple ratio
                    if (ratio - 0.5).abs() < 0.05 {
                        Some(0.5)
                    } else if (ratio - 2.0).abs() < 0.1 {
                        Some(2.0)
                    } else if (ratio - 1.5).abs() < 0.1 {
                        Some(1.5)
                    } else if (ratio - 0.75).abs() < 0.05 {
                        Some(0.75)
                    } else {
                        None
                    }
                } else {
                    None
                };

            CompatibleFile {
                id: candidate.id as i32,
                file_name: candidate.filename.clone(),
                compatibility_score: score.total_score as i32, // Convert f32 to i32
                key_match: ref_file.key_signature == candidate.key_signature,
                bpm_difference: if let (Some(ref_bpm), Some(cand_bpm)) =
                    (ref_file.bpm, candidate.bpm)
                {
                    Some((ref_bpm - cand_bpm).abs() as f32)
                } else {
                    None
                },
                time_signature_match: ref_file.time_signature == candidate.time_signature,
                suggested_bpm_multiplier,
                category: candidate.primary_category.clone(),
            }
        })
        .collect();

    // Sort by compatibility score (descending)
    compatible_files.sort_by(|a, b| b.compatibility_score.cmp(&a.compatibility_score));

    // Take top N results
    compatible_files.truncate(max as usize);

    debug!(
        "Returning {} compatible files (top score: {})",
        compatible_files.len(),
        compatible_files.first().map(|f| f.compatibility_score).unwrap_or(0)
    );

    Ok(compatible_files)
}

/// Add file to favorites
#[tauri::command]
pub async fn add_favorite(file_id: i32, state: State<'_, AppState>) -> Result<(), String> {
    debug!("Adding file {} to favorites", file_id);

    // Insert into favorites table (ON CONFLICT DO NOTHING to handle duplicates)
    sqlx::query!(
        "INSERT INTO favorites (file_id) VALUES ($1) ON CONFLICT (file_id) DO NOTHING",
        file_id as i64
    )
    .execute(state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?)
    .await
    .map_err(|e| {
        error!("Failed to add favorite: {}", e);
        format!("Failed to add favorite: {}", e)
    })?;

    debug!("Successfully added file {} to favorites", file_id);
    Ok(())
}

/// Remove file from favorites
#[tauri::command]
pub async fn remove_favorite(file_id: i32, state: State<'_, AppState>) -> Result<(), String> {
    debug!("Removing file {} from favorites", file_id);

    sqlx::query!("DELETE FROM favorites WHERE file_id = $1", file_id as i64)
        .execute(state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?)
        .await
        .map_err(|e| {
            error!("Failed to remove favorite: {}", e);
            format!("Failed to remove favorite: {}", e)
        })?;

    debug!("Successfully removed file {} from favorites", file_id);
    Ok(())
}

/// Check if a file is favorited
#[tauri::command]
pub async fn is_favorite(file_id: i32, state: State<'_, AppState>) -> Result<bool, String> {
    let result = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM favorites WHERE file_id = $1) as is_fav",
        file_id as i64
    )
    .fetch_one(state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?)
    .await
    .map_err(|e| format!("Failed to check favorite status: {}", e))?;

    Ok(result.is_fav.unwrap_or(false))
}

/// Get all favorite files with full details
#[tauri::command]
pub async fn get_favorites(
    state: State<'_, AppState>,
) -> Result<Vec<crate::models::midi_file::FileDetails>, String> {
    debug!("Getting all favorite files");

    let favorites = sqlx::query_as!(
        crate::models::midi_file::FileDetails,
        r#"
        SELECT
            f.id as "id!",
            f.filename as "filename!",
            f.filepath as "filepath!",
            f.file_size_bytes as "file_size_bytes!",
            f.num_tracks as "track_count!",
            f.content_hash as "content_hash!",
            f.parent_folder as "parent_folder?",
            f.created_at as "created_at!",
            mm.bpm::FLOAT8 as "bpm?",
            mm.key_signature::TEXT as "key_signature?",
            CASE
                WHEN mm.time_signature_numerator IS NOT NULL
                THEN mm.time_signature_numerator::TEXT || '/' || mm.time_signature_denominator::TEXT
                ELSE NULL
            END as "time_signature?",
            f.duration_seconds::FLOAT8 as "duration_seconds?",
            COALESCE(mm.total_notes, 0) > 0 as "has_notes!",
            mm.total_notes as "total_notes?",
            mm.is_percussive as "has_drums?",
            fc.primary_category::TEXT as "primary_category?",
            f.manufacturer as "manufacturer?",
            f.collection_name as "collection_name?",
            COALESCE(f.folder_tags, ARRAY[]::TEXT[]) as "tags!",
            true as "is_favorite!"
        FROM favorites fav
        INNER JOIN files f ON fav.file_id = f.id
        LEFT JOIN musical_metadata mm ON f.id = mm.file_id
        LEFT JOIN file_categories fc ON f.id = fc.file_id
        ORDER BY fav.created_at DESC
        "#
    )
    .fetch_all(state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?)
    .await
    .map_err(|e| {
        error!("Failed to fetch favorites: {}", e);
        format!("Failed to fetch favorites: {}", e)
    })?;

    debug!("Retrieved {} favorite files", favorites.len());
    Ok(favorites)
}

/// Get usage statistics
#[tauri::command]
pub async fn get_usage_stats(state: State<'_, AppState>) -> Result<String, String> {
    debug!("Getting usage statistics");

    // Gather various statistics using proper table and column names
    let total_files: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files")
        .fetch_one(state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?)
        .await
        .map_err(|e| format!("Failed to count files: {}", e))?;

    let total_duration: Option<f64> = sqlx::query_scalar(
        "SELECT SUM(duration_seconds) FROM files WHERE duration_seconds IS NOT NULL",
    )
    .fetch_one(state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?)
    .await
    .map_err(|e| format!("Failed to sum duration: {}", e))?;

    let total_notes: Option<i64> =
        sqlx::query_scalar("SELECT SUM(total_notes) FROM musical_metadata")
            .fetch_one(
                state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?,
            )
            .await
            .map_err(|e| format!("Failed to sum notes: {}", e))?;

    let avg_bpm: Option<f64> =
        sqlx::query_scalar("SELECT AVG(bpm) FROM musical_metadata WHERE bpm IS NOT NULL")
            .fetch_one(
                state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?,
            )
            .await
            .map_err(|e| format!("Failed to calculate average BPM: {}", e))?;

    // Most common key
    let most_common_key: Option<(String,)> = sqlx::query_as(
        "SELECT key_signature::TEXT
         FROM musical_metadata
         WHERE key_signature IS NOT NULL
         GROUP BY key_signature
         ORDER BY COUNT(*) DESC
         LIMIT 1",
    )
    .fetch_optional(state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?)
    .await
    .map_err(|e| format!("Failed to find most common key: {}", e))?;

    // Most common time signature
    let most_common_time: Option<(String,)> = sqlx::query_as(
        "SELECT time_signature_numerator::TEXT || '/' || time_signature_denominator::TEXT
         FROM musical_metadata
         WHERE time_signature_numerator IS NOT NULL
           AND time_signature_denominator IS NOT NULL
         GROUP BY time_signature_numerator, time_signature_denominator
         ORDER BY COUNT(*) DESC
         LIMIT 1",
    )
    .fetch_optional(state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?)
    .await
    .map_err(|e| format!("Failed to find most common time signature: {}", e))?;

    // Format as JSON
    let stats = serde_json::json!({
        "total_files": total_files,
        "total_duration_hours": total_duration.unwrap_or(0.0) / 3600.0,
        "total_notes": total_notes.unwrap_or(0),
        "average_bpm": avg_bpm.unwrap_or(0.0),
        "most_common_key": most_common_key.map(|(k,)| k),
        "most_common_time_signature": most_common_time.map(|(t,)| t),
    });

    Ok(stats.to_string())
}

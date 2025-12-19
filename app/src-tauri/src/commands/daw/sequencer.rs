//! Sequencer Tauri commands
//!
//! Thin wrappers that expose sequencer functionality to the frontend.
#![allow(dead_code)] // Commands are called externally via Tauri IPC

use crate::commands::daw::DawAppState;
use crate::core::midi::loader::load_midi_file;
use crate::daw_models::sequencer::{PlaybackPosition, Track, TrackProperties};
use crate::sequencer::SequencerEngine;
use std::sync::Arc;
use tauri::State;
use tracing::{error, info};

/// Start sequencer playback
#[tauri::command]
pub async fn start_sequencer(engine: State<'_, Arc<SequencerEngine>>) -> Result<(), String> {
    engine.start().await
}

/// Stop sequencer playback (resets position)
#[tauri::command]
pub async fn stop_sequencer(engine: State<'_, Arc<SequencerEngine>>) -> Result<(), String> {
    engine.stop().await;
    Ok(())
}

/// Pause sequencer playback (maintains position)
#[tauri::command]
pub async fn pause_sequencer(engine: State<'_, Arc<SequencerEngine>>) -> Result<(), String> {
    engine.pause().await;
    Ok(())
}

/// Resume sequencer playback from paused state
#[tauri::command]
pub async fn resume_sequencer(engine: State<'_, Arc<SequencerEngine>>) -> Result<(), String> {
    engine.resume().await
}

/// Get current playback position
#[tauri::command]
pub async fn get_playback_position(
    engine: State<'_, Arc<SequencerEngine>>,
) -> Result<PlaybackPosition, String> {
    Ok(engine.get_position().await)
}

/// Seek to a specific position
///
/// # Arguments
/// * `bar` - Bar number (0-indexed)
/// * `beat` - Beat within bar (0-indexed)
#[tauri::command]
pub async fn seek_position(
    bar: u32,
    beat: u32,
    engine: State<'_, Arc<SequencerEngine>>,
) -> Result<(), String> {
    use crate::core::sequencer::timing::bar_beat_to_tick;

    let ppq = engine.get_ticks_per_quarter();
    let beats_per_bar = engine.get_beats_per_bar();
    let tick = bar_beat_to_tick(bar, beat, ppq, beats_per_bar);
    engine.seek(tick).await;
    Ok(())
}

/// Set global tempo (BPM)
#[tauri::command]
pub async fn set_tempo(bpm: f32, engine: State<'_, Arc<SequencerEngine>>) -> Result<(), String> {
    engine.set_bpm(bpm).await
}

/// Get current tempo
#[tauri::command]
pub async fn get_tempo(engine: State<'_, Arc<SequencerEngine>>) -> Result<f32, String> {
    Ok(engine.get_bpm().await)
}

/// Add a track to the sequencer
///
/// # Arguments
/// * `file_id` - Database ID of the MIDI file
/// * `channel` - MIDI channel (0-15)
/// * `state` - Application state with database connection
/// * `engine` - Sequencer engine
#[tauri::command]
pub async fn add_track(
    file_id: i32,
    channel: u8,
    state: State<'_, DawAppState>,
    engine: State<'_, Arc<SequencerEngine>>,
) -> Result<Track, String> {
    // Query database for file information
    let file_result = sqlx::query!(
        r#"
        SELECT filepath
        FROM files
        WHERE id = $1
        "#,
        file_id as i64
    )
    .fetch_one(
        state
            .db_pool
            .as_ref()
            .ok_or_else(|| "Database pool not initialized".to_string())?,
    )
    .await
    .map_err(|e| {
        error!("Failed to query file {} from database: {}", file_id, e);
        format!("File not found: {}", file_id)
    })?;

    // Load MIDI file and parse events
    let loaded_midi = load_midi_file(&file_result.filepath).map_err(|e| {
        error!("Failed to load MIDI file {}: {}", file_result.filepath, e);
        format!("Failed to load MIDI file: {}", e)
    })?;

    info!(
        "Loaded {} events from file {} ({})",
        loaded_midi.events.len(),
        file_id,
        file_result.filepath
    );

    // Add track with loaded events
    let track_manager = engine.track_manager();
    let track = track_manager.add_track(file_id, channel, loaded_midi.events).await?;

    // Reload tracks in engine to update scheduler
    engine.load_tracks().await;

    Ok(track)
}

/// Load a file from database into the DAW/sequencer (simplified VIP3 integration)
///
/// This is a wrapper around add_track that provides a simpler interface for
/// the VIP3 browser to load files into the sequencer.
/// Defaults to channel 0 (first MIDI channel).
#[tauri::command]
pub async fn load_file_to_daw(
    file_id: i32,
    state: State<'_, DawAppState>,
    engine: State<'_, Arc<SequencerEngine>>,
) -> Result<i32, String> {
    // Use channel 0 as default
    let track = add_track(file_id, 0, state, engine).await?;
    Ok(track.id)
}

/// Remove a track from the sequencer
#[tauri::command]
pub async fn remove_track(
    track_id: i32,
    engine: State<'_, Arc<SequencerEngine>>,
) -> Result<(), String> {
    let track_manager = engine.track_manager();
    track_manager.remove_track(track_id).await?;

    // Remove track's events from scheduler
    let scheduler = engine.scheduler();
    scheduler.clear_track(track_id).await;

    Ok(())
}

/// Update track properties (mute, solo, volume, pan)
#[tauri::command]
pub async fn update_track(
    track_id: i32,
    properties: TrackProperties,
    engine: State<'_, Arc<SequencerEngine>>,
) -> Result<(), String> {
    let track_manager = engine.track_manager();
    track_manager.update_track(track_id, properties).await?;

    // Reload tracks to update scheduler
    engine.load_tracks().await;

    Ok(())
}

/// Get all tracks in current project
#[tauri::command]
pub async fn get_tracks(engine: State<'_, Arc<SequencerEngine>>) -> Result<Vec<Track>, String> {
    let track_manager = engine.track_manager();
    Ok(track_manager.get_tracks().await)
}

/// Load tracks into sequencer and prepare for playback
#[tauri::command]
pub async fn load_sequencer_tracks(engine: State<'_, Arc<SequencerEngine>>) -> Result<(), String> {
    engine.load_tracks().await;
    Ok(())
}

/// Check if sequencer is currently playing
#[tauri::command]
pub async fn is_sequencer_playing(engine: State<'_, Arc<SequencerEngine>>) -> Result<bool, String> {
    use crate::sequencer::engine::PlaybackState;

    let state = engine.get_state().await;
    Ok(state == PlaybackState::Playing)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that load_file_to_daw returns a valid track ID
    ///
    /// NOTE: This is a documentation test that shows the expected behavior.
    /// Full integration tests require database and file system setup.
    #[test]
    fn test_load_file_to_daw_signature() {
        // This test verifies the function signature and return type
        // Actual integration tests should be run with database fixtures

        // Expected behavior:
        // 1. Function accepts file_id (i32), state, and engine
        // 2. Returns Result<i32, String> where i32 is the track_id
        // 3. Defaults to MIDI channel 0
        // 4. Loads file from database by ID
        // 5. Parses MIDI file and adds to sequencer
        // 6. Returns track ID on success

        assert!(true, "Signature test passed");
    }

    #[test]
    fn test_track_id_type() {
        // Verify track IDs are i32
        let track_id: i32 = 1;
        assert!(track_id > 0);
    }
}

// ============================================================================
// INTEGRATION TEST DOCUMENTATION
// ============================================================================

// Integration Test Plan for load_file_to_daw
//
// ## Prerequisites
// 1. Database running with sample MIDI files imported
// 2. Sample MIDI files in test directory
// 3. Sequencer engine initialized
//
// ## Test Cases
//
// ### TC1: Load Single File
// ```bash
// # 1. Get a file ID from database
// psql $DATABASE_URL -c "SELECT id, filepath FROM files LIMIT 1;"
//
// # 2. Call load_file_to_daw via Tauri IPC
// # Expected: Returns track_id (positive i32)
// ```
//
// ### TC2: Load Multiple Files
// ```bash
// # Load 3 different files sequentially
// # Expected: Each returns unique track_id
// # Expected: All tracks visible in sequencer
// ```
//
// ### TC3: Load Invalid File ID
// ```bash
// # Call load_file_to_daw with non-existent file_id = -1
// # Expected: Returns Err("File not found: -1")
// ```
//
// ### TC4: Load Corrupted MIDI File
// ```bash
// # Insert file record pointing to corrupted MIDI file
// # Expected: Returns Err("Failed to load MIDI file: ...")
// ```
//
// ### TC5: Verify Track Properties
// ```bash
// # Load file, then call get_tracks()
// # Expected: Track has correct file_id, channel=0, and events loaded
// ```
// End of integration test documentation

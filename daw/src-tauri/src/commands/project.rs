/// Project and track loading commands
///
/// Commands for loading multiple tracks into the sequencer from the database.
use crate::commands::AppState;
use crate::core::midi::loader::load_midi_file;
use crate::models::sequencer::Track;
use crate::sequencer::{ScheduledEvent, SequencerEngine};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tracing::{error, info, warn};

/// Track with loaded events ready for scheduling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackWithEvents {
    pub track: Track,
    pub events: Vec<ScheduledEvent>,
}

/// Load multiple MIDI files as sequencer tracks
///
/// This command loads multiple files from the database and prepares them
/// as sequencer tracks with their MIDI events ready for playback.
///
/// # Arguments
/// * `file_ids` - List of database file IDs to load
/// * `state` - Application state with database connection
/// * `engine` - Sequencer engine
#[tauri::command]
pub async fn load_multiple_tracks(
    file_ids: Vec<i32>,
    state: State<'_, AppState>,
    engine: State<'_, Arc<SequencerEngine>>,
) -> Result<Vec<Track>, String> {
    info!("Loading {} files as sequencer tracks", file_ids.len());

    let mut loaded_tracks = Vec::new();
    let mut failed_count = 0;

    for (idx, file_id) in file_ids.iter().enumerate() {
        // Query database for file information
        let file_result = match sqlx::query!(
            r#"
            SELECT id, filepath, filename
            FROM files
            WHERE id = $1
            "#,
            *file_id as i64
        )
        .fetch_one(state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?)
        .await
        {
            Ok(result) => result,
            Err(e) => {
                error!("Failed to query file {} from database: {}", file_id, e);
                failed_count += 1;
                continue;
            },
        };

        // Load MIDI file and parse events
        let loaded_midi = match load_midi_file(&file_result.filepath) {
            Ok(midi) => midi,
            Err(e) => {
                error!(
                    "Failed to load MIDI file {} ({}): {}",
                    file_result.filename, file_result.filepath, e
                );
                failed_count += 1;
                continue;
            },
        };

        info!(
            "Loaded {} events from {} ({}/{})",
            loaded_midi.events.len(),
            file_result.filename,
            idx + 1,
            file_ids.len()
        );

        // Add track with loaded events
        let track_manager = engine.track_manager();
        let channel = (idx % 16) as u8; // Distribute across MIDI channels

        match track_manager
            .add_track(file_result.id as i32, channel, loaded_midi.events)
            .await
        {
            Ok(track) => loaded_tracks.push(track),
            Err(e) => {
                error!("Failed to add track for file {}: {}", file_id, e);
                failed_count += 1;
            },
        }
    }

    if failed_count > 0 {
        warn!(
            "Failed to load {} out of {} tracks",
            failed_count,
            file_ids.len()
        );
    }

    // Reload tracks in engine to update scheduler
    engine.load_tracks().await;

    info!(
        "Successfully loaded {} tracks into sequencer",
        loaded_tracks.len()
    );

    Ok(loaded_tracks)
}

/// Clear all tracks from the sequencer
#[tauri::command]
pub async fn clear_all_tracks(engine: State<'_, Arc<SequencerEngine>>) -> Result<(), String> {
    info!("Clearing all tracks from sequencer");

    let track_manager = engine.track_manager();
    track_manager.clear().await;

    let scheduler = engine.scheduler();
    scheduler.clear().await;

    Ok(())
}

/// Get detailed information about loaded tracks
#[tauri::command]
pub async fn get_track_details(
    engine: State<'_, Arc<SequencerEngine>>,
) -> Result<Vec<TrackDetails>, String> {
    let track_manager = engine.track_manager();
    let tracks = track_manager.get_tracks().await;

    let details: Vec<TrackDetails> = tracks
        .into_iter()
        .map(|track| {
            let event_count = track.events.len();
            TrackDetails {
                id: track.id,
                name: track.name,
                file_id: track.file_id,
                channel: track.channel,
                muted: track.muted,
                solo: track.solo,
                volume: track.volume,
                pan: track.pan,
                event_count,
            }
        })
        .collect();

    Ok(details)
}

/// Track details for frontend display
#[derive(Debug, Serialize, Deserialize)]
pub struct TrackDetails {
    pub id: i32,
    pub name: String,
    pub file_id: i32,
    pub channel: u8,
    pub muted: bool,
    pub solo: bool,
    pub volume: u8,
    pub pan: u8,
    pub event_count: usize,
}

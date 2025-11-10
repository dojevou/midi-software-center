pub mod analysis;
/// Tauri command handlers
///
/// Grown-up Scripts: Thin wrappers that expose backend functionality to frontend.
/// All commands delegate business logic to Trusty Modules or Grown-up Scripts.
pub mod automation;
pub mod export;
pub mod midi;
pub mod project;
pub mod search;
pub mod sequencer;
pub mod window;

// Re-export commonly used types
pub use automation::AutomationState;

/// Shared application state across all commands
///
/// Contains database pool for read-only access to MIDI file metadata.
#[allow(dead_code)]
pub struct AppState {
    pub db_pool: Option<sqlx::PgPool>,
}

#[tauri::command]
pub async fn initialize_database(state: tauri::State<'_, AppState>) -> Result<(), String> {
    // Get database connection pool
    let pool = state.db_pool.as_ref().ok_or_else(|| {
        "Database connection not available. Please set DATABASE_URL environment variable."
            .to_string()
    })?;

    // Test the connection with a simple query
    sqlx::query("SELECT COUNT(*) FROM files")
        .execute(pool)
        .await
        .map_err(|e| format!("Database connection test failed: {}", e))?;

    Ok(())
}

// Re-export all command functions for easy registration
#[allow(unused_imports)]
pub use midi::{
    midi_connect, midi_disconnect, midi_get_current_device, midi_is_connected, midi_list_devices,
    midi_send_test_note,
};

#[allow(unused_imports)]
pub use sequencer::{
    add_track, get_playback_position, get_tempo, get_tracks, is_sequencer_playing,
    load_sequencer_tracks, pause_sequencer, remove_track, resume_sequencer, seek_position,
    set_tempo, start_sequencer, stop_sequencer, update_track,
};

#[allow(unused_imports)]
pub use search::{get_file_details, get_search_suggestions, search_files};

#[allow(unused_imports)]
pub use analysis::{
    add_favorite, find_compatible_files, get_favorites, get_usage_stats, is_favorite,
    remove_favorite,
};

#[allow(unused_imports)]
pub use export::export_project_midi;

#[allow(unused_imports)]
pub use project::{clear_all_tracks, get_track_details, load_multiple_tracks};

// Window commands
#[allow(unused_imports)]
pub use window::{
    add_window_track, get_all_window_tracks, get_bpm, get_daw_state, get_key_signature,
    get_mixer_state, get_playback_state, get_time_signature, get_track_info, pause_transport,
    play_transport, remove_window_track, reset_daw_state, set_bpm, set_channel_mute,
    set_channel_pan, set_channel_solo, set_channel_volume, set_key_signature,
    set_playback_position, set_time_signature, set_track_muted, set_track_soloed,
    set_track_visible, stop_transport, update_track_label, DAWState,
};

// Automation types (commands not yet implemented)
#[allow(unused_imports)]
pub use crate::automation::{
    AutomationCurve, AutomationLane, AutomationPoint, CurveType, ParameterType,
};

//! Tauri command handlers
//!
//! Grown-up Scripts: Thin wrappers that expose backend functionality to frontend.
//! All commands delegate business logic to Trusty Modules or Grown-up Scripts.

pub mod midi;
pub mod sequencer;
pub mod search;
pub mod analysis;
pub mod export;
pub mod project;
pub mod window;

/// Shared application state across all commands
///
/// Contains database pool for read-only access to MIDI file metadata.
pub struct AppState {
    pub db_pool: Option<sqlx::PgPool>,
}

#[tauri::command]
pub async fn initialize_database(state: tauri::State<'_, AppState>) -> Result<(), String> {
    // Get database connection pool
    let pool = state.db_pool.as_ref().ok_or_else(|| {
        "Database connection not available. Please set DATABASE_URL environment variable.".to_string()
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
    midi_list_devices, midi_connect, midi_disconnect,
    midi_is_connected, midi_get_current_device, midi_send_test_note,
};

#[allow(unused_imports)]
pub use sequencer::{
    start_sequencer, stop_sequencer, pause_sequencer, resume_sequencer,
    get_playback_position, seek_position, set_tempo, get_tempo,
    add_track, remove_track, update_track, get_tracks,
    load_sequencer_tracks, is_sequencer_playing,
};

#[allow(unused_imports)]
pub use search::{
    search_files, get_file_details, get_search_suggestions,
};

#[allow(unused_imports)]
pub use analysis::{
    find_compatible_files, add_favorite, remove_favorite,
    is_favorite, get_favorites, get_usage_stats,
};

#[allow(unused_imports)]
pub use export::{
    export_project_midi,
};

#[allow(unused_imports)]
pub use project::{
    load_multiple_tracks, clear_all_tracks, get_track_details,
};

// Window commands
#[allow(unused_imports)]
pub use window::{
    DAWState,
    play_transport, stop_transport, pause_transport,
    set_playback_position, get_playback_state,
    set_bpm, get_bpm, set_time_signature, get_time_signature,
    set_key_signature, get_key_signature,
    add_window_track, remove_window_track, get_all_window_tracks,
    set_track_visible, set_track_muted, set_track_soloed,
    get_track_info, update_track_label,
    get_mixer_state, set_channel_volume, set_channel_pan,
    set_channel_mute, set_channel_solo,
    get_daw_state, reset_daw_state,
};

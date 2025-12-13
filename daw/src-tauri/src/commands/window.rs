//! Tauri command handlers for DAW windows
//!
//! Grown-up Scripts: Expose DAW window operations to frontend.
//! All commands use proper error handling with Result<T, String>.
#![allow(dead_code)] // Commands are called externally via Tauri IPC

use crate::windows::{
    DAWWindowState, MixerChannel, MixerWindowState, PlaybackPosition, PlaybackState, TrackInfo,
    TransportInfo,
};
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::RwLock;

/// Shared state for DAW windows
///
/// Managed by Tauri and shared across all window commands.
pub struct DAWState {
    /// DAW window state (transport, tracks, playback)
    pub daw: Arc<RwLock<DAWWindowState>>,
    /// Mixer window state (channels, volume, pan)
    pub mixer: Arc<RwLock<MixerWindowState>>,
}

impl DAWState {
    /// Create new DAW state
    pub fn new() -> Self {
        DAWState {
            daw: Arc::new(RwLock::new(DAWWindowState::new())),
            mixer: Arc::new(RwLock::new(MixerWindowState::new())),
        }
    }
}

impl Default for DAWState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Transport Commands
// ============================================================================

/// Start playback
#[tauri::command]
pub async fn play_transport(state: tauri::State<'_, DAWState>) -> Result<(), String> {
    let mut daw = state.daw.write().await;
    daw.playback_state = PlaybackState::Playing;
    Ok(())
}

/// Stop playback (reset position to start)
#[tauri::command]
pub async fn stop_transport(state: tauri::State<'_, DAWState>) -> Result<(), String> {
    let mut daw = state.daw.write().await;
    daw.playback_state = PlaybackState::Stopped;
    daw.transport.position = PlaybackPosition::default();
    Ok(())
}

/// Pause playback (keep current position)
#[tauri::command]
pub async fn pause_transport(state: tauri::State<'_, DAWState>) -> Result<(), String> {
    let mut daw = state.daw.write().await;
    if !daw.playback_state.can_pause() {
        return Err("Cannot pause when not playing".to_string());
    }
    daw.playback_state = PlaybackState::Paused;
    Ok(())
}

/// Set playback position
#[tauri::command]
pub async fn set_playback_position(
    state: tauri::State<'_, DAWState>,
    bar: i32,
    beat: i32,
    tick: i32,
) -> Result<(), String> {
    let mut daw = state.daw.write().await;
    let position = PlaybackPosition::new(bar, beat, tick);

    if !position.is_valid() {
        return Err(format!(
            "Invalid position: bar={}, beat={}, tick={}",
            bar, beat, tick
        ));
    }

    daw.transport.position = position;
    Ok(())
}

/// Get current playback state
#[tauri::command]
pub async fn get_playback_state(
    state: tauri::State<'_, DAWState>,
) -> Result<PlaybackState, String> {
    let daw = state.daw.read().await;
    Ok(daw.playback_state)
}

// ============================================================================
// Tempo and Key Commands
// ============================================================================

/// Set tempo (BPM)
#[tauri::command]
pub async fn set_bpm(state: tauri::State<'_, DAWState>, bpm: f32) -> Result<(), String> {
    if !(20.0..=999.0).contains(&bpm) {
        return Err(format!("BPM {} out of range (20-999)", bpm));
    }

    let mut daw = state.daw.write().await;
    daw.transport.bpm = bpm;
    Ok(())
}

/// Get current tempo
#[tauri::command]
pub async fn get_bpm(state: tauri::State<'_, DAWState>) -> Result<f32, String> {
    let daw = state.daw.read().await;
    Ok(daw.transport.bpm)
}

/// Set time signature
#[tauri::command]
pub async fn set_time_signature(
    state: tauri::State<'_, DAWState>,
    numerator: u8,
    denominator: u8,
) -> Result<(), String> {
    if !(1..=32).contains(&numerator) {
        return Err(format!("Invalid numerator: {}", numerator));
    }

    if !matches!(denominator, 1 | 2 | 4 | 8 | 16 | 32) {
        return Err(format!("Invalid denominator: {}", denominator));
    }

    let mut daw = state.daw.write().await;
    daw.transport.time_signature_numerator = numerator;
    daw.transport.time_signature_denominator = denominator;
    Ok(())
}

/// Get current time signature
#[tauri::command]
pub async fn get_time_signature(state: tauri::State<'_, DAWState>) -> Result<(u8, u8), String> {
    let daw = state.daw.read().await;
    Ok((
        daw.transport.time_signature_numerator,
        daw.transport.time_signature_denominator,
    ))
}

/// Set key signature
#[tauri::command]
pub async fn set_key_signature(
    state: tauri::State<'_, DAWState>,
    key: String,
) -> Result<(), String> {
    // Validate key signature format (basic check)
    if key.is_empty() || key.len() > 3 {
        return Err(format!("Invalid key signature: {}", key));
    }

    let mut daw = state.daw.write().await;
    daw.transport.key_signature = key;
    Ok(())
}

/// Get current key signature
#[tauri::command]
pub async fn get_key_signature(state: tauri::State<'_, DAWState>) -> Result<String, String> {
    let daw = state.daw.read().await;
    Ok(daw.transport.key_signature.clone())
}

// ============================================================================
// Track Commands
// ============================================================================

/// Add a new track to DAW window
#[tauri::command]
pub async fn add_window_track(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, DAWState>,
    label: String,
) -> Result<i32, String> {
    if label.is_empty() {
        return Err("Track label cannot be empty".to_string());
    }

    let mut daw = state.daw.write().await;
    let track_id = daw.add_track(label.clone());

    // Sync with mixer
    let track = daw.get_track(track_id).ok_or("Track not found")?;
    let mut mixer = state.mixer.write().await;
    mixer.add_channel_from_track(track);

    // Emit event to notify frontend windows
    let _ = app_handle.emit("track-added", serde_json::json!({ "track_id": track_id }));
    tracing::info!("[add_window_track] Emitted track-added event for track_id={}", track_id);

    Ok(track_id)
}

/// Remove a track from DAW window
#[tauri::command]
pub async fn remove_window_track(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, DAWState>,
    track_id: i32,
) -> Result<(), String> {
    let mut daw = state.daw.write().await;
    daw.remove_track(track_id)
        .ok_or_else(|| format!("Track {} not found", track_id))?;

    // Remove from mixer
    let mut mixer = state.mixer.write().await;
    mixer.remove_channel(track_id);

    // Emit event to notify frontend windows
    let _ = app_handle.emit("track-removed", serde_json::json!({ "track_id": track_id }));
    tracing::info!("[remove_window_track] Emitted track-removed event for track_id={}", track_id);

    Ok(())
}

/// Get all tracks from DAW window
#[tauri::command]
pub async fn get_all_window_tracks(
    state: tauri::State<'_, DAWState>,
) -> Result<Vec<TrackInfo>, String> {
    let daw = state.daw.read().await;
    Ok(daw.get_all_tracks())
}

/// Set track visibility
#[tauri::command]
pub async fn set_track_visible(
    state: tauri::State<'_, DAWState>,
    track_id: i32,
    visible: bool,
) -> Result<(), String> {
    let mut daw = state.daw.write().await;
    let track = daw
        .get_track_mut(track_id)
        .ok_or_else(|| format!("Track {} not found", track_id))?;
    track.visible = visible;
    Ok(())
}

/// Set track muted
#[tauri::command]
pub async fn set_track_muted(
    state: tauri::State<'_, DAWState>,
    track_id: i32,
    muted: bool,
) -> Result<(), String> {
    let mut daw = state.daw.write().await;
    let track = daw
        .get_track_mut(track_id)
        .ok_or_else(|| format!("Track {} not found", track_id))?;
    track.muted = muted;

    // Sync with mixer
    let mut mixer = state.mixer.write().await;
    if let Some(channel) = mixer.get_channel_mut(track_id) {
        channel.muted = muted;
    }

    Ok(())
}

/// Set track soloed
#[tauri::command]
pub async fn set_track_soloed(
    state: tauri::State<'_, DAWState>,
    track_id: i32,
    soloed: bool,
) -> Result<(), String> {
    let mut daw = state.daw.write().await;
    let track = daw
        .get_track_mut(track_id)
        .ok_or_else(|| format!("Track {} not found", track_id))?;
    track.soloed = soloed;

    // Sync with mixer
    let mut mixer = state.mixer.write().await;
    if let Some(channel) = mixer.get_channel_mut(track_id) {
        channel.soloed = soloed;
    }

    Ok(())
}

/// Get track info by ID
#[tauri::command]
pub async fn get_track_info(
    state: tauri::State<'_, DAWState>,
    track_id: i32,
) -> Result<TrackInfo, String> {
    let daw = state.daw.read().await;
    daw.get_track(track_id)
        .cloned()
        .ok_or_else(|| format!("Track {} not found", track_id))
}

/// Update track label
#[tauri::command]
pub async fn update_track_label(
    state: tauri::State<'_, DAWState>,
    track_id: i32,
    label: String,
) -> Result<(), String> {
    if label.is_empty() {
        return Err("Track label cannot be empty".to_string());
    }

    let mut daw = state.daw.write().await;
    let track = daw
        .get_track_mut(track_id)
        .ok_or_else(|| format!("Track {} not found", track_id))?;
    track.label = label.clone();

    // Sync with mixer
    let mut mixer = state.mixer.write().await;
    if let Some(channel) = mixer.get_channel_mut(track_id) {
        channel.label = label;
    }

    Ok(())
}

// ============================================================================
// Mixer Commands
// ============================================================================

/// Get complete mixer state
#[tauri::command]
pub async fn get_mixer_state(
    state: tauri::State<'_, DAWState>,
) -> Result<MixerWindowState, String> {
    tracing::info!("[get_mixer_state] Called!");
    let mixer = state.mixer.read().await;
    tracing::info!("[get_mixer_state] Returning mixer state with {} channels", mixer.channels.len());
    Ok(mixer.clone())
}

/// Set channel volume
#[tauri::command]
pub async fn set_channel_volume(
    state: tauri::State<'_, DAWState>,
    channel_id: i32,
    volume: f32,
) -> Result<(), String> {
    if !(0.0..=1.0).contains(&volume) {
        return Err(format!("Volume {} out of range (0.0-1.0)", volume));
    }

    let mut mixer = state.mixer.write().await;

    if channel_id == -1 {
        mixer.master.volume = volume;
    } else {
        let channel = mixer
            .get_channel_mut(channel_id)
            .ok_or_else(|| format!("Channel {} not found", channel_id))?;
        channel.volume = volume;
    }

    Ok(())
}

/// Set channel pan
#[tauri::command]
pub async fn set_channel_pan(
    state: tauri::State<'_, DAWState>,
    channel_id: i32,
    pan: f32,
) -> Result<(), String> {
    if !(-1.0..=1.0).contains(&pan) {
        return Err(format!("Pan {} out of range (-1.0 to 1.0)", pan));
    }

    let mut mixer = state.mixer.write().await;

    if channel_id == -1 {
        mixer.master.pan = pan;
    } else {
        let channel = mixer
            .get_channel_mut(channel_id)
            .ok_or_else(|| format!("Channel {} not found", channel_id))?;
        channel.pan = pan;
    }

    Ok(())
}

/// Set channel mute
#[tauri::command]
pub async fn set_channel_mute(
    state: tauri::State<'_, DAWState>,
    channel_id: i32,
    muted: bool,
) -> Result<(), String> {
    let mut mixer = state.mixer.write().await;

    if channel_id == -1 {
        mixer.master.muted = muted;
    } else {
        let channel = mixer
            .get_channel_mut(channel_id)
            .ok_or_else(|| format!("Channel {} not found", channel_id))?;
        channel.muted = muted;
    }

    // Sync with DAW tracks
    if channel_id != -1 {
        let mut daw = state.daw.write().await;
        if let Some(track) = daw.get_track_mut(channel_id) {
            track.muted = muted;
        }
    }

    Ok(())
}

/// Set channel solo
#[tauri::command]
pub async fn set_channel_solo(
    state: tauri::State<'_, DAWState>,
    channel_id: i32,
    soloed: bool,
) -> Result<(), String> {
    let mut mixer = state.mixer.write().await;

    let channel = mixer
        .get_channel_mut(channel_id)
        .ok_or_else(|| format!("Channel {} not found", channel_id))?;
    channel.soloed = soloed;

    // Sync with DAW tracks
    let mut daw = state.daw.write().await;
    if let Some(track) = daw.get_track_mut(channel_id) {
        track.soloed = soloed;
    }

    Ok(())
}

// ============================================================================
// State Commands
// ============================================================================

/// Get complete DAW state
#[tauri::command]
pub async fn get_daw_state(state: tauri::State<'_, DAWState>) -> Result<DAWWindowState, String> {
    let daw = state.daw.read().await;
    Ok(daw.clone())
}

/// Reset DAW state to defaults
#[tauri::command]
pub async fn reset_daw_state(state: tauri::State<'_, DAWState>) -> Result<(), String> {
    let mut daw = state.daw.write().await;
    *daw = DAWWindowState::new();

    let mut mixer = state.mixer.write().await;
    *mixer = MixerWindowState::new();

    Ok(())
}

// ============================================================================
// Extended State Commands - Wiring up all state methods
// ============================================================================

/// Check if transport is currently active (playing or recording)
#[tauri::command]
pub async fn is_transport_active(state: tauri::State<'_, DAWState>) -> Result<bool, String> {
    let daw = state.daw.read().await;
    Ok(daw.playback_state.is_active())
}

/// Check if transport can be resumed from pause
#[tauri::command]
pub async fn can_resume_transport(state: tauri::State<'_, DAWState>) -> Result<bool, String> {
    let daw = state.daw.read().await;
    Ok(daw.playback_state.can_resume())
}

/// Resume playback from paused state
#[tauri::command]
pub async fn resume_transport(state: tauri::State<'_, DAWState>) -> Result<(), String> {
    let mut daw = state.daw.write().await;
    if !daw.playback_state.can_resume() {
        return Err("Cannot resume when not paused".to_string());
    }
    daw.playback_state = PlaybackState::Playing;
    Ok(())
}

/// Get playback position from total ticks
#[tauri::command]
pub async fn get_position_from_ticks(
    state: tauri::State<'_, DAWState>,
    total_ticks: u64,
) -> Result<PlaybackPosition, String> {
    let daw = state.daw.read().await;
    let position = PlaybackPosition::from_ticks(
        total_ticks,
        daw.transport.ticks_per_quarter,
        daw.transport.time_signature_numerator,
    );
    Ok(position)
}

/// Validate current transport settings
#[tauri::command]
pub async fn validate_transport(
    state: tauri::State<'_, DAWState>,
) -> Result<(bool, bool), String> {
    let daw = state.daw.read().await;
    let bpm_valid = daw.transport.is_bpm_valid();
    let time_sig_valid = daw.transport.is_time_signature_valid();
    Ok((bpm_valid, time_sig_valid))
}

/// Create transport with builder pattern and set it
#[tauri::command]
pub async fn configure_transport(
    state: tauri::State<'_, DAWState>,
    bpm: Option<f32>,
    time_sig_num: Option<u8>,
    time_sig_denom: Option<u8>,
    key: Option<String>,
) -> Result<(), String> {
    let mut daw = state.daw.write().await;

    // Use builder pattern methods
    let mut transport = std::mem::take(&mut daw.transport);

    if let Some(b) = bpm {
        transport = transport.with_bpm(b);
    }
    if let (Some(num), Some(denom)) = (time_sig_num, time_sig_denom) {
        transport = transport.with_time_signature(num, denom);
    }
    if let Some(k) = key {
        transport = transport.with_key_signature(k);
    }

    // Validate before applying
    if !transport.is_bpm_valid() {
        return Err(format!("Invalid BPM: {}", transport.bpm));
    }
    if !transport.is_time_signature_valid() {
        return Err(format!(
            "Invalid time signature: {}/{}",
            transport.time_signature_numerator, transport.time_signature_denominator
        ));
    }

    daw.transport = transport;
    Ok(())
}

/// Create track with builder pattern
#[tauri::command]
pub async fn create_configured_track(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, DAWState>,
    label: String,
    midi_channel: Option<u8>,
    color: Option<String>,
) -> Result<i32, String> {
    if label.is_empty() {
        return Err("Track label cannot be empty".to_string());
    }

    let mut daw = state.daw.write().await;
    let track_id = daw.next_track_id;

    // Use builder pattern
    let mut track = TrackInfo::new(track_id, &label);
    if let Some(ch) = midi_channel {
        track = track.with_midi_channel(ch);
    }
    if let Some(c) = color {
        track = track.with_color(c);
    }

    // Validate MIDI channel
    if !track.is_midi_channel_valid() {
        return Err(format!("Invalid MIDI channel: {}", track.midi_channel));
    }

    daw.tracks.insert(track_id, track.clone());
    daw.next_track_id += 1;

    // Sync with mixer
    let mut mixer = state.mixer.write().await;
    mixer.add_channel_from_track(&track);

    let _ = app_handle.emit("track-added", serde_json::json!({ "track_id": track_id }));
    Ok(track_id)
}

/// Check if a track should play based on mute/solo state
#[tauri::command]
pub async fn should_track_play(
    state: tauri::State<'_, DAWState>,
    track_id: i32,
) -> Result<bool, String> {
    let daw = state.daw.read().await;
    let track = daw
        .get_track(track_id)
        .ok_or_else(|| format!("Track {} not found", track_id))?;
    let any_solo = daw.has_soloed_tracks();
    Ok(track.should_play(any_solo))
}

/// Check if any tracks are soloed in DAW
#[tauri::command]
pub async fn has_any_soloed_tracks(state: tauri::State<'_, DAWState>) -> Result<bool, String> {
    let daw = state.daw.read().await;
    Ok(daw.has_soloed_tracks())
}

/// Get total track count
#[tauri::command]
pub async fn get_track_count(state: tauri::State<'_, DAWState>) -> Result<usize, String> {
    let daw = state.daw.read().await;
    Ok(daw.track_count())
}

/// Set mixer channel volume using method
#[tauri::command]
pub async fn set_mixer_channel_volume(
    state: tauri::State<'_, DAWState>,
    channel_id: i32,
    volume: f32,
) -> Result<(), String> {
    let mut mixer = state.mixer.write().await;

    if channel_id == -1 {
        mixer.master.set_volume(volume);
    } else {
        let channel = mixer
            .get_channel_mut(channel_id)
            .ok_or_else(|| format!("Channel {} not found", channel_id))?;
        channel.set_volume(volume);
    }

    Ok(())
}

/// Set mixer channel pan using method
#[tauri::command]
pub async fn set_mixer_channel_pan(
    state: tauri::State<'_, DAWState>,
    channel_id: i32,
    pan: f32,
) -> Result<(), String> {
    let mut mixer = state.mixer.write().await;

    if channel_id == -1 {
        mixer.master.set_pan(pan);
    } else {
        let channel = mixer
            .get_channel_mut(channel_id)
            .ok_or_else(|| format!("Channel {} not found", channel_id))?;
        channel.set_pan(pan);
    }

    Ok(())
}

/// Check if mixer channel is valid
#[tauri::command]
pub async fn is_mixer_channel_valid(
    state: tauri::State<'_, DAWState>,
    channel_id: i32,
) -> Result<bool, String> {
    let mixer = state.mixer.read().await;

    if channel_id == -1 {
        Ok(mixer.master.is_valid())
    } else {
        let channel = mixer
            .get_channel(channel_id)
            .ok_or_else(|| format!("Channel {} not found", channel_id))?;
        Ok(channel.is_valid())
    }
}

/// Get all mixer channels
#[tauri::command]
pub async fn get_all_mixer_channels(
    state: tauri::State<'_, DAWState>,
) -> Result<Vec<MixerChannel>, String> {
    let mixer = state.mixer.read().await;
    Ok(mixer.get_all_channels())
}

/// Sync mixer with current tracks
#[tauri::command]
pub async fn sync_mixer_with_tracks(state: tauri::State<'_, DAWState>) -> Result<(), String> {
    let daw = state.daw.read().await;
    let tracks = daw.tracks.clone();
    drop(daw);

    let mut mixer = state.mixer.write().await;
    mixer.sync_with_tracks(&tracks);

    Ok(())
}

/// Get current transport info
///
/// Returns the full transport state including BPM, time signature, and key
#[tauri::command]
pub async fn get_transport_info(
    state: tauri::State<'_, DAWState>,
) -> Result<TransportInfo, String> {
    let daw = state.daw.read().await;
    Ok(daw.transport.clone())
}

#[cfg(test)]
mod tests {
    // Temporarily disabled - Tauri State mocking needs proper setup
    /*
    fn create_test_state() -> DAWState {
        DAWState::new()
    }

    #[tokio::test]
    async fn test_transport_commands() {
        let state = create_test_state();

        // Start playback
        play_transport(tauri::State::from(&state))
            .await
            .unwrap();
        let playback_state = get_playback_state(tauri::State::from(&state))
            .await
            .unwrap();
        assert_eq!(playback_state, PlaybackState::Playing);

        // Pause
        pause_transport(tauri::State::from(&state))
            .await
            .unwrap();
        let playback_state = get_playback_state(tauri::State::from(&state))
            .await
            .unwrap();
        assert_eq!(playback_state, PlaybackState::Paused);

        // Stop
        stop_transport(tauri::State::from(&state)).await.unwrap();
        let playback_state = get_playback_state(tauri::State::from(&state))
            .await
            .unwrap();
        assert_eq!(playback_state, PlaybackState::Stopped);
    }

    #[tokio::test]
    async fn test_bpm_commands() {
        let state = create_test_state();

        set_bpm(tauri::State::from(&state), 140.0)
            .await
            .unwrap();
        let bpm = get_bpm(tauri::State::from(&state)).await.unwrap();
        assert_eq!(bpm, 140.0);

        // Test invalid BPM
        let result = set_bpm(tauri::State::from(&state), 1000.0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_track_commands() {
        let state = create_test_state();

        // Add track
        let track_id = add_window_track(tauri::State::from(&state), "Piano".to_string())
            .await
            .unwrap();
        assert_eq!(track_id, 1);

        // Get tracks
        let tracks = get_all_window_tracks(tauri::State::from(&state))
            .await
            .unwrap();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].label, "Piano");

        // Set track muted
        set_track_muted(tauri::State::from(&state), track_id, true)
            .await
            .unwrap();
        let track = get_track_info(tauri::State::from(&state), track_id)
            .await
            .unwrap();
        assert!(track.muted);

        // Remove track
        remove_window_track(tauri::State::from(&state), track_id)
            .await
            .unwrap();
        let tracks = get_all_window_tracks(tauri::State::from(&state))
            .await
            .unwrap();
        assert_eq!(tracks.len(), 0);
    }

    #[tokio::test]
    async fn test_mixer_sync() {
        let state = create_test_state();

        // Add track
        let track_id = add_window_track(tauri::State::from(&state), "Piano".to_string())
            .await
            .unwrap();

        // Check mixer has channel
        let mixer_state = get_mixer_state(tauri::State::from(&state))
            .await
            .unwrap();
        assert!(mixer_state.channels.contains_key(&track_id));

        // Set channel volume
        set_channel_volume(tauri::State::from(&state), track_id, 0.5)
            .await
            .unwrap();
        let mixer_state = get_mixer_state(tauri::State::from(&state))
            .await
            .unwrap();
        let channel = mixer_state.channels.get(&track_id).unwrap();
        assert_eq!(channel.volume, 0.5);
    }
    */
}

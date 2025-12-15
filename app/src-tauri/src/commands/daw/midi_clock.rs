use crate::midi_clock::{
    clock::{ClockState, MidiClock, PPQN},
    messages::{FrameRate, MidiTimecode, SongPosition},
    sync::{SyncManager, SyncMode, SyncStatus},
    transport::{TimeSignature, Transport, TransportState},
};
use std::sync::Arc;
use tauri::{command, State};
use tokio::sync::RwLock;

pub struct MidiClockState {
    pub clock: Arc<MidiClock>,
    pub sync_manager: Arc<RwLock<SyncManager>>,
}

#[command]
pub async fn get_transport(state: State<'_, MidiClockState>) -> Result<Transport, String> {
    let clock = &state.clock;
    let (bar, beat, tick) = clock.get_position_bbt().await;
    let position_millis = clock.get_position_millis().await;
    let bpm = clock.get_bpm().await;

    let clock_state = clock.get_state().await;
    let transport_state = match clock_state {
        ClockState::Stopped => TransportState::Stopped,
        ClockState::Playing => TransportState::Playing,
        ClockState::Paused => TransportState::Paused,
    };

    Ok(Transport {
        state: transport_state,
        bpm,
        time_signature: TimeSignature::default(),
        position_ticks: clock.get_tick_count(),
        position_millis,
        bar,
        beat,
        tick,
        ..Transport::default()
    })
}

#[command]
pub async fn transport_play(state: State<'_, MidiClockState>) -> Result<(), String> {
    state.clock.start().await;
    Ok(())
}

#[command]
pub async fn transport_pause(state: State<'_, MidiClockState>) -> Result<(), String> {
    state.clock.pause().await;
    Ok(())
}

#[command]
pub async fn transport_stop(state: State<'_, MidiClockState>) -> Result<(), String> {
    state.clock.stop().await;
    Ok(())
}

#[command]
pub async fn transport_continue(state: State<'_, MidiClockState>) -> Result<(), String> {
    state.clock.continue_playback().await;
    Ok(())
}

#[command]
pub async fn clock_set_bpm(state: State<'_, MidiClockState>, bpm: f64) -> Result<(), String> {
    if !(20.0..=300.0).contains(&bpm) {
        return Err("BPM must be between 20 and 300".to_string());
    }
    state.clock.set_bpm(bpm).await;
    Ok(())
}

#[command]
pub async fn clock_get_bpm(state: State<'_, MidiClockState>) -> Result<f64, String> {
    Ok(state.clock.get_bpm().await)
}

#[command]
pub async fn clock_set_time_signature(
    state: State<'_, MidiClockState>,
    numerator: u8,
    denominator: u8,
) -> Result<(), String> {
    if numerator == 0 || numerator > 16 {
        return Err("Numerator must be between 1 and 16".to_string());
    }
    if !matches!(denominator, 2 | 4 | 8 | 16) {
        return Err("Denominator must be 2, 4, 8, or 16".to_string());
    }

    state.clock.set_time_signature(TimeSignature::new(numerator, denominator)).await;
    Ok(())
}

#[command]
pub async fn set_sync_mode(state: State<'_, MidiClockState>, mode: String) -> Result<(), String> {
    let sync_mode = match mode.to_lowercase().as_str() {
        "internal" => SyncMode::Internal,
        "external" => SyncMode::External,
        "mtc" | "timecode" => SyncMode::MidiTimecode,
        _ => return Err(format!("Unknown sync mode: {}", mode)),
    };

    state.sync_manager.write().await.set_mode(sync_mode).await;
    Ok(())
}

#[command]
pub async fn get_sync_status(state: State<'_, MidiClockState>) -> Result<SyncStatus, String> {
    Ok(state.sync_manager.read().await.get_status().await)
}

#[command]
pub async fn set_position_bars(
    state: State<'_, MidiClockState>,
    bar: u32,
    beat: u32,
) -> Result<(), String> {
    // Convert bar:beat to MIDI beats (16th notes)
    // Assuming 4/4 time signature for simplicity
    let midi_beats = (bar.saturating_sub(1) * 16) + (beat.saturating_sub(1) * 4);

    state.clock.set_position(SongPosition::new(midi_beats as u16)).await;
    Ok(())
}

// =============================================================================
// MTC/SMPTE COMMANDS
// =============================================================================

/// Get current timecode (MTC/SMPTE format)
#[command]
pub async fn clock_get_timecode(
    state: State<'_, MidiClockState>,
    frame_rate: Option<String>,
) -> Result<MidiTimecode, String> {
    let position_millis = state.clock.get_position_millis().await;

    let rate = match frame_rate.as_deref() {
        Some("24") | Some("fps24") => FrameRate::Fps24,
        Some("25") | Some("fps25") => FrameRate::Fps25,
        Some("29.97") | Some("30drop") | Some("fps30drop") => FrameRate::Fps30Drop,
        Some("30") | Some("fps30") | None => FrameRate::Fps30,
        Some(other) => return Err(format!("Unknown frame rate: {}", other)),
    };

    Ok(MidiTimecode::from_millis(position_millis, rate))
}

/// Set current position from timecode
#[command]
pub async fn clock_set_timecode(
    state: State<'_, MidiClockState>,
    hours: u8,
    minutes: u8,
    seconds: u8,
    frames: u8,
    frame_rate: Option<String>,
) -> Result<(), String> {
    let rate = match frame_rate.as_deref() {
        Some("24") | Some("fps24") => FrameRate::Fps24,
        Some("25") | Some("fps25") => FrameRate::Fps25,
        Some("29.97") | Some("30drop") | Some("fps30drop") => FrameRate::Fps30Drop,
        Some("30") | Some("fps30") | None => FrameRate::Fps30,
        Some(other) => return Err(format!("Unknown frame rate: {}", other)),
    };

    let timecode = MidiTimecode { hours, minutes, seconds, frames, frame_rate: rate };

    let millis = timecode.to_millis();
    let bpm = state.clock.get_bpm().await;
    let ppqn = PPQN as f64;

    // Convert millis to ticks: ticks = millis * bpm * ppqn / 60000
    let ticks = (millis as f64 * bpm * ppqn / 60000.0) as u64;

    // Use set_position with SongPosition (16th notes = ticks / ppqn * 4)
    let song_beats = ((ticks as f64 / ppqn) * 4.0) as u16;
    state.clock.set_position(SongPosition::new(song_beats)).await;

    Ok(())
}

/// Get MTC quarter frame bytes for current position
#[command]
pub async fn clock_get_mtc_quarter_frames(
    state: State<'_, MidiClockState>,
    frame_rate: Option<String>,
) -> Result<Vec<u8>, String> {
    let position_millis = state.clock.get_position_millis().await;

    let rate = match frame_rate.as_deref() {
        Some("24") | Some("fps24") => FrameRate::Fps24,
        Some("25") | Some("fps25") => FrameRate::Fps25,
        Some("29.97") | Some("30drop") | Some("fps30drop") => FrameRate::Fps30Drop,
        Some("30") | Some("fps30") | None => FrameRate::Fps30,
        Some(other) => return Err(format!("Unknown frame rate: {}", other)),
    };

    let timecode = MidiTimecode::from_millis(position_millis, rate);
    Ok(timecode.to_quarter_frames().to_vec())
}

/// Get song position pointer (SPP)
#[command]
pub async fn clock_get_song_position(
    state: State<'_, MidiClockState>,
) -> Result<SongPosition, String> {
    let (bar, _beat, _tick) = state.clock.get_position_bbt().await;
    let time_sig = TimeSignature::default(); // 4/4

    Ok(SongPosition::from_bars(bar as u16, time_sig.numerator))
}

/// Set position from song position pointer
#[command]
pub async fn clock_set_song_position(
    state: State<'_, MidiClockState>,
    beats: u16,
) -> Result<(), String> {
    state.clock.set_position(SongPosition::new(beats)).await;
    Ok(())
}

/// Get current sync mode
#[command]
pub async fn clock_get_sync_mode(state: State<'_, MidiClockState>) -> Result<String, String> {
    let mode = state.sync_manager.read().await.get_mode().await;
    let mode_str = match mode {
        SyncMode::Internal => "internal",
        SyncMode::External => "external",
        SyncMode::MidiTimecode => "mtc",
    };
    Ok(mode_str.to_string())
}

/// Get all available frame rates
#[command]
pub fn clock_get_frame_rates() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({"id": "fps24", "name": "24 fps", "fps": 24.0}),
        serde_json::json!({"id": "fps25", "name": "25 fps (PAL)", "fps": 25.0}),
        serde_json::json!({"id": "fps30drop", "name": "29.97 fps (Drop Frame)", "fps": 29.97}),
        serde_json::json!({"id": "fps30", "name": "30 fps (Non-Drop)", "fps": 30.0}),
    ]
}

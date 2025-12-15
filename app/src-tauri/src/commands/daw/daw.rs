#![allow(dead_code)]
//! DAW commands for Tauri frontend

use crate::core::midi::loader::load_midi_from_bytes;
use crate::core::midi::playback_types::MidiMessage as CustomMidiMessage;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use midly::{MidiMessage, Smf, Timing};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{command, Emitter, State, Window};
use tokio::time::interval;
use tracing::info;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Track {
    pub id: u32,
    pub name: String,
    pub instrument: String,
    pub color: String,
    pub volume: f32,
    pub pan: f32,
    pub muted: bool,
    pub solo: bool,
    pub clips: Vec<Clip>,
    pub effects: Vec<TrackEffect>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Clip {
    pub id: u64,
    pub name: String,
    pub start_time: f64,
    pub duration: f64,
    pub file_path: Option<String>,
    pub notes: Vec<MidiNote>,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MidiNote {
    pub pitch: u8,
    pub velocity: u8,
    pub start_time: f64,
    pub duration: f64,
    pub channel: u8,
}

/// Simple effect reference for tracks
/// Named TrackEffect to avoid conflict with effect::Effect
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TrackEffect {
    pub id: u32,
    pub name: String,
    pub enabled: bool,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TransportState {
    pub playing: bool,
    pub recording: bool,
    pub looping: bool,
    pub loop_start: f64,
    pub loop_end: f64,
    pub position: f64,
    pub bpm: f32,
    pub time_signature: (u8, u8),
    pub metronome_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaybackInfo {
    pub position: f64,
    pub bar: u32,
    pub beat: u32,
    pub tick: u32,
    pub bpm: f32,
    pub time_signature: (u8, u8),
    pub total_bars: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TimelineMarker {
    pub position: f64,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct DawState {
    pub transport: Arc<Mutex<TransportState>>,
    pub tracks: Arc<Mutex<Vec<Track>>>,
    pub markers: Arc<Mutex<Vec<TimelineMarker>>>,
    pub current_project: Arc<Mutex<Option<String>>>,
    pub playback_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl Default for DawState {
    fn default() -> Self {
        Self {
            transport: Arc::new(Mutex::new(TransportState {
                playing: false,
                recording: false,
                looping: false,
                loop_start: 0.0,
                loop_end: 120.0,
                position: 0.0,
                bpm: 120.0,
                time_signature: (4, 4),
                metronome_enabled: true,
            })),
            tracks: Arc::new(Mutex::new(Vec::new())),
            markers: Arc::new(Mutex::new(Vec::new())),
            current_project: Arc::new(Mutex::new(None)),
            playback_handle: Arc::new(Mutex::new(None)),
        }
    }
}

#[command]
pub async fn daw_play(state: State<'_, DawState>, window: Window) -> Result<(), String> {
    let mut transport = state.transport.lock().unwrap();
    if transport.playing {
        return Ok(()); // Already playing
    }
    transport.playing = true;

    // Start playback task if not running
    let mut handle_guard = state.playback_handle.lock().unwrap();
    if handle_guard.is_none() {
        let window_clone = window.clone();
        let state_clone = Arc::new(state.inner().clone());
        let task_handle = tokio::spawn(playback_loop(window_clone, state_clone));
        *handle_guard = Some(task_handle);
    }

    let _ = window.emit("daw::playback-started", transport.position);
    info!("DAW: Playback started at position {}", transport.position);
    Ok(())
}

#[command]
pub async fn daw_pause(state: State<'_, DawState>, window: Window) -> Result<(), String> {
    let mut transport = state.transport.lock().unwrap();
    if !transport.playing {
        return Ok(()); // Not playing
    }
    transport.playing = false;

    let _ = window.emit("daw::playback-paused", transport.position);
    info!("DAW: Playback paused at position {}", transport.position);
    Ok(())
}

#[command]
pub async fn daw_stop(state: State<'_, DawState>, window: Window) -> Result<(), String> {
    let mut transport = state.transport.lock().unwrap();
    transport.playing = false;
    transport.recording = false;
    transport.position = 0.0;

    // Stop playback task
    let mut handle = state.playback_handle.lock().unwrap();
    if let Some(h) = handle.take() {
        h.abort();
    }

    let _ = window.emit("daw::playback-stopped", ());
    info!("DAW: Playback stopped and reset");
    Ok(())
}

#[command]
pub async fn daw_record(state: State<'_, DawState>) -> Result<(), String> {
    let mut transport = state.transport.lock().unwrap();
    transport.recording = !transport.recording;
    info!(
        "DAW: Recording {}",
        if transport.recording {
            "started"
        } else {
            "stopped"
        }
    );
    Ok(())
}

#[command]
pub async fn daw_set_bpm(state: State<'_, DawState>, bpm: f32) -> Result<(), String> {
    let mut transport = state.transport.lock().unwrap();
    transport.bpm = bpm.clamp(30.0, 300.0);
    info!("DAW: BPM set to {}", transport.bpm);
    Ok(())
}

#[command]
pub async fn daw_set_time_signature(
    state: State<'_, DawState>,
    numerator: u8,
    denominator: u8,
) -> Result<(), String> {
    let mut transport = state.transport.lock().unwrap();
    transport.time_signature = (numerator, denominator);
    info!("DAW: Time signature set to {}/{}", numerator, denominator);
    Ok(())
}

#[command]
pub async fn daw_set_loop(
    state: State<'_, DawState>,
    start: f64,
    end: f64,
    enabled: bool,
) -> Result<(), String> {
    let mut transport = state.transport.lock().unwrap();
    transport.looping = enabled;
    transport.loop_start = start;
    transport.loop_end = end;
    info!(
        "DAW: Loop {} from {} to {}",
        if enabled { "enabled" } else { "disabled" },
        start,
        end
    );
    Ok(())
}

#[command]
pub async fn daw_set_metronome(state: State<'_, DawState>, enabled: bool) -> Result<(), String> {
    let mut transport = state.transport.lock().unwrap();
    transport.metronome_enabled = enabled;
    info!(
        "DAW: Metronome {}",
        if enabled { "enabled" } else { "disabled" }
    );
    Ok(())
}

#[command]
pub async fn daw_get_transport_state(state: State<'_, DawState>) -> Result<TransportState, String> {
    let transport = state.transport.lock().unwrap();
    Ok(transport.clone())
}

#[command]
pub async fn daw_get_playback_info(state: State<'_, DawState>) -> Result<PlaybackInfo, String> {
    let transport = state.transport.lock().unwrap();
    let tracks = state.tracks.lock().unwrap();

    let ticks_per_beat = 480u32;
    let beats_per_bar = transport.time_signature.0 as f64;
    let seconds_per_beat = 60.0 / transport.bpm as f64;
    let seconds_per_bar = seconds_per_beat * beats_per_bar;

    let total_seconds = transport.position;
    let total_beats = total_seconds / seconds_per_beat;

    let bar = (total_beats / beats_per_bar).floor() as u32;
    let beat = (total_beats % beats_per_bar).floor() as u32;
    let beat_fraction = total_beats.fract();
    let tick = (beat_fraction * ticks_per_beat as f64).floor() as u32;

    // Calculate total_bars from track content or loop region
    let max_clip_end = tracks
        .iter()
        .flat_map(|track| track.clips.iter())
        .map(|clip| clip.start_time + clip.duration)
        .fold(0.0_f64, f64::max);

    // Use loop_end if looping is enabled and greater than clip content
    let max_duration = if transport.looping && transport.loop_end > max_clip_end {
        transport.loop_end
    } else {
        max_clip_end
    };

    // Default to 16 bars minimum, or calculate from content
    let total_bars = if max_duration > 0.0 {
        ((max_duration / seconds_per_bar).ceil() as u32).max(16)
    } else {
        16
    };

    Ok(PlaybackInfo {
        position: transport.position,
        bar,
        beat,
        tick,
        bpm: transport.bpm,
        time_signature: transport.time_signature,
        total_bars,
    })
}

#[command]
pub async fn daw_get_tracks(state: State<'_, DawState>) -> Result<Vec<Track>, String> {
    let tracks = state.tracks.lock().unwrap();
    Ok(tracks.clone())
}

#[command]
pub async fn daw_add_track(
    state: State<'_, DawState>,
    name: String,
    instrument: String,
    color: Option<String>,
) -> Result<Track, String> {
    let mut tracks = state.tracks.lock().unwrap();
    let id = tracks.len() as u32 + 1;

    let track = Track {
        id,
        name,
        instrument,
        color: color.unwrap_or_else(|| "#3498db".to_string()),
        ..Default::default()
    };

    tracks.push(track.clone());
    info!(
        "DAW: Added track '{}' ({}) with ID {}",
        track.name, track.instrument, id
    );
    Ok(track)
}

#[command]
pub async fn daw_remove_track(state: State<'_, DawState>, track_id: u32) -> Result<(), String> {
    let mut tracks = state.tracks.lock().unwrap();
    let initial_len = tracks.len();
    tracks.retain(|t| t.id != track_id);

    if tracks.len() < initial_len {
        info!("DAW: Removed track ID {}", track_id);
        Ok(())
    } else {
        Err(format!("Track ID {} not found", track_id))
    }
}

#[command]
pub async fn daw_update_track(
    state: State<'_, DawState>,
    updated_track: Track,
) -> Result<(), String> {
    let mut tracks = state.tracks.lock().unwrap();

    if let Some(track) = tracks.iter_mut().find(|t| t.id == updated_track.id) {
        *track = updated_track;
        info!("DAW: Updated track ID {}", track.id);
        Ok(())
    } else {
        Err(format!("Track ID {} not found", updated_track.id))
    }
}

#[command]
pub async fn daw_set_track_mute(
    state: State<'_, DawState>,
    track_id: u32,
    muted: bool,
) -> Result<(), String> {
    let mut tracks = state.tracks.lock().unwrap();

    if let Some(track) = tracks.iter_mut().find(|t| t.id == track_id) {
        track.muted = muted;
        info!(
            "DAW: {} track ID {}",
            if muted { "Muted" } else { "Unmuted" },
            track_id
        );
        Ok(())
    } else {
        Err(format!("Track ID {} not found", track_id))
    }
}

#[command]
pub async fn daw_set_track_solo(
    state: State<'_, DawState>,
    track_id: u32,
    solo: bool,
) -> Result<(), String> {
    let mut tracks = state.tracks.lock().unwrap();

    if let Some(track) = tracks.iter_mut().find(|t| t.id == track_id) {
        track.solo = solo;
        info!(
            "DAW: {} solo for track ID {}",
            if solo { "Set" } else { "Cleared" },
            track_id
        );
        Ok(())
    } else {
        Err(format!("Track ID {} not found", track_id))
    }
}

#[command]
pub async fn daw_add_clip(
    state: State<'_, DawState>,
    track_id: u32,
    name: String,
    start_time: f64,
    duration: f64,
    file_path: Option<String>,
) -> Result<Clip, String> {
    let mut tracks = state.tracks.lock().unwrap();

    if let Some(track) = tracks.iter_mut().find(|t| t.id == track_id) {
        let clip_id = track.clips.len() as u64 + 1;
        let clip = Clip {
            id: clip_id,
            name,
            start_time,
            duration,
            file_path,
            notes: Vec::new(),
            color: "#e74c3c".to_string(),
        };

        track.clips.push(clip.clone());
        info!(
            "DAW: Added clip '{}' to track ID {} at time {}",
            clip.name, track_id, start_time
        );
        Ok(clip)
    } else {
        Err(format!("Track ID {} not found", track_id))
    }
}

#[command]
pub async fn daw_remove_clip(
    state: State<'_, DawState>,
    track_id: u32,
    clip_id: u64,
) -> Result<(), String> {
    let mut tracks = state.tracks.lock().unwrap();

    if let Some(track) = tracks.iter_mut().find(|t| t.id == track_id) {
        let initial_len = track.clips.len();
        track.clips.retain(|c| c.id != clip_id);

        if track.clips.len() < initial_len {
            info!(
                "DAW: Removed clip ID {} from track ID {}",
                clip_id, track_id
            );
            Ok(())
        } else {
            Err(format!(
                "Clip ID {} not found in track ID {}",
                clip_id, track_id
            ))
        }
    } else {
        Err(format!("Track ID {} not found", track_id))
    }
}

#[command]
pub async fn daw_load_midi_file(
    _state: State<'_, DawState>,
    file_path: String,
) -> Result<Track, String> {
    let bytes = std::fs::read(&file_path).map_err(|e| e.to_string())?;
    let smf = Smf::parse(&bytes).map_err(|e| format!("MIDI parse error: {}", e))?;

    let file_name = std::path::Path::new(&file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let mut notes = Vec::new();
    let current_time: f64 = 0.0;

    // Extract ticks per beat from header timing
    let ticks_per_beat = match smf.header.timing {
        Timing::Metrical(ticks) => ticks.as_int() as f64,
        Timing::Timecode(_, _) => 480.0, // Default for timecode format
    };
    let bpm = 120.0; // Default, can be extracted from tempo events

    for track_events in smf.tracks {
        let mut track_time = 0.0;
        for event in track_events {
            // Convert u28 delta to f64 using as_int()
            let delta_ticks = event.delta.as_int() as f64;
            track_time += delta_ticks / ticks_per_beat * (60.0 / bpm);
            if let midly::TrackEventKind::Midi { channel, message } = event.kind {
                match message {
                    MidiMessage::NoteOn { key, vel } if vel > 0 => {
                        notes.push(MidiNote {
                            pitch: key.as_int(),
                            velocity: vel.as_int(),
                            start_time: track_time,
                            duration: 0.5, // Default, can be calculated from note off
                            channel: channel.as_int(),
                        });
                    },
                    _ => {},
                }
            }
        }
    }

    let track = Track {
        id: 0, // To be set on add
        name: file_name.clone(),
        instrument: "MIDI".to_string(),
        color: "#9b59b6".to_string(),
        volume: 0.8,
        pan: 0.0,
        muted: false,
        solo: false,
        clips: vec![Clip {
            id: 1,
            name: file_name,
            start_time: 0.0,
            duration: current_time.max(1.0),
            file_path: Some(file_path),
            notes,
            color: "#9b59b6".to_string(),
        }],
        effects: Vec::new(),
    };

    info!(
        "DAW: Loaded MIDI file with {} notes",
        track.clips[0].notes.len()
    );
    Ok(track)
}

/// Load MIDI data from base64-encoded bytes
///
/// Uses the custom MIDI parser for byte-level parsing.
/// Useful for clipboard data, drag-drop, or network sources.
#[command]
pub async fn daw_load_midi_from_bytes(
    _state: State<'_, DawState>,
    base64_data: String,
) -> Result<Track, String> {
    // Decode base64 to bytes
    let bytes = STANDARD
        .decode(&base64_data)
        .map_err(|e| format!("Base64 decode error: {}", e))?;

    // Parse using custom parser
    let loaded = load_midi_from_bytes(&bytes)?;

    // Convert LoadedMidiFile to Track format
    let mut notes = Vec::new();
    let ticks_per_quarter = loaded.ticks_per_quarter_note();
    let bpm = 120.0_f64; // Default BPM

    for event in &loaded.events {
        if let crate::daw_models::midi::MidiEventType::NoteOn = event.event_type {
            if let (Some(pitch), Some(velocity)) = (event.note, event.velocity) {
                if velocity > 0 {
                    // Convert tick to seconds
                    let start_seconds =
                        (event.tick as f64 / ticks_per_quarter as f64) * (60.0 / bpm);

                    notes.push(MidiNote {
                        pitch,
                        velocity,
                        start_time: start_seconds,
                        duration: 0.25, // Default duration, would need NoteOff pairing for accuracy
                        channel: event.channel,
                    });
                }
            }
        }
    }

    let track = Track {
        id: 1,
        name: "Imported MIDI".to_string(),
        instrument: "Piano".to_string(),
        color: "#3498db".to_string(),
        volume: 1.0,
        pan: 0.0,
        muted: false,
        solo: false,
        clips: vec![Clip {
            id: 1,
            name: "Imported Clip".to_string(),
            start_time: 0.0,
            duration: notes.iter().map(|n| n.start_time + n.duration).fold(0.0_f64, f64::max),
            file_path: None,
            notes,
            color: "#e74c3c".to_string(),
        }],
        effects: Vec::new(),
    };

    info!(
        "DAW: Loaded MIDI from bytes with {} notes (tpq: {})",
        track.clips[0].notes.len(),
        ticks_per_quarter
    );
    Ok(track)
}

#[command]
pub async fn daw_get_markers(state: State<'_, DawState>) -> Result<Vec<TimelineMarker>, String> {
    let markers = state.markers.lock().unwrap();
    Ok(markers.clone())
}

#[command]
pub async fn daw_add_marker(
    state: State<'_, DawState>,
    position: f64,
    name: String,
    color: Option<String>,
) -> Result<(), String> {
    let mut markers = state.markers.lock().unwrap();

    let marker = TimelineMarker {
        position,
        name: name.clone(),
        color: color.unwrap_or_else(|| "#f39c12".to_string()),
    };

    markers.push(marker);
    info!("DAW: Added marker '{}' at position {}", name, position);
    Ok(())
}

// Playback loop task
async fn playback_loop(window: Window, state: Arc<DawState>) {
    let mut interval = interval(Duration::from_millis(16)); // ~60 FPS
    let start_instant = Instant::now();

    loop {
        interval.tick().await;

        // Check if playing
        let playing = {
            let transport = state.transport.lock().unwrap();
            transport.playing
        };
        if !playing {
            break;
        }

        // Update position
        let mut transport = state.transport.lock().unwrap();
        let elapsed = start_instant.elapsed().as_secs_f64();
        transport.position = elapsed % transport.loop_end.max(1.0);

        // Handle looping
        if transport.looping && transport.position >= transport.loop_end {
            transport.position = transport.loop_start;
        }

        // Emit position update
        let _ = window.emit("daw::position-updated", transport.position);

        // Metronome
        if transport.metronome_enabled {
            let beats_per_bar = transport.time_signature.0 as f64;
            let seconds_per_beat = 60.0 / transport.bpm as f64;
            let current_beat = (transport.position / seconds_per_beat) % beats_per_bar;

            if current_beat < 0.1 {
                // New beat
                let _ = window.emit("daw::metronome-click", ());
            }
        }

        drop(transport);
    }

    info!("DAW: Playback loop stopped");
}

/// Parse raw MIDI bytes into a message
///
/// # Arguments
/// * `bytes` - Raw MIDI bytes
///
/// # Returns
/// Parsed MIDI message or error
#[command]
pub fn daw_parse_midi_bytes(bytes: Vec<u8>) -> Result<CustomMidiMessage, String> {
    CustomMidiMessage::from_bytes(&bytes)
}

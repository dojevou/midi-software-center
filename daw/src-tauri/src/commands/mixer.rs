//! MIDI Mixer commands - used via Tauri invoke_handler from app crate
#![allow(dead_code)] // Commands are called externally via Tauri IPC

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{command, Emitter, State, Window};
use tokio::time::interval;

/// MIDI Mixer Channel with full MIDI parameter control
/// Matches database schema from migration 018 and frontend midiMixerStore.ts
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MixerChannel {
    pub track_id: u32,
    pub name: String,
    pub enabled: bool,

    // MIDI Channel Configuration
    pub midi_channel: u8,           // 1-16
    pub midi_port_id: Option<u64>,  // Reference to midi_ports table

    // Core MIDI Parameters
    pub transpose: i8,              // -48 to +48 semitones
    pub octave_shift: i8,           // -4 to +4 octaves
    pub velocity: u8,               // 0-127 base velocity
    pub velocity_scale: u8,         // 1-200% velocity multiplier
    pub velocity_min: u8,           // 0-127 minimum velocity
    pub velocity_max: u8,           // 0-127 maximum velocity
    pub velocity_offset: i16,       // -127 to +127 velocity offset

    // Pitch Control
    pub pitch_bend: i16,            // -8192 to +8191
    pub pitch_bend_range: u8,       // 1-24 semitones

    // Note Range Filtering
    pub note_low: u8,               // 0-127 lowest note
    pub note_high: u8,              // 0-127 highest note
    pub key_scale: Option<String>,  // Key scale filter (e.g., "C major")

    // Timing Parameters
    pub quantize_grid: Option<String>, // e.g., "1/16", "1/8", "1/4"
    pub quantize_strength: u8,      // 0-100% quantize strength
    pub swing: u8,                  // 0-100%
    pub delay_ms: i16,              // -100 to +100 ms
    pub humanize: u8,               // 0-100%

    // Channel Controls (CC Messages)
    pub volume: u8,                 // CC7: 0-127
    pub pan: u8,                    // CC10: 0-127 (64 = center)
    pub expression: u8,             // CC11: 0-127
    pub modulation: u8,             // CC1: 0-127
    pub sustain_pedal: bool,        // CC64: on/off

    // Additional CCs (ADSR-style synthesis controls)
    pub cc74_brightness: u8,        // CC74: Filter cutoff/brightness
    pub cc71_resonance: u8,         // CC71: Filter resonance
    pub cc73_attack: u8,            // CC73: Attack time
    pub cc75_decay: u8,             // CC75: Decay time
    pub cc72_release: u8,           // CC72: Release time

    // Program Change / Bank Select
    pub program_change: Option<u8>, // 0-127
    pub bank_msb: Option<u8>,       // CC0: Bank Select MSB
    pub bank_lsb: Option<u8>,       // CC32: Bank Select LSB

    // Channel State
    pub muted: bool,
    pub solo: bool,

    // Metering (runtime only, not persisted)
    pub vu_level: (f32, f32),
    pub peak_level: (f32, f32),

    // Effects Chain
    pub effects: Vec<EffectSlot>,
}

impl Default for MixerChannel {
    fn default() -> Self {
        Self {
            track_id: 0,
            name: String::new(),
            enabled: true,
            midi_channel: 1,
            midi_port_id: None,
            transpose: 0,
            octave_shift: 0,
            velocity: 100,
            velocity_scale: 100,
            velocity_min: 1,
            velocity_max: 127,
            velocity_offset: 0,
            pitch_bend: 0,
            pitch_bend_range: 2,
            note_low: 0,
            note_high: 127,
            key_scale: None,
            quantize_grid: None,
            quantize_strength: 100,
            swing: 0,
            delay_ms: 0,
            humanize: 0,
            volume: 100,
            pan: 64,
            expression: 127,
            modulation: 0,
            sustain_pedal: false,
            cc74_brightness: 64,
            cc71_resonance: 64,
            cc73_attack: 64,
            cc75_decay: 64,
            cc72_release: 64,
            program_change: None,
            bank_msb: None,
            bank_lsb: None,
            muted: false,
            solo: false,
            vu_level: (-60.0, -60.0),
            peak_level: (-60.0, -60.0),
            effects: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct EffectSlot {
    pub id: u32,
    pub name: String,
    pub enabled: bool,
    pub wet_dry: f32,
    pub parameters: std::collections::HashMap<String, f32>,
}

/// MIDI Master Channel with global MIDI controls
/// Matches database schema from migration 018 and frontend midiMixerStore.ts
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MasterChannel {
    pub enabled: bool,

    // Global MIDI Parameters
    pub velocity: u8,               // 0-127 global velocity
    pub velocity_scale: u8,         // 1-200% global velocity scale
    pub velocity_min: u8,           // 0-127 global minimum velocity
    pub velocity_max: u8,           // 0-127 global maximum velocity
    pub velocity_offset: i16,       // -127 to +127 global velocity offset
    pub transpose: i8,              // -48 to +48 global transpose
    pub volume: u8,                 // CC7: 0-127 global volume
    pub pitch_bend: i16,            // -8192 to +8191 global pitch bend

    // Note Range Filtering
    pub note_low: u8,               // 0-127 lowest note
    pub note_high: u8,              // 0-127 highest note

    // Global Quantization
    pub quantize_grid: Option<String>, // e.g., "1/16", "1/8", "1/4"
    pub quantize_strength: u8,      // 0-100% global quantize strength

    // Tempo Control
    pub tempo: f32,                 // 20-300 BPM
    pub tempo_multiplier: f32,      // Speed multiplier (0.5, 1.0, 2.0, etc.)

    // Global Timing
    pub global_delay_ms: i16,       // Global delay offset
    pub global_swing: u8,           // 0-100% global swing
    pub global_humanize: u8,        // 0-100% global humanization

    // Output Configuration
    pub output_device: Option<String>,
    pub output_port: u8,

    // MIDI Clock & Transport
    pub clock_enabled: bool,        // Send MIDI clock
    pub transport_enabled: bool,    // Send transport (start/stop/continue)

    // Master Processing
    pub limiter_enabled: bool,
    pub compressor_enabled: bool,

    // Metering (runtime only)
    pub vu_level: (f32, f32),
    pub peak_level: (f32, f32),
}

impl Default for MasterChannel {
    fn default() -> Self {
        Self {
            enabled: true,
            velocity: 100,
            velocity_scale: 100,
            velocity_min: 1,
            velocity_max: 127,
            velocity_offset: 0,
            transpose: 0,
            volume: 100,
            pitch_bend: 0,
            note_low: 0,
            note_high: 127,
            quantize_grid: None,
            quantize_strength: 100,
            tempo: 120.0,
            tempo_multiplier: 1.0,
            global_delay_ms: 0,
            global_swing: 0,
            global_humanize: 0,
            output_device: None,
            output_port: 0,
            clock_enabled: true,
            transport_enabled: true,
            limiter_enabled: true,
            compressor_enabled: false,
            vu_level: (-60.0, -60.0),
            peak_level: (-60.0, -60.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeterData {
    pub track_id: u32,
    pub vu_left: f32,
    pub vu_right: f32,
    pub peak_left: f32,
    pub peak_right: f32,
}

#[derive(Debug, Clone)]
pub struct MixerState {
    pub channels: Arc<Mutex<Vec<MixerChannel>>>,
    pub master: Arc<Mutex<MasterChannel>>,
    pub meter_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl Default for MixerState {
    fn default() -> Self {
        Self {
            channels: Arc::new(Mutex::new(Vec::new())),
            master: Arc::new(Mutex::new(MasterChannel::default())),
            meter_handle: Arc::new(Mutex::new(None)),
        }
    }
}

#[command]
pub async fn mixer_get_channels(state: State<'_, MixerState>) -> Result<Vec<MixerChannel>, String> {
    let channels = state.channels.lock().unwrap();
    Ok(channels.clone())
}

#[command]
pub async fn mixer_get_master(state: State<'_, MixerState>) -> Result<MasterChannel, String> {
    let master = state.master.lock().unwrap();
    Ok(master.clone())
}

#[command]
pub async fn mixer_add_channel(
    state: State<'_, MixerState>,
    window: Window,
    track_id: u32,
    name: String,
    midi_channel: Option<u8>,
) -> Result<MixerChannel, String> {
    let mut channels = state.channels.lock().unwrap();

    let channel = MixerChannel {
        track_id,
        name: name.clone(),
        midi_channel: midi_channel.unwrap_or(1).clamp(1, 16),
        ..MixerChannel::default()
    };

    channels.push(channel.clone());

    // Start meter update if not running
    let mut handle_guard = state.meter_handle.lock().unwrap();
    if handle_guard.is_none() {
        let window_clone = window.clone();
        let state_clone = Arc::new(state.inner().clone());
        let spawn_handle = tokio::spawn(meter_update_loop(window_clone, state_clone));
        *handle_guard = Some(spawn_handle);
    }

    Ok(channel)
}

#[command]
pub async fn mixer_remove_channel(
    state: State<'_, MixerState>,
    track_id: u32,
) -> Result<(), String> {
    let mut channels = state.channels.lock().unwrap();
    let initial_len = channels.len();
    channels.retain(|c| c.track_id != track_id);

    if channels.len() < initial_len {
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set channel volume (CC7: 0-127)
#[command]
pub async fn mixer_set_volume(
    state: State<'_, MixerState>,
    track_id: u32,
    volume: u8,
) -> Result<(), String> {
    let volume = volume.min(127);
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.volume = volume;
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set channel pan (CC10: 0-127, 64 = center)
#[command]
pub async fn mixer_set_pan(
    state: State<'_, MixerState>,
    track_id: u32,
    pan: u8,
) -> Result<(), String> {
    let pan = pan.min(127);
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.pan = pan;
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set channel transpose (-48 to +48 semitones)
#[command]
pub async fn mixer_set_transpose(
    state: State<'_, MixerState>,
    track_id: u32,
    transpose: i8,
) -> Result<(), String> {
    let transpose = transpose.clamp(-48, 48);
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.transpose = transpose;
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set channel octave shift (-4 to +4 octaves)
#[command]
pub async fn mixer_set_octave_shift(
    state: State<'_, MixerState>,
    track_id: u32,
    octave_shift: i8,
) -> Result<(), String> {
    let octave_shift = octave_shift.clamp(-4, 4);
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.octave_shift = octave_shift;
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set channel velocity (0-127 base velocity scale)
#[command]
pub async fn mixer_set_velocity(
    state: State<'_, MixerState>,
    track_id: u32,
    velocity: u8,
) -> Result<(), String> {
    let velocity = velocity.min(127);
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.velocity = velocity;
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set channel velocity range (min: 0-127, max: 0-127)
#[command]
pub async fn mixer_set_velocity_range(
    state: State<'_, MixerState>,
    track_id: u32,
    velocity_min: u8,
    velocity_max: u8,
) -> Result<(), String> {
    let velocity_min = velocity_min.min(127);
    let velocity_max = velocity_max.min(127);
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.velocity_min = velocity_min;
        channel.velocity_max = velocity_max;
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set channel velocity scale (1-200%)
#[command]
pub async fn mixer_set_velocity_scale(
    state: State<'_, MixerState>,
    track_id: u32,
    scale: u8,
) -> Result<(), String> {
    let scale = scale.clamp(1, 200);
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.velocity_scale = scale;
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set channel velocity offset (-127 to +127)
#[command]
pub async fn mixer_set_velocity_offset(
    state: State<'_, MixerState>,
    track_id: u32,
    offset: i16,
) -> Result<(), String> {
    let offset = offset.clamp(-127, 127);
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.velocity_offset = offset;
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set channel pitch bend (-8192 to +8191)
#[command]
pub async fn mixer_set_pitch_bend(
    state: State<'_, MixerState>,
    track_id: u32,
    pitch_bend: i16,
) -> Result<(), String> {
    let pitch_bend = pitch_bend.clamp(-8192, 8191);
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.pitch_bend = pitch_bend;
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set channel pitch bend range (1-24 semitones)
#[command]
pub async fn mixer_set_pitch_bend_range(
    state: State<'_, MixerState>,
    track_id: u32,
    range: u8,
) -> Result<(), String> {
    let range = range.clamp(1, 24);
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.pitch_bend_range = range;
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set channel note range filter (0-127 for both low and high)
#[command]
pub async fn mixer_set_note_range(
    state: State<'_, MixerState>,
    track_id: u32,
    note_low: u8,
    note_high: u8,
) -> Result<(), String> {
    let note_low = note_low.min(127);
    let note_high = note_high.min(127);
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.note_low = note_low;
        channel.note_high = note_high;
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set channel key scale filter (e.g., "C major", "D minor")
#[command]
pub async fn mixer_set_key_scale(
    state: State<'_, MixerState>,
    track_id: u32,
    key_scale: Option<String>,
) -> Result<(), String> {
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.key_scale = key_scale;
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set channel quantize grid (e.g., "1/16", "1/8", "1/4")
#[command]
pub async fn mixer_set_quantize_grid(
    state: State<'_, MixerState>,
    track_id: u32,
    quantize_grid: Option<String>,
) -> Result<(), String> {
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.quantize_grid = quantize_grid;
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set channel quantize strength (0-100%)
#[command]
pub async fn mixer_set_quantize_strength(
    state: State<'_, MixerState>,
    track_id: u32,
    strength: u8,
) -> Result<(), String> {
    let strength = strength.min(100);
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.quantize_strength = strength;
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set channel swing (0-100%)
#[command]
pub async fn mixer_set_swing(
    state: State<'_, MixerState>,
    track_id: u32,
    swing: u8,
) -> Result<(), String> {
    let swing = swing.min(100);
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.swing = swing;
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set channel delay in milliseconds (-100 to +100)
#[command]
pub async fn mixer_set_delay(
    state: State<'_, MixerState>,
    track_id: u32,
    delay_ms: i16,
) -> Result<(), String> {
    let delay_ms = delay_ms.clamp(-100, 100);
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.delay_ms = delay_ms;
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set channel humanize (0-100%)
#[command]
pub async fn mixer_set_humanize(
    state: State<'_, MixerState>,
    track_id: u32,
    humanize: u8,
) -> Result<(), String> {
    let humanize = humanize.min(100);
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.humanize = humanize;
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set channel expression (CC11: 0-127)
#[command]
pub async fn mixer_set_expression(
    state: State<'_, MixerState>,
    track_id: u32,
    expression: u8,
) -> Result<(), String> {
    let expression = expression.min(127);
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.expression = expression;
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set channel modulation (CC1: 0-127)
#[command]
pub async fn mixer_set_modulation(
    state: State<'_, MixerState>,
    track_id: u32,
    modulation: u8,
) -> Result<(), String> {
    let modulation = modulation.min(127);
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.modulation = modulation;
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set channel sustain pedal (CC64: on/off)
#[command]
pub async fn mixer_set_sustain_pedal(
    state: State<'_, MixerState>,
    track_id: u32,
    sustain: bool,
) -> Result<(), String> {
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.sustain_pedal = sustain;
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set channel CC values (brightness, resonance, attack, decay, release)
#[command]
pub async fn mixer_set_cc_values(
    state: State<'_, MixerState>,
    track_id: u32,
    cc74_brightness: Option<u8>,
    cc71_resonance: Option<u8>,
    cc73_attack: Option<u8>,
    cc75_decay: Option<u8>,
    cc72_release: Option<u8>,
) -> Result<(), String> {
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        if let Some(v) = cc74_brightness {
            channel.cc74_brightness = v.min(127);
        }
        if let Some(v) = cc71_resonance {
            channel.cc71_resonance = v.min(127);
        }
        if let Some(v) = cc73_attack {
            channel.cc73_attack = v.min(127);
        }
        if let Some(v) = cc75_decay {
            channel.cc75_decay = v.min(127);
        }
        if let Some(v) = cc72_release {
            channel.cc72_release = v.min(127);
        }
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set channel program change and bank select
#[command]
pub async fn mixer_set_program_change(
    state: State<'_, MixerState>,
    track_id: u32,
    program: Option<u8>,
    bank_msb: Option<u8>,
    bank_lsb: Option<u8>,
) -> Result<(), String> {
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.program_change = program.map(|p| p.min(127));
        channel.bank_msb = bank_msb.map(|b| b.min(127));
        channel.bank_lsb = bank_lsb.map(|b| b.min(127));
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set channel MIDI output port
#[command]
pub async fn mixer_set_midi_port(
    state: State<'_, MixerState>,
    track_id: u32,
    midi_port_id: Option<u64>,
    midi_channel: Option<u8>,
) -> Result<(), String> {
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.midi_port_id = midi_port_id;
        if let Some(ch) = midi_channel {
            channel.midi_channel = ch.clamp(1, 16);
        }
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set channel enabled state
#[command]
pub async fn mixer_set_enabled(
    state: State<'_, MixerState>,
    track_id: u32,
    enabled: bool,
) -> Result<(), String> {
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.enabled = enabled;
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

#[command]
pub async fn mixer_set_mute(
    state: State<'_, MixerState>,
    track_id: u32,
    muted: bool,
) -> Result<(), String> {
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.muted = muted;
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

#[command]
pub async fn mixer_set_solo(
    state: State<'_, MixerState>,
    track_id: u32,
    solo: bool,
) -> Result<(), String> {
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.solo = solo;
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set master volume (CC7: 0-127)
#[command]
pub async fn mixer_set_master_volume(
    state: State<'_, MixerState>,
    volume: u8,
) -> Result<(), String> {
    let volume = volume.min(127);
    let mut master = state.master.lock().unwrap();
    master.volume = volume;
    Ok(())
}

#[command]
pub async fn mixer_set_master_limiter(
    state: State<'_, MixerState>,
    enabled: bool,
) -> Result<(), String> {
    let mut master = state.master.lock().unwrap();
    master.limiter_enabled = enabled;
    Ok(())
}

#[command]
pub async fn mixer_set_master_compressor(
    state: State<'_, MixerState>,
    enabled: bool,
) -> Result<(), String> {
    let mut master = state.master.lock().unwrap();
    master.compressor_enabled = enabled;
    Ok(())
}

/// Set master transpose (-48 to +48 semitones)
#[command]
pub async fn mixer_set_master_transpose(
    state: State<'_, MixerState>,
    transpose: i8,
) -> Result<(), String> {
    let transpose = transpose.clamp(-48, 48);
    let mut master = state.master.lock().unwrap();
    master.transpose = transpose;
    Ok(())
}

/// Set master velocity (0-127)
#[command]
pub async fn mixer_set_master_velocity(
    state: State<'_, MixerState>,
    velocity: u8,
) -> Result<(), String> {
    let velocity = velocity.min(127);
    let mut master = state.master.lock().unwrap();
    master.velocity = velocity;
    Ok(())
}

/// Set master velocity scale (1-200%)
#[command]
pub async fn mixer_set_master_velocity_scale(
    state: State<'_, MixerState>,
    scale: u8,
) -> Result<(), String> {
    let scale = scale.clamp(1, 200);
    let mut master = state.master.lock().unwrap();
    master.velocity_scale = scale;
    Ok(())
}

/// Set master velocity range (min: 0-127, max: 0-127)
#[command]
pub async fn mixer_set_master_velocity_range(
    state: State<'_, MixerState>,
    velocity_min: u8,
    velocity_max: u8,
) -> Result<(), String> {
    let velocity_min = velocity_min.min(127);
    let velocity_max = velocity_max.min(127);
    let mut master = state.master.lock().unwrap();
    master.velocity_min = velocity_min;
    master.velocity_max = velocity_max;
    Ok(())
}

/// Set master velocity offset (-127 to +127)
#[command]
pub async fn mixer_set_master_velocity_offset(
    state: State<'_, MixerState>,
    offset: i16,
) -> Result<(), String> {
    let offset = offset.clamp(-127, 127);
    let mut master = state.master.lock().unwrap();
    master.velocity_offset = offset;
    Ok(())
}

/// Set master pitch bend (-8192 to +8191)
#[command]
pub async fn mixer_set_master_pitch_bend(
    state: State<'_, MixerState>,
    pitch_bend: i16,
) -> Result<(), String> {
    let pitch_bend = pitch_bend.clamp(-8192, 8191);
    let mut master = state.master.lock().unwrap();
    master.pitch_bend = pitch_bend;
    Ok(())
}

/// Set master note range (0-127 for both low and high)
#[command]
pub async fn mixer_set_master_note_range(
    state: State<'_, MixerState>,
    note_low: u8,
    note_high: u8,
) -> Result<(), String> {
    let note_low = note_low.min(127);
    let note_high = note_high.min(127);
    let mut master = state.master.lock().unwrap();
    master.note_low = note_low;
    master.note_high = note_high;
    Ok(())
}

/// Set master quantize grid (e.g., "1/16", "1/8", "1/4")
#[command]
pub async fn mixer_set_master_quantize_grid(
    state: State<'_, MixerState>,
    quantize_grid: Option<String>,
) -> Result<(), String> {
    let mut master = state.master.lock().unwrap();
    master.quantize_grid = quantize_grid;
    Ok(())
}

/// Set master quantize strength (0-100%)
#[command]
pub async fn mixer_set_master_quantize_strength(
    state: State<'_, MixerState>,
    strength: u8,
) -> Result<(), String> {
    let strength = strength.min(100);
    let mut master = state.master.lock().unwrap();
    master.quantize_strength = strength;
    Ok(())
}

/// Set master tempo (20-300 BPM)
#[command]
pub async fn mixer_set_master_tempo(
    state: State<'_, MixerState>,
    tempo: f32,
) -> Result<(), String> {
    let tempo = tempo.clamp(20.0, 300.0);
    let mut master = state.master.lock().unwrap();
    master.tempo = tempo;
    Ok(())
}

/// Set master tempo multiplier (0.25 to 4.0)
#[command]
pub async fn mixer_set_master_tempo_multiplier(
    state: State<'_, MixerState>,
    multiplier: f32,
) -> Result<(), String> {
    let multiplier = multiplier.clamp(0.25, 4.0);
    let mut master = state.master.lock().unwrap();
    master.tempo_multiplier = multiplier;
    Ok(())
}

/// Set master global delay offset in milliseconds
#[command]
pub async fn mixer_set_master_delay(
    state: State<'_, MixerState>,
    delay_ms: i16,
) -> Result<(), String> {
    let mut master = state.master.lock().unwrap();
    master.global_delay_ms = delay_ms;
    Ok(())
}

/// Set master global swing (0-100%)
#[command]
pub async fn mixer_set_master_swing(
    state: State<'_, MixerState>,
    swing: u8,
) -> Result<(), String> {
    let swing = swing.min(100);
    let mut master = state.master.lock().unwrap();
    master.global_swing = swing;
    Ok(())
}

/// Set master global humanize (0-100%)
#[command]
pub async fn mixer_set_master_humanize(
    state: State<'_, MixerState>,
    humanize: u8,
) -> Result<(), String> {
    let humanize = humanize.min(100);
    let mut master = state.master.lock().unwrap();
    master.global_humanize = humanize;
    Ok(())
}

/// Set master output device
#[command]
pub async fn mixer_set_master_output(
    state: State<'_, MixerState>,
    output_device: Option<String>,
    output_port: Option<u8>,
) -> Result<(), String> {
    let mut master = state.master.lock().unwrap();
    master.output_device = output_device;
    if let Some(port) = output_port {
        master.output_port = port;
    }
    Ok(())
}

/// Set master MIDI clock enabled
#[command]
pub async fn mixer_set_master_clock_enabled(
    state: State<'_, MixerState>,
    enabled: bool,
) -> Result<(), String> {
    let mut master = state.master.lock().unwrap();
    master.clock_enabled = enabled;
    Ok(())
}

/// Set master transport enabled
#[command]
pub async fn mixer_set_master_transport_enabled(
    state: State<'_, MixerState>,
    enabled: bool,
) -> Result<(), String> {
    let mut master = state.master.lock().unwrap();
    master.transport_enabled = enabled;
    Ok(())
}

/// Set master enabled state
#[command]
pub async fn mixer_set_master_enabled(
    state: State<'_, MixerState>,
    enabled: bool,
) -> Result<(), String> {
    let mut master = state.master.lock().unwrap();
    master.enabled = enabled;
    Ok(())
}

/// Reset all peaks (channels and master)
#[command]
pub async fn mixer_reset_peaks(state: State<'_, MixerState>) -> Result<(), String> {
    let mut channels = state.channels.lock().unwrap();
    let mut master = state.master.lock().unwrap();

    for channel in channels.iter_mut() {
        channel.peak_level = (-60.0, -60.0);
    }
    master.peak_level = (-60.0, -60.0);
    Ok(())
}

/// Get a single channel by track ID
#[command]
pub async fn mixer_get_channel(
    state: State<'_, MixerState>,
    track_id: u32,
) -> Result<MixerChannel, String> {
    let channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter().find(|c| c.track_id == track_id) {
        Ok(channel.clone())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Batch update channel settings (for efficiency)
#[command]
pub async fn mixer_update_channel(
    state: State<'_, MixerState>,
    track_id: u32,
    updates: HashMap<String, serde_json::Value>,
) -> Result<MixerChannel, String> {
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        // Apply updates from the HashMap
        for (key, value) in updates {
            match key.as_str() {
                "enabled" => {
                    if let Some(v) = value.as_bool() {
                        channel.enabled = v;
                    }
                }
                "name" => {
                    if let Some(v) = value.as_str() {
                        channel.name = v.to_string();
                    }
                }
                "midi_channel" => {
                    if let Some(v) = value.as_u64() {
                        channel.midi_channel = (v as u8).clamp(1, 16);
                    }
                }
                "transpose" => {
                    if let Some(v) = value.as_i64() {
                        channel.transpose = (v as i8).clamp(-48, 48);
                    }
                }
                "octave_shift" => {
                    if let Some(v) = value.as_i64() {
                        channel.octave_shift = (v as i8).clamp(-4, 4);
                    }
                }
                "velocity" => {
                    if let Some(v) = value.as_u64() {
                        channel.velocity = (v as u8).min(127);
                    }
                }
                "velocity_min" => {
                    if let Some(v) = value.as_u64() {
                        channel.velocity_min = (v as u8).min(127);
                    }
                }
                "velocity_max" => {
                    if let Some(v) = value.as_u64() {
                        channel.velocity_max = (v as u8).min(127);
                    }
                }
                "pitch_bend" => {
                    if let Some(v) = value.as_i64() {
                        channel.pitch_bend = (v as i16).clamp(-8192, 8191);
                    }
                }
                "pitch_bend_range" => {
                    if let Some(v) = value.as_u64() {
                        channel.pitch_bend_range = (v as u8).clamp(1, 24);
                    }
                }
                "note_low" => {
                    if let Some(v) = value.as_u64() {
                        channel.note_low = (v as u8).min(127);
                    }
                }
                "note_high" => {
                    if let Some(v) = value.as_u64() {
                        channel.note_high = (v as u8).min(127);
                    }
                }
                "swing" => {
                    if let Some(v) = value.as_u64() {
                        channel.swing = (v as u8).min(100);
                    }
                }
                "delay_ms" => {
                    if let Some(v) = value.as_i64() {
                        channel.delay_ms = (v as i16).clamp(-100, 100);
                    }
                }
                "humanize" => {
                    if let Some(v) = value.as_u64() {
                        channel.humanize = (v as u8).min(100);
                    }
                }
                "volume" => {
                    if let Some(v) = value.as_u64() {
                        channel.volume = (v as u8).min(127);
                    }
                }
                "pan" => {
                    if let Some(v) = value.as_u64() {
                        channel.pan = (v as u8).min(127);
                    }
                }
                "expression" => {
                    if let Some(v) = value.as_u64() {
                        channel.expression = (v as u8).min(127);
                    }
                }
                "modulation" => {
                    if let Some(v) = value.as_u64() {
                        channel.modulation = (v as u8).min(127);
                    }
                }
                "sustain_pedal" => {
                    if let Some(v) = value.as_bool() {
                        channel.sustain_pedal = v;
                    }
                }
                "muted" => {
                    if let Some(v) = value.as_bool() {
                        channel.muted = v;
                    }
                }
                "solo" => {
                    if let Some(v) = value.as_bool() {
                        channel.solo = v;
                    }
                }
                _ => {} // Ignore unknown fields
            }
        }
        Ok(channel.clone())
    } else {
        Err("Channel not found".to_string())
    }
}

#[command]
pub async fn mixer_add_effect(
    state: State<'_, MixerState>,
    track_id: u32,
    effect_name: String,
) -> Result<EffectSlot, String> {
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        let effect_id = channel.effects.len() as u32 + 1;
        let effect = EffectSlot {
            id: effect_id,
            name: effect_name.clone(),
            enabled: true,
            wet_dry: 1.0,
            parameters: std::collections::HashMap::new(),
        };

        channel.effects.push(effect.clone());
        Ok(effect)
    } else {
        Err("Channel not found".to_string())
    }
}

#[command]
pub async fn mixer_remove_effect(
    state: State<'_, MixerState>,
    track_id: u32,
    effect_id: u32,
) -> Result<(), String> {
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        let initial_len = channel.effects.len();
        channel.effects.retain(|e| e.id != effect_id);

        if channel.effects.len() < initial_len {
            Ok(())
        } else {
            Err("Effect not found".to_string())
        }
    } else {
        Err("Channel not found".to_string())
    }
}

#[command]
pub async fn mixer_set_effect_enabled(
    state: State<'_, MixerState>,
    track_id: u32,
    effect_id: u32,
    enabled: bool,
) -> Result<(), String> {
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        if let Some(effect) = channel.effects.iter_mut().find(|e| e.id == effect_id) {
            effect.enabled = enabled;
            Ok(())
        } else {
            Err("Effect not found".to_string())
        }
    } else {
        Err("Channel not found".to_string())
    }
}

/// Toggle an effect's enabled state on a channel
#[command]
pub async fn mixer_toggle_channel_effect(
    state: State<'_, MixerState>,
    track_id: u32,
    effect_id: u32,
) -> Result<bool, String> {
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        if let Some(effect) = channel.effects.iter_mut().find(|e| e.id == effect_id) {
            effect.enabled = !effect.enabled;
            Ok(effect.enabled)
        } else {
            Err("Effect not found".to_string())
        }
    } else {
        Err("Channel not found".to_string())
    }
}

/// Set an effect parameter value
#[command]
pub async fn mixer_set_effect_parameter(
    state: State<'_, MixerState>,
    track_id: u32,
    effect_id: u32,
    parameter_name: String,
    value: f32,
) -> Result<(), String> {
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        if let Some(effect) = channel.effects.iter_mut().find(|e| e.id == effect_id) {
            effect.parameters.insert(parameter_name, value);
            Ok(())
        } else {
            Err("Effect not found".to_string())
        }
    } else {
        Err("Channel not found".to_string())
    }
}

/// Reorder effects on a channel
#[command]
pub async fn mixer_reorder_effects(
    state: State<'_, MixerState>,
    track_id: u32,
    effect_ids: Vec<u32>,
) -> Result<(), String> {
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        // Validate all effect IDs exist
        let current_ids: Vec<u32> = channel.effects.iter().map(|e| e.id).collect();
        for id in &effect_ids {
            if !current_ids.contains(id) {
                return Err(format!("Effect {} not found", id));
            }
        }

        // Reorder effects based on the provided order
        let mut reordered: Vec<EffectSlot> = Vec::with_capacity(effect_ids.len());
        for id in effect_ids {
            if let Some(effect) = channel.effects.iter().find(|e| e.id == id) {
                reordered.push(effect.clone());
            }
        }
        channel.effects = reordered;

        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

// Meter update loop
async fn meter_update_loop(window: Window, state: Arc<MixerState>) {
    let mut interval = interval(Duration::from_millis(50)); // 20 FPS

    loop {
        interval.tick().await;

        let mut meter_updates = Vec::new();

        {
            let mut rng = rand::thread_rng();
            let mut channels = state.channels.lock().unwrap();
            let mut master = state.master.lock().unwrap();

            // Update channels
            for channel in channels.iter_mut() {
                let activity = if channel.muted {
                    -60.0
                } else {
                    // Scale volume (0-127) to dB attenuation
                    let vol_db = (channel.volume as f32 / 127.0) * 30.0 - 30.0;
                    vol_db + rng.gen_range(-10.0..10.0)
                };

                // Convert pan (0-127, 64=center) to stereo balance (-1.0 to 1.0)
                let pan_normalized = (channel.pan as f32 - 64.0) / 64.0;
                let left_level = activity + (pan_normalized * -6.0).min(0.0);
                let right_level = activity + (pan_normalized * 6.0).max(0.0);

                channel.vu_level = (left_level, right_level);
                channel.peak_level.0 = channel.peak_level.0.max(left_level);
                channel.peak_level.1 = channel.peak_level.1.max(right_level);

                meter_updates.push(MeterData {
                    track_id: channel.track_id,
                    vu_left: left_level,
                    vu_right: right_level,
                    peak_left: channel.peak_level.0,
                    peak_right: channel.peak_level.1,
                });
            }

            // Update master
            let master_level = if channels.is_empty() {
                -60.0
            } else {
                channels.iter().map(|c| (c.vu_level.0 + c.vu_level.1) / 2.0).sum::<f32>()
                    / channels.len() as f32
            };

            master.vu_level = (master_level, master_level);
            master.peak_level.0 = master.peak_level.0.max(master_level);
            master.peak_level.1 = master.peak_level.1.max(master_level);
        }

        if !meter_updates.is_empty() {
            let _ = window.emit("mixer::meter-update", meter_updates);
        }

        // Reset peaks occasionally
        if rand::thread_rng().gen_bool(0.01) {
            let mut channels = state.channels.lock().unwrap();
            let mut master = state.master.lock().unwrap();

            for channel in channels.iter_mut() {
                channel.peak_level = (-60.0, -60.0);
            }
            master.peak_level = (-60.0, -60.0);
        }
    }
}

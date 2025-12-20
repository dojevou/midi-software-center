//! MIDI Mixer commands - used via Tauri invoke_handler from app crate
#![allow(dead_code)] // Commands are called externally via Tauri IPC

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{command, Emitter, State, Window};
use tokio::time::interval;

/// Monitoring mode for recording
/// - Off: No input monitoring
/// - Input: Always hear live input (latency-free)
/// - Auto: Hear input only when record-armed, playback when not
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Default)]
pub enum MonitoringMode {
    #[default]
    Off,
    Input,
    Auto,
}

/// MIDI Mixer Channel with full MIDI parameter control
/// Matches database schema from migration 018 and frontend midiMixerStore.ts
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MixerChannel {
    pub track_id: u32,
    pub name: String,
    pub enabled: bool,

    // MIDI Channel Configuration
    pub midi_channel: u8,          // 1-16
    pub midi_port_id: Option<u64>, // Reference to midi_ports table

    // Core MIDI Parameters
    pub transpose: i8,        // -48 to +48 semitones
    pub octave_shift: i8,     // -4 to +4 octaves
    pub velocity: u8,         // 0-127 base velocity
    pub velocity_scale: u8,   // 1-200% velocity multiplier
    pub velocity_min: u8,     // 0-127 minimum velocity
    pub velocity_max: u8,     // 0-127 maximum velocity
    pub velocity_offset: i16, // -127 to +127 velocity offset

    // Pitch Control
    pub pitch_bend: i16,      // -8192 to +8191
    pub pitch_bend_range: u8, // 1-24 semitones

    // Note Range Filtering
    pub note_low: u8,              // 0-127 lowest note
    pub note_high: u8,             // 0-127 highest note
    pub key_scale: Option<String>, // Key scale filter (e.g., "C major")

    // Timing Parameters
    pub quantize_grid: Option<String>, // e.g., "1/16", "1/8", "1/4"
    pub quantize_strength: u8,         // 0-100% quantize strength
    pub swing: u8,                     // 0-100%
    pub delay_ms: i16,                 // -100 to +100 ms
    pub humanize: u8,                  // 0-100%

    // Channel Controls (CC Messages)
    pub volume: u8,          // CC7: 0-127
    pub pan: u8,             // CC10: 0-127 (64 = center)
    pub expression: u8,      // CC11: 0-127
    pub modulation: u8,      // CC1: 0-127
    pub sustain_pedal: bool, // CC64: on/off

    // Additional CCs (ADSR-style synthesis controls)
    pub cc74_brightness: u8, // CC74: Filter cutoff/brightness
    pub cc71_resonance: u8,  // CC71: Filter resonance
    pub cc73_attack: u8,     // CC73: Attack time
    pub cc75_decay: u8,      // CC75: Decay time
    pub cc72_release: u8,    // CC72: Release time

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

    // Sends
    pub sends: Vec<Send>,

    // Routing
    pub output_bus: Option<u32>, // None = master, Some(id) = bus

    // Monitoring & Recording (Day 3)
    pub monitoring_mode: MonitoringMode,
    pub record_armed: bool,
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
            sends: Vec::new(),
            output_bus: None,
            monitoring_mode: MonitoringMode::default(),
            record_armed: false,
        }
    }
}

/// Effect slot in the mixer's effect chain
///
/// This is a lightweight data structure that stores effect metadata and parameters.
/// For actual DSP processing, use the `create_effect_instance()` function to instantiate
/// the corresponding `BuiltInEffect` from `crate::daw::effects`.
///
/// ## Supported Built-in Effects:
/// - "EQ3Band" - 3-band parametric equalizer
/// - "Compressor" - Dynamic range compressor
/// - "Reverb" - Algorithmic reverb (Freeverb-style)
/// - "Delay" - Stereo delay with ping-pong mode
///
/// ## Integration with DSP:
/// ```rust,ignore
/// use crate::daw::effects::{BuiltInEffect, StereoBuffer};
///
/// // Create effect instance from slot
/// let mut effect_instance = create_effect_instance(&effect_slot)?;
///
/// // Process audio in real-time
/// let mut buffer = StereoBuffer::new(512);
/// effect_instance.process(&mut buffer, 44100.0);
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct EffectSlot {
    pub id: u32,
    pub name: String,
    pub enabled: bool,
    pub wet_dry: f32,
    pub parameters: std::collections::HashMap<String, f32>,
}

impl EffectSlot {
    /// Create a DSP effect instance from this slot
    ///
    /// Returns a Box<dyn BuiltInEffect> that can be used for real-time audio processing.
    /// The effect parameters from this slot are applied to the created instance.
    pub fn create_effect_instance(&self) -> Result<Box<dyn crate::daw::BuiltInEffect>, String> {
        use crate::daw::{
            Compressor, CompressorParams, Delay, DelayParams, EQ3Band, EQ3BandParams, Reverb,
            ReverbParams,
        };

        match self.name.as_str() {
            "EQ3Band" => {
                let params = EQ3BandParams {
                    low_gain: self.parameters.get("low_gain").copied().unwrap_or(0.0),
                    mid_gain: self.parameters.get("mid_gain").copied().unwrap_or(0.0),
                    high_gain: self.parameters.get("high_gain").copied().unwrap_or(0.0),
                    low_freq: self.parameters.get("low_freq").copied().unwrap_or(100.0),
                    mid_freq: self.parameters.get("mid_freq").copied().unwrap_or(1000.0),
                    high_freq: self.parameters.get("high_freq").copied().unwrap_or(8000.0),
                    low_q: self.parameters.get("low_q").copied().unwrap_or(0.707),
                    mid_q: self.parameters.get("mid_q").copied().unwrap_or(0.707),
                    high_q: self.parameters.get("high_q").copied().unwrap_or(0.707),
                };
                Ok(Box::new(EQ3Band::new(params)))
            }
            "Compressor" => {
                let params = CompressorParams {
                    threshold: self.parameters.get("threshold").copied().unwrap_or(-20.0),
                    ratio: self.parameters.get("ratio").copied().unwrap_or(4.0),
                    attack: self.parameters.get("attack").copied().unwrap_or(10.0),
                    release: self.parameters.get("release").copied().unwrap_or(100.0),
                    makeup: self.parameters.get("makeup").copied().unwrap_or(0.0),
                    knee: self.parameters.get("knee").copied().unwrap_or(3.0),
                };
                Ok(Box::new(Compressor::new(params)))
            }
            "Reverb" => {
                let params = ReverbParams {
                    room_size: self.parameters.get("room_size").copied().unwrap_or(0.5),
                    damping: self.parameters.get("damping").copied().unwrap_or(0.5),
                    width: self.parameters.get("width").copied().unwrap_or(1.0),
                    wet: self.parameters.get("wet").copied().unwrap_or(0.3),
                    dry: self.parameters.get("dry").copied().unwrap_or(0.7),
                };
                Ok(Box::new(Reverb::new(params)))
            }
            "Delay" => {
                let params = DelayParams {
                    time_l: self.parameters.get("time_l").copied().unwrap_or(250.0),
                    time_r: self.parameters.get("time_r").copied().unwrap_or(250.0),
                    feedback: self.parameters.get("feedback").copied().unwrap_or(0.5),
                    wet: self.parameters.get("wet").copied().unwrap_or(0.3),
                    dry: self.parameters.get("dry").copied().unwrap_or(0.7),
                    ping_pong: self
                        .parameters
                        .get("ping_pong")
                        .copied()
                        .unwrap_or(0.0)
                        > 0.5,
                };
                Ok(Box::new(Delay::new(params, 44100.0)))
            }
            _ => Err(format!("Unknown effect type: {}", self.name)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Send {
    pub id: u32,
    pub destination: SendDestination,
    pub level: u8,      // 0-127
    pub pre_fader: bool,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SendDestination {
    Bus(u32),
    ExternalOutput(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bus {
    pub id: u32,
    pub name: String,
    pub volume: u8,           // 0-127
    pub pan: u8,              // 0-127
    pub muted: bool,
    pub solo: bool,
    pub effects: Vec<EffectSlot>,
    pub vu_level: (f32, f32),
    pub peak_level: (f32, f32),
}

/// MIDI Master Channel with global MIDI controls
/// Matches database schema from migration 018 and frontend midiMixerStore.ts
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MasterChannel {
    pub enabled: bool,

    // Global MIDI Parameters
    pub velocity: u8,         // 0-127 global velocity
    pub velocity_scale: u8,   // 1-200% global velocity scale
    pub velocity_min: u8,     // 0-127 global minimum velocity
    pub velocity_max: u8,     // 0-127 global maximum velocity
    pub velocity_offset: i16, // -127 to +127 global velocity offset
    pub transpose: i8,        // -48 to +48 global transpose
    pub volume: u8,           // CC7: 0-127 global volume
    pub pan: u8,              // CC10: 0-127 master pan (64 = center)
    pub pitch_bend: i16,      // -8192 to +8191 global pitch bend

    // Note Range Filtering
    pub note_low: u8,  // 0-127 lowest note
    pub note_high: u8, // 0-127 highest note

    // Global Quantization
    pub quantize_grid: Option<String>, // e.g., "1/16", "1/8", "1/4"
    pub quantize_strength: u8,         // 0-100% global quantize strength

    // Tempo Control
    pub tempo: f32,            // 20-300 BPM
    pub tempo_multiplier: f32, // Speed multiplier (0.5, 1.0, 2.0, etc.)

    // Global Timing
    pub global_delay_ms: i16, // Global delay offset
    pub global_swing: u8,     // 0-100% global swing
    pub global_humanize: u8,  // 0-100% global humanization

    // Output Configuration
    pub output_device: Option<String>,
    pub output_port: u8,

    // MIDI Clock & Transport
    pub clock_enabled: bool,     // Send MIDI clock
    pub transport_enabled: bool, // Send transport (start/stop/continue)

    // Master Processing
    pub limiter_enabled: bool,
    pub compressor_enabled: bool,

    // Latency Compensation (Day 3)
    pub latency_compensation_enabled: bool,
    pub plugin_delay_compensation_ms: f32, // PDC in milliseconds
    pub hardware_latency_ms: f32,          // Audio interface latency

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
            pan: 64,
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
            latency_compensation_enabled: true,
            plugin_delay_compensation_ms: 0.0,
            hardware_latency_ms: 0.0,
            vu_level: (-60.0, -60.0),
            peak_level: (-60.0, -60.0),
        }
    }
}

/// Latency report for the mixer system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyReport {
    pub hardware_latency_ms: f32,
    pub plugin_delay_compensation_ms: f32,
    pub total_latency_ms: f32,
    pub compensation_enabled: bool,
    pub per_track_latency: Vec<TrackLatency>,
}

/// Per-track latency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackLatency {
    pub track_id: u32,
    pub name: String,
    pub effect_latency_ms: f32,
    pub routing_latency_ms: f32,
    pub total_latency_ms: f32,
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
    pub buses: Arc<Mutex<Vec<Bus>>>,
    pub meter_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl Default for MixerState {
    fn default() -> Self {
        Self {
            channels: Arc::new(Mutex::new(Vec::new())),
            master: Arc::new(Mutex::new(MasterChannel::default())),
            buses: Arc::new(Mutex::new(Vec::new())),
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

/// Toggle mute state for a channel
#[command]
pub async fn mixer_toggle_mute(
    state: State<'_, MixerState>,
    track_id: u32,
) -> Result<bool, String> {
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.muted = !channel.muted;
        Ok(channel.muted)
    } else {
        Err("Channel not found".to_string())
    }
}

/// Toggle solo state for a channel
#[command]
pub async fn mixer_toggle_solo(
    state: State<'_, MixerState>,
    track_id: u32,
) -> Result<bool, String> {
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.solo = !channel.solo;
        Ok(channel.solo)
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

/// Set master pan (CC10: 0-127, 64 = center)
#[command]
pub async fn mixer_set_master_pan(
    state: State<'_, MixerState>,
    pan: u8,
) -> Result<(), String> {
    let pan = pan.min(127);
    let mut master = state.master.lock().unwrap();
    master.pan = pan;
    Ok(())
}

/// Get current meter values for all channels
#[command]
pub async fn mixer_get_meters(state: State<'_, MixerState>) -> Result<Vec<MeterData>, String> {
    let channels = state.channels.lock().unwrap();
    let meter_data: Vec<MeterData> = channels
        .iter()
        .map(|channel| MeterData {
            track_id: channel.track_id,
            vu_left: channel.vu_level.0,
            vu_right: channel.vu_level.1,
            peak_left: channel.peak_level.0,
            peak_right: channel.peak_level.1,
        })
        .collect();
    Ok(meter_data)
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
pub async fn mixer_set_master_swing(state: State<'_, MixerState>, swing: u8) -> Result<(), String> {
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
                },
                "name" => {
                    if let Some(v) = value.as_str() {
                        channel.name = v.to_string();
                    }
                },
                "midi_channel" => {
                    if let Some(v) = value.as_u64() {
                        channel.midi_channel = (v as u8).clamp(1, 16);
                    }
                },
                "transpose" => {
                    if let Some(v) = value.as_i64() {
                        channel.transpose = (v as i8).clamp(-48, 48);
                    }
                },
                "octave_shift" => {
                    if let Some(v) = value.as_i64() {
                        channel.octave_shift = (v as i8).clamp(-4, 4);
                    }
                },
                "velocity" => {
                    if let Some(v) = value.as_u64() {
                        channel.velocity = (v as u8).min(127);
                    }
                },
                "velocity_min" => {
                    if let Some(v) = value.as_u64() {
                        channel.velocity_min = (v as u8).min(127);
                    }
                },
                "velocity_max" => {
                    if let Some(v) = value.as_u64() {
                        channel.velocity_max = (v as u8).min(127);
                    }
                },
                "pitch_bend" => {
                    if let Some(v) = value.as_i64() {
                        channel.pitch_bend = (v as i16).clamp(-8192, 8191);
                    }
                },
                "pitch_bend_range" => {
                    if let Some(v) = value.as_u64() {
                        channel.pitch_bend_range = (v as u8).clamp(1, 24);
                    }
                },
                "note_low" => {
                    if let Some(v) = value.as_u64() {
                        channel.note_low = (v as u8).min(127);
                    }
                },
                "note_high" => {
                    if let Some(v) = value.as_u64() {
                        channel.note_high = (v as u8).min(127);
                    }
                },
                "swing" => {
                    if let Some(v) = value.as_u64() {
                        channel.swing = (v as u8).min(100);
                    }
                },
                "delay_ms" => {
                    if let Some(v) = value.as_i64() {
                        channel.delay_ms = (v as i16).clamp(-100, 100);
                    }
                },
                "humanize" => {
                    if let Some(v) = value.as_u64() {
                        channel.humanize = (v as u8).min(100);
                    }
                },
                "volume" => {
                    if let Some(v) = value.as_u64() {
                        channel.volume = (v as u8).min(127);
                    }
                },
                "pan" => {
                    if let Some(v) = value.as_u64() {
                        channel.pan = (v as u8).min(127);
                    }
                },
                "expression" => {
                    if let Some(v) = value.as_u64() {
                        channel.expression = (v as u8).min(127);
                    }
                },
                "modulation" => {
                    if let Some(v) = value.as_u64() {
                        channel.modulation = (v as u8).min(127);
                    }
                },
                "sustain_pedal" => {
                    if let Some(v) = value.as_bool() {
                        channel.sustain_pedal = v;
                    }
                },
                "muted" => {
                    if let Some(v) = value.as_bool() {
                        channel.muted = v;
                    }
                },
                "solo" => {
                    if let Some(v) = value.as_bool() {
                        channel.solo = v;
                    }
                },
                _ => {}, // Ignore unknown fields
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
    position: Option<usize>,
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

        // Insert at position if specified, otherwise append
        if let Some(pos) = position {
            let insert_pos = pos.min(channel.effects.len());
            channel.effects.insert(insert_pos, effect.clone());
        } else {
            channel.effects.push(effect.clone());
        }

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

/// Bypass/unbypass an effect (alias for mixer_set_effect_enabled with inverted logic)
#[command]
pub async fn mixer_bypass_effect(
    state: State<'_, MixerState>,
    track_id: u32,
    effect_id: u32,
    bypass: bool,
) -> Result<(), String> {
    // Bypass = true means enabled = false
    mixer_set_effect_enabled(state, track_id, effect_id, !bypass).await
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

// ============================================================================
// SEND CONTROLS
// ============================================================================

/// Set send level for a channel
#[command]
pub async fn mixer_set_send(
    state: State<'_, MixerState>,
    track_id: u32,
    send_id: u32,
    level: u8,
) -> Result<(), String> {
    let level = level.min(127);
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        if let Some(send) = channel.sends.iter_mut().find(|s| s.id == send_id) {
            send.level = level;
            Ok(())
        } else {
            Err("Send not found".to_string())
        }
    } else {
        Err("Channel not found".to_string())
    }
}

/// Add a send to a channel
#[command]
pub async fn mixer_add_send(
    state: State<'_, MixerState>,
    track_id: u32,
    destination_bus: u32,
    pre_fader: bool,
) -> Result<Send, String> {
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        let send_id = channel.sends.len() as u32 + 1;
        let send = Send {
            id: send_id,
            destination: SendDestination::Bus(destination_bus),
            level: 100,
            pre_fader,
            enabled: true,
        };
        channel.sends.push(send.clone());
        Ok(send)
    } else {
        Err("Channel not found".to_string())
    }
}

/// Remove a send from a channel
#[command]
pub async fn mixer_remove_send(
    state: State<'_, MixerState>,
    track_id: u32,
    send_id: u32,
) -> Result<(), String> {
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        let initial_len = channel.sends.len();
        channel.sends.retain(|s| s.id != send_id);

        if channel.sends.len() < initial_len {
            Ok(())
        } else {
            Err("Send not found".to_string())
        }
    } else {
        Err("Channel not found".to_string())
    }
}

// ============================================================================
// BUS MANAGEMENT AND ROUTING
// ============================================================================

/// Create a new bus
#[command]
pub async fn mixer_create_bus(
    state: State<'_, MixerState>,
    name: String,
) -> Result<Bus, String> {
    let mut buses = state.buses.lock().unwrap();
    let bus_id = buses.len() as u32 + 1;

    let bus = Bus {
        id: bus_id,
        name,
        volume: 100,
        pan: 64,
        muted: false,
        solo: false,
        effects: Vec::new(),
        vu_level: (-60.0, -60.0),
        peak_level: (-60.0, -60.0),
    };

    buses.push(bus.clone());
    Ok(bus)
}

/// Remove a bus
#[command]
pub async fn mixer_remove_bus(
    state: State<'_, MixerState>,
    bus_id: u32,
) -> Result<(), String> {
    let mut buses = state.buses.lock().unwrap();
    let initial_len = buses.len();
    buses.retain(|b| b.id != bus_id);

    if buses.len() < initial_len {
        Ok(())
    } else {
        Err("Bus not found".to_string())
    }
}

/// Route a track to a bus (or master if bus_id is None)
#[command]
pub async fn mixer_route_track(
    state: State<'_, MixerState>,
    track_id: u32,
    bus_id: Option<u32>,
) -> Result<(), String> {
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.output_bus = bus_id;
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Get current routing configuration
#[command]
pub async fn mixer_get_routing(
    state: State<'_, MixerState>,
) -> Result<HashMap<u32, Option<u32>>, String> {
    let channels = state.channels.lock().unwrap();
    let routing: HashMap<u32, Option<u32>> = channels
        .iter()
        .map(|ch| (ch.track_id, ch.output_bus))
        .collect();
    Ok(routing)
}

/// Get all buses
#[command]
pub async fn mixer_get_buses(state: State<'_, MixerState>) -> Result<Vec<Bus>, String> {
    let buses = state.buses.lock().unwrap();
    Ok(buses.clone())
}

// ============================================================================
// RESET AND COPY COMMANDS
// ============================================================================

/// Reset a track to default settings
#[command]
pub async fn mixer_reset_track(
    state: State<'_, MixerState>,
    track_id: u32,
) -> Result<(), String> {
    let mut channels = state.channels.lock().unwrap();

    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        let track_id = channel.track_id;
        let name = channel.name.clone();
        let midi_channel = channel.midi_channel;

        *channel = MixerChannel {
            track_id,
            name,
            midi_channel,
            ..MixerChannel::default()
        };
        Ok(())
    } else {
        Err("Channel not found".to_string())
    }
}

/// Reset all tracks to default settings
#[command]
pub async fn mixer_reset_all(state: State<'_, MixerState>) -> Result<(), String> {
    let mut channels = state.channels.lock().unwrap();

    for channel in channels.iter_mut() {
        let track_id = channel.track_id;
        let name = channel.name.clone();
        let midi_channel = channel.midi_channel;

        *channel = MixerChannel {
            track_id,
            name,
            midi_channel,
            ..MixerChannel::default()
        };
    }

    Ok(())
}

/// Copy settings from one track to another
#[command]
pub async fn mixer_copy_settings(
    state: State<'_, MixerState>,
    from_track: u32,
    to_track: u32,
) -> Result<(), String> {
    let mut channels = state.channels.lock().unwrap();

    // Find source channel
    let source = channels
        .iter()
        .find(|c| c.track_id == from_track)
        .cloned()
        .ok_or("Source channel not found")?;

    // Update destination channel
    if let Some(dest) = channels.iter_mut().find(|c| c.track_id == to_track) {
        // Preserve track_id and name, copy everything else
        let track_id = dest.track_id;
        let name = dest.name.clone();

        *dest = source.clone();
        dest.track_id = track_id;
        dest.name = name;

        Ok(())
    } else {
        Err("Destination channel not found".to_string())
    }
}

// ============================================
// Day 3: Monitoring & Recording Commands
// ============================================

/// Set monitoring mode for a track
/// - Off: No input monitoring
/// - Input: Always hear live input (latency-free)
/// - Auto: Hear input only when record-armed, playback when not
#[command]
pub fn mixer_set_monitoring(
    track_id: u32,
    mode: String,
    state: State<'_, Arc<MixerState>>,
) -> Result<MixerChannel, String> {
    let monitoring_mode = match mode.to_lowercase().as_str() {
        "off" => MonitoringMode::Off,
        "input" => MonitoringMode::Input,
        "auto" => MonitoringMode::Auto,
        _ => return Err(format!("Invalid monitoring mode: {}. Use 'off', 'input', or 'auto'", mode)),
    };

    let mut channels = state.channels.lock().unwrap();
    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.monitoring_mode = monitoring_mode;
        Ok(channel.clone())
    } else {
        Err(format!("Channel {} not found", track_id))
    }
}

/// Set record arm state for a track
/// When armed, the track is ready to receive/record MIDI input
#[command]
pub fn mixer_set_record_arm(
    track_id: u32,
    armed: bool,
    state: State<'_, Arc<MixerState>>,
) -> Result<MixerChannel, String> {
    let mut channels = state.channels.lock().unwrap();
    if let Some(channel) = channels.iter_mut().find(|c| c.track_id == track_id) {
        channel.record_armed = armed;

        // If arming and monitoring is Auto, enable monitoring automatically
        if armed && channel.monitoring_mode == MonitoringMode::Auto {
            // The channel will hear input when armed in Auto mode
            // This is handled by the audio engine based on the mode
        }

        Ok(channel.clone())
    } else {
        Err(format!("Channel {} not found", track_id))
    }
}

/// Configure latency compensation settings
/// - enabled: Turn PDC on/off
/// - hardware_latency_ms: Audio interface round-trip latency
/// - plugin_delay_ms: Additional plugin-induced delay
#[command]
pub fn mixer_set_latency_compensation(
    enabled: Option<bool>,
    hardware_latency_ms: Option<f32>,
    plugin_delay_ms: Option<f32>,
    state: State<'_, Arc<MixerState>>,
) -> Result<MasterChannel, String> {
    let mut master = state.master.lock().unwrap();

    if let Some(en) = enabled {
        master.latency_compensation_enabled = en;
    }

    if let Some(hw_latency) = hardware_latency_ms {
        if !(0.0..=500.0).contains(&hw_latency) {
            return Err("Hardware latency must be between 0 and 500 ms".to_string());
        }
        master.hardware_latency_ms = hw_latency;
    }

    if let Some(pdc) = plugin_delay_ms {
        if !(0.0..=1000.0).contains(&pdc) {
            return Err("Plugin delay compensation must be between 0 and 1000 ms".to_string());
        }
        master.plugin_delay_compensation_ms = pdc;
    }

    Ok(master.clone())
}

/// Get detailed latency report for all tracks
/// Returns hardware latency, plugin delay compensation, and per-track latency
#[command]
pub fn mixer_get_latency_report(
    state: State<'_, Arc<MixerState>>,
) -> Result<LatencyReport, String> {
    let master = state.master.lock().unwrap();
    let channels = state.channels.lock().unwrap();

    let mut per_track_latency = Vec::new();

    for channel in channels.iter() {
        // Calculate effect latency from enabled effects
        let effect_latency_ms = channel.effects.iter()
            .filter(|e| e.enabled)
            .map(|e| {
                // Estimate latency based on effect type
                // Most built-in effects are zero-latency, but some may have lookahead
                match e.name.as_str() {
                    "Compressor" => {
                        // Compressor may use lookahead for attack detection
                        e.parameters.get("lookahead_ms").copied().unwrap_or(0.0)
                    }
                    "Reverb" => {
                        // Reverb typically has pre-delay
                        e.parameters.get("predelay_ms").copied().unwrap_or(0.0)
                    }
                    "Delay" => {
                        // Delay effect's latency is the delay time (if not compensated)
                        0.0 // Delay is intentional, not latency
                    }
                    _ => 0.0,
                }
            })
            .sum();

        // Calculate routing latency (buses add minimal latency)
        let routing_latency_ms = if channel.output_bus.is_some() {
            0.25 // Typical bus routing overhead
        } else {
            0.0
        };

        per_track_latency.push(TrackLatency {
            track_id: channel.track_id,
            name: channel.name.clone(),
            effect_latency_ms,
            routing_latency_ms,
            total_latency_ms: effect_latency_ms + routing_latency_ms,
        });
    }

    // Find the maximum track latency for PDC
    let max_track_latency = per_track_latency.iter()
        .map(|t| t.total_latency_ms)
        .fold(0.0f32, |a, b| a.max(b));

    Ok(LatencyReport {
        hardware_latency_ms: master.hardware_latency_ms,
        plugin_delay_compensation_ms: master.plugin_delay_compensation_ms + max_track_latency,
        total_latency_ms: master.hardware_latency_ms + master.plugin_delay_compensation_ms + max_track_latency,
        compensation_enabled: master.latency_compensation_enabled,
        per_track_latency,
    })
}

// ============================================
// Day 4: Preset System Commands
// ============================================

/// Preset metadata for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetInfo {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub preset_type: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub use_count: i32,
    pub is_favorite: bool,
    pub is_factory: bool,
    pub created_at: String,
}

/// Save current mixer channel settings as a preset
/// Serializes the MixerChannel to JSONB and stores in database
#[command]
pub fn mixer_save_preset(
    track_id: u32,
    name: String,
    description: Option<String>,
    category: Option<String>,
    tags: Option<Vec<String>>,
    state: State<'_, Arc<MixerState>>,
) -> Result<PresetInfo, String> {
    let channels = state.channels.lock().unwrap();
    let channel = channels
        .iter()
        .find(|c| c.track_id == track_id)
        .ok_or_else(|| format!("Channel {} not found", track_id))?;

    // Serialize channel to JSON
    let preset_data = serde_json::to_value(channel)
        .map_err(|e| format!("Failed to serialize channel: {}", e))?;

    // Create preset info (would normally save to database)
    // For now, return the created preset info
    let preset = PresetInfo {
        id: chrono::Utc::now().timestamp_millis(),
        name: name.clone(),
        description,
        preset_type: "channel".to_string(),
        category,
        tags: tags.unwrap_or_default(),
        use_count: 0,
        is_favorite: false,
        is_factory: false,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    // In a full implementation, this would save to the mixer_presets table
    // For now we log the action
    tracing::info!(
        "Saved preset '{}' for track {} with data: {}",
        name,
        track_id,
        serde_json::to_string_pretty(&preset_data).unwrap_or_default()
    );

    Ok(preset)
}

/// Load a preset and apply it to a mixer channel
/// Retrieves preset from storage and updates the channel settings
#[command]
pub fn mixer_load_preset(
    track_id: u32,
    preset_data: serde_json::Value,
    state: State<'_, Arc<MixerState>>,
) -> Result<MixerChannel, String> {
    let mut channels = state.channels.lock().unwrap();
    let channel = channels
        .iter_mut()
        .find(|c| c.track_id == track_id)
        .ok_or_else(|| format!("Channel {} not found", track_id))?;

    // Preserve track identity
    let original_track_id = channel.track_id;
    let original_name = channel.name.clone();

    // Deserialize and apply preset data
    let preset_channel: MixerChannel = serde_json::from_value(preset_data)
        .map_err(|e| format!("Failed to parse preset data: {}", e))?;

    // Apply preset settings (preserve identity)
    channel.enabled = preset_channel.enabled;
    channel.midi_channel = preset_channel.midi_channel;
    channel.transpose = preset_channel.transpose;
    channel.octave_shift = preset_channel.octave_shift;
    channel.velocity = preset_channel.velocity;
    channel.velocity_scale = preset_channel.velocity_scale;
    channel.velocity_min = preset_channel.velocity_min;
    channel.velocity_max = preset_channel.velocity_max;
    channel.velocity_offset = preset_channel.velocity_offset;
    channel.pitch_bend = preset_channel.pitch_bend;
    channel.pitch_bend_range = preset_channel.pitch_bend_range;
    channel.note_low = preset_channel.note_low;
    channel.note_high = preset_channel.note_high;
    channel.key_scale = preset_channel.key_scale;
    channel.quantize_grid = preset_channel.quantize_grid;
    channel.quantize_strength = preset_channel.quantize_strength;
    channel.swing = preset_channel.swing;
    channel.delay_ms = preset_channel.delay_ms;
    channel.humanize = preset_channel.humanize;
    channel.volume = preset_channel.volume;
    channel.pan = preset_channel.pan;
    channel.expression = preset_channel.expression;
    channel.modulation = preset_channel.modulation;
    channel.sustain_pedal = preset_channel.sustain_pedal;
    channel.cc74_brightness = preset_channel.cc74_brightness;
    channel.cc71_resonance = preset_channel.cc71_resonance;
    channel.cc73_attack = preset_channel.cc73_attack;
    channel.cc75_decay = preset_channel.cc75_decay;
    channel.cc72_release = preset_channel.cc72_release;
    channel.program_change = preset_channel.program_change;
    channel.bank_msb = preset_channel.bank_msb;
    channel.bank_lsb = preset_channel.bank_lsb;
    channel.muted = preset_channel.muted;
    channel.solo = preset_channel.solo;
    channel.effects = preset_channel.effects;
    channel.sends = preset_channel.sends;
    channel.output_bus = preset_channel.output_bus;
    channel.monitoring_mode = preset_channel.monitoring_mode;
    channel.record_armed = preset_channel.record_armed;

    // Restore identity
    channel.track_id = original_track_id;
    channel.name = original_name;

    tracing::info!("Applied preset to track {}", track_id);
    Ok(channel.clone())
}

/// Get list of available presets
/// Returns preset metadata without the full preset data
#[command]
pub fn mixer_get_presets(
    preset_type: Option<String>,
    category: Option<String>,
    favorites_only: Option<bool>,
) -> Result<Vec<PresetInfo>, String> {
    // Return factory presets for now
    // In full implementation, this would query mixer_presets table
    let presets = vec![
        PresetInfo {
            id: 1,
            name: "Clean Channel".to_string(),
            description: Some("Default clean channel with no effects".to_string()),
            preset_type: "channel".to_string(),
            category: Some("utility".to_string()),
            tags: vec!["clean".to_string(), "default".to_string()],
            use_count: 0,
            is_favorite: false,
            is_factory: true,
            created_at: "2025-12-17T00:00:00Z".to_string(),
        },
        PresetInfo {
            id: 2,
            name: "Drums Channel".to_string(),
            description: Some("Optimized for drum tracks with compression".to_string()),
            preset_type: "channel".to_string(),
            category: Some("drums".to_string()),
            tags: vec!["drums".to_string(), "percussion".to_string()],
            use_count: 0,
            is_favorite: false,
            is_factory: true,
            created_at: "2025-12-17T00:00:00Z".to_string(),
        },
        PresetInfo {
            id: 3,
            name: "Bass Channel".to_string(),
            description: Some("Warm bass with subtle compression".to_string()),
            preset_type: "channel".to_string(),
            category: Some("bass".to_string()),
            tags: vec!["bass".to_string(), "low-end".to_string()],
            use_count: 0,
            is_favorite: false,
            is_factory: true,
            created_at: "2025-12-17T00:00:00Z".to_string(),
        },
        PresetInfo {
            id: 4,
            name: "Piano Channel".to_string(),
            description: Some("Classic piano with reverb".to_string()),
            preset_type: "channel".to_string(),
            category: Some("keys".to_string()),
            tags: vec!["piano".to_string(), "keys".to_string()],
            use_count: 0,
            is_favorite: false,
            is_factory: true,
            created_at: "2025-12-17T00:00:00Z".to_string(),
        },
        PresetInfo {
            id: 5,
            name: "Synth Lead".to_string(),
            description: Some("Bright synth lead with delay".to_string()),
            preset_type: "channel".to_string(),
            category: Some("synth".to_string()),
            tags: vec!["synth".to_string(), "lead".to_string()],
            use_count: 0,
            is_favorite: false,
            is_factory: true,
            created_at: "2025-12-17T00:00:00Z".to_string(),
        },
        PresetInfo {
            id: 6,
            name: "Pad Channel".to_string(),
            description: Some("Atmospheric pad with lush reverb".to_string()),
            preset_type: "channel".to_string(),
            category: Some("synth".to_string()),
            tags: vec!["pad".to_string(), "ambient".to_string()],
            use_count: 0,
            is_favorite: false,
            is_factory: true,
            created_at: "2025-12-17T00:00:00Z".to_string(),
        },
    ];

    // Filter by type
    let filtered: Vec<PresetInfo> = presets
        .into_iter()
        .filter(|p| {
            preset_type.as_ref().is_none_or(|t| &p.preset_type == t)
                && category.as_ref().is_none_or(|c| p.category.as_ref() == Some(c))
                && favorites_only.is_none_or(|f| !f || p.is_favorite)
        })
        .collect();

    Ok(filtered)
}

/// Delete a preset by ID
/// Cannot delete factory presets
#[command]
pub fn mixer_delete_preset(preset_id: i64) -> Result<bool, String> {
    // In full implementation, this would delete from mixer_presets table
    // For now, just validate the request

    // Factory presets (IDs 1-6) cannot be deleted
    if preset_id <= 6 {
        return Err("Cannot delete factory presets".to_string());
    }

    tracing::info!("Deleted preset {}", preset_id);
    Ok(true)
}

/// Toggle preset as favorite
#[command]
pub fn mixer_toggle_preset_favorite(preset_id: i64) -> Result<bool, String> {
    // In full implementation, this would update is_favorite in mixer_presets table
    if preset_id <= 0 {
        return Err("Invalid preset ID".to_string());
    }

    tracing::info!("Toggled favorite for preset {}", preset_id);
    Ok(true)
}

// ============================================
// Plugin Discovery System (Day 5)
// ============================================

/// Plugin type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginType {
    BuiltIn,
    VST2,
    VST3,
    AU,      // Audio Unit (macOS)
    CLAP,    // CLever Audio Plug-in
    LV2,     // Linux Audio Plugin
}

/// Plugin category for organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginCategory {
    Effect,
    Instrument,
    Analyzer,
    Utility,
    Unknown,
}

/// Information about an available plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Unique plugin identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Plugin vendor/manufacturer
    pub vendor: String,
    /// Plugin version string
    pub version: String,
    /// Plugin type (VST2, VST3, AU, etc.)
    pub plugin_type: PluginType,
    /// Plugin category
    pub category: PluginCategory,
    /// File path (for external plugins)
    pub path: Option<String>,
    /// Whether the plugin is currently available/loaded
    pub is_available: bool,
    /// Number of audio inputs
    pub num_inputs: u32,
    /// Number of audio outputs
    pub num_outputs: u32,
    /// Whether plugin supports MIDI input
    pub accepts_midi: bool,
    /// Brief description
    pub description: Option<String>,
    /// Tags for searchability
    pub tags: Vec<String>,
}

/// Result of a plugin scan operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginScanResult {
    /// Number of plugins found
    pub plugins_found: usize,
    /// Number of plugins that failed to load
    pub plugins_failed: usize,
    /// Directories that were scanned
    pub directories_scanned: Vec<String>,
    /// Time taken in milliseconds
    pub scan_time_ms: u64,
    /// Any errors encountered
    pub errors: Vec<String>,
}

/// Get list of all available plugins (built-in + discovered)
///
/// Returns built-in effects and any VST/AU plugins found during scanning.
/// Built-in plugins are always available regardless of scan status.
#[command]
#[allow(clippy::vec_init_then_push)]
pub fn mixer_get_plugin_list(
    plugin_type: Option<String>,
    category: Option<String>,
) -> Result<Vec<PluginInfo>, String> {
    let mut plugins = Vec::new();

    // Add built-in effects (from Day 2 DSP module)
    plugins.push(PluginInfo {
        id: "builtin:eq3band".to_string(),
        name: "EQ3Band".to_string(),
        vendor: "MIDI Software Center".to_string(),
        version: "1.0.0".to_string(),
        plugin_type: PluginType::BuiltIn,
        category: PluginCategory::Effect,
        path: None,
        is_available: true,
        num_inputs: 2,
        num_outputs: 2,
        accepts_midi: false,
        description: Some("3-band parametric equalizer with low/mid/high controls".to_string()),
        tags: vec!["eq".to_string(), "equalizer".to_string(), "tone".to_string(), "filter".to_string()],
    });

    plugins.push(PluginInfo {
        id: "builtin:compressor".to_string(),
        name: "Compressor".to_string(),
        vendor: "MIDI Software Center".to_string(),
        version: "1.0.0".to_string(),
        plugin_type: PluginType::BuiltIn,
        category: PluginCategory::Effect,
        path: None,
        is_available: true,
        num_inputs: 2,
        num_outputs: 2,
        accepts_midi: false,
        description: Some("Dynamic range compressor with threshold, ratio, attack, release, and makeup gain".to_string()),
        tags: vec!["dynamics".to_string(), "compressor".to_string(), "leveling".to_string()],
    });

    plugins.push(PluginInfo {
        id: "builtin:reverb".to_string(),
        name: "Reverb".to_string(),
        vendor: "MIDI Software Center".to_string(),
        version: "1.0.0".to_string(),
        plugin_type: PluginType::BuiltIn,
        category: PluginCategory::Effect,
        path: None,
        is_available: true,
        num_inputs: 2,
        num_outputs: 2,
        accepts_midi: false,
        description: Some("Freeverb-style algorithmic reverb with room size, damping, and width controls".to_string()),
        tags: vec!["reverb".to_string(), "space".to_string(), "ambience".to_string(), "room".to_string()],
    });

    plugins.push(PluginInfo {
        id: "builtin:delay".to_string(),
        name: "Delay".to_string(),
        vendor: "MIDI Software Center".to_string(),
        version: "1.0.0".to_string(),
        plugin_type: PluginType::BuiltIn,
        category: PluginCategory::Effect,
        path: None,
        is_available: true,
        num_inputs: 2,
        num_outputs: 2,
        accepts_midi: false,
        description: Some("Stereo delay with independent L/R timing, feedback, and ping-pong mode".to_string()),
        tags: vec!["delay".to_string(), "echo".to_string(), "time".to_string(), "ping-pong".to_string()],
    });

    // Add built-in MIDI utilities
    plugins.push(PluginInfo {
        id: "builtin:midi_filter".to_string(),
        name: "MIDI Filter".to_string(),
        vendor: "MIDI Software Center".to_string(),
        version: "1.0.0".to_string(),
        plugin_type: PluginType::BuiltIn,
        category: PluginCategory::Utility,
        path: None,
        is_available: true,
        num_inputs: 0,
        num_outputs: 0,
        accepts_midi: true,
        description: Some("Filter MIDI events by type, channel, or note range".to_string()),
        tags: vec!["midi".to_string(), "filter".to_string(), "utility".to_string()],
    });

    plugins.push(PluginInfo {
        id: "builtin:midi_monitor".to_string(),
        name: "MIDI Monitor".to_string(),
        vendor: "MIDI Software Center".to_string(),
        version: "1.0.0".to_string(),
        plugin_type: PluginType::BuiltIn,
        category: PluginCategory::Analyzer,
        path: None,
        is_available: true,
        num_inputs: 0,
        num_outputs: 0,
        accepts_midi: true,
        description: Some("Monitor and visualize MIDI events in real-time".to_string()),
        tags: vec!["midi".to_string(), "monitor".to_string(), "analyzer".to_string(), "debug".to_string()],
    });

    plugins.push(PluginInfo {
        id: "builtin:velocity_curve".to_string(),
        name: "Velocity Curve".to_string(),
        vendor: "MIDI Software Center".to_string(),
        version: "1.0.0".to_string(),
        plugin_type: PluginType::BuiltIn,
        category: PluginCategory::Utility,
        path: None,
        is_available: true,
        num_inputs: 0,
        num_outputs: 0,
        accepts_midi: true,
        description: Some("Apply custom velocity curves and scaling to MIDI notes".to_string()),
        tags: vec!["midi".to_string(), "velocity".to_string(), "curve".to_string(), "dynamics".to_string()],
    });

    plugins.push(PluginInfo {
        id: "builtin:arpeggiator".to_string(),
        name: "Arpeggiator".to_string(),
        vendor: "MIDI Software Center".to_string(),
        version: "1.0.0".to_string(),
        plugin_type: PluginType::BuiltIn,
        category: PluginCategory::Effect,
        path: None,
        is_available: true,
        num_inputs: 0,
        num_outputs: 0,
        accepts_midi: true,
        description: Some("Generate arpeggiated patterns from held notes".to_string()),
        tags: vec!["midi".to_string(), "arpeggio".to_string(), "pattern".to_string(), "sequence".to_string()],
    });

    // Filter by type if specified
    let plugins = if let Some(ref type_filter) = plugin_type {
        let target_type = match type_filter.to_lowercase().as_str() {
            "builtin" | "built-in" => Some(PluginType::BuiltIn),
            "vst2" | "vst" => Some(PluginType::VST2),
            "vst3" => Some(PluginType::VST3),
            "au" | "audiounit" => Some(PluginType::AU),
            "clap" => Some(PluginType::CLAP),
            "lv2" => Some(PluginType::LV2),
            _ => None,
        };

        if let Some(t) = target_type {
            plugins.into_iter().filter(|p| p.plugin_type == t).collect()
        } else {
            plugins
        }
    } else {
        plugins
    };

    // Filter by category if specified
    let plugins = if let Some(ref cat_filter) = category {
        let target_cat = match cat_filter.to_lowercase().as_str() {
            "effect" | "effects" | "fx" => Some(PluginCategory::Effect),
            "instrument" | "instruments" | "synth" => Some(PluginCategory::Instrument),
            "analyzer" | "analysis" => Some(PluginCategory::Analyzer),
            "utility" | "utilities" | "tool" => Some(PluginCategory::Utility),
            _ => None,
        };

        if let Some(c) = target_cat {
            plugins.into_iter().filter(|p| p.category == c).collect()
        } else {
            plugins
        }
    } else {
        plugins
    };

    tracing::info!("Returning {} plugins (type: {:?}, category: {:?})",
        plugins.len(), plugin_type, category);

    Ok(plugins)
}

/// Scan filesystem for VST/AU plugins
///
/// Scans standard plugin directories for available plugins.
/// On Linux: ~/.vst, ~/.vst3, /usr/lib/vst, /usr/lib/vst3, /usr/lib/lv2
/// On macOS: ~/Library/Audio/Plug-Ins, /Library/Audio/Plug-Ins
/// On Windows: C:\Program Files\VstPlugins, C:\Program Files\Common Files\VST3
#[command]
pub async fn mixer_scan_plugins(
    custom_paths: Option<Vec<String>>,
) -> Result<PluginScanResult, String> {
    use std::time::Instant;

    let start = Instant::now();
    let mut directories_scanned = Vec::new();
    let mut plugins_found = 0;
    let mut plugins_failed = 0;
    let mut errors = Vec::new();

    // Determine OS-specific plugin directories
    let mut scan_dirs: Vec<String> = Vec::new();

    #[cfg(target_os = "linux")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let home = home.to_string_lossy();
            scan_dirs.push(format!("{}/.vst", home));
            scan_dirs.push(format!("{}/.vst3", home));
            scan_dirs.push(format!("{}/.lv2", home));
            scan_dirs.push(format!("{}/.clap", home));
        }
        scan_dirs.push("/usr/lib/vst".to_string());
        scan_dirs.push("/usr/lib/vst3".to_string());
        scan_dirs.push("/usr/lib/lv2".to_string());
        scan_dirs.push("/usr/local/lib/vst".to_string());
        scan_dirs.push("/usr/local/lib/vst3".to_string());
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let home = home.to_string_lossy();
            scan_dirs.push(format!("{}/Library/Audio/Plug-Ins/VST", home));
            scan_dirs.push(format!("{}/Library/Audio/Plug-Ins/VST3", home));
            scan_dirs.push(format!("{}/Library/Audio/Plug-Ins/Components", home));
        }
        scan_dirs.push("/Library/Audio/Plug-Ins/VST".to_string());
        scan_dirs.push("/Library/Audio/Plug-Ins/VST3".to_string());
        scan_dirs.push("/Library/Audio/Plug-Ins/Components".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        scan_dirs.push("C:\\Program Files\\VstPlugins".to_string());
        scan_dirs.push("C:\\Program Files\\Steinberg\\VstPlugins".to_string());
        scan_dirs.push("C:\\Program Files\\Common Files\\VST3".to_string());
        scan_dirs.push("C:\\Program Files (x86)\\VstPlugins".to_string());
        scan_dirs.push("C:\\Program Files (x86)\\Steinberg\\VstPlugins".to_string());
        scan_dirs.push("C:\\Program Files (x86)\\Common Files\\VST3".to_string());
    }

    // Add custom paths if provided
    if let Some(custom) = custom_paths {
        scan_dirs.extend(custom);
    }

    // Scan each directory
    for dir in &scan_dirs {
        let path = std::path::Path::new(dir);

        if !path.exists() {
            continue;
        }

        directories_scanned.push(dir.clone());

        match scan_plugin_directory(path) {
            Ok((found, failed)) => {
                plugins_found += found;
                plugins_failed += failed;
            }
            Err(e) => {
                errors.push(format!("Error scanning {}: {}", dir, e));
            }
        }
    }

    let scan_time_ms = start.elapsed().as_millis() as u64;

    tracing::info!(
        "Plugin scan complete: {} found, {} failed, {} dirs scanned in {}ms",
        plugins_found, plugins_failed, directories_scanned.len(), scan_time_ms
    );

    Ok(PluginScanResult {
        plugins_found,
        plugins_failed,
        directories_scanned,
        scan_time_ms,
        errors,
    })
}

/// Helper function to scan a single plugin directory
fn scan_plugin_directory(path: &std::path::Path) -> Result<(usize, usize), std::io::Error> {
    let mut found = 0;
    let mut failed = 0;

    let entries = std::fs::read_dir(path)?;

    for entry in entries.flatten() {
        let entry_path = entry.path();
        let extension = entry_path.extension().and_then(|e| e.to_str()).unwrap_or("");

        // Check for plugin file extensions
        let is_plugin = matches!(
            extension.to_lowercase().as_str(),
            "dll" | "so" | "vst" | "vst3" | "component" | "lv2" | "clap"
        );

        // Also check for plugin bundles (directories with specific extensions)
        let is_bundle = entry_path.is_dir() && matches!(
            extension.to_lowercase().as_str(),
            "vst3" | "component" | "lv2" | "clap"
        );

        if is_plugin || is_bundle {
            // In a full implementation, we would:
            // 1. Load the plugin binary
            // 2. Query its metadata (name, vendor, category, etc.)
            // 3. Validate it loads correctly
            // 4. Add to the plugin database

            // For now, count as found (actual loading would happen separately)
            if entry_path.exists() {
                found += 1;
                tracing::debug!("Found plugin: {:?}", entry_path);
            } else {
                failed += 1;
            }
        }

        // Recursively scan subdirectories (for VST3/LV2 bundles)
        if entry_path.is_dir() && !is_bundle {
            if let Ok((sub_found, sub_failed)) = scan_plugin_directory(&entry_path) {
                found += sub_found;
                failed += sub_failed;
            }
        }
    }

    Ok((found, failed))
}

// ============================================
// Meter Update Loop
// ============================================

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

// ============================================
// UNIT TESTS
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // HELPER FUNCTIONS
    // ========================================

    fn create_test_state() -> Arc<MixerState> {
        Arc::new(MixerState::default())
    }

    fn create_test_channel(track_id: u32) -> MixerChannel {
        MixerChannel {
            track_id,
            name: format!("Track {}", track_id),
            ..MixerChannel::default()
        }
    }

    // ========================================
    // MIXERCHANNEL TESTS
    // ========================================

    #[test]
    fn test_mixer_channel_default() {
        let channel = MixerChannel::default();
        assert_eq!(channel.track_id, 0);
        assert_eq!(channel.volume, 100);
        assert_eq!(channel.pan, 64);
        assert!(!channel.muted);
        assert!(!channel.solo);
        assert_eq!(channel.midi_channel, 1);
        assert_eq!(channel.velocity, 100);
        assert_eq!(channel.transpose, 0);
        assert_eq!(channel.monitoring_mode, MonitoringMode::Off);
        assert!(!channel.record_armed);
    }

    #[test]
    fn test_mixer_channel_serialization() {
        let channel = MixerChannel::default();
        let json = serde_json::to_string(&channel).unwrap();
        let deserialized: MixerChannel = serde_json::from_str(&json).unwrap();
        assert_eq!(channel.track_id, deserialized.track_id);
        assert_eq!(channel.volume, deserialized.volume);
        assert_eq!(channel.pan, deserialized.pan);
    }

    // ========================================
    // MASTER CHANNEL TESTS
    // ========================================

    #[test]
    fn test_master_channel_default() {
        let master = MasterChannel::default();
        assert_eq!(master.volume, 100);
        assert_eq!(master.pan, 64);
        assert!(master.enabled);
        // Default enables latency compensation
        assert!(master.latency_compensation_enabled);
        assert_eq!(master.plugin_delay_compensation_ms, 0.0);
        assert_eq!(master.hardware_latency_ms, 0.0);
    }

    // ========================================
    // MIXER STATE TESTS
    // ========================================

    #[test]
    fn test_mixer_state_default() {
        let state = MixerState::default();
        let channels = state.channels.lock().unwrap();
        let master = state.master.lock().unwrap();
        let buses = state.buses.lock().unwrap();

        assert_eq!(channels.len(), 0);
        assert_eq!(buses.len(), 0);
        assert_eq!(master.volume, 100);
    }

    // ========================================
    // EFFECT SLOT TESTS
    // ========================================

    #[test]
    fn test_effect_slot_default() {
        let effect = EffectSlot::default();
        assert_eq!(effect.id, 0);
        assert_eq!(effect.name, String::new());
        // derive(Default) gives false for bool
        assert!(!effect.enabled);
        // derive(Default) gives 0.0 for f32
        assert_eq!(effect.wet_dry, 0.0);
        assert!(effect.parameters.is_empty());
    }

    #[test]
    fn test_effect_slot_with_parameters() {
        let mut params = HashMap::new();
        params.insert("threshold".to_string(), -15.0f32);
        params.insert("ratio".to_string(), 4.0f32);

        let effect = EffectSlot {
            id: 1,
            name: "Compressor".to_string(),
            enabled: true,
            wet_dry: 1.0,
            parameters: params.clone(),
        };

        assert_eq!(effect.parameters.get("threshold"), Some(&-15.0));
        assert_eq!(effect.parameters.get("ratio"), Some(&4.0));
    }

    // ========================================
    // SEND TESTS
    // ========================================

    #[test]
    fn test_send_creation() {
        let send = Send {
            id: 1,
            destination: SendDestination::Bus(1),
            level: 100,
            pre_fader: false,
            enabled: true,
        };
        assert_eq!(send.id, 1);
        assert_eq!(send.level, 100);
        assert!(!send.pre_fader);
        assert!(send.enabled);
    }

    #[test]
    fn test_send_external_destination() {
        let send = Send {
            id: 2,
            destination: SendDestination::ExternalOutput("MIDI Out".to_string()),
            level: 80,
            pre_fader: true,
            enabled: true,
        };
        assert_eq!(send.id, 2);
        assert!(send.pre_fader);
    }

    // ========================================
    // BUS TESTS
    // ========================================

    #[test]
    fn test_bus_creation() {
        let bus = Bus {
            id: 1,
            name: "Reverb Bus".to_string(),
            volume: 100,
            pan: 64,
            muted: false,
            solo: false,
            effects: vec![],
            vu_level: (0.0, 0.0),
            peak_level: (-60.0, -60.0),
        };
        assert_eq!(bus.id, 1);
        assert_eq!(bus.name, "Reverb Bus");
        assert_eq!(bus.volume, 100);
        assert_eq!(bus.pan, 64);
        assert!(!bus.muted);
        assert!(bus.effects.is_empty());
    }

    // ========================================
    // MONITORING MODE TESTS
    // ========================================

    #[test]
    fn test_monitoring_mode_default() {
        let mode = MonitoringMode::default();
        assert_eq!(mode, MonitoringMode::Off);
    }

    #[test]
    fn test_monitoring_mode_serialization() {
        let modes = [MonitoringMode::Off, MonitoringMode::Input, MonitoringMode::Auto];
        for mode in modes {
            let json = serde_json::to_string(&mode).unwrap();
            let deserialized: MonitoringMode = serde_json::from_str(&json).unwrap();
            assert_eq!(mode, deserialized);
        }
    }

    // ========================================
    // PLUGIN TYPE TESTS
    // ========================================

    #[test]
    fn test_plugin_type_serialization() {
        let types = [
            PluginType::BuiltIn,
            PluginType::VST2,
            PluginType::VST3,
            PluginType::AU,
            PluginType::CLAP,
            PluginType::LV2,
        ];
        for plugin_type in types {
            let json = serde_json::to_string(&plugin_type).unwrap();
            let deserialized: PluginType = serde_json::from_str(&json).unwrap();
            assert_eq!(plugin_type, deserialized);
        }
    }

    // ========================================
    // PLUGIN CATEGORY TESTS
    // ========================================

    #[test]
    fn test_plugin_category_serialization() {
        let categories = [
            PluginCategory::Effect,
            PluginCategory::Instrument,
            PluginCategory::Analyzer,
            PluginCategory::Utility,
            PluginCategory::Unknown,
        ];
        for category in categories {
            let json = serde_json::to_string(&category).unwrap();
            let deserialized: PluginCategory = serde_json::from_str(&json).unwrap();
            assert_eq!(category, deserialized);
        }
    }

    // ========================================
    // PLUGIN INFO TESTS
    // ========================================

    #[test]
    fn test_plugin_info_creation() {
        let plugin = PluginInfo {
            id: "test:plugin".to_string(),
            name: "Test Plugin".to_string(),
            vendor: "Test Vendor".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::BuiltIn,
            category: PluginCategory::Effect,
            path: None,
            is_available: true,
            num_inputs: 2,
            num_outputs: 2,
            accepts_midi: false,
            description: Some("A test plugin".to_string()),
            tags: vec!["test".to_string()],
        };

        assert_eq!(plugin.id, "test:plugin");
        assert_eq!(plugin.name, "Test Plugin");
        assert!(plugin.is_available);
    }

    // ========================================
    // PRESET INFO TESTS
    // ========================================

    #[test]
    fn test_preset_info_creation() {
        let preset = PresetInfo {
            id: 1,
            name: "Test Preset".to_string(),
            description: Some("A test preset".to_string()),
            preset_type: "channel".to_string(),
            category: Some("test".to_string()),
            tags: vec!["test".to_string()],
            use_count: 0,
            is_favorite: false,
            is_factory: false,
            created_at: "2025-01-01".to_string(),
        };

        assert_eq!(preset.id, 1);
        assert_eq!(preset.name, "Test Preset");
        assert!(!preset.is_factory);
    }

    // ========================================
    // LATENCY REPORT TESTS
    // ========================================

    #[test]
    fn test_latency_report_creation() {
        let report = LatencyReport {
            hardware_latency_ms: 5.0,
            plugin_delay_compensation_ms: 10.0,
            total_latency_ms: 15.0,
            compensation_enabled: true,
            per_track_latency: vec![],
        };

        assert_eq!(report.hardware_latency_ms, 5.0);
        assert_eq!(report.total_latency_ms, 15.0);
        assert!(report.compensation_enabled);
    }

    #[test]
    fn test_track_latency_creation() {
        let track_latency = TrackLatency {
            track_id: 1,
            name: "Track 1".to_string(),
            effect_latency_ms: 5.0,
            routing_latency_ms: 0.25,
            total_latency_ms: 5.25,
        };

        assert_eq!(track_latency.track_id, 1);
        assert_eq!(track_latency.total_latency_ms, 5.25);
    }

    // ========================================
    // MIXER GET PLUGIN LIST TESTS
    // ========================================

    #[test]
    fn test_mixer_get_plugin_list_all() {
        let result = mixer_get_plugin_list(None, None);
        assert!(result.is_ok());
        let plugins = result.unwrap();
        assert!(plugins.len() >= 8); // At least 8 built-in plugins
    }

    #[test]
    fn test_mixer_get_plugin_list_filter_by_type() {
        let result = mixer_get_plugin_list(Some("builtin".to_string()), None);
        assert!(result.is_ok());
        let plugins = result.unwrap();
        assert!(plugins.iter().all(|p| p.plugin_type == PluginType::BuiltIn));
    }

    #[test]
    fn test_mixer_get_plugin_list_filter_by_category() {
        let result = mixer_get_plugin_list(None, Some("effect".to_string()));
        assert!(result.is_ok());
        let plugins = result.unwrap();
        assert!(plugins.iter().all(|p| p.category == PluginCategory::Effect));
    }

    #[test]
    fn test_mixer_get_plugin_list_filter_by_utility() {
        let result = mixer_get_plugin_list(None, Some("utility".to_string()));
        assert!(result.is_ok());
        let plugins = result.unwrap();
        assert!(plugins.iter().all(|p| p.category == PluginCategory::Utility));
    }

    #[test]
    fn test_mixer_get_plugin_list_filter_by_analyzer() {
        let result = mixer_get_plugin_list(None, Some("analyzer".to_string()));
        assert!(result.is_ok());
        let plugins = result.unwrap();
        assert!(plugins.iter().all(|p| p.category == PluginCategory::Analyzer));
    }

    #[test]
    fn test_mixer_get_plugin_list_builtin_effects() {
        let result = mixer_get_plugin_list(Some("builtin".to_string()), Some("effect".to_string()));
        assert!(result.is_ok());
        let plugins = result.unwrap();

        // Should have at least EQ3Band, Compressor, Reverb, Delay, Arpeggiator
        let plugin_names: Vec<&str> = plugins.iter().map(|p| p.name.as_str()).collect();
        assert!(plugin_names.contains(&"EQ3Band"));
        assert!(plugin_names.contains(&"Compressor"));
        assert!(plugin_names.contains(&"Reverb"));
        assert!(plugin_names.contains(&"Delay"));
    }

    // ========================================
    // PRESET COMMAND TESTS
    // ========================================

    #[test]
    fn test_mixer_get_presets_all() {
        let result = mixer_get_presets(None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mixer_get_presets_filter_by_type() {
        let result = mixer_get_presets(Some("channel".to_string()), None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mixer_delete_preset_invalid_id() {
        // ID 0 is treated as factory preset (ID 1-6)
        let result = mixer_delete_preset(0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot delete factory presets"));
    }

    #[test]
    fn test_mixer_delete_preset_negative_id() {
        let result = mixer_delete_preset(-1);
        assert!(result.is_err());
    }

    #[test]
    fn test_mixer_toggle_preset_favorite_invalid_id() {
        let result = mixer_toggle_preset_favorite(0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid preset ID"));
    }

    #[test]
    fn test_mixer_toggle_preset_favorite_valid_id() {
        let result = mixer_toggle_preset_favorite(1);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    // ========================================
    // VELOCITY RANGE TESTS
    // ========================================

    #[test]
    fn test_mixer_channel_velocity_range() {
        let channel = MixerChannel {
            velocity_min: 20,
            velocity_max: 100,
            velocity_scale: 80,
            ..MixerChannel::default()
        };

        assert!(channel.velocity_min < channel.velocity_max);
        assert!(channel.velocity_scale <= 200);
    }

    // ========================================
    // METER DATA TESTS
    // ========================================

    #[test]
    fn test_meter_data_creation() {
        let meter = MeterData {
            track_id: 1,
            vu_left: -12.0,
            vu_right: -10.0,
            peak_left: -6.0,
            peak_right: -5.0,
        };

        assert_eq!(meter.track_id, 1);
        assert!(meter.peak_left >= meter.vu_left);
        assert!(meter.peak_right >= meter.vu_right);
    }

    // ========================================
    // CHANNEL ROUTING TESTS
    // ========================================

    #[test]
    fn test_channel_with_sends() {
        let send = Send {
            id: 1,
            destination: SendDestination::Bus(1),
            level: 100,
            pre_fader: false,
            enabled: true,
        };

        let channel = MixerChannel {
            track_id: 1,
            sends: vec![send],
            output_bus: Some(1),
            ..MixerChannel::default()
        };

        assert_eq!(channel.track_id, 1);
        assert_eq!(channel.output_bus, Some(1));
        assert_eq!(channel.sends.len(), 1);
    }

    #[test]
    fn test_channel_routing_to_master() {
        let channel = MixerChannel {
            track_id: 1,
            output_bus: None, // None = route to master
            ..MixerChannel::default()
        };

        assert!(channel.output_bus.is_none());
    }

    // ========================================
    // PLUGIN SCAN RESULT TESTS
    // ========================================

    #[test]
    fn test_plugin_scan_result_creation() {
        let result = PluginScanResult {
            plugins_found: 10,
            plugins_failed: 2,
            directories_scanned: vec!["/usr/lib/vst".to_string()],
            scan_time_ms: 150,
            errors: vec![],
        };

        assert_eq!(result.plugins_found, 10);
        assert_eq!(result.plugins_failed, 2);
        assert_eq!(result.directories_scanned.len(), 1);
        assert!(result.errors.is_empty());
    }

    // ========================================
    // COMPREHENSIVE STRUCT COVERAGE
    // ========================================

    #[test]
    fn test_mixer_channel_all_fields() {
        let channel = MixerChannel {
            track_id: 1,
            name: "Test Track".to_string(),
            enabled: true,
            midi_channel: 1,
            midi_port_id: Some(1),
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
            vu_level: (0.0, 0.0),
            peak_level: (-60.0, -60.0),
            effects: vec![],
            sends: vec![],
            output_bus: None,
            monitoring_mode: MonitoringMode::Off,
            record_armed: false,
        };

        // Verify all fields are set correctly
        assert_eq!(channel.track_id, 1);
        assert_eq!(channel.name, "Test Track");
        assert!(channel.enabled);
        assert_eq!(channel.midi_channel, 1);
        assert_eq!(channel.velocity, 100);
        assert_eq!(channel.pitch_bend_range, 2);
    }

    // ========================================
    // EDGE CASE TESTS
    // ========================================

    #[test]
    fn test_empty_effect_chain() {
        let channel = MixerChannel::default();
        assert!(channel.effects.is_empty());
    }

    #[test]
    fn test_empty_sends() {
        let channel = MixerChannel::default();
        assert!(channel.sends.is_empty());
    }

    #[test]
    fn test_master_output_routing() {
        let channel = MixerChannel::default();
        assert!(channel.output_bus.is_none()); // None = master output
    }

    // ========================================
    // BOUNDARY VALUE TESTS
    // ========================================

    #[test]
    fn test_volume_boundary_values() {
        let channel = MixerChannel {
            volume: 127, // Max MIDI value
            ..MixerChannel::default()
        };
        assert_eq!(channel.volume, 127);

        let channel = MixerChannel {
            volume: 0, // Min MIDI value
            ..MixerChannel::default()
        };
        assert_eq!(channel.volume, 0);
    }

    #[test]
    fn test_pan_boundary_values() {
        let channel = MixerChannel {
            pan: 0, // Full left
            ..MixerChannel::default()
        };
        assert_eq!(channel.pan, 0);

        let channel = MixerChannel {
            pan: 127, // Full right
            ..MixerChannel::default()
        };
        assert_eq!(channel.pan, 127);

        let channel = MixerChannel {
            pan: 64, // Center
            ..MixerChannel::default()
        };
        assert_eq!(channel.pan, 64);
    }

    #[test]
    fn test_transpose_boundary_values() {
        let channel = MixerChannel {
            transpose: 48, // Max transpose
            ..MixerChannel::default()
        };
        assert_eq!(channel.transpose, 48);

        let channel = MixerChannel {
            transpose: -48, // Min transpose
            ..MixerChannel::default()
        };
        assert_eq!(channel.transpose, -48);
    }

    #[test]
    fn test_octave_shift_boundary_values() {
        let channel = MixerChannel {
            octave_shift: 4, // Max octave shift
            ..MixerChannel::default()
        };
        assert_eq!(channel.octave_shift, 4);

        let channel = MixerChannel {
            octave_shift: -4, // Min octave shift
            ..MixerChannel::default()
        };
        assert_eq!(channel.octave_shift, -4);
    }

    #[test]
    fn test_pitch_bend_boundary_values() {
        let channel = MixerChannel {
            pitch_bend: 8191, // Max pitch bend
            ..MixerChannel::default()
        };
        assert_eq!(channel.pitch_bend, 8191);

        let channel = MixerChannel {
            pitch_bend: -8192, // Min pitch bend
            ..MixerChannel::default()
        };
        assert_eq!(channel.pitch_bend, -8192);
    }

    // ========================================
    // JSON ROUNDTRIP TESTS
    // ========================================

    #[test]
    fn test_mixer_channel_json_roundtrip() {
        let original = MixerChannel {
            track_id: 42,
            name: "Test".to_string(),
            volume: 80,
            pan: 32,
            muted: true,
            solo: false,
            ..MixerChannel::default()
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: MixerChannel = serde_json::from_str(&json).unwrap();

        assert_eq!(original.track_id, restored.track_id);
        assert_eq!(original.name, restored.name);
        assert_eq!(original.volume, restored.volume);
        assert_eq!(original.pan, restored.pan);
        assert_eq!(original.muted, restored.muted);
        assert_eq!(original.solo, restored.solo);
    }

    #[test]
    fn test_effect_slot_json_roundtrip() {
        let mut params = HashMap::new();
        params.insert("gain".to_string(), 6.0f32);

        let original = EffectSlot {
            id: 1,
            name: "EQ".to_string(),
            enabled: true,
            wet_dry: 0.75,
            parameters: params,
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: EffectSlot = serde_json::from_str(&json).unwrap();

        assert_eq!(original.id, restored.id);
        assert_eq!(original.name, restored.name);
        assert_eq!(original.wet_dry, restored.wet_dry);
        assert_eq!(original.parameters.get("gain"), restored.parameters.get("gain"));
    }

    #[test]
    fn test_plugin_info_json_roundtrip() {
        let original = PluginInfo {
            id: "builtin:eq".to_string(),
            name: "EQ3Band".to_string(),
            vendor: "MSC".to_string(),
            version: "1.0".to_string(),
            plugin_type: PluginType::BuiltIn,
            category: PluginCategory::Effect,
            path: None,
            is_available: true,
            num_inputs: 2,
            num_outputs: 2,
            accepts_midi: false,
            description: Some("3-band EQ".to_string()),
            tags: vec!["eq".to_string(), "tone".to_string()],
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: PluginInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(original.id, restored.id);
        assert_eq!(original.name, restored.name);
        assert_eq!(original.plugin_type, restored.plugin_type);
        assert_eq!(original.category, restored.category);
    }
}

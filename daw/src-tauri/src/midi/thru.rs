#![allow(dead_code)]

//! MIDI Thru Module
//!
//! Real-time MIDI pass-through for monitoring and live performance.
//! Supports both "soft thru" (processed through router) and "hard thru" (direct bypass).

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

// =============================================================================
// TYPES AND CONFIGURATION
// =============================================================================

/// MIDI thru mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThruMode {
    /// Direct bypass - minimal latency, no processing
    #[default]
    Hard,
    /// Route through MIDI router - allows filtering/transforms
    Soft,
}

/// Velocity curve for scaling
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum VelocityCurve {
    /// Linear pass-through (no change)
    #[default]
    Linear,
    /// Compress dynamics (soft compression)
    Compress { ratio: f32 },
    /// Expand dynamics
    Expand { ratio: f32 },
    /// Fixed velocity (all notes same velocity)
    Fixed { velocity: u8 },
    /// Scale by percentage (0.0-2.0)
    Scale { factor: f32 },
    /// Custom curve with control points
    Custom { points: Vec<(u8, u8)> },
    /// S-curve for natural feel
    SCurve { amount: f32 },
}

impl VelocityCurve {
    /// Apply velocity curve to a value
    pub fn apply(&self, velocity: u8) -> u8 {
        match self {
            VelocityCurve::Linear => velocity,

            VelocityCurve::Compress { ratio } => {
                // Soft compression towards center
                let normalized = velocity as f32 / 127.0;
                let compressed = 0.5 + (normalized - 0.5) * ratio;
                (compressed * 127.0).clamp(1.0, 127.0) as u8
            },

            VelocityCurve::Expand { ratio } => {
                // Expand dynamics away from center
                let normalized = velocity as f32 / 127.0;
                let expanded = 0.5 + (normalized - 0.5) * ratio;
                (expanded * 127.0).clamp(1.0, 127.0) as u8
            },

            VelocityCurve::Fixed { velocity: fixed } => *fixed,

            VelocityCurve::Scale { factor } => ((velocity as f32) * factor).clamp(1.0, 127.0) as u8,

            VelocityCurve::Custom { points } => {
                // Linear interpolation between control points
                if points.is_empty() {
                    return velocity;
                }
                if points.len() == 1 {
                    return points[0].1;
                }

                // Find surrounding points
                for window in points.windows(2) {
                    let (x1, y1) = window[0];
                    let (x2, y2) = window[1];

                    if velocity >= x1 && velocity <= x2 {
                        // Linear interpolation
                        let t = (velocity - x1) as f32 / (x2 - x1).max(1) as f32;
                        return (y1 as f32 + t * (y2 as f32 - y1 as f32)) as u8;
                    }
                }

                // Outside range - use nearest endpoint
                if velocity < points[0].0 {
                    points[0].1
                } else {
                    points.last().map(|p| p.1).unwrap_or(velocity)
                }
            },

            VelocityCurve::SCurve { amount } => {
                // S-curve using tanh
                let normalized = (velocity as f32 / 127.0 - 0.5) * 2.0; // -1 to 1
                let curved = (normalized * amount).tanh() / amount.tanh();
                let result = (curved + 1.0) / 2.0; // back to 0-1
                (result * 127.0).clamp(1.0, 127.0) as u8
            },
        }
    }
}

/// Channel filter/remapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    /// Input channel (0-15, or None for all channels)
    pub input_channel: Option<u8>,
    /// Output channel (0-15, or None to preserve input channel)
    pub output_channel: Option<u8>,
    /// Whether this channel is enabled
    pub enabled: bool,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            input_channel: None,  // Accept all channels
            output_channel: None, // Preserve channel
            enabled: true,
        }
    }
}

/// Complete MIDI thru configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThruConfig {
    /// Thru mode (Hard/Soft)
    pub mode: ThruMode,
    /// Output device ID
    pub output_device: Option<String>,
    /// Channel configuration
    pub channel: ChannelConfig,
    /// Velocity curve
    pub velocity_curve: VelocityCurve,
    /// Transpose in semitones (-48 to +48)
    pub transpose: i8,
    /// Latency compensation in microseconds
    pub latency_compensation_us: i64,
    /// Pass through control messages (CC, pitch bend, etc.)
    pub pass_control: bool,
    /// Pass through system messages (clock, start, stop)
    pub pass_system: bool,
    /// Pass through program changes
    pub pass_program_change: bool,
    /// Pass through aftertouch
    pub pass_aftertouch: bool,
    /// Pass through pitch bend
    pub pass_pitch_bend: bool,
    /// Note range filter (only pass notes in range)
    pub note_range: Option<(u8, u8)>,
    /// Velocity threshold (ignore notes below this)
    pub velocity_threshold: u8,
}

impl Default for ThruConfig {
    fn default() -> Self {
        Self {
            mode: ThruMode::Hard,
            output_device: None,
            channel: ChannelConfig::default(),
            velocity_curve: VelocityCurve::Linear,
            transpose: 0,
            latency_compensation_us: 0,
            pass_control: true,
            pass_system: true,
            pass_program_change: true,
            pass_aftertouch: true,
            pass_pitch_bend: true,
            note_range: None,
            velocity_threshold: 0,
        }
    }
}

/// Thru route event for output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThruEvent {
    pub output_device: String,
    pub data: Vec<u8>,
    pub timestamp_us: u64,
}

/// MIDI Thru state and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThruStatus {
    pub enabled: bool,
    pub mode: ThruMode,
    pub output_device: Option<String>,
    pub messages_passed: u64,
    pub messages_filtered: u64,
    pub last_message_time: Option<u64>,
    pub current_transpose: i8,
    pub current_channel: Option<u8>,
}

// =============================================================================
// MIDI THRU MANAGER
// =============================================================================

/// Statistics tracking
#[derive(Debug, Default)]
struct ThruStats {
    messages_passed: u64,
    messages_filtered: u64,
    last_message_time: Option<u64>,
}

/// MIDI Thru Manager
///
/// Routes MIDI input to output with optional processing.
pub struct MidiThru {
    /// Whether thru is enabled
    enabled: RwLock<bool>,

    /// Current configuration
    config: RwLock<ThruConfig>,

    /// Statistics
    stats: RwLock<ThruStats>,

    /// Event channel for thru messages (to be sent to output)
    event_tx: broadcast::Sender<ThruEvent>,
}

impl MidiThru {
    /// Create a new MIDI thru manager
    pub fn new() -> (Self, broadcast::Receiver<ThruEvent>) {
        let (event_tx, event_rx) = broadcast::channel(2048);

        (
            Self {
                enabled: RwLock::new(false),
                config: RwLock::new(ThruConfig::default()),
                stats: RwLock::new(ThruStats::default()),
                event_tx,
            },
            event_rx,
        )
    }

    /// Subscribe to thru events
    pub fn subscribe(&self) -> broadcast::Receiver<ThruEvent> {
        self.event_tx.subscribe()
    }

    /// Enable MIDI thru
    pub fn enable(&self) {
        *self.enabled.write() = true;
        info!("MIDI thru enabled");
    }

    /// Disable MIDI thru
    pub fn disable(&self) {
        *self.enabled.write() = false;
        info!("MIDI thru disabled");
    }

    /// Check if thru is enabled
    pub fn is_enabled(&self) -> bool {
        *self.enabled.read()
    }

    /// Set thru mode
    pub fn set_mode(&self, mode: ThruMode) {
        self.config.write().mode = mode;
        info!("MIDI thru mode set to {:?}", mode);
    }

    /// Get current mode
    pub fn get_mode(&self) -> ThruMode {
        self.config.read().mode
    }

    /// Set output device
    pub fn set_output(&self, device_id: Option<String>) {
        let mut config = self.config.write();
        config.output_device = device_id.clone();
        info!("MIDI thru output set to {:?}", device_id);
    }

    /// Set channel configuration
    pub fn set_channel(&self, input: Option<u8>, output: Option<u8>) {
        let mut config = self.config.write();
        config.channel.input_channel = input;
        config.channel.output_channel = output;
        info!("MIDI thru channel: {:?} -> {:?}", input, output);
    }

    /// Set velocity curve
    pub fn set_velocity_curve(&self, curve: VelocityCurve) {
        let mut config = self.config.write();
        config.velocity_curve = curve;
        info!("MIDI thru velocity curve updated");
    }

    /// Set transpose
    pub fn set_transpose(&self, semitones: i8) {
        let clamped = semitones.clamp(-48, 48);
        self.config.write().transpose = clamped;
        info!("MIDI thru transpose set to {} semitones", clamped);
    }

    /// Set latency compensation
    pub fn set_latency_compensation(&self, microseconds: i64) {
        self.config.write().latency_compensation_us = microseconds;
        info!("MIDI thru latency compensation set to {}us", microseconds);
    }

    /// Update full configuration
    pub fn set_config(&self, config: ThruConfig) {
        *self.config.write() = config;
        info!("MIDI thru configuration updated");
    }

    /// Get current configuration
    pub fn get_config(&self) -> ThruConfig {
        self.config.read().clone()
    }

    /// Get current status
    pub fn get_status(&self) -> ThruStatus {
        let config = self.config.read();
        let stats = self.stats.read();

        ThruStatus {
            enabled: *self.enabled.read(),
            mode: config.mode,
            output_device: config.output_device.clone(),
            messages_passed: stats.messages_passed,
            messages_filtered: stats.messages_filtered,
            last_message_time: stats.last_message_time,
            current_transpose: config.transpose,
            current_channel: config.channel.output_channel,
        }
    }

    /// Process incoming MIDI message for thru
    ///
    /// Returns true if message was passed through, false if filtered.
    pub fn process(&self, device_id: &str, timestamp_us: u64, message: &[u8]) -> bool {
        if !*self.enabled.read() {
            return false;
        }

        let config = self.config.read();

        // Check if we have an output device
        let output_device = match &config.output_device {
            Some(dev) => dev.clone(),
            None => {
                self.stats.write().messages_filtered += 1;
                return false;
            },
        };

        // Process message based on type
        let processed = self.process_message(message, &config);

        match processed {
            Some(data) => {
                // Apply latency compensation (positive = delay, negative = early)
                let adjusted_time = if config.latency_compensation_us >= 0 {
                    timestamp_us.saturating_add(config.latency_compensation_us as u64)
                } else {
                    timestamp_us.saturating_sub((-config.latency_compensation_us) as u64)
                };

                let event = ThruEvent { output_device, data, timestamp_us: adjusted_time };

                // Send to output
                if self.event_tx.send(event).is_ok() {
                    let mut stats = self.stats.write();
                    stats.messages_passed += 1;
                    stats.last_message_time = Some(timestamp_us);
                    debug!(
                        "MIDI thru: passed message from {} at {}us",
                        device_id, timestamp_us
                    );
                    true
                } else {
                    warn!("MIDI thru: no receivers for output");
                    false
                }
            },
            None => {
                self.stats.write().messages_filtered += 1;
                false
            },
        }
    }

    /// Process a single MIDI message
    fn process_message(&self, message: &[u8], config: &ThruConfig) -> Option<Vec<u8>> {
        if message.is_empty() {
            return None;
        }

        let status = message[0];

        // Handle system messages (0xF0+)
        if status >= 0xF0 {
            return self.process_system_message(status, message, config);
        }

        // Channel messages
        let channel = status & 0x0F;
        let command = status & 0xF0;

        // Channel filter
        if let Some(input_ch) = config.channel.input_channel {
            if channel != input_ch {
                return None;
            }
        }

        // Determine output channel
        let output_channel = config.channel.output_channel.unwrap_or(channel);

        // Process based on command type
        match command {
            0x80 | 0x90 => {
                // Note Off / Note On
                if message.len() < 3 {
                    return None;
                }
                self.process_note_message(command, output_channel, message, config)
            },

            0xA0 => {
                // Poly Aftertouch
                if !config.pass_aftertouch || message.len() < 3 {
                    return None;
                }
                let mut out = message.to_vec();
                out[0] = 0xA0 | output_channel;
                Some(out)
            },

            0xB0 => {
                // Control Change
                if !config.pass_control || message.len() < 3 {
                    return None;
                }
                let mut out = message.to_vec();
                out[0] = 0xB0 | output_channel;
                Some(out)
            },

            0xC0 => {
                // Program Change
                if !config.pass_program_change || message.len() < 2 {
                    return None;
                }
                let mut out = message.to_vec();
                out[0] = 0xC0 | output_channel;
                Some(out)
            },

            0xD0 => {
                // Channel Aftertouch
                if !config.pass_aftertouch || message.len() < 2 {
                    return None;
                }
                let mut out = message.to_vec();
                out[0] = 0xD0 | output_channel;
                Some(out)
            },

            0xE0 => {
                // Pitch Bend
                if !config.pass_pitch_bend || message.len() < 3 {
                    return None;
                }
                let mut out = message.to_vec();
                out[0] = 0xE0 | output_channel;
                Some(out)
            },

            _ => None,
        }
    }

    /// Process note messages with transpose and velocity curve
    fn process_note_message(
        &self,
        command: u8,
        output_channel: u8,
        message: &[u8],
        config: &ThruConfig,
    ) -> Option<Vec<u8>> {
        let note = message[1];
        let velocity = message[2];

        // Velocity threshold
        if command == 0x90 && velocity > 0 && velocity < config.velocity_threshold {
            return None;
        }

        // Note range filter
        if let Some((min, max)) = config.note_range {
            if note < min || note > max {
                return None;
            }
        }

        // Apply transpose
        let transposed_note = (note as i16 + config.transpose as i16).clamp(0, 127) as u8;

        // Apply velocity curve
        let scaled_velocity = if command == 0x90 && velocity > 0 {
            config.velocity_curve.apply(velocity)
        } else {
            velocity
        };

        Some(vec![
            command | output_channel,
            transposed_note,
            scaled_velocity,
        ])
    }

    /// Process system messages
    fn process_system_message(
        &self,
        status: u8,
        message: &[u8],
        config: &ThruConfig,
    ) -> Option<Vec<u8>> {
        match status {
            // Real-time messages
            0xF8 | 0xFA | 0xFB | 0xFC | 0xFE | 0xFF => {
                if config.pass_system {
                    Some(message.to_vec())
                } else {
                    None
                }
            },

            // SysEx - pass through if enabled
            0xF0 => {
                if config.pass_system {
                    Some(message.to_vec())
                } else {
                    None
                }
            },

            // Other system common messages
            0xF1 | 0xF2 | 0xF3 | 0xF6 => {
                if config.pass_system {
                    Some(message.to_vec())
                } else {
                    None
                }
            },

            _ => None,
        }
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        *self.stats.write() = ThruStats::default();
    }
}

impl Default for MidiThru {
    fn default() -> Self {
        Self::new().0
    }
}

// =============================================================================
// TAURI COMMANDS
// =============================================================================

use tauri::State;

pub struct MidiThruState(pub Arc<MidiThru>);

// SAFETY: MidiThru only contains channels and atomics, which are Send+Sync
unsafe impl Send for MidiThruState {}
unsafe impl Sync for MidiThruState {}

impl Default for MidiThruState {
    fn default() -> Self {
        let (thru, _receiver) = MidiThru::new();
        Self(Arc::new(thru))
    }
}

/// Enable MIDI thru
#[tauri::command]
pub fn enable_midi_thru(state: State<MidiThruState>) {
    state.0.enable();
}

/// Disable MIDI thru
#[tauri::command]
pub fn disable_midi_thru(state: State<MidiThruState>) {
    state.0.disable();
}

/// Set thru output device
#[tauri::command]
pub fn set_thru_output(state: State<MidiThruState>, device_id: Option<String>) {
    state.0.set_output(device_id);
}

/// Set thru channel configuration
#[tauri::command]
pub fn set_thru_channel(
    state: State<MidiThruState>,
    input_channel: Option<u8>,
    output_channel: Option<u8>,
) {
    state.0.set_channel(input_channel, output_channel);
}

/// Set thru mode (hard/soft)
#[tauri::command]
pub fn set_thru_mode(state: State<MidiThruState>, mode: ThruMode) {
    state.0.set_mode(mode);
}

/// Set velocity curve
#[tauri::command]
pub fn set_thru_velocity_curve(state: State<MidiThruState>, curve: VelocityCurve) {
    state.0.set_velocity_curve(curve);
}

/// Set transpose in semitones
#[tauri::command]
pub fn set_thru_transpose(state: State<MidiThruState>, semitones: i8) {
    state.0.set_transpose(semitones);
}

/// Set latency compensation in microseconds
#[tauri::command]
pub fn set_thru_latency_compensation(state: State<MidiThruState>, microseconds: i64) {
    state.0.set_latency_compensation(microseconds);
}

/// Get current thru status
#[tauri::command]
pub fn get_thru_status(state: State<MidiThruState>) -> ThruStatus {
    state.0.get_status()
}

/// Get current thru configuration
#[tauri::command]
pub fn get_thru_config(state: State<MidiThruState>) -> ThruConfig {
    state.0.get_config()
}

/// Set full thru configuration
#[tauri::command]
pub fn set_thru_config(state: State<MidiThruState>, config: ThruConfig) {
    state.0.set_config(config);
}

/// Check if thru is enabled
#[tauri::command]
pub fn is_thru_enabled(state: State<MidiThruState>) -> bool {
    state.0.is_enabled()
}

/// Reset thru statistics
#[tauri::command]
pub fn reset_thru_stats(state: State<MidiThruState>) {
    state.0.reset_stats();
}

/// Process MIDI message through thru (for testing/manual routing)
#[tauri::command]
pub fn process_thru_message(
    state: State<MidiThruState>,
    device_id: String,
    timestamp_us: u64,
    message: Vec<u8>,
) -> bool {
    state.0.process(&device_id, timestamp_us, &message)
}

/// Subscribe to MIDI thru events
#[tauri::command]
pub async fn subscribe_midi_thru(
    app: tauri::AppHandle,
    state: State<'_, MidiThruState>,
) -> Result<(), String> {
    use tauri::Emitter;

    let mut rx = state.0.subscribe();

    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            if app.emit("midi_thru_event", &event).is_err() {
                break;
            }
        }
    });

    Ok(())
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_velocity_linear() {
        let curve = VelocityCurve::Linear;
        assert_eq!(curve.apply(0), 0);
        assert_eq!(curve.apply(64), 64);
        assert_eq!(curve.apply(127), 127);
    }

    #[test]
    fn test_velocity_fixed() {
        let curve = VelocityCurve::Fixed { velocity: 100 };
        assert_eq!(curve.apply(1), 100);
        assert_eq!(curve.apply(64), 100);
        assert_eq!(curve.apply(127), 100);
    }

    #[test]
    fn test_velocity_scale() {
        let curve = VelocityCurve::Scale { factor: 0.5 };
        assert_eq!(curve.apply(100), 50);
        assert_eq!(curve.apply(127), 63);
    }

    #[test]
    fn test_velocity_custom_curve() {
        let curve = VelocityCurve::Custom { points: vec![(0, 0), (64, 100), (127, 127)] };
        assert_eq!(curve.apply(0), 0);
        assert_eq!(curve.apply(64), 100);
        assert_eq!(curve.apply(127), 127);
    }

    #[test]
    fn test_thru_enable_disable() {
        let (thru, _rx) = MidiThru::new();

        assert!(!thru.is_enabled());
        thru.enable();
        assert!(thru.is_enabled());
        thru.disable();
        assert!(!thru.is_enabled());
    }

    #[test]
    fn test_thru_transpose() {
        let (thru, _rx) = MidiThru::new();

        thru.set_transpose(12);
        assert_eq!(thru.get_config().transpose, 12);

        thru.set_transpose(-24);
        assert_eq!(thru.get_config().transpose, -24);

        // Test clamping
        thru.set_transpose(100);
        assert_eq!(thru.get_config().transpose, 48);

        thru.set_transpose(-100);
        assert_eq!(thru.get_config().transpose, -48);
    }

    #[test]
    fn test_thru_mode() {
        let (thru, _rx) = MidiThru::new();

        assert_eq!(thru.get_mode(), ThruMode::Hard);
        thru.set_mode(ThruMode::Soft);
        assert_eq!(thru.get_mode(), ThruMode::Soft);
    }

    #[test]
    fn test_process_note_transpose() {
        let (thru, _rx) = MidiThru::new();
        thru.enable();
        thru.set_output(Some("test_output".into()));
        thru.set_transpose(12);

        // Note on: C4 (60) should become C5 (72)
        let config = thru.get_config();
        let result = thru.process_message(&[0x90, 60, 100], &config);

        assert!(result.is_some());
        let msg = result.unwrap();
        assert_eq!(msg[0], 0x90); // Note on, channel 0
        assert_eq!(msg[1], 72); // C5
        assert_eq!(msg[2], 100); // velocity unchanged
    }

    #[test]
    fn test_process_channel_remap() {
        let (thru, _rx) = MidiThru::new();
        thru.enable();
        thru.set_output(Some("test_output".into()));
        thru.set_channel(Some(0), Some(9)); // Channel 1 -> 10

        let config = thru.get_config();

        // Note on channel 0 should be remapped to channel 9
        let result = thru.process_message(&[0x90, 60, 100], &config);
        assert!(result.is_some());
        let msg = result.unwrap();
        assert_eq!(msg[0], 0x99); // Note on, channel 9

        // Note on channel 1 should be filtered out
        let result2 = thru.process_message(&[0x91, 60, 100], &config);
        assert!(result2.is_none());
    }

    #[test]
    fn test_process_velocity_threshold() {
        let (thru, _rx) = MidiThru::new();
        thru.enable();
        thru.set_output(Some("test_output".into()));

        let mut config = thru.get_config();
        config.velocity_threshold = 20;

        // Velocity 10 should be filtered
        let result = thru.process_message(&[0x90, 60, 10], &config);
        assert!(result.is_none());

        // Velocity 50 should pass
        let result2 = thru.process_message(&[0x90, 60, 50], &config);
        assert!(result2.is_some());
    }

    #[test]
    fn test_process_note_range() {
        let (thru, _rx) = MidiThru::new();
        thru.enable();
        thru.set_output(Some("test_output".into()));

        let mut config = thru.get_config();
        config.note_range = Some((48, 72)); // C3 to C5

        // Note in range should pass
        let result = thru.process_message(&[0x90, 60, 100], &config);
        assert!(result.is_some());

        // Note below range should be filtered
        let result2 = thru.process_message(&[0x90, 36, 100], &config);
        assert!(result2.is_none());

        // Note above range should be filtered
        let result3 = thru.process_message(&[0x90, 84, 100], &config);
        assert!(result3.is_none());
    }

    #[test]
    fn test_process_cc_pass() {
        let (thru, _rx) = MidiThru::new();
        thru.enable();
        thru.set_output(Some("test_output".into()));

        let config = thru.get_config();

        // CC should pass through
        let result = thru.process_message(&[0xB0, 7, 100], &config);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), vec![0xB0, 7, 100]);
    }

    #[test]
    fn test_process_cc_filter() {
        let (thru, _rx) = MidiThru::new();
        thru.enable();
        thru.set_output(Some("test_output".into()));

        let mut config = thru.get_config();
        config.pass_control = false;

        // CC should be filtered
        let result = thru.process_message(&[0xB0, 7, 100], &config);
        assert!(result.is_none());
    }

    #[test]
    fn test_process_clock_pass() {
        let (thru, _rx) = MidiThru::new();
        thru.enable();
        thru.set_output(Some("test_output".into()));

        let config = thru.get_config();

        // Clock should pass through
        let result = thru.process_message(&[0xF8], &config);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), vec![0xF8]);

        // Start should pass through
        let result2 = thru.process_message(&[0xFA], &config);
        assert!(result2.is_some());
    }

    #[test]
    fn test_process_clock_filter() {
        let (thru, _rx) = MidiThru::new();
        thru.enable();
        thru.set_output(Some("test_output".into()));

        let mut config = thru.get_config();
        config.pass_system = false;

        // Clock should be filtered
        let result = thru.process_message(&[0xF8], &config);
        assert!(result.is_none());
    }
}

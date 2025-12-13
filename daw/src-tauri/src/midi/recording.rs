#![allow(dead_code)]

//! MIDI Recording Module
//!
//! Real-time MIDI recording with microsecond timestamps and quantization.
//! Supports overdub/replace modes and punch in/out at bar/beat positions.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use super::input::MidiInputMessage;

// =============================================================================
// TYPES
// =============================================================================

/// Quantize grid values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum QuantizeValue {
    /// No quantization - preserve exact timing
    #[default]
    Off,
    /// Quarter note (1/4)
    Quarter,
    /// Eighth note (1/8)
    Eighth,
    /// Sixteenth note (1/16)
    Sixteenth,
    /// Thirty-second note (1/32)
    ThirtySecond,
    /// Quarter note triplet (1/4T)
    QuarterTriplet,
    /// Eighth note triplet (1/8T)
    EighthTriplet,
    /// Sixteenth note triplet (1/16T)
    SixteenthTriplet,
}

impl QuantizeValue {
    /// Get the duration of one grid unit in ticks (assuming 480 PPQ)
    pub fn ticks_per_grid(&self, ppq: u32) -> Option<u32> {
        match self {
            QuantizeValue::Off => None,
            QuantizeValue::Quarter => Some(ppq),
            QuantizeValue::Eighth => Some(ppq / 2),
            QuantizeValue::Sixteenth => Some(ppq / 4),
            QuantizeValue::ThirtySecond => Some(ppq / 8),
            QuantizeValue::QuarterTriplet => Some(ppq * 2 / 3),
            QuantizeValue::EighthTriplet => Some(ppq / 3),
            QuantizeValue::SixteenthTriplet => Some(ppq / 6),
        }
    }

    /// Quantize a tick position to the nearest grid
    pub fn quantize(&self, tick: u64, ppq: u32) -> u64 {
        match self.ticks_per_grid(ppq) {
            Some(grid) => {
                let grid = grid as u64;
                let remainder = tick % grid;
                if remainder >= grid / 2 {
                    tick + (grid - remainder)
                } else {
                    tick - remainder
                }
            },
            None => tick,
        }
    }
}

/// Recording mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecordMode {
    /// Layer new events on top of existing
    #[default]
    Overdub,
    /// Replace existing events in the recording range
    Replace,
}

/// Punch point (bar:beat:tick)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PunchPoint {
    pub bar: u32,
    pub beat: u32,
    pub tick: u32,
}

impl PunchPoint {
    pub fn new(bar: u32, beat: u32, tick: u32) -> Self {
        Self { bar, beat, tick }
    }

    /// Convert to absolute tick position
    pub fn to_ticks(self, ppq: u32, beats_per_bar: u32) -> u64 {
        let ticks_per_bar = ppq as u64 * beats_per_bar as u64;
        let ticks_per_beat = ppq as u64;

        (self.bar as u64 - 1) * ticks_per_bar
            + (self.beat as u64 - 1) * ticks_per_beat
            + self.tick as u64
    }
}

/// A recorded MIDI event with timing info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedEvent {
    /// Absolute timestamp in microseconds from recording start
    pub timestamp_us: u64,
    /// Tick position (calculated from tempo)
    pub tick: u64,
    /// Quantized tick position (if quantization enabled)
    pub quantized_tick: u64,
    /// The original MIDI input message
    pub message: MidiInputMessage,
    /// Whether this event was quantized
    pub was_quantized: bool,
}

/// Recording state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordingState {
    /// Not recording
    Stopped,
    /// Waiting for punch-in point
    Armed,
    /// Actively recording
    Recording,
}

/// Recording status for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingStatus {
    pub state: RecordingState,
    pub mode: RecordMode,
    pub quantize: QuantizeValue,
    pub punch_in: Option<PunchPoint>,
    pub punch_out: Option<PunchPoint>,
    pub event_count: usize,
    pub duration_us: u64,
    pub current_tick: u64,
}

// =============================================================================
// MIDI RECORDER
// =============================================================================

/// MIDI Recording Manager
///
/// Records incoming MIDI events with precise timing and optional quantization.
pub struct MidiRecorder {
    /// Current recording state
    state: RwLock<RecordingState>,

    /// Record mode (overdub/replace)
    mode: RwLock<RecordMode>,

    /// Quantize setting
    quantize: RwLock<QuantizeValue>,

    /// Punch-in point (optional)
    punch_in: RwLock<Option<PunchPoint>>,

    /// Punch-out point (optional)
    punch_out: RwLock<Option<PunchPoint>>,

    /// Recorded events buffer
    events: RwLock<VecDeque<RecordedEvent>>,

    /// Pre-existing events (for overdub mode)
    existing_events: RwLock<Vec<RecordedEvent>>,

    /// Recording start time in microseconds
    start_time_us: RwLock<u64>,

    /// Current position in ticks
    current_tick: RwLock<u64>,

    /// Tempo in BPM (for tick calculations)
    tempo_bpm: RwLock<f64>,

    /// Pulses per quarter note
    ppq: RwLock<u32>,

    /// Beats per bar (time signature numerator)
    beats_per_bar: RwLock<u32>,

    /// Maximum buffer size (prevent memory issues)
    max_events: usize,

    /// Receiver for incoming MIDI messages
    rx: RwLock<Option<mpsc::UnboundedReceiver<MidiInputMessage>>>,
}

impl MidiRecorder {
    /// Create a new MIDI recorder
    pub fn new() -> Self {
        Self {
            state: RwLock::new(RecordingState::Stopped),
            mode: RwLock::new(RecordMode::Overdub),
            quantize: RwLock::new(QuantizeValue::Off),
            punch_in: RwLock::new(None),
            punch_out: RwLock::new(None),
            events: RwLock::new(VecDeque::new()),
            existing_events: RwLock::new(Vec::new()),
            start_time_us: RwLock::new(0),
            current_tick: RwLock::new(0),
            tempo_bpm: RwLock::new(120.0),
            ppq: RwLock::new(480),
            beats_per_bar: RwLock::new(4),
            max_events: 100_000,
            rx: RwLock::new(None),
        }
    }

    /// Create recording channel and return sender for input system
    pub fn create_channel(&self) -> mpsc::UnboundedSender<MidiInputMessage> {
        let (tx, rx) = mpsc::unbounded_channel();
        *self.rx.write() = Some(rx);
        tx
    }

    /// Set tempo (BPM)
    pub fn set_tempo(&self, bpm: f64) {
        *self.tempo_bpm.write() = bpm.max(20.0).min(300.0);
        debug!("Recording tempo set to {} BPM", bpm);
    }

    /// Set PPQ (pulses per quarter note)
    pub fn set_ppq(&self, ppq: u32) {
        *self.ppq.write() = ppq.max(24).min(960);
    }

    /// Set time signature (beats per bar)
    pub fn set_time_signature(&self, beats_per_bar: u32) {
        *self.beats_per_bar.write() = beats_per_bar.max(1).min(16);
    }

    /// Set quantize value
    pub fn set_quantize(&self, value: QuantizeValue) {
        *self.quantize.write() = value;
        info!("Quantize set to {:?}", value);
    }

    /// Get current quantize value
    pub fn get_quantize(&self) -> QuantizeValue {
        *self.quantize.read()
    }

    /// Set record mode
    pub fn set_mode(&self, mode: RecordMode) {
        *self.mode.write() = mode;
        info!("Record mode set to {:?}", mode);
    }

    /// Get current record mode
    pub fn get_mode(&self) -> RecordMode {
        *self.mode.read()
    }

    /// Set punch-in point
    pub fn set_punch_in(&self, point: PunchPoint) {
        *self.punch_in.write() = Some(point);
        info!(
            "Punch-in set to {}:{}:{}",
            point.bar, point.beat, point.tick
        );
    }

    /// Set punch-out point
    pub fn set_punch_out(&self, point: PunchPoint) {
        *self.punch_out.write() = Some(point);
        info!(
            "Punch-out set to {}:{}:{}",
            point.bar, point.beat, point.tick
        );
    }

    /// Clear punch points
    pub fn clear_punch_points(&self) {
        *self.punch_in.write() = None;
        *self.punch_out.write() = None;
        info!("Punch points cleared");
    }

    /// Get punch-in point
    pub fn get_punch_in(&self) -> Option<PunchPoint> {
        *self.punch_in.read()
    }

    /// Get punch-out point
    pub fn get_punch_out(&self) -> Option<PunchPoint> {
        *self.punch_out.read()
    }

    /// Update current playback position
    pub fn update_position(&self, tick: u64) {
        *self.current_tick.write() = tick;

        let state = *self.state.read();
        let ppq = *self.ppq.read();
        let beats_per_bar = *self.beats_per_bar.read();

        // Check punch-in
        if state == RecordingState::Armed {
            if let Some(punch_in) = *self.punch_in.read() {
                let punch_tick = punch_in.to_ticks(ppq, beats_per_bar);
                if tick >= punch_tick {
                    *self.state.write() = RecordingState::Recording;
                    info!("Punch-in triggered at tick {}", tick);
                }
            }
        }

        // Check punch-out
        if state == RecordingState::Recording {
            if let Some(punch_out) = *self.punch_out.read() {
                let punch_tick = punch_out.to_ticks(ppq, beats_per_bar);
                if tick >= punch_tick {
                    *self.state.write() = RecordingState::Stopped;
                    info!("Punch-out triggered at tick {}", tick);
                }
            }
        }
    }

    /// Start recording
    pub fn start(&self, start_time_us: u64) -> Result<(), String> {
        let current_state = *self.state.read();

        if current_state == RecordingState::Recording {
            return Err("Already recording".into());
        }

        // Check if we should arm for punch-in or start immediately
        if self.punch_in.read().is_some() {
            *self.state.write() = RecordingState::Armed;
            info!("Recording armed, waiting for punch-in");
        } else {
            *self.state.write() = RecordingState::Recording;
            info!("Recording started");
        }

        *self.start_time_us.write() = start_time_us;

        // In replace mode, clear existing events
        if *self.mode.read() == RecordMode::Replace {
            self.events.write().clear();
        }

        Ok(())
    }

    /// Stop recording
    pub fn stop(&self) -> Vec<RecordedEvent> {
        let was_recording = *self.state.read() != RecordingState::Stopped;
        *self.state.write() = RecordingState::Stopped;

        if was_recording {
            info!(
                "Recording stopped, {} events captured",
                self.events.read().len()
            );
        }

        self.get_events()
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        *self.state.read() == RecordingState::Recording
    }

    /// Get current state
    pub fn get_state(&self) -> RecordingState {
        *self.state.read()
    }

    /// Record an incoming MIDI message
    pub fn record_message(&self, msg: MidiInputMessage, current_time_us: u64) {
        let state = *self.state.read();
        if state != RecordingState::Recording {
            return;
        }

        let events_len = self.events.read().len();
        if events_len >= self.max_events {
            warn!("Recording buffer full, dropping event");
            return;
        }

        let start_time = *self.start_time_us.read();
        let tempo = *self.tempo_bpm.read();
        let ppq = *self.ppq.read();
        let quantize = *self.quantize.read();

        // Calculate relative timestamp
        let relative_us = current_time_us.saturating_sub(start_time);

        // Convert microseconds to ticks
        // ticks = (time_us * tempo_bpm * ppq) / (60 * 1_000_000)
        let tick = (relative_us as f64 * tempo * ppq as f64) / 60_000_000.0;
        let tick = tick.round() as u64;

        // Apply quantization
        let quantized_tick = quantize.quantize(tick, ppq);
        let was_quantized = quantized_tick != tick;

        let event = RecordedEvent {
            timestamp_us: relative_us,
            tick,
            quantized_tick,
            message: msg,
            was_quantized,
        };

        self.events.write().push_back(event);
    }

    /// Process incoming messages (call from async task)
    pub async fn process_messages(&self) {
        let mut rx = self.rx.write();
        if let Some(ref mut receiver) = *rx {
            while let Ok(msg) = receiver.try_recv() {
                let timestamp = msg.timestamp_us;
                self.record_message(msg, timestamp);
            }
        }
    }

    /// Get all recorded events
    pub fn get_events(&self) -> Vec<RecordedEvent> {
        let recorded: Vec<_> = self.events.read().iter().cloned().collect();

        // In overdub mode, merge with existing events
        if *self.mode.read() == RecordMode::Overdub {
            let existing = self.existing_events.read().clone();
            let mut all_events = existing;
            all_events.extend(recorded);
            // Sort by tick position
            all_events.sort_by_key(|e| e.quantized_tick);
            all_events
        } else {
            recorded
        }
    }

    /// Get event count
    pub fn event_count(&self) -> usize {
        self.events.read().len()
    }

    /// Clear recorded events
    pub fn clear(&self) {
        self.events.write().clear();
        self.existing_events.write().clear();
        info!("Recording buffer cleared");
    }

    /// Set existing events (for overdub mode)
    pub fn set_existing_events(&self, events: Vec<RecordedEvent>) {
        *self.existing_events.write() = events;
    }

    /// Get recording status
    pub fn get_status(&self) -> RecordingStatus {
        let _start = *self.start_time_us.read();
        let events = self.events.read();

        let duration_us = if let Some(last) = events.back() {
            last.timestamp_us
        } else {
            0
        };

        RecordingStatus {
            state: *self.state.read(),
            mode: *self.mode.read(),
            quantize: *self.quantize.read(),
            punch_in: *self.punch_in.read(),
            punch_out: *self.punch_out.read(),
            event_count: events.len(),
            duration_us,
            current_tick: *self.current_tick.read(),
        }
    }
}

impl Default for MidiRecorder {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// TAURI COMMANDS
// =============================================================================

use tauri::State;

/// Recording state wrapper for Tauri
pub struct MidiRecordingState(pub Arc<MidiRecorder>);

// SAFETY: MidiRecorder only contains channels and atomics, which are Send+Sync
unsafe impl Send for MidiRecordingState {}
unsafe impl Sync for MidiRecordingState {}

impl Default for MidiRecordingState {
    fn default() -> Self {
        Self(Arc::new(MidiRecorder::new()))
    }
}

/// Start MIDI recording
#[tauri::command]
pub fn start_midi_recording(
    state: State<MidiRecordingState>,
    start_time_us: Option<u64>,
) -> Result<(), String> {
    let time = start_time_us.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0)
    });
    state.0.start(time)
}

/// Stop MIDI recording and return events
#[tauri::command]
pub fn stop_midi_recording(state: State<MidiRecordingState>) -> Vec<RecordedEvent> {
    state.0.stop()
}

/// Set quantize value
#[tauri::command]
pub fn set_quantize_value(state: State<MidiRecordingState>, value: QuantizeValue) {
    state.0.set_quantize(value);
}

/// Get current quantize value
#[tauri::command]
pub fn get_quantize_value(state: State<MidiRecordingState>) -> QuantizeValue {
    state.0.get_quantize()
}

/// Set record mode
#[tauri::command]
pub fn set_record_mode(state: State<MidiRecordingState>, mode: RecordMode) {
    state.0.set_mode(mode);
}

/// Get current record mode
#[tauri::command]
pub fn get_record_mode(state: State<MidiRecordingState>) -> RecordMode {
    state.0.get_mode()
}

/// Set punch-in point
#[tauri::command]
pub fn set_punch_in(state: State<MidiRecordingState>, bar: u32, beat: u32, tick: u32) {
    state.0.set_punch_in(PunchPoint::new(bar, beat, tick));
}

/// Set punch-out point
#[tauri::command]
pub fn set_punch_out(state: State<MidiRecordingState>, bar: u32, beat: u32, tick: u32) {
    state.0.set_punch_out(PunchPoint::new(bar, beat, tick));
}

/// Clear punch in/out points
#[tauri::command]
pub fn clear_punch_points(state: State<MidiRecordingState>) {
    state.0.clear_punch_points();
}

/// Get all recorded events
#[tauri::command]
pub fn get_recorded_events(state: State<MidiRecordingState>) -> Vec<RecordedEvent> {
    state.0.get_events()
}

/// Clear recording buffer
#[tauri::command]
pub fn clear_recording(state: State<MidiRecordingState>) {
    state.0.clear();
}

/// Get recording status
#[tauri::command]
pub fn get_recording_status(state: State<MidiRecordingState>) -> RecordingStatus {
    state.0.get_status()
}

/// Set tempo for recording
#[tauri::command]
pub fn set_recording_tempo(state: State<MidiRecordingState>, bpm: f64) {
    state.0.set_tempo(bpm);
}

/// Set PPQ for recording
#[tauri::command]
pub fn set_recording_ppq(state: State<MidiRecordingState>, ppq: u32) {
    state.0.set_ppq(ppq);
}

/// Set time signature for recording
#[tauri::command]
pub fn set_recording_time_signature(state: State<MidiRecordingState>, beats_per_bar: u32) {
    state.0.set_time_signature(beats_per_bar);
}

/// Update playback position (for punch in/out)
#[tauri::command]
pub fn update_recording_position(state: State<MidiRecordingState>, tick: u64) {
    state.0.update_position(tick);
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantize_value_ticks() {
        let ppq = 480;

        assert_eq!(QuantizeValue::Off.ticks_per_grid(ppq), None);
        assert_eq!(QuantizeValue::Quarter.ticks_per_grid(ppq), Some(480));
        assert_eq!(QuantizeValue::Eighth.ticks_per_grid(ppq), Some(240));
        assert_eq!(QuantizeValue::Sixteenth.ticks_per_grid(ppq), Some(120));
        assert_eq!(QuantizeValue::ThirtySecond.ticks_per_grid(ppq), Some(60));
        assert_eq!(QuantizeValue::EighthTriplet.ticks_per_grid(ppq), Some(160));
    }

    #[test]
    fn test_quantize_snap() {
        let ppq = 480;

        // Sixteenth note grid (120 ticks)
        let q = QuantizeValue::Sixteenth;

        // Exact grid position
        assert_eq!(q.quantize(0, ppq), 0);
        assert_eq!(q.quantize(120, ppq), 120);

        // Should snap forward (past midpoint)
        assert_eq!(q.quantize(61, ppq), 120);
        assert_eq!(q.quantize(100, ppq), 120);

        // Should snap backward (before midpoint)
        assert_eq!(q.quantize(59, ppq), 0);
        assert_eq!(q.quantize(30, ppq), 0);
    }

    #[test]
    fn test_punch_point_to_ticks() {
        let ppq = 480;
        let beats_per_bar = 4;

        // Bar 1, beat 1, tick 0 = 0
        let p = PunchPoint::new(1, 1, 0);
        assert_eq!(p.to_ticks(ppq, beats_per_bar), 0);

        // Bar 1, beat 2, tick 0 = 480
        let p = PunchPoint::new(1, 2, 0);
        assert_eq!(p.to_ticks(ppq, beats_per_bar), 480);

        // Bar 2, beat 1, tick 0 = 1920 (4 beats * 480 ppq)
        let p = PunchPoint::new(2, 1, 0);
        assert_eq!(p.to_ticks(ppq, beats_per_bar), 1920);

        // Bar 2, beat 3, tick 240 = 1920 + 960 + 240 = 3120
        let p = PunchPoint::new(2, 3, 240);
        assert_eq!(p.to_ticks(ppq, beats_per_bar), 3120);
    }

    #[test]
    fn test_recorder_state() {
        let recorder = MidiRecorder::new();

        assert_eq!(recorder.get_state(), RecordingState::Stopped);
        assert!(!recorder.is_recording());

        recorder.start(0).unwrap();
        assert_eq!(recorder.get_state(), RecordingState::Recording);
        assert!(recorder.is_recording());

        recorder.stop();
        assert_eq!(recorder.get_state(), RecordingState::Stopped);
        assert!(!recorder.is_recording());
    }

    #[test]
    fn test_recorder_punch_armed() {
        let recorder = MidiRecorder::new();

        // Set punch-in point
        recorder.set_punch_in(PunchPoint::new(2, 1, 0));

        recorder.start(0).unwrap();
        assert_eq!(recorder.get_state(), RecordingState::Armed);

        // Update position past punch-in
        recorder.update_position(1920); // Bar 2
        assert_eq!(recorder.get_state(), RecordingState::Recording);
    }

    #[test]
    fn test_recorder_punch_out() {
        let recorder = MidiRecorder::new();

        // Set punch-out at bar 3
        recorder.set_punch_out(PunchPoint::new(3, 1, 0));

        recorder.start(0).unwrap();
        assert_eq!(recorder.get_state(), RecordingState::Recording);

        // Update position past punch-out
        recorder.update_position(3840); // Bar 3
        assert_eq!(recorder.get_state(), RecordingState::Stopped);
    }

    #[test]
    fn test_recorder_modes() {
        let recorder = MidiRecorder::new();

        assert_eq!(recorder.get_mode(), RecordMode::Overdub);

        recorder.set_mode(RecordMode::Replace);
        assert_eq!(recorder.get_mode(), RecordMode::Replace);
    }

    #[test]
    fn test_recorder_quantize() {
        let recorder = MidiRecorder::new();

        assert_eq!(recorder.get_quantize(), QuantizeValue::Off);

        recorder.set_quantize(QuantizeValue::Sixteenth);
        assert_eq!(recorder.get_quantize(), QuantizeValue::Sixteenth);
    }

    #[test]
    fn test_recorder_clear() {
        let recorder = MidiRecorder::new();

        recorder.start(0).unwrap();
        // Would normally record events here
        recorder.clear();

        assert_eq!(recorder.event_count(), 0);
    }

    #[test]
    fn test_triplet_quantize() {
        let ppq = 480;
        let q = QuantizeValue::EighthTriplet;

        // Eighth triplet grid is 160 ticks
        assert_eq!(q.ticks_per_grid(ppq), Some(160));

        // Should snap to triplet grid
        assert_eq!(q.quantize(0, ppq), 0);
        assert_eq!(q.quantize(160, ppq), 160);
        assert_eq!(q.quantize(80, ppq), 160); // Past midpoint
        assert_eq!(q.quantize(79, ppq), 0); // Before midpoint
    }
}

# Terminal 6: MIDI Clock/Sync Implementation Guide

## Overview
This guide covers implementing MIDI clock master/slave synchronization for tempo-synced playback and recording across MIDI devices.

---

## Part 1: Rust Backend - MIDI Clock Module

### 1.1 Module Structure

**File: `daw/src-tauri/src/midi_clock/mod.rs`**
```rust
pub mod clock;
pub mod sync;
pub mod transport;
pub mod messages;

pub use clock::{MidiClock, ClockState, ClockConfig};
pub use sync::{SyncManager, SyncMode, SyncStatus};
pub use transport::{Transport, TransportState, TimeSignature};
pub use messages::{MidiClockMessage, MidiTimecode};
```

### 1.2 MIDI Clock Messages

**File: `daw/src-tauri/src/midi_clock/messages.rs`**
```rust
use serde::{Deserialize, Serialize};

/// MIDI Real-Time Messages for clock synchronization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiClockMessage {
    /// Timing Clock (0xF8) - sent 24 times per quarter note
    TimingClock,
    /// Start (0xFA) - start playback from beginning
    Start,
    /// Continue (0xFB) - continue from current position
    Continue,
    /// Stop (0xFC) - stop playback
    Stop,
    /// Active Sensing (0xFE) - keepalive, sent every 300ms
    ActiveSensing,
    /// System Reset (0xFF) - reset to power-on state
    SystemReset,
}

impl MidiClockMessage {
    pub fn to_byte(self) -> u8 {
        match self {
            Self::TimingClock => 0xF8,
            Self::Start => 0xFA,
            Self::Continue => 0xFB,
            Self::Stop => 0xFC,
            Self::ActiveSensing => 0xFE,
            Self::SystemReset => 0xFF,
        }
    }

    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0xF8 => Some(Self::TimingClock),
            0xFA => Some(Self::Start),
            0xFB => Some(Self::Continue),
            0xFC => Some(Self::Stop),
            0xFE => Some(Self::ActiveSensing),
            0xFF => Some(Self::SystemReset),
            _ => None,
        }
    }
}

/// MIDI Time Code (MTC) Quarter Frame message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MidiTimecode {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub frames: u8,
    pub frame_rate: FrameRate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameRate {
    Fps24,
    Fps25,
    Fps30Drop,
    Fps30,
}

impl FrameRate {
    pub fn frames_per_second(&self) -> f64 {
        match self {
            Self::Fps24 => 24.0,
            Self::Fps25 => 25.0,
            Self::Fps30Drop => 29.97,
            Self::Fps30 => 30.0,
        }
    }

    pub fn to_mtc_type(&self) -> u8 {
        match self {
            Self::Fps24 => 0,
            Self::Fps25 => 1,
            Self::Fps30Drop => 2,
            Self::Fps30 => 3,
        }
    }
}

impl MidiTimecode {
    pub fn new(frame_rate: FrameRate) -> Self {
        Self {
            hours: 0,
            minutes: 0,
            seconds: 0,
            frames: 0,
            frame_rate,
        }
    }

    pub fn from_millis(millis: u64, frame_rate: FrameRate) -> Self {
        let fps = frame_rate.frames_per_second();
        let total_frames = (millis as f64 / 1000.0 * fps) as u64;

        let frames = (total_frames % fps as u64) as u8;
        let total_seconds = total_frames / fps as u64;
        let seconds = (total_seconds % 60) as u8;
        let total_minutes = total_seconds / 60;
        let minutes = (total_minutes % 60) as u8;
        let hours = (total_minutes / 60) as u8;

        Self {
            hours,
            minutes,
            seconds,
            frames,
            frame_rate,
        }
    }

    pub fn to_millis(&self) -> u64 {
        let fps = self.frame_rate.frames_per_second();
        let total_seconds = self.hours as u64 * 3600
            + self.minutes as u64 * 60
            + self.seconds as u64;
        let frame_millis = (self.frames as f64 / fps * 1000.0) as u64;

        total_seconds * 1000 + frame_millis
    }

    /// Generate MTC Quarter Frame messages
    pub fn to_quarter_frames(&self) -> [u8; 8] {
        let frame_type = self.frame_rate.to_mtc_type();
        [
            0x00 | (self.frames & 0x0F),           // Frame number low nibble
            0x10 | ((self.frames >> 4) & 0x01),    // Frame number high nibble
            0x20 | (self.seconds & 0x0F),          // Seconds low nibble
            0x30 | ((self.seconds >> 4) & 0x03),   // Seconds high nibble
            0x40 | (self.minutes & 0x0F),          // Minutes low nibble
            0x50 | ((self.minutes >> 4) & 0x03),   // Minutes high nibble
            0x60 | (self.hours & 0x0F),            // Hours low nibble
            0x70 | ((self.hours >> 4) & 0x01) | (frame_type << 1), // Hours high + type
        ]
    }
}

/// Song Position Pointer (SPP) - position in MIDI beats (1/16th notes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SongPosition {
    pub beats: u16, // 14-bit value (0-16383)
}

impl SongPosition {
    pub fn new(beats: u16) -> Self {
        Self {
            beats: beats.min(16383),
        }
    }

    pub fn from_bars(bars: u16, beats_per_bar: u8) -> Self {
        let total_16ths = bars as u16 * beats_per_bar as u16 * 4;
        Self::new(total_16ths)
    }

    pub fn to_bytes(&self) -> [u8; 2] {
        [
            (self.beats & 0x7F) as u8,        // LSB (7 bits)
            ((self.beats >> 7) & 0x7F) as u8, // MSB (7 bits)
        ]
    }

    pub fn from_bytes(lsb: u8, msb: u8) -> Self {
        Self {
            beats: (lsb as u16 & 0x7F) | ((msb as u16 & 0x7F) << 7),
        }
    }
}
```

### 1.3 Core Clock Implementation

**File: `daw/src-tauri/src/midi_clock/clock.rs`**
```rust
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::interval;
use tracing::{debug, info, warn};

use super::messages::{MidiClockMessage, MidiTimecode, SongPosition};
use super::transport::TimeSignature;

/// MIDI Clock pulses per quarter note (industry standard)
pub const PPQN: u32 = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone)]
pub struct ClockConfig {
    pub bpm: f64,
    pub time_signature: TimeSignature,
    pub send_active_sensing: bool,
    pub pre_roll_bars: u8,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            bpm: 120.0,
            time_signature: TimeSignature::default(),
            send_active_sensing: true,
            pre_roll_bars: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClockTick {
    pub tick: u64,
    pub beat: u32,
    pub bar: u32,
    pub timestamp: Instant,
    pub position_millis: u64,
}

pub struct MidiClock {
    config: Arc<RwLock<ClockConfig>>,
    state: Arc<RwLock<ClockState>>,
    tick_count: Arc<AtomicU64>,
    running: Arc<AtomicBool>,

    // Channels for clock events
    tick_tx: broadcast::Sender<ClockTick>,
    message_tx: mpsc::Sender<MidiClockMessage>,

    // Timing
    last_tick_time: Arc<RwLock<Option<Instant>>>,
    start_time: Arc<RwLock<Option<Instant>>>,
}

impl MidiClock {
    pub fn new() -> (Self, broadcast::Receiver<ClockTick>, mpsc::Receiver<MidiClockMessage>) {
        let (tick_tx, tick_rx) = broadcast::channel(256);
        let (message_tx, message_rx) = mpsc::channel(64);

        let clock = Self {
            config: Arc::new(RwLock::new(ClockConfig::default())),
            state: Arc::new(RwLock::new(ClockState::Stopped)),
            tick_count: Arc::new(AtomicU64::new(0)),
            running: Arc::new(AtomicBool::new(false)),
            tick_tx,
            message_tx,
            last_tick_time: Arc::new(RwLock::new(None)),
            start_time: Arc::new(RwLock::new(None)),
        };

        (clock, tick_rx, message_rx)
    }

    pub async fn set_bpm(&self, bpm: f64) {
        let mut config = self.config.write().await;
        config.bpm = bpm.clamp(20.0, 300.0);
        info!(bpm = config.bpm, "BPM changed");
    }

    pub async fn get_bpm(&self) -> f64 {
        self.config.read().await.bpm
    }

    pub async fn set_time_signature(&self, time_sig: TimeSignature) {
        let mut config = self.config.write().await;
        config.time_signature = time_sig;
    }

    pub async fn get_state(&self) -> ClockState {
        *self.state.read().await
    }

    pub fn get_tick_count(&self) -> u64 {
        self.tick_count.load(Ordering::SeqCst)
    }

    /// Calculate interval between clock ticks based on BPM
    fn tick_interval(bpm: f64) -> Duration {
        // BPM = beats per minute
        // PPQN = 24 ticks per beat
        // Interval = 60 / (BPM * PPQN) seconds
        let seconds_per_tick = 60.0 / (bpm * PPQN as f64);
        Duration::from_secs_f64(seconds_per_tick)
    }

    /// Start the clock (from beginning)
    pub async fn start(&self) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }

        // Reset position
        self.tick_count.store(0, Ordering::SeqCst);
        *self.start_time.write().await = Some(Instant::now());
        *self.state.write().await = ClockState::Playing;

        // Send MIDI Start message
        let _ = self.message_tx.send(MidiClockMessage::Start).await;

        self.run_clock_loop().await;
    }

    /// Continue from current position
    pub async fn continue_playback(&self) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }

        *self.state.write().await = ClockState::Playing;

        // Send MIDI Continue message
        let _ = self.message_tx.send(MidiClockMessage::Continue).await;

        self.run_clock_loop().await;
    }

    /// Stop the clock
    pub async fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        *self.state.write().await = ClockState::Stopped;

        // Send MIDI Stop message
        let _ = self.message_tx.send(MidiClockMessage::Stop).await;
    }

    /// Pause (stop without resetting position)
    pub async fn pause(&self) {
        self.running.store(false, Ordering::SeqCst);
        *self.state.write().await = ClockState::Paused;

        // Send MIDI Stop message
        let _ = self.message_tx.send(MidiClockMessage::Stop).await;
    }

    /// Set position via Song Position Pointer
    pub async fn set_position(&self, position: SongPosition) {
        // Convert MIDI beats (16th notes) to clock ticks
        // 1 MIDI beat = 6 clock ticks (24 PPQN / 4 = 6 ticks per 16th note)
        let ticks = position.beats as u64 * 6;
        self.tick_count.store(ticks, Ordering::SeqCst);
    }

    pub async fn get_position(&self) -> SongPosition {
        let ticks = self.tick_count.load(Ordering::SeqCst);
        let beats = (ticks / 6) as u16;
        SongPosition::new(beats)
    }

    /// Get current position as bar:beat:tick
    pub async fn get_position_bbt(&self) -> (u32, u32, u32) {
        let ticks = self.tick_count.load(Ordering::SeqCst);
        let config = self.config.read().await;

        let ticks_per_beat = PPQN as u64;
        let ticks_per_bar = ticks_per_beat * config.time_signature.numerator as u64;

        let bar = (ticks / ticks_per_bar) as u32;
        let beat = ((ticks % ticks_per_bar) / ticks_per_beat) as u32;
        let tick = (ticks % ticks_per_beat) as u32;

        (bar + 1, beat + 1, tick) // 1-indexed for display
    }

    /// Get current position in milliseconds
    pub async fn get_position_millis(&self) -> u64 {
        let ticks = self.tick_count.load(Ordering::SeqCst);
        let bpm = self.config.read().await.bpm;

        // millis = ticks * (60000 / (BPM * PPQN))
        ((ticks as f64 * 60000.0) / (bpm * PPQN as f64)) as u64
    }

    async fn run_clock_loop(&self) {
        self.running.store(true, Ordering::SeqCst);

        let config = self.config.clone();
        let state = self.state.clone();
        let tick_count = self.tick_count.clone();
        let running = self.running.clone();
        let tick_tx = self.tick_tx.clone();
        let message_tx = self.message_tx.clone();
        let last_tick_time = self.last_tick_time.clone();

        tokio::spawn(async move {
            let mut bpm = config.read().await.bpm;
            let mut tick_interval = interval(Self::tick_interval(bpm));
            tick_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            while running.load(Ordering::SeqCst) {
                tick_interval.tick().await;

                // Check if BPM changed
                let current_bpm = config.read().await.bpm;
                if (current_bpm - bpm).abs() > 0.01 {
                    bpm = current_bpm;
                    tick_interval = interval(Self::tick_interval(bpm));
                    tick_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                }

                // Increment tick
                let tick = tick_count.fetch_add(1, Ordering::SeqCst);
                let now = Instant::now();
                *last_tick_time.write().await = Some(now);

                // Calculate position
                let time_sig = config.read().await.time_signature;
                let ticks_per_beat = PPQN;
                let ticks_per_bar = ticks_per_beat * time_sig.numerator as u32;

                let bar = (tick as u32 / ticks_per_bar);
                let beat = (tick as u32 % ticks_per_bar) / ticks_per_beat;
                let position_millis = ((tick as f64 * 60000.0) / (bpm * PPQN as f64)) as u64;

                // Send clock tick message
                let _ = message_tx.send(MidiClockMessage::TimingClock).await;

                // Broadcast tick event
                let clock_tick = ClockTick {
                    tick,
                    beat,
                    bar,
                    timestamp: now,
                    position_millis,
                };
                let _ = tick_tx.send(clock_tick);
            }
        });
    }

    /// Subscribe to clock ticks
    pub fn subscribe(&self) -> broadcast::Receiver<ClockTick> {
        self.tick_tx.subscribe()
    }
}

impl Default for MidiClock {
    fn default() -> Self {
        Self::new().0
    }
}
```

### 1.4 Sync Manager

**File: `daw/src-tauri/src/midi_clock/sync.rs`**
```rust
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, info, warn};

use super::clock::{ClockTick, MidiClock, PPQN};
use super::messages::{MidiClockMessage, SongPosition};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SyncMode {
    #[default]
    Internal,      // We are the master
    External,      // Sync to external MIDI clock
    MidiTimecode,  // Sync to MTC
}

#[derive(Debug, Clone, Default)]
pub struct SyncStatus {
    pub mode: SyncMode,
    pub is_locked: bool,
    pub external_bpm: Option<f64>,
    pub drift_ms: f64,
    pub clock_source: Option<String>,
    pub last_sync: Option<Instant>,
}

/// BPM detector for external clock
struct BpmDetector {
    tick_times: Vec<Instant>,
    max_samples: usize,
}

impl BpmDetector {
    fn new(max_samples: usize) -> Self {
        Self {
            tick_times: Vec::with_capacity(max_samples),
            max_samples,
        }
    }

    fn add_tick(&mut self, time: Instant) {
        self.tick_times.push(time);
        if self.tick_times.len() > self.max_samples {
            self.tick_times.remove(0);
        }
    }

    fn calculate_bpm(&self) -> Option<f64> {
        if self.tick_times.len() < 2 {
            return None;
        }

        // Calculate average tick interval
        let mut total_duration = Duration::ZERO;
        for i in 1..self.tick_times.len() {
            total_duration += self.tick_times[i].duration_since(self.tick_times[i - 1]);
        }

        let avg_tick_interval = total_duration.as_secs_f64() / (self.tick_times.len() - 1) as f64;

        // BPM = 60 / (tick_interval * PPQN)
        let bpm = 60.0 / (avg_tick_interval * PPQN as f64);

        // Sanity check
        if bpm >= 20.0 && bpm <= 300.0 {
            Some(bpm)
        } else {
            None
        }
    }

    fn reset(&mut self) {
        self.tick_times.clear();
    }
}

pub struct SyncManager {
    clock: Arc<MidiClock>,
    mode: Arc<RwLock<SyncMode>>,
    status: Arc<RwLock<SyncStatus>>,

    // External clock input
    external_message_rx: Option<mpsc::Receiver<MidiClockMessage>>,

    // BPM detection
    bpm_detector: Arc<RwLock<BpmDetector>>,

    // Sync parameters
    lock_threshold_ms: f64,
    max_drift_ms: f64,
}

impl SyncManager {
    pub fn new(clock: Arc<MidiClock>) -> Self {
        Self {
            clock,
            mode: Arc::new(RwLock::new(SyncMode::Internal)),
            status: Arc::new(RwLock::new(SyncStatus::default())),
            external_message_rx: None,
            bpm_detector: Arc::new(RwLock::new(BpmDetector::new(48))), // 2 beats worth
            lock_threshold_ms: 5.0,
            max_drift_ms: 50.0,
        }
    }

    pub fn with_external_input(mut self, rx: mpsc::Receiver<MidiClockMessage>) -> Self {
        self.external_message_rx = Some(rx);
        self
    }

    pub async fn set_mode(&self, mode: SyncMode) {
        *self.mode.write().await = mode;

        // Reset BPM detector when changing modes
        self.bpm_detector.write().await.reset();

        // Update status
        let mut status = self.status.write().await;
        status.mode = mode;
        status.is_locked = false;
        status.external_bpm = None;

        info!(?mode, "Sync mode changed");
    }

    pub async fn get_mode(&self) -> SyncMode {
        *self.mode.read().await
    }

    pub async fn get_status(&self) -> SyncStatus {
        self.status.read().await.clone()
    }

    /// Process incoming external MIDI clock message
    pub async fn process_external_clock(&self, message: MidiClockMessage) {
        let mode = *self.mode.read().await;
        if mode != SyncMode::External {
            return;
        }

        match message {
            MidiClockMessage::TimingClock => {
                self.handle_external_tick().await;
            }
            MidiClockMessage::Start => {
                info!("External clock start received");
                self.clock.start().await;
            }
            MidiClockMessage::Continue => {
                info!("External clock continue received");
                self.clock.continue_playback().await;
            }
            MidiClockMessage::Stop => {
                info!("External clock stop received");
                self.clock.stop().await;
            }
            _ => {}
        }
    }

    async fn handle_external_tick(&self) {
        let now = Instant::now();

        // Add to BPM detector
        let mut detector = self.bpm_detector.write().await;
        detector.add_tick(now);

        // Calculate external BPM
        if let Some(external_bpm) = detector.calculate_bpm() {
            let current_bpm = self.clock.get_bpm().await;
            let bpm_diff = (external_bpm - current_bpm).abs();

            // Update internal clock BPM if significantly different
            if bpm_diff > 0.5 {
                self.clock.set_bpm(external_bpm).await;
                debug!(external_bpm, current_bpm, "Adjusting BPM to match external clock");
            }

            // Update status
            let mut status = self.status.write().await;
            status.external_bpm = Some(external_bpm);
            status.last_sync = Some(now);
            status.is_locked = bpm_diff < 1.0;
            status.drift_ms = bpm_diff * 500.0 / external_bpm; // Approximate drift per beat
        }
    }

    /// Send MIDI clock output (when we are master)
    pub async fn get_clock_output(&self) -> Option<broadcast::Receiver<ClockTick>> {
        let mode = *self.mode.read().await;
        if mode == SyncMode::Internal {
            Some(self.clock.subscribe())
        } else {
            None
        }
    }

    /// Start the sync manager's processing loop
    pub async fn run(&mut self) {
        if let Some(mut rx) = self.external_message_rx.take() {
            let mode = self.mode.clone();
            let bpm_detector = self.bpm_detector.clone();
            let status = self.status.clone();
            let clock = self.clock.clone();

            tokio::spawn(async move {
                while let Some(message) = rx.recv().await {
                    let current_mode = *mode.read().await;
                    if current_mode != SyncMode::External {
                        continue;
                    }

                    match message {
                        MidiClockMessage::TimingClock => {
                            let now = Instant::now();

                            let mut detector = bpm_detector.write().await;
                            detector.add_tick(now);

                            if let Some(external_bpm) = detector.calculate_bpm() {
                                let current_bpm = clock.get_bpm().await;
                                let bpm_diff = (external_bpm - current_bpm).abs();

                                if bpm_diff > 0.5 {
                                    clock.set_bpm(external_bpm).await;
                                }

                                let mut s = status.write().await;
                                s.external_bpm = Some(external_bpm);
                                s.last_sync = Some(now);
                                s.is_locked = bpm_diff < 1.0;
                            }
                        }
                        MidiClockMessage::Start => {
                            clock.start().await;
                        }
                        MidiClockMessage::Continue => {
                            clock.continue_playback().await;
                        }
                        MidiClockMessage::Stop => {
                            clock.stop().await;
                        }
                        _ => {}
                    }
                }
            });
        }
    }
}
```

### 1.5 Transport State

**File: `daw/src-tauri/src/midi_clock/transport.rs`**
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportState {
    Stopped,
    Playing,
    Recording,
    Paused,
}

impl Default for TransportState {
    fn default() -> Self {
        Self::Stopped
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeSignature {
    pub numerator: u8,
    pub denominator: u8,
}

impl Default for TimeSignature {
    fn default() -> Self {
        Self {
            numerator: 4,
            denominator: 4,
        }
    }
}

impl TimeSignature {
    pub fn new(numerator: u8, denominator: u8) -> Self {
        Self { numerator, denominator }
    }

    pub fn beats_per_bar(&self) -> u8 {
        self.numerator
    }

    /// Beat value relative to quarter note
    pub fn beat_value(&self) -> f64 {
        4.0 / self.denominator as f64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transport {
    pub state: TransportState,
    pub bpm: f64,
    pub time_signature: TimeSignature,
    pub position_ticks: u64,
    pub position_millis: u64,
    pub bar: u32,
    pub beat: u32,
    pub tick: u32,
    pub loop_enabled: bool,
    pub loop_start_ticks: u64,
    pub loop_end_ticks: u64,
    pub metronome_enabled: bool,
    pub count_in_enabled: bool,
    pub count_in_bars: u8,
}

impl Default for Transport {
    fn default() -> Self {
        Self {
            state: TransportState::Stopped,
            bpm: 120.0,
            time_signature: TimeSignature::default(),
            position_ticks: 0,
            position_millis: 0,
            bar: 1,
            beat: 1,
            tick: 0,
            loop_enabled: false,
            loop_start_ticks: 0,
            loop_end_ticks: 0,
            metronome_enabled: false,
            count_in_enabled: false,
            count_in_bars: 1,
        }
    }
}
```

---

## Part 2: Tauri Commands

**File: `daw/src-tauri/src/commands/midi_clock.rs`**
```rust
use crate::midi_clock::{
    clock::{ClockState, MidiClock},
    sync::{SyncManager, SyncMode, SyncStatus},
    transport::{TimeSignature, Transport, TransportState},
};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

pub struct MidiClockState {
    pub clock: Arc<MidiClock>,
    pub sync_manager: Arc<RwLock<SyncManager>>,
}

#[tauri::command]
pub async fn get_transport(
    state: State<'_, MidiClockState>,
) -> Result<Transport, String> {
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

#[tauri::command]
pub async fn transport_play(
    state: State<'_, MidiClockState>,
) -> Result<(), String> {
    state.clock.start().await;
    Ok(())
}

#[tauri::command]
pub async fn transport_pause(
    state: State<'_, MidiClockState>,
) -> Result<(), String> {
    state.clock.pause().await;
    Ok(())
}

#[tauri::command]
pub async fn transport_stop(
    state: State<'_, MidiClockState>,
) -> Result<(), String> {
    state.clock.stop().await;
    Ok(())
}

#[tauri::command]
pub async fn transport_continue(
    state: State<'_, MidiClockState>,
) -> Result<(), String> {
    state.clock.continue_playback().await;
    Ok(())
}

#[tauri::command]
pub async fn set_bpm(
    state: State<'_, MidiClockState>,
    bpm: f64,
) -> Result<(), String> {
    if bpm < 20.0 || bpm > 300.0 {
        return Err("BPM must be between 20 and 300".to_string());
    }
    state.clock.set_bpm(bpm).await;
    Ok(())
}

#[tauri::command]
pub async fn get_bpm(
    state: State<'_, MidiClockState>,
) -> Result<f64, String> {
    Ok(state.clock.get_bpm().await)
}

#[tauri::command]
pub async fn set_time_signature(
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

#[tauri::command]
pub async fn set_sync_mode(
    state: State<'_, MidiClockState>,
    mode: String,
) -> Result<(), String> {
    let sync_mode = match mode.to_lowercase().as_str() {
        "internal" => SyncMode::Internal,
        "external" => SyncMode::External,
        "mtc" | "timecode" => SyncMode::MidiTimecode,
        _ => return Err(format!("Unknown sync mode: {}", mode)),
    };

    state.sync_manager.write().await.set_mode(sync_mode).await;
    Ok(())
}

#[tauri::command]
pub async fn get_sync_status(
    state: State<'_, MidiClockState>,
) -> Result<SyncStatus, String> {
    Ok(state.sync_manager.read().await.get_status().await)
}

#[tauri::command]
pub async fn set_position_bars(
    state: State<'_, MidiClockState>,
    bar: u32,
    beat: u32,
) -> Result<(), String> {
    use crate::midi_clock::messages::SongPosition;

    // Convert bar:beat to MIDI beats (16th notes)
    // Assuming 4/4 time signature for simplicity
    let midi_beats = (bar.saturating_sub(1) * 16) + (beat.saturating_sub(1) * 4);

    state.clock.set_position(SongPosition::new(midi_beats as u16)).await;
    Ok(())
}
```

---

## Part 3: Frontend Implementation

### 3.1 MIDI Clock Store

**File: `app/src/lib/stores/midiClockStore.ts`**
```typescript
import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// Types
export type TransportState = 'stopped' | 'playing' | 'recording' | 'paused';
export type SyncMode = 'internal' | 'external' | 'mtc';

export interface TimeSignature {
  numerator: number;
  denominator: number;
}

export interface Transport {
  state: TransportState;
  bpm: number;
  time_signature: TimeSignature;
  position_ticks: number;
  position_millis: number;
  bar: number;
  beat: number;
  tick: number;
  loop_enabled: boolean;
  loop_start_ticks: number;
  loop_end_ticks: number;
  metronome_enabled: boolean;
  count_in_enabled: boolean;
  count_in_bars: number;
}

export interface SyncStatus {
  mode: SyncMode;
  is_locked: boolean;
  external_bpm: number | null;
  drift_ms: number;
  clock_source: string | null;
  last_sync: number | null;
}

interface MidiClockState {
  transport: Transport;
  syncStatus: SyncStatus;
  isConnected: boolean;
  lastUpdate: number;
}

const defaultTransport: Transport = {
  state: 'stopped',
  bpm: 120.0,
  time_signature: { numerator: 4, denominator: 4 },
  position_ticks: 0,
  position_millis: 0,
  bar: 1,
  beat: 1,
  tick: 0,
  loop_enabled: false,
  loop_start_ticks: 0,
  loop_end_ticks: 0,
  metronome_enabled: false,
  count_in_enabled: false,
  count_in_bars: 1,
};

const defaultSyncStatus: SyncStatus = {
  mode: 'internal',
  is_locked: false,
  external_bpm: null,
  drift_ms: 0,
  clock_source: null,
  last_sync: null,
};

// Store
function createMidiClockStore() {
  const { subscribe, set, update } = writable<MidiClockState>({
    transport: defaultTransport,
    syncStatus: defaultSyncStatus,
    isConnected: false,
    lastUpdate: Date.now(),
  });

  let updateInterval: ReturnType<typeof setInterval> | null = null;
  let eventUnlisten: (() => void) | null = null;

  // Transport controls
  async function play(): Promise<void> {
    try {
      await invoke('transport_play');
      update(s => ({
        ...s,
        transport: { ...s.transport, state: 'playing' },
      }));
    } catch (error) {
      console.error('Failed to play:', error);
      throw error;
    }
  }

  async function pause(): Promise<void> {
    try {
      await invoke('transport_pause');
      update(s => ({
        ...s,
        transport: { ...s.transport, state: 'paused' },
      }));
    } catch (error) {
      console.error('Failed to pause:', error);
      throw error;
    }
  }

  async function stop(): Promise<void> {
    try {
      await invoke('transport_stop');
      update(s => ({
        ...s,
        transport: { ...s.transport, state: 'stopped', bar: 1, beat: 1, tick: 0 },
      }));
    } catch (error) {
      console.error('Failed to stop:', error);
      throw error;
    }
  }

  async function continuePlayback(): Promise<void> {
    try {
      await invoke('transport_continue');
      update(s => ({
        ...s,
        transport: { ...s.transport, state: 'playing' },
      }));
    } catch (error) {
      console.error('Failed to continue:', error);
      throw error;
    }
  }

  async function setBpm(bpm: number): Promise<void> {
    const clampedBpm = Math.max(20, Math.min(300, bpm));
    try {
      await invoke('set_bpm', { bpm: clampedBpm });
      update(s => ({
        ...s,
        transport: { ...s.transport, bpm: clampedBpm },
      }));
    } catch (error) {
      console.error('Failed to set BPM:', error);
      throw error;
    }
  }

  async function setTimeSignature(numerator: number, denominator: number): Promise<void> {
    try {
      await invoke('set_time_signature', { numerator, denominator });
      update(s => ({
        ...s,
        transport: {
          ...s.transport,
          time_signature: { numerator, denominator },
        },
      }));
    } catch (error) {
      console.error('Failed to set time signature:', error);
      throw error;
    }
  }

  async function setSyncMode(mode: SyncMode): Promise<void> {
    try {
      await invoke('set_sync_mode', { mode });
      update(s => ({
        ...s,
        syncStatus: { ...s.syncStatus, mode },
      }));
    } catch (error) {
      console.error('Failed to set sync mode:', error);
      throw error;
    }
  }

  async function setPosition(bar: number, beat: number): Promise<void> {
    try {
      await invoke('set_position_bars', { bar, beat });
      update(s => ({
        ...s,
        transport: { ...s.transport, bar, beat, tick: 0 },
      }));
    } catch (error) {
      console.error('Failed to set position:', error);
      throw error;
    }
  }

  async function fetchTransport(): Promise<void> {
    try {
      const transport = await invoke<Transport>('get_transport');
      update(s => ({
        ...s,
        transport,
        isConnected: true,
        lastUpdate: Date.now(),
      }));
    } catch (error) {
      update(s => ({ ...s, isConnected: false }));
    }
  }

  async function fetchSyncStatus(): Promise<void> {
    try {
      const syncStatus = await invoke<SyncStatus>('get_sync_status');
      update(s => ({ ...s, syncStatus }));
    } catch (error) {
      console.error('Failed to fetch sync status:', error);
    }
  }

  function startUpdates(intervalMs: number = 50): void {
    stopUpdates();

    // Initial fetch
    fetchTransport();
    fetchSyncStatus();

    // Regular updates (for position display)
    updateInterval = setInterval(() => {
      const state = get({ subscribe });
      if (state.transport.state === 'playing' || state.transport.state === 'recording') {
        fetchTransport();
      }
    }, intervalMs);
  }

  function stopUpdates(): void {
    if (updateInterval) {
      clearInterval(updateInterval);
      updateInterval = null;
    }
  }

  async function setupEventListener(): Promise<void> {
    // Listen for transport events from backend
    eventUnlisten = await listen<Transport>('transport-update', (event) => {
      update(s => ({
        ...s,
        transport: event.payload,
        lastUpdate: Date.now(),
      }));
    });
  }

  function cleanup(): void {
    stopUpdates();
    if (eventUnlisten) {
      eventUnlisten();
      eventUnlisten = null;
    }
  }

  return {
    subscribe,
    play,
    pause,
    stop,
    continue: continuePlayback,
    setBpm,
    setTimeSignature,
    setSyncMode,
    setPosition,
    fetchTransport,
    fetchSyncStatus,
    startUpdates,
    stopUpdates,
    setupEventListener,
    cleanup,
  };
}

export const midiClockStore = createMidiClockStore();

// Derived stores
export const transport = derived(midiClockStore, ($store) => $store.transport);
export const transportState = derived(midiClockStore, ($store) => $store.transport.state);
export const bpm = derived(midiClockStore, ($store) => $store.transport.bpm);
export const timeSignature = derived(midiClockStore, ($store) => $store.transport.time_signature);
export const position = derived(midiClockStore, ($store) => ({
  bar: $store.transport.bar,
  beat: $store.transport.beat,
  tick: $store.transport.tick,
  millis: $store.transport.position_millis,
}));
export const syncStatus = derived(midiClockStore, ($store) => $store.syncStatus);
export const isPlaying = derived(midiClockStore, ($store) =>
  $store.transport.state === 'playing' || $store.transport.state === 'recording'
);
export const isSyncLocked = derived(midiClockStore, ($store) => $store.syncStatus.is_locked);
```

### 3.2 Transport Controls Component

**File: `app/src/lib/components/midi/TransportControls.svelte`**
```svelte
<script lang="ts">
  import {
    midiClockStore,
    transport,
    transportState,
    bpm,
    position,
    isPlaying
  } from '$lib/stores/midiClockStore';
  import { onMount, onDestroy } from 'svelte';

  export let compact = false;
  export let showPosition = true;
  export let showBpm = true;

  let bpmInput = 120;
  let isEditingBpm = false;

  $: bpmInput = $bpm;

  function formatPosition(bar: number, beat: number, tick: number): string {
    return `${bar.toString().padStart(3, '0')}:${beat}:${tick.toString().padStart(2, '0')}`;
  }

  function formatTime(millis: number): string {
    const seconds = Math.floor(millis / 1000);
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    const remainingMillis = millis % 1000;

    return `${minutes.toString().padStart(2, '0')}:${remainingSeconds.toString().padStart(2, '0')}.${Math.floor(remainingMillis / 100)}`;
  }

  async function handleBpmChange() {
    isEditingBpm = false;
    if (bpmInput !== $bpm) {
      await midiClockStore.setBpm(bpmInput);
    }
  }

  function handleBpmKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      handleBpmChange();
    } else if (event.key === 'Escape') {
      isEditingBpm = false;
      bpmInput = $bpm;
    }
  }

  function tapTempo() {
    // Simple tap tempo implementation
    const now = Date.now();
    if (!tapTimes) {
      tapTimes = [now];
      return;
    }

    // Reset if more than 2 seconds since last tap
    if (now - tapTimes[tapTimes.length - 1] > 2000) {
      tapTimes = [now];
      return;
    }

    tapTimes.push(now);
    if (tapTimes.length > 8) {
      tapTimes.shift();
    }

    if (tapTimes.length >= 2) {
      const intervals = [];
      for (let i = 1; i < tapTimes.length; i++) {
        intervals.push(tapTimes[i] - tapTimes[i - 1]);
      }
      const avgInterval = intervals.reduce((a, b) => a + b, 0) / intervals.length;
      const calculatedBpm = Math.round(60000 / avgInterval);

      if (calculatedBpm >= 20 && calculatedBpm <= 300) {
        midiClockStore.setBpm(calculatedBpm);
      }
    }
  }

  let tapTimes: number[] | null = null;

  onMount(() => {
    midiClockStore.startUpdates();
  });

  onDestroy(() => {
    midiClockStore.stopUpdates();
  });
</script>

<div class="transport-controls" class:compact role="toolbar" aria-label="Transport controls">
  <!-- Transport buttons -->
  <div class="transport-buttons">
    <button
      class="transport-btn"
      on:click={() => midiClockStore.stop()}
      aria-label="Stop"
      title="Stop (Home)"
    >
      <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
        <rect x="6" y="6" width="12" height="12" />
      </svg>
    </button>

    {#if $isPlaying}
      <button
        class="transport-btn"
        on:click={() => midiClockStore.pause()}
        aria-label="Pause"
        title="Pause (Space)"
      >
        <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
          <rect x="6" y="5" width="4" height="14" />
          <rect x="14" y="5" width="4" height="14" />
        </svg>
      </button>
    {:else}
      <button
        class="transport-btn play"
        on:click={() => $transportState === 'paused' ? midiClockStore.continue() : midiClockStore.play()}
        aria-label="Play"
        title="Play (Space)"
      >
        <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
          <polygon points="8,5 19,12 8,19" />
        </svg>
      </button>
    {/if}

    <button
      class="transport-btn record"
      class:active={$transportState === 'recording'}
      aria-label="Record"
      aria-pressed={$transportState === 'recording'}
      title="Record (R)"
    >
      <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
        <circle cx="12" cy="12" r="7" />
      </svg>
    </button>
  </div>

  <!-- Position display -->
  {#if showPosition && !compact}
    <div class="position-display" aria-label="Position">
      <span class="position-bbt">{formatPosition($position.bar, $position.beat, $position.tick)}</span>
      <span class="position-time">{formatTime($position.millis)}</span>
    </div>
  {/if}

  <!-- BPM control -->
  {#if showBpm}
    <div class="bpm-control">
      {#if isEditingBpm}
        <input
          type="number"
          class="bpm-input"
          bind:value={bpmInput}
          on:blur={handleBpmChange}
          on:keydown={handleBpmKeydown}
          min="20"
          max="300"
          step="0.1"
          aria-label="BPM"
          autofocus
        />
      {:else}
        <button
          class="bpm-display"
          on:click={() => isEditingBpm = true}
          aria-label="BPM: {$bpm.toFixed(1)}. Click to edit"
        >
          {$bpm.toFixed(1)}
        </button>
      {/if}
      <span class="bpm-label">BPM</span>

      {#if !compact}
        <button
          class="tap-tempo"
          on:click={tapTempo}
          title="Tap Tempo"
          aria-label="Tap tempo"
        >
          TAP
        </button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .transport-controls {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 8px 12px;
    background: var(--panel-bg, #1a1a1a);
    border-radius: 6px;
  }

  .transport-controls.compact {
    gap: 8px;
    padding: 4px 8px;
  }

  .transport-buttons {
    display: flex;
    gap: 4px;
  }

  .transport-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    padding: 0;
    background: var(--button-bg, #333);
    border: none;
    border-radius: 4px;
    color: var(--text-color, #fff);
    cursor: pointer;
    transition: background-color 0.15s, transform 0.1s;
  }

  .compact .transport-btn {
    width: 28px;
    height: 28px;
  }

  .transport-btn:hover {
    background: var(--button-hover-bg, #444);
  }

  .transport-btn:active {
    transform: scale(0.95);
  }

  .transport-btn.play {
    color: var(--color-success, #22c55e);
  }

  .transport-btn.record {
    color: var(--text-muted, #666);
  }

  .transport-btn.record.active,
  .transport-btn.record:hover {
    color: var(--color-error, #ef4444);
  }

  .position-display {
    display: flex;
    flex-direction: column;
    align-items: center;
    min-width: 100px;
    font-family: monospace;
  }

  .position-bbt {
    font-size: 18px;
    font-weight: bold;
    letter-spacing: 1px;
  }

  .position-time {
    font-size: 11px;
    color: var(--text-muted, #888);
  }

  .bpm-control {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .bpm-display {
    min-width: 60px;
    padding: 4px 8px;
    background: var(--input-bg, #222);
    border: 1px solid var(--border-color, #333);
    border-radius: 4px;
    color: inherit;
    font-family: monospace;
    font-size: 16px;
    font-weight: bold;
    text-align: center;
    cursor: pointer;
    transition: border-color 0.15s;
  }

  .bpm-display:hover {
    border-color: var(--accent-color, #3b82f6);
  }

  .bpm-input {
    width: 60px;
    padding: 4px 8px;
    background: var(--input-bg, #222);
    border: 1px solid var(--accent-color, #3b82f6);
    border-radius: 4px;
    color: inherit;
    font-family: monospace;
    font-size: 16px;
    font-weight: bold;
    text-align: center;
  }

  .bpm-input:focus {
    outline: none;
  }

  .bpm-label {
    font-size: 11px;
    color: var(--text-muted, #888);
    text-transform: uppercase;
  }

  .tap-tempo {
    padding: 4px 10px;
    background: var(--button-bg, #333);
    border: none;
    border-radius: 4px;
    color: inherit;
    font-size: 11px;
    font-weight: bold;
    cursor: pointer;
    transition: background-color 0.15s;
  }

  .tap-tempo:hover {
    background: var(--button-hover-bg, #444);
  }

  .tap-tempo:active {
    background: var(--accent-color, #3b82f6);
  }
</style>
```

### 3.3 Sync Status Component

**File: `app/src/lib/components/midi/SyncStatus.svelte`**
```svelte
<script lang="ts">
  import { midiClockStore, syncStatus, type SyncMode } from '$lib/stores/midiClockStore';
  import { onMount } from 'svelte';

  export let showModeSelector = true;

  let selectedMode: SyncMode = 'internal';
  let expanded = false;

  $: selectedMode = $syncStatus.mode;

  const modeLabels: Record<SyncMode, string> = {
    internal: 'Internal',
    external: 'External MIDI',
    mtc: 'MIDI Timecode',
  };

  const modeDescriptions: Record<SyncMode, string> = {
    internal: 'This device is the clock master',
    external: 'Sync to external MIDI clock',
    mtc: 'Sync to MIDI Time Code',
  };

  async function handleModeChange(mode: SyncMode) {
    await midiClockStore.setSyncMode(mode);
    expanded = false;
  }

  onMount(() => {
    midiClockStore.fetchSyncStatus();
  });
</script>

<div class="sync-status">
  <div class="sync-header">
    <span class="sync-label">SYNC</span>

    <span
      class="sync-indicator"
      class:locked={$syncStatus.is_locked}
      class:external={$syncStatus.mode !== 'internal'}
      title={$syncStatus.is_locked ? 'Locked to external clock' : 'Not locked'}
    >
      {#if $syncStatus.mode === 'internal'}
        <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
          <circle cx="12" cy="12" r="3" />
          <path d="M12 2v4M12 18v4M2 12h4M18 12h4" stroke="currentColor" stroke-width="2" fill="none" />
        </svg>
      {:else if $syncStatus.is_locked}
        <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
          <rect x="3" y="11" width="18" height="11" rx="2" />
          <path d="M7 11V7a5 5 0 0 1 10 0v4" stroke="currentColor" stroke-width="2" fill="none" />
        </svg>
      {:else}
        <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
          <rect x="3" y="11" width="18" height="11" rx="2" />
          <path d="M7 11V7a5 5 0 0 1 9.9-1" stroke="currentColor" stroke-width="2" fill="none" />
        </svg>
      {/if}
    </span>
  </div>

  {#if showModeSelector}
    <div class="mode-selector">
      <button
        class="mode-button"
        on:click={() => expanded = !expanded}
        aria-expanded={expanded}
        aria-haspopup="listbox"
      >
        {modeLabels[$syncStatus.mode]}
        <svg viewBox="0 0 24 24" width="12" height="12" fill="currentColor">
          <polyline points="6 9 12 15 18 9" stroke="currentColor" stroke-width="2" fill="none" />
        </svg>
      </button>

      {#if expanded}
        <ul class="mode-dropdown" role="listbox">
          {#each Object.entries(modeLabels) as [mode, label]}
            <li>
              <button
                role="option"
                aria-selected={mode === $syncStatus.mode}
                class:selected={mode === $syncStatus.mode}
                on:click={() => handleModeChange(mode as SyncMode)}
              >
                <span class="mode-name">{label}</span>
                <span class="mode-desc">{modeDescriptions[mode as SyncMode]}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}

  {#if $syncStatus.mode !== 'internal'}
    <div class="sync-details">
      {#if $syncStatus.external_bpm}
        <span class="external-bpm">
          EXT: {$syncStatus.external_bpm.toFixed(1)} BPM
        </span>
      {/if}

      {#if $syncStatus.drift_ms > 0}
        <span
          class="drift"
          class:warning={$syncStatus.drift_ms > 10}
        >
          Drift: {$syncStatus.drift_ms.toFixed(1)}ms
        </span>
      {/if}

      {#if $syncStatus.clock_source}
        <span class="clock-source">
          Source: {$syncStatus.clock_source}
        </span>
      {/if}
    </div>
  {/if}
</div>

<style>
  .sync-status {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px;
    background: var(--panel-bg, #1a1a1a);
    border-radius: 6px;
    font-size: 12px;
  }

  .sync-header {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .sync-label {
    font-size: 10px;
    font-weight: bold;
    text-transform: uppercase;
    color: var(--text-muted, #888);
    letter-spacing: 1px;
  }

  .sync-indicator {
    display: flex;
    align-items: center;
    color: var(--text-muted, #666);
  }

  .sync-indicator.external {
    color: var(--color-warning, #eab308);
  }

  .sync-indicator.locked {
    color: var(--color-success, #22c55e);
  }

  .mode-selector {
    position: relative;
  }

  .mode-button {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    background: var(--button-bg, #333);
    border: none;
    border-radius: 4px;
    color: inherit;
    font-size: 12px;
    cursor: pointer;
    transition: background-color 0.15s;
  }

  .mode-button:hover {
    background: var(--button-hover-bg, #444);
  }

  .mode-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: 4px;
    padding: 4px;
    background: var(--dropdown-bg, #222);
    border: 1px solid var(--border-color, #333);
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    list-style: none;
    z-index: 100;
    min-width: 180px;
  }

  .mode-dropdown li {
    margin: 0;
    padding: 0;
  }

  .mode-dropdown button {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    width: 100%;
    padding: 8px 10px;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: inherit;
    text-align: left;
    cursor: pointer;
    transition: background-color 0.15s;
  }

  .mode-dropdown button:hover {
    background: var(--hover-bg, rgba(255, 255, 255, 0.05));
  }

  .mode-dropdown button.selected {
    background: var(--selected-bg, rgba(59, 130, 246, 0.2));
  }

  .mode-name {
    font-weight: 500;
  }

  .mode-desc {
    font-size: 10px;
    color: var(--text-muted, #888);
    margin-top: 2px;
  }

  .sync-details {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    padding-top: 6px;
    border-top: 1px solid var(--border-color, #333);
  }

  .external-bpm {
    font-family: monospace;
    color: var(--color-success, #22c55e);
  }

  .drift {
    font-family: monospace;
    color: var(--text-muted, #888);
  }

  .drift.warning {
    color: var(--color-warning, #eab308);
  }

  .clock-source {
    color: var(--text-muted, #888);
  }
</style>
```

---

## Part 4: Integration

### 4.1 Register module in lib.rs

**Add to `daw/src-tauri/src/lib.rs`:**
```rust
pub mod midi_clock;
```

### 4.2 Register commands in main.rs

**Add to your Tauri command registration:**
```rust
use daw::commands::midi_clock::{
    get_transport,
    transport_play,
    transport_pause,
    transport_stop,
    transport_continue,
    set_bpm,
    get_bpm,
    set_time_signature,
    set_sync_mode,
    get_sync_status,
    set_position_bars,
    MidiClockState,
};
use daw::midi_clock::clock::MidiClock;
use daw::midi_clock::sync::SyncManager;
use std::sync::Arc;

// In your main/setup:
let (clock, _tick_rx, _message_rx) = MidiClock::new();
let clock = Arc::new(clock);
let sync_manager = Arc::new(RwLock::new(SyncManager::new(clock.clone())));

app.manage(MidiClockState {
    clock,
    sync_manager,
});

// Add to invoke_handler:
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    get_transport,
    transport_play,
    transport_pause,
    transport_stop,
    transport_continue,
    set_bpm,
    get_bpm,
    set_time_signature,
    set_sync_mode,
    get_sync_status,
    set_position_bars,
])
```

### 4.3 Add to Cargo.toml dependencies

**Add to `daw/src-tauri/Cargo.toml`:**
```toml
[dependencies]
tokio = { version = "1", features = ["sync", "time", "rt-multi-thread"] }
chrono = { version = "0.4", features = ["serde"] }
```

---

## Summary Checklist

### Rust Backend
- [ ] Create `daw/src-tauri/src/midi_clock/mod.rs`
- [ ] Create `daw/src-tauri/src/midi_clock/messages.rs`
- [ ] Create `daw/src-tauri/src/midi_clock/clock.rs`
- [ ] Create `daw/src-tauri/src/midi_clock/sync.rs`
- [ ] Create `daw/src-tauri/src/midi_clock/transport.rs`
- [ ] Create `daw/src-tauri/src/commands/midi_clock.rs`

### Frontend
- [ ] Create `app/src/lib/stores/midiClockStore.ts`
- [ ] Create `app/src/lib/components/midi/TransportControls.svelte`
- [ ] Create `app/src/lib/components/midi/SyncStatus.svelte`

### Integration
- [ ] Export midi_clock module from `daw/src-tauri/src/lib.rs`
- [ ] Register MIDI clock commands in Tauri
- [ ] Add TransportControls to DAW toolbar
- [ ] Add SyncStatus to status bar or settings

### Testing
- [ ] Test internal clock at various BPMs
- [ ] Test tap tempo functionality
- [ ] Test transport controls (play/pause/stop/continue)
- [ ] Test position display accuracy
- [ ] Test external sync mode (if hardware available)
- [ ] Test BPM detection from external clock

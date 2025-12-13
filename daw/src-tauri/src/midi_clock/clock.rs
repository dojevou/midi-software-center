#![allow(dead_code)]

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::interval;
use tracing::info;

use super::messages::{MidiClockMessage, SongPosition};
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
    pub fn new() -> (
        Self,
        broadcast::Receiver<ClockTick>,
        mpsc::Receiver<MidiClockMessage>,
    ) {
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
        let _state = self.state.clone();
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

                let bar = tick as u32 / ticks_per_bar;
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

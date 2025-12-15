#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info};

use super::clock::{ClockTick, MidiClock, PPQN};
use super::messages::MidiClockMessage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SyncMode {
    #[default]
    Internal, // We are the master
    External,     // Sync to external MIDI clock
    MidiTimecode, // Sync to MTC
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncStatus {
    pub mode: SyncMode,
    pub is_locked: bool,
    pub external_bpm: Option<f64>,
    pub drift_ms: f64,
    pub clock_source: Option<String>,
    #[serde(skip)]
    pub last_sync: Option<Instant>,
}

/// BPM detector for external clock
struct BpmDetector {
    tick_times: Vec<Instant>,
    max_samples: usize,
}

impl BpmDetector {
    fn new(max_samples: usize) -> Self {
        Self { tick_times: Vec::with_capacity(max_samples), max_samples }
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
        if (20.0..=300.0).contains(&bpm) {
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
            },
            MidiClockMessage::Start => {
                info!("External clock start received");
                self.clock.start().await;
            },
            MidiClockMessage::Continue => {
                info!("External clock continue received");
                self.clock.continue_playback().await;
            },
            MidiClockMessage::Stop => {
                info!("External clock stop received");
                self.clock.stop().await;
            },
            _ => {},
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
                debug!(
                    external_bpm,
                    current_bpm, "Adjusting BPM to match external clock"
                );
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
    pub async fn get_clock_output(&self) -> Option<tokio::sync::broadcast::Receiver<ClockTick>> {
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
                        },
                        MidiClockMessage::Start => {
                            clock.start().await;
                        },
                        MidiClockMessage::Continue => {
                            clock.continue_playback().await;
                        },
                        MidiClockMessage::Stop => {
                            clock.stop().await;
                        },
                        _ => {},
                    }
                }
            });
        }
    }
}

#![allow(dead_code)]
/// Sequencer playback engine
///
/// Grown-up Script: Coordinates playback, timing, and MIDI output.
/// Integrates TrackManager, EventScheduler, and MidiManager.
use crate::core::sequencer::timing;
use crate::midi::MidiManager;
use crate::models::sequencer::PlaybackPosition;
use crate::sequencer::{EventScheduler, ScheduledEvent, TrackManager};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tokio::time;
use tracing::{debug, error, info, warn};

/// Playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

/// Main sequencer engine
///
/// Manages playback state, timing, and coordinates all sequencer components.
pub struct SequencerEngine {
    track_manager: Arc<TrackManager>,
    scheduler: Arc<EventScheduler>,
    midi_manager: Arc<MidiManager>,

    // Playback state
    state: Arc<RwLock<PlaybackState>>,
    current_tick: Arc<RwLock<u64>>,
    start_time: Arc<Mutex<Option<Instant>>>,

    // Transport settings
    bpm: Arc<RwLock<f32>>,
    ticks_per_quarter: u16,
    beats_per_bar: u8,

    #[allow(dead_code)]
    // Playback control
    loop_enabled: Arc<RwLock<bool>>,
    loop_start: Arc<RwLock<u64>>,
    loop_end: Arc<RwLock<u64>>,
}

impl SequencerEngine {
    /// Create a new sequencer engine
    ///
    /// # Arguments
    /// * `midi_manager` - MIDI output manager
    /// * `bpm` - Initial tempo (default: 120.0)
    /// * `ticks_per_quarter` - MIDI resolution (default: 480)
    pub fn new(midi_manager: Arc<MidiManager>, bpm: f32, ticks_per_quarter: u16) -> Self {
        Self {
            track_manager: Arc::new(TrackManager::new()),
            scheduler: Arc::new(EventScheduler::new()),
            midi_manager,
            state: Arc::new(RwLock::new(PlaybackState::Stopped)),
            current_tick: Arc::new(RwLock::new(0)),
            start_time: Arc::new(Mutex::new(None)),
            bpm: Arc::new(RwLock::new(bpm)),
            ticks_per_quarter,
            beats_per_bar: 4,
            loop_enabled: Arc::new(RwLock::new(false)),
            loop_start: Arc::new(RwLock::new(0)),
            loop_end: Arc::new(RwLock::new(0)),
        }
    }

    /// Get track manager reference
    pub fn track_manager(&self) -> Arc<TrackManager> {
        self.track_manager.clone()
    }

    /// Get event scheduler reference
    pub fn scheduler(&self) -> Arc<EventScheduler> {
        self.scheduler.clone()
    }

    /// Start playback
    pub async fn start(&self) -> Result<(), String> {
        let mut state = self.state.write().await;

        if *state == PlaybackState::Playing {
            return Ok(()); // Already playing
        }

        // Check if MIDI is connected
        if !self.midi_manager.is_connected().await {
            return Err("MIDI device not connected".to_string());
        }

        info!("Starting sequencer playback");

        *state = PlaybackState::Playing;
        let mut start_time = self.start_time.lock().await;
        *start_time = Some(Instant::now());

        // Spawn playback task
        self.spawn_playback_task().await;

        Ok(())
    }

    /// Stop playback and reset position
    ///
    /// Always resets position to 0, regardless of current state.
    /// Only sends panic message if actually stopping playback.
    pub async fn stop(&self) {
        let mut state = self.state.write().await;
        let was_playing = *state != PlaybackState::Stopped;

        if was_playing {
            info!("Stopping sequencer playback");
            *state = PlaybackState::Stopped;

            // Send all notes off to prevent stuck notes
            if let Err(e) = self.send_panic().await {
                warn!("Failed to send panic message: {}", e);
            }
        }

        // Always reset position to 0 when stop is called
        let mut current_tick = self.current_tick.write().await;
        *current_tick = 0;
    }

    /// Pause playback (maintains position)
    pub async fn pause(&self) {
        let mut state = self.state.write().await;

        if *state != PlaybackState::Playing {
            return;
        }

        info!("Pausing sequencer playback");
        *state = PlaybackState::Paused;

        // Send all notes off to prevent stuck notes
        if let Err(e) = self.send_panic().await {
            warn!("Failed to send panic message: {}", e);
        }
    }

    /// Resume playback from paused state
    pub async fn resume(&self) -> Result<(), String> {
        let mut state = self.state.write().await;

        if *state != PlaybackState::Paused {
            return Ok(());
        }

        if !self.midi_manager.is_connected().await {
            return Err("MIDI device not connected".to_string());
        }

        info!("Resuming sequencer playback");

        *state = PlaybackState::Playing;
        let mut start_time = self.start_time.lock().await;
        *start_time = Some(Instant::now());

        self.spawn_playback_task().await;

        Ok(())
    }

    /// Get current playback state
    pub async fn get_state(&self) -> PlaybackState {
        *self.state.read().await
    }

    /// Set tempo (BPM)
    pub async fn set_bpm(&self, bpm: f32) -> Result<(), String> {
        if !(20.0..=300.0).contains(&bpm) {
            return Err(format!("Invalid BPM: {}. Must be 20-300", bpm));
        }

        let mut current_bpm = self.bpm.write().await;
        *current_bpm = bpm;
        info!("Tempo set to {} BPM", bpm);

        Ok(())
    }

    /// Get current tempo
    pub async fn get_bpm(&self) -> f32 {
        *self.bpm.read().await
    }

    /// Get current playback position
    pub async fn get_position(&self) -> PlaybackPosition {
        let tick = *self.current_tick.read().await;
        let (bar, beat) =
            timing::tick_to_bar_beat(tick, self.ticks_per_quarter, self.beats_per_bar);

        PlaybackPosition { current_tick: tick, current_bar: bar, current_beat: beat }
    }

    /// Get current playback position in seconds
    ///
    /// Returns the current position converted to real time in seconds.
    pub async fn get_position_seconds(&self) -> f64 {
        let tick = *self.current_tick.read().await;
        let bpm = *self.bpm.read().await;
        timing::ticks_to_seconds(tick, bpm, self.ticks_per_quarter)
    }

    /// Get detailed bar position (bar, beat, tick within beat)
    ///
    /// Returns a BarPosition struct with full musical position information.
    pub async fn get_bar_position(&self) -> timing::BarPosition {
        let tick = *self.current_tick.read().await;
        timing::calculate_bar_position(tick, self.ticks_per_quarter, self.beats_per_bar, 4)
    }

    /// Convert tick position to seconds for display
    ///
    /// Useful for showing position in time-based views.
    pub fn tick_to_seconds(&self, tick: u64, bpm: f32) -> f64 {
        timing::ticks_to_seconds(tick, bpm, self.ticks_per_quarter)
    }

    /// Convert seconds to tick position for seeking
    ///
    /// Useful for seeking to time-based positions.
    pub fn seconds_to_tick(&self, seconds: f64, bpm: f32) -> u64 {
        timing::seconds_to_ticks(seconds, bpm, self.ticks_per_quarter)
    }

    /// Seek to a specific tick position
    pub async fn seek(&self, tick: u64) {
        let mut current_tick = self.current_tick.write().await;
        *current_tick = tick;
        debug!("Seeked to tick {}", tick);
    }

    /// Schedule a single MIDI event
    ///
    /// Useful for real-time note input or automation.
    pub async fn schedule_event(
        &self,
        message: crate::core::midi::types::MidiMessage,
        tick: u64,
        track_id: i32,
    ) {
        self.scheduler.schedule(message, tick, track_id).await;
    }

    /// Get the next scheduled event at or before current tick
    ///
    /// Returns None if no events are ready.
    pub async fn pop_next_event(&self) -> Option<ScheduledEvent> {
        let current_tick = *self.current_tick.read().await;
        self.scheduler.pop_next(current_tick).await
    }

    /// Peek at the next event's tick without removing it
    ///
    /// Useful for lookahead timing calculations.
    pub async fn peek_next_tick(&self) -> Option<u64> {
        self.scheduler.peek_next().await
    }

    /// Check if the scheduler has any pending events
    pub async fn has_pending_events(&self) -> bool {
        !self.scheduler.is_empty().await
    }

    /// Get the number of scheduled events
    pub async fn scheduled_event_count(&self) -> usize {
        self.scheduler.len().await
    }

    /// Convert tick position to microseconds for precise timing
    ///
    /// Useful for hardware timing and precise scheduling.
    pub fn tick_to_microseconds(&self, tick: u64, bpm: f32) -> u64 {
        timing::ticks_to_microseconds(tick, self.ticks_per_quarter, bpm)
    }

    /// Convert microseconds to tick position
    ///
    /// Useful for converting hardware timing to MIDI ticks.
    pub fn microseconds_to_tick(&self, micros: u64, bpm: f32) -> u64 {
        timing::microseconds_to_ticks(micros, self.ticks_per_quarter, bpm)
    }

    /// Get ticks per bar for current time signature
    ///
    /// Returns the number of ticks in one bar based on time signature.
    pub fn get_ticks_per_bar(&self) -> u64 {
        timing::ticks_per_bar(self.ticks_per_quarter, self.beats_per_bar, 4)
    }

    /// Get ticks per quarter note (PPQ/resolution)
    pub fn get_ticks_per_quarter(&self) -> u16 {
        self.ticks_per_quarter
    }

    /// Get beats per bar (time signature numerator)
    pub fn get_beats_per_bar(&self) -> u8 {
        self.beats_per_bar
    }

    /// Send a control change message via MIDI manager
    ///
    /// Directly sends CC without scheduling.
    pub async fn send_control_change(
        &self,
        channel: u8,
        controller: u8,
        value: u8,
    ) -> Result<(), String> {
        self.midi_manager
            .send_control_change(channel, controller, value)
            .await
    }

    /// Send a program change message via MIDI manager
    ///
    /// Directly sends program change without scheduling.
    pub async fn send_program_change(&self, channel: u8, program: u8) -> Result<(), String> {
        self.midi_manager.send_program_change(channel, program).await
    }

    /// Get a specific track by ID
    pub async fn get_track(&self, track_id: i32) -> Option<crate::models::sequencer::Track> {
        self.track_manager.get_track(track_id).await
    }

    /// Check if any tracks are soloed
    pub async fn has_soloed_tracks(&self) -> bool {
        self.track_manager.has_solo().await
    }

    /// Load tracks into the scheduler
    ///
    /// Clears existing scheduled events and loads events from all active tracks.
    pub async fn load_tracks(&self) {
        use crate::core::midi::types::{MidiEventType as CoreEventType, MidiMessage};
        use crate::models::midi::MidiEventType;

        self.scheduler.clear().await;

        let tracks = self.track_manager.get_active_tracks().await;
        let mut all_events = Vec::new();

        for track in tracks {
            for event in track.events {
                // Convert MidiEvent to MidiMessage
                let message = match event.event_type {
                    MidiEventType::NoteOn => MidiMessage {
                        event_type: CoreEventType::NoteOn,
                        channel: track.channel,
                        data1: event.note.unwrap_or(0),
                        data2: event.velocity.unwrap_or(0),
                        timestamp: event.tick,
                    },
                    MidiEventType::NoteOff => MidiMessage {
                        event_type: CoreEventType::NoteOff,
                        channel: track.channel,
                        data1: event.note.unwrap_or(0),
                        data2: event.velocity.unwrap_or(0),
                        timestamp: event.tick,
                    },
                    MidiEventType::ControlChange => MidiMessage {
                        event_type: CoreEventType::ControlChange,
                        channel: track.channel,
                        data1: event.controller.unwrap_or(0),
                        data2: event.value.unwrap_or(0),
                        timestamp: event.tick,
                    },
                    MidiEventType::ProgramChange => MidiMessage {
                        event_type: CoreEventType::ProgramChange,
                        channel: track.channel,
                        data1: event.program.unwrap_or(0),
                        data2: 0,
                        timestamp: event.tick,
                    },
                    MidiEventType::PitchBend => MidiMessage {
                        event_type: CoreEventType::PitchBend,
                        channel: track.channel,
                        data1: event.value.unwrap_or(0),
                        data2: 0,
                        timestamp: event.tick,
                    },
                    MidiEventType::Aftertouch => MidiMessage {
                        event_type: CoreEventType::Aftertouch,
                        channel: track.channel,
                        data1: event.note.unwrap_or(0),
                        data2: event.value.unwrap_or(0),
                        timestamp: event.tick,
                    },
                };

                all_events.push(ScheduledEvent { message, tick: event.tick, track_id: track.id });
            }
        }

        self.scheduler.schedule_many(all_events).await;
        info!(
            "Loaded {} events into scheduler",
            self.scheduler.len().await
        );
    }

    /// Send MIDI panic (all notes off on all channels)
    async fn send_panic(&self) -> Result<(), String> {
        use crate::core::midi::types::{MidiEventType, MidiMessage};

        for channel in 0..16 {
            let panic_msg = MidiMessage {
                event_type: MidiEventType::ControlChange,
                channel,
                data1: 123, // All Notes Off
                data2: 0,
                timestamp: 0,
            };

            self.midi_manager.send_message(&panic_msg).await?;
        }

        Ok(())
    }

    /// Spawn the playback task
    async fn spawn_playback_task(&self) {
        let state = self.state.clone();
        let current_tick = self.current_tick.clone();
        let start_time = self.start_time.clone();
        let bpm = self.bpm.clone();
        let ticks_per_quarter = self.ticks_per_quarter;
        let scheduler = self.scheduler.clone();
        let midi_manager = self.midi_manager.clone();

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(1)); // 1ms resolution

            loop {
                interval.tick().await;

                // Check if still playing
                {
                    let state_guard = state.read().await;
                    if *state_guard != PlaybackState::Playing {
                        break;
                    }
                }

                // Calculate current tick from elapsed time
                let elapsed = {
                    let start_guard = start_time.lock().await;
                    match start_guard.as_ref() {
                        Some(instant) => instant.elapsed(),
                        None => {
                            error!("Start time not set in sequencer playback loop");
                            break;
                        },
                    }
                };

                let bpm_val = *bpm.read().await;
                let tick =
                    timing::seconds_to_ticks(elapsed.as_secs_f64(), bpm_val, ticks_per_quarter);

                // Update current tick
                {
                    let mut tick_guard = current_tick.write().await;
                    *tick_guard = tick;
                }

                // Get ready events
                let ready_events = scheduler.pop_ready(tick).await;

                // Send MIDI messages
                for event in ready_events {
                    if let Err(e) = midi_manager.send_message(&event.message).await {
                        error!("Failed to send MIDI message: {}", e);
                    }
                }
            }

            debug!("Playback task stopped");
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_engine_creation() {
        let midi_manager = Arc::new(MidiManager::new());
        let engine = SequencerEngine::new(midi_manager, 120.0, 480);

        assert_eq!(engine.get_state().await, PlaybackState::Stopped);
        assert_eq!(engine.get_bpm().await, 120.0);
    }

    #[tokio::test]
    async fn test_set_bpm() {
        let midi_manager = Arc::new(MidiManager::new());
        let engine = SequencerEngine::new(midi_manager, 120.0, 480);

        assert!(engine.set_bpm(140.0).await.is_ok());
        assert_eq!(engine.get_bpm().await, 140.0);
    }

    #[tokio::test]
    async fn test_set_bpm_invalid() {
        let midi_manager = Arc::new(MidiManager::new());
        let engine = SequencerEngine::new(midi_manager, 120.0, 480);

        assert!(engine.set_bpm(500.0).await.is_err());
        assert!(engine.set_bpm(10.0).await.is_err());
    }

    #[tokio::test]
    async fn test_get_position() {
        let midi_manager = Arc::new(MidiManager::new());
        let engine = SequencerEngine::new(midi_manager, 120.0, 480);

        let pos = engine.get_position().await;
        assert_eq!(pos.current_tick, 0);
        assert_eq!(pos.current_bar, 0);
        assert_eq!(pos.current_beat, 0);
    }

    #[tokio::test]
    async fn test_seek() {
        let midi_manager = Arc::new(MidiManager::new());
        let engine = SequencerEngine::new(midi_manager, 120.0, 480);

        engine.seek(1920).await; // 1 bar

        let pos = engine.get_position().await;
        assert_eq!(pos.current_tick, 1920);
        assert_eq!(pos.current_bar, 1);
        assert_eq!(pos.current_beat, 0);
    }

    #[tokio::test]
    async fn test_stop_from_playing() {
        let midi_manager = Arc::new(MidiManager::new());
        let engine = SequencerEngine::new(midi_manager, 120.0, 480);

        engine.seek(1000).await;
        engine.stop().await;

        // Stop should reset position
        let pos = engine.get_position().await;
        assert_eq!(pos.current_tick, 0);
        assert_eq!(engine.get_state().await, PlaybackState::Stopped);
    }
}

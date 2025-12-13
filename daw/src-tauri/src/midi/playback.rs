#![allow(dead_code)]
//! Sample-accurate MIDI playback engine with dedicated thread pattern
//!
//! Provides high-precision MIDI playback with:
//! - Sub-millisecond timing accuracy
//! - Seamless loop regions
//! - Chase feature (send PC/CC when starting mid-song)
//! - Multiple output device support
//! - Thread-safe ALSA handling via dedicated playback thread

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use midir::{MidiOutput, MidiOutputConnection};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, oneshot};
use tracing::{debug, info, warn};

use crate::core::midi::loader::load_midi_file;
use crate::core::midi::types::{MidiEventType, MidiMessage};
use crate::models::midi::MidiEvent;

// =============================================================================
// CONSTANTS
// =============================================================================

/// Playback engine tick rate (microseconds) - 100us for sample-accurate timing
const PLAYBACK_TICK_US: u64 = 100;

/// Default pulses per quarter note
const DEFAULT_PPQN: u32 = 480;

// =============================================================================
// TYPES
// =============================================================================

/// Playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

/// Loop region definition
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LoopRegion {
    /// Start position in ticks
    pub start_tick: u64,
    /// End position in ticks
    pub end_tick: u64,
}

impl LoopRegion {
    pub fn new(start_tick: u64, end_tick: u64) -> Self {
        Self { start_tick, end_tick }
    }

    pub fn duration_ticks(&self) -> u64 {
        self.end_tick.saturating_sub(self.start_tick)
    }

    pub fn contains(&self, tick: u64) -> bool {
        tick >= self.start_tick && tick < self.end_tick
    }
}

/// Playback position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackPosition {
    /// Current tick position
    pub tick: u64,
    /// Position in milliseconds
    pub millis: u64,
    /// Current bar (1-indexed)
    pub bar: u32,
    /// Current beat within bar (1-indexed)
    pub beat: u32,
    /// Current sub-beat (0-indexed, within beat)
    pub sub_beat: u32,
    /// Current loop iteration (0 if not looping)
    pub loop_count: u32,
}

/// Chase state - tracks PC/CC values for each channel
#[derive(Debug, Clone, Default)]
pub struct ChaseState {
    /// Program changes per channel (channel -> program)
    pub programs: HashMap<u8, u8>,
    /// Control changes per channel (channel -> (controller -> value))
    pub controllers: HashMap<u8, HashMap<u8, u8>>,
    /// Last pitch bend value per channel
    pub pitch_bends: HashMap<u8, (u8, u8)>,
}

impl ChaseState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update state from a MIDI event
    pub fn update_from_event(&mut self, event: &MidiEvent) {
        match event.event_type {
            MidiEventType::ProgramChange => {
                if let Some(program) = event.program {
                    self.programs.insert(event.channel, program);
                }
            },
            MidiEventType::ControlChange => {
                if let (Some(controller), Some(value)) = (event.controller, event.value) {
                    self.controllers.entry(event.channel).or_default().insert(controller, value);
                }
            },
            MidiEventType::PitchBend => {
                if let Some(value) = event.value {
                    // Store as LSB, MSB (7-bit each)
                    let lsb = value & 0x7F;
                    let msb = (value >> 7) & 0x7F;
                    self.pitch_bends.insert(event.channel, (lsb, msb));
                }
            },
            _ => {},
        }
    }

    /// Build chase state from events up to a given tick
    pub fn build_from_events(events: &[MidiEvent], up_to_tick: u64) -> Self {
        let mut state = Self::new();
        for event in events.iter().filter(|e| e.tick < up_to_tick) {
            state.update_from_event(event);
        }
        state
    }

    /// Convert chase state to MIDI messages for sending
    pub fn to_messages(&self) -> Vec<MidiMessage> {
        let mut messages = Vec::new();

        // Send program changes
        for (&channel, &program) in &self.programs {
            messages.push(MidiMessage {
                event_type: MidiEventType::ProgramChange,
                channel,
                data1: program,
                data2: 0,
                timestamp: 0,
            });
        }

        // Send control changes
        for (&channel, controllers) in &self.controllers {
            for (&controller, &value) in controllers {
                messages.push(MidiMessage {
                    event_type: MidiEventType::ControlChange,
                    channel,
                    data1: controller,
                    data2: value,
                    timestamp: 0,
                });
            }
        }

        // Send pitch bends
        for (&channel, &(lsb, msb)) in &self.pitch_bends {
            messages.push(MidiMessage {
                event_type: MidiEventType::PitchBend,
                channel,
                data1: lsb,
                data2: msb,
                timestamp: 0,
            });
        }

        messages
    }
}

/// Playback configuration
#[derive(Debug, Clone)]
pub struct PlaybackConfig {
    pub bpm: f64,
    pub ticks_per_quarter: u32,
    pub time_signature_numerator: u8,
    pub time_signature_denominator: u8,
    pub loop_enabled: bool,
    pub chase_enabled: bool,
}

impl Default for PlaybackConfig {
    fn default() -> Self {
        Self {
            bpm: 120.0,
            ticks_per_quarter: DEFAULT_PPQN,
            time_signature_numerator: 4,
            time_signature_denominator: 4,
            loop_enabled: false,
            chase_enabled: true,
        }
    }
}

/// Event sent when playback position changes
#[derive(Debug, Clone)]
pub struct PlaybackEvent {
    pub position: PlaybackPosition,
    pub state: PlaybackState,
}

// =============================================================================
// THREAD COMMUNICATION
// =============================================================================

/// Commands sent to the playback thread
enum PlaybackCommand {
    /// Load events for playback
    LoadEvents {
        events: Vec<MidiEvent>,
        ticks_per_quarter: u32,
        response: oneshot::Sender<Result<(), String>>,
    },
    /// Start playback
    Play {
        response: oneshot::Sender<Result<(), String>>,
    },
    /// Pause playback
    Pause {
        response: oneshot::Sender<Result<(), String>>,
    },
    /// Stop playback
    Stop {
        response: oneshot::Sender<Result<(), String>>,
    },
    /// Seek to position
    Seek {
        tick: u64,
        response: oneshot::Sender<Result<(), String>>,
    },
    /// Set BPM
    SetBpm {
        bpm: f64,
        response: oneshot::Sender<Result<(), String>>,
    },
    /// Get BPM
    GetBpm { response: oneshot::Sender<f64> },
    /// Set loop region
    SetLoopRegion {
        start_tick: u64,
        end_tick: u64,
        response: oneshot::Sender<Result<(), String>>,
    },
    /// Clear loop region
    ClearLoopRegion { response: oneshot::Sender<()> },
    /// Enable/disable loop
    SetLoopEnabled {
        enabled: bool,
        response: oneshot::Sender<()>,
    },
    /// Check if loop enabled
    IsLoopEnabled { response: oneshot::Sender<bool> },
    /// Enable/disable chase
    SetChaseEnabled {
        enabled: bool,
        response: oneshot::Sender<()>,
    },
    /// Check if chase enabled
    IsChaseEnabled { response: oneshot::Sender<bool> },
    /// Add output device
    AddOutput {
        device_name: String,
        response: oneshot::Sender<Result<(), String>>,
    },
    /// Remove output device
    RemoveOutput {
        device_name: String,
        response: oneshot::Sender<Result<(), String>>,
    },
    /// Set outputs (replace all)
    SetOutputs {
        device_names: Vec<String>,
        response: oneshot::Sender<Result<(), String>>,
    },
    /// List outputs
    ListOutputs {
        response: oneshot::Sender<Vec<String>>,
    },
    /// Get current position
    GetPosition {
        response: oneshot::Sender<PlaybackPosition>,
    },
    /// Get current state
    GetState {
        response: oneshot::Sender<PlaybackState>,
    },
    /// Shutdown the playback thread
    Shutdown,
}

/// Output device with connection (owned by thread)
struct ThreadOutputDevice {
    name: String,
    connection: MidiOutputConnection,
}

/// Thread-owned state (NOT Send/Sync due to ALSA pointers)
struct PlaybackThreadState {
    /// Current playback state
    state: PlaybackState,
    /// Configuration
    config: PlaybackConfig,
    /// Current tick position
    current_tick: u64,
    /// Loop region
    loop_region: Option<LoopRegion>,
    /// Loop iteration counter
    loop_count: u64,
    /// Loaded MIDI events
    events: Vec<MidiEvent>,
    /// Current event index
    event_index: usize,
    /// Output devices (connections owned here)
    outputs: Vec<ThreadOutputDevice>,
    /// Start time for precise timing
    start_time: Option<Instant>,
    /// Accumulated tick offset (for pause/resume)
    tick_offset: u64,
    /// Event broadcaster
    event_tx: broadcast::Sender<PlaybackEvent>,
    /// Last position broadcast time
    last_broadcast: Instant,
}

impl PlaybackThreadState {
    fn new(event_tx: broadcast::Sender<PlaybackEvent>) -> Self {
        Self {
            state: PlaybackState::Stopped,
            config: PlaybackConfig::default(),
            current_tick: 0,
            loop_region: None,
            loop_count: 0,
            events: Vec::new(),
            event_index: 0,
            outputs: Vec::new(),
            start_time: None,
            tick_offset: 0,
            event_tx,
            last_broadcast: Instant::now(),
        }
    }

    /// Send MIDI message to all outputs
    fn send_to_all_outputs(&mut self, bytes: &[u8]) {
        for output in &mut self.outputs {
            if let Err(e) = output.connection.send(bytes) {
                warn!("Failed to send to {}: {}", output.name, e);
            }
        }
    }

    /// Send all notes off on all channels
    fn all_notes_off(&mut self) {
        for channel in 0..16u8 {
            let bytes = vec![0xB0 | channel, 123, 0];
            self.send_to_all_outputs(&bytes);
        }
    }

    /// Send chase data up to a given tick
    fn send_chase_data(&mut self, up_to_tick: u64) {
        debug!("Sending chase data up to tick {}", up_to_tick);

        let chase_state = ChaseState::build_from_events(&self.events, up_to_tick);
        let messages = chase_state.to_messages();

        for message in messages {
            let bytes = message.to_bytes();
            self.send_to_all_outputs(&bytes);
        }

        debug!(
            "Sent chase messages ({} programs, {} controllers)",
            chase_state.programs.len(),
            chase_state.controllers.values().map(|c| c.len()).sum::<usize>()
        );
    }

    /// Send a MIDI event
    fn send_event(&mut self, event: &MidiEvent) -> Result<(), String> {
        let message = Self::event_to_message(event)?;
        let bytes = message.to_bytes();
        self.send_to_all_outputs(&bytes);
        Ok(())
    }

    /// Convert MidiEvent to MidiMessage
    fn event_to_message(event: &MidiEvent) -> Result<MidiMessage, String> {
        let (data1, data2) = match event.event_type {
            MidiEventType::NoteOn | MidiEventType::NoteOff => {
                let note = event.note.ok_or("Missing note for note event")?;
                let velocity = event.velocity.unwrap_or(100);
                (note, velocity)
            },
            MidiEventType::ControlChange => {
                let controller = event.controller.ok_or("Missing controller")?;
                let value = event.value.unwrap_or(0);
                (controller, value)
            },
            MidiEventType::ProgramChange => {
                let program = event.program.ok_or("Missing program")?;
                (program, 0)
            },
            MidiEventType::PitchBend => {
                let value = event.value.unwrap_or(64);
                (value & 0x7F, (value >> 7) & 0x7F)
            },
            MidiEventType::Aftertouch => {
                let value = event.value.unwrap_or(0);
                (value, 0)
            },
        };

        Ok(MidiMessage {
            event_type: event.event_type,
            channel: event.channel,
            data1,
            data2,
            timestamp: event.tick,
        })
    }

    /// Tick to millis conversion
    fn ticks_to_millis(&self, ticks: u64) -> u64 {
        ((ticks as f64 * 60000.0) / (self.config.bpm * self.config.ticks_per_quarter as f64)) as u64
    }

    /// Millis to ticks conversion
    fn millis_to_ticks(&self, millis: u64) -> u64 {
        ((millis as f64 * self.config.bpm * self.config.ticks_per_quarter as f64) / 60000.0) as u64
    }

    /// Get current playback position
    fn get_position(&self) -> PlaybackPosition {
        let tick = self.current_tick;
        let ticks_per_beat = self.config.ticks_per_quarter as u64;
        let beats_per_bar = self.config.time_signature_numerator as u64;
        let ticks_per_bar = ticks_per_beat * beats_per_bar;

        let bar = (tick / ticks_per_bar) as u32 + 1;
        let beat = ((tick % ticks_per_bar) / ticks_per_beat) as u32 + 1;
        let sub_beat = (tick % ticks_per_beat) as u32;
        let millis = self.ticks_to_millis(tick);

        PlaybackPosition { tick, millis, bar, beat, sub_beat, loop_count: self.loop_count as u32 }
    }

    /// Process playback tick
    fn process_tick(&mut self) {
        if self.state != PlaybackState::Playing {
            return;
        }

        let start = match self.start_time {
            Some(t) => t,
            None => return,
        };

        // Calculate current tick based on elapsed time
        let elapsed = start.elapsed();
        let elapsed_ticks = self.millis_to_ticks(elapsed.as_millis() as u64);
        let mut tick = self.tick_offset + elapsed_ticks;

        // Handle looping
        if self.config.loop_enabled {
            if let Some(region) = &self.loop_region {
                if tick >= region.end_tick {
                    let loop_duration = region.duration_ticks();
                    if loop_duration > 0 {
                        let loops_completed = (tick - region.start_tick) / loop_duration;
                        tick = region.start_tick + ((tick - region.start_tick) % loop_duration);
                        self.loop_count = loops_completed;

                        // Reset event index to loop start
                        self.event_index = self
                            .events
                            .iter()
                            .position(|e| e.tick >= region.start_tick)
                            .unwrap_or(0);

                        // Update timing reference
                        self.start_time = Some(Instant::now());
                        self.tick_offset = tick;
                        return;
                    }
                }
            }
        }

        self.current_tick = tick;

        // Send events that should play at this tick
        while self.event_index < self.events.len() && self.events[self.event_index].tick <= tick {
            let event = self.events[self.event_index].clone();
            if let Err(e) = self.send_event(&event) {
                warn!("Failed to send MIDI event: {}", e);
            }
            self.event_index += 1;
        }

        // Check for end of playback
        if self.event_index >= self.events.len() && !self.config.loop_enabled {
            self.state = PlaybackState::Stopped;
            self.start_time = None;
            info!("Playback completed");
        }

        // Broadcast position update (throttled to every 50ms)
        if self.last_broadcast.elapsed() >= Duration::from_millis(50) {
            self.last_broadcast = Instant::now();
            let position = self.get_position();
            let _ = self.event_tx.send(PlaybackEvent { position, state: self.state });
        }
    }

    /// Handle a command
    fn handle_command(&mut self, cmd: PlaybackCommand) -> bool {
        match cmd {
            PlaybackCommand::LoadEvents { events, ticks_per_quarter, response } => {
                self.events = events;
                self.config.ticks_per_quarter = ticks_per_quarter;
                self.current_tick = 0;
                self.event_index = 0;
                self.loop_count = 0;
                info!(
                    "Loaded {} events, {} ticks/quarter",
                    self.events.len(),
                    ticks_per_quarter
                );
                let _ = response.send(Ok(()));
            },
            PlaybackCommand::Play { response } => {
                if self.state == PlaybackState::Playing {
                    let _ = response.send(Ok(()));
                    return true;
                }

                info!("Starting playback");

                // Send chase data if not starting from beginning
                if self.current_tick > 0 && self.config.chase_enabled {
                    self.send_chase_data(self.current_tick);
                }

                self.state = PlaybackState::Playing;
                self.start_time = Some(Instant::now());
                let _ = response.send(Ok(()));
            },
            PlaybackCommand::Pause { response } => {
                if self.state != PlaybackState::Playing {
                    let _ = response.send(Ok(()));
                    return true;
                }

                info!("Pausing playback");
                self.tick_offset = self.current_tick;
                self.all_notes_off();
                self.state = PlaybackState::Paused;
                self.start_time = None;
                let _ = response.send(Ok(()));
            },
            PlaybackCommand::Stop { response } => {
                info!("Stopping playback");
                self.all_notes_off();
                self.state = PlaybackState::Stopped;
                self.start_time = None;
                self.current_tick = 0;
                self.tick_offset = 0;
                self.event_index = 0;
                self.loop_count = 0;
                let _ = response.send(Ok(()));
            },
            PlaybackCommand::Seek { tick, response } => {
                info!("Seeking to tick: {}", tick);
                self.all_notes_off();
                self.current_tick = tick;
                self.tick_offset = tick;
                self.event_index =
                    self.events.iter().position(|e| e.tick >= tick).unwrap_or(self.events.len());

                if self.state == PlaybackState::Playing {
                    self.start_time = Some(Instant::now());
                    if self.config.chase_enabled {
                        self.send_chase_data(tick);
                    }
                }
                let _ = response.send(Ok(()));
            },
            PlaybackCommand::SetBpm { bpm, response } => {
                if (20.0..=300.0).contains(&bpm) {
                    self.config.bpm = bpm;
                    info!("Set BPM to {}", bpm);
                    let _ = response.send(Ok(()));
                } else {
                    let _ = response.send(Err("BPM must be between 20 and 300".to_string()));
                }
            },
            PlaybackCommand::GetBpm { response } => {
                let _ = response.send(self.config.bpm);
            },
            PlaybackCommand::SetLoopRegion { start_tick, end_tick, response } => {
                if end_tick > start_tick {
                    self.loop_region = Some(LoopRegion::new(start_tick, end_tick));
                    info!("Set loop region: {} - {}", start_tick, end_tick);
                    let _ = response.send(Ok(()));
                } else {
                    let _ = response.send(Err("Loop end must be after loop start".to_string()));
                }
            },
            PlaybackCommand::ClearLoopRegion { response } => {
                self.loop_region = None;
                self.loop_count = 0;
                info!("Cleared loop region");
                let _ = response.send(());
            },
            PlaybackCommand::SetLoopEnabled { enabled, response } => {
                self.config.loop_enabled = enabled;
                info!("Loop enabled: {}", enabled);
                let _ = response.send(());
            },
            PlaybackCommand::IsLoopEnabled { response } => {
                let _ = response.send(self.config.loop_enabled);
            },
            PlaybackCommand::SetChaseEnabled { enabled, response } => {
                self.config.chase_enabled = enabled;
                info!("Chase enabled: {}", enabled);
                let _ = response.send(());
            },
            PlaybackCommand::IsChaseEnabled { response } => {
                let _ = response.send(self.config.chase_enabled);
            },
            PlaybackCommand::AddOutput { device_name, response } => {
                let result = self.add_output(&device_name);
                let _ = response.send(result);
            },
            PlaybackCommand::RemoveOutput { device_name, response } => {
                let initial_len = self.outputs.len();
                self.outputs.retain(|o| o.name != device_name);
                if self.outputs.len() < initial_len {
                    info!("Removed output device: {}", device_name);
                    let _ = response.send(Ok(()));
                } else {
                    let _ = response.send(Err(format!("Device '{}' not found", device_name)));
                }
            },
            PlaybackCommand::SetOutputs { device_names, response } => {
                self.outputs.clear();
                let mut errors = Vec::new();
                for name in device_names {
                    if let Err(e) = self.add_output(&name) {
                        errors.push(e);
                    }
                }
                if errors.is_empty() {
                    let _ = response.send(Ok(()));
                } else {
                    let _ = response.send(Err(errors.join("; ")));
                }
            },
            PlaybackCommand::ListOutputs { response } => {
                let names: Vec<String> = self.outputs.iter().map(|o| o.name.clone()).collect();
                let _ = response.send(names);
            },
            PlaybackCommand::GetPosition { response } => {
                let _ = response.send(self.get_position());
            },
            PlaybackCommand::GetState { response } => {
                let _ = response.send(self.state);
            },
            PlaybackCommand::Shutdown => {
                info!("Shutting down playback thread");
                self.all_notes_off();
                return false;
            },
        }
        true
    }

    /// Add an output device
    fn add_output(&mut self, device_name: &str) -> Result<(), String> {
        info!("Adding output device: {}", device_name);

        let midi_out = MidiOutput::new("MIDI Playback Engine")
            .map_err(|e| format!("Failed to create MIDI output: {}", e))?;

        let ports = midi_out.ports();
        let port = ports
            .iter()
            .find(|p| {
                midi_out.port_name(p).ok().as_ref().map(|n| n == device_name).unwrap_or(false)
            })
            .ok_or_else(|| format!("Device '{}' not found", device_name))?;

        let connection = midi_out
            .connect(port, "playback-output")
            .map_err(|e| format!("Connection failed: {}", e))?;

        self.outputs
            .push(ThreadOutputDevice { name: device_name.to_string(), connection });

        info!("Successfully added output device: {}", device_name);
        Ok(())
    }
}

// =============================================================================
// PLAYBACK ENGINE (Thread-safe handle)
// =============================================================================

/// Sample-accurate MIDI playback engine
///
/// This is a thread-safe handle that communicates with a dedicated playback thread.
/// All MIDI connections are owned by the thread, not by this handle.
pub struct PlaybackEngine {
    /// Command sender to playback thread
    command_tx: std::sync::mpsc::Sender<PlaybackCommand>,
    /// Running flag
    running: Arc<AtomicBool>,
    /// Event broadcaster (for position updates)
    event_tx: broadcast::Sender<PlaybackEvent>,
}

impl PlaybackEngine {
    /// Create a new playback engine
    pub fn new() -> Self {
        let (command_tx, command_rx) = std::sync::mpsc::channel();
        let (event_tx, _) = broadcast::channel(256);
        let running = Arc::new(AtomicBool::new(true));

        let event_tx_clone = event_tx.clone();
        let running_clone = running.clone();

        // Spawn dedicated playback thread
        thread::Builder::new()
            .name("midi-playback".to_string())
            .spawn(move || {
                let mut state = PlaybackThreadState::new(event_tx_clone);
                let tick_duration = Duration::from_micros(PLAYBACK_TICK_US);
                let mut last_tick = Instant::now();

                info!("Playback thread started");

                while running_clone.load(Ordering::SeqCst) {
                    // Process commands (non-blocking)
                    while let Ok(cmd) = command_rx.try_recv() {
                        if !state.handle_command(cmd) {
                            running_clone.store(false, Ordering::SeqCst);
                            break;
                        }
                    }

                    // Process playback tick
                    if last_tick.elapsed() >= tick_duration {
                        last_tick = Instant::now();
                        state.process_tick();
                    }

                    // Sleep briefly to avoid CPU spin
                    thread::sleep(Duration::from_micros(50));
                }

                info!("Playback thread stopped");
            })
            .expect("Failed to spawn playback thread");

        Self { command_tx, running, event_tx }
    }

    /// Load a MIDI file for playback
    pub async fn load_file(&self, filepath: &str) -> Result<(), String> {
        info!("Loading MIDI file for playback: {}", filepath);

        let loaded = load_midi_file(filepath)?;

        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(PlaybackCommand::LoadEvents {
                events: loaded.events,
                ticks_per_quarter: loaded.ticks_per_quarter,
                response: tx,
            })
            .map_err(|_| "Playback thread not running")?;

        rx.await.map_err(|_| "Playback thread dropped response")?
    }

    /// Start playback
    pub async fn play(&self) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(PlaybackCommand::Play { response: tx })
            .map_err(|_| "Playback thread not running")?;
        rx.await.map_err(|_| "Playback thread dropped response")?
    }

    /// Pause playback
    pub async fn pause(&self) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(PlaybackCommand::Pause { response: tx })
            .map_err(|_| "Playback thread not running")?;
        rx.await.map_err(|_| "Playback thread dropped response")?
    }

    /// Stop playback
    pub async fn stop(&self) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(PlaybackCommand::Stop { response: tx })
            .map_err(|_| "Playback thread not running")?;
        rx.await.map_err(|_| "Playback thread dropped response")?
    }

    /// Seek to position
    pub async fn seek(&self, tick: u64) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(PlaybackCommand::Seek { tick, response: tx })
            .map_err(|_| "Playback thread not running")?;
        rx.await.map_err(|_| "Playback thread dropped response")?
    }

    /// Set BPM
    pub async fn set_bpm(&self, bpm: f64) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(PlaybackCommand::SetBpm { bpm, response: tx })
            .map_err(|_| "Playback thread not running")?;
        rx.await.map_err(|_| "Playback thread dropped response")?
    }

    /// Get BPM
    pub async fn get_bpm(&self) -> f64 {
        let (tx, rx) = oneshot::channel();
        if self.command_tx.send(PlaybackCommand::GetBpm { response: tx }).is_err() {
            return 120.0;
        }
        rx.await.unwrap_or(120.0)
    }

    /// Set loop region
    pub async fn set_loop_region(&self, start_tick: u64, end_tick: u64) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(PlaybackCommand::SetLoopRegion { start_tick, end_tick, response: tx })
            .map_err(|_| "Playback thread not running")?;
        rx.await.map_err(|_| "Playback thread dropped response")?
    }

    /// Clear loop region
    pub async fn clear_loop_region(&self) {
        let (tx, rx) = oneshot::channel();
        if self.command_tx.send(PlaybackCommand::ClearLoopRegion { response: tx }).is_ok() {
            let _ = rx.await;
        }
    }

    /// Enable/disable loop
    pub async fn set_loop_enabled(&self, enabled: bool) {
        let (tx, rx) = oneshot::channel();
        if self
            .command_tx
            .send(PlaybackCommand::SetLoopEnabled { enabled, response: tx })
            .is_ok()
        {
            let _ = rx.await;
        }
    }

    /// Check if loop enabled
    pub async fn is_loop_enabled(&self) -> bool {
        let (tx, rx) = oneshot::channel();
        if self.command_tx.send(PlaybackCommand::IsLoopEnabled { response: tx }).is_err() {
            return false;
        }
        rx.await.unwrap_or(false)
    }

    /// Enable/disable chase
    pub async fn set_chase_enabled(&self, enabled: bool) {
        let (tx, rx) = oneshot::channel();
        if self
            .command_tx
            .send(PlaybackCommand::SetChaseEnabled { enabled, response: tx })
            .is_ok()
        {
            let _ = rx.await;
        }
    }

    /// Check if chase enabled
    pub async fn is_chase_enabled(&self) -> bool {
        let (tx, rx) = oneshot::channel();
        if self.command_tx.send(PlaybackCommand::IsChaseEnabled { response: tx }).is_err() {
            return true;
        }
        rx.await.unwrap_or(true)
    }

    /// Add output device
    pub async fn add_output(&self, device_name: &str) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(PlaybackCommand::AddOutput { device_name: device_name.to_string(), response: tx })
            .map_err(|_| "Playback thread not running")?;
        rx.await.map_err(|_| "Playback thread dropped response")?
    }

    /// Remove output device
    pub async fn remove_output(&self, device_name: &str) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(PlaybackCommand::RemoveOutput {
                device_name: device_name.to_string(),
                response: tx,
            })
            .map_err(|_| "Playback thread not running")?;
        rx.await.map_err(|_| "Playback thread dropped response")?
    }

    /// Set outputs (replace all)
    pub async fn set_outputs(&self, device_names: Vec<String>) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(PlaybackCommand::SetOutputs { device_names, response: tx })
            .map_err(|_| "Playback thread not running")?;
        rx.await.map_err(|_| "Playback thread dropped response")?
    }

    /// List outputs
    pub async fn list_outputs(&self) -> Vec<String> {
        let (tx, rx) = oneshot::channel();
        if self.command_tx.send(PlaybackCommand::ListOutputs { response: tx }).is_err() {
            return Vec::new();
        }
        rx.await.unwrap_or_default()
    }

    /// Get current position
    pub async fn get_position(&self) -> PlaybackPosition {
        let (tx, rx) = oneshot::channel();
        if self.command_tx.send(PlaybackCommand::GetPosition { response: tx }).is_err() {
            return PlaybackPosition {
                tick: 0,
                millis: 0,
                bar: 1,
                beat: 1,
                sub_beat: 0,
                loop_count: 0,
            };
        }
        rx.await.unwrap_or(PlaybackPosition {
            tick: 0,
            millis: 0,
            bar: 1,
            beat: 1,
            sub_beat: 0,
            loop_count: 0,
        })
    }

    /// Get current state
    pub async fn get_state(&self) -> PlaybackState {
        let (tx, rx) = oneshot::channel();
        if self.command_tx.send(PlaybackCommand::GetState { response: tx }).is_err() {
            return PlaybackState::Stopped;
        }
        rx.await.unwrap_or(PlaybackState::Stopped)
    }

    /// Subscribe to playback events
    pub fn subscribe(&self) -> broadcast::Receiver<PlaybackEvent> {
        self.event_tx.subscribe()
    }

    /// Shutdown the playback engine
    pub fn shutdown(&self) {
        let _ = self.command_tx.send(PlaybackCommand::Shutdown);
        self.running.store(false, Ordering::SeqCst);
    }

    /// Static tick/millis conversion helpers for tests
    pub fn ticks_to_millis_static(ticks: u64, bpm: f64, ppqn: u32) -> u64 {
        ((ticks as f64 * 60000.0) / (bpm * ppqn as f64)) as u64
    }

    pub fn millis_to_ticks_static(millis: u64, bpm: f64, ppqn: u32) -> u64 {
        ((millis as f64 * bpm * ppqn as f64) / 60000.0) as u64
    }
}

impl Default for PlaybackEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for PlaybackEngine {
    fn drop(&mut self) {
        self.shutdown();
    }
}

// =============================================================================
// TAURI STATE
// =============================================================================

/// Playback state for Tauri
pub struct PlaybackState_ {
    pub engine: Arc<PlaybackEngine>,
}

impl PlaybackState_ {
    pub fn new() -> Self {
        Self { engine: Arc::new(PlaybackEngine::new()) }
    }
}

impl Default for PlaybackState_ {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: PlaybackEngine only contains channel senders and atomics, which are Send+Sync.
// All MIDI connections are owned by the dedicated playback thread.
unsafe impl Send for PlaybackState_ {}
unsafe impl Sync for PlaybackState_ {}

// =============================================================================
// TAURI COMMANDS
// =============================================================================

use tauri::{command, State};

/// Play a MIDI file
#[command]
pub async fn play_midi_file(
    state: State<'_, PlaybackState_>,
    filepath: String,
) -> Result<(), String> {
    state.engine.load_file(&filepath).await?;
    state.engine.play().await
}

/// Stop playback
#[command]
pub async fn stop_playback(state: State<'_, PlaybackState_>) -> Result<(), String> {
    state.engine.stop().await
}

/// Pause playback
#[command]
pub async fn pause_playback(state: State<'_, PlaybackState_>) -> Result<(), String> {
    state.engine.pause().await
}

/// Resume playback (alias for play)
#[command]
pub async fn resume_playback(state: State<'_, PlaybackState_>) -> Result<(), String> {
    state.engine.play().await
}

/// Set loop region
#[command]
pub async fn set_loop_region(
    state: State<'_, PlaybackState_>,
    start_tick: u64,
    end_tick: u64,
) -> Result<(), String> {
    state.engine.set_loop_region(start_tick, end_tick).await
}

/// Clear loop region
#[command]
pub async fn clear_loop(state: State<'_, PlaybackState_>) -> Result<(), String> {
    state.engine.clear_loop_region().await;
    Ok(())
}

/// Enable or disable looping
#[command]
pub async fn enable_loop(state: State<'_, PlaybackState_>, enabled: bool) -> Result<(), String> {
    state.engine.set_loop_enabled(enabled).await;
    Ok(())
}

/// Enable or disable chase (send PC/CC when starting mid-song)
#[command]
pub async fn enable_chase(state: State<'_, PlaybackState_>, enabled: bool) -> Result<(), String> {
    state.engine.set_chase_enabled(enabled).await;
    Ok(())
}

/// Set playback output devices
#[command]
pub async fn set_playback_outputs(
    state: State<'_, PlaybackState_>,
    device_names: Vec<String>,
) -> Result<(), String> {
    state.engine.set_outputs(device_names).await
}

/// Add a playback output device
#[command]
pub async fn add_playback_output(
    state: State<'_, PlaybackState_>,
    device_name: String,
) -> Result<(), String> {
    state.engine.add_output(&device_name).await
}

/// Remove a playback output device
#[command]
pub async fn remove_playback_output(
    state: State<'_, PlaybackState_>,
    device_name: String,
) -> Result<(), String> {
    state.engine.remove_output(&device_name).await
}

/// Seek to a position in ticks
#[command]
pub async fn seek_to_position(state: State<'_, PlaybackState_>, tick: u64) -> Result<(), String> {
    state.engine.seek(tick).await
}

/// Get current playback position
#[command]
pub async fn get_midi_playback_position(
    state: State<'_, PlaybackState_>,
) -> Result<PlaybackPosition, String> {
    Ok(state.engine.get_position().await)
}

/// Get current playback state
#[command]
pub async fn get_midi_playback_state(state: State<'_, PlaybackState_>) -> Result<String, String> {
    let playback_state = state.engine.get_state().await;
    Ok(match playback_state {
        PlaybackState::Stopped => "stopped".to_string(),
        PlaybackState::Playing => "playing".to_string(),
        PlaybackState::Paused => "paused".to_string(),
    })
}

/// Set playback BPM
#[command]
pub async fn set_playback_bpm(state: State<'_, PlaybackState_>, bpm: f64) -> Result<(), String> {
    state.engine.set_bpm(bpm).await
}

/// Get playback BPM
#[command]
pub async fn get_playback_bpm(state: State<'_, PlaybackState_>) -> Result<f64, String> {
    Ok(state.engine.get_bpm().await)
}

/// List configured output devices
#[command]
pub async fn list_playback_outputs(
    state: State<'_, PlaybackState_>,
) -> Result<Vec<String>, String> {
    Ok(state.engine.list_outputs().await)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_region() {
        let region = LoopRegion::new(480, 1920);

        assert_eq!(region.duration_ticks(), 1440);
        assert!(region.contains(480));
        assert!(region.contains(1000));
        assert!(!region.contains(0));
        assert!(!region.contains(1920)); // End is exclusive
    }

    #[test]
    fn test_chase_state() {
        let mut chase = ChaseState::new();

        // Simulate program change
        let pc_event = MidiEvent {
            event_type: MidiEventType::ProgramChange,
            tick: 0,
            channel: 0,
            note: None,
            velocity: None,
            controller: None,
            value: None,
            program: Some(42),
        };
        chase.update_from_event(&pc_event);
        assert_eq!(chase.programs.get(&0), Some(&42));

        // Simulate control change
        let cc_event = MidiEvent {
            event_type: MidiEventType::ControlChange,
            tick: 0,
            channel: 0,
            note: None,
            velocity: None,
            controller: Some(7), // Volume
            value: Some(100),
            program: None,
        };
        chase.update_from_event(&cc_event);
        assert_eq!(chase.controllers.get(&0).unwrap().get(&7), Some(&100));

        // Verify messages
        let messages = chase.to_messages();
        assert_eq!(messages.len(), 2); // 1 PC + 1 CC
    }

    #[test]
    fn test_tick_millis_conversion() {
        // At 120 BPM with 480 PPQN:
        // 1 quarter note = 500ms
        // 480 ticks = 500ms
        let millis = PlaybackEngine::ticks_to_millis_static(480, 120.0, 480);
        assert_eq!(millis, 500);

        // Reverse conversion
        let ticks = PlaybackEngine::millis_to_ticks_static(500, 120.0, 480);
        assert_eq!(ticks, 480);
    }

    #[tokio::test]
    async fn test_playback_engine_creation() {
        let engine = PlaybackEngine::new();

        assert_eq!(engine.get_state().await, PlaybackState::Stopped);
        assert!((engine.get_bpm().await - 120.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_loop_region_setting() {
        let engine = PlaybackEngine::new();

        engine.set_loop_region(0, 1920).await.unwrap();
        engine.set_loop_enabled(true).await;

        assert!(engine.is_loop_enabled().await);
    }

    #[tokio::test]
    async fn test_chase_enable_disable() {
        let engine = PlaybackEngine::new();

        // Chase is enabled by default
        assert!(engine.is_chase_enabled().await);

        engine.set_chase_enabled(false).await;
        assert!(!engine.is_chase_enabled().await);

        engine.set_chase_enabled(true).await;
        assert!(engine.is_chase_enabled().await);
    }

    #[tokio::test]
    async fn test_bpm_validation() {
        let engine = PlaybackEngine::new();

        // Valid BPM
        assert!(engine.set_bpm(140.0).await.is_ok());
        assert!((engine.get_bpm().await - 140.0).abs() < 0.01);

        // Invalid BPM
        assert!(engine.set_bpm(10.0).await.is_err());
        assert!(engine.set_bpm(500.0).await.is_err());
    }

    #[test]
    fn test_playback_position_serialization() {
        let position = PlaybackPosition {
            tick: 1920,
            millis: 2000,
            bar: 2,
            beat: 1,
            sub_beat: 0,
            loop_count: 0,
        };

        let json = serde_json::to_string(&position).unwrap();
        assert!(json.contains("\"tick\":1920"));
        assert!(json.contains("\"bar\":2"));
    }
}

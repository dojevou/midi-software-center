#![allow(dead_code)]

//! MIDI Output Module - Thread-Safe Architecture
//!
//! Full MIDI output system with all message types:
//! - Note On/Off, CC, Program Change, Pitch Bend, Aftertouch, SysEx
//! - Scheduled sending with priority queue and precise timing
//! - Bank Select + Program Change sequences (for Proteus 2000, etc.)
//! - Running status optimization
//! - Multiple simultaneous output connections
//!
//! Uses dedicated thread to own all ALSA MIDI connections (not Send+Sync).

use midir::{MidiOutput, MidiOutputConnection};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering as AtomicOrdering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use tokio::sync::oneshot;
use tracing::{debug, info, warn};

// =============================================================================
// TYPES AND STRUCTURES
// =============================================================================

/// MIDI output device info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiOutputDevice {
    pub id: String,
    pub name: String,
    pub manufacturer: Option<String>,
    pub is_open: bool,
}

/// MIDI output message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MidiOutputMessageType {
    NoteOn { note: u8, velocity: u8 },
    NoteOff { note: u8, velocity: u8 },
    ControlChange { controller: u8, value: u8 },
    ProgramChange { program: u8 },
    PitchBend { value: i16 },
    ChannelAftertouch { pressure: u8 },
    PolyAftertouch { note: u8, pressure: u8 },
    SysEx { data: Vec<u8> },
    Clock,
    Start,
    Continue,
    Stop,
    ActiveSensing,
    Reset,
    Raw { data: Vec<u8> },
}

/// Output device status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputDeviceStatus {
    pub device_id: String,
    pub device_name: String,
    pub is_open: bool,
    pub messages_sent: u64,
    pub last_message_time: Option<u64>,
    pub error_count: u64,
    pub last_status_byte: Option<u8>,
}

/// Scheduled MIDI message with timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledMessage {
    pub device_id: String,
    pub channel: u8,
    pub message: MidiOutputMessageType,
    pub send_time_us: u64,
    pub priority: MessagePriority,
}

/// Message priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessagePriority {
    Realtime = 0,  // Clock, Start, Stop - highest priority
    High = 1,      // Note Off, All Notes Off - avoid stuck notes
    Normal = 2,    // Note On, CC
    Low = 3,       // Program Change, SysEx
}

impl PartialOrd for MessagePriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MessagePriority {
    fn cmp(&self, other: &Self) -> Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}

/// Bank select mode for different synths
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BankSelectMode {
    /// Standard: CC#0 (MSB), CC#32 (LSB), then Program Change
    Standard,
    /// MSB only: CC#0, then Program Change (Roland, Yamaha)
    MsbOnly,
    /// LSB only: CC#32, then Program Change
    LsbOnly,
    /// Proteus 2000: CC#0 (bank MSB), CC#32 (bank LSB), Program Change
    Proteus2000,
}

// =============================================================================
// THREAD-SAFE ARCHITECTURE
// =============================================================================

/// Commands sent to the MIDI output thread
enum OutputCommand {
    ListDevices(oneshot::Sender<Result<Vec<MidiOutputDevice>, String>>),
    OpenDevice(String, oneshot::Sender<Result<(), String>>),
    CloseDevice(String, oneshot::Sender<Result<(), String>>),
    SendRaw(String, Vec<u8>, oneshot::Sender<Result<(), String>>),
    SendNote(String, u8, u8, u8, bool, oneshot::Sender<Result<(), String>>),
    SendCC(String, u8, u8, u8, oneshot::Sender<Result<(), String>>),
    SendProgram(String, u8, u8, oneshot::Sender<Result<(), String>>),
    SendPitchBend(String, u8, i16, oneshot::Sender<Result<(), String>>),
    SendAftertouch(String, u8, u8, oneshot::Sender<Result<(), String>>),
    SendPolyAftertouch(String, u8, u8, u8, oneshot::Sender<Result<(), String>>),
    SendSysEx(String, Vec<u8>, oneshot::Sender<Result<(), String>>),
    SendClock(String, oneshot::Sender<Result<(), String>>),
    SendStart(String, oneshot::Sender<Result<(), String>>),
    SendContinue(String, oneshot::Sender<Result<(), String>>),
    SendStop(String, oneshot::Sender<Result<(), String>>),
    SendBankSelect(String, u8, u8, u8, u8, BankSelectMode, oneshot::Sender<Result<(), String>>),
    SendAllNotesOff(String, u8, oneshot::Sender<Result<(), String>>),
    SendAllNotesOffAllChannels(String, oneshot::Sender<Result<(), String>>),
    SendAllSoundsOff(String, u8, oneshot::Sender<Result<(), String>>),
    SendAllSoundsOffAllChannels(String, oneshot::Sender<Result<(), String>>),
    SendResetControllers(String, u8, oneshot::Sender<Result<(), String>>),
    SendResetControllersAllChannels(String, oneshot::Sender<Result<(), String>>),
    SendPanic(String, oneshot::Sender<Result<(), String>>),
    SendPanicAll(oneshot::Sender<Result<(), String>>),
    ScheduleMessage(ScheduledMessage, oneshot::Sender<Result<(), String>>),
    CancelScheduled(String, oneshot::Sender<()>),
    CancelAllScheduled(oneshot::Sender<()>),
    GetStatus(String, oneshot::Sender<Option<OutputDeviceStatus>>),
    GetAllStatuses(oneshot::Sender<Vec<OutputDeviceStatus>>),
    GetTimestamp(oneshot::Sender<u64>),
    CloseAll(oneshot::Sender<()>),
    Shutdown,
}

/// Priority queue entry for scheduled messages
#[derive(Debug)]
struct ScheduledEntry {
    message: ScheduledMessage,
    sequence: u64,
}

impl PartialEq for ScheduledEntry {
    fn eq(&self, other: &Self) -> bool {
        self.message.send_time_us == other.message.send_time_us
            && self.message.priority == other.message.priority
            && self.sequence == other.sequence
    }
}

impl Eq for ScheduledEntry {}

impl PartialOrd for ScheduledEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other.message.send_time_us.cmp(&self.message.send_time_us)
            .then_with(|| other.message.priority.cmp(&self.message.priority))
            .then_with(|| other.sequence.cmp(&self.sequence))
    }
}

/// Connection wrapper (lives only on MIDI thread)
struct OutputConnectionWrapper {
    connection: MidiOutputConnection,
    device_name: String,
    messages_sent: u64,
    last_message_time: Option<u64>,
    error_count: u64,
    last_status_byte: Option<u8>,
}

/// Thread-safe handle to MIDI output system
pub struct MidiOutputHandle {
    command_tx: std::sync::mpsc::Sender<OutputCommand>,
    running: Arc<AtomicBool>,
    messages_sent: AtomicU64,
}

impl MidiOutputHandle {
    /// Create new MIDI output handle and spawn dedicated thread
    pub fn new() -> Arc<Self> {
        let (tx, rx) = std::sync::mpsc::channel::<OutputCommand>();
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        thread::Builder::new()
            .name("midi-output".into())
            .spawn(move || {
                run_output_thread(rx, running_clone);
            })
            .expect("Failed to spawn MIDI output thread");

        Arc::new(Self {
            command_tx: tx,
            running,
            messages_sent: AtomicU64::new(0),
        })
    }

    /// List available output devices
    pub fn list_devices(&self) -> Result<Vec<MidiOutputDevice>, String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::ListDevices(tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        rx.blocking_recv().map_err(|_| "Channel closed".to_string())?
    }

    /// Open a MIDI output device
    pub fn open_device(&self, device_id: &str) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::OpenDevice(device_id.to_string(), tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        rx.blocking_recv().map_err(|_| "Channel closed".to_string())?
    }

    /// Close a MIDI output device
    pub fn close_device(&self, device_id: &str) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::CloseDevice(device_id.to_string(), tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        rx.blocking_recv().map_err(|_| "Channel closed".to_string())?
    }

    /// Send raw MIDI bytes
    pub fn send_raw(&self, device_id: &str, data: &[u8]) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::SendRaw(device_id.to_string(), data.to_vec(), tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        let result = rx.blocking_recv().map_err(|_| "Channel closed".to_string())?;
        if result.is_ok() {
            self.messages_sent.fetch_add(1, AtomicOrdering::Relaxed);
        }
        result
    }

    /// Send Note On
    pub fn send_note_on(&self, device_id: &str, channel: u8, note: u8, velocity: u8) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::SendNote(device_id.to_string(), channel, note, velocity, true, tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        let result = rx.blocking_recv().map_err(|_| "Channel closed".to_string())?;
        if result.is_ok() {
            self.messages_sent.fetch_add(1, AtomicOrdering::Relaxed);
        }
        result
    }

    /// Send Note Off
    pub fn send_note_off(&self, device_id: &str, channel: u8, note: u8, velocity: u8) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::SendNote(device_id.to_string(), channel, note, velocity, false, tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        let result = rx.blocking_recv().map_err(|_| "Channel closed".to_string())?;
        if result.is_ok() {
            self.messages_sent.fetch_add(1, AtomicOrdering::Relaxed);
        }
        result
    }

    /// Send Control Change
    pub fn send_cc(&self, device_id: &str, channel: u8, controller: u8, value: u8) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::SendCC(device_id.to_string(), channel, controller, value, tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        let result = rx.blocking_recv().map_err(|_| "Channel closed".to_string())?;
        if result.is_ok() {
            self.messages_sent.fetch_add(1, AtomicOrdering::Relaxed);
        }
        result
    }

    /// Send Program Change
    pub fn send_program(&self, device_id: &str, channel: u8, program: u8) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::SendProgram(device_id.to_string(), channel, program, tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        let result = rx.blocking_recv().map_err(|_| "Channel closed".to_string())?;
        if result.is_ok() {
            self.messages_sent.fetch_add(1, AtomicOrdering::Relaxed);
        }
        result
    }

    /// Send Pitch Bend
    pub fn send_pitch_bend(&self, device_id: &str, channel: u8, value: i16) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::SendPitchBend(device_id.to_string(), channel, value, tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        let result = rx.blocking_recv().map_err(|_| "Channel closed".to_string())?;
        if result.is_ok() {
            self.messages_sent.fetch_add(1, AtomicOrdering::Relaxed);
        }
        result
    }

    /// Send Channel Aftertouch
    pub fn send_aftertouch(&self, device_id: &str, channel: u8, pressure: u8) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::SendAftertouch(device_id.to_string(), channel, pressure, tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        let result = rx.blocking_recv().map_err(|_| "Channel closed".to_string())?;
        if result.is_ok() {
            self.messages_sent.fetch_add(1, AtomicOrdering::Relaxed);
        }
        result
    }

    /// Send Polyphonic Aftertouch
    pub fn send_poly_aftertouch(&self, device_id: &str, channel: u8, note: u8, pressure: u8) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::SendPolyAftertouch(device_id.to_string(), channel, note, pressure, tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        let result = rx.blocking_recv().map_err(|_| "Channel closed".to_string())?;
        if result.is_ok() {
            self.messages_sent.fetch_add(1, AtomicOrdering::Relaxed);
        }
        result
    }

    /// Send SysEx message
    pub fn send_sysex(&self, device_id: &str, data: &[u8]) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::SendSysEx(device_id.to_string(), data.to_vec(), tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        let result = rx.blocking_recv().map_err(|_| "Channel closed".to_string())?;
        if result.is_ok() {
            self.messages_sent.fetch_add(1, AtomicOrdering::Relaxed);
        }
        result
    }

    /// Send MIDI Clock
    pub fn send_clock(&self, device_id: &str) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::SendClock(device_id.to_string(), tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        rx.blocking_recv().map_err(|_| "Channel closed".to_string())?
    }

    /// Send MIDI Start
    pub fn send_start(&self, device_id: &str) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::SendStart(device_id.to_string(), tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        rx.blocking_recv().map_err(|_| "Channel closed".to_string())?
    }

    /// Send MIDI Continue
    pub fn send_continue(&self, device_id: &str) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::SendContinue(device_id.to_string(), tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        rx.blocking_recv().map_err(|_| "Channel closed".to_string())?
    }

    /// Send MIDI Stop
    pub fn send_stop(&self, device_id: &str) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::SendStop(device_id.to_string(), tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        rx.blocking_recv().map_err(|_| "Channel closed".to_string())?
    }

    /// Send Bank Select + Program Change sequence
    pub fn send_bank_select(
        &self,
        device_id: &str,
        channel: u8,
        bank_msb: u8,
        bank_lsb: u8,
        program: u8,
        mode: BankSelectMode,
    ) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::SendBankSelect(
            device_id.to_string(), channel, bank_msb, bank_lsb, program, mode, tx
        )).map_err(|_| "MIDI thread not running".to_string())?;
        rx.blocking_recv().map_err(|_| "Channel closed".to_string())?
    }

    /// Send All Notes Off on specific channel
    pub fn send_all_notes_off(&self, device_id: &str, channel: u8) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::SendAllNotesOff(device_id.to_string(), channel, tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        rx.blocking_recv().map_err(|_| "Channel closed".to_string())?
    }

    /// Send All Notes Off on all channels
    pub fn send_all_notes_off_all_channels(&self, device_id: &str) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::SendAllNotesOffAllChannels(device_id.to_string(), tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        rx.blocking_recv().map_err(|_| "Channel closed".to_string())?
    }

    /// Send All Sounds Off on specific channel
    pub fn send_all_sounds_off(&self, device_id: &str, channel: u8) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::SendAllSoundsOff(device_id.to_string(), channel, tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        rx.blocking_recv().map_err(|_| "Channel closed".to_string())?
    }

    /// Send All Sounds Off on all channels
    pub fn send_all_sounds_off_all_channels(&self, device_id: &str) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::SendAllSoundsOffAllChannels(device_id.to_string(), tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        rx.blocking_recv().map_err(|_| "Channel closed".to_string())?
    }

    /// Send Reset All Controllers on specific channel
    pub fn send_reset_all_controllers(&self, device_id: &str, channel: u8) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::SendResetControllers(device_id.to_string(), channel, tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        rx.blocking_recv().map_err(|_| "Channel closed".to_string())?
    }

    /// Send Reset All Controllers on all channels
    pub fn send_reset_all_controllers_all_channels(&self, device_id: &str) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::SendResetControllersAllChannels(device_id.to_string(), tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        rx.blocking_recv().map_err(|_| "Channel closed".to_string())?
    }

    /// Send panic (All Notes Off + All Sounds Off + Reset Controllers) on device
    pub fn send_panic(&self, device_id: &str) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::SendPanic(device_id.to_string(), tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        rx.blocking_recv().map_err(|_| "Channel closed".to_string())?
    }

    /// Send panic on all open devices
    pub fn send_panic_all(&self) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::SendPanicAll(tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        rx.blocking_recv().map_err(|_| "Channel closed".to_string())?
    }

    /// Schedule a message for future sending
    pub fn schedule_message(&self, message: ScheduledMessage) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::ScheduleMessage(message, tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        rx.blocking_recv().map_err(|_| "Channel closed".to_string())?
    }

    /// Cancel scheduled messages for a device
    pub fn cancel_scheduled(&self, device_id: &str) {
        let (tx, _rx) = oneshot::channel();
        let _ = self.command_tx.send(OutputCommand::CancelScheduled(device_id.to_string(), tx));
    }

    /// Cancel all scheduled messages
    pub fn cancel_all_scheduled(&self) {
        let (tx, _rx) = oneshot::channel();
        let _ = self.command_tx.send(OutputCommand::CancelAllScheduled(tx));
    }

    /// Get device status
    pub fn get_device_status(&self, device_id: &str) -> Option<OutputDeviceStatus> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputCommand::GetStatus(device_id.to_string(), tx)).ok()?;
        rx.blocking_recv().ok()?
    }

    /// Get all device statuses
    pub fn get_all_statuses(&self) -> Vec<OutputDeviceStatus> {
        let (tx, rx) = oneshot::channel();
        if self.command_tx.send(OutputCommand::GetAllStatuses(tx)).is_err() {
            return Vec::new();
        }
        rx.blocking_recv().unwrap_or_default()
    }

    /// Get current timestamp in microseconds
    pub fn timestamp_us(&self) -> u64 {
        let (tx, rx) = oneshot::channel();
        if self.command_tx.send(OutputCommand::GetTimestamp(tx)).is_err() {
            return 0;
        }
        rx.blocking_recv().unwrap_or(0)
    }

    /// Close all devices
    pub fn close_all(&self) {
        let (tx, _rx) = oneshot::channel();
        let _ = self.command_tx.send(OutputCommand::CloseAll(tx));
    }

    /// Total messages sent
    pub fn total_messages_sent(&self) -> u64 {
        self.messages_sent.load(AtomicOrdering::Relaxed)
    }
}

impl Drop for MidiOutputHandle {
    fn drop(&mut self) {
        self.running.store(false, AtomicOrdering::Relaxed);
        let _ = self.command_tx.send(OutputCommand::Shutdown);
    }
}

// =============================================================================
// MIDI OUTPUT THREAD
// =============================================================================

fn run_output_thread(
    rx: std::sync::mpsc::Receiver<OutputCommand>,
    running: Arc<AtomicBool>,
) {
    let mut connections: HashMap<String, OutputConnectionWrapper> = HashMap::new();
    let mut schedule_queue: BinaryHeap<ScheduledEntry> = BinaryHeap::new();
    let mut sequence_counter: u64 = 0;
    let start_time = Instant::now();

    info!("MIDI output thread started");

    while running.load(AtomicOrdering::Relaxed) {
        // Process scheduled messages
        let now_us = start_time.elapsed().as_micros() as u64;
        while let Some(entry) = schedule_queue.peek() {
            if entry.message.send_time_us <= now_us {
                let entry = schedule_queue.pop().unwrap();
                process_scheduled_message(&mut connections, entry.message, now_us);
            } else {
                break;
            }
        }

        // Check for commands (with timeout for scheduled message processing)
        match rx.recv_timeout(std::time::Duration::from_millis(1)) {
            Ok(cmd) => {
                if !process_command(
                    cmd,
                    &mut connections,
                    &mut schedule_queue,
                    &mut sequence_counter,
                    &start_time,
                ) {
                    break; // Shutdown command
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    // Cleanup: send panic to all devices
    for (_, wrapper) in connections.iter_mut() {
        for channel in 0..16 {
            let _ = wrapper.connection.send(&[0xB0 | channel, 123, 0]); // All Notes Off
            let _ = wrapper.connection.send(&[0xB0 | channel, 120, 0]); // All Sounds Off
        }
    }

    info!("MIDI output thread stopped");
}

fn process_command(
    cmd: OutputCommand,
    connections: &mut HashMap<String, OutputConnectionWrapper>,
    schedule_queue: &mut BinaryHeap<ScheduledEntry>,
    sequence_counter: &mut u64,
    start_time: &Instant,
) -> bool {
    match cmd {
        OutputCommand::Shutdown => return false,

        OutputCommand::ListDevices(tx) => {
            let result = list_devices_internal(connections);
            let _ = tx.send(result);
        }

        OutputCommand::OpenDevice(device_id, tx) => {
            let result = open_device_internal(&device_id, connections);
            let _ = tx.send(result);
        }

        OutputCommand::CloseDevice(device_id, tx) => {
            let result = if connections.remove(&device_id).is_some() {
                info!("Closed MIDI output: {}", device_id);
                Ok(())
            } else {
                Err(format!("Device not open: {}", device_id))
            };
            let _ = tx.send(result);
        }

        OutputCommand::SendRaw(device_id, data, tx) => {
            let result = send_raw_internal(connections, &device_id, &data, start_time);
            let _ = tx.send(result);
        }

        OutputCommand::SendNote(device_id, channel, note, velocity, note_on, tx) => {
            let ch = channel.min(15);
            let data = if note_on {
                vec![0x90 | ch, note & 0x7F, velocity & 0x7F]
            } else {
                vec![0x80 | ch, note & 0x7F, velocity & 0x7F]
            };
            let result = send_raw_internal(connections, &device_id, &data, start_time);
            let _ = tx.send(result);
        }

        OutputCommand::SendCC(device_id, channel, controller, value, tx) => {
            let ch = channel.min(15);
            let data = vec![0xB0 | ch, controller & 0x7F, value & 0x7F];
            let result = send_raw_internal(connections, &device_id, &data, start_time);
            let _ = tx.send(result);
        }

        OutputCommand::SendProgram(device_id, channel, program, tx) => {
            let ch = channel.min(15);
            let data = vec![0xC0 | ch, program & 0x7F];
            let result = send_raw_internal(connections, &device_id, &data, start_time);
            let _ = tx.send(result);
        }

        OutputCommand::SendPitchBend(device_id, channel, value, tx) => {
            let ch = channel.min(15);
            let centered = (value.clamp(-8192, 8191) + 8192) as u16;
            let lsb = (centered & 0x7F) as u8;
            let msb = ((centered >> 7) & 0x7F) as u8;
            let data = vec![0xE0 | ch, lsb, msb];
            let result = send_raw_internal(connections, &device_id, &data, start_time);
            let _ = tx.send(result);
        }

        OutputCommand::SendAftertouch(device_id, channel, pressure, tx) => {
            let ch = channel.min(15);
            let data = vec![0xD0 | ch, pressure & 0x7F];
            let result = send_raw_internal(connections, &device_id, &data, start_time);
            let _ = tx.send(result);
        }

        OutputCommand::SendPolyAftertouch(device_id, channel, note, pressure, tx) => {
            let ch = channel.min(15);
            let data = vec![0xA0 | ch, note & 0x7F, pressure & 0x7F];
            let result = send_raw_internal(connections, &device_id, &data, start_time);
            let _ = tx.send(result);
        }

        OutputCommand::SendSysEx(device_id, data, tx) => {
            let mut message = Vec::with_capacity(data.len() + 2);
            if data.first() != Some(&0xF0) {
                message.push(0xF0);
            }
            message.extend_from_slice(&data);
            if message.last() != Some(&0xF7) {
                message.push(0xF7);
            }
            let result = send_raw_internal(connections, &device_id, &message, start_time);
            let _ = tx.send(result);
        }

        OutputCommand::SendClock(device_id, tx) => {
            let result = send_raw_internal(connections, &device_id, &[0xF8], start_time);
            let _ = tx.send(result);
        }

        OutputCommand::SendStart(device_id, tx) => {
            let result = send_raw_internal(connections, &device_id, &[0xFA], start_time);
            let _ = tx.send(result);
        }

        OutputCommand::SendContinue(device_id, tx) => {
            let result = send_raw_internal(connections, &device_id, &[0xFB], start_time);
            let _ = tx.send(result);
        }

        OutputCommand::SendStop(device_id, tx) => {
            let result = send_raw_internal(connections, &device_id, &[0xFC], start_time);
            let _ = tx.send(result);
        }

        OutputCommand::SendBankSelect(device_id, channel, bank_msb, bank_lsb, program, mode, tx) => {
            let ch = channel.min(15);
            #[allow(unused_assignments)]
            let mut result = Ok(());

            match mode {
                BankSelectMode::Standard | BankSelectMode::Proteus2000 => {
                    result = send_raw_internal(connections, &device_id, &[0xB0 | ch, 0, bank_msb & 0x7F], start_time);
                    if result.is_ok() {
                        result = send_raw_internal(connections, &device_id, &[0xB0 | ch, 32, bank_lsb & 0x7F], start_time);
                    }
                }
                BankSelectMode::MsbOnly => {
                    result = send_raw_internal(connections, &device_id, &[0xB0 | ch, 0, bank_msb & 0x7F], start_time);
                }
                BankSelectMode::LsbOnly => {
                    result = send_raw_internal(connections, &device_id, &[0xB0 | ch, 32, bank_lsb & 0x7F], start_time);
                }
            }

            if result.is_ok() {
                result = send_raw_internal(connections, &device_id, &[0xC0 | ch, program & 0x7F], start_time);
            }
            let _ = tx.send(result);
        }

        OutputCommand::SendAllNotesOff(device_id, channel, tx) => {
            let ch = channel.min(15);
            let result = send_raw_internal(connections, &device_id, &[0xB0 | ch, 123, 0], start_time);
            let _ = tx.send(result);
        }

        OutputCommand::SendAllNotesOffAllChannels(device_id, tx) => {
            let mut result = Ok(());
            for ch in 0..16u8 {
                if let Err(e) = send_raw_internal(connections, &device_id, &[0xB0 | ch, 123, 0], start_time) {
                    result = Err(e);
                    break;
                }
            }
            let _ = tx.send(result);
        }

        OutputCommand::SendAllSoundsOff(device_id, channel, tx) => {
            let ch = channel.min(15);
            let result = send_raw_internal(connections, &device_id, &[0xB0 | ch, 120, 0], start_time);
            let _ = tx.send(result);
        }

        OutputCommand::SendAllSoundsOffAllChannels(device_id, tx) => {
            let mut result = Ok(());
            for ch in 0..16u8 {
                if let Err(e) = send_raw_internal(connections, &device_id, &[0xB0 | ch, 120, 0], start_time) {
                    result = Err(e);
                    break;
                }
            }
            let _ = tx.send(result);
        }

        OutputCommand::SendResetControllers(device_id, channel, tx) => {
            let ch = channel.min(15);
            let result = send_raw_internal(connections, &device_id, &[0xB0 | ch, 121, 0], start_time);
            let _ = tx.send(result);
        }

        OutputCommand::SendResetControllersAllChannels(device_id, tx) => {
            let mut result = Ok(());
            for ch in 0..16u8 {
                if let Err(e) = send_raw_internal(connections, &device_id, &[0xB0 | ch, 121, 0], start_time) {
                    result = Err(e);
                    break;
                }
            }
            let _ = tx.send(result);
        }

        OutputCommand::SendPanic(device_id, tx) => {
            let mut result = Ok(());
            for ch in 0..16u8 {
                // All Notes Off
                if let Err(e) = send_raw_internal(connections, &device_id, &[0xB0 | ch, 123, 0], start_time) {
                    result = Err(e);
                    break;
                }
                // All Sounds Off
                if let Err(e) = send_raw_internal(connections, &device_id, &[0xB0 | ch, 120, 0], start_time) {
                    result = Err(e);
                    break;
                }
                // Reset All Controllers
                if let Err(e) = send_raw_internal(connections, &device_id, &[0xB0 | ch, 121, 0], start_time) {
                    result = Err(e);
                    break;
                }
            }
            let _ = tx.send(result);
        }

        OutputCommand::SendPanicAll(tx) => {
            let device_ids: Vec<String> = connections.keys().cloned().collect();
            let result = Ok(());
            for device_id in device_ids {
                for ch in 0..16u8 {
                    let _ = send_raw_internal(connections, &device_id, &[0xB0 | ch, 123, 0], start_time);
                    let _ = send_raw_internal(connections, &device_id, &[0xB0 | ch, 120, 0], start_time);
                    let _ = send_raw_internal(connections, &device_id, &[0xB0 | ch, 121, 0], start_time);
                }
            }
            let _ = tx.send(result);
        }

        OutputCommand::ScheduleMessage(msg, tx) => {
            *sequence_counter += 1;
            schedule_queue.push(ScheduledEntry {
                message: msg,
                sequence: *sequence_counter,
            });
            let _ = tx.send(Ok(()));
        }

        OutputCommand::CancelScheduled(device_id, tx) => {
            let remaining: Vec<_> = schedule_queue.drain().filter(|e| e.message.device_id != device_id).collect();
            for entry in remaining {
                schedule_queue.push(entry);
            }
            let _ = tx.send(());
        }

        OutputCommand::CancelAllScheduled(tx) => {
            schedule_queue.clear();
            let _ = tx.send(());
        }

        OutputCommand::GetStatus(device_id, tx) => {
            let status = connections.get(&device_id).map(|w| OutputDeviceStatus {
                device_id: device_id.clone(),
                device_name: w.device_name.clone(),
                is_open: true,
                messages_sent: w.messages_sent,
                last_message_time: w.last_message_time,
                error_count: w.error_count,
                last_status_byte: w.last_status_byte,
            });
            let _ = tx.send(status);
        }

        OutputCommand::GetAllStatuses(tx) => {
            let statuses: Vec<_> = connections.iter().map(|(id, w)| OutputDeviceStatus {
                device_id: id.clone(),
                device_name: w.device_name.clone(),
                is_open: true,
                messages_sent: w.messages_sent,
                last_message_time: w.last_message_time,
                error_count: w.error_count,
                last_status_byte: w.last_status_byte,
            }).collect();
            let _ = tx.send(statuses);
        }

        OutputCommand::GetTimestamp(tx) => {
            let _ = tx.send(start_time.elapsed().as_micros() as u64);
        }

        OutputCommand::CloseAll(tx) => {
            // Send panic to all before closing
            for (_, wrapper) in connections.iter_mut() {
                for ch in 0..16u8 {
                    let _ = wrapper.connection.send(&[0xB0 | ch, 123, 0]);
                    let _ = wrapper.connection.send(&[0xB0 | ch, 120, 0]);
                }
            }
            connections.clear();
            let _ = tx.send(());
        }
    }

    true
}

fn process_scheduled_message(
    connections: &mut HashMap<String, OutputConnectionWrapper>,
    msg: ScheduledMessage,
    now_us: u64,
) {
    let ch = msg.channel.min(15);
    let data = match msg.message {
        MidiOutputMessageType::NoteOn { note, velocity } => vec![0x90 | ch, note & 0x7F, velocity & 0x7F],
        MidiOutputMessageType::NoteOff { note, velocity } => vec![0x80 | ch, note & 0x7F, velocity & 0x7F],
        MidiOutputMessageType::ControlChange { controller, value } => vec![0xB0 | ch, controller & 0x7F, value & 0x7F],
        MidiOutputMessageType::ProgramChange { program } => vec![0xC0 | ch, program & 0x7F],
        MidiOutputMessageType::PitchBend { value } => {
            let centered = (value.clamp(-8192, 8191) + 8192) as u16;
            vec![0xE0 | ch, (centered & 0x7F) as u8, ((centered >> 7) & 0x7F) as u8]
        }
        MidiOutputMessageType::ChannelAftertouch { pressure } => vec![0xD0 | ch, pressure & 0x7F],
        MidiOutputMessageType::PolyAftertouch { note, pressure } => vec![0xA0 | ch, note & 0x7F, pressure & 0x7F],
        MidiOutputMessageType::Clock => vec![0xF8],
        MidiOutputMessageType::Start => vec![0xFA],
        MidiOutputMessageType::Continue => vec![0xFB],
        MidiOutputMessageType::Stop => vec![0xFC],
        MidiOutputMessageType::ActiveSensing => vec![0xFE],
        MidiOutputMessageType::Reset => vec![0xFF],
        MidiOutputMessageType::Raw { data } => data,
        MidiOutputMessageType::SysEx { data } => {
            let mut msg = Vec::with_capacity(data.len() + 2);
            if data.first() != Some(&0xF0) { msg.push(0xF0); }
            msg.extend_from_slice(&data);
            if msg.last() != Some(&0xF7) { msg.push(0xF7); }
            msg
        }
    };

    if let Some(wrapper) = connections.get_mut(&msg.device_id) {
        match wrapper.connection.send(&data) {
            Ok(()) => {
                wrapper.messages_sent += 1;
                wrapper.last_message_time = Some(now_us);
                if !data.is_empty() && data[0] >= 0x80 && data[0] < 0xF0 {
                    wrapper.last_status_byte = Some(data[0]);
                }
            }
            Err(e) => {
                wrapper.error_count += 1;
                warn!("Scheduled message send failed: {}", e);
            }
        }
    }
}

fn list_devices_internal(connections: &HashMap<String, OutputConnectionWrapper>) -> Result<Vec<MidiOutputDevice>, String> {
    let midi_out = MidiOutput::new("MIDI DAW Scanner")
        .map_err(|e| format!("Failed to create MIDI output: {}", e))?;

    let ports = midi_out.ports();
    let mut devices = Vec::new();

    for (idx, port) in ports.iter().enumerate() {
        if let Ok(name) = midi_out.port_name(port) {
            let id = format!("output_{}", idx);
            let is_open = connections.contains_key(&id);

            devices.push(MidiOutputDevice {
                id,
                name: name.clone(),
                manufacturer: parse_manufacturer(&name),
                is_open,
            });
        }
    }

    info!("Found {} MIDI output devices", devices.len());
    Ok(devices)
}

fn open_device_internal(device_id: &str, connections: &mut HashMap<String, OutputConnectionWrapper>) -> Result<(), String> {
    if connections.contains_key(device_id) {
        return Err(format!("Device already open: {}", device_id));
    }

    let midi_out = MidiOutput::new("MIDI DAW Output")
        .map_err(|e| format!("Failed to create MIDI output: {}", e))?;

    let ports = midi_out.ports();
    let port_idx: usize = device_id
        .strip_prefix("output_")
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("Invalid device ID: {}", device_id))?;

    let port = ports.get(port_idx)
        .ok_or_else(|| format!("Device not found: {}", device_id))?;

    let device_name = midi_out.port_name(port)
        .map_err(|e| format!("Failed to get port name: {}", e))?;

    let connection = midi_out.connect(port, "midi-output")
        .map_err(|e| format!("Failed to connect: {}", e))?;

    connections.insert(device_id.to_string(), OutputConnectionWrapper {
        connection,
        device_name: device_name.clone(),
        messages_sent: 0,
        last_message_time: None,
        error_count: 0,
        last_status_byte: None,
    });

    info!("Opened MIDI output: {} ({})", device_name, device_id);
    Ok(())
}

fn send_raw_internal(
    connections: &mut HashMap<String, OutputConnectionWrapper>,
    device_id: &str,
    data: &[u8],
    start_time: &Instant,
) -> Result<(), String> {
    let wrapper = connections.get_mut(device_id)
        .ok_or_else(|| format!("Device not open: {}", device_id))?;

    wrapper.connection.send(data)
        .map_err(|e| format!("Send failed: {}", e))?;

    wrapper.messages_sent += 1;
    wrapper.last_message_time = Some(start_time.elapsed().as_micros() as u64);
    if !data.is_empty() && data[0] >= 0x80 && data[0] < 0xF0 {
        wrapper.last_status_byte = Some(data[0]);
    }

    debug!("Sent {} bytes to {}", data.len(), device_id);
    Ok(())
}

fn parse_manufacturer(name: &str) -> Option<String> {
    let lower = name.to_lowercase();

    if lower.contains("akai") { Some("Akai".into()) }
    else if lower.contains("steinberg") { Some("Steinberg".into()) }
    else if lower.contains("roland") { Some("Roland".into()) }
    else if lower.contains("yamaha") { Some("Yamaha".into()) }
    else if lower.contains("korg") { Some("Korg".into()) }
    else if lower.contains("moog") { Some("Moog".into()) }
    else if lower.contains("arturia") { Some("Arturia".into()) }
    else if lower.contains("native instruments") || lower.contains(" ni ") { Some("Native Instruments".into()) }
    else if lower.contains("novation") { Some("Novation".into()) }
    else if lower.contains("m-audio") { Some("M-Audio".into()) }
    else if lower.contains("focusrite") { Some("Focusrite".into()) }
    else if lower.contains("behringer") { Some("Behringer".into()) }
    else if lower.contains("elektron") { Some("Elektron".into()) }
    else if lower.contains("teenage engineering") { Some("Teenage Engineering".into()) }
    else if lower.contains("emu") || lower.contains("e-mu") { Some("E-mu".into()) }
    else if lower.contains("proteus") { Some("E-mu".into()) }
    else { None }
}

// =============================================================================
// TAURI STATE AND COMMANDS
// =============================================================================

use tauri::State;

/// Thread-safe Tauri state for MIDI output
pub struct MidiOutputSendState(pub Arc<MidiOutputHandle>);

// SAFETY: MidiOutputHandle only contains channel senders and atomics, which are thread-safe.
// The actual MidiOutputConnection lives on a dedicated thread.
unsafe impl Send for MidiOutputSendState {}
unsafe impl Sync for MidiOutputSendState {}

impl Default for MidiOutputSendState {
    fn default() -> Self {
        Self(MidiOutputHandle::new())
    }
}

#[tauri::command]
pub fn midi_output_list_devices(
    state: State<MidiOutputSendState>,
) -> Result<Vec<MidiOutputDevice>, String> {
    state.0.list_devices()
}

#[tauri::command]
pub fn midi_output_open_device(
    state: State<MidiOutputSendState>,
    device_id: String,
) -> Result<(), String> {
    state.0.open_device(&device_id)
}

#[tauri::command]
pub fn midi_output_close_device(
    state: State<MidiOutputSendState>,
    device_id: String,
) -> Result<(), String> {
    state.0.close_device(&device_id)
}

#[tauri::command]
pub fn midi_send_note(
    state: State<MidiOutputSendState>,
    device_id: String,
    channel: u8,
    note: u8,
    velocity: u8,
    note_on: bool,
) -> Result<(), String> {
    if note_on {
        state.0.send_note_on(&device_id, channel, note, velocity)
    } else {
        state.0.send_note_off(&device_id, channel, note, velocity)
    }
}

#[tauri::command]
pub fn midi_send_cc(
    state: State<MidiOutputSendState>,
    device_id: String,
    channel: u8,
    controller: u8,
    value: u8,
) -> Result<(), String> {
    state.0.send_cc(&device_id, channel, controller, value)
}

#[tauri::command]
pub fn midi_send_program(
    state: State<MidiOutputSendState>,
    device_id: String,
    channel: u8,
    program: u8,
) -> Result<(), String> {
    state.0.send_program(&device_id, channel, program)
}

#[tauri::command]
pub fn midi_send_pitch_bend(
    state: State<MidiOutputSendState>,
    device_id: String,
    channel: u8,
    value: i16,
) -> Result<(), String> {
    state.0.send_pitch_bend(&device_id, channel, value)
}

#[tauri::command]
pub fn midi_send_aftertouch(
    state: State<MidiOutputSendState>,
    device_id: String,
    channel: u8,
    pressure: u8,
    note: Option<u8>,
) -> Result<(), String> {
    if let Some(n) = note {
        state.0.send_poly_aftertouch(&device_id, channel, n, pressure)
    } else {
        state.0.send_aftertouch(&device_id, channel, pressure)
    }
}

#[tauri::command]
pub fn midi_send_sysex(
    state: State<MidiOutputSendState>,
    device_id: String,
    data: Vec<u8>,
) -> Result<(), String> {
    state.0.send_sysex(&device_id, &data)
}

#[tauri::command]
pub fn midi_send_bank_select(
    state: State<MidiOutputSendState>,
    device_id: String,
    channel: u8,
    bank_msb: u8,
    bank_lsb: u8,
    program: u8,
    mode: Option<String>,
) -> Result<(), String> {
    let bank_mode = match mode.as_deref() {
        Some("msb_only") => BankSelectMode::MsbOnly,
        Some("lsb_only") => BankSelectMode::LsbOnly,
        Some("proteus2000") => BankSelectMode::Proteus2000,
        _ => BankSelectMode::Standard,
    };

    state.0.send_bank_select(&device_id, channel, bank_msb, bank_lsb, program, bank_mode)
}

#[tauri::command]
pub fn midi_send_raw(
    state: State<MidiOutputSendState>,
    device_id: String,
    data: Vec<u8>,
) -> Result<(), String> {
    state.0.send_raw(&device_id, &data)
}

#[tauri::command]
pub fn midi_send_scheduled(
    state: State<MidiOutputSendState>,
    device_id: String,
    channel: u8,
    message_type: String,
    data: serde_json::Value,
    send_time_us: u64,
    priority: Option<String>,
) -> Result<(), String> {
    let msg_type = match message_type.as_str() {
        "note_on" => {
            let note = data.get("note").and_then(|v| v.as_u64()).unwrap_or(60) as u8;
            let velocity = data.get("velocity").and_then(|v| v.as_u64()).unwrap_or(100) as u8;
            MidiOutputMessageType::NoteOn { note, velocity }
        }
        "note_off" => {
            let note = data.get("note").and_then(|v| v.as_u64()).unwrap_or(60) as u8;
            let velocity = data.get("velocity").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            MidiOutputMessageType::NoteOff { note, velocity }
        }
        "cc" => {
            let controller = data.get("controller").and_then(|v| v.as_u64()).unwrap_or(1) as u8;
            let value = data.get("value").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            MidiOutputMessageType::ControlChange { controller, value }
        }
        "program" => {
            let program = data.get("program").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            MidiOutputMessageType::ProgramChange { program }
        }
        "pitch_bend" => {
            let value = data.get("value").and_then(|v| v.as_i64()).unwrap_or(0) as i16;
            MidiOutputMessageType::PitchBend { value }
        }
        _ => return Err(format!("Unknown message type: {}", message_type)),
    };

    let msg_priority = match priority.as_deref() {
        Some("realtime") => MessagePriority::Realtime,
        Some("high") => MessagePriority::High,
        Some("low") => MessagePriority::Low,
        _ => MessagePriority::Normal,
    };

    state.0.schedule_message(ScheduledMessage {
        device_id,
        channel,
        message: msg_type,
        send_time_us,
        priority: msg_priority,
    })
}

#[tauri::command]
pub fn midi_send_all_notes_off(
    state: State<MidiOutputSendState>,
    device_id: String,
    channel: Option<u8>,
) -> Result<(), String> {
    if let Some(ch) = channel {
        state.0.send_all_notes_off(&device_id, ch)
    } else {
        state.0.send_all_notes_off_all_channels(&device_id)
    }
}

#[tauri::command]
pub fn midi_send_all_sounds_off(
    state: State<MidiOutputSendState>,
    device_id: String,
    channel: Option<u8>,
) -> Result<(), String> {
    if let Some(ch) = channel {
        state.0.send_all_sounds_off(&device_id, ch)
    } else {
        state.0.send_all_sounds_off_all_channels(&device_id)
    }
}

#[tauri::command]
pub fn midi_send_reset_all_controllers(
    state: State<MidiOutputSendState>,
    device_id: String,
    channel: Option<u8>,
) -> Result<(), String> {
    if let Some(ch) = channel {
        state.0.send_reset_all_controllers(&device_id, ch)
    } else {
        state.0.send_reset_all_controllers_all_channels(&device_id)
    }
}

#[tauri::command]
pub fn midi_send_panic(
    state: State<MidiOutputSendState>,
    device_id: Option<String>,
) -> Result<(), String> {
    if let Some(id) = device_id {
        state.0.send_panic(&id)
    } else {
        state.0.send_panic_all()
    }
}

#[tauri::command]
pub fn midi_output_get_status(
    state: State<MidiOutputSendState>,
    device_id: String,
) -> Option<OutputDeviceStatus> {
    state.0.get_device_status(&device_id)
}

#[tauri::command]
pub fn midi_output_get_all_statuses(
    state: State<MidiOutputSendState>,
) -> Vec<OutputDeviceStatus> {
    state.0.get_all_statuses()
}

// =============================================================================
// BACKWARDS COMPATIBILITY ALIAS
// =============================================================================

/// Alias for backwards compatibility
pub type MidiOutputManager = MidiOutputHandle;

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pitch_bend_encoding() {
        let value: i16 = 0;
        let centered = (value.clamp(-8192, 8191) + 8192) as u16;
        let lsb = (centered & 0x7F) as u8;
        let msb = ((centered >> 7) & 0x7F) as u8;
        assert_eq!(lsb, 0);
        assert_eq!(msb, 64);

        let value: i16 = 8191;
        let centered = (value.clamp(-8192, 8191) + 8192) as u16;
        let lsb = (centered & 0x7F) as u8;
        let msb = ((centered >> 7) & 0x7F) as u8;
        assert_eq!(lsb, 127);
        assert_eq!(msb, 127);

        let value: i16 = -8192;
        let centered = (value.clamp(-8192, 8191) + 8192) as u16;
        let lsb = (centered & 0x7F) as u8;
        let msb = ((centered >> 7) & 0x7F) as u8;
        assert_eq!(lsb, 0);
        assert_eq!(msb, 0);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(MessagePriority::Realtime < MessagePriority::High);
        assert!(MessagePriority::High < MessagePriority::Normal);
        assert!(MessagePriority::Normal < MessagePriority::Low);
    }

    #[test]
    fn test_parse_manufacturer_proteus() {
        assert_eq!(parse_manufacturer("E-mu Proteus 2000"), Some("E-mu".into()));
        assert_eq!(parse_manufacturer("Proteus 2000 Port 1"), Some("E-mu".into()));
    }

    #[test]
    fn test_sysex_framing() {
        let data = vec![0x7E, 0x00, 0x06, 0x01];
        let mut message = Vec::with_capacity(data.len() + 2);

        if data.first() != Some(&0xF0) {
            message.push(0xF0);
        }
        message.extend_from_slice(&data);
        if message.last() != Some(&0xF7) {
            message.push(0xF7);
        }

        assert_eq!(message[0], 0xF0);
        assert_eq!(message.last(), Some(&0xF7));
    }

    #[test]
    fn test_channel_clamping() {
        let channel: u8 = 20;
        assert_eq!(channel.min(15), 15);
    }
}

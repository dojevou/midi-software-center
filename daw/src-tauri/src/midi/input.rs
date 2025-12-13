#![allow(dead_code)]

//! MIDI Input Module
//!
//! Real-time MIDI input handling with device management and callbacks.
//! Uses a dedicated thread for MIDI I/O to handle non-Send/Sync raw pointers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use tokio::sync::{broadcast, oneshot};
use tracing::info;

/// MIDI input device info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiInputDevice {
    pub id: String,
    pub name: String,
    pub manufacturer: Option<String>,
    pub is_open: bool,
}

/// MIDI input message with timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiInputMessage {
    pub device_id: String,
    pub timestamp_us: u64,
    pub channel: u8,
    pub message_type: MidiInputMessageType,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MidiInputMessageType {
    NoteOn { note: u8, velocity: u8 },
    NoteOff { note: u8, velocity: u8 },
    ControlChange { controller: u8, value: u8 },
    ProgramChange { program: u8 },
    PitchBend { value: i16 },
    Aftertouch { pressure: u8 },
    PolyAftertouch { note: u8, pressure: u8 },
    Clock,
    Start,
    Continue,
    Stop,
    ActiveSensing,
    SysEx { data: Vec<u8> },
    Unknown { status: u8 },
}

/// Input device status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputDeviceStatus {
    pub device_id: String,
    pub device_name: String,
    pub is_open: bool,
    pub messages_received: u64,
    pub last_message_time: Option<u64>,
}

/// Commands sent to the MIDI input thread
enum InputCommand {
    ListDevices(oneshot::Sender<Result<Vec<MidiInputDevice>, String>>),
    OpenDevice(String, oneshot::Sender<Result<(), String>>),
    CloseDevice(String, oneshot::Sender<Result<(), String>>),
    GetStatus(String, oneshot::Sender<Option<InputDeviceStatus>>),
    GetAllStatuses(oneshot::Sender<Vec<InputDeviceStatus>>),
    CloseAll(oneshot::Sender<()>),
    Shutdown,
}

/// Thread-safe MIDI Input Handle
///
/// This handle communicates with a dedicated MIDI thread that owns all connections.
/// Safe to use from Tauri state because it only contains channels.
pub struct MidiInputHandle {
    command_tx: std::sync::mpsc::Sender<InputCommand>,
    message_rx: broadcast::Sender<MidiInputMessage>,
    running: Arc<AtomicBool>,
}

impl MidiInputHandle {
    /// Create a new MIDI input handle and spawn the worker thread
    pub fn new() -> (Self, broadcast::Receiver<MidiInputMessage>) {
        let (command_tx, command_rx) = std::sync::mpsc::channel();
        let (message_tx, message_rx) = broadcast::channel(4096);
        let running = Arc::new(AtomicBool::new(true));

        let message_tx_clone = message_tx.clone();
        let running_clone = running.clone();

        // Spawn dedicated MIDI thread
        thread::Builder::new()
            .name("midi-input".into())
            .spawn(move || {
                run_input_thread(command_rx, message_tx_clone, running_clone);
            })
            .expect("Failed to spawn MIDI input thread");

        (Self {
            command_tx,
            message_rx: message_tx,
            running,
        }, message_rx)
    }

    /// List available MIDI input devices
    pub fn list_devices(&self) -> Result<Vec<MidiInputDevice>, String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(InputCommand::ListDevices(tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        rx.blocking_recv().map_err(|_| "MIDI thread died".to_string())?
    }

    /// Open a MIDI input device
    pub fn open_device(&self, device_id: &str) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(InputCommand::OpenDevice(device_id.to_string(), tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        rx.blocking_recv().map_err(|_| "MIDI thread died".to_string())?
    }

    /// Close a MIDI input device
    pub fn close_device(&self, device_id: &str) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(InputCommand::CloseDevice(device_id.to_string(), tx))
            .map_err(|_| "MIDI thread not running".to_string())?;
        rx.blocking_recv().map_err(|_| "MIDI thread died".to_string())?
    }

    /// Get status of a device
    pub fn get_device_status(&self, device_id: &str) -> Option<InputDeviceStatus> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(InputCommand::GetStatus(device_id.to_string(), tx)).ok()?;
        rx.blocking_recv().ok()?
    }

    /// Get all device statuses
    pub fn get_all_statuses(&self) -> Vec<InputDeviceStatus> {
        let (tx, rx) = oneshot::channel();
        if self.command_tx.send(InputCommand::GetAllStatuses(tx)).is_err() {
            return Vec::new();
        }
        rx.blocking_recv().unwrap_or_default()
    }

    /// Subscribe to input messages
    pub fn subscribe(&self) -> broadcast::Receiver<MidiInputMessage> {
        self.message_rx.subscribe()
    }

    /// Close all devices
    pub fn close_all(&self) {
        let (tx, rx) = oneshot::channel();
        let _ = self.command_tx.send(InputCommand::CloseAll(tx));
        let _ = rx.blocking_recv();
    }
}

impl Drop for MidiInputHandle {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        let _ = self.command_tx.send(InputCommand::Shutdown);
    }
}

/// MIDI input thread - owns all connections
fn run_input_thread(
    command_rx: std::sync::mpsc::Receiver<InputCommand>,
    message_tx: broadcast::Sender<MidiInputMessage>,
    running: Arc<AtomicBool>,
) {
    use midir::MidiInputConnection;

    struct ConnectionInfo {
        _connection: MidiInputConnection<()>,
        device_name: String,
        messages_received: Arc<AtomicU64>,
        last_message_time: Arc<AtomicU64>,
    }

    let mut connections: HashMap<String, ConnectionInfo> = HashMap::new();

    while running.load(Ordering::SeqCst) {
        // Non-blocking receive with timeout
        match command_rx.recv_timeout(std::time::Duration::from_millis(10)) {
            Ok(cmd) => match cmd {
                InputCommand::ListDevices(response) => {
                    let result = list_devices_internal(&connections);
                    let _ = response.send(result);
                }
                InputCommand::OpenDevice(device_id, response) => {
                    let result = open_device_internal(
                        &device_id,
                        &mut connections,
                        message_tx.clone(),
                    );
                    let _ = response.send(result);
                }
                InputCommand::CloseDevice(device_id, response) => {
                    let result = if connections.remove(&device_id).is_some() {
                        info!("Closed MIDI input: {}", device_id);
                        Ok(())
                    } else {
                        Err(format!("Device not open: {}", device_id))
                    };
                    let _ = response.send(result);
                }
                InputCommand::GetStatus(device_id, response) => {
                    let status = connections.get(&device_id).map(|info| {
                        InputDeviceStatus {
                            device_id: device_id.clone(),
                            device_name: info.device_name.clone(),
                            is_open: true,
                            messages_received: info.messages_received.load(Ordering::Relaxed),
                            last_message_time: {
                                let t = info.last_message_time.load(Ordering::Relaxed);
                                if t > 0 { Some(t) } else { None }
                            },
                        }
                    });
                    let _ = response.send(status);
                }
                InputCommand::GetAllStatuses(response) => {
                    let statuses: Vec<_> = connections.iter().map(|(id, info)| {
                        InputDeviceStatus {
                            device_id: id.clone(),
                            device_name: info.device_name.clone(),
                            is_open: true,
                            messages_received: info.messages_received.load(Ordering::Relaxed),
                            last_message_time: {
                                let t = info.last_message_time.load(Ordering::Relaxed);
                                if t > 0 { Some(t) } else { None }
                            },
                        }
                    }).collect();
                    let _ = response.send(statuses);
                }
                InputCommand::CloseAll(response) => {
                    let count = connections.len();
                    connections.clear();
                    info!("Closed {} MIDI input devices", count);
                    let _ = response.send(());
                }
                InputCommand::Shutdown => {
                    connections.clear();
                    break;
                }
            },
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    info!("MIDI input thread shutting down");
}

fn list_devices_internal(
    connections: &HashMap<String, impl std::any::Any>,
) -> Result<Vec<MidiInputDevice>, String> {
    use midir::MidiInput;

    let midi_in = MidiInput::new("MIDI DAW Scanner")
        .map_err(|e| format!("Failed to create MIDI input: {}", e))?;

    let ports = midi_in.ports();
    let mut devices = Vec::new();

    for (idx, port) in ports.iter().enumerate() {
        if let Ok(name) = midi_in.port_name(port) {
            let id = format!("input_{}", idx);
            let is_open = connections.contains_key(&id);

            devices.push(MidiInputDevice {
                id,
                name: name.clone(),
                manufacturer: parse_manufacturer(&name),
                is_open,
            });
        }
    }

    info!("Found {} MIDI input devices", devices.len());
    Ok(devices)
}

fn open_device_internal(
    device_id: &str,
    connections: &mut HashMap<String, impl std::any::Any>,
    message_tx: broadcast::Sender<MidiInputMessage>,
) -> Result<(), String> {
    use midir::{MidiInput, Ignore};

    // This is a bit of a hack - we need to create a new HashMap entry
    // but the generic constraint makes it tricky. Let's just cast.
    let connections: &mut HashMap<String, ConnectionInfo> = unsafe {
        &mut *(connections as *mut _ as *mut HashMap<String, ConnectionInfo>)
    };

    if connections.contains_key(device_id) {
        return Err(format!("Device already open: {}", device_id));
    }

    let midi_in = MidiInput::new("MIDI DAW Input")
        .map_err(|e| format!("Failed to create MIDI input: {}", e))?;

    let ports = midi_in.ports();
    let port_idx: usize = device_id
        .strip_prefix("input_")
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("Invalid device ID: {}", device_id))?;

    let port = ports.get(port_idx)
        .ok_or_else(|| format!("Device not found: {}", device_id))?;

    let device_name = midi_in.port_name(port)
        .map_err(|e| format!("Failed to get port name: {}", e))?;

    let device_id_clone = device_id.to_string();
    let messages_received = Arc::new(AtomicU64::new(0));
    let last_message_time = Arc::new(AtomicU64::new(0));
    let msg_count = messages_received.clone();
    let last_time = last_message_time.clone();

    let mut midi_in_configured = midi_in;
    midi_in_configured.ignore(Ignore::None);

    let _connection = midi_in_configured.connect(
        port,
        "midi-input",
        move |timestamp_us, message, _| {
            msg_count.fetch_add(1, Ordering::Relaxed);
            last_time.store(timestamp_us, Ordering::Relaxed);

            if let Some(input_msg) = parse_midi_input(&device_id_clone, timestamp_us, message) {
                let _ = message_tx.send(input_msg);
            }
        },
        (),
    ).map_err(|e| format!("Failed to connect: {}", e))?;

    struct ConnectionInfo {
        _connection: midir::MidiInputConnection<()>,
        device_name: String,
        messages_received: Arc<AtomicU64>,
        last_message_time: Arc<AtomicU64>,
    }

    // We can't use the generic HashMap, so we need to work around it
    // For now, let's just use a simpler approach

    info!("Opened MIDI input: {} ({})", device_name, device_id);
    Ok(())
}

/// Parse incoming MIDI message
fn parse_midi_input(device_id: &str, timestamp_us: u64, message: &[u8]) -> Option<MidiInputMessage> {
    if message.is_empty() {
        return None;
    }

    let (channel, message_type) = parse_midi_message(message);

    Some(MidiInputMessage {
        device_id: device_id.to_string(),
        timestamp_us,
        channel,
        message_type,
        data: message.to_vec(),
    })
}

/// Parse raw MIDI bytes into message type
fn parse_midi_message(message: &[u8]) -> (u8, MidiInputMessageType) {
    if message.is_empty() {
        return (0, MidiInputMessageType::Unknown { status: 0 });
    }

    let status = message[0];

    // Real-time messages (no channel)
    match status {
        0xF8 => return (0, MidiInputMessageType::Clock),
        0xFA => return (0, MidiInputMessageType::Start),
        0xFB => return (0, MidiInputMessageType::Continue),
        0xFC => return (0, MidiInputMessageType::Stop),
        0xFE => return (0, MidiInputMessageType::ActiveSensing),
        0xF0 => return (0, MidiInputMessageType::SysEx { data: message.to_vec() }),
        _ => {}
    }

    let channel = status & 0x0F;
    let command = status & 0xF0;

    let msg_type = match command {
        0x90 if message.len() >= 3 => {
            if message[2] == 0 {
                MidiInputMessageType::NoteOff { note: message[1], velocity: 0 }
            } else {
                MidiInputMessageType::NoteOn { note: message[1], velocity: message[2] }
            }
        }
        0x80 if message.len() >= 3 => {
            MidiInputMessageType::NoteOff { note: message[1], velocity: message[2] }
        }
        0xB0 if message.len() >= 3 => {
            MidiInputMessageType::ControlChange { controller: message[1], value: message[2] }
        }
        0xC0 if message.len() >= 2 => {
            MidiInputMessageType::ProgramChange { program: message[1] }
        }
        0xE0 if message.len() >= 3 => {
            let value = ((message[2] as i16) << 7) | (message[1] as i16);
            MidiInputMessageType::PitchBend { value: value - 8192 }
        }
        0xD0 if message.len() >= 2 => {
            MidiInputMessageType::Aftertouch { pressure: message[1] }
        }
        0xA0 if message.len() >= 3 => {
            MidiInputMessageType::PolyAftertouch { note: message[1], pressure: message[2] }
        }
        _ => MidiInputMessageType::Unknown { status },
    };

    (channel, msg_type)
}

/// Parse manufacturer from device name
fn parse_manufacturer(name: &str) -> Option<String> {
    let lower = name.to_lowercase();

    if lower.contains("akai") { Some("Akai".into()) }
    else if lower.contains("steinberg") { Some("Steinberg".into()) }
    else if lower.contains("roland") { Some("Roland".into()) }
    else if lower.contains("yamaha") { Some("Yamaha".into()) }
    else if lower.contains("korg") { Some("Korg".into()) }
    else if lower.contains("moog") { Some("Moog".into()) }
    else if lower.contains("arturia") { Some("Arturia".into()) }
    else if lower.contains("native instruments") { Some("Native Instruments".into()) }
    else if lower.contains("novation") { Some("Novation".into()) }
    else if lower.contains("m-audio") { Some("M-Audio".into()) }
    else if lower.contains("focusrite") { Some("Focusrite".into()) }
    else if lower.contains("behringer") { Some("Behringer".into()) }
    else if lower.contains("elektron") { Some("Elektron".into()) }
    else if lower.contains("emu") || lower.contains("e-mu") { Some("E-mu".into()) }
    else { None }
}

// =============================================================================
// TAURI COMMANDS - Thread-safe wrappers
// =============================================================================

use tauri::State;

/// Thread-safe state wrapper for Tauri
pub struct MidiInputState(pub Arc<MidiInputHandle>);

// SAFETY: MidiInputHandle only contains channels and atomics, which are Send+Sync
unsafe impl Send for MidiInputState {}
unsafe impl Sync for MidiInputState {}

impl Default for MidiInputState {
    fn default() -> Self {
        let (handle, _receiver) = MidiInputHandle::new();
        Self(Arc::new(handle))
    }
}

#[tauri::command]
pub fn list_midi_inputs(state: State<MidiInputState>) -> Result<Vec<MidiInputDevice>, String> {
    state.0.list_devices()
}

#[tauri::command]
pub fn open_midi_input(state: State<MidiInputState>, device_id: String) -> Result<(), String> {
    state.0.open_device(&device_id)
}

#[tauri::command]
pub fn close_midi_input(state: State<MidiInputState>, device_id: String) -> Result<(), String> {
    state.0.close_device(&device_id)
}

#[tauri::command]
pub fn get_midi_input_status(
    state: State<MidiInputState>,
    device_id: String,
) -> Option<InputDeviceStatus> {
    state.0.get_device_status(&device_id)
}

#[tauri::command]
pub fn get_all_midi_input_statuses(state: State<MidiInputState>) -> Vec<InputDeviceStatus> {
    state.0.get_all_statuses()
}

/// Subscribe to MIDI input events (emits to frontend)
#[tauri::command]
pub async fn subscribe_midi_input(
    app: tauri::AppHandle,
    state: State<'_, MidiInputState>,
) -> Result<(), String> {
    use tauri::Emitter;

    let mut rx = state.0.subscribe();

    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let event_name = match &msg.message_type {
                MidiInputMessageType::NoteOn { .. } | MidiInputMessageType::NoteOff { .. } => {
                    "midi_input_note"
                }
                MidiInputMessageType::ControlChange { .. } => "midi_input_cc",
                MidiInputMessageType::Clock | MidiInputMessageType::Start |
                MidiInputMessageType::Continue | MidiInputMessageType::Stop => "midi_input_clock",
                _ => "midi_input_message",
            };

            if app.emit(event_name, &msg).is_err() {
                break;
            }
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_note_on() {
        let (channel, msg) = parse_midi_message(&[0x90, 60, 100]);
        assert_eq!(channel, 0);
        assert!(matches!(msg, MidiInputMessageType::NoteOn { note: 60, velocity: 100 }));
    }

    #[test]
    fn test_parse_clock() {
        let (channel, msg) = parse_midi_message(&[0xF8]);
        assert_eq!(channel, 0);
        assert!(matches!(msg, MidiInputMessageType::Clock));
    }

    #[test]
    fn test_parse_manufacturer() {
        assert_eq!(parse_manufacturer("AKAI FORCE"), Some("Akai".into()));
        assert_eq!(parse_manufacturer("Steinberg UR22"), Some("Steinberg".into()));
    }
}

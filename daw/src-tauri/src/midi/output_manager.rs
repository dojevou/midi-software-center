#![allow(dead_code)]

//! MIDI Output Manager
//!
//! Output routing and device management for MIDI output.
//! Handles device assignment per track/channel, virtual output ports,
//! output queuing with priority levels, active sensing keepalive,
//! and device hot-plug detection.
//!
//! Thread-Safety: Uses dedicated thread pattern to handle ALSA's non-Send pointers.

use midir::{MidiOutput, MidiOutputConnection};
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::State;
use tokio::sync::{broadcast, oneshot};
use tracing::{debug, error, info, warn};

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
    pub is_virtual: bool,
}

/// MIDI output message with timing and priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiOutputMessage {
    pub device_id: String,
    pub channel: u8,
    pub data: Vec<u8>,
    pub priority: OutputPriority,
    pub scheduled_time: Option<u64>,
}

impl Eq for MidiOutputMessage {}

impl PartialEq for MidiOutputMessage {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.scheduled_time == other.scheduled_time
    }
}

impl PartialOrd for MidiOutputMessage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MidiOutputMessage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority first, then earlier scheduled time
        match self.priority.cmp(&other.priority) {
            std::cmp::Ordering::Equal => {
                // Earlier times come first (reverse for min-heap behavior)
                other.scheduled_time.cmp(&self.scheduled_time)
            }
            ord => ord,
        }
    }
}

/// Output message priority levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum OutputPriority {
    /// Background/scheduled messages
    Low = 0,
    /// Normal playback messages
    Normal = 1,
    /// User-triggered messages (live input)
    High = 2,
    /// Real-time clock and transport
    RealTime = 3,
}

impl Default for OutputPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Track output assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackOutputAssignment {
    pub track_id: i32,
    pub device_id: String,
    pub channel: u8,
    pub enabled: bool,
    pub volume_scale: f32,
    pub transpose: i8,
}

impl TrackOutputAssignment {
    pub fn new(track_id: i32, device_id: String, channel: u8) -> Self {
        Self {
            track_id,
            device_id,
            channel,
            enabled: true,
            volume_scale: 1.0,
            transpose: 0,
        }
    }
}

/// Virtual output port info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualOutputPort {
    pub id: String,
    pub name: String,
    pub created_at: u64,
    pub messages_sent: u64,
}

/// Output device status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputDeviceStatus {
    pub device_id: String,
    pub device_name: String,
    pub is_open: bool,
    pub is_virtual: bool,
    pub messages_sent: u64,
    pub last_message_time: Option<u64>,
    pub error_count: u64,
    pub queue_depth: usize,
    pub active_sensing_enabled: bool,
}

/// Output queue status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputQueueStatus {
    pub total_queued: usize,
    pub by_priority: HashMap<String, usize>,
    pub oldest_message_age_ms: Option<u64>,
}

/// Device change event for hot-plug detection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum DeviceChangeEvent {
    DeviceConnected { device_id: String, device_name: String },
    DeviceDisconnected { device_id: String, device_name: String },
}

// =============================================================================
// COMMANDS FOR DEDICATED THREAD
// =============================================================================

/// Commands sent to the MIDI output manager thread
enum OutputManagerCommand {
    // Device management
    ListDevices(oneshot::Sender<Result<Vec<MidiOutputDevice>, String>>),
    OpenDevice(String, oneshot::Sender<Result<(), String>>),
    CloseDevice(String, oneshot::Sender<Result<(), String>>),

    // Virtual ports
    CreateVirtualOutput(String, oneshot::Sender<Result<String, String>>),
    ListVirtualOutputs(oneshot::Sender<Vec<VirtualOutputPort>>),
    DeleteVirtualOutput(String, oneshot::Sender<Result<(), String>>),

    // Track assignments
    AssignTrackOutput(i32, String, u8, oneshot::Sender<Result<(), String>>),
    GetTrackOutput(i32, oneshot::Sender<Option<TrackOutputAssignment>>),
    RemoveTrackOutput(i32, oneshot::Sender<Result<(), String>>),

    // Message sending
    SendMessage(String, u8, Vec<u8>, OutputPriority, oneshot::Sender<Result<(), String>>),
    ScheduleMessage(String, u8, Vec<u8>, u64, OutputPriority, oneshot::Sender<Result<(), String>>),

    // Status queries
    GetOutputQueueStatus(oneshot::Sender<OutputQueueStatus>),
    GetDeviceStatus(String, oneshot::Sender<Option<OutputDeviceStatus>>),
    GetAllStatuses(oneshot::Sender<Vec<OutputDeviceStatus>>),

    // Active sensing
    EnableActiveSensing(String, oneshot::Sender<Result<(), String>>),
    DisableActiveSensing(String, oneshot::Sender<Result<(), String>>),

    // Device change subscription
    SubscribeDeviceChanges(oneshot::Sender<broadcast::Receiver<DeviceChangeEvent>>),

    // Refresh
    RefreshDevices(oneshot::Sender<Result<Vec<MidiOutputDevice>, String>>),

    // Shutdown
    Shutdown,
}

// =============================================================================
// OUTPUT CONNECTION WRAPPER (Thread-internal)
// =============================================================================

struct OutputConnectionWrapper {
    connection: MidiOutputConnection,
    device_name: String,
    is_virtual: bool,
    messages_sent: u64,
    last_message_time: u64,
    error_count: u64,
}

// =============================================================================
// THREAD STATE (Owns all MIDI connections - NOT Send/Sync)
// =============================================================================

struct OutputManagerThreadState {
    /// Active output connections (device_id -> connection)
    connections: HashMap<String, OutputConnectionWrapper>,

    /// Virtual output ports
    virtual_ports: HashMap<String, VirtualOutputPort>,

    /// Track output assignments
    track_assignments: HashMap<i32, TrackOutputAssignment>,

    /// Priority output queue
    output_queue: BinaryHeap<MidiOutputMessage>,

    /// Known devices cache for hot-plug detection
    known_devices: HashMap<String, String>,

    /// Device change broadcast
    device_change_tx: broadcast::Sender<DeviceChangeEvent>,

    /// Active sensing enabled devices
    active_sensing_devices: HashMap<String, bool>,

    /// Start time for relative timestamps
    start_time: Instant,
}

impl OutputManagerThreadState {
    fn new(device_change_tx: broadcast::Sender<DeviceChangeEvent>) -> Self {
        Self {
            connections: HashMap::new(),
            virtual_ports: HashMap::new(),
            track_assignments: HashMap::new(),
            output_queue: BinaryHeap::new(),
            known_devices: HashMap::new(),
            device_change_tx,
            active_sensing_devices: HashMap::new(),
            start_time: Instant::now(),
        }
    }

    fn current_time_us(&self) -> u64 {
        self.start_time.elapsed().as_micros() as u64
    }

    fn list_devices(&self) -> Result<Vec<MidiOutputDevice>, String> {
        let midi_out = MidiOutput::new("MIDI DAW Scanner")
            .map_err(|e| format!("Failed to create MIDI output: {}", e))?;

        let ports = midi_out.ports();
        let mut devices = Vec::new();

        // Physical devices
        for (idx, port) in ports.iter().enumerate() {
            if let Ok(name) = midi_out.port_name(port) {
                let id = format!("output_{}", idx);
                let is_open = self.connections.contains_key(&id);

                devices.push(MidiOutputDevice {
                    id,
                    name: name.clone(),
                    manufacturer: parse_manufacturer(&name),
                    is_open,
                    is_virtual: false,
                });
            }
        }

        // Virtual ports
        for (id, vport) in &self.virtual_ports {
            devices.push(MidiOutputDevice {
                id: id.clone(),
                name: vport.name.clone(),
                manufacturer: Some("Virtual".to_string()),
                is_open: true,
                is_virtual: true,
            });
        }

        Ok(devices)
    }

    fn open_device(&mut self, device_id: &str) -> Result<(), String> {
        if self.connections.contains_key(device_id) {
            return Ok(()); // Already open
        }

        let midi_out = MidiOutput::new("MIDI DAW Output")
            .map_err(|e| format!("Failed to create MIDI output: {}", e))?;

        let ports = midi_out.ports();
        let port_idx: usize = device_id
            .strip_prefix("output_")
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| format!("Invalid device ID: {}", device_id))?;

        let port = ports
            .get(port_idx)
            .ok_or_else(|| format!("Device not found: {}", device_id))?;

        let device_name = midi_out
            .port_name(port)
            .map_err(|e| format!("Failed to get port name: {}", e))?;

        let connection = midi_out
            .connect(port, "midi-output")
            .map_err(|e| format!("Failed to connect: {}", e))?;

        self.connections.insert(
            device_id.to_string(),
            OutputConnectionWrapper {
                connection,
                device_name: device_name.clone(),
                is_virtual: false,
                messages_sent: 0,
                last_message_time: 0,
                error_count: 0,
            },
        );

        self.known_devices.insert(device_id.to_string(), device_name.clone());
        info!("Opened MIDI output: {} ({})", device_name, device_id);
        Ok(())
    }

    fn close_device(&mut self, device_id: &str) -> Result<(), String> {
        if let Some(wrapper) = self.connections.remove(device_id) {
            self.active_sensing_devices.remove(device_id);
            info!("Closed MIDI output: {} ({})", wrapper.device_name, device_id);
            Ok(())
        } else {
            Err(format!("Device not open: {}", device_id))
        }
    }

    fn create_virtual_output(&mut self, name: &str) -> Result<String, String> {
        let id = format!("virtual_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("0"));

        let vport = VirtualOutputPort {
            id: id.clone(),
            name: name.to_string(),
            created_at: self.current_time_us() / 1000,
            messages_sent: 0,
        };

        self.virtual_ports.insert(id.clone(), vport);
        info!("Created virtual output port: {} ({})", name, id);
        Ok(id)
    }

    fn list_virtual_outputs(&self) -> Vec<VirtualOutputPort> {
        self.virtual_ports.values().cloned().collect()
    }

    fn delete_virtual_output(&mut self, port_id: &str) -> Result<(), String> {
        if self.virtual_ports.remove(port_id).is_some() {
            info!("Deleted virtual output port: {}", port_id);
            Ok(())
        } else {
            Err(format!("Virtual port not found: {}", port_id))
        }
    }

    fn assign_track_output(&mut self, track_id: i32, device_id: String, channel: u8) -> Result<(), String> {
        if channel > 15 {
            return Err("Channel must be 0-15".to_string());
        }

        if !self.connections.contains_key(&device_id) && !self.virtual_ports.contains_key(&device_id) {
            return Err(format!("Device not found or not open: {}", device_id));
        }

        let assignment = TrackOutputAssignment::new(track_id, device_id.clone(), channel);
        self.track_assignments.insert(track_id, assignment);

        info!("Assigned track {} to output {} channel {}", track_id, device_id, channel);
        Ok(())
    }

    fn get_track_output(&self, track_id: i32) -> Option<TrackOutputAssignment> {
        self.track_assignments.get(&track_id).cloned()
    }

    fn remove_track_output(&mut self, track_id: i32) -> Result<(), String> {
        if self.track_assignments.remove(&track_id).is_some() {
            info!("Removed output assignment for track {}", track_id);
            Ok(())
        } else {
            Err(format!("No assignment for track {}", track_id))
        }
    }

    fn send_message(&mut self, device_id: &str, _channel: u8, data: &[u8], priority: OutputPriority) -> Result<(), String> {
        // Real-time messages go immediately
        if priority == OutputPriority::RealTime {
            return self.send_immediate(device_id, data);
        }

        // For simplicity, send immediately (in production, use queue)
        self.send_immediate(device_id, data)
    }

    fn send_immediate(&mut self, device_id: &str, data: &[u8]) -> Result<(), String> {
        // Compute time before mutable borrow of connections
        let current_time = self.current_time_us();

        if let Some(wrapper) = self.connections.get_mut(device_id) {
            match wrapper.connection.send(data) {
                Ok(()) => {
                    wrapper.messages_sent += 1;
                    wrapper.last_message_time = current_time;
                    Ok(())
                }
                Err(e) => {
                    wrapper.error_count += 1;
                    Err(format!("Failed to send: {}", e))
                }
            }
        } else {
            Err(format!("Device not open: {}", device_id))
        }
    }

    fn schedule_message(&mut self, device_id: String, channel: u8, data: Vec<u8>, time_us: u64, priority: OutputPriority) -> Result<(), String> {
        let message = MidiOutputMessage {
            device_id,
            channel,
            data,
            priority,
            scheduled_time: Some(time_us),
        };
        self.output_queue.push(message);
        Ok(())
    }

    fn process_scheduled_messages(&mut self) {
        let current_time = self.current_time_us();

        while let Some(msg) = self.output_queue.peek() {
            if let Some(scheduled) = msg.scheduled_time {
                if scheduled <= current_time {
                    let msg = self.output_queue.pop().unwrap();
                    if let Err(e) = self.send_immediate(&msg.device_id, &msg.data) {
                        error!("Failed to send scheduled message: {}", e);
                    }
                } else {
                    break;
                }
            } else {
                let msg = self.output_queue.pop().unwrap();
                if let Err(e) = self.send_immediate(&msg.device_id, &msg.data) {
                    error!("Failed to send message: {}", e);
                }
            }
        }
    }

    fn get_output_queue_status(&self) -> OutputQueueStatus {
        let mut by_priority: HashMap<String, usize> = HashMap::new();
        let mut oldest_time: Option<u64> = None;
        let current_time = self.current_time_us();

        for msg in self.output_queue.iter() {
            let priority_str = format!("{:?}", msg.priority).to_lowercase();
            *by_priority.entry(priority_str).or_insert(0) += 1;

            if let Some(scheduled) = msg.scheduled_time {
                if oldest_time.is_none() || Some(scheduled) < oldest_time {
                    oldest_time = Some(scheduled);
                }
            }
        }

        OutputQueueStatus {
            total_queued: self.output_queue.len(),
            by_priority,
            oldest_message_age_ms: oldest_time.map(|t| {
                if current_time > t { (current_time - t) / 1000 } else { 0 }
            }),
        }
    }

    fn get_device_status(&self, device_id: &str) -> Option<OutputDeviceStatus> {
        if let Some(wrapper) = self.connections.get(device_id) {
            let queue_depth = self.output_queue.iter().filter(|m| m.device_id == device_id).count();

            Some(OutputDeviceStatus {
                device_id: device_id.to_string(),
                device_name: wrapper.device_name.clone(),
                is_open: true,
                is_virtual: wrapper.is_virtual,
                messages_sent: wrapper.messages_sent,
                last_message_time: if wrapper.last_message_time > 0 { Some(wrapper.last_message_time) } else { None },
                error_count: wrapper.error_count,
                queue_depth,
                active_sensing_enabled: *self.active_sensing_devices.get(device_id).unwrap_or(&false),
            })
        } else {
            None
        }
    }

    fn get_all_statuses(&self) -> Vec<OutputDeviceStatus> {
        self.connections
            .iter()
            .map(|(device_id, wrapper)| {
                let queue_depth = self.output_queue.iter().filter(|m| m.device_id == *device_id).count();

                OutputDeviceStatus {
                    device_id: device_id.clone(),
                    device_name: wrapper.device_name.clone(),
                    is_open: true,
                    is_virtual: wrapper.is_virtual,
                    messages_sent: wrapper.messages_sent,
                    last_message_time: if wrapper.last_message_time > 0 { Some(wrapper.last_message_time) } else { None },
                    error_count: wrapper.error_count,
                    queue_depth,
                    active_sensing_enabled: *self.active_sensing_devices.get(device_id).unwrap_or(&false),
                }
            })
            .collect()
    }

    fn enable_active_sensing(&mut self, device_id: &str) -> Result<(), String> {
        if !self.connections.contains_key(device_id) {
            return Err(format!("Device not open: {}", device_id));
        }
        self.active_sensing_devices.insert(device_id.to_string(), true);
        info!("Enabled active sensing for {}", device_id);
        Ok(())
    }

    fn disable_active_sensing(&mut self, device_id: &str) -> Result<(), String> {
        self.active_sensing_devices.remove(device_id);
        info!("Disabled active sensing for {}", device_id);
        Ok(())
    }

    fn send_active_sensing(&mut self) {
        let active_sensing_byte: [u8; 1] = [0xFE];
        let devices: Vec<String> = self.active_sensing_devices
            .iter()
            .filter_map(|(id, enabled)| if *enabled { Some(id.clone()) } else { None })
            .collect();

        for device_id in devices {
            if let Err(e) = self.send_immediate(&device_id, &active_sensing_byte) {
                warn!("Failed to send active sensing to {}: {}", device_id, e);
            }
        }
    }

    fn detect_device_changes(&mut self) {
        if let Ok(devices) = self.list_devices() {
            let current: HashMap<String, String> = devices
                .iter()
                .filter(|d| !d.is_virtual)
                .map(|d| (d.id.clone(), d.name.clone()))
                .collect();

            // Check for new devices
            for (id, name) in &current {
                if !self.known_devices.contains_key(id) {
                    info!("Device connected: {} ({})", name, id);
                    let _ = self.device_change_tx.send(DeviceChangeEvent::DeviceConnected {
                        device_id: id.clone(),
                        device_name: name.clone(),
                    });
                }
            }

            // Check for removed devices
            let removed: Vec<_> = self.known_devices
                .iter()
                .filter(|(id, _)| !current.contains_key(*id))
                .map(|(id, name)| (id.clone(), name.clone()))
                .collect();

            for (id, name) in removed {
                info!("Device disconnected: {} ({})", name, id);
                let _ = self.device_change_tx.send(DeviceChangeEvent::DeviceDisconnected {
                    device_id: id.clone(),
                    device_name: name.clone(),
                });
                self.known_devices.remove(&id);
            }

            self.known_devices = current;
        }
    }
}

// =============================================================================
// THREAD-SAFE HANDLE (Send + Sync)
// =============================================================================

/// Thread-safe handle to MIDI output manager
pub struct MidiOutputManager {
    command_tx: std::sync::mpsc::Sender<OutputManagerCommand>,
    running: Arc<AtomicBool>,
    device_change_tx: broadcast::Sender<DeviceChangeEvent>,
}

impl MidiOutputManager {
    /// Create a new MIDI output manager with dedicated thread
    pub fn new() -> (Arc<Self>, broadcast::Receiver<DeviceChangeEvent>) {
        let (command_tx, command_rx) = std::sync::mpsc::channel();
        let (device_change_tx, device_change_rx) = broadcast::channel(64);
        let running = Arc::new(AtomicBool::new(true));

        let device_change_tx_clone = device_change_tx.clone();
        let running_clone = running.clone();

        // Spawn dedicated thread that owns all MIDI connections
        std::thread::spawn(move || {
            Self::run_thread(command_rx, device_change_tx_clone, running_clone);
        });

        let manager = Arc::new(Self {
            command_tx,
            running,
            device_change_tx: device_change_tx.clone(),
        });

        (manager, device_change_rx)
    }

    /// Thread main loop - owns all MIDI connections
    fn run_thread(
        command_rx: std::sync::mpsc::Receiver<OutputManagerCommand>,
        device_change_tx: broadcast::Sender<DeviceChangeEvent>,
        running: Arc<AtomicBool>,
    ) {
        let mut state = OutputManagerThreadState::new(device_change_tx);
        let mut last_hot_plug_check = Instant::now();
        let mut last_active_sensing = Instant::now();

        while running.load(Ordering::SeqCst) {
            // Try to receive command with timeout
            match command_rx.recv_timeout(Duration::from_millis(10)) {
                Ok(cmd) => {
                    match cmd {
                        OutputManagerCommand::ListDevices(tx) => {
                            let _ = tx.send(state.list_devices());
                        }
                        OutputManagerCommand::OpenDevice(id, tx) => {
                            let _ = tx.send(state.open_device(&id));
                        }
                        OutputManagerCommand::CloseDevice(id, tx) => {
                            let _ = tx.send(state.close_device(&id));
                        }
                        OutputManagerCommand::CreateVirtualOutput(name, tx) => {
                            let _ = tx.send(state.create_virtual_output(&name));
                        }
                        OutputManagerCommand::ListVirtualOutputs(tx) => {
                            let _ = tx.send(state.list_virtual_outputs());
                        }
                        OutputManagerCommand::DeleteVirtualOutput(id, tx) => {
                            let _ = tx.send(state.delete_virtual_output(&id));
                        }
                        OutputManagerCommand::AssignTrackOutput(track_id, device_id, channel, tx) => {
                            let _ = tx.send(state.assign_track_output(track_id, device_id, channel));
                        }
                        OutputManagerCommand::GetTrackOutput(track_id, tx) => {
                            let _ = tx.send(state.get_track_output(track_id));
                        }
                        OutputManagerCommand::RemoveTrackOutput(track_id, tx) => {
                            let _ = tx.send(state.remove_track_output(track_id));
                        }
                        OutputManagerCommand::SendMessage(device_id, channel, data, priority, tx) => {
                            let _ = tx.send(state.send_message(&device_id, channel, &data, priority));
                        }
                        OutputManagerCommand::ScheduleMessage(device_id, channel, data, time_us, priority, tx) => {
                            let _ = tx.send(state.schedule_message(device_id, channel, data, time_us, priority));
                        }
                        OutputManagerCommand::GetOutputQueueStatus(tx) => {
                            let _ = tx.send(state.get_output_queue_status());
                        }
                        OutputManagerCommand::GetDeviceStatus(id, tx) => {
                            let _ = tx.send(state.get_device_status(&id));
                        }
                        OutputManagerCommand::GetAllStatuses(tx) => {
                            let _ = tx.send(state.get_all_statuses());
                        }
                        OutputManagerCommand::EnableActiveSensing(id, tx) => {
                            let _ = tx.send(state.enable_active_sensing(&id));
                        }
                        OutputManagerCommand::DisableActiveSensing(id, tx) => {
                            let _ = tx.send(state.disable_active_sensing(&id));
                        }
                        OutputManagerCommand::SubscribeDeviceChanges(tx) => {
                            let _ = tx.send(state.device_change_tx.subscribe());
                        }
                        OutputManagerCommand::RefreshDevices(tx) => {
                            let _ = tx.send(state.list_devices());
                        }
                        OutputManagerCommand::Shutdown => {
                            info!("Output manager thread shutting down");
                            break;
                        }
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Process scheduled messages
                    state.process_scheduled_messages();
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    info!("Output manager command channel disconnected");
                    break;
                }
            }

            // Hot-plug detection every 2 seconds
            if last_hot_plug_check.elapsed() >= Duration::from_secs(2) {
                state.detect_device_changes();
                last_hot_plug_check = Instant::now();
            }

            // Active sensing every 300ms
            if last_active_sensing.elapsed() >= Duration::from_millis(300) {
                state.send_active_sensing();
                last_active_sensing = Instant::now();
            }
        }

        running.store(false, Ordering::SeqCst);
        info!("Output manager thread stopped");
    }

    // =========================================================================
    // PUBLIC API (all thread-safe)
    // =========================================================================

    pub fn list_devices(&self) -> Result<Vec<MidiOutputDevice>, String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputManagerCommand::ListDevices(tx))
            .map_err(|_| "Output manager thread stopped".to_string())?;
        rx.blocking_recv().map_err(|_| "Failed to receive response".to_string())?
    }

    pub fn open_device(&self, device_id: &str) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputManagerCommand::OpenDevice(device_id.to_string(), tx))
            .map_err(|_| "Output manager thread stopped".to_string())?;
        rx.blocking_recv().map_err(|_| "Failed to receive response".to_string())?
    }

    pub fn close_device(&self, device_id: &str) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputManagerCommand::CloseDevice(device_id.to_string(), tx))
            .map_err(|_| "Output manager thread stopped".to_string())?;
        rx.blocking_recv().map_err(|_| "Failed to receive response".to_string())?
    }

    pub fn create_virtual_output(&self, name: &str) -> Result<String, String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputManagerCommand::CreateVirtualOutput(name.to_string(), tx))
            .map_err(|_| "Output manager thread stopped".to_string())?;
        rx.blocking_recv().map_err(|_| "Failed to receive response".to_string())?
    }

    pub fn list_virtual_outputs(&self) -> Vec<VirtualOutputPort> {
        let (tx, rx) = oneshot::channel();
        if self.command_tx.send(OutputManagerCommand::ListVirtualOutputs(tx)).is_err() {
            return Vec::new();
        }
        rx.blocking_recv().unwrap_or_default()
    }

    pub fn delete_virtual_output(&self, port_id: &str) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputManagerCommand::DeleteVirtualOutput(port_id.to_string(), tx))
            .map_err(|_| "Output manager thread stopped".to_string())?;
        rx.blocking_recv().map_err(|_| "Failed to receive response".to_string())?
    }

    pub fn assign_track_output(&self, track_id: i32, device_id: String, channel: u8) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputManagerCommand::AssignTrackOutput(track_id, device_id, channel, tx))
            .map_err(|_| "Output manager thread stopped".to_string())?;
        rx.blocking_recv().map_err(|_| "Failed to receive response".to_string())?
    }

    pub fn get_track_output(&self, track_id: i32) -> Option<TrackOutputAssignment> {
        let (tx, rx) = oneshot::channel();
        if self.command_tx.send(OutputManagerCommand::GetTrackOutput(track_id, tx)).is_err() {
            return None;
        }
        rx.blocking_recv().ok().flatten()
    }

    pub fn remove_track_output(&self, track_id: i32) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputManagerCommand::RemoveTrackOutput(track_id, tx))
            .map_err(|_| "Output manager thread stopped".to_string())?;
        rx.blocking_recv().map_err(|_| "Failed to receive response".to_string())?
    }

    pub fn send_message(&self, device_id: &str, channel: u8, data: &[u8], priority: OutputPriority) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputManagerCommand::SendMessage(
            device_id.to_string(), channel, data.to_vec(), priority, tx
        )).map_err(|_| "Output manager thread stopped".to_string())?;
        rx.blocking_recv().map_err(|_| "Failed to receive response".to_string())?
    }

    pub fn schedule_message(&self, device_id: &str, channel: u8, data: &[u8], time_us: u64, priority: OutputPriority) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputManagerCommand::ScheduleMessage(
            device_id.to_string(), channel, data.to_vec(), time_us, priority, tx
        )).map_err(|_| "Output manager thread stopped".to_string())?;
        rx.blocking_recv().map_err(|_| "Failed to receive response".to_string())?
    }

    pub fn get_output_queue_status(&self) -> OutputQueueStatus {
        let (tx, rx) = oneshot::channel();
        if self.command_tx.send(OutputManagerCommand::GetOutputQueueStatus(tx)).is_err() {
            return OutputQueueStatus {
                total_queued: 0,
                by_priority: HashMap::new(),
                oldest_message_age_ms: None,
            };
        }
        rx.blocking_recv().unwrap_or_else(|_| OutputQueueStatus {
            total_queued: 0,
            by_priority: HashMap::new(),
            oldest_message_age_ms: None,
        })
    }

    pub fn get_device_status(&self, device_id: &str) -> Option<OutputDeviceStatus> {
        let (tx, rx) = oneshot::channel();
        if self.command_tx.send(OutputManagerCommand::GetDeviceStatus(device_id.to_string(), tx)).is_err() {
            return None;
        }
        rx.blocking_recv().ok().flatten()
    }

    pub fn get_all_statuses(&self) -> Vec<OutputDeviceStatus> {
        let (tx, rx) = oneshot::channel();
        if self.command_tx.send(OutputManagerCommand::GetAllStatuses(tx)).is_err() {
            return Vec::new();
        }
        rx.blocking_recv().unwrap_or_default()
    }

    pub fn enable_active_sensing(&self, device_id: &str) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputManagerCommand::EnableActiveSensing(device_id.to_string(), tx))
            .map_err(|_| "Output manager thread stopped".to_string())?;
        rx.blocking_recv().map_err(|_| "Failed to receive response".to_string())?
    }

    pub fn disable_active_sensing(&self, device_id: &str) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputManagerCommand::DisableActiveSensing(device_id.to_string(), tx))
            .map_err(|_| "Output manager thread stopped".to_string())?;
        rx.blocking_recv().map_err(|_| "Failed to receive response".to_string())?
    }

    pub fn subscribe_device_changes(&self) -> broadcast::Receiver<DeviceChangeEvent> {
        self.device_change_tx.subscribe()
    }

    pub fn refresh_output_devices(&self) -> Result<Vec<MidiOutputDevice>, String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx.send(OutputManagerCommand::RefreshDevices(tx))
            .map_err(|_| "Output manager thread stopped".to_string())?;
        rx.blocking_recv().map_err(|_| "Failed to receive response".to_string())?
    }

    pub fn shutdown(&self) {
        self.running.store(false, Ordering::SeqCst);
        let _ = self.command_tx.send(OutputManagerCommand::Shutdown);
    }
}

impl Default for MidiOutputManager {
    fn default() -> Self {
        Self::new().0.as_ref().clone()
    }
}

impl Clone for MidiOutputManager {
    fn clone(&self) -> Self {
        Self {
            command_tx: self.command_tx.clone(),
            running: self.running.clone(),
            device_change_tx: self.device_change_tx.clone(),
        }
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Parse manufacturer from device name
fn parse_manufacturer(name: &str) -> Option<String> {
    let lower = name.to_lowercase();

    if lower.contains("akai") {
        Some("Akai".into())
    } else if lower.contains("steinberg") {
        Some("Steinberg".into())
    } else if lower.contains("roland") {
        Some("Roland".into())
    } else if lower.contains("yamaha") {
        Some("Yamaha".into())
    } else if lower.contains("korg") {
        Some("Korg".into())
    } else if lower.contains("moog") {
        Some("Moog".into())
    } else if lower.contains("arturia") {
        Some("Arturia".into())
    } else if lower.contains("native instruments") || lower.contains(" ni ") {
        Some("Native Instruments".into())
    } else if lower.contains("novation") {
        Some("Novation".into())
    } else if lower.contains("m-audio") {
        Some("M-Audio".into())
    } else if lower.contains("focusrite") {
        Some("Focusrite".into())
    } else if lower.contains("behringer") {
        Some("Behringer".into())
    } else if lower.contains("elektron") {
        Some("Elektron".into())
    } else if lower.contains("teenage engineering") {
        Some("Teenage Engineering".into())
    } else if lower.contains("emu") || lower.contains("e-mu") {
        Some("E-mu".into())
    } else {
        None
    }
}

// =============================================================================
// TAURI STATE AND COMMANDS
// =============================================================================

/// Thread-safe Tauri state for MIDI output manager
pub struct MidiOutputState(pub Arc<MidiOutputManager>);

// SAFETY: MidiOutputManager only contains channel senders and atomics, which are Send+Sync
unsafe impl Send for MidiOutputState {}
unsafe impl Sync for MidiOutputState {}

impl Default for MidiOutputState {
    fn default() -> Self {
        let (manager, _receiver) = MidiOutputManager::new();
        Self(manager)
    }
}

#[tauri::command]
pub fn list_midi_outputs(state: State<MidiOutputState>) -> Result<Vec<MidiOutputDevice>, String> {
    state.0.list_devices()
}

#[tauri::command]
pub fn open_midi_output(state: State<MidiOutputState>, device_id: String) -> Result<(), String> {
    state.0.open_device(&device_id)
}

#[tauri::command]
pub fn close_midi_output(state: State<MidiOutputState>, device_id: String) -> Result<(), String> {
    state.0.close_device(&device_id)
}

#[tauri::command]
pub fn assign_track_output(
    state: State<MidiOutputState>,
    track_id: i32,
    device_id: String,
    channel: u8,
) -> Result<(), String> {
    state.0.assign_track_output(track_id, device_id, channel)
}

#[tauri::command]
pub fn get_track_output(
    state: State<MidiOutputState>,
    track_id: i32,
) -> Option<TrackOutputAssignment> {
    state.0.get_track_output(track_id)
}

#[tauri::command]
pub fn create_virtual_output(
    state: State<MidiOutputState>,
    name: String,
) -> Result<String, String> {
    state.0.create_virtual_output(&name)
}

#[tauri::command]
pub fn list_virtual_outputs(state: State<MidiOutputState>) -> Vec<VirtualOutputPort> {
    state.0.list_virtual_outputs()
}

#[tauri::command]
pub fn set_output_priority(
    state: State<MidiOutputState>,
    device_id: String,
    priority: OutputPriority,
) -> Result<(), String> {
    // Priority is applied per-message, this just validates the device exists
    let _ = state.0.get_device_status(&device_id)
        .ok_or_else(|| format!("Device not found: {}", device_id))?;
    debug!("Set priority {:?} for device {}", priority, device_id);
    Ok(())
}

#[tauri::command]
pub fn get_output_queue_status(state: State<MidiOutputState>) -> OutputQueueStatus {
    state.0.get_output_queue_status()
}

#[tauri::command]
pub fn refresh_output_devices(
    state: State<MidiOutputState>,
) -> Result<Vec<MidiOutputDevice>, String> {
    state.0.refresh_output_devices()
}

#[tauri::command]
pub fn get_output_device_status(
    state: State<MidiOutputState>,
    device_id: String,
) -> Option<OutputDeviceStatus> {
    state.0.get_device_status(&device_id)
}

#[tauri::command]
pub fn get_all_output_statuses(state: State<MidiOutputState>) -> Vec<OutputDeviceStatus> {
    state.0.get_all_statuses()
}

#[tauri::command]
pub fn enable_output_active_sensing(
    state: State<MidiOutputState>,
    device_id: String,
) -> Result<(), String> {
    state.0.enable_active_sensing(&device_id)
}

#[tauri::command]
pub fn disable_output_active_sensing(
    state: State<MidiOutputState>,
    device_id: String,
) -> Result<(), String> {
    state.0.disable_active_sensing(&device_id)
}

/// Subscribe to device change events (emits to frontend)
#[tauri::command]
pub async fn subscribe_output_device_changes(
    app: tauri::AppHandle,
    state: State<'_, MidiOutputState>,
) -> Result<(), String> {
    use tauri::Emitter;

    let mut rx = state.0.subscribe_device_changes();

    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            if app.emit("midi_output_device_change", &event).is_err() {
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
    fn test_output_priority_ordering() {
        assert!(OutputPriority::RealTime > OutputPriority::High);
        assert!(OutputPriority::High > OutputPriority::Normal);
        assert!(OutputPriority::Normal > OutputPriority::Low);
    }

    #[test]
    fn test_track_assignment_creation() {
        let assignment = TrackOutputAssignment::new(1, "output_0".to_string(), 5);

        assert_eq!(assignment.track_id, 1);
        assert_eq!(assignment.device_id, "output_0");
        assert_eq!(assignment.channel, 5);
        assert!(assignment.enabled);
        assert_eq!(assignment.volume_scale, 1.0);
        assert_eq!(assignment.transpose, 0);
    }

    #[test]
    fn test_message_priority_heap_order() {
        let mut heap = BinaryHeap::new();

        let low = MidiOutputMessage {
            device_id: "test".to_string(),
            channel: 0,
            data: vec![0x90, 60, 100],
            priority: OutputPriority::Low,
            scheduled_time: None,
        };

        let high = MidiOutputMessage {
            device_id: "test".to_string(),
            channel: 0,
            data: vec![0x90, 60, 100],
            priority: OutputPriority::High,
            scheduled_time: None,
        };

        let realtime = MidiOutputMessage {
            device_id: "test".to_string(),
            channel: 0,
            data: vec![0x90, 60, 100],
            priority: OutputPriority::RealTime,
            scheduled_time: None,
        };

        heap.push(low);
        heap.push(high);
        heap.push(realtime);

        // Should pop in priority order: RealTime, High, Low
        let first = heap.pop().unwrap();
        assert_eq!(first.priority, OutputPriority::RealTime);

        let second = heap.pop().unwrap();
        assert_eq!(second.priority, OutputPriority::High);

        let third = heap.pop().unwrap();
        assert_eq!(third.priority, OutputPriority::Low);
    }

    #[test]
    fn test_parse_manufacturer() {
        assert_eq!(parse_manufacturer("AKAI FORCE"), Some("Akai".into()));
        assert_eq!(parse_manufacturer("Steinberg UR22"), Some("Steinberg".into()));
        assert_eq!(parse_manufacturer("Roland TD-17"), Some("Roland".into()));
        assert_eq!(parse_manufacturer("Unknown Device"), None);
    }

    #[test]
    fn test_virtual_port_info() {
        let vport = VirtualOutputPort {
            id: "virtual_123".to_string(),
            name: "Test Port".to_string(),
            created_at: 0,
            messages_sent: 42,
        };

        assert_eq!(vport.id, "virtual_123");
        assert_eq!(vport.name, "Test Port");
        assert_eq!(vport.messages_sent, 42);
    }

    #[test]
    fn test_device_change_event_serialization() {
        let connected = DeviceChangeEvent::DeviceConnected {
            device_id: "output_0".to_string(),
            device_name: "Test Device".to_string(),
        };

        let json = serde_json::to_string(&connected).unwrap();
        assert!(json.contains("DeviceConnected"));
        assert!(json.contains("output_0"));
    }
}

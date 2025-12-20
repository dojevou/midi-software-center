//! ZeroMQ Broker for Hardware Sync
//!
//! High-performance message broker for MIDI clock synchronization with external hardware.
//! Provides pub/sub messaging for MIDI clock events with sub-millisecond latency.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────┐     PUB/SUB      ┌──────────────┐
//! │  MIDI Clock     │────────────────▶│  MPC ONE     │
//! │  (Internal)     │                  └──────────────┘
//! │                 │────────────────▶│  UR22        │
//! │                 │                  └──────────────┘
//! │                 │────────────────▶│  Svelte UI   │
//! └─────────────────┘                  └──────────────┘
//!
//! ┌─────────────────┐     SUB          ┌──────────────┐
//! │  External       │◀────────────────│  MPC ONE     │
//! │  Clock Receiver │                  │  (Master)    │
//! └─────────────────┘                  └──────────────┘
//! ```
//!
//! ## Endpoints
//!
//! - `tcp://127.0.0.1:5555` - Clock publisher (internal master)
//! - `tcp://127.0.0.1:5556` - Clock subscriber (external master)
//! - `tcp://127.0.0.1:5557` - Hardware discovery (REQ/REP)
//! - `tcp://127.0.0.1:5558` - Bidirectional sync control

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, error, info, warn};
use zeromq::{PubSocket, RepSocket, Socket, SocketRecv, SocketSend, SubSocket};

use crate::midi_clock::{ClockTick, MidiClockMessage, SyncMode, PPQN};

// =============================================================================
// Constants
// =============================================================================

/// Default port for clock publisher
pub const CLOCK_PUB_PORT: u16 = 5555;
/// Default port for clock subscriber (external master)
pub const CLOCK_SUB_PORT: u16 = 5556;
/// Default port for hardware discovery
pub const DISCOVERY_PORT: u16 = 5557;
/// Default port for sync control
pub const SYNC_CONTROL_PORT: u16 = 5558;

/// Topic prefixes for ZeroMQ pub/sub
pub mod topics {
    pub const CLOCK_TICK: &str = "CLK";
    pub const TRANSPORT: &str = "TRN";
    pub const POSITION: &str = "POS";
    pub const BPM: &str = "BPM";
    pub const SYNC_STATUS: &str = "SYN";
    pub const DEVICE_EVENT: &str = "DEV";
}

// =============================================================================
// Message Types
// =============================================================================

/// Wire-format for clock tick messages (optimized for size)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClockTickMessage {
    /// Current tick count
    pub tick: u64,
    /// Current beat (0-indexed)
    pub beat: u32,
    /// Current bar (0-indexed)
    pub bar: u32,
    /// Position in milliseconds
    pub position_ms: u64,
    /// Timestamp (microseconds since broker start)
    pub timestamp_us: u64,
}

impl From<&ClockTick> for ClockTickMessage {
    fn from(tick: &ClockTick) -> Self {
        Self {
            tick: tick.tick,
            beat: tick.beat,
            bar: tick.bar,
            position_ms: tick.position_millis,
            timestamp_us: tick.timestamp.elapsed().as_micros() as u64,
        }
    }
}

/// Transport state messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportMessage {
    Start,
    Stop,
    Continue,
    Pause,
    /// Song Position Pointer (in MIDI beats = 16th notes)
    SetPosition(u16),
}

impl From<MidiClockMessage> for TransportMessage {
    fn from(msg: MidiClockMessage) -> Self {
        match msg {
            MidiClockMessage::Start => Self::Start,
            MidiClockMessage::Stop => Self::Stop,
            MidiClockMessage::Continue => Self::Continue,
            _ => Self::Stop, // Default fallback
        }
    }
}

/// BPM change message
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BpmMessage {
    pub bpm: f64,
    pub source: BpmSource,
}

/// Source of BPM value
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BpmSource {
    Internal,
    External,
    Manual,
}

/// Sync status message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatusMessage {
    pub mode: SyncMode,
    pub is_locked: bool,
    pub master_device: Option<String>,
    pub drift_us: i64,
    pub external_bpm: Option<f64>,
}

/// Hardware device event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceEvent {
    Connected(HardwareDevice),
    Disconnected(String),
    ClockReceived { device_id: String, message: MidiClockMessage },
    Error { device_id: String, error: String },
}

/// Hardware device info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareDevice {
    pub id: String,
    pub name: String,
    pub device_type: HardwareDeviceType,
    pub can_send_clock: bool,
    pub can_receive_clock: bool,
    pub latency_us: u64,
}

/// Type of hardware device
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareDeviceType {
    MidiController,
    AudioInterface,
    DrumMachine,
    Sampler,
    Synthesizer,
    Sequencer,
    Unknown,
}

impl HardwareDeviceType {
    pub fn from_name(name: &str) -> Self {
        let name_lower = name.to_lowercase();
        if name_lower.contains("mpc") || name_lower.contains("maschine") {
            Self::DrumMachine
        } else if name_lower.contains("ur22") || name_lower.contains("interface") {
            Self::AudioInterface
        } else if name_lower.contains("synth") || name_lower.contains("prophet") {
            Self::Synthesizer
        } else if name_lower.contains("sampler") {
            Self::Sampler
        } else if name_lower.contains("controller") || name_lower.contains("pad") {
            Self::MidiController
        } else {
            Self::Unknown
        }
    }
}

// =============================================================================
// Discovery Protocol
// =============================================================================

/// Discovery request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryRequest {
    /// List all connected devices
    ListDevices,
    /// Get device info by ID
    GetDevice(String),
    /// Subscribe to device events
    Subscribe(String),
    /// Unsubscribe from device events
    Unsubscribe(String),
    /// Ping for health check
    Ping,
}

/// Discovery response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryResponse {
    /// List of devices
    Devices(Vec<HardwareDevice>),
    /// Single device info
    Device(Option<HardwareDevice>),
    /// Subscription confirmation
    Subscribed(String),
    /// Unsubscription confirmation
    Unsubscribed(String),
    /// Pong response
    Pong { uptime_ms: u64 },
    /// Error response
    Error(String),
}

// =============================================================================
// Sync Control Protocol
// =============================================================================

/// Sync control request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncControlRequest {
    /// Set sync mode
    SetMode(SyncMode),
    /// Set master device (for external sync)
    SetMaster(String),
    /// Get current sync status
    GetStatus,
    /// Force resync
    Resync,
    /// Set BPM (only valid in Internal mode)
    SetBpm(f64),
    /// Nudge tempo (temporary adjustment)
    NudgeTempo(f64),
}

/// Sync control response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncControlResponse {
    /// Mode changed
    ModeChanged(SyncMode),
    /// Master device set
    MasterSet(String),
    /// Current status
    Status(SyncStatusMessage),
    /// Resync initiated
    ResyncStarted,
    /// BPM set
    BpmSet(f64),
    /// Tempo nudged
    TempoNudged(f64),
    /// Error response
    Error(String),
}

// =============================================================================
// ZeroMQ Broker
// =============================================================================

/// Configuration for the ZeroMQ broker
#[derive(Debug, Clone)]
pub struct ZmqBrokerConfig {
    pub clock_pub_port: u16,
    pub clock_sub_port: u16,
    pub discovery_port: u16,
    pub sync_control_port: u16,
    pub bind_address: String,
}

impl Default for ZmqBrokerConfig {
    fn default() -> Self {
        Self {
            clock_pub_port: CLOCK_PUB_PORT,
            clock_sub_port: CLOCK_SUB_PORT,
            discovery_port: DISCOVERY_PORT,
            sync_control_port: SYNC_CONTROL_PORT,
            bind_address: "127.0.0.1".to_string(),
        }
    }
}

/// ZeroMQ broker state
pub struct ZmqBrokerState {
    /// Current sync mode
    pub mode: SyncMode,
    /// Current master device (for external sync)
    pub master_device: Option<String>,
    /// Connected devices
    pub devices: HashMap<String, HardwareDevice>,
    /// Device subscriptions
    pub subscriptions: Vec<String>,
    /// Is clock locked to external
    pub is_locked: bool,
    /// External BPM (detected from master)
    pub external_bpm: Option<f64>,
    /// Drift from external clock (microseconds)
    pub drift_us: i64,
    /// Broker start time
    pub start_time: Instant,
}

impl Default for ZmqBrokerState {
    fn default() -> Self {
        Self {
            mode: SyncMode::Internal,
            master_device: None,
            devices: HashMap::new(),
            subscriptions: Vec::new(),
            is_locked: false,
            external_bpm: None,
            drift_us: 0,
            start_time: Instant::now(),
        }
    }
}

/// Main ZeroMQ broker for hardware sync
pub struct ZmqBroker {
    /// Broker configuration
    pub config: ZmqBrokerConfig,
    /// Shared broker state
    pub state: Arc<RwLock<ZmqBrokerState>>,
    running: Arc<AtomicBool>,

    // Internal channels
    clock_tx: broadcast::Sender<ClockTickMessage>,
    transport_tx: broadcast::Sender<TransportMessage>,
    device_event_tx: mpsc::Sender<DeviceEvent>,

    // External clock input (for bidirectional sync)
    external_clock_rx: Option<mpsc::Receiver<MidiClockMessage>>,

    // BPM detection for external clock
    tick_count: Arc<AtomicU64>,
    last_tick_times: Arc<RwLock<Vec<Instant>>>,
}

impl ZmqBroker {
    /// Create a new ZeroMQ broker
    pub fn new(config: ZmqBrokerConfig) -> Self {
        let (clock_tx, _) = broadcast::channel(256);
        let (transport_tx, _) = broadcast::channel(64);
        let (device_event_tx, _) = mpsc::channel(64);

        Self {
            config,
            state: Arc::new(RwLock::new(ZmqBrokerState::default())),
            running: Arc::new(AtomicBool::new(false)),
            clock_tx,
            transport_tx,
            device_event_tx,
            external_clock_rx: None,
            tick_count: Arc::new(AtomicU64::new(0)),
            last_tick_times: Arc::new(RwLock::new(Vec::with_capacity(48))),
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(ZmqBrokerConfig::default())
    }

    /// Set external clock receiver (for bidirectional sync)
    pub fn with_external_clock(mut self, rx: mpsc::Receiver<MidiClockMessage>) -> Self {
        self.external_clock_rx = Some(rx);
        self
    }

    /// Subscribe to clock tick events
    pub fn subscribe_clock(&self) -> broadcast::Receiver<ClockTickMessage> {
        self.clock_tx.subscribe()
    }

    /// Subscribe to transport events
    pub fn subscribe_transport(&self) -> broadcast::Receiver<TransportMessage> {
        self.transport_tx.subscribe()
    }

    /// Get current sync status
    pub async fn get_status(&self) -> SyncStatusMessage {
        let state = self.state.read().await;
        SyncStatusMessage {
            mode: state.mode,
            is_locked: state.is_locked,
            master_device: state.master_device.clone(),
            drift_us: state.drift_us,
            external_bpm: state.external_bpm,
        }
    }

    /// Set sync mode
    pub async fn set_mode(&self, mode: SyncMode) {
        let mut state = self.state.write().await;
        state.mode = mode;
        state.is_locked = false;
        state.external_bpm = None;
        info!(?mode, "Sync mode changed");
    }

    /// Set master device for external sync
    pub async fn set_master(&self, device_id: String) {
        let mut state = self.state.write().await;
        state.master_device = Some(device_id.clone());
        info!(device_id, "Master device set");
    }

    /// Register a hardware device
    pub async fn register_device(&self, device: HardwareDevice) {
        let mut state = self.state.write().await;
        info!(device_id = device.id, name = device.name, "Device registered");
        state.devices.insert(device.id.clone(), device.clone());

        // Send device event
        let _ = self.device_event_tx.send(DeviceEvent::Connected(device)).await;
    }

    /// Unregister a hardware device
    pub async fn unregister_device(&self, device_id: &str) {
        let mut state = self.state.write().await;
        if state.devices.remove(device_id).is_some() {
            info!(device_id, "Device unregistered");

            // Clear master if it was this device
            if state.master_device.as_deref() == Some(device_id) {
                state.master_device = None;
                state.is_locked = false;
            }

            // Send device event
            let _ = self
                .device_event_tx
                .send(DeviceEvent::Disconnected(device_id.to_string()))
                .await;
        }
    }

    /// Publish a clock tick
    pub fn publish_clock_tick(&self, tick: &ClockTick) {
        let msg = ClockTickMessage::from(tick);
        let _ = self.clock_tx.send(msg);
    }

    /// Publish a transport message
    pub fn publish_transport(&self, msg: TransportMessage) {
        let _ = self.transport_tx.send(msg);
    }

    /// Start the broker
    pub async fn start(&self) -> anyhow::Result<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Ok(()); // Already running
        }

        info!("Starting ZeroMQ broker...");

        // Start clock publisher
        self.start_clock_publisher().await?;

        // Start discovery service
        self.start_discovery_service().await?;

        // Start sync control service
        self.start_sync_control_service().await?;

        // Start external clock subscriber (if in external mode)
        self.start_external_clock_subscriber().await?;

        info!(
            clock_pub = format!("tcp://{}:{}", self.config.bind_address, self.config.clock_pub_port),
            discovery = format!("tcp://{}:{}", self.config.bind_address, self.config.discovery_port),
            sync_control = format!("tcp://{}:{}", self.config.bind_address, self.config.sync_control_port),
            "ZeroMQ broker started"
        );

        Ok(())
    }

    /// Stop the broker
    pub async fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        info!("ZeroMQ broker stopped");
    }

    /// Check if broker is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    // -------------------------------------------------------------------------
    // Private methods
    // -------------------------------------------------------------------------

    async fn start_clock_publisher(&self) -> anyhow::Result<()> {
        let address = format!("tcp://{}:{}", self.config.bind_address, self.config.clock_pub_port);
        let running = self.running.clone();
        let mut clock_rx = self.clock_tx.subscribe();
        let mut transport_rx = self.transport_tx.subscribe();

        tokio::spawn(async move {
            let mut socket = PubSocket::new();

            if let Err(e) = socket.bind(&address).await {
                error!(error = %e, "Failed to bind clock publisher");
                return;
            }

            info!(address, "Clock publisher bound");

            while running.load(Ordering::SeqCst) {
                tokio::select! {
                    Ok(tick) = clock_rx.recv() => {
                        let mut msg = Vec::with_capacity(32);
                        msg.extend_from_slice(topics::CLOCK_TICK.as_bytes());
                        if let Ok(data) = bincode::serialize(&tick) {
                            msg.extend_from_slice(&data);
                            if let Err(e) = socket.send(msg.into()).await {
                                debug!(error = %e, "Failed to send clock tick");
                            }
                        }
                    }
                    Ok(transport) = transport_rx.recv() => {
                        let mut msg = Vec::with_capacity(16);
                        msg.extend_from_slice(topics::TRANSPORT.as_bytes());
                        if let Ok(data) = bincode::serialize(&transport) {
                            msg.extend_from_slice(&data);
                            if let Err(e) = socket.send(msg.into()).await {
                                debug!(error = %e, "Failed to send transport");
                            }
                        }
                    }
                    _ = tokio::time::sleep(Duration::from_millis(100)) => {
                        // Keepalive / check running
                    }
                }
            }
        });

        Ok(())
    }

    async fn start_discovery_service(&self) -> anyhow::Result<()> {
        let address = format!("tcp://{}:{}", self.config.bind_address, self.config.discovery_port);
        let running = self.running.clone();
        let state = self.state.clone();

        tokio::spawn(async move {
            let mut socket = RepSocket::new();

            if let Err(e) = socket.bind(&address).await {
                error!(error = %e, "Failed to bind discovery service");
                return;
            }

            info!(address, "Discovery service bound");

            while running.load(Ordering::SeqCst) {
                match tokio::time::timeout(Duration::from_millis(100), socket.recv()).await {
                    Ok(Ok(msg)) => {
                        // Get first frame of the message
                        let bytes = msg.get(0).map(|b| b.to_vec()).unwrap_or_default();
                        let request: Result<DiscoveryRequest, _> = bincode::deserialize(&bytes);

                        let response = match request {
                            Ok(DiscoveryRequest::ListDevices) => {
                                let s = state.read().await;
                                let devices: Vec<HardwareDevice> = s.devices.values().cloned().collect();
                                DiscoveryResponse::Devices(devices)
                            }
                            Ok(DiscoveryRequest::GetDevice(id)) => {
                                let s = state.read().await;
                                DiscoveryResponse::Device(s.devices.get(&id).cloned())
                            }
                            Ok(DiscoveryRequest::Subscribe(id)) => {
                                let mut s = state.write().await;
                                if !s.subscriptions.contains(&id) {
                                    s.subscriptions.push(id.clone());
                                }
                                DiscoveryResponse::Subscribed(id)
                            }
                            Ok(DiscoveryRequest::Unsubscribe(id)) => {
                                let mut s = state.write().await;
                                s.subscriptions.retain(|x| x != &id);
                                DiscoveryResponse::Unsubscribed(id)
                            }
                            Ok(DiscoveryRequest::Ping) => {
                                let s = state.read().await;
                                DiscoveryResponse::Pong {
                                    uptime_ms: s.start_time.elapsed().as_millis() as u64,
                                }
                            }
                            Err(e) => DiscoveryResponse::Error(e.to_string()),
                        };

                        if let Ok(data) = bincode::serialize(&response) {
                            let _ = socket.send(data.into()).await;
                        }
                    }
                    Ok(Err(e)) => {
                        debug!(error = %e, "Discovery recv error");
                    }
                    Err(_) => {
                        // Timeout - continue
                    }
                }
            }
        });

        Ok(())
    }

    async fn start_sync_control_service(&self) -> anyhow::Result<()> {
        let address =
            format!("tcp://{}:{}", self.config.bind_address, self.config.sync_control_port);
        let running = self.running.clone();
        let state = self.state.clone();

        tokio::spawn(async move {
            let mut socket = RepSocket::new();

            if let Err(e) = socket.bind(&address).await {
                error!(error = %e, "Failed to bind sync control service");
                return;
            }

            info!(address, "Sync control service bound");

            while running.load(Ordering::SeqCst) {
                match tokio::time::timeout(Duration::from_millis(100), socket.recv()).await {
                    Ok(Ok(msg)) => {
                        // Get first frame of the message
                        let bytes = msg.get(0).map(|b| b.to_vec()).unwrap_or_default();
                        let request: Result<SyncControlRequest, _> = bincode::deserialize(&bytes);

                        let response = match request {
                            Ok(SyncControlRequest::SetMode(mode)) => {
                                let mut s = state.write().await;
                                s.mode = mode;
                                s.is_locked = false;
                                SyncControlResponse::ModeChanged(mode)
                            }
                            Ok(SyncControlRequest::SetMaster(device_id)) => {
                                let mut s = state.write().await;
                                s.master_device = Some(device_id.clone());
                                SyncControlResponse::MasterSet(device_id)
                            }
                            Ok(SyncControlRequest::GetStatus) => {
                                let s = state.read().await;
                                SyncControlResponse::Status(SyncStatusMessage {
                                    mode: s.mode,
                                    is_locked: s.is_locked,
                                    master_device: s.master_device.clone(),
                                    drift_us: s.drift_us,
                                    external_bpm: s.external_bpm,
                                })
                            }
                            Ok(SyncControlRequest::Resync) => {
                                let mut s = state.write().await;
                                s.is_locked = false;
                                s.drift_us = 0;
                                SyncControlResponse::ResyncStarted
                            }
                            Ok(SyncControlRequest::SetBpm(bpm)) => {
                                // Note: Actual BPM change should go through MidiClock
                                SyncControlResponse::BpmSet(bpm)
                            }
                            Ok(SyncControlRequest::NudgeTempo(delta)) => {
                                SyncControlResponse::TempoNudged(delta)
                            }
                            Err(e) => SyncControlResponse::Error(e.to_string()),
                        };

                        if let Ok(data) = bincode::serialize(&response) {
                            let _ = socket.send(data.into()).await;
                        }
                    }
                    Ok(Err(e)) => {
                        debug!(error = %e, "Sync control recv error");
                    }
                    Err(_) => {
                        // Timeout - continue
                    }
                }
            }
        });

        Ok(())
    }

    async fn start_external_clock_subscriber(&self) -> anyhow::Result<()> {
        let address = format!("tcp://{}:{}", self.config.bind_address, self.config.clock_sub_port);
        let running = self.running.clone();
        let state = self.state.clone();
        let tick_count = self.tick_count.clone();
        let last_tick_times = self.last_tick_times.clone();

        tokio::spawn(async move {
            let mut socket = SubSocket::new();

            // Subscribe to clock tick topic
            if let Err(e) = socket.subscribe(topics::CLOCK_TICK).await {
                warn!(error = %e, "Failed to subscribe to clock topic");
            }

            if let Err(e) = socket.connect(&address).await {
                // This is expected to fail if no external master is publishing
                debug!(error = %e, "No external clock source available");
                return;
            }

            info!(address, "External clock subscriber connected");

            while running.load(Ordering::SeqCst) {
                // Only process if in External mode
                let current_mode = state.read().await.mode;
                if current_mode != SyncMode::External {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }

                match tokio::time::timeout(Duration::from_millis(100), socket.recv()).await {
                    Ok(Ok(msg)) => {
                        // Parse message (skip topic prefix)
                        if msg.len() > 3 {
                            let now = Instant::now();
                            tick_count.fetch_add(1, Ordering::SeqCst);

                            // Update tick times for BPM detection
                            let mut times = last_tick_times.write().await;
                            times.push(now);
                            if times.len() > 48 {
                                times.remove(0);
                            }

                            // Calculate external BPM
                            if times.len() >= 2 {
                                let total_duration: Duration =
                                    times.windows(2).map(|w| w[1].duration_since(w[0])).sum();

                                let avg_tick_interval =
                                    total_duration.as_secs_f64() / (times.len() - 1) as f64;

                                let bpm = 60.0 / (avg_tick_interval * PPQN as f64);

                                if (20.0..=300.0).contains(&bpm) {
                                    let mut s = state.write().await;
                                    s.external_bpm = Some(bpm);
                                    s.is_locked = true;
                                }
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        debug!(error = %e, "External clock recv error");
                    }
                    Err(_) => {
                        // Timeout - continue
                    }
                }
            }
        });

        Ok(())
    }
}

// =============================================================================
// Hardware Sync Bridge
// =============================================================================

/// Bridge between ZeroMQ broker and MIDI clock
///
/// This connects the ZeroMQ pub/sub layer to the existing MidiClock and SyncManager.
pub struct HardwareSyncBridge {
    broker: Arc<ZmqBroker>,
    running: Arc<AtomicBool>,
}

impl HardwareSyncBridge {
    pub fn new(broker: Arc<ZmqBroker>) -> Self {
        Self { broker, running: Arc::new(AtomicBool::new(false)) }
    }

    /// Start bridging clock ticks from MidiClock to ZeroMQ
    pub async fn start_clock_bridge(
        &self,
        mut clock_rx: broadcast::Receiver<ClockTick>,
    ) -> anyhow::Result<()> {
        let broker = self.broker.clone();
        let running = self.running.clone();

        running.store(true, Ordering::SeqCst);

        tokio::spawn(async move {
            while running.load(Ordering::SeqCst) {
                match clock_rx.recv().await {
                    Ok(tick) => {
                        broker.publish_clock_tick(&tick);
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!(skipped = n, "Clock bridge lagged");
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Clock channel closed");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Start bridging transport messages
    pub async fn start_transport_bridge(
        &self,
        mut message_rx: mpsc::Receiver<MidiClockMessage>,
    ) -> anyhow::Result<()> {
        let broker = self.broker.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            while running.load(Ordering::SeqCst) {
                match message_rx.recv().await {
                    Some(msg) => {
                        let transport = match msg {
                            MidiClockMessage::Start => TransportMessage::Start,
                            MidiClockMessage::Stop => TransportMessage::Stop,
                            MidiClockMessage::Continue => TransportMessage::Continue,
                            _ => continue,
                        };
                        broker.publish_transport(transport);
                    }
                    None => {
                        info!("Transport channel closed");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop all bridges
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

// =============================================================================
// ZeroMQ Client (for external processes)
// =============================================================================

/// Client for connecting to the ZeroMQ broker from external processes
pub struct ZmqClient {
    config: ZmqBrokerConfig,
}

impl ZmqClient {
    pub fn new(config: ZmqBrokerConfig) -> Self {
        Self { config }
    }

    /// Subscribe to clock ticks
    pub async fn subscribe_clock(&self) -> anyhow::Result<SubSocket> {
        let address = format!("tcp://{}:{}", self.config.bind_address, self.config.clock_pub_port);

        let mut socket = SubSocket::new();
        socket.subscribe(topics::CLOCK_TICK).await?;
        socket.connect(&address).await?;

        Ok(socket)
    }

    /// Send discovery request
    pub async fn discover(&self, request: DiscoveryRequest) -> anyhow::Result<DiscoveryResponse> {
        let address = format!("tcp://{}:{}", self.config.bind_address, self.config.discovery_port);

        let mut socket = zeromq::ReqSocket::new();
        socket.connect(&address).await?;

        let data = bincode::serialize(&request)?;
        socket.send(data.into()).await?;

        let response = socket.recv().await?;
        let bytes = response.get(0).map(|b| b.to_vec()).unwrap_or_default();
        let result: DiscoveryResponse = bincode::deserialize(&bytes)?;

        Ok(result)
    }

    /// Send sync control request
    pub async fn sync_control(
        &self,
        request: SyncControlRequest,
    ) -> anyhow::Result<SyncControlResponse> {
        let address =
            format!("tcp://{}:{}", self.config.bind_address, self.config.sync_control_port);

        let mut socket = zeromq::ReqSocket::new();
        socket.connect(&address).await?;

        let data = bincode::serialize(&request)?;
        socket.send(data.into()).await?;

        let response = socket.recv().await?;
        let bytes = response.get(0).map(|b| b.to_vec()).unwrap_or_default();
        let result: SyncControlResponse = bincode::deserialize(&bytes)?;

        Ok(result)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_tick_message_from() {
        let tick = ClockTick {
            tick: 24,
            beat: 1,
            bar: 0,
            timestamp: Instant::now(),
            position_millis: 500,
        };

        let msg = ClockTickMessage::from(&tick);

        assert_eq!(msg.tick, 24);
        assert_eq!(msg.beat, 1);
        assert_eq!(msg.bar, 0);
        assert_eq!(msg.position_ms, 500);
    }

    #[test]
    fn test_transport_message_from_midi_clock() {
        assert_eq!(TransportMessage::from(MidiClockMessage::Start), TransportMessage::Start);
        assert_eq!(TransportMessage::from(MidiClockMessage::Stop), TransportMessage::Stop);
        assert_eq!(
            TransportMessage::from(MidiClockMessage::Continue),
            TransportMessage::Continue
        );
    }

    #[test]
    fn test_hardware_device_type_from_name() {
        assert_eq!(HardwareDeviceType::from_name("MPC ONE"), HardwareDeviceType::DrumMachine);
        assert_eq!(
            HardwareDeviceType::from_name("UR22mkII"),
            HardwareDeviceType::AudioInterface
        );
        assert_eq!(
            HardwareDeviceType::from_name("Maschine MK3"),
            HardwareDeviceType::DrumMachine
        );
        assert_eq!(
            HardwareDeviceType::from_name("Unknown Device"),
            HardwareDeviceType::Unknown
        );
    }

    #[test]
    fn test_zmq_broker_config_default() {
        let config = ZmqBrokerConfig::default();

        assert_eq!(config.clock_pub_port, 5555);
        assert_eq!(config.clock_sub_port, 5556);
        assert_eq!(config.discovery_port, 5557);
        assert_eq!(config.sync_control_port, 5558);
        assert_eq!(config.bind_address, "127.0.0.1");
    }

    #[test]
    fn test_zmq_broker_state_default() {
        let state = ZmqBrokerState::default();

        assert_eq!(state.mode, SyncMode::Internal);
        assert!(state.master_device.is_none());
        assert!(state.devices.is_empty());
        assert!(!state.is_locked);
    }

    #[test]
    fn test_hardware_device_serialization() {
        let device = HardwareDevice {
            id: "mpc-one".to_string(),
            name: "MPC ONE".to_string(),
            device_type: HardwareDeviceType::DrumMachine,
            can_send_clock: true,
            can_receive_clock: true,
            latency_us: 5000,
        };

        let json = serde_json::to_string(&device).unwrap();
        let deserialized: HardwareDevice = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, device.id);
        assert_eq!(deserialized.name, device.name);
        assert_eq!(deserialized.device_type, device.device_type);
    }

    #[test]
    fn test_discovery_request_serialization() {
        let requests = [
            DiscoveryRequest::ListDevices,
            DiscoveryRequest::GetDevice("test".to_string()),
            DiscoveryRequest::Ping,
        ];

        for request in requests {
            let data = bincode::serialize(&request).unwrap();
            let _decoded: DiscoveryRequest = bincode::deserialize(&data).unwrap();
        }
    }

    #[test]
    fn test_sync_control_request_serialization() {
        let requests = [
            SyncControlRequest::SetMode(SyncMode::External),
            SyncControlRequest::SetMaster("mpc-one".to_string()),
            SyncControlRequest::GetStatus,
            SyncControlRequest::SetBpm(128.0),
        ];

        for request in requests {
            let data = bincode::serialize(&request).unwrap();
            let _decoded: SyncControlRequest = bincode::deserialize(&data).unwrap();
        }
    }

    #[tokio::test]
    async fn test_zmq_broker_creation() {
        let broker = ZmqBroker::with_defaults();

        assert!(!broker.is_running());
    }

    #[tokio::test]
    async fn test_zmq_broker_set_mode() {
        let broker = ZmqBroker::with_defaults();

        broker.set_mode(SyncMode::External).await;

        let status = broker.get_status().await;
        assert_eq!(status.mode, SyncMode::External);
        assert!(!status.is_locked);
    }

    #[tokio::test]
    async fn test_zmq_broker_register_device() {
        let broker = ZmqBroker::with_defaults();

        let device = HardwareDevice {
            id: "test-device".to_string(),
            name: "Test Device".to_string(),
            device_type: HardwareDeviceType::MidiController,
            can_send_clock: false,
            can_receive_clock: true,
            latency_us: 10000,
        };

        broker.register_device(device).await;

        let state = broker.state.read().await;
        assert!(state.devices.contains_key("test-device"));
    }

    #[tokio::test]
    async fn test_zmq_broker_unregister_device() {
        let broker = ZmqBroker::with_defaults();

        let device = HardwareDevice {
            id: "test-device".to_string(),
            name: "Test Device".to_string(),
            device_type: HardwareDeviceType::MidiController,
            can_send_clock: false,
            can_receive_clock: true,
            latency_us: 10000,
        };

        broker.register_device(device).await;
        broker.unregister_device("test-device").await;

        let state = broker.state.read().await;
        assert!(!state.devices.contains_key("test-device"));
    }

    #[tokio::test]
    async fn test_zmq_broker_set_master() {
        let broker = ZmqBroker::with_defaults();

        broker.set_master("mpc-one".to_string()).await;

        let state = broker.state.read().await;
        assert_eq!(state.master_device.as_deref(), Some("mpc-one"));
    }

    #[tokio::test]
    async fn test_zmq_broker_publish_clock() {
        let broker = ZmqBroker::with_defaults();
        let mut rx = broker.subscribe_clock();

        let tick = ClockTick {
            tick: 0,
            beat: 0,
            bar: 0,
            timestamp: Instant::now(),
            position_millis: 0,
        };

        broker.publish_clock_tick(&tick);

        // Should receive the tick
        let result = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;

        assert!(result.is_ok());
        let received = result.unwrap().unwrap();
        assert_eq!(received.tick, 0);
    }

    #[tokio::test]
    async fn test_zmq_broker_publish_transport() {
        let broker = ZmqBroker::with_defaults();
        let mut rx = broker.subscribe_transport();

        broker.publish_transport(TransportMessage::Start);

        let result = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().unwrap(), TransportMessage::Start);
    }

    #[tokio::test]
    async fn test_hardware_sync_bridge_creation() {
        let broker = Arc::new(ZmqBroker::with_defaults());
        let bridge = HardwareSyncBridge::new(broker);

        assert!(!bridge.running.load(Ordering::SeqCst));
    }
}

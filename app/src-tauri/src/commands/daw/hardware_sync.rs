//! Tauri commands for hardware sync via ZeroMQ
//!
//! These commands provide the frontend with access to the ZeroMQ broker
//! for hardware synchronization, device discovery, and bidirectional clock sync.

use crate::midi_clock::SyncMode;
use crate::services::zeromq_broker::{
    HardwareDevice, HardwareDeviceType, SyncStatusMessage, ZmqBroker,
};
use std::sync::Arc;
use tauri::{command, State};
use tokio::sync::RwLock;

/// State for hardware sync via ZeroMQ
pub struct HardwareSyncState {
    pub broker: Arc<ZmqBroker>,
}

impl HardwareSyncState {
    pub fn new(broker: Arc<ZmqBroker>) -> Self {
        Self { broker }
    }
}

// =============================================================================
// Broker Lifecycle Commands
// =============================================================================

/// Start the ZeroMQ broker
#[command]
pub async fn hardware_sync_start(state: State<'_, HardwareSyncState>) -> Result<(), String> {
    state.broker.start().await.map_err(|e| e.to_string())
}

/// Stop the ZeroMQ broker
#[command]
pub async fn hardware_sync_stop(state: State<'_, HardwareSyncState>) -> Result<(), String> {
    state.broker.stop().await;
    Ok(())
}

/// Check if broker is running
#[command]
pub fn hardware_sync_is_running(state: State<'_, HardwareSyncState>) -> bool {
    state.broker.is_running()
}

// =============================================================================
// Sync Mode Commands
// =============================================================================

/// Get current sync status
#[command]
pub async fn hardware_sync_get_status(
    state: State<'_, HardwareSyncState>,
) -> Result<SyncStatusMessage, String> {
    Ok(state.broker.get_status().await)
}

/// Set sync mode (internal/external/mtc)
#[command]
pub async fn hardware_sync_set_mode(
    state: State<'_, HardwareSyncState>,
    mode: String,
) -> Result<(), String> {
    let sync_mode = match mode.to_lowercase().as_str() {
        "internal" => SyncMode::Internal,
        "external" => SyncMode::External,
        "mtc" | "timecode" => SyncMode::MidiTimecode,
        _ => return Err(format!("Unknown sync mode: {}", mode)),
    };

    state.broker.set_mode(sync_mode).await;
    Ok(())
}

/// Set master device for external sync
#[command]
pub async fn hardware_sync_set_master(
    state: State<'_, HardwareSyncState>,
    device_id: String,
) -> Result<(), String> {
    state.broker.set_master(device_id).await;
    Ok(())
}

// =============================================================================
// Device Management Commands
// =============================================================================

/// List all registered hardware devices
#[command]
pub async fn hardware_sync_list_devices(
    state: State<'_, HardwareSyncState>,
) -> Result<Vec<HardwareDevice>, String> {
    let broker_state = state.broker.state.read().await;
    Ok(broker_state.devices.values().cloned().collect())
}

/// Get a specific device by ID
#[command]
pub async fn hardware_sync_get_device(
    state: State<'_, HardwareSyncState>,
    device_id: String,
) -> Result<Option<HardwareDevice>, String> {
    let broker_state = state.broker.state.read().await;
    Ok(broker_state.devices.get(&device_id).cloned())
}

/// Register a hardware device
#[command]
pub async fn hardware_sync_register_device(
    state: State<'_, HardwareSyncState>,
    id: String,
    name: String,
    can_send_clock: bool,
    can_receive_clock: bool,
    latency_us: u64,
) -> Result<(), String> {
    let device = HardwareDevice {
        id,
        name: name.clone(),
        device_type: HardwareDeviceType::from_name(&name),
        can_send_clock,
        can_receive_clock,
        latency_us,
    };

    state.broker.register_device(device).await;
    Ok(())
}

/// Unregister a hardware device
#[command]
pub async fn hardware_sync_unregister_device(
    state: State<'_, HardwareSyncState>,
    device_id: String,
) -> Result<(), String> {
    state.broker.unregister_device(&device_id).await;
    Ok(())
}

// =============================================================================
// Connection Info Commands
// =============================================================================

/// Get ZeroMQ endpoint information for external processes
#[command]
pub fn hardware_sync_get_endpoints(state: State<'_, HardwareSyncState>) -> HardwareSyncEndpoints {
    let config = &state.broker.config;
    HardwareSyncEndpoints {
        clock_publisher: format!("tcp://{}:{}", config.bind_address, config.clock_pub_port),
        clock_subscriber: format!("tcp://{}:{}", config.bind_address, config.clock_sub_port),
        discovery: format!("tcp://{}:{}", config.bind_address, config.discovery_port),
        sync_control: format!("tcp://{}:{}", config.bind_address, config.sync_control_port),
    }
}

/// Endpoint information for external processes
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HardwareSyncEndpoints {
    /// Subscribe to clock ticks from this endpoint
    pub clock_publisher: String,
    /// Publish clock ticks to this endpoint (when acting as subscriber)
    pub clock_subscriber: String,
    /// Send discovery requests to this endpoint
    pub discovery: String,
    /// Send sync control commands to this endpoint
    pub sync_control: String,
}

// =============================================================================
// Statistics Commands
// =============================================================================

/// Get broker statistics
#[command]
pub async fn hardware_sync_get_stats(
    state: State<'_, HardwareSyncState>,
) -> Result<HardwareSyncStats, String> {
    let broker_state = state.broker.state.read().await;
    Ok(HardwareSyncStats {
        is_running: state.broker.is_running(),
        connected_devices: broker_state.devices.len(),
        active_subscriptions: broker_state.subscriptions.len(),
        uptime_ms: broker_state.start_time.elapsed().as_millis() as u64,
        is_locked: broker_state.is_locked,
        external_bpm: broker_state.external_bpm,
        drift_us: broker_state.drift_us,
    })
}

/// Statistics about the hardware sync broker
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HardwareSyncStats {
    pub is_running: bool,
    pub connected_devices: usize,
    pub active_subscriptions: usize,
    pub uptime_ms: u64,
    pub is_locked: bool,
    pub external_bpm: Option<f64>,
    pub drift_us: i64,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::zeromq_broker::ZmqBrokerConfig;

    fn create_test_broker() -> Arc<ZmqBroker> {
        Arc::new(ZmqBroker::new(ZmqBrokerConfig::default()))
    }

    #[tokio::test]
    async fn test_hardware_sync_state_creation() {
        let broker = create_test_broker();
        let _state = HardwareSyncState::new(broker);
    }

    #[test]
    fn test_hardware_sync_endpoints_serialization() {
        let endpoints = HardwareSyncEndpoints {
            clock_publisher: "tcp://127.0.0.1:5555".to_string(),
            clock_subscriber: "tcp://127.0.0.1:5556".to_string(),
            discovery: "tcp://127.0.0.1:5557".to_string(),
            sync_control: "tcp://127.0.0.1:5558".to_string(),
        };

        let json = serde_json::to_string(&endpoints).unwrap();
        let deserialized: HardwareSyncEndpoints = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.clock_publisher, endpoints.clock_publisher);
    }

    #[test]
    fn test_hardware_sync_stats_serialization() {
        let stats = HardwareSyncStats {
            is_running: true,
            connected_devices: 2,
            active_subscriptions: 1,
            uptime_ms: 5000,
            is_locked: true,
            external_bpm: Some(120.0),
            drift_us: 50,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: HardwareSyncStats = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.is_running, stats.is_running);
        assert_eq!(deserialized.connected_devices, stats.connected_devices);
        assert_eq!(deserialized.external_bpm, stats.external_bpm);
    }
}

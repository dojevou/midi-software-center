// Hardware module for MIDI device management
// Provides real-time device detection, routing, and monitoring

pub mod device_manager;
pub mod midi_monitor;
pub mod midi_router;

pub use device_manager::{
    connect_device, disconnect_device, get_device_info, list_devices, set_device_mapping,
    MidiDevice, MidiDeviceState,
};

pub use midi_monitor::{
    clear_events, get_events, start_monitoring, stop_monitoring, MidiEvent, MidiMonitorState,
};

pub use midi_router::{
    create_route, delete_route, disable_route, enable_route, get_all_routes, test_route,
    MidiRoute, MidiRouterState,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Hardware state container for all hardware-related state
#[derive(Clone)]
pub struct HardwareState {
    pub device_state: Arc<RwLock<MidiDeviceState>>,
    pub monitor_state: Arc<RwLock<MidiMonitorState>>,
    pub router_state: Arc<RwLock<MidiRouterState>>,
}

impl HardwareState {
    /// Create new hardware state with default values
    pub fn new() -> Self {
        Self {
            device_state: Arc::new(RwLock::new(MidiDeviceState::new())),
            monitor_state: Arc::new(RwLock::new(MidiMonitorState::new())),
            router_state: Arc::new(RwLock::new(MidiRouterState::new())),
        }
    }

    /// Initialize hardware state and scan for devices
    pub async fn initialize(&self) -> Result<()> {
        // Scan for available MIDI devices
        let mut device_state = self.device_state.write().await;
        device_state.scan_devices()?;
        Ok(())
    }

    /// Shutdown hardware state and cleanup resources
    pub async fn shutdown(&self) -> Result<()> {
        // Stop monitoring
        let mut monitor_state = self.monitor_state.write().await;
        monitor_state.stop()?;

        // Disconnect all devices
        let mut device_state = self.device_state.write().await;
        device_state.disconnect_all()?;

        Ok(())
    }
}

impl Default for HardwareState {
    fn default() -> Self {
        Self::new()
    }
}

/// Common error types for hardware operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HardwareError {
    DeviceNotFound(String),
    DeviceAlreadyConnected(String),
    ConnectionFailed(String),
    MonitoringError(String),
    RoutingError(String),
    InvalidConfiguration(String),
}

impl std::fmt::Display for HardwareError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HardwareError::DeviceNotFound(id) => write!(f, "Device not found: {}", id),
            HardwareError::DeviceAlreadyConnected(id) => {
                write!(f, "Device already connected: {}", id)
            }
            HardwareError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            HardwareError::MonitoringError(msg) => write!(f, "Monitoring error: {}", msg),
            HardwareError::RoutingError(msg) => write!(f, "Routing error: {}", msg),
            HardwareError::InvalidConfiguration(msg) => {
                write!(f, "Invalid configuration: {}", msg)
            }
        }
    }
}

impl std::error::Error for HardwareError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hardware_state_creation() {
        let state = HardwareState::new();
        assert!(Arc::strong_count(&state.device_state) >= 1);
        assert!(Arc::strong_count(&state.monitor_state) >= 1);
        assert!(Arc::strong_count(&state.router_state) >= 1);
    }

    #[tokio::test]
    async fn test_hardware_state_default() {
        let state = HardwareState::default();
        let device_state = state.device_state.read().await;
        assert!(device_state.connected_devices.is_empty());
    }
}

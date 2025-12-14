// MIDI Device Manager - Handles device detection and connection
// GROWN-UP SCRIPT - I/O operations with proper error handling

use anyhow::{Context, Result};
use midir::{MidiInput, MidiOutput};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// MIDI device information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MidiDevice {
    pub id: String,
    pub name: String,
    pub manufacturer: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub latency_ms: f64,
    pub is_connected: bool,
}

impl MidiDevice {
    /// Create a new MIDI device
    pub fn new(id: String, name: String, manufacturer: String) -> Self {
        Self {
            id,
            name,
            manufacturer,
            inputs: Vec::new(),
            outputs: Vec::new(),
            latency_ms: 0.0,
            is_connected: false,
        }
    }

    /// Add input port to device
    pub fn add_input(&mut self, port: String) {
        if !self.inputs.contains(&port) {
            self.inputs.push(port);
        }
    }

    /// Add output port to device
    pub fn add_output(&mut self, port: String) {
        if !self.outputs.contains(&port) {
            self.outputs.push(port);
        }
    }
}

/// Device mapping configuration (CC mapping, note mapping, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceMapping {
    pub cc_mappings: HashMap<u8, String>,
    pub note_mappings: HashMap<u8, String>,
    pub channel_mapping: Option<u8>,
}

/// State for MIDI device management
#[derive(Debug, Clone)]
pub struct MidiDeviceState {
    pub connected_devices: HashMap<String, MidiDevice>,
    pub available_devices: HashMap<String, MidiDevice>,
    pub device_mappings: HashMap<String, DeviceMapping>,
}

impl MidiDeviceState {
    /// Create new device state
    pub fn new() -> Self {
        Self {
            connected_devices: HashMap::new(),
            available_devices: HashMap::new(),
            device_mappings: HashMap::new(),
        }
    }

    /// Scan for available MIDI devices using midir
    pub fn scan_devices(&mut self) -> Result<()> {
        self.available_devices.clear();
        info!("Scanning for MIDI devices...");

        // Create temporary midir instances for scanning
        let midi_in = MidiInput::new("MIDI Software Center Scan")
            .context("Failed to create MIDI input for scanning")?;
        let midi_out = MidiOutput::new("MIDI Software Center Scan")
            .context("Failed to create MIDI output for scanning")?;

        // Collect all unique device names from both inputs and outputs
        let mut device_map: HashMap<String, MidiDevice> = HashMap::new();

        // Scan input ports
        for port in midi_in.ports() {
            if let Ok(name) = midi_in.port_name(&port) {
                let device_id = Self::normalize_device_id(&name);
                let manufacturer = Self::extract_manufacturer(&name);

                let device = device_map.entry(device_id.clone()).or_insert_with(|| {
                    debug!("Found MIDI device: {}", name);
                    let mut dev = MidiDevice::new(device_id.clone(), name.clone(), manufacturer);
                    dev.latency_ms = 10.0; // Default ~10ms latency for USB MIDI
                    dev
                });

                device.add_input(name.clone());
            }
        }

        // Scan output ports
        for port in midi_out.ports() {
            if let Ok(name) = midi_out.port_name(&port) {
                let device_id = Self::normalize_device_id(&name);
                let manufacturer = Self::extract_manufacturer(&name);

                let device = device_map.entry(device_id.clone()).or_insert_with(|| {
                    debug!("Found MIDI device: {}", name);
                    let mut dev = MidiDevice::new(device_id.clone(), name.clone(), manufacturer);
                    dev.latency_ms = 10.0;
                    dev
                });

                device.add_output(name.clone());
            }
        }

        // Move devices to available_devices
        for (id, device) in device_map {
            info!(
                "Registered device: {} ({} inputs, {} outputs)",
                device.name,
                device.inputs.len(),
                device.outputs.len()
            );
            self.available_devices.insert(id, device);
        }

        info!(
            "MIDI scan complete: {} devices found",
            self.available_devices.len()
        );
        Ok(())
    }

    /// Scan for mock MIDI devices (for testing)
    /// This provides consistent test data without requiring real hardware
    #[cfg(test)]
    pub fn scan_devices_mock(&mut self) -> Result<()> {
        self.available_devices.clear();

        // Create mock devices for testing
        let mut device1 = MidiDevice::new(
            "midi-device-1".to_string(),
            "MIDI Device 1".to_string(),
            "Test Manufacturer".to_string(),
        );
        device1.add_input("MIDI Device 1 Input".to_string());
        device1.add_output("MIDI Device 1 Output".to_string());
        device1.latency_ms = 5.0;

        let mut device2 = MidiDevice::new(
            "midi-device-2".to_string(),
            "MIDI Device 2".to_string(),
            "Test Manufacturer".to_string(),
        );
        device2.add_input("MIDI Device 2 Input".to_string());
        device2.add_output("MIDI Device 2 Output".to_string());
        device2.latency_ms = 10.0;

        self.available_devices.insert("midi-device-1".to_string(), device1);
        self.available_devices.insert("midi-device-2".to_string(), device2);

        Ok(())
    }

    /// Normalize device name to a consistent ID
    fn normalize_device_id(name: &str) -> String {
        // Remove port numbers and create a base device ID
        // e.g., "UR22mkII:UR22mkII MIDI 1 20:0" -> "ur22mkii"
        let name_lower = name.to_lowercase();
        let base = name_lower
            .split(':')
            .next()
            .unwrap_or(&name_lower)
            .trim()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>();

        if base.is_empty() {
            format!("midi-device-{}", name.len())
        } else {
            base
        }
    }

    /// Extract manufacturer from device name
    fn extract_manufacturer(name: &str) -> String {
        let name_lower = name.to_lowercase();

        // Known manufacturers
        let manufacturers = [
            ("akai", "AKAI Professional"),
            ("steinberg", "Steinberg"),
            ("ur22", "Steinberg"),
            ("yamaha", "Yamaha"),
            ("roland", "Roland"),
            ("korg", "KORG"),
            ("native instruments", "Native Instruments"),
            ("focusrite", "Focusrite"),
            ("m-audio", "M-Audio"),
            ("novation", "Novation"),
            ("arturia", "Arturia"),
            ("behringer", "Behringer"),
            ("mpc", "AKAI Professional"),
            ("force", "AKAI Professional"),
            ("emu", "E-mu Systems"),
            ("proteus", "E-mu Systems"),
        ];

        for (pattern, manufacturer) in manufacturers {
            if name_lower.contains(pattern) {
                return manufacturer.to_string();
            }
        }

        "Unknown".to_string()
    }

    /// Connect to a device
    pub fn connect(&mut self, device_id: &str) -> Result<()> {
        // Check if already connected
        if self.connected_devices.contains_key(device_id) {
            return Err(anyhow::anyhow!("Device already connected: {}", device_id));
        }

        // Find device in available devices
        let device = self
            .available_devices
            .get(device_id)
            .ok_or_else(|| anyhow::anyhow!("Device not found: {}", device_id))?
            .clone();

        // Mark as connected
        let mut connected_device = device;
        connected_device.is_connected = true;

        self.connected_devices.insert(device_id.to_string(), connected_device);

        Ok(())
    }

    /// Disconnect from a device
    pub fn disconnect(&mut self, device_id: &str) -> Result<()> {
        if !self.connected_devices.contains_key(device_id) {
            return Err(anyhow::anyhow!("Device not connected: {}", device_id));
        }

        self.connected_devices.remove(device_id);
        Ok(())
    }

    /// Disconnect all devices
    pub fn disconnect_all(&mut self) -> Result<()> {
        self.connected_devices.clear();
        Ok(())
    }

    /// Get device information
    pub fn get_device(&self, device_id: &str) -> Result<MidiDevice> {
        // Check connected devices first
        if let Some(device) = self.connected_devices.get(device_id) {
            return Ok(device.clone());
        }

        // Check available devices
        self.available_devices
            .get(device_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Device not found: {}", device_id))
    }

    /// Set device mapping
    pub fn set_mapping(&mut self, device_id: String, mapping: DeviceMapping) {
        self.device_mappings.insert(device_id, mapping);
    }

    /// Get device mapping
    pub fn get_mapping(&self, device_id: &str) -> Option<&DeviceMapping> {
        self.device_mappings.get(device_id)
    }
}

impl Default for MidiDeviceState {
    fn default() -> Self {
        Self::new()
    }
}

// Tauri Commands

/// List all available MIDI devices
#[tauri::command]
pub async fn list_devices(
    state: State<'_, Arc<RwLock<MidiDeviceState>>>,
) -> Result<Vec<MidiDevice>, String> {
    list_devices_impl(&state).await.map_err(|e| e.to_string())
}

/// Implementation function for testing - accepts Arc directly
pub async fn list_devices_impl(state: &Arc<RwLock<MidiDeviceState>>) -> Result<Vec<MidiDevice>> {
    let mut device_state = state.write().await;
    device_state.scan_devices().context("Failed to scan devices")?;

    let mut devices: Vec<MidiDevice> = device_state.available_devices.values().cloned().collect();

    // Mark connected devices
    for device in &mut devices {
        if device_state.connected_devices.contains_key(&device.id) {
            device.is_connected = true;
        }
    }

    devices.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(devices)
}

/// Implementation function for testing - doesn't rescan devices
/// This allows tests to use pre-populated mock device data
#[cfg(test)]
pub async fn list_devices_impl_no_scan(
    state: &Arc<RwLock<MidiDeviceState>>,
) -> Result<Vec<MidiDevice>> {
    let device_state = state.read().await;

    let mut devices: Vec<MidiDevice> = device_state.available_devices.values().cloned().collect();

    // Mark connected devices
    for device in &mut devices {
        if device_state.connected_devices.contains_key(&device.id) {
            device.is_connected = true;
        }
    }

    devices.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(devices)
}

/// Connect to a MIDI device
#[tauri::command]
pub async fn connect_device(
    device_id: String,
    state: State<'_, Arc<RwLock<MidiDeviceState>>>,
) -> Result<(), String> {
    connect_device_impl(&device_id, &state).await.map_err(|e| e.to_string())
}

/// Implementation function for testing - accepts Arc directly
pub async fn connect_device_impl(
    device_id: &str,
    state: &Arc<RwLock<MidiDeviceState>>,
) -> Result<()> {
    let mut device_state = state.write().await;
    device_state.connect(device_id).context("Failed to connect to device")?;
    Ok(())
}

/// Disconnect from a MIDI device
#[tauri::command]
pub async fn disconnect_device(
    device_id: String,
    state: State<'_, Arc<RwLock<MidiDeviceState>>>,
) -> Result<(), String> {
    disconnect_device_impl(&device_id, &state).await.map_err(|e| e.to_string())
}

/// Implementation function for testing - accepts Arc directly
pub async fn disconnect_device_impl(
    device_id: &str,
    state: &Arc<RwLock<MidiDeviceState>>,
) -> Result<()> {
    let mut device_state = state.write().await;
    device_state.disconnect(device_id).context("Failed to disconnect from device")?;
    Ok(())
}

/// Get information about a specific device
#[tauri::command]
pub async fn get_device_info(
    device_id: String,
    state: State<'_, Arc<RwLock<MidiDeviceState>>>,
) -> Result<MidiDevice, String> {
    get_device_info_impl(&device_id, &state).await.map_err(|e| e.to_string())
}

/// Implementation function for testing - accepts Arc directly
pub async fn get_device_info_impl(
    device_id: &str,
    state: &Arc<RwLock<MidiDeviceState>>,
) -> Result<MidiDevice> {
    let device_state = state.read().await;
    device_state.get_device(device_id).context("Failed to get device info")
}

/// Set device mapping configuration
#[tauri::command]
pub async fn set_device_mapping(
    device_id: String,
    mapping: HashMap<String, String>,
    state: State<'_, Arc<RwLock<MidiDeviceState>>>,
) -> Result<(), String> {
    set_device_mapping_impl(&device_id, mapping, &state)
        .await
        .map_err(|e| e.to_string())
}

/// Implementation function for testing - accepts Arc directly
pub async fn set_device_mapping_impl(
    device_id: &str,
    mapping: HashMap<String, String>,
    state: &Arc<RwLock<MidiDeviceState>>,
) -> Result<()> {
    let mut device_state = state.write().await;

    // Parse mapping into DeviceMapping structure
    let mut device_mapping = DeviceMapping::default();

    for (key, value) in mapping {
        if let Some(cc_str) = key.strip_prefix("cc_") {
            if let Ok(cc_num) = cc_str.parse::<u8>() {
                device_mapping.cc_mappings.insert(cc_num, value);
            }
        } else if let Some(note_str) = key.strip_prefix("note_") {
            if let Ok(note_num) = note_str.parse::<u8>() {
                device_mapping.note_mappings.insert(note_num, value);
            }
        } else if key == "channel" {
            if let Ok(channel) = value.parse::<u8>() {
                device_mapping.channel_mapping = Some(channel);
            }
        }
    }

    device_state.set_mapping(device_id.to_string(), device_mapping);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_device_creation() {
        let device = MidiDevice::new(
            "test-1".to_string(),
            "Test Device".to_string(),
            "Test Manufacturer".to_string(),
        );

        assert_eq!(device.id, "test-1");
        assert_eq!(device.name, "Test Device");
        assert_eq!(device.manufacturer, "Test Manufacturer");
        assert!(device.inputs.is_empty());
        assert!(device.outputs.is_empty());
        assert_eq!(device.latency_ms, 0.0);
        assert!(!device.is_connected);
    }

    #[test]
    fn test_midi_device_add_ports() {
        let mut device = MidiDevice::new(
            "test-1".to_string(),
            "Test Device".to_string(),
            "Test".to_string(),
        );

        device.add_input("Input 1".to_string());
        device.add_output("Output 1".to_string());

        assert_eq!(device.inputs.len(), 1);
        assert_eq!(device.outputs.len(), 1);

        // Adding duplicate should not increase count
        device.add_input("Input 1".to_string());
        assert_eq!(device.inputs.len(), 1);
    }

    #[test]
    fn test_device_state_creation() {
        let state = MidiDeviceState::new();

        assert!(state.connected_devices.is_empty());
        assert!(state.available_devices.is_empty());
        assert!(state.device_mappings.is_empty());
    }

    #[test]
    fn test_scan_devices_mock() {
        let mut state = MidiDeviceState::new();
        let result = state.scan_devices_mock();

        assert!(result.is_ok());
        assert_eq!(state.available_devices.len(), 2);
    }

    #[test]
    #[ignore] // Requires real MIDI hardware - run with: cargo test -- --ignored
    fn test_scan_devices_real() {
        // Test that real device scanning works (may find 0 or more devices)
        let mut state = MidiDeviceState::new();
        let result = state.scan_devices();

        assert!(result.is_ok());
        // We don't assert on count - may vary by machine
    }

    #[test]
    fn test_connect_device() {
        let mut state = MidiDeviceState::new();
        state.scan_devices_mock().unwrap();

        let result = state.connect("midi-device-1");
        assert!(result.is_ok());
        assert_eq!(state.connected_devices.len(), 1);

        let device = state.connected_devices.get("midi-device-1").unwrap();
        assert!(device.is_connected);
    }

    #[test]
    fn test_connect_nonexistent_device() {
        let mut state = MidiDeviceState::new();
        state.scan_devices_mock().unwrap();

        let result = state.connect("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_connect_already_connected_device() {
        let mut state = MidiDeviceState::new();
        state.scan_devices_mock().unwrap();

        state.connect("midi-device-1").unwrap();
        let result = state.connect("midi-device-1");

        assert!(result.is_err());
    }

    #[test]
    fn test_disconnect_device() {
        let mut state = MidiDeviceState::new();
        state.scan_devices_mock().unwrap();

        state.connect("midi-device-1").unwrap();
        let result = state.disconnect("midi-device-1");

        assert!(result.is_ok());
        assert!(state.connected_devices.is_empty());
    }

    #[test]
    fn test_disconnect_not_connected_device() {
        let mut state = MidiDeviceState::new();
        state.scan_devices_mock().unwrap();

        let result = state.disconnect("midi-device-1");
        assert!(result.is_err());
    }

    #[test]
    fn test_disconnect_all() {
        let mut state = MidiDeviceState::new();
        state.scan_devices_mock().unwrap();

        state.connect("midi-device-1").unwrap();
        state.connect("midi-device-2").unwrap();

        assert_eq!(state.connected_devices.len(), 2);

        state.disconnect_all().unwrap();
        assert!(state.connected_devices.is_empty());
    }

    #[test]
    fn test_get_device() {
        let mut state = MidiDeviceState::new();
        state.scan_devices_mock().unwrap();

        let device = state.get_device("midi-device-1");
        assert!(device.is_ok());

        let device = device.unwrap();
        assert_eq!(device.id, "midi-device-1");
    }

    #[test]
    fn test_get_connected_device() {
        let mut state = MidiDeviceState::new();
        state.scan_devices_mock().unwrap();
        state.connect("midi-device-1").unwrap();

        let device = state.get_device("midi-device-1");
        assert!(device.is_ok());

        let device = device.unwrap();
        assert!(device.is_connected);
    }

    #[test]
    fn test_set_mapping() {
        let mut state = MidiDeviceState::new();

        let mut mapping = DeviceMapping::default();
        mapping.cc_mappings.insert(1, "Modulation".to_string());
        mapping.channel_mapping = Some(1);

        state.set_mapping("midi-device-1".to_string(), mapping);

        let retrieved = state.get_mapping("midi-device-1");
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.cc_mappings.get(&1).unwrap(), "Modulation");
        assert_eq!(retrieved.channel_mapping, Some(1));
    }

    #[tokio::test]
    async fn test_list_devices_impl() {
        let state = Arc::new(RwLock::new(MidiDeviceState::new()));
        {
            let mut s = state.write().await;
            s.scan_devices_mock().unwrap();
        }

        // Use the no-scan version to preserve mock devices
        let devices = list_devices_impl_no_scan(&state).await;
        assert!(devices.is_ok());

        let devices = devices.unwrap();
        assert_eq!(devices.len(), 2);
    }

    #[tokio::test]
    #[ignore] // Requires real MIDI hardware - run with: cargo test -- --ignored
    async fn test_list_devices_impl_real() {
        // Test real device listing (may find 0 or more devices)
        let state = Arc::new(RwLock::new(MidiDeviceState::new()));
        let devices = list_devices_impl(&state).await;
        assert!(devices.is_ok());
        // Don't assert on count - depends on connected hardware
    }

    #[tokio::test]
    async fn test_connect_device_impl() {
        let state = Arc::new(RwLock::new(MidiDeviceState::new()));
        {
            let mut s = state.write().await;
            s.scan_devices_mock().unwrap();
        }

        let result = connect_device_impl("midi-device-1", &state).await;

        assert!(result.is_ok());

        let s = state.read().await;
        assert_eq!(s.connected_devices.len(), 1);
    }

    #[tokio::test]
    async fn test_disconnect_device_impl() {
        let state = Arc::new(RwLock::new(MidiDeviceState::new()));
        {
            let mut s = state.write().await;
            s.scan_devices_mock().unwrap();
            s.connect("midi-device-1").unwrap();
        }

        let result = disconnect_device_impl("midi-device-1", &state).await;

        assert!(result.is_ok());

        let s = state.read().await;
        assert!(s.connected_devices.is_empty());
    }

    #[tokio::test]
    async fn test_get_device_info_impl() {
        let state = Arc::new(RwLock::new(MidiDeviceState::new()));
        {
            let mut s = state.write().await;
            s.scan_devices_mock().unwrap();
        }

        let device = get_device_info_impl("midi-device-1", &state).await;

        assert!(device.is_ok());
        let device = device.unwrap();
        assert_eq!(device.id, "midi-device-1");
    }

    #[tokio::test]
    async fn test_set_device_mapping_impl() {
        let state = Arc::new(RwLock::new(MidiDeviceState::new()));

        let mut mapping = HashMap::new();
        mapping.insert("cc_1".to_string(), "Modulation".to_string());
        mapping.insert("note_60".to_string(), "Middle C".to_string());
        mapping.insert("channel".to_string(), "1".to_string());

        let result = set_device_mapping_impl("midi-device-1", mapping, &state).await;

        assert!(result.is_ok());

        let s = state.read().await;
        let device_mapping = s.get_mapping("midi-device-1");
        assert!(device_mapping.is_some());

        let device_mapping = device_mapping.unwrap();
        assert_eq!(device_mapping.cc_mappings.len(), 1);
        assert_eq!(device_mapping.note_mappings.len(), 1);
        assert_eq!(device_mapping.channel_mapping, Some(1));
    }
}

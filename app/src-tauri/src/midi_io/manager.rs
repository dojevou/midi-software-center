/// MIDI connection manager
///
/// Grown-up Script: Handles MIDI device connections and message transmission.
/// Delegates validation and encoding to Trusty Modules.
use midir::{MidiOutput, MidiOutputConnection};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use crate::core::midi::playback_types::{MidiEventType, MidiMessage};
use crate::core::midi::validator;
use crate::daw_models::MidiDevice;

/// Thread-safe MIDI connection manager
///
/// Manages MIDI output connections with thread-safe access.
/// Uses Arc<Mutex<>> for safe concurrent access from multiple threads.
pub struct MidiManager {
    connection: Arc<Mutex<Option<MidiOutputConnection>>>,
    current_device: Arc<Mutex<Option<String>>>,
}

impl MidiManager {
    /// Create a new MIDI manager
    pub fn new() -> Self {
        info!("Creating MIDI manager");
        Self {
            connection: Arc::new(Mutex::new(None)),
            current_device: Arc::new(Mutex::new(None)),
        }
    }

    /// List available MIDI output devices
    ///
    /// Returns list of all MIDI output ports found on the system.
    pub fn list_devices(&self) -> Result<Vec<MidiDevice>, String> {
        let midi_out = MidiOutput::new("MIDI Library DAW").map_err(|e| {
            error!("Failed to create MIDI output: {}", e);
            format!("Failed to create MIDI output: {}", e)
        })?;

        let ports = midi_out.ports();
        let mut devices = Vec::new();

        for port in ports {
            if let Ok(name) = midi_out.port_name(&port) {
                devices.push(MidiDevice {
                    name: name.clone(),
                    manufacturer: parse_manufacturer(&name),
                });
            }
        }

        info!("Found {} MIDI devices", devices.len());
        Ok(devices)
    }

    /// Connect to a MIDI device
    ///
    /// Establishes connection to the specified MIDI output device.
    pub async fn connect(&self, device_name: &str) -> Result<(), String> {
        info!("Attempting to connect to MIDI device: {}", device_name);

        // Create MIDI output
        let midi_out = MidiOutput::new("MIDI Library DAW").map_err(|e| {
            error!("Failed to create MIDI output: {}", e);
            format!("Failed to create MIDI output: {}", e)
        })?;

        // Find the port
        let ports = midi_out.ports();
        let port = ports
            .iter()
            .find(|p| {
                midi_out.port_name(p).ok().as_ref().map(|n| n == device_name).unwrap_or(false)
            })
            .ok_or_else(|| {
                error!("Device '{}' not found", device_name);
                format!("Device '{}' not found", device_name)
            })?;

        // Connect
        let connection = midi_out.connect(port, "daw-output").map_err(|e| {
            error!("Connection failed: {}", e);
            format!("Connection failed: {}", e)
        })?;

        // Store connection
        let mut conn_lock = self.connection.lock().await;
        *conn_lock = Some(connection);

        let mut device_lock = self.current_device.lock().await;
        *device_lock = Some(device_name.to_string());

        info!("Successfully connected to MIDI device: {}", device_name);

        Ok(())
    }

    /// Disconnect from MIDI device
    ///
    /// Closes the current MIDI connection if one exists.
    pub async fn disconnect(&self) {
        let mut conn_lock = self.connection.lock().await;
        *conn_lock = None;

        let mut device_lock = self.current_device.lock().await;
        let device_name = device_lock.take();

        if let Some(name) = device_name {
            info!("Disconnected from MIDI device: {}", name);
        } else {
            info!("Disconnected (no device was connected)");
        }
    }

    /// Check if connected
    ///
    /// Returns true if a MIDI device is currently connected.
    pub async fn is_connected(&self) -> bool {
        let conn_lock = self.connection.lock().await;
        conn_lock.is_some()
    }

    /// Get current device name
    ///
    /// Returns the name of the currently connected device, if any.
    pub async fn current_device(&self) -> Option<String> {
        let device_lock = self.current_device.lock().await;
        device_lock.clone()
    }

    /// Send a MIDI message
    ///
    /// Validates and sends a MIDI message to the connected device.
    /// Uses Trusty Module validation before sending.
    pub async fn send_message(&self, msg: &MidiMessage) -> Result<(), String> {
        // Validate message (Trusty Module)
        validator::validate_message(msg)?;

        // Get connection
        let mut conn_lock = self.connection.lock().await;

        if let Some(connection) = conn_lock.as_mut() {
            let bytes = msg.to_bytes();
            connection.send(&bytes).map_err(|e| {
                error!("Failed to send MIDI message: {}", e);
                format!("Failed to send MIDI message: {}", e)
            })?;
            Ok(())
        } else {
            warn!("Attempted to send MIDI message with no device connected");
            Err("No MIDI device connected".to_string())
        }
    }

    /// Send a note on message
    ///
    /// Convenience method for sending note on events.
    pub async fn send_note_on(&self, channel: u8, note: u8, velocity: u8) -> Result<(), String> {
        let msg = MidiMessage {
            event_type: MidiEventType::NoteOn,
            channel,
            data1: note,
            data2: velocity,
            timestamp: 0,
        };
        self.send_message(&msg).await
    }

    /// Send a note off message
    ///
    /// Convenience method for sending note off events.
    pub async fn send_note_off(&self, channel: u8, note: u8) -> Result<(), String> {
        let msg = MidiMessage {
            event_type: MidiEventType::NoteOff,
            channel,
            data1: note,
            data2: 0,
            timestamp: 0,
        };
        self.send_message(&msg).await
    }

    /// Send a control change message
    ///
    /// Convenience method for sending control change events.
    pub async fn send_control_change(
        &self,
        channel: u8,
        controller: u8,
        value: u8,
    ) -> Result<(), String> {
        let msg = MidiMessage {
            event_type: MidiEventType::ControlChange,
            channel,
            data1: controller,
            data2: value,
            timestamp: 0,
        };
        self.send_message(&msg).await
    }

    /// Send a program change message
    ///
    /// Convenience method for sending program change events.
    pub async fn send_program_change(&self, channel: u8, program: u8) -> Result<(), String> {
        let msg = MidiMessage {
            event_type: MidiEventType::ProgramChange,
            channel,
            data1: program,
            data2: 0,
            timestamp: 0,
        };
        self.send_message(&msg).await
    }
}

impl Default for MidiManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse manufacturer from device name
///
/// Attempts to extract manufacturer name from MIDI device name.
/// This is a best-effort approach as device naming is not standardized.
fn parse_manufacturer(name: &str) -> Option<String> {
    // Common manufacturer patterns
    if name.contains("Steinberg") {
        Some("Steinberg".to_string())
    } else if name.contains("Akai") {
        Some("Akai".to_string())
    } else if name.contains("Roland") {
        Some("Roland".to_string())
    } else if name.contains("Yamaha") {
        Some("Yamaha".to_string())
    } else if name.contains("Korg") {
        Some("Korg".to_string())
    } else if name.contains("Moog") {
        Some("Moog".to_string())
    } else if name.contains("Arturia") {
        Some("Arturia".to_string())
    } else if name.contains("Native Instruments") || name.contains("NI") {
        Some("Native Instruments".to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires real MIDI hardware - run with: cargo test -- --ignored
    async fn test_list_devices() {
        let manager = MidiManager::new();
        let result = manager.list_devices();

        // Should not error, even if no devices
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_connect_invalid_device() {
        let manager = MidiManager::new();
        let result = manager.connect("NonexistentDevice123").await;

        // Should error for nonexistent device
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_disconnect_when_not_connected() {
        let manager = MidiManager::new();
        manager.disconnect().await;

        // Should not panic
        assert!(!manager.is_connected().await);
    }

    #[tokio::test]
    async fn test_is_connected_initially_false() {
        let manager = MidiManager::new();
        assert!(!manager.is_connected().await);
    }

    #[tokio::test]
    async fn test_current_device_initially_none() {
        let manager = MidiManager::new();
        assert_eq!(manager.current_device().await, None);
    }

    #[test]
    fn test_parse_manufacturer() {
        assert_eq!(
            parse_manufacturer("Steinberg UR22"),
            Some("Steinberg".to_string())
        );
        assert_eq!(parse_manufacturer("Akai MPC One"), Some("Akai".to_string()));
        assert_eq!(parse_manufacturer("Unknown Device"), None);
    }
}

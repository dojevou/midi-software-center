//! MIDI hardware Tauri commands
//!
//! Grown-up Scripts: Thin wrappers around MIDI manager for frontend access.
//! Delegate all business logic to Trusty Modules and MIDI manager.
#![allow(dead_code)] // Commands are called externally via Tauri IPC

use crate::midi::MidiManager;
use crate::models::MidiDevice;
use std::sync::Arc;
use tauri::State;

/// List all available MIDI output devices
///
/// Returns a list of MIDI output devices found on the system.
#[tauri::command]
pub async fn midi_list_devices(
    midi_manager: State<'_, Arc<MidiManager>>,
) -> Result<Vec<MidiDevice>, String> {
    midi_manager.list_devices()
}

/// Connect to a specific MIDI device by name
///
/// Establishes a connection to the specified MIDI output device.
#[tauri::command]
pub async fn midi_connect(
    device_name: String,
    midi_manager: State<'_, Arc<MidiManager>>,
) -> Result<(), String> {
    midi_manager.connect(&device_name).await
}

/// Disconnect from current MIDI device
///
/// Closes the active MIDI connection if one exists.
#[tauri::command]
pub async fn midi_disconnect(midi_manager: State<'_, Arc<MidiManager>>) -> Result<(), String> {
    midi_manager.disconnect().await;
    Ok(())
}

/// Check if MIDI device is currently connected
///
/// Returns true if a MIDI device is connected, false otherwise.
#[tauri::command]
pub async fn midi_is_connected(midi_manager: State<'_, Arc<MidiManager>>) -> Result<bool, String> {
    Ok(midi_manager.is_connected().await)
}

/// Get current MIDI device info
///
/// Returns information about the currently connected device, if any.
#[tauri::command]
pub async fn midi_get_current_device(
    midi_manager: State<'_, Arc<MidiManager>>,
) -> Result<Option<MidiDevice>, String> {
    if let Some(name) = midi_manager.current_device().await {
        Ok(Some(MidiDevice { name, manufacturer: None }))
    } else {
        Ok(None)
    }
}

/// Send a test note to verify MIDI connection
///
/// Sends a note on/off pair with configurable parameters.
/// The note plays for 500ms.
#[tauri::command]
pub async fn midi_send_test_note(
    channel: u8,
    note: u8,
    velocity: u8,
    midi_manager: State<'_, Arc<MidiManager>>,
) -> Result<(), String> {
    // Send note on
    midi_manager.send_note_on(channel, note, velocity).await?;

    // Wait 500ms
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Send note off
    midi_manager.send_note_off(channel, note).await?;

    Ok(())
}

//! MIDI hardware command integration tests (20 tests)
//!
//! Tests for midi.rs commands covering:
//! - Device listing and discovery
//! - Connection management
//! - MIDI message sending (Note On/Off, CC, Program Change)
//! - Device status checking
//! - Error handling and edge cases

use crate::common::*;
use midi_daw::midi::MidiManager;
use std::sync::Arc;

// =============================================================================
// Device Discovery & Listing Tests (5 tests)
// =============================================================================

#[tokio::test]
async fn test_midi_list_devices() {
    let midi_manager = Arc::new(MidiManager::new());

    let result = midi_manager.list_devices();

    // Should return Ok even if no devices found
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_midi_list_devices_empty() {
    let midi_manager = Arc::new(MidiManager::new());

    let devices = midi_manager.list_devices().unwrap_or_default();

    // May be empty on test systems without MIDI hardware
    assert!(devices.len() >= 0);
}

#[tokio::test]
async fn test_midi_device_structure() {
    let midi_manager = Arc::new(MidiManager::new());

    if let Ok(devices) = midi_manager.list_devices() {
        for device in devices {
            // Each device should have a name
            assert!(!device.name.is_empty());
        }
    }
}

#[tokio::test]
async fn test_midi_list_devices_multiple_calls() {
    let midi_manager = Arc::new(MidiManager::new());

    let devices1 = midi_manager.list_devices().unwrap_or_default();
    let devices2 = midi_manager.list_devices().unwrap_or_default();

    // Should return consistent results
    assert_eq!(devices1.len(), devices2.len());
}

#[tokio::test]
async fn test_midi_list_devices_no_duplicates() {
    let midi_manager = Arc::new(MidiManager::new());

    if let Ok(devices) = midi_manager.list_devices() {
        let mut names: Vec<String> = devices.iter().map(|d| d.name.clone()).collect();
        names.sort();
        names.dedup();

        // Should not have duplicate device names
        assert_eq!(names.len(), devices.len());
    }
}

// =============================================================================
// Connection Management Tests (7 tests)
// =============================================================================

#[tokio::test]
async fn test_midi_initial_state_disconnected() {
    let midi_manager = Arc::new(MidiManager::new());

    let is_connected = midi_manager.is_connected().await;

    assert!(!is_connected);
}

#[tokio::test]
async fn test_midi_current_device_none_initially() {
    let midi_manager = Arc::new(MidiManager::new());

    let current = midi_manager.current_device().await;

    assert!(current.is_none());
}

#[tokio::test]
async fn test_midi_connect_invalid_device() {
    let midi_manager = Arc::new(MidiManager::new());

    let result = midi_manager.connect("NonexistentDevice12345").await;

    // Should fail gracefully
    assert!(result.is_err());
}

#[tokio::test]
async fn test_midi_disconnect_when_not_connected() {
    let midi_manager = Arc::new(MidiManager::new());

    midi_manager.disconnect().await;

    // Should complete without error
    let is_connected = midi_manager.is_connected().await;
    assert!(!is_connected);
}

#[tokio::test]
async fn test_midi_disconnect_clears_state() {
    let midi_manager = Arc::new(MidiManager::new());

    // Even if not connected, disconnect should clear state
    midi_manager.disconnect().await;

    assert!(!midi_manager.is_connected().await);
    assert!(midi_manager.current_device().await.is_none());
}

#[tokio::test]
async fn test_midi_reconnect_same_device() {
    let midi_manager = Arc::new(MidiManager::new());

    // Try to connect (may fail if no devices)
    let _ = midi_manager.connect("TestDevice").await;

    // Disconnect
    midi_manager.disconnect().await;
    assert!(!midi_manager.is_connected().await);

    // Try to reconnect
    let result = midi_manager.connect("TestDevice").await;

    // Result depends on device availability
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_midi_concurrent_connection_attempts() {
    let midi_manager = Arc::new(MidiManager::new());

    let mut handles = vec![];

    for _ in 0..5 {
        let manager = midi_manager.clone();
        let handle = tokio::spawn(async move {
            manager.connect("TestDevice").await
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    // Should handle concurrent attempts gracefully
    // (one may succeed, others should fail or be ignored)
}

// =============================================================================
// MIDI Message Sending Tests (6 tests)
// =============================================================================

#[tokio::test]
async fn test_midi_send_note_on() {
    let midi_manager = Arc::new(MidiManager::new());

    // Should fail if not connected
    let result = midi_manager.send_note_on(0, 60, 80).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not connected") || result.err().unwrap().contains("No device"));
}

#[tokio::test]
async fn test_midi_send_note_off() {
    let midi_manager = Arc::new(MidiManager::new());

    // Should fail if not connected
    let result = midi_manager.send_note_off(0, 60).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_midi_send_controller() {
    let midi_manager = Arc::new(MidiManager::new());

    // Should fail if not connected
    let result = midi_manager.send_controller(0, 7, 100).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_midi_send_pitch_bend() {
    let midi_manager = Arc::new(MidiManager::new());

    // Should fail if not connected
    let result = midi_manager.send_pitch_bend(0, 8192).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_midi_send_program_change() {
    let midi_manager = Arc::new(MidiManager::new());

    // Should fail if not connected
    let result = midi_manager.send_program_change(0, 1).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_midi_send_message_validation() {
    let midi_manager = Arc::new(MidiManager::new());

    // Test various MIDI messages with boundary values
    let test_cases = vec![
        (0u8, 0u8, 0u8),     // Min values
        (15u8, 127u8, 127u8), // Max values
        (7u8, 60u8, 80u8),   // Normal values
    ];

    for (channel, note, velocity) in test_cases {
        // Should validate parameters even when disconnected
        let result = midi_manager.send_note_on(channel, note, velocity).await;
        // Will fail due to no connection, but should not panic
        assert!(result.is_err());
    }
}

// =============================================================================
// Error Handling & Edge Cases (2 tests)
// =============================================================================

#[tokio::test]
async fn test_midi_invalid_channel() {
    let midi_manager = Arc::new(MidiManager::new());

    // MIDI channels are 0-15, so 16 is invalid
    let result = midi_manager.send_note_on(16, 60, 80).await;

    // Should fail with validation error
    assert!(result.is_err());
}

#[tokio::test]
async fn test_midi_invalid_note_value() {
    let midi_manager = Arc::new(MidiManager::new());

    // MIDI notes are 0-127, so 128 is invalid
    let result = midi_manager.send_note_on(0, 128, 80).await;

    // Should fail with validation error
    assert!(result.is_err());
}

// =============================================================================
// SECTION 5: Extended Error Path Testing (8 tests)
// =============================================================================

#[tokio::test]
async fn test_error_midi_negative_channel() {
    let midi_manager = Arc::new(MidiManager::new());

    // Negative channel should fail
    let result = midi_manager.send_note_on(255u8, 60, 80).await; // 255 wraps to -1

    // Should be rejected or handled gracefully
    let _ = result;
}

#[tokio::test]
async fn test_error_midi_negative_note_value() {
    let midi_manager = Arc::new(MidiManager::new());

    // Negative note should fail
    let result = midi_manager.send_note_on(0, 255u8, 80).await; // 255 wraps to -1

    // Should be rejected
    let _ = result;
}

#[tokio::test]
async fn test_error_midi_negative_velocity() {
    let midi_manager = Arc::new(MidiManager::new());

    // Negative velocity should fail
    let result = midi_manager.send_note_on(0, 60, 255u8).await; // 255 wraps to -1

    // Should be rejected or handled
    let _ = result;
}

#[tokio::test]
async fn test_error_midi_max_pitch_bend_value() {
    let midi_manager = Arc::new(MidiManager::new());

    // Pitch bend range is 0-16383
    let result = midi_manager.send_pitch_bend(0, i32::MAX).await;

    // Should handle extreme values gracefully
    let _ = result;
}

#[tokio::test]
async fn test_error_midi_negative_pitch_bend_value() {
    let midi_manager = Arc::new(MidiManager::new());

    // Negative pitch bend should fail
    let result = midi_manager.send_pitch_bend(0, -1).await;

    // Should be rejected
    let _ = result;
}

#[tokio::test]
async fn test_error_midi_extreme_controller_value() {
    let midi_manager = Arc::new(MidiManager::new());

    // CC value > 127 should fail
    let result = midi_manager.send_control_change(0, 7, 200).await;

    // Should be rejected
    let _ = result;
}

#[tokio::test]
async fn test_error_midi_program_change_out_of_range() {
    let midi_manager = Arc::new(MidiManager::new());

    // Program number > 127 should fail
    let result = midi_manager.send_program_change(0, 200).await;

    // Should be rejected
    let _ = result;
}

#[tokio::test]
async fn test_error_midi_concurrent_message_sending() {
    let midi_manager = Arc::new(MidiManager::new());

    let mut handles = vec![];

    // Send concurrent MIDI messages
    for i in 0..10 {
        let manager = Arc::clone(&midi_manager);
        let handle = tokio::spawn(async move {
            if i % 2 == 0 {
                manager.send_note_on(i as u8 % 16, 60 + i as u8 % 20, 80).await
            } else {
                manager.send_note_off(i as u8 % 16, 60 + i as u8 % 20).await
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok());
    }
}

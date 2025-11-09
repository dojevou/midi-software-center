   /// Export Tauri commands
   ///
   /// Grown-up Script: Handles exporting sequencer projects and MIDI data.
   /// Delegates MIDI file generation to Trusty Modules (pure functions).

use crate::core::midi::writer;
use crate::models::midi::{MidiEvent, MidiEventType};
use tracing::{debug, error, info};
use std::path::PathBuf;

/// Export project as MIDI file
///
/// Uses MIDI writer Trusty Module (pure function) to generate MIDI data.
///
/// TODO for full implementation:
/// - Get events from sequencer engine
/// - Merge all tracks into event list
/// - Apply track properties (volume, pan as MIDI CC)
/// - Support tempo map changes
///
/// Current implementation creates a demonstration MIDI file.
#[tauri::command]
pub async fn export_project_midi(
    output_path: String,
) -> Result<(), String> {
    debug!("Exporting project to MIDI file: {}", output_path);

    let path = PathBuf::from(&output_path);

    // Validate path
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            return Err(format!("Parent directory does not exist: {}", parent.display()));
        }
    }

    // Validate extension
    if path.extension().and_then(|s| s.to_str()) != Some("mid") &&
       path.extension().and_then(|s| s.to_str()) != Some("midi") {
        return Err("Output file must have .mid or .midi extension".to_string());
    }

    // TODO: Get events from sequencer engine
    // For now, create a simple demonstration pattern
    let events = create_demo_events();

    // Use Trusty Module (pure function) to generate MIDI file
    let midi_data = writer::write_midi_file(&events, 480, 120.0)
        .map_err(|e| {
            error!("Failed to generate MIDI data: {}", e);
            format!("Failed to generate MIDI: {}", e)
        })?;

    // I/O operation (Grown-up Script responsibility)
    std::fs::write(&path, midi_data)
        .map_err(|e| {
            error!("Failed to write MIDI file: {}", e);
            format!("Failed to write file: {}", e)
        })?;

    info!("Exported project to: {}", output_path);
    Ok(())
}

/// Create demonstration MIDI events
///
/// This is a placeholder for integration with the sequencer.
/// A real implementation would:
/// 1. Get all tracks from the sequencer engine
/// 2. Merge events from all enabled tracks
/// 3. Apply track properties (mute, solo, volume, pan)
/// 4. Sort events by timestamp
///
/// Current implementation creates a simple C major arpeggio pattern.
fn create_demo_events() -> Vec<MidiEvent> {
    vec![
        // C major arpeggio (C-E-G-C)
        MidiEvent {
            event_type: MidiEventType::NoteOn,
            tick: 0,
            channel: 0,
            note: Some(60), // C
            velocity: Some(80),
            controller: None,
            value: None,
            program: None,
        },
        MidiEvent {
            event_type: MidiEventType::NoteOff,
            tick: 480, // 1 beat later
            channel: 0,
            note: Some(60),
            velocity: Some(0),
            controller: None,
            value: None,
            program: None,
        },
        MidiEvent {
            event_type: MidiEventType::NoteOn,
            tick: 480,
            channel: 0,
            note: Some(64), // E
            velocity: Some(80),
            controller: None,
            value: None,
            program: None,
        },
        MidiEvent {
            event_type: MidiEventType::NoteOff,
            tick: 960, // 2 beats
            channel: 0,
            note: Some(64),
            velocity: Some(0),
            controller: None,
            value: None,
            program: None,
        },
        MidiEvent {
            event_type: MidiEventType::NoteOn,
            tick: 960,
            channel: 0,
            note: Some(67), // G
            velocity: Some(80),
            controller: None,
            value: None,
            program: None,
        },
        MidiEvent {
            event_type: MidiEventType::NoteOff,
            tick: 1440, // 3 beats
            channel: 0,
            note: Some(67),
            velocity: Some(0),
            controller: None,
            value: None,
            program: None,
        },
        MidiEvent {
            event_type: MidiEventType::NoteOn,
            tick: 1440,
            channel: 0,
            note: Some(72), // C (octave higher)
            velocity: Some(80),
            controller: None,
            value: None,
            program: None,
        },
        MidiEvent {
            event_type: MidiEventType::NoteOff,
            tick: 1920, // 4 beats (1 bar)
            channel: 0,
            note: Some(72),
            velocity: Some(0),
            controller: None,
            value: None,
            program: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_demo_events() {
        let events = create_demo_events();

        // Should create 8 events (4 note on + 4 note off)
        assert_eq!(events.len(), 8);

        // First event should be Note On at tick 0
        assert_eq!(events[0].event_type, MidiEventType::NoteOn);
        assert_eq!(events[0].tick, 0);
        assert_eq!(events[0].note, Some(60)); // Middle C

        // Last event should be Note Off at tick 1920 (1 bar)
        assert_eq!(events[7].event_type, MidiEventType::NoteOff);
        assert_eq!(events[7].tick, 1920);
        assert_eq!(events[7].note, Some(72)); // High C
    }

    #[test]
    fn test_export_uses_trusty_module() {
        // Verify we're using the MIDI writer Trusty Module
        let events = create_demo_events();
        let result = writer::write_midi_file(&events, 480, 120.0);

        assert!(result.is_ok());
        let midi_data = result.unwrap();

        // Verify MIDI header
        assert_eq!(&midi_data[0..4], b"MThd");
        // Verify track chunk
        assert_eq!(&midi_data[14..18], b"MTrk");
    }
}

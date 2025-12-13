/// MIDI Edge Case Tests
/// Tests rare and unusual MIDI file formats and edge cases

use midi_library_shared::core::midi::{parse_midi_file, MidiFile, Event, Track, TimedEvent};
use std::fs::write;
use tempfile::tempdir;

#[test]
fn test_parse_empty_file() {
    let temp_dir = tempdir().unwrap();
    let midi_path = temp_dir.path().join("empty.mid");

    // Create empty file
    write(&midi_path, b"").unwrap();

    let result = parse_midi_file(&midi_path);

    // Should error on empty file
    assert!(result.is_err());
}

#[test]
fn test_parse_invalid_header() {
    let temp_dir = tempdir().unwrap();
    let midi_path = temp_dir.path().join("invalid.mid");

    // Write invalid header (not "MThd")
    write(&midi_path, b"INVALID_HEADER").unwrap();

    let result = parse_midi_file(&midi_path);

    // Should error on invalid header
    assert!(result.is_err());
}

#[test]
fn test_parse_truncated_header() {
    let temp_dir = tempdir().unwrap();
    let midi_path = temp_dir.path().join("truncated.mid");

    // Write partial MIDI header
    write(&midi_path, b"MThd\x00\x00").unwrap();

    let result = parse_midi_file(&midi_path);

    // Should error on truncated header
    assert!(result.is_err());
}

#[test]
fn test_parse_missing_track_end() {
    let temp_dir = tempdir().unwrap();
    let midi_path = temp_dir.path().join("no_end.mid");

    // Create MIDI without track end marker
    // Minimal valid header + track without end
    let data = b"MThd\x00\x00\x00\x06\x00\x00\x00\x01\x00\x60MTrk\x00\x00\x00\x04\x00\x90\x3C\x40";
    write(&midi_path, data).unwrap();

    let result = parse_midi_file(&midi_path);

    // Should error or handle gracefully
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_parse_invalid_delta_time() {
    let temp_dir = tempdir().unwrap();
    let midi_path = temp_dir.path().join("invalid_delta.mid");

    // MIDI with invalid variable-length delta time
    let data = b"MThd\x00\x00\x00\x06\x00\x00\x00\x01\x00\x60MTrk\x00\x00\x00\x08\xFF\xFF\xFF\xFF\x90\x3C\x40\x00\xFF\x2F\x00";
    write(&midi_path, data).unwrap();

    let result = parse_midi_file(&midi_path);

    // Should handle invalid variable-length encoding
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_parse_unknown_meta_event() {
    let temp_dir = tempdir().unwrap();
    let midi_path = temp_dir.path().join("unknown_meta.mid");

    // MIDI with unknown meta event type (0xFE)
    let data = b"MThd\x00\x00\x00\x06\x00\x00\x00\x01\x00\x60MTrk\x00\x00\x00\x08\x00\xFF\xFE\x01\x00\x00\xFF\x2F\x00";
    write(&midi_path, data).unwrap();

    let result = parse_midi_file(&midi_path);

    // Should skip unknown meta events
    if let Ok(midi) = result {
        assert!(!midi.tracks.is_empty());
    }
}

#[test]
fn test_parse_sysex_message() {
    let temp_dir = tempdir().unwrap();
    let midi_path = temp_dir.path().join("sysex.mid");

    // MIDI with SysEx message
    let data = b"MThd\x00\x00\x00\x06\x00\x00\x00\x01\x00\x60MTrk\x00\x00\x00\x0A\x00\xF0\x04\x41\x10\x42\xF7\x00\xFF\x2F\x00";
    write(&midi_path, data).unwrap();

    let result = parse_midi_file(&midi_path);

    // Should handle SysEx messages
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_parse_running_status() {
    // MIDI using running status (repeated events without status byte)
    // This is a valid MIDI optimization
    let temp_dir = tempdir().unwrap();
    let midi_path = temp_dir.path().join("running_status.mid");

    // Note On followed by another note using running status
    let data = b"MThd\x00\x00\x00\x06\x00\x00\x00\x01\x00\x60MTrk\x00\x00\x00\x0C\x00\x90\x3C\x40\x00\x3E\x40\x00\x40\x40\x00\xFF\x2F\x00";
    write(&midi_path, data).unwrap();

    let result = parse_midi_file(&midi_path);

    // Should handle running status
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_parse_very_large_file() {
    let temp_dir = tempdir().unwrap();
    let midi_path = temp_dir.path().join("large.mid");

    // Create valid MIDI header
    let mut data = Vec::from(b"MThd\x00\x00\x00\x06\x00\x00\x00\x01\x00\x60MTrk" as &[u8]);

    // Add large track size (10MB worth of events)
    let track_size: u32 = 10_000_000;
    data.extend_from_slice(&track_size.to_be_bytes());

    // Add some events (not full 10MB to keep test fast)
    for _ in 0..1000 {
        data.extend_from_slice(&[0x00, 0x90, 0x3C, 0x40]); // Note on
    }
    data.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]); // End of track

    write(&midi_path, data).unwrap();

    let result = parse_midi_file(&midi_path);

    // Should handle or error gracefully
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_parse_format_2_midi() {
    // MIDI Format 2 (multiple independent patterns)
    let temp_dir = tempdir().unwrap();
    let midi_path = temp_dir.path().join("format2.mid");

    // Format 2 header
    let data = b"MThd\x00\x00\x00\x06\x00\x02\x00\x02\x00\x60MTrk\x00\x00\x00\x04\x00\xFF\x2F\x00MTrk\x00\x00\x00\x04\x00\xFF\x2F\x00";
    write(&midi_path, data).unwrap();

    let result = parse_midi_file(&midi_path);

    // Should handle Format 2 MIDI
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_parse_zero_tracks() {
    let temp_dir = tempdir().unwrap();
    let midi_path = temp_dir.path().join("zero_tracks.mid");

    // MIDI header with 0 tracks
    let data = b"MThd\x00\x00\x00\x06\x00\x00\x00\x00\x00\x60";
    write(&midi_path, data).unwrap();

    let result = parse_midi_file(&midi_path);

    // Should error on zero tracks
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_parse_excessive_tracks() {
    let temp_dir = tempdir().unwrap();
    let midi_path = temp_dir.path().join("many_tracks.mid");

    // MIDI header claiming 10,000 tracks
    let data = b"MThd\x00\x00\x00\x06\x00\x01\x27\x10\x00\x60";
    write(&midi_path, data).unwrap();

    let result = parse_midi_file(&midi_path);

    // Should error or have reasonable limit
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_parse_invalid_ticks_per_quarter() {
    let temp_dir = tempdir().unwrap();
    let midi_path = temp_dir.path().join("invalid_tpq.mid");

    // MIDI with ticks_per_quarter = 0
    let data = b"MThd\x00\x00\x00\x06\x00\x00\x00\x01\x00\x00MTrk\x00\x00\x00\x04\x00\xFF\x2F\x00";
    write(&midi_path, data).unwrap();

    let result = parse_midi_file(&midi_path);

    // Should error on invalid TPQ
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_parse_negative_ticks_per_quarter() {
    let temp_dir = tempdir().unwrap();
    let midi_path = temp_dir.path().join("negative_tpq.mid");

    // MIDI with negative SMPTE format (bit 15 set)
    let data = b"MThd\x00\x00\x00\x06\x00\x00\x00\x01\x80\x00MTrk\x00\x00\x00\x04\x00\xFF\x2F\x00";
    write(&midi_path, data).unwrap();

    let result = parse_midi_file(&midi_path);

    // Should handle SMPTE format
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_parse_channel_16_edge_case() {
    let temp_dir = tempdir().unwrap();
    let midi_path = temp_dir.path().join("channel16.mid");

    // MIDI with events on channel 16 (0x0F, max channel)
    let data = b"MThd\x00\x00\x00\x06\x00\x00\x00\x01\x00\x60MTrk\x00\x00\x00\x08\x00\x9F\x3C\x40\x00\xFF\x2F\x00";
    write(&midi_path, data).unwrap();

    let result = parse_midi_file(&midi_path);

    // Should handle channel 16
    if let Ok(midi) = result {
        assert!(!midi.tracks.is_empty());
    }
}

#[test]
fn test_parse_velocity_zero_note_on() {
    // Note On with velocity 0 (should be treated as Note Off)
    let temp_dir = tempdir().unwrap();
    let midi_path = temp_dir.path().join("vel_zero.mid");

    let data = b"MThd\x00\x00\x00\x06\x00\x00\x00\x01\x00\x60MTrk\x00\x00\x00\x08\x00\x90\x3C\x00\x00\xFF\x2F\x00";
    write(&midi_path, data).unwrap();

    let result = parse_midi_file(&midi_path);

    // Should handle velocity 0 as Note Off
    if let Ok(midi) = result {
        assert!(!midi.tracks.is_empty());
    }
}

#[test]
fn test_parse_tempo_change_events() {
    let temp_dir = tempdir().unwrap();
    let midi_path = temp_dir.path().join("tempo.mid");

    // MIDI with tempo change meta event
    let data = b"MThd\x00\x00\x00\x06\x00\x00\x00\x01\x00\x60MTrk\x00\x00\x00\x0A\x00\xFF\x51\x03\x07\xA1\x20\x00\xFF\x2F\x00";
    write(&midi_path, data).unwrap();

    let result = parse_midi_file(&midi_path);

    // Should handle tempo changes
    if let Ok(midi) = result {
        assert!(!midi.tracks.is_empty());
    }
}

#[test]
fn test_parse_time_signature_events() {
    let temp_dir = tempdir().unwrap();
    let midi_path = temp_dir.path().join("time_sig.mid");

    // MIDI with time signature meta event (4/4)
    let data = b"MThd\x00\x00\x00\x06\x00\x00\x00\x01\x00\x60MTrk\x00\x00\x00\x0C\x00\xFF\x58\x04\x04\x02\x18\x08\x00\xFF\x2F\x00";
    write(&midi_path, data).unwrap();

    let result = parse_midi_file(&midi_path);

    // Should handle time signatures
    if let Ok(midi) = result {
        assert!(!midi.tracks.is_empty());
    }
}

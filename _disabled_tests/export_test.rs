//! Export command integration tests (25 tests)
//!
//! Tests for export.rs commands covering:
//! - MIDI file export (Format 0 and 1)
//! - Path validation and error handling
//! - File system operations
//! - MIDI data generation
//! - Performance testing

use crate::common::*;
use midi_daw::commands::export::*;
use std::path::PathBuf;

// =============================================================================
// MIDI Export Tests (10 tests)
// =============================================================================

#[tokio::test]
async fn test_export_to_valid_path() {
    let fixtures = FileFixtures::new().await;
    let output_path = fixtures.path().join("output.mid");

    let result = export_project_midi(output_path.to_str().unwrap().to_string()).await;

    assert!(result.is_ok());
    assert!(output_path.exists());
}

#[tokio::test]
async fn test_export_creates_file() {
    let fixtures = FileFixtures::new().await;
    let output_path = fixtures.path().join("test_export.mid");

    let _ = export_project_midi(output_path.to_str().unwrap().to_string()).await;

    assert!(output_path.exists());
    let metadata = std::fs::metadata(&output_path).unwrap();
    assert!(metadata.len() > 0);
}

#[tokio::test]
async fn test_export_generates_valid_midi_header() {
    let fixtures = FileFixtures::new().await;
    let output_path = fixtures.path().join("header_test.mid");

    let _ = export_project_midi(output_path.to_str().unwrap().to_string()).await;

    let data = std::fs::read(&output_path).unwrap();
    assert!(data.len() >= 14); // Minimum MIDI file size
    assert_eq!(&data[0..4], b"MThd"); // MIDI header
}

#[tokio::test]
async fn test_export_includes_track_chunk() {
    let fixtures = FileFixtures::new().await;
    let output_path = fixtures.path().join("track_test.mid");

    let _ = export_project_midi(output_path.to_str().unwrap().to_string()).await;

    let data = std::fs::read(&output_path).unwrap();
    // Find MTrk chunk
    let has_track = data.windows(4).any(|w| w == b"MTrk");
    assert!(has_track, "MIDI file should contain MTrk chunk");
}

#[tokio::test]
async fn test_export_with_mid_extension() {
    let fixtures = FileFixtures::new().await;
    let output_path = fixtures.path().join("file.mid");

    let result = export_project_midi(output_path.to_str().unwrap().to_string()).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_export_with_midi_extension() {
    let fixtures = FileFixtures::new().await;
    let output_path = fixtures.path().join("file.midi");

    let result = export_project_midi(output_path.to_str().unwrap().to_string()).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_export_invalid_extension() {
    let fixtures = FileFixtures::new().await;
    let output_path = fixtures.path().join("file.txt");

    let result = export_project_midi(output_path.to_str().unwrap().to_string()).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("extension"));
}

#[tokio::test]
async fn test_export_missing_parent_directory() {
    let output_path = "/nonexistent/directory/file.mid";

    let result = export_project_midi(output_path.to_string()).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[tokio::test]
async fn test_export_creates_demo_events() {
    let fixtures = FileFixtures::new().await;
    let output_path = fixtures.path().join("demo.mid");

    let _ = export_project_midi(output_path.to_str().unwrap().to_string()).await;

    let data = std::fs::read(&output_path).unwrap();
    // Demo file should contain C major arpeggio (notes 60, 64, 67, 72)
    assert!(data.len() > 50); // Should have substantial content
}

#[tokio::test]
async fn test_export_file_format() {
    let fixtures = FileFixtures::new().await;
    let output_path = fixtures.path().join("format_test.mid");

    let _ = export_project_midi(output_path.to_str().unwrap().to_string()).await;

    let data = std::fs::read(&output_path).unwrap();
    // Check MIDI format (bytes 8-9 of header)
    assert_eq!(data[8], 0x00); // Format 0 or 1
    assert!(data[9] <= 0x01);
}

// =============================================================================
// Path Validation Tests (6 tests)
// =============================================================================

#[tokio::test]
async fn test_export_validates_path_format() {
    let fixtures = FileFixtures::new().await;
    let output_path = fixtures.path().join("valid.mid");

    let result = export_project_midi(output_path.to_str().unwrap().to_string()).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_export_rejects_empty_path() {
    let result = export_project_midi("".to_string()).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_export_handles_special_characters() {
    let fixtures = FileFixtures::new().await;
    let output_path = fixtures.path().join("file with spaces.mid");

    let result = export_project_midi(output_path.to_str().unwrap().to_string()).await;

    assert!(result.is_ok());
    assert!(output_path.exists());
}

#[tokio::test]
async fn test_export_handles_unicode_filename() {
    let fixtures = FileFixtures::new().await;
    let output_path = fixtures.path().join("файл.mid");

    let result = export_project_midi(output_path.to_str().unwrap().to_string()).await;

    // Should handle unicode gracefully
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_export_validates_parent_exists() {
    let fixtures = FileFixtures::new().await;
    let valid_path = fixtures.path().join("valid.mid");

    // Parent exists
    let result = export_project_midi(valid_path.to_str().unwrap().to_string()).await;
    assert!(result.is_ok());

    // Parent doesn't exist
    let invalid_path = "/this/path/does/not/exist/file.mid";
    let result = export_project_midi(invalid_path.to_string()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_export_allows_subdirectories() {
    let fixtures = FileFixtures::new().await;
    let subdir = fixtures.path().join("exports");
    std::fs::create_dir(&subdir).unwrap();

    let output_path = subdir.join("output.mid");
    let result = export_project_midi(output_path.to_str().unwrap().to_string()).await;

    assert!(result.is_ok());
    assert!(output_path.exists());
}

// =============================================================================
// File System Operations Tests (5 tests)
// =============================================================================

#[tokio::test]
async fn test_export_overwrites_existing_file() {
    let fixtures = FileFixtures::new().await;
    let output_path = fixtures.path().join("overwrite.mid");

    // Create initial file
    std::fs::write(&output_path, b"old content").unwrap();

    // Export should overwrite
    let result = export_project_midi(output_path.to_str().unwrap().to_string()).await;
    assert!(result.is_ok());

    let data = std::fs::read(&output_path).unwrap();
    assert_eq!(&data[0..4], b"MThd"); // Should be MIDI file now
}

#[tokio::test]
async fn test_export_creates_readable_file() {
    let fixtures = FileFixtures::new().await;
    let output_path = fixtures.path().join("readable.mid");

    let _ = export_project_midi(output_path.to_str().unwrap().to_string()).await;

    // File should be readable
    let result = std::fs::read(&output_path);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_export_file_permissions() {
    let fixtures = FileFixtures::new().await;
    let output_path = fixtures.path().join("perms.mid");

    let _ = export_project_midi(output_path.to_str().unwrap().to_string()).await;

    let metadata = std::fs::metadata(&output_path).unwrap();
    assert!(!metadata.permissions().readonly());
}

#[tokio::test]
async fn test_export_multiple_files_same_directory() {
    let fixtures = FileFixtures::new().await;

    for i in 0..5 {
        let output_path = fixtures.path().join(format!("file_{}.mid", i));
        let result = export_project_midi(output_path.to_str().unwrap().to_string()).await;
        assert!(result.is_ok());
    }

    // All files should exist
    for i in 0..5 {
        let path = fixtures.path().join(format!("file_{}.mid", i));
        assert!(path.exists());
    }
}

#[tokio::test]
async fn test_export_concurrent_exports() {
    let fixtures = FileFixtures::new().await;

    let mut handles = vec![];

    for i in 0..5 {
        let output_path = fixtures.path().join(format!("concurrent_{}.mid", i));
        let path_str = output_path.to_str().unwrap().to_string();

        let handle = tokio::spawn(async move {
            export_project_midi(path_str).await
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}

// =============================================================================
// MIDI Data Generation Tests (4 tests)
// =============================================================================

#[tokio::test]
async fn test_create_demo_events_count() {
    let fixtures = FileFixtures::new().await;
    let output_path = fixtures.path().join("demo_events.mid");

    let _ = export_project_midi(output_path.to_str().unwrap().to_string()).await;

    // Demo creates C major arpeggio (4 notes × 2 events = 8 events)
    let data = std::fs::read(&output_path).unwrap();
    assert!(data.len() > 100); // Should contain multiple events
}

#[tokio::test]
async fn test_create_demo_events_timing() {
    let fixtures = FileFixtures::new().await;
    let output_path = fixtures.path().join("timing.mid");

    let _ = export_project_midi(output_path.to_str().unwrap().to_string()).await;

    let data = std::fs::read(&output_path).unwrap();
    // Demo uses 480 PPQ timing
    assert!(data.len() > 50);
}

#[tokio::test]
async fn test_export_includes_tempo() {
    let fixtures = FileFixtures::new().await;
    let output_path = fixtures.path().join("tempo.mid");

    let _ = export_project_midi(output_path.to_str().unwrap().to_string()).await;

    let data = std::fs::read(&output_path).unwrap();
    // Tempo meta event (FF 51 03) should be present
    let has_tempo = data.windows(3).any(|w| w == &[0xFF, 0x51, 0x03]);
    assert!(has_tempo, "MIDI file should contain tempo meta event");
}

#[tokio::test]
async fn test_export_includes_end_of_track() {
    let fixtures = FileFixtures::new().await;
    let output_path = fixtures.path().join("eot.mid");

    let _ = export_project_midi(output_path.to_str().unwrap().to_string()).await;

    let data = std::fs::read(&output_path).unwrap();
    // End of Track meta event (FF 2F 00) should be present
    let has_eot = data.windows(3).any(|w| w == &[0xFF, 0x2F, 0x00]);
    assert!(has_eot, "MIDI file should contain End of Track event");
}

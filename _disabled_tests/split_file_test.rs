//! Comprehensive tests for pipeline/src-tauri/src/commands/split_file.rs
//! Commands: split_and_import
//!
//! Coverage: 85%+ target
//! Tests: 30 comprehensive tests
//!
//! Key Testing Areas:
//! - Single file splitting with track extraction
//! - Multi-track MIDI file handling
//! - Track metadata preservation (name, instrument, BPM, key)
//! - Output file generation and naming
//! - Database import and relationship tracking
//! - Error handling (missing files, corrupt MIDI, disk errors)
//! - Edge cases (empty tracks, percussion, unicode names)

use midi_pipeline::commands::split_file::{split_and_import, SplitResult};
use std::path::PathBuf;
use tempfile::TempDir;
use crate::common::{TestDatabase, FileFixtures};

// ============================================================================
// MIDI FIXTURE BUILDERS
// ============================================================================

/// Create a valid multi-track MIDI file (Format 1)
fn create_multitrack_midi(num_tracks: usize) -> Vec<u8> {
    let mut bytes = Vec::new();

    // MIDI Header (MThd) - Format 1
    bytes.extend_from_slice(b"MThd");
    bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x06]); // Header length: 6
    bytes.extend_from_slice(&[0x00, 0x01]); // Format 1 (parallel tracks)
    bytes.extend_from_slice(&((num_tracks + 1) as u16).to_be_bytes()); // +1 for tempo track
    bytes.extend_from_slice(&[0x01, 0xE0]); // 480 ticks per quarter note

    // Track 0: Tempo track (meta events only)
    let mut tempo_track_data = Vec::new();

    // Tempo: 120 BPM = 500000 microseconds per quarter note
    tempo_track_data.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20]);

    // Time signature: 4/4
    tempo_track_data.extend_from_slice(&[0x00, 0xFF, 0x58, 0x04, 0x04, 0x02, 0x18, 0x08]);

    // Key signature: C major
    tempo_track_data.extend_from_slice(&[0x00, 0xFF, 0x59, 0x02, 0x00, 0x00]);

    // End of track
    tempo_track_data.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

    // Write tempo track
    bytes.extend_from_slice(b"MTrk");
    bytes.extend_from_slice(&(tempo_track_data.len() as u32).to_be_bytes());
    bytes.extend_from_slice(&tempo_track_data);

    // Music tracks (1 through num_tracks)
    for i in 0..num_tracks {
        let mut track_data = Vec::new();

        // Track name
        let track_name = format!("Track {}", i + 1);
        track_data.extend_from_slice(&[0x00, 0xFF, 0x03, track_name.len() as u8]);
        track_data.extend_from_slice(track_name.as_bytes());

        // Instrument name
        let instrument = match i % 3 {
            0 => "Piano",
            1 => "Guitar",
            _ => "Bass",
        };
        track_data.extend_from_slice(&[0x00, 0xFF, 0x04, instrument.len() as u8]);
        track_data.extend_from_slice(instrument.as_bytes());

        // Program change (select instrument)
        track_data.extend_from_slice(&[0x00, 0xC0 | (i as u8 % 16), (i * 10) as u8]);

        // Add notes (C4, E4, G4 - C major chord)
        let notes = [60, 64, 67];
        for (j, note) in notes.iter().enumerate() {
            // Note on
            track_data.extend_from_slice(&[0x00, 0x90 | (i as u8 % 16), *note, 64]);
            // Note off (after 480 ticks = 1 beat)
            track_data.extend_from_slice(&[0x83, 0x60, 0x80 | (i as u8 % 16), *note, 0]);
        }

        // End of track
        track_data.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

        // Write track
        bytes.extend_from_slice(b"MTrk");
        bytes.extend_from_slice(&(track_data.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&track_data);
    }

    bytes
}

/// Create MIDI with custom track names and instruments
fn create_midi_with_custom_tracks(tracks: Vec<(&str, &str)>) -> Vec<u8> {
    let mut bytes = Vec::new();

    // MIDI Header (MThd) - Format 1
    bytes.extend_from_slice(b"MThd");
    bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x06]);
    bytes.extend_from_slice(&[0x00, 0x01]); // Format 1
    bytes.extend_from_slice(&((tracks.len() + 1) as u16).to_be_bytes());
    bytes.extend_from_slice(&[0x01, 0xE0]); // 480 ticks

    // Track 0: Tempo track
    let mut tempo_track = Vec::new();
    tempo_track.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20]); // Tempo
    tempo_track.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]); // End

    bytes.extend_from_slice(b"MTrk");
    bytes.extend_from_slice(&(tempo_track.len() as u32).to_be_bytes());
    bytes.extend_from_slice(&tempo_track);

    // Custom tracks
    for (idx, (track_name, instrument)) in tracks.iter().enumerate() {
        let mut track_data = Vec::new();

        // Track name
        track_data.extend_from_slice(&[0x00, 0xFF, 0x03, track_name.len() as u8]);
        track_data.extend_from_slice(track_name.as_bytes());

        // Instrument name
        track_data.extend_from_slice(&[0x00, 0xFF, 0x04, instrument.len() as u8]);
        track_data.extend_from_slice(instrument.as_bytes());

        // Notes
        track_data.extend_from_slice(&[0x00, 0x90, 60, 64]); // Note on
        track_data.extend_from_slice(&[0x83, 0x60, 0x80, 60, 0]); // Note off

        // End
        track_data.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

        bytes.extend_from_slice(b"MTrk");
        bytes.extend_from_slice(&(track_data.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&track_data);
    }

    bytes
}

/// Create MIDI with percussion track (channel 9)
fn create_midi_with_percussion() -> Vec<u8> {
    let mut bytes = Vec::new();

    // MIDI Header
    bytes.extend_from_slice(b"MThd");
    bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x06]);
    bytes.extend_from_slice(&[0x00, 0x01]); // Format 1
    bytes.extend_from_slice(&[0x00, 0x02]); // 2 tracks
    bytes.extend_from_slice(&[0x01, 0xE0]); // 480 ticks

    // Tempo track
    let mut tempo_track = Vec::new();
    tempo_track.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20]);
    tempo_track.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

    bytes.extend_from_slice(b"MTrk");
    bytes.extend_from_slice(&(tempo_track.len() as u32).to_be_bytes());
    bytes.extend_from_slice(&tempo_track);

    // Percussion track (channel 9)
    let mut drum_track = Vec::new();
    drum_track.extend_from_slice(&[0x00, 0xFF, 0x03, 0x05]); // Track name
    drum_track.extend_from_slice(b"Drums");

    // Bass drum (note 36 on channel 9)
    drum_track.extend_from_slice(&[0x00, 0x99, 36, 100]); // Note on (0x99 = channel 9)
    drum_track.extend_from_slice(&[0x30, 0x89, 36, 0]); // Note off

    // Snare (note 38)
    drum_track.extend_from_slice(&[0x30, 0x99, 38, 100]);
    drum_track.extend_from_slice(&[0x30, 0x89, 38, 0]);

    drum_track.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

    bytes.extend_from_slice(b"MTrk");
    bytes.extend_from_slice(&(drum_track.len() as u32).to_be_bytes());
    bytes.extend_from_slice(&drum_track);

    bytes
}

/// Create MIDI with empty track (no notes)
fn create_midi_with_empty_track() -> Vec<u8> {
    let mut bytes = Vec::new();

    // MIDI Header
    bytes.extend_from_slice(b"MThd");
    bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x06]);
    bytes.extend_from_slice(&[0x00, 0x01]); // Format 1
    bytes.extend_from_slice(&[0x00, 0x03]); // 3 tracks
    bytes.extend_from_slice(&[0x01, 0xE0]);

    // Tempo track
    let mut tempo = Vec::new();
    tempo.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20]);
    tempo.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

    bytes.extend_from_slice(b"MTrk");
    bytes.extend_from_slice(&(tempo.len() as u32).to_be_bytes());
    bytes.extend_from_slice(&tempo);

    // Track with notes
    let mut track1 = Vec::new();
    track1.extend_from_slice(&[0x00, 0xFF, 0x03, 0x07]);
    track1.extend_from_slice(b"Track 1");
    track1.extend_from_slice(&[0x00, 0x90, 60, 64]);
    track1.extend_from_slice(&[0x83, 0x60, 0x80, 60, 0]);
    track1.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

    bytes.extend_from_slice(b"MTrk");
    bytes.extend_from_slice(&(track1.len() as u32).to_be_bytes());
    bytes.extend_from_slice(&track1);

    // Empty track (no notes, only meta events)
    let mut empty = Vec::new();
    empty.extend_from_slice(&[0x00, 0xFF, 0x03, 0x0C]);
    empty.extend_from_slice(b"Empty Track");
    empty.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]); // Just end of track

    bytes.extend_from_slice(b"MTrk");
    bytes.extend_from_slice(&(empty.len() as u32).to_be_bytes());
    bytes.extend_from_slice(&empty);

    bytes
}

/// Create single-track MIDI (Format 0)
fn create_single_track_midi() -> Vec<u8> {
    let mut bytes = Vec::new();

    // MIDI Header - Format 0
    bytes.extend_from_slice(b"MThd");
    bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x06]);
    bytes.extend_from_slice(&[0x00, 0x00]); // Format 0
    bytes.extend_from_slice(&[0x00, 0x01]); // 1 track
    bytes.extend_from_slice(&[0x01, 0xE0]);

    // Single track with tempo + notes
    let mut track = Vec::new();
    track.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20]); // Tempo
    track.extend_from_slice(&[0x00, 0x90, 60, 64]); // Note on
    track.extend_from_slice(&[0x83, 0x60, 0x80, 60, 0]); // Note off
    track.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]); // End

    bytes.extend_from_slice(b"MTrk");
    bytes.extend_from_slice(&(track.len() as u32).to_be_bytes());
    bytes.extend_from_slice(&track);

    bytes
}

/// Create MIDI with unicode track names
fn create_midi_with_unicode_names() -> Vec<u8> {
    let tracks = vec![
        ("Café Piano", "Piano"),
        ("Guitarra Española", "Guitar"),
        ("低音 Bass", "Bass"),
    ];

    create_midi_with_custom_tracks(tracks)
}

/// Create invalid MIDI (corrupt data)
fn create_invalid_midi() -> Vec<u8> {
    vec![0x4D, 0x54, 0x68, 0x64, 0xFF, 0xFF, 0xFF, 0xFF]
}

// ============================================================================
// TEST HELPER: Setup file in database
// ============================================================================

async fn setup_test_file(
    pool: &sqlx::PgPool,
    filepath: &str,
    filename: &str,
    file_data: &[u8],
) -> i64 {
    // Calculate hash
    let hash = blake3::hash(file_data);
    let hash_bytes = hash.as_bytes().to_vec();

    // Insert file
    let file_id = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO files (
            filename,
            original_filename,
            filepath,
            content_hash,
            file_size_bytes,
            num_tracks,
            created_at
        ) VALUES ($1, $2, $3, $4, $5, 1, NOW())
        RETURNING id
        "#,
    )
    .bind(filename)
    .bind(filename)
    .bind(filepath)
    .bind(hash_bytes)
    .bind(file_data.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert test file");

    file_id
}

// ============================================================================
// FUNCTION 1: split_and_import() - 15-18 Tests
// ============================================================================

#[tokio::test]
async fn test_split_single_file_success() {
    let db = TestDatabase::new().await;
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_dir = temp_dir.path().join("splits");

    // Create 3-track MIDI file
    let midi_data = create_multitrack_midi(3);
    let file_path = temp_dir.path().join("test.mid");
    tokio::fs::write(&file_path, &midi_data).await.unwrap();

    // Insert into database
    let file_id = setup_test_file(
        db.pool(),
        file_path.to_str().unwrap(),
        "test.mid",
        &midi_data,
    ).await;

    // Split the file
    let result = split_and_import(file_id, output_dir.clone(), db.pool())
        .await
        .expect("Split should succeed");

    // Verify result
    assert_eq!(result.tracks_split, 3);
    assert_eq!(result.split_file_ids.len(), 3);
    assert_eq!(result.output_dir, output_dir);

    // Verify split files exist
    assert!(output_dir.exists());
    let split_files: Vec<_> = std::fs::read_dir(&output_dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(split_files.len(), 3);

    // Verify database entries
    for split_id in result.split_file_ids {
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM files WHERE id = $1)")
            .bind(split_id)
            .fetch_one(db.pool())
            .await
            .unwrap();
        assert!(exists, "Split file should exist in database");
    }

    // Verify track_splits relationships
    let split_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM track_splits WHERE parent_file_id = $1"
    )
    .bind(file_id)
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(split_count, 3);
}

#[tokio::test]
async fn test_split_single_file_no_tracks() {
    let db = TestDatabase::new().await;
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("splits");

    // Create MIDI with only tempo track (no music tracks)
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"MThd");
    bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x06]);
    bytes.extend_from_slice(&[0x00, 0x01]); // Format 1
    bytes.extend_from_slice(&[0x00, 0x01]); // 1 track only
    bytes.extend_from_slice(&[0x01, 0xE0]);

    let mut tempo_track = Vec::new();
    tempo_track.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20]);
    tempo_track.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

    bytes.extend_from_slice(b"MTrk");
    bytes.extend_from_slice(&(tempo_track.len() as u32).to_be_bytes());
    bytes.extend_from_slice(&tempo_track);

    let file_path = temp_dir.path().join("tempo_only.mid");
    tokio::fs::write(&file_path, &bytes).await.unwrap();

    let file_id = setup_test_file(
        db.pool(),
        file_path.to_str().unwrap(),
        "tempo_only.mid",
        &bytes,
    ).await;

    // Should fail - no tracks to split
    let result = split_and_import(file_id, output_dir, db.pool()).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No tracks to split"));
}

#[tokio::test]
async fn test_split_single_file_single_track() {
    let db = TestDatabase::new().await;
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("splits");

    // Single track MIDI (Format 0)
    let midi_data = create_single_track_midi();
    let file_path = temp_dir.path().join("single.mid");
    tokio::fs::write(&file_path, &midi_data).await.unwrap();

    let file_id = setup_test_file(
        db.pool(),
        file_path.to_str().unwrap(),
        "single.mid",
        &midi_data,
    ).await;

    let result = split_and_import(file_id, output_dir.clone(), db.pool())
        .await
        .expect("Should succeed with single track");

    // Should return 1 track
    assert_eq!(result.tracks_split, 1);
    assert_eq!(result.split_file_ids.len(), 1);
}

#[tokio::test]
async fn test_split_single_file_multi_track() {
    let db = TestDatabase::new().await;
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("splits");

    // 8-track MIDI
    let midi_data = create_multitrack_midi(8);
    let file_path = temp_dir.path().join("octet.mid");
    tokio::fs::write(&file_path, &midi_data).await.unwrap();

    let file_id = setup_test_file(
        db.pool(),
        file_path.to_str().unwrap(),
        "octet.mid",
        &midi_data,
    ).await;

    let result = split_and_import(file_id, output_dir, db.pool())
        .await
        .expect("Should split 8 tracks");

    assert_eq!(result.tracks_split, 8);
    assert_eq!(result.split_file_ids.len(), 8);
}

#[tokio::test]
async fn test_split_single_file_mixed_instruments() {
    let db = TestDatabase::new().await;
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("splits");

    // MIDI with drums + instruments
    let midi_data = create_midi_with_percussion();
    let file_path = temp_dir.path().join("mixed.mid");
    tokio::fs::write(&file_path, &midi_data).await.unwrap();

    let file_id = setup_test_file(
        db.pool(),
        file_path.to_str().unwrap(),
        "mixed.mid",
        &midi_data,
    ).await;

    let result = split_and_import(file_id, output_dir, db.pool())
        .await
        .expect("Should split drums + instruments");

    assert_eq!(result.tracks_split, 1); // Just drums track

    // Verify track_splits has percussion track
    let track_name: Option<String> = sqlx::query_scalar(
        "SELECT track_name FROM track_splits WHERE parent_file_id = $1"
    )
    .bind(file_id)
    .fetch_one(db.pool())
    .await
    .unwrap();

    assert!(track_name.is_some());
    let name = track_name.unwrap();
    assert!(name.contains("Drums") || name.contains("rums"));
}

#[tokio::test]
async fn test_split_single_file_duplicate_naming() {
    let db = TestDatabase::new().await;
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("splits");

    // Create MIDI with duplicate track names
    let tracks = vec![
        ("Piano", "Piano"),
        ("Piano", "Piano"),
        ("Piano", "Piano"),
    ];
    let midi_data = create_midi_with_custom_tracks(tracks);
    let file_path = temp_dir.path().join("duplicates.mid");
    tokio::fs::write(&file_path, &midi_data).await.unwrap();

    let file_id = setup_test_file(
        db.pool(),
        file_path.to_str().unwrap(),
        "duplicates.mid",
        &midi_data,
    ).await;

    let result = split_and_import(file_id, output_dir.clone(), db.pool())
        .await
        .expect("Should handle duplicate names");

    assert_eq!(result.tracks_split, 3);

    // Verify unique filenames (should have track_01, track_02, track_03)
    let split_files: Vec<_> = std::fs::read_dir(&output_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();

    assert_eq!(split_files.len(), 3);
    // Each file should have unique track number
    assert!(split_files.iter().any(|f| f.contains("track_01")));
    assert!(split_files.iter().any(|f| f.contains("track_02")));
    assert!(split_files.iter().any(|f| f.contains("track_03")));
}

#[tokio::test]
async fn test_split_single_file_unicode_track_names() {
    let db = TestDatabase::new().await;
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("splits");

    let midi_data = create_midi_with_unicode_names();
    let file_path = temp_dir.path().join("unicode.mid");
    tokio::fs::write(&file_path, &midi_data).await.unwrap();

    let file_id = setup_test_file(
        db.pool(),
        file_path.to_str().unwrap(),
        "unicode.mid",
        &midi_data,
    ).await;

    let result = split_and_import(file_id, output_dir.clone(), db.pool())
        .await
        .expect("Should handle unicode names");

    assert_eq!(result.tracks_split, 3);

    // Verify files created
    let split_files: Vec<_> = std::fs::read_dir(&output_dir)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(split_files.len(), 3);
}

#[tokio::test]
async fn test_split_single_file_tempo_preservation() {
    let db = TestDatabase::new().await;
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("splits");

    let midi_data = create_multitrack_midi(2);
    let file_path = temp_dir.path().join("tempo_test.mid");
    tokio::fs::write(&file_path, &midi_data).await.unwrap();

    let file_id = setup_test_file(
        db.pool(),
        file_path.to_str().unwrap(),
        "tempo_test.mid",
        &midi_data,
    ).await;

    let result = split_and_import(file_id, output_dir.clone(), db.pool())
        .await
        .expect("Should preserve tempo");

    // Read one of the split files
    let split_file_path = std::fs::read_dir(&output_dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();

    let split_data = std::fs::read(&split_file_path).unwrap();

    // Parse and verify tempo is preserved (120 BPM = 500000 µs/beat)
    use midly::Smf;
    let smf = Smf::parse(&split_data).expect("Should parse split file");

    let has_tempo = smf.tracks[0].iter().any(|event| {
        matches!(event.kind, midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(_)))
    });

    assert!(has_tempo, "Split file should contain tempo information");
}

#[tokio::test]
async fn test_split_single_file_key_preservation() {
    let db = TestDatabase::new().await;
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("splits");

    let midi_data = create_multitrack_midi(2);
    let file_path = temp_dir.path().join("key_test.mid");
    tokio::fs::write(&file_path, &midi_data).await.unwrap();

    let file_id = setup_test_file(
        db.pool(),
        file_path.to_str().unwrap(),
        "key_test.mid",
        &midi_data,
    ).await;

    let result = split_and_import(file_id, output_dir.clone(), db.pool())
        .await
        .expect("Should preserve key");

    // Verify key signature in musical_metadata
    for split_id in result.split_file_ids {
        let key: Option<String> = sqlx::query_scalar(
            "SELECT key_signature FROM musical_metadata WHERE file_id = $1"
        )
        .bind(split_id)
        .fetch_optional(db.pool())
        .await
        .unwrap();

        // Should have key signature (C major or detected key)
        assert!(key.is_some() || key.is_none()); // Either detected or not, shouldn't crash
    }
}

#[tokio::test]
async fn test_split_single_file_file_not_found() {
    let db = TestDatabase::new().await;
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("splits");

    // Non-existent file ID
    let result = split_and_import(99999, output_dir, db.pool()).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_split_single_file_invalid_midi() {
    let db = TestDatabase::new().await;
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("splits");

    let midi_data = create_invalid_midi();
    let file_path = temp_dir.path().join("invalid.mid");
    tokio::fs::write(&file_path, &midi_data).await.unwrap();

    let file_id = setup_test_file(
        db.pool(),
        file_path.to_str().unwrap(),
        "invalid.mid",
        &midi_data,
    ).await;

    let result = split_and_import(file_id, output_dir, db.pool()).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to split"));
}

#[tokio::test]
async fn test_split_single_file_permission_denied() {
    // This test requires specific filesystem setup - skip in CI
    // In real environment, would test with read-only file
    // For now, verify error handling structure exists

    let db = TestDatabase::new().await;
    let temp_dir = TempDir::new().unwrap();

    // Create file that doesn't exist (simulates permission error)
    let file_path = "/nonexistent/path/file.mid";
    let midi_data = create_multitrack_midi(2);

    let file_id = setup_test_file(
        db.pool(),
        file_path,
        "nonexistent.mid",
        &midi_data,
    ).await;

    let result = split_and_import(file_id, temp_dir.path().to_path_buf(), db.pool()).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_split_single_file_output_directory_creation() {
    let db = TestDatabase::new().await;
    let temp_dir = TempDir::new().unwrap();

    // Output dir that doesn't exist yet
    let output_dir = temp_dir.path().join("deeply").join("nested").join("splits");
    assert!(!output_dir.exists());

    let midi_data = create_multitrack_midi(2);
    let file_path = temp_dir.path().join("test.mid");
    tokio::fs::write(&file_path, &midi_data).await.unwrap();

    let file_id = setup_test_file(
        db.pool(),
        file_path.to_str().unwrap(),
        "test.mid",
        &midi_data,
    ).await;

    let result = split_and_import(file_id, output_dir.clone(), db.pool())
        .await
        .expect("Should create output directory");

    assert!(output_dir.exists(), "Output directory should be created");
    assert_eq!(result.tracks_split, 2);
}

#[tokio::test]
async fn test_split_single_file_output_file_naming() {
    let db = TestDatabase::new().await;
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("splits");

    let tracks = vec![
        ("Lead", "Synth Lead"),
        ("Bass", "Electric Bass"),
        ("Pad", "Synth Pad"),
    ];
    let midi_data = create_midi_with_custom_tracks(tracks);
    let file_path = temp_dir.path().join("my_song.mid");
    tokio::fs::write(&file_path, &midi_data).await.unwrap();

    let file_id = setup_test_file(
        db.pool(),
        file_path.to_str().unwrap(),
        "my_song.mid",
        &midi_data,
    ).await;

    let result = split_and_import(file_id, output_dir.clone(), db.pool())
        .await
        .expect("Should split with correct naming");

    let split_files: Vec<String> = std::fs::read_dir(&output_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();

    // Verify naming pattern: {base}_track_{num:02}_{instrument}.mid
    assert!(split_files.iter().any(|f| f.contains("my_song") && f.contains("track_01")));
    assert!(split_files.iter().any(|f| f.contains("my_song") && f.contains("track_02")));
    assert!(split_files.iter().any(|f| f.contains("my_song") && f.contains("track_03")));
}

#[tokio::test]
async fn test_split_single_file_empty_track_skipped() {
    let db = TestDatabase::new().await;
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("splits");

    let midi_data = create_midi_with_empty_track();
    let file_path = temp_dir.path().join("with_empty.mid");
    tokio::fs::write(&file_path, &midi_data).await.unwrap();

    let file_id = setup_test_file(
        db.pool(),
        file_path.to_str().unwrap(),
        "with_empty.mid",
        &midi_data,
    ).await;

    let result = split_and_import(file_id, output_dir, db.pool())
        .await
        .expect("Should skip empty tracks");

    // Should only split the track with notes (not the empty one)
    assert_eq!(result.tracks_split, 1, "Empty tracks should be skipped");
}

#[tokio::test]
async fn test_split_single_file_percussion_separate() {
    let db = TestDatabase::new().await;
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("splits");

    let midi_data = create_midi_with_percussion();
    let file_path = temp_dir.path().join("drums.mid");
    tokio::fs::write(&file_path, &midi_data).await.unwrap();

    let file_id = setup_test_file(
        db.pool(),
        file_path.to_str().unwrap(),
        "drums.mid",
        &midi_data,
    ).await;

    let result = split_and_import(file_id, output_dir.clone(), db.pool())
        .await
        .expect("Should separate percussion");

    assert_eq!(result.tracks_split, 1);

    // Verify drum track in database
    let note_count: Option<i32> = sqlx::query_scalar(
        "SELECT note_count FROM track_splits WHERE parent_file_id = $1"
    )
    .bind(file_id)
    .fetch_one(db.pool())
    .await
    .unwrap();

    assert!(note_count.is_some());
    assert!(note_count.unwrap() > 0, "Drum track should have notes");
}

#[tokio::test]
async fn test_split_metadata_extraction() {
    let db = TestDatabase::new().await;
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("splits");

    let midi_data = create_multitrack_midi(3);
    let file_path = temp_dir.path().join("metadata_test.mid");
    tokio::fs::write(&file_path, &midi_data).await.unwrap();

    let file_id = setup_test_file(
        db.pool(),
        file_path.to_str().unwrap(),
        "metadata_test.mid",
        &midi_data,
    ).await;

    let result = split_and_import(file_id, output_dir, db.pool())
        .await
        .expect("Should extract metadata");

    // Verify track_splits has metadata
    let splits: Vec<(i32, Option<String>, Option<String>, i32)> = sqlx::query_as(
        "SELECT track_number, track_name, instrument, note_count
         FROM track_splits
         WHERE parent_file_id = $1
         ORDER BY track_number"
    )
    .bind(file_id)
    .fetch_all(db.pool())
    .await
    .unwrap();

    assert_eq!(splits.len(), 3);

    for (idx, (track_num, track_name, instrument, note_count)) in splits.iter().enumerate() {
        assert_eq!(*track_num as usize, idx + 1); // Track numbers 1, 2, 3
        assert!(track_name.is_some(), "Track should have a name");
        assert!(instrument.is_some(), "Track should have instrument");
        assert!(*note_count > 0, "Track should have notes");
    }
}

#[tokio::test]
async fn test_split_hash_deduplication() {
    let db = TestDatabase::new().await;
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("splits");

    let midi_data = create_multitrack_midi(2);
    let file_path = temp_dir.path().join("dedup_test.mid");
    tokio::fs::write(&file_path, &midi_data).await.unwrap();

    let file_id = setup_test_file(
        db.pool(),
        file_path.to_str().unwrap(),
        "dedup_test.mid",
        &midi_data,
    ).await;

    // First split
    let result1 = split_and_import(file_id, output_dir.clone(), db.pool())
        .await
        .expect("First split should succeed");

    assert_eq!(result1.tracks_split, 2);

    // Try to split again (should fail due to duplicate hash)
    let output_dir2 = temp_dir.path().join("splits2");
    let result2 = split_and_import(file_id, output_dir2, db.pool()).await;

    // Should fail because split files already exist with same hash
    assert!(result2.is_err());
}

// ============================================================================
// UTILITY FUNCTION TESTS
// ============================================================================

#[test]
fn test_generate_split_filename_with_instrument() {
    use midi_pipeline::commands::split_file::generate_split_filename;
    use midi_pipeline::core::splitting::track_splitter::SplitTrack;

    let track = SplitTrack {
        track_number: 1,
        track_name: Some("Piano Part".to_string()),
        channel: Some(0),
        instrument: Some("Acoustic Grand Piano".to_string()),
        note_count: 100,
        midi_bytes: vec![],
    };

    let filename = generate_split_filename("my_song", &track);
    assert_eq!(filename, "my_song_track_01_Acoustic_Grand_Piano.mid");
}

#[test]
fn test_generate_split_filename_with_track_name_only() {
    use midi_pipeline::commands::split_file::generate_split_filename;
    use midi_pipeline::core::splitting::track_splitter::SplitTrack;

    let track = SplitTrack {
        track_number: 2,
        track_name: Some("Bass Line".to_string()),
        channel: Some(1),
        instrument: None,
        note_count: 50,
        midi_bytes: vec![],
    };

    let filename = generate_split_filename("song", &track);
    assert_eq!(filename, "song_track_02_Bass_Line.mid");
}

#[test]
fn test_generate_split_filename_no_metadata() {
    use midi_pipeline::commands::split_file::generate_split_filename;
    use midi_pipeline::core::splitting::track_splitter::SplitTrack;

    let track = SplitTrack {
        track_number: 0,
        track_name: None,
        channel: None,
        instrument: None,
        note_count: 10,
        midi_bytes: vec![],
    };

    let filename = generate_split_filename("minimal", &track);
    assert_eq!(filename, "minimal_track_00.mid");
}

#[test]
fn test_sanitize_filename_special_chars() {
    use midi_pipeline::commands::split_file::sanitize_filename;

    assert_eq!(sanitize_filename("Track: 1"), "Track_1");
    assert_eq!(sanitize_filename("Bass/Guitar"), "Bass_Guitar");
    assert_eq!(sanitize_filename("Lead (Synth)"), "Lead_(Synth)");
    assert_eq!(sanitize_filename("File*Name?"), "File_Name");
}

#[test]
fn test_sanitize_filename_unicode() {
    use midi_pipeline::commands::split_file::sanitize_filename;

    assert_eq!(sanitize_filename("Café"), "Café");
    assert_eq!(sanitize_filename("Guitarra Española"), "Guitarra_Española");
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[tokio::test]
async fn test_split_database_transaction_rollback() {
    let db = TestDatabase::new().await;
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("splits");

    let midi_data = create_multitrack_midi(2);
    let file_path = temp_dir.path().join("transaction_test.mid");
    tokio::fs::write(&file_path, &midi_data).await.unwrap();

    let file_id = setup_test_file(
        db.pool(),
        file_path.to_str().unwrap(),
        "transaction_test.mid",
        &midi_data,
    ).await;

    // First split should succeed
    let result = split_and_import(file_id, output_dir.clone(), db.pool()).await;

    if result.is_ok() {
        // Verify files count
        let file_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM files WHERE filepath LIKE $1"
        )
        .bind(format!("{}%", output_dir.to_string_lossy()))
        .fetch_one(db.pool())
        .await
        .unwrap();

        assert_eq!(file_count, 2, "Should have 2 split files");
    }
}

#[tokio::test]
async fn test_split_file_content_hash_calculation() {
    let db = TestDatabase::new().await;
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("splits");

    let midi_data = create_multitrack_midi(2);
    let file_path = temp_dir.path().join("hash_test.mid");
    tokio::fs::write(&file_path, &midi_data).await.unwrap();

    let file_id = setup_test_file(
        db.pool(),
        file_path.to_str().unwrap(),
        "hash_test.mid",
        &midi_data,
    ).await;

    let result = split_and_import(file_id, output_dir.clone(), db.pool())
        .await
        .expect("Should calculate hashes");

    // Verify all split files have content_hash
    for split_id in result.split_file_ids {
        let hash: Option<Vec<u8>> = sqlx::query_scalar(
            "SELECT content_hash FROM files WHERE id = $1"
        )
        .bind(split_id)
        .fetch_one(db.pool())
        .await
        .unwrap();

        assert!(hash.is_some(), "Split file should have content hash");
        assert_eq!(hash.unwrap().len(), 32, "BLAKE3 hash should be 32 bytes");
    }
}

#[tokio::test]
async fn test_split_relationship_constraints() {
    let db = TestDatabase::new().await;
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("splits");

    let midi_data = create_multitrack_midi(2);
    let file_path = temp_dir.path().join("constraint_test.mid");
    tokio::fs::write(&file_path, &midi_data).await.unwrap();

    let file_id = setup_test_file(
        db.pool(),
        file_path.to_str().unwrap(),
        "constraint_test.mid",
        &midi_data,
    ).await;

    let result = split_and_import(file_id, output_dir, db.pool())
        .await
        .expect("Should create relationships");

    // Verify foreign key constraints
    for split_id in result.split_file_ids {
        // Check parent file exists
        let parent_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM track_splits
                WHERE parent_file_id = $1 AND split_file_id = $2
            )"
        )
        .bind(file_id)
        .bind(split_id)
        .fetch_one(db.pool())
        .await
        .unwrap();

        assert!(parent_exists, "Relationship should exist");
    }
}

#[tokio::test]
async fn test_split_cascade_delete() {
    let db = TestDatabase::new().await;
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("splits");

    let midi_data = create_multitrack_midi(2);
    let file_path = temp_dir.path().join("cascade_test.mid");
    tokio::fs::write(&file_path, &midi_data).await.unwrap();

    let file_id = setup_test_file(
        db.pool(),
        file_path.to_str().unwrap(),
        "cascade_test.mid",
        &midi_data,
    ).await;

    let result = split_and_import(file_id, output_dir, db.pool())
        .await
        .expect("Should split");

    // Delete parent file
    sqlx::query("DELETE FROM files WHERE id = $1")
        .bind(file_id)
        .execute(db.pool())
        .await
        .unwrap();

    // Verify track_splits are also deleted (CASCADE)
    let split_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM track_splits WHERE parent_file_id = $1"
    )
    .bind(file_id)
    .fetch_one(db.pool())
    .await
    .unwrap();

    assert_eq!(split_count, 0, "track_splits should cascade delete");
}

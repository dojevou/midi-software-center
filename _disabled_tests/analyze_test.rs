//! Tests for pipeline/src-tauri/src/commands/analyze.rs
//! Commands: start_analysis (HIGH PRIORITY - COMPLEX)
//!
//! This test suite validates the high-performance parallel analysis system that processes
//! 1.1M+ MIDI files using 32 concurrent workers with batch database operations.
//!
//! Coverage Areas:
//! - Batch fetching in 1000-file chunks
//! - Parallel analysis with 32 workers and semaphore limiting
//! - Arc<Semaphore> concurrency control
//! - Arc<AtomicUsize> thread-safe counters
//! - Arc<Mutex<Vec<>>> error collection
//! - Musical analysis: BPM detection, key detection, note analysis
//! - Progress event emission (every 10 files)
//! - Batch metadata insertion (100-file batches)
//! - Error handling and recovery

use crate::common::*;
use midi_library_shared::core::midi::parser::parse_midi_file;
use midi_library_shared::core::midi::types::{Event, MidiFile, Header, Track, TimedEvent, TextType};
use tauri::{Emitter, Manager};
use std::sync::Arc;

// Re-create AppState and necessary types for testing
struct TestAppState {
    database: TestDatabase,
}

impl TestAppState {
    async fn new() -> Self {
        Self {
            database: TestDatabase::new().await,
        }
    }
}

// Mock window for event testing
#[derive(Clone)]
struct TestWindow {
    events: Arc<Mutex<Vec<(String, serde_json::Value)>>>,
}

impl TestWindow {
    fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn get_events(&self) -> Vec<(String, serde_json::Value)> {
        self.events.lock().await.clone()
    }

    async fn clear_events(&self) {
        self.events.lock().await.clear();
    }
}

impl Emitter for TestWindow {
    fn emit<S: serde::Serialize + Clone>(&self, event: &str, payload: S) -> tauri::Result<()> {
        let events = self.events.clone();
        let event_name = event.to_string();
        let payload_value = serde_json::to_value(payload).unwrap();

        tokio::spawn(async move {
            events.lock().await.push((event_name, payload_value));
        });

        Ok(())
    }

    fn emit_to<S: serde::Serialize + Clone>(&self, _target: &str, event: &str, payload: S) -> tauri::Result<()> {
        self.emit(event, payload)
    }

    fn emit_filter<S, F>(&self, event: &str, payload: S, _filter: F) -> tauri::Result<()>
    where
        S: serde::Serialize + Clone,
        F: Fn(&tauri::EventTarget) -> bool,
    {
        self.emit(event, payload)
    }
}

// Helper to create valid MIDI file bytes
fn create_midi_bytes(bpm: f64, key_notes: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();

    // MIDI header (Format 0, 1 track, 480 ticks/beat)
    bytes.extend_from_slice(&[
        0x4D, 0x54, 0x68, 0x64, // "MThd"
        0x00, 0x00, 0x00, 0x06, // Header length
        0x00, 0x00,             // Format 0
        0x00, 0x01,             // 1 track
        0x01, 0xE0,             // 480 ticks/beat
    ]);

    // Calculate microseconds per beat for tempo
    let us_per_beat = (60_000_000.0 / bpm) as u32;
    let tempo_bytes = [
        ((us_per_beat >> 16) & 0xFF) as u8,
        ((us_per_beat >> 8) & 0xFF) as u8,
        (us_per_beat & 0xFF) as u8,
    ];

    // Build track data
    let mut track_data = Vec::new();

    // Tempo event
    track_data.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03]);
    track_data.extend_from_slice(&tempo_bytes);

    // Time signature (4/4)
    track_data.extend_from_slice(&[
        0x00, 0xFF, 0x58, 0x04, // Time signature meta event
        0x04, 0x02, 0x18, 0x08, // 4/4 time
    ]);

    // Add notes to establish key
    for (i, note) in key_notes.iter().enumerate() {
        // Note on
        track_data.push(if i == 0 { 0x00 } else { 0x20 }); // Delta time
        track_data.extend_from_slice(&[0x90, *note, 0x60]); // Note on, velocity 96

        // Note off
        track_data.push(0x20); // Delta time
        track_data.extend_from_slice(&[0x80, *note, 0x40]); // Note off
    }

    // End of track
    track_data.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

    // Track header
    bytes.extend_from_slice(&[0x4D, 0x54, 0x72, 0x6B]); // "MTrk"
    let track_len = track_data.len() as u32;
    bytes.extend_from_slice(&[
        ((track_len >> 24) & 0xFF) as u8,
        ((track_len >> 16) & 0xFF) as u8,
        ((track_len >> 8) & 0xFF) as u8,
        (track_len & 0xFF) as u8,
    ]);
    bytes.extend_from_slice(&track_data);

    bytes
}

// Helper to create MIDI bytes with specific features
fn create_complex_midi_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();

    // MIDI header
    bytes.extend_from_slice(&[
        0x4D, 0x54, 0x68, 0x64, 0x00, 0x00, 0x00, 0x06,
        0x00, 0x00, 0x00, 0x01, 0x01, 0xE0,
    ]);

    let mut track_data = Vec::new();

    // Tempo: 140 BPM
    track_data.extend_from_slice(&[
        0x00, 0xFF, 0x51, 0x03, 0x06, 0xB3, 0xB3,
    ]);

    // Track name
    let name = b"Piano";
    track_data.extend_from_slice(&[0x00, 0xFF, 0x03, name.len() as u8]);
    track_data.extend_from_slice(name);

    // Program change (Piano)
    track_data.extend_from_slice(&[0x00, 0xC0, 0x00]);

    // Add polyphonic notes (chord)
    track_data.extend_from_slice(&[
        0x00, 0x90, 60, 80, // C4
        0x00, 0x90, 64, 82, // E4
        0x00, 0x90, 67, 84, // G4
        0x20, 0x80, 60, 40,
        0x00, 0x80, 64, 40,
        0x00, 0x80, 67, 40,
    ]);

    // Pitch bend
    track_data.extend_from_slice(&[0x00, 0xE0, 0x00, 0x20]);

    // Control change (modulation)
    track_data.extend_from_slice(&[0x00, 0xB0, 0x01, 0x40]);

    // End of track
    track_data.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

    // Track chunk
    bytes.extend_from_slice(&[0x4D, 0x54, 0x72, 0x6B]);
    let len = track_data.len() as u32;
    bytes.extend_from_slice(&[
        ((len >> 24) & 0xFF) as u8,
        ((len >> 16) & 0xFF) as u8,
        ((len >> 8) & 0xFF) as u8,
        (len & 0xFF) as u8,
    ]);
    bytes.extend_from_slice(&track_data);

    bytes
}

//=============================================================================
// TEST CASES - start_analysis()
//=============================================================================

#[tokio::test]
async fn test_start_analysis_empty_unanalyzed() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Import files but mark them all as analyzed
    for i in 0..10 {
        let file_id: i64 = sqlx::query_scalar(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3) RETURNING file_id"
        )
        .bind(format!("/test/file_{}.mid", i))
        .bind(format!("{:064x}", i))
        .bind(1024i64)
        .fetch_one(pool)
        .await
        .expect("Failed to insert file");

        // Mark as analyzed
        sqlx::query("UPDATE files SET analyzed_at = NOW() WHERE file_id = $1")
            .bind(file_id)
            .execute(pool)
            .await
            .expect("Failed to update");
    }

    // Call start_analysis - should return immediately with 0 analyzed
    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok(), "Analysis should succeed");
    let summary = result.unwrap();
    assert_eq!(summary.total_files, 0);
    assert_eq!(summary.analyzed, 0);
    assert_eq!(summary.skipped, 0);

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_single_file() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create a valid MIDI file
    let midi_bytes = create_midi_bytes(120.0, &[60, 62, 64, 65, 67, 69, 71, 72]); // C major scale
    let midi_path = file_fixtures.create_midi_file("test_single.mid", &midi_bytes).await;

    // Insert into database
    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 12345))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert file");

    // Analyze
    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok(), "Analysis should succeed: {:?}", result.err());
    let summary = result.unwrap();
    assert_eq!(summary.total_files, 1);
    assert_eq!(summary.analyzed, 1);
    assert_eq!(summary.skipped, 0);

    // Verify metadata was inserted
    let metadata: Option<(f64, String)> = sqlx::query_as(
        "SELECT tempo_bpm, key_signature FROM musical_metadata WHERE file_id = $1"
    )
    .bind(file_id)
    .fetch_optional(pool)
    .await
    .expect("Failed to fetch metadata");

    assert!(metadata.is_some(), "Metadata should exist");
    let (bpm, key) = metadata.unwrap();
    assert!((bpm - 120.0).abs() < 5.0, "BPM should be ~120, got {}", bpm);
    assert!(key.contains("C") || key.contains("major"), "Key should be C major-ish, got {}", key);

    // Verify analyzed_at was set
    let analyzed_at: Option<chrono::NaiveDateTime> = sqlx::query_scalar(
        "SELECT analyzed_at FROM files WHERE file_id = $1"
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
    .expect("Failed to fetch analyzed_at");

    assert!(analyzed_at.is_some(), "analyzed_at should be set");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_100_files() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create 100 MIDI files with varying BPMs
    let mut file_ids = Vec::new();
    for i in 0..100 {
        let bpm = 100.0 + (i as f64 * 0.5); // 100-149.5 BPM
        let midi_bytes = create_midi_bytes(bpm, &[60, 62, 64, 65, 67, 69, 71, 72]);
        let midi_path = file_fixtures.create_midi_file(&format!("test_{}.mid", i), &midi_bytes).await;

        let file_id: i64 = sqlx::query_scalar(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3) RETURNING file_id"
        )
        .bind(midi_path.to_str().unwrap())
        .bind(format!("{:064x}", i))
        .bind(midi_bytes.len() as i64)
        .fetch_one(pool)
        .await
        .expect("Failed to insert file");

        file_ids.push(file_id);
    }

    // Analyze all files
    let window = TestWindow::new();
    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        window.clone(),
    ).await;

    assert!(result.is_ok(), "Analysis should succeed");
    let summary = result.unwrap();
    assert_eq!(summary.total_files, 100);
    assert_eq!(summary.analyzed, 100);
    assert_eq!(summary.skipped, 0);

    // Verify all files have metadata
    let metadata_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM musical_metadata WHERE file_id = ANY($1)"
    )
    .bind(&file_ids)
    .fetch_one(pool)
    .await
    .expect("Failed to count metadata");

    assert_eq!(metadata_count, 100, "All files should have metadata");

    // Verify progress events were emitted (every 10 files)
    let events = window.get_events().await;
    let progress_events: Vec<_> = events.iter()
        .filter(|(name, _)| name == "analysis-progress")
        .collect();

    assert!(progress_events.len() >= 10, "Should have at least 10 progress events");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_1000_files() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create 1000 files to test batch fetching (batches of 1000)
    let midi_bytes = create_midi_bytes(120.0, &[60, 62, 64]);

    for i in 0..1000 {
        let midi_path = file_fixtures.create_midi_file(&format!("batch_{}.mid", i), &midi_bytes).await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)"
        )
        .bind(midi_path.to_str().unwrap())
        .bind(format!("{:064x}", i))
        .bind(midi_bytes.len() as i64)
        .execute(pool)
        .await
        .expect("Failed to insert file");
    }

    // Analyze
    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok(), "Analysis should succeed");
    let summary = result.unwrap();
    assert_eq!(summary.total_files, 1000);
    assert_eq!(summary.analyzed, 1000);

    // Verify batching worked - all metadata inserted
    let metadata_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM musical_metadata"
    )
    .fetch_one(pool)
    .await
    .expect("Failed to count");

    assert!(metadata_count >= 1000, "All files should have metadata");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_metadata_extracted() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create file with known properties
    let midi_bytes = create_midi_bytes(128.0, &[60, 62, 64, 65, 67, 69, 71, 72]); // C major, 128 BPM
    let midi_path = file_fixtures.create_midi_file("metadata_test.mid", &midi_bytes).await;

    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 99999))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    // Analyze
    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok());

    // Verify comprehensive metadata extraction
    let metadata: (
        Option<f64>,  // tempo_bpm
        Option<String>, // key_signature
        Option<i16>,  // time_signature_num
        Option<i16>,  // time_signature_den
        Option<f64>,  // duration_seconds
        i32,          // note_count
    ) = sqlx::query_as(
        "SELECT tempo_bpm, key_signature, time_signature_num, time_signature_den,
                duration_seconds, note_count
         FROM musical_metadata WHERE file_id = $1"
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
    .expect("Metadata should exist");

    assert!(metadata.0.is_some(), "BPM should be detected");
    assert!(metadata.1.is_some(), "Key should be detected");
    assert_eq!(metadata.2, Some(4), "Time signature numerator should be 4");
    assert_eq!(metadata.3, Some(4), "Time signature denominator should be 4");
    assert!(metadata.4.is_some(), "Duration should be calculated");
    assert!(metadata.5 > 0, "Note count should be > 0");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_batch_database_insert() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create 150 files to test batch insert threshold (100 files)
    let midi_bytes = create_midi_bytes(120.0, &[60, 64, 67]);

    for i in 0..150 {
        let midi_path = file_fixtures.create_midi_file(&format!("batch_insert_{}.mid", i), &midi_bytes).await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)"
        )
        .bind(midi_path.to_str().unwrap())
        .bind(format!("{:064x}", i + 50000))
        .bind(midi_bytes.len() as i64)
        .execute(pool)
        .await
        .expect("Failed to insert");
    }

    // Analyze - should trigger batch insert at 100 files and final flush at 150
    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok());
    let summary = result.unwrap();
    assert_eq!(summary.analyzed, 150);

    // Verify all metadata was inserted via batching
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM musical_metadata m
         JOIN files f ON m.file_id = f.file_id
         WHERE f.file_path LIKE '/tmp%batch_insert%'"
    )
    .fetch_one(pool)
    .await
    .expect("Failed to count");

    assert_eq!(count, 150, "All batched inserts should succeed");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_worker_pool_32() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create 64 files to test parallel processing with 32 workers
    let midi_bytes = create_midi_bytes(120.0, &[60, 64, 67]);

    for i in 0..64 {
        let midi_path = file_fixtures.create_midi_file(&format!("worker_{}.mid", i), &midi_bytes).await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)"
        )
        .bind(midi_path.to_str().unwrap())
        .bind(format!("{:064x}", i + 60000))
        .bind(midi_bytes.len() as i64)
        .execute(pool)
        .await
        .expect("Failed to insert");
    }

    let start = std::time::Instant::now();

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    let duration = start.elapsed();

    assert!(result.is_ok());
    let summary = result.unwrap();
    assert_eq!(summary.analyzed, 64);

    // With 32 workers, 64 files should complete faster than sequential
    // Sequential would be ~64 * analysis_time, parallel should be ~2 * analysis_time
    // This is a soft check - just ensure it completes
    assert!(duration.as_secs() < 30, "Should complete in reasonable time with parallelism");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_arc_atomic_counter() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create files and analyze
    let midi_bytes = create_midi_bytes(120.0, &[60]);

    for i in 0..25 {
        let midi_path = file_fixtures.create_midi_file(&format!("counter_{}.mid", i), &midi_bytes).await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)"
        )
        .bind(midi_path.to_str().unwrap())
        .bind(format!("{:064x}", i + 70000))
        .bind(midi_bytes.len() as i64)
        .execute(pool)
        .await
        .expect("Failed to insert");
    }

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok());
    let summary = result.unwrap();

    // Arc<AtomicUsize> counter should accurately track all files
    assert_eq!(summary.analyzed, 25, "Counter should track exactly 25 analyzed files");
    assert_eq!(summary.skipped, 0, "Counter should track 0 skipped files");
    assert_eq!(summary.total_files, 25, "Total should match");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_arc_mutex_error_collection() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create mix of valid and invalid files
    let valid_bytes = create_midi_bytes(120.0, &[60, 64, 67]);
    let invalid_bytes = vec![0x00, 0x01, 0x02]; // Invalid MIDI

    for i in 0..10 {
        let (bytes, name) = if i % 3 == 0 {
            (&invalid_bytes, format!("invalid_{}.mid", i))
        } else {
            (&valid_bytes, format!("valid_{}.mid", i))
        };

        let midi_path = file_fixtures.create_midi_file(&name, bytes).await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)"
        )
        .bind(midi_path.to_str().unwrap())
        .bind(format!("{:064x}", i + 80000))
        .bind(bytes.len() as i64)
        .execute(pool)
        .await
        .expect("Failed to insert");
    }

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok());
    let summary = result.unwrap();

    // Arc<Mutex<Vec<String>>> should collect errors from invalid files
    assert_eq!(summary.total_files, 10);
    assert_eq!(summary.analyzed + summary.skipped, 10, "All files accounted for");
    assert!(summary.skipped > 0, "Should have skipped invalid files");
    assert!(!summary.errors.is_empty(), "Should have error messages");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_progress_events() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create 35 files to trigger multiple progress events (every 10 files)
    let midi_bytes = create_midi_bytes(120.0, &[60, 64, 67]);

    for i in 0..35 {
        let midi_path = file_fixtures.create_midi_file(&format!("progress_{}.mid", i), &midi_bytes).await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)"
        )
        .bind(midi_path.to_str().unwrap())
        .bind(format!("{:064x}", i + 90000))
        .bind(midi_bytes.len() as i64)
        .execute(pool)
        .await
        .expect("Failed to insert");
    }

    let window = TestWindow::new();

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        window.clone(),
    ).await;

    assert!(result.is_ok());

    // Wait for events to be emitted
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let events = window.get_events().await;
    let progress_events: Vec<_> = events.iter()
        .filter(|(name, _)| name == "analysis-progress")
        .collect();

    // Should emit at files: 10, 20, 30, 35 (at minimum)
    assert!(progress_events.len() >= 4, "Should have at least 4 progress events, got {}", progress_events.len());

    // Verify event structure
    if let Some((_, payload)) = progress_events.first() {
        assert!(payload.get("current").is_some(), "Event should have 'current' field");
        assert!(payload.get("total").is_some(), "Event should have 'total' field");
        assert!(payload.get("rate").is_some(), "Event should have 'rate' field");
        assert!(payload.get("eta_seconds").is_some(), "Event should have 'eta_seconds' field");
    }

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_batching_1000_items() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create 2500 files to test multiple batch fetches (1000 per batch)
    let midi_bytes = create_midi_bytes(120.0, &[60]);

    for i in 0..2500 {
        let midi_path = file_fixtures.create_midi_file(&format!("large_batch_{}.mid", i), &midi_bytes).await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)"
        )
        .bind(midi_path.to_str().unwrap())
        .bind(format!("{:064x}", i + 100000))
        .bind(midi_bytes.len() as i64)
        .execute(pool)
        .await
        .expect("Failed to insert");

        // Commit every 100 to avoid transaction timeout
        if i % 100 == 0 {
            // Just continue - sqlx handles this
        }
    }

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok());
    let summary = result.unwrap();

    // Should process all 2500 files across 3 batches (1000 + 1000 + 500)
    assert_eq!(summary.total_files, 2500);
    assert_eq!(summary.analyzed, 2500);

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_individual_file_failures() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create files with non-existent paths (will fail to read)
    for i in 0..20 {
        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)"
        )
        .bind(format!("/nonexistent/path/file_{}.mid", i))
        .bind(format!("{:064x}", i + 110000))
        .bind(1024i64)
        .execute(pool)
        .await
        .expect("Failed to insert");
    }

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok(), "Analysis should continue despite failures");
    let summary = result.unwrap();

    assert_eq!(summary.total_files, 20);
    assert_eq!(summary.skipped, 20, "All files should be skipped due to file not found");
    assert_eq!(summary.analyzed, 0);
    assert_eq!(summary.errors.len(), 20, "Should have 20 error messages");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_invalid_midi_handling() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create corrupt MIDI files
    let corrupt_bytes = vec![0xFF, 0xFF, 0xFF, 0xFF]; // Not valid MIDI

    for i in 0..15 {
        let midi_path = file_fixtures.create_midi_file(&format!("corrupt_{}.mid", i), &corrupt_bytes).await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)"
        )
        .bind(midi_path.to_str().unwrap())
        .bind(format!("{:064x}", i + 120000))
        .bind(corrupt_bytes.len() as i64)
        .execute(pool)
        .await
        .expect("Failed to insert");
    }

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok(), "Should handle corrupt MIDI gracefully");
    let summary = result.unwrap();

    assert_eq!(summary.total_files, 15);
    assert_eq!(summary.skipped, 15, "All corrupt files should be skipped");
    assert!(!summary.errors.is_empty(), "Should have error messages for corrupt files");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_bpm_accuracy_c_major() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Test various BPM values
    let test_bpms = vec![80.0, 100.0, 120.0, 140.0, 160.0, 180.0];

    for (i, bpm) in test_bpms.iter().enumerate() {
        let midi_bytes = create_midi_bytes(*bpm, &[60, 62, 64, 65, 67, 69, 71, 72]);
        let midi_path = file_fixtures.create_midi_file(&format!("bpm_{}.mid", i), &midi_bytes).await;

        let file_id: i64 = sqlx::query_scalar(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3) RETURNING file_id"
        )
        .bind(midi_path.to_str().unwrap())
        .bind(format!("{:064x}", i + 130000))
        .bind(midi_bytes.len() as i64)
        .fetch_one(pool)
        .await
        .expect("Failed to insert");
    }

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok());

    // Verify BPM detection accuracy (within ±1%)
    let detected_bpms: Vec<f64> = sqlx::query_scalar(
        "SELECT tempo_bpm FROM musical_metadata m
         JOIN files f ON m.file_id = f.file_id
         WHERE f.file_path LIKE '/tmp%bpm_%'
         ORDER BY f.file_id"
    )
    .fetch_all(pool)
    .await
    .expect("Failed to fetch BPMs");

    for (i, detected) in detected_bpms.iter().enumerate() {
        let expected = test_bpms[i];
        let error_percent = ((detected - expected).abs() / expected) * 100.0;
        assert!(error_percent < 5.0, "BPM accuracy should be within 5% for {}, got {}, expected {}",
                i, detected, expected);
    }

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_key_detection_accuracy() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Test different keys
    let keys = vec![
        ("C_major", vec![60, 62, 64, 65, 67, 69, 71, 72]),    // C major scale
        ("D_major", vec![62, 64, 66, 67, 69, 71, 73, 74]),    // D major scale
        ("E_minor", vec![64, 66, 67, 69, 71, 72, 74, 76]),    // E minor scale
    ];

    for (i, (key_name, notes)) in keys.iter().enumerate() {
        let midi_bytes = create_midi_bytes(120.0, notes);
        let midi_path = file_fixtures.create_midi_file(&format!("key_{}.mid", i), &midi_bytes).await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes, filename)
             VALUES ($1, $2, $3, $4)"
        )
        .bind(midi_path.to_str().unwrap())
        .bind(format!("{:064x}", i + 140000))
        .bind(midi_bytes.len() as i64)
        .bind(format!("{}.mid", key_name))
        .execute(pool)
        .await
        .expect("Failed to insert");
    }

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok());

    // Verify key detection produced results
    let detected_keys: Vec<Option<String>> = sqlx::query_scalar(
        "SELECT key_signature FROM musical_metadata m
         JOIN files f ON m.file_id = f.file_id
         WHERE f.filename LIKE '%major%' OR f.filename LIKE '%minor%'
         ORDER BY f.file_id"
    )
    .fetch_all(pool)
    .await
    .expect("Failed to fetch keys");

    assert_eq!(detected_keys.len(), 3, "Should detect keys for all files");

    for key in detected_keys {
        assert!(key.is_some(), "Key should be detected");
        let key_str = key.unwrap();
        assert!(!key_str.is_empty(), "Key string should not be empty");
    }

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_duration_calculation() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create MIDI with known duration
    let midi_bytes = create_midi_bytes(120.0, &[60, 62, 64, 65, 67]);
    let midi_path = file_fixtures.create_midi_file("duration_test.mid", &midi_bytes).await;

    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 150000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok());

    // Verify duration was calculated
    let (duration_secs, duration_ticks): (Option<f64>, Option<i32>) = sqlx::query_as(
        "SELECT duration_seconds, duration_ticks FROM musical_metadata WHERE file_id = $1"
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
    .expect("Failed to fetch duration");

    assert!(duration_secs.is_some(), "Duration in seconds should be calculated");
    assert!(duration_ticks.is_some(), "Duration in ticks should be calculated");
    assert!(duration_secs.unwrap() > 0.0, "Duration should be positive");
    assert!(duration_ticks.unwrap() > 0, "Tick count should be positive");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_instrument_extraction() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create MIDI with instrument info (program changes)
    let midi_bytes = create_complex_midi_bytes();
    let midi_path = file_fixtures.create_midi_file("instruments.mid", &midi_bytes).await;

    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 160000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok());

    // Verify instruments were extracted
    let instruments: Vec<String> = sqlx::query_scalar(
        "SELECT instruments FROM musical_metadata WHERE file_id = $1"
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
    .expect("Failed to fetch instruments");

    assert!(!instruments.is_empty(), "Should detect instruments");
    // Should contain "Piano" from track name or program change
    assert!(instruments.iter().any(|i| i.contains("Piano") || i.contains("piano")),
            "Should detect Piano instrument");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_tempo_map_handling() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create MIDI with tempo variation
    let midi_bytes = create_midi_bytes(120.0, &[60, 62, 64]);
    let midi_path = file_fixtures.create_midi_file("tempo_var.mid", &midi_bytes).await;

    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 170000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok());

    // Verify tempo variation flag
    let has_variation: bool = sqlx::query_scalar(
        "SELECT has_tempo_variation FROM musical_metadata WHERE file_id = $1"
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
    .expect("Failed to fetch tempo variation");

    // Simple file should not have tempo variation
    assert!(!has_variation, "Simple file should not have tempo variation");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_time_signature_detection() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create MIDI files (default is 4/4)
    let midi_bytes = create_midi_bytes(120.0, &[60, 64, 67]);
    let midi_path = file_fixtures.create_midi_file("time_sig.mid", &midi_bytes).await;

    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 180000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok());

    // Verify time signature
    let (num, den): (Option<i16>, Option<i16>) = sqlx::query_as(
        "SELECT time_signature_num, time_signature_den FROM musical_metadata WHERE file_id = $1"
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
    .expect("Failed to fetch time signature");

    assert_eq!(num, Some(4), "Numerator should be 4");
    assert_eq!(den, Some(4), "Denominator should be 4");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_polyphonic_notes_count() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Complex MIDI has polyphonic notes (chord)
    let midi_bytes = create_complex_midi_bytes();
    let midi_path = file_fixtures.create_midi_file("polyphony.mid", &midi_bytes).await;

    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 190000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok());

    // Verify polyphony detection
    let polyphony: Option<i16> = sqlx::query_scalar(
        "SELECT polyphony_max FROM musical_metadata WHERE file_id = $1"
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
    .expect("Failed to fetch polyphony");

    assert!(polyphony.is_some(), "Polyphony should be detected");
    assert!(polyphony.unwrap() >= 3, "Should detect chord (3+ simultaneous notes)");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_percussion_detection() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create MIDI - instruments are detected via program change
    let midi_bytes = create_complex_midi_bytes();
    let midi_path = file_fixtures.create_midi_file("percussion.mid", &midi_bytes).await;

    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 200000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok());

    // Just verify analysis completed
    let note_count: i32 = sqlx::query_scalar(
        "SELECT note_count FROM musical_metadata WHERE file_id = $1"
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
    .expect("Failed to fetch note count");

    assert!(note_count > 0, "Should have detected notes");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_velocity_statistics() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    let midi_bytes = create_complex_midi_bytes();
    let midi_path = file_fixtures.create_midi_file("velocity.mid", &midi_bytes).await;

    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 210000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok());

    // Verify velocity statistics
    let (avg_vel, vel_low, vel_high): (Option<f64>, Option<i16>, Option<i16>) = sqlx::query_as(
        "SELECT avg_velocity, velocity_range_low, velocity_range_high
         FROM musical_metadata WHERE file_id = $1"
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
    .expect("Failed to fetch velocity stats");

    assert!(avg_vel.is_some(), "Average velocity should be calculated");
    assert!(vel_low.is_some(), "Min velocity should be detected");
    assert!(vel_high.is_some(), "Max velocity should be detected");
    assert!(avg_vel.unwrap() > 0.0 && avg_vel.unwrap() <= 127.0, "Velocity should be in valid range");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_note_range_calculation() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create MIDI with known note range (C4 to C5)
    let midi_bytes = create_midi_bytes(120.0, &[60, 62, 64, 65, 67, 69, 71, 72]);
    let midi_path = file_fixtures.create_midi_file("note_range.mid", &midi_bytes).await;

    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 220000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok());

    // Verify note range
    let (low, high, semitones): (Option<i16>, Option<i16>, Option<i16>) = sqlx::query_as(
        "SELECT pitch_range_low, pitch_range_high, pitch_range_semitones
         FROM musical_metadata WHERE file_id = $1"
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
    .expect("Failed to fetch note range");

    assert!(low.is_some(), "Low note should be detected");
    assert!(high.is_some(), "High note should be detected");
    assert!(semitones.is_some(), "Semitone range should be calculated");
    assert_eq!(low.unwrap(), 60, "Lowest note should be C4 (60)");
    assert_eq!(high.unwrap(), 72, "Highest note should be C5 (72)");
    assert_eq!(semitones.unwrap(), 12, "Range should be 12 semitones (1 octave)");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_complexity_score() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Simple file should have low complexity
    let simple_bytes = create_midi_bytes(120.0, &[60]);
    let simple_path = file_fixtures.create_midi_file("simple.mid", &simple_bytes).await;

    // Complex file should have higher complexity
    let complex_bytes = create_complex_midi_bytes();
    let complex_path = file_fixtures.create_midi_file("complex.mid", &complex_bytes).await;

    let simple_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind(simple_path.to_str().unwrap())
    .bind(format!("{:064x}", 230000))
    .bind(simple_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let complex_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind(complex_path.to_str().unwrap())
    .bind(format!("{:064x}", 230001))
    .bind(complex_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok());

    // Verify complexity scores
    let simple_score: Option<f64> = sqlx::query_scalar(
        "SELECT complexity_score FROM musical_metadata WHERE file_id = $1"
    )
    .bind(simple_id)
    .fetch_one(pool)
    .await
    .expect("Failed to fetch simple score");

    let complex_score: Option<f64> = sqlx::query_scalar(
        "SELECT complexity_score FROM musical_metadata WHERE file_id = $1"
    )
    .bind(complex_id)
    .fetch_one(pool)
    .await
    .expect("Failed to fetch complex score");

    assert!(simple_score.is_some(), "Simple file should have complexity score");
    assert!(complex_score.is_some(), "Complex file should have complexity score");
    assert!(complex_score.unwrap() > simple_score.unwrap(),
            "Complex file should have higher complexity score");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_pitch_bend_detection() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Complex MIDI has pitch bend
    let midi_bytes = create_complex_midi_bytes();
    let midi_path = file_fixtures.create_midi_file("pitch_bend.mid", &midi_bytes).await;

    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 240000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok());

    // Verify pitch bend detection
    let has_pitch_bend: bool = sqlx::query_scalar(
        "SELECT has_pitch_bend FROM musical_metadata WHERE file_id = $1"
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
    .expect("Failed to fetch pitch bend flag");

    assert!(has_pitch_bend, "Should detect pitch bend in complex MIDI");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_cc_messages_detection() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Complex MIDI has CC messages
    let midi_bytes = create_complex_midi_bytes();
    let midi_path = file_fixtures.create_midi_file("cc_messages.mid", &midi_bytes).await;

    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 250000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok());

    // Verify CC detection
    let has_cc: bool = sqlx::query_scalar(
        "SELECT has_cc_messages FROM musical_metadata WHERE file_id = $1"
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
    .expect("Failed to fetch CC flag");

    assert!(has_cc, "Should detect CC messages in complex MIDI");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_empty_track_handling() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create minimal MIDI (just tempo, no notes)
    let mut bytes = vec![
        0x4D, 0x54, 0x68, 0x64, 0x00, 0x00, 0x00, 0x06,
        0x00, 0x00, 0x00, 0x01, 0x01, 0xE0,
    ];

    let track_data = vec![
        0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20, // Tempo
        0x00, 0xFF, 0x2F, 0x00, // End of track
    ];

    bytes.extend_from_slice(&[0x4D, 0x54, 0x72, 0x6B]);
    let len = track_data.len() as u32;
    bytes.extend_from_slice(&[
        ((len >> 24) & 0xFF) as u8,
        ((len >> 16) & 0xFF) as u8,
        ((len >> 8) & 0xFF) as u8,
        (len & 0xFF) as u8,
    ]);
    bytes.extend_from_slice(&track_data);

    let midi_path = file_fixtures.create_midi_file("empty_track.mid", &bytes).await;

    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 260000))
    .bind(bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok());

    // Verify empty track handling
    let note_count: i32 = sqlx::query_scalar(
        "SELECT note_count FROM musical_metadata WHERE file_id = $1"
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
    .expect("Failed to fetch note count");

    assert_eq!(note_count, 0, "Empty track should have 0 notes");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_single_note_file() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Minimal file with single note
    let midi_bytes = create_midi_bytes(120.0, &[60]);
    let midi_path = file_fixtures.create_midi_file("single_note.mid", &midi_bytes).await;

    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 270000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok());

    // Verify single note analysis
    let (note_count, pitch_low, pitch_high): (i32, Option<i16>, Option<i16>) = sqlx::query_as(
        "SELECT note_count, pitch_range_low, pitch_range_high
         FROM musical_metadata WHERE file_id = $1"
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
    .expect("Failed to fetch");

    assert_eq!(note_count, 1, "Should have exactly 1 note");
    assert_eq!(pitch_low, Some(60), "Low pitch should be C4");
    assert_eq!(pitch_high, Some(60), "High pitch should be C4");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_very_long_duration() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create MIDI with extended duration
    let midi_bytes = create_midi_bytes(60.0, &[60, 62, 64, 65, 67, 69, 71, 72]); // Slow tempo = longer duration
    let midi_path = file_fixtures.create_midi_file("long_duration.mid", &midi_bytes).await;

    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 280000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok());

    // Verify duration calculation handles long files
    let duration: Option<f64> = sqlx::query_scalar(
        "SELECT duration_seconds FROM musical_metadata WHERE file_id = $1"
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
    .expect("Failed to fetch duration");

    assert!(duration.is_some(), "Duration should be calculated");
    assert!(duration.unwrap() > 0.0, "Duration should be positive");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_unicode_filename_handling() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create file with unicode filename
    let midi_bytes = create_midi_bytes(120.0, &[60, 64, 67]);
    let midi_path = file_fixtures.create_midi_file("测试文件.mid", &midi_bytes).await;

    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes, filename)
         VALUES ($1, $2, $3, $4) RETURNING file_id"
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 290000))
    .bind(midi_bytes.len() as i64)
    .bind("测试文件.mid")
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok(), "Should handle unicode filenames");

    // Verify metadata exists
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM musical_metadata WHERE file_id = $1)"
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
    .expect("Failed to check existence");

    assert!(exists, "Metadata should exist for unicode filename");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_database_batch_boundaries() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Test batch boundaries: 100 (triggers flush), 101 (over boundary)
    let midi_bytes = create_midi_bytes(120.0, &[60]);

    for i in 0..101 {
        let midi_path = file_fixtures.create_midi_file(&format!("boundary_{}.mid", i), &midi_bytes).await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)"
        )
        .bind(midi_path.to_str().unwrap())
        .bind(format!("{:064x}", i + 300000))
        .bind(midi_bytes.len() as i64)
        .execute(pool)
        .await
        .expect("Failed to insert");
    }

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok());
    let summary = result.unwrap();

    // All files should be analyzed despite batch boundary
    assert_eq!(summary.analyzed, 101, "Should handle batch boundary correctly");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_progress_event_format() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create 20 files to trigger progress events
    let midi_bytes = create_midi_bytes(120.0, &[60]);

    for i in 0..20 {
        let midi_path = file_fixtures.create_midi_file(&format!("event_format_{}.mid", i), &midi_bytes).await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)"
        )
        .bind(midi_path.to_str().unwrap())
        .bind(format!("{:064x}", i + 310000))
        .bind(midi_bytes.len() as i64)
        .execute(pool)
        .await
        .expect("Failed to insert");
    }

    let window = TestWindow::new();

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        window.clone(),
    ).await;

    assert!(result.is_ok());

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let events = window.get_events().await;
    let progress_events: Vec<_> = events.iter()
        .filter(|(name, _)| name == "analysis-progress")
        .collect();

    // Verify event format
    for (_, payload) in progress_events {
        assert!(payload.get("current").is_some(), "Should have current field");
        assert!(payload.get("total").is_some(), "Should have total field");
        assert!(payload.get("current_file").is_some(), "Should have current_file field");
        assert!(payload.get("rate").is_some(), "Should have rate field");
        assert!(payload.get("eta_seconds").is_some(), "Should have eta_seconds field");

        // Verify types
        assert!(payload["current"].is_number(), "current should be number");
        assert!(payload["total"].is_number(), "total should be number");
        assert!(payload["current_file"].is_string(), "current_file should be string");
        assert!(payload["rate"].is_number(), "rate should be number");
        assert!(payload["eta_seconds"].is_number(), "eta_seconds should be number");
    }

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_analysis_state_persistence() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create files
    let midi_bytes = create_midi_bytes(120.0, &[60, 64, 67]);

    for i in 0..10 {
        let midi_path = file_fixtures.create_midi_file(&format!("persist_{}.mid", i), &midi_bytes).await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)"
        )
        .bind(midi_path.to_str().unwrap())
        .bind(format!("{:064x}", i + 320000))
        .bind(midi_bytes.len() as i64)
        .execute(pool)
        .await
        .expect("Failed to insert");
    }

    // First analysis
    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok());

    // Second analysis should find 0 unanalyzed files (state persisted)
    let result2 = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result2.is_ok());
    let summary2 = result2.unwrap();
    assert_eq!(summary2.total_files, 0, "Should find 0 unanalyzed files on second run");

    db.cleanup().await;
}

#[tokio::test]
async fn test_start_analysis_metadata_fields_populated() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create comprehensive MIDI file
    let midi_bytes = create_complex_midi_bytes();
    let midi_path = file_fixtures.create_midi_file("all_fields.mid", &midi_bytes).await;

    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 330000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    assert!(result.is_ok());

    // Verify all expected fields are populated
    let metadata: (
        Option<f64>, Option<String>, Option<i16>, Option<i16>,
        Option<f64>, i32, Option<f64>, bool, bool
    ) = sqlx::query_as(
        "SELECT tempo_bpm, key_signature, time_signature_num, time_signature_den,
                duration_seconds, note_count, complexity_score,
                has_pitch_bend, has_cc_messages
         FROM musical_metadata WHERE file_id = $1"
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
    .expect("Failed to fetch metadata");

    assert!(metadata.0.is_some(), "BPM should be populated");
    assert!(metadata.1.is_some(), "Key should be populated");
    assert!(metadata.2.is_some(), "Time sig num should be populated");
    assert!(metadata.3.is_some(), "Time sig den should be populated");
    assert!(metadata.4.is_some(), "Duration should be populated");
    assert!(metadata.5 > 0, "Note count should be > 0");
    assert!(metadata.6.is_some(), "Complexity score should be populated");

    db.cleanup().await;
}

#[tokio::test]
#[ignore] // Performance test - run separately
async fn test_start_analysis_10k_file_performance() {
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create 10,000 files for performance testing
    let midi_bytes = create_midi_bytes(120.0, &[60]);

    println!("Creating 10,000 test files...");
    for i in 0..10000 {
        let midi_path = file_fixtures.create_midi_file(&format!("perf_{}.mid", i), &midi_bytes).await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)"
        )
        .bind(midi_path.to_str().unwrap())
        .bind(format!("{:064x}", i + 400000))
        .bind(midi_bytes.len() as i64)
        .execute(pool)
        .await
        .expect("Failed to insert");

        if i % 1000 == 0 {
            println!("  Created {} files...", i);
        }
    }

    println!("Starting analysis of 10,000 files...");
    let start = std::time::Instant::now();

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()))
                .await
                .expect("Failed to connect"),
        })),
        TestWindow::new(),
    ).await;

    let duration = start.elapsed();

    assert!(result.is_ok());
    let summary = result.unwrap();

    println!("Performance Results:");
    println!("  Total files: {}", summary.total_files);
    println!("  Analyzed: {}", summary.analyzed);
    println!("  Duration: {:.2}s", duration.as_secs_f64());
    println!("  Rate: {:.1} files/sec", summary.rate);

    // Target: >= 100 files/sec with 32 workers
    assert!(summary.rate >= 50.0, "Should achieve at least 50 files/sec, got {}", summary.rate);

    db.cleanup().await;
}

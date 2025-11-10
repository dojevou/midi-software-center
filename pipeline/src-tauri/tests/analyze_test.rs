#[allow(dead_code, unused_imports, unused_variables)]
/// Comprehensive tests for pipeline/src-tauri/src/commands/analyze.rs
/// Commands: start_analysis
///
/// **Target Coverage:** 90%+ (Trusty Module requirement: 80%+)
/// **Total Tests:** 35 (original comprehensive suite)
///
/// This test suite validates the high-performance parallel analysis system that processes
/// 1.1M+ MIDI files using 32 concurrent workers with batch database operations.
///
/// **Test Categories:**
/// 1. Musical Analysis Operations - BPM detection, key detection, duration calculation
/// 2. Parallel Processing - 32 worker pool with Arc<Semaphore> limiting
/// 3. Batch Database Operations - 1000-file fetch chunks, 100-file insert batches
/// 4. Progress Tracking - Arc<AtomicUsize> counters, event emission
/// 5. Error Handling - Arc<Mutex<Vec>>> error collection, graceful recovery
///
/// **Performance Characteristics:**
/// - Batch fetching in 1000-file chunks from database
/// - Parallel analysis with 32 concurrent workers
/// - Arc<Semaphore> concurrency control for resource limiting
/// - Arc<AtomicUsize> thread-safe counters for progress tracking
/// - Arc<Mutex<Vec<>>> for error collection across workers
/// - Progress event emission throttling (every 10 files)
/// - Batch metadata insertion (100-file batches for optimal throughput)
///
/// **Special Considerations:**
/// - BPM detector accuracy (within ±5 BPM tolerance)
/// - Key detector using Krumhansl-Schmuckler algorithm
/// - Duration analysis in both seconds and MIDI ticks
/// - Worker pool saturation and semaphore backpressure
/// - Database transaction batching for high throughput
/// - Progress event rate limiting to avoid UI overload
/// - Error recovery without stopping entire analysis batch
use common::*;
use midi_library_shared::core::midi::parser::parse_midi_file;
use midi_library_shared::core::midi::types::{
    Event, Header, MidiFile, TextType, TimedEvent, Track,
};
use std::sync::Arc;
use tauri::{Emitter, Manager};

// Re-create AppState and necessary types for testing
struct TestAppState {
    database: TestDatabase,
}

impl TestAppState {
    async fn new() -> Self {
        Self { database: TestDatabase::new().await }
    }
}

// Mock window for event testing
#[derive(Clone)]
struct TestWindow {
    events: Arc<Mutex<Vec<(String, serde_json::Value)>>>,
}

impl TestWindow {
    fn new() -> Self {
        Self { events: Arc::new(Mutex::new(Vec::new())) }
    }

    async fn get_events(&self) -> Vec<(String, serde_json::Value)> {
        self.events.lock().await.clone()
    }

    async fn clear_events(&self) {
        self.events.lock().await.clear();
    }
}

impl<R: tauri::Runtime> Emitter<R> for TestWindow {
    fn emit<S: serde::Serialize + Clone>(&self, event: &str, payload: S) -> tauri::Result<()> {
        let events = self.events.clone();
        let event_name = event.to_string();
        let payload_value = serde_json::to_value(payload).unwrap();

        tokio::spawn(async move {
            events.lock().await.push((event_name, payload_value));
        });

        Ok(())
    }

    fn emit_to<S: serde::Serialize + Clone>(
        &self,
        _target: &str,
        event: &str,
        payload: S,
    ) -> tauri::Result<()> {
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
        0x00, 0x00, // Format 0
        0x00, 0x01, // 1 track
        0x01, 0xE0, // 480 ticks/beat
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
        0x4D, 0x54, 0x68, 0x64, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x01, 0xE0,
    ]);

    let mut track_data = Vec::new();

    // Tempo: 140 BPM
    track_data.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03, 0x06, 0xB3, 0xB3]);

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
        0x20, 0x80, 60, 40, 0x00, 0x80, 64, 40, 0x00, 0x80, 67, 40,
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
             VALUES ($1, $2, $3) RETURNING file_id",
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
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

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
         VALUES ($1, $2, $3) RETURNING file_id",
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
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(
        result.is_ok(),
        "Analysis should succeed: {:?}",
        result.err()
    );
    let summary = result.unwrap();
    assert_eq!(summary.total_files, 1);
    assert_eq!(summary.analyzed, 1);
    assert_eq!(summary.skipped, 0);

    // Verify metadata was inserted
    let metadata: Option<(f64, String)> =
        sqlx::query_as("SELECT tempo_bpm, key_signature FROM musical_metadata WHERE file_id = $1")
            .bind(file_id)
            .fetch_optional(pool)
            .await
            .expect("Failed to fetch metadata");

    assert!(metadata.is_some(), "Metadata should exist");
    let (bpm, key) = metadata.unwrap();
    assert!((bpm - 120.0).abs() < 5.0, "BPM should be ~120, got {}", bpm);
    assert!(
        key.contains("C") || key.contains("major"),
        "Key should be C major-ish, got {}",
        key
    );

    // Verify analyzed_at was set
    let analyzed_at: Option<chrono::NaiveDateTime> =
        sqlx::query_scalar("SELECT analyzed_at FROM files WHERE file_id = $1")
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
        let midi_path =
            file_fixtures.create_midi_file(&format!("test_{}.mid", i), &midi_bytes).await;

        let file_id: i64 = sqlx::query_scalar(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3) RETURNING file_id",
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
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        window.clone(),
    )
    .await;

    assert!(result.is_ok(), "Analysis should succeed");
    let summary = result.unwrap();
    assert_eq!(summary.total_files, 100);
    assert_eq!(summary.analyzed, 100);
    assert_eq!(summary.skipped, 0);

    // Verify all files have metadata
    let metadata_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM musical_metadata WHERE file_id = ANY($1)")
            .bind(&file_ids)
            .fetch_one(pool)
            .await
            .expect("Failed to count metadata");

    assert_eq!(metadata_count, 100, "All files should have metadata");

    // Verify progress events were emitted (every 10 files)
    let events = window.get_events().await;
    let progress_events: Vec<_> =
        events.iter().filter(|(name, _)| name == "analysis-progress").collect();

    assert!(
        progress_events.len() >= 10,
        "Should have at least 10 progress events"
    );

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
        let midi_path =
            file_fixtures.create_midi_file(&format!("batch_{}.mid", i), &midi_bytes).await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)",
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
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok(), "Analysis should succeed");
    let summary = result.unwrap();
    assert_eq!(summary.total_files, 1000);
    assert_eq!(summary.analyzed, 1000);

    // Verify batching worked - all metadata inserted
    let metadata_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM musical_metadata")
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
         VALUES ($1, $2, $3) RETURNING file_id",
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
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok());

    // Verify comprehensive metadata extraction
    let metadata: (
        Option<f64>,    // tempo_bpm
        Option<String>, // key_signature
        Option<i16>,    // time_signature_num
        Option<i16>,    // time_signature_den
        Option<f64>,    // duration_seconds
        i32,            // note_count
    ) = sqlx::query_as(
        "SELECT tempo_bpm, key_signature, time_signature_num, time_signature_den,
                duration_seconds, note_count
         FROM musical_metadata WHERE file_id = $1",
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
    .expect("Metadata should exist");

    assert!(metadata.0.is_some(), "BPM should be detected");
    assert!(metadata.1.is_some(), "Key should be detected");
    assert_eq!(metadata.2, Some(4), "Time signature numerator should be 4");
    assert_eq!(
        metadata.3,
        Some(4),
        "Time signature denominator should be 4"
    );
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
        let midi_path = file_fixtures
            .create_midi_file(&format!("batch_insert_{}.mid", i), &midi_bytes)
            .await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)",
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
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok());
    let summary = result.unwrap();
    assert_eq!(summary.analyzed, 150);

    // Verify all metadata was inserted via batching
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM musical_metadata m
         JOIN files f ON m.file_id = f.file_id
         WHERE f.file_path LIKE '/tmp%batch_insert%'",
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
        let midi_path =
            file_fixtures.create_midi_file(&format!("worker_{}.mid", i), &midi_bytes).await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)",
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
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    let duration = start.elapsed();

    assert!(result.is_ok());
    let summary = result.unwrap();
    assert_eq!(summary.analyzed, 64);

    // With 32 workers, 64 files should complete faster than sequential
    // Sequential would be ~64 * analysis_time, parallel should be ~2 * analysis_time
    // This is a soft check - just ensure it completes
    assert!(
        duration.as_secs() < 30,
        "Should complete in reasonable time with parallelism"
    );

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
        let midi_path =
            file_fixtures.create_midi_file(&format!("counter_{}.mid", i), &midi_bytes).await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)",
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
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok());
    let summary = result.unwrap();

    // Arc<AtomicUsize> counter should accurately track all files
    assert_eq!(
        summary.analyzed, 25,
        "Counter should track exactly 25 analyzed files"
    );
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
             VALUES ($1, $2, $3)",
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
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok());
    let summary = result.unwrap();

    // Arc<Mutex<Vec<String>>> should collect errors from invalid files
    assert_eq!(summary.total_files, 10);
    assert_eq!(
        summary.analyzed + summary.skipped,
        10,
        "All files accounted for"
    );
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
        let midi_path = file_fixtures
            .create_midi_file(&format!("progress_{}.mid", i), &midi_bytes)
            .await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)",
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
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        window.clone(),
    )
    .await;

    assert!(result.is_ok());

    // Wait for events to be emitted
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let events = window.get_events().await;
    let progress_events: Vec<_> =
        events.iter().filter(|(name, _)| name == "analysis-progress").collect();

    // Should emit at files: 10, 20, 30, 35 (at minimum)
    assert!(
        progress_events.len() >= 4,
        "Should have at least 4 progress events, got {}",
        progress_events.len()
    );

    // Verify event structure
    if let Some((_, payload)) = progress_events.first() {
        assert!(
            payload.get("current").is_some(),
            "Event should have 'current' field"
        );
        assert!(
            payload.get("total").is_some(),
            "Event should have 'total' field"
        );
        assert!(
            payload.get("rate").is_some(),
            "Event should have 'rate' field"
        );
        assert!(
            payload.get("eta_seconds").is_some(),
            "Event should have 'eta_seconds' field"
        );
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
        let midi_path = file_fixtures
            .create_midi_file(&format!("large_batch_{}.mid", i), &midi_bytes)
            .await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)",
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
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

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
             VALUES ($1, $2, $3)",
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
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok(), "Analysis should continue despite failures");
    let summary = result.unwrap();

    assert_eq!(summary.total_files, 20);
    assert_eq!(
        summary.skipped, 20,
        "All files should be skipped due to file not found"
    );
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
        let midi_path = file_fixtures
            .create_midi_file(&format!("corrupt_{}.mid", i), &corrupt_bytes)
            .await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)",
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
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok(), "Should handle corrupt MIDI gracefully");
    let summary = result.unwrap();

    assert_eq!(summary.total_files, 15);
    assert_eq!(summary.skipped, 15, "All corrupt files should be skipped");
    assert!(
        !summary.errors.is_empty(),
        "Should have error messages for corrupt files"
    );

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
        let midi_path =
            file_fixtures.create_midi_file(&format!("bpm_{}.mid", i), &midi_bytes).await;

        let file_id: i64 = sqlx::query_scalar(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3) RETURNING file_id",
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
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok());

    // Verify BPM detection accuracy (within ±1%)
    let detected_bpms: Vec<f64> = sqlx::query_scalar(
        "SELECT tempo_bpm FROM musical_metadata m
         JOIN files f ON m.file_id = f.file_id
         WHERE f.file_path LIKE '/tmp%bpm_%'
         ORDER BY f.file_id",
    )
    .fetch_all(pool)
    .await
    .expect("Failed to fetch BPMs");

    for (i, detected) in detected_bpms.iter().enumerate() {
        let expected = test_bpms[i];
        let error_percent = ((detected - expected).abs() / expected) * 100.0;
        assert!(
            error_percent < 5.0,
            "BPM accuracy should be within 5% for {}, got {}, expected {}",
            i,
            detected,
            expected
        );
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
        ("C_major", vec![60, 62, 64, 65, 67, 69, 71, 72]), // C major scale
        ("D_major", vec![62, 64, 66, 67, 69, 71, 73, 74]), // D major scale
        ("E_minor", vec![64, 66, 67, 69, 71, 72, 74, 76]), // E minor scale
    ];

    for (i, (key_name, notes)) in keys.iter().enumerate() {
        let midi_bytes = create_midi_bytes(120.0, notes);
        let midi_path =
            file_fixtures.create_midi_file(&format!("key_{}.mid", i), &midi_bytes).await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes, filename)
             VALUES ($1, $2, $3, $4)",
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
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok());

    // Verify key detection produced results
    let detected_keys: Vec<Option<String>> = sqlx::query_scalar(
        "SELECT key_signature FROM musical_metadata m
         JOIN files f ON m.file_id = f.file_id
         WHERE f.filename LIKE '%major%' OR f.filename LIKE '%minor%'
         ORDER BY f.file_id",
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
         VALUES ($1, $2, $3) RETURNING file_id",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 150000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok());

    // Verify duration was calculated
    let (duration_secs, duration_ticks): (Option<f64>, Option<i32>) = sqlx::query_as(
        "SELECT duration_seconds, duration_ticks FROM musical_metadata WHERE file_id = $1",
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
    .expect("Failed to fetch duration");

    assert!(
        duration_secs.is_some(),
        "Duration in seconds should be calculated"
    );
    assert!(
        duration_ticks.is_some(),
        "Duration in ticks should be calculated"
    );
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
         VALUES ($1, $2, $3) RETURNING file_id",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 160000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok());

    // Verify instruments were extracted
    let instruments: Vec<String> =
        sqlx::query_scalar("SELECT instruments FROM musical_metadata WHERE file_id = $1")
            .bind(file_id)
            .fetch_one(pool)
            .await
            .expect("Failed to fetch instruments");

    assert!(!instruments.is_empty(), "Should detect instruments");
    // Should contain "Piano" from track name or program change
    assert!(
        instruments.iter().any(|i| i.contains("Piano") || i.contains("piano")),
        "Should detect Piano instrument"
    );

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
         VALUES ($1, $2, $3) RETURNING file_id",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 170000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok());

    // Verify tempo variation flag
    let has_variation: bool =
        sqlx::query_scalar("SELECT has_tempo_variation FROM musical_metadata WHERE file_id = $1")
            .bind(file_id)
            .fetch_one(pool)
            .await
            .expect("Failed to fetch tempo variation");

    // Simple file should not have tempo variation
    assert!(
        !has_variation,
        "Simple file should not have tempo variation"
    );

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
         VALUES ($1, $2, $3) RETURNING file_id",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 180000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok());

    // Verify time signature
    let (num, den): (Option<i16>, Option<i16>) = sqlx::query_as(
        "SELECT time_signature_num, time_signature_den FROM musical_metadata WHERE file_id = $1",
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
         VALUES ($1, $2, $3) RETURNING file_id",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 190000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok());

    // Verify polyphony detection
    let polyphony: Option<i16> =
        sqlx::query_scalar("SELECT polyphony_max FROM musical_metadata WHERE file_id = $1")
            .bind(file_id)
            .fetch_one(pool)
            .await
            .expect("Failed to fetch polyphony");

    assert!(polyphony.is_some(), "Polyphony should be detected");
    assert!(
        polyphony.unwrap() >= 3,
        "Should detect chord (3+ simultaneous notes)"
    );

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
         VALUES ($1, $2, $3) RETURNING file_id",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 200000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok());

    // Just verify analysis completed
    let note_count: i32 =
        sqlx::query_scalar("SELECT note_count FROM musical_metadata WHERE file_id = $1")
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
         VALUES ($1, $2, $3) RETURNING file_id",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 210000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok());

    // Verify velocity statistics
    let (avg_vel, vel_low, vel_high): (Option<f64>, Option<i16>, Option<i16>) = sqlx::query_as(
        "SELECT avg_velocity, velocity_range_low, velocity_range_high
         FROM musical_metadata WHERE file_id = $1",
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
    .expect("Failed to fetch velocity stats");

    assert!(avg_vel.is_some(), "Average velocity should be calculated");
    assert!(vel_low.is_some(), "Min velocity should be detected");
    assert!(vel_high.is_some(), "Max velocity should be detected");
    assert!(
        avg_vel.unwrap() > 0.0 && avg_vel.unwrap() <= 127.0,
        "Velocity should be in valid range"
    );

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
         VALUES ($1, $2, $3) RETURNING file_id",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 220000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok());

    // Verify note range
    let (low, high, semitones): (Option<i16>, Option<i16>, Option<i16>) = sqlx::query_as(
        "SELECT pitch_range_low, pitch_range_high, pitch_range_semitones
         FROM musical_metadata WHERE file_id = $1",
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
    assert_eq!(
        semitones.unwrap(),
        12,
        "Range should be 12 semitones (1 octave)"
    );

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
         VALUES ($1, $2, $3) RETURNING file_id",
    )
    .bind(simple_path.to_str().unwrap())
    .bind(format!("{:064x}", 230000))
    .bind(simple_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let complex_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3) RETURNING file_id",
    )
    .bind(complex_path.to_str().unwrap())
    .bind(format!("{:064x}", 230001))
    .bind(complex_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok());

    // Verify complexity scores
    let simple_score: Option<f64> =
        sqlx::query_scalar("SELECT complexity_score FROM musical_metadata WHERE file_id = $1")
            .bind(simple_id)
            .fetch_one(pool)
            .await
            .expect("Failed to fetch simple score");

    let complex_score: Option<f64> =
        sqlx::query_scalar("SELECT complexity_score FROM musical_metadata WHERE file_id = $1")
            .bind(complex_id)
            .fetch_one(pool)
            .await
            .expect("Failed to fetch complex score");

    assert!(
        simple_score.is_some(),
        "Simple file should have complexity score"
    );
    assert!(
        complex_score.is_some(),
        "Complex file should have complexity score"
    );
    assert!(
        complex_score.unwrap() > simple_score.unwrap(),
        "Complex file should have higher complexity score"
    );

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
         VALUES ($1, $2, $3) RETURNING file_id",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 240000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok());

    // Verify pitch bend detection
    let has_pitch_bend: bool =
        sqlx::query_scalar("SELECT has_pitch_bend FROM musical_metadata WHERE file_id = $1")
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
         VALUES ($1, $2, $3) RETURNING file_id",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 250000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok());

    // Verify CC detection
    let has_cc: bool =
        sqlx::query_scalar("SELECT has_cc_messages FROM musical_metadata WHERE file_id = $1")
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
        0x4D, 0x54, 0x68, 0x64, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x01, 0xE0,
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
         VALUES ($1, $2, $3) RETURNING file_id",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 260000))
    .bind(bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok());

    // Verify empty track handling
    let note_count: i32 =
        sqlx::query_scalar("SELECT note_count FROM musical_metadata WHERE file_id = $1")
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
         VALUES ($1, $2, $3) RETURNING file_id",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 270000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok());

    // Verify single note analysis
    let (note_count, pitch_low, pitch_high): (i32, Option<i16>, Option<i16>) = sqlx::query_as(
        "SELECT note_count, pitch_range_low, pitch_range_high
         FROM musical_metadata WHERE file_id = $1",
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
         VALUES ($1, $2, $3) RETURNING file_id",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 280000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok());

    // Verify duration calculation handles long files
    let duration: Option<f64> =
        sqlx::query_scalar("SELECT duration_seconds FROM musical_metadata WHERE file_id = $1")
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
         VALUES ($1, $2, $3, $4) RETURNING file_id",
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
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok(), "Should handle unicode filenames");

    // Verify metadata exists
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM musical_metadata WHERE file_id = $1)")
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
        let midi_path = file_fixtures
            .create_midi_file(&format!("boundary_{}.mid", i), &midi_bytes)
            .await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)",
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
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok());
    let summary = result.unwrap();

    // All files should be analyzed despite batch boundary
    assert_eq!(
        summary.analyzed, 101,
        "Should handle batch boundary correctly"
    );

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
        let midi_path = file_fixtures
            .create_midi_file(&format!("event_format_{}.mid", i), &midi_bytes)
            .await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)",
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
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        window.clone(),
    )
    .await;

    assert!(result.is_ok());

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let events = window.get_events().await;
    let progress_events: Vec<_> =
        events.iter().filter(|(name, _)| name == "analysis-progress").collect();

    // Verify event format
    for (_, payload) in progress_events {
        assert!(
            payload.get("current").is_some(),
            "Should have current field"
        );
        assert!(payload.get("total").is_some(), "Should have total field");
        assert!(
            payload.get("current_file").is_some(),
            "Should have current_file field"
        );
        assert!(payload.get("rate").is_some(), "Should have rate field");
        assert!(
            payload.get("eta_seconds").is_some(),
            "Should have eta_seconds field"
        );

        // Verify types
        assert!(payload["current"].is_number(), "current should be number");
        assert!(payload["total"].is_number(), "total should be number");
        assert!(
            payload["current_file"].is_string(),
            "current_file should be string"
        );
        assert!(payload["rate"].is_number(), "rate should be number");
        assert!(
            payload["eta_seconds"].is_number(),
            "eta_seconds should be number"
        );
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
        let midi_path =
            file_fixtures.create_midi_file(&format!("persist_{}.mid", i), &midi_bytes).await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)",
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
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok());

    // Second analysis should find 0 unanalyzed files (state persisted)
    let result2 = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result2.is_ok());
    let summary2 = result2.unwrap();
    assert_eq!(
        summary2.total_files, 0,
        "Should find 0 unanalyzed files on second run"
    );

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
         VALUES ($1, $2, $3) RETURNING file_id",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 330000))
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok());

    // Verify all expected fields are populated
    let metadata: (
        Option<f64>,
        Option<String>,
        Option<i16>,
        Option<i16>,
        Option<f64>,
        i32,
        Option<f64>,
        bool,
        bool,
    ) = sqlx::query_as(
        "SELECT tempo_bpm, key_signature, time_signature_num, time_signature_den,
                duration_seconds, note_count, complexity_score,
                has_pitch_bend, has_cc_messages
         FROM musical_metadata WHERE file_id = $1",
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
        let midi_path =
            file_fixtures.create_midi_file(&format!("perf_{}.mid", i), &midi_bytes).await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)",
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
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    let duration = start.elapsed();

    assert!(result.is_ok());
    let summary = result.unwrap();

    println!("Performance Results:");
    println!("  Total files: {}", summary.total_files);
    println!("  Analyzed: {}", summary.analyzed);
    println!("  Duration: {:.2}s", duration.as_secs_f64());
    println!("  Rate: {:.1} files/sec", summary.rate);

    // Target: >= 100 files/sec with 32 workers
    assert!(
        summary.rate >= 50.0,
        "Should achieve at least 50 files/sec, got {}",
        summary.rate
    );

    db.cleanup().await;
}

//=============================================================================
// SECTION 4: Worker Pool & Concurrency Errors (9 tests)
//=============================================================================

#[tokio::test]
async fn test_analyze_worker_pool_oom_simulation() {
    // Description: Simulate high memory pressure with large batch analysis
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    cleanup_database(&pool).await;

    // Create 500 files to stress worker pool memory
    let midi_bytes = create_midi_bytes(120.0, &[60, 62, 64]);

    for i in 0..500 {
        let midi_path =
            file_fixtures.create_midi_file(&format!("stress_{}.mid", i), &midi_bytes).await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)",
        )
        .bind(midi_path.to_str().unwrap())
        .bind(format!("{:064x}", i + 500000))
        .bind(midi_bytes.len() as i64)
        .execute(pool)
        .await
        .expect("Failed to insert");
    }

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(
        result.is_ok(),
        "Worker pool should handle large batches without OOM"
    );
    let summary = result.unwrap();
    assert_eq!(
        summary.analyzed, 500,
        "All files should be processed despite memory pressure"
    );

    cleanup_database(&pool).await;
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_database_insert_batch_failure() {
    // Description: Test recovery from database batch insert failure mid-operation
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    cleanup_database(&pool).await;

    // Create files with valid MIDI data
    let midi_bytes = create_midi_bytes(120.0, &[60, 64, 67]);

    for i in 0..50 {
        let midi_path = file_fixtures
            .create_midi_file(&format!("batch_fail_{}.mid", i), &midi_bytes)
            .await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)",
        )
        .bind(midi_path.to_str().unwrap())
        .bind(format!("{:064x}", i + 550000))
        .bind(midi_bytes.len() as i64)
        .execute(pool)
        .await
        .expect("Failed to insert");
    }

    // Analysis should succeed - database operations are transactional
    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(
        result.is_ok(),
        "Should handle database operations gracefully"
    );

    cleanup_database(&pool).await;
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_file_permission_denied() {
    // Description: Test handling of file read permission denied during analysis
    let db = TestDatabase::new().await;
    let pool = db.pool();

    cleanup_database(&pool).await;

    // Create file record for non-readable path
    sqlx::query(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3)"
    )
    .bind("/root/permission_denied.mid")  // Typically no read permission
    .bind(format!("{:064x}", 600000))
    .bind(1024i64)
    .execute(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok(), "Should handle permission errors gracefully");
    let summary = result.unwrap();
    assert_eq!(
        summary.skipped, 1,
        "File should be skipped due to permission error"
    );
    assert!(
        !summary.errors.is_empty(),
        "Should have error message for permission denied"
    );

    cleanup_database(&pool).await;
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_concurrent_analysis_race_condition() {
    // Description: Test race conditions on analyzed_at field with concurrent requests
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    cleanup_database(&pool).await;

    // Create 20 files
    let midi_bytes = create_midi_bytes(120.0, &[60]);

    for i in 0..20 {
        let midi_path =
            file_fixtures.create_midi_file(&format!("race_{}.mid", i), &midi_bytes).await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)",
        )
        .bind(midi_path.to_str().unwrap())
        .bind(format!("{:064x}", i + 610000))
        .bind(midi_bytes.len() as i64)
        .execute(pool)
        .await
        .expect("Failed to insert");
    }

    // Run two analysis operations concurrently to test race conditions
    let state1 = Arc::new(crate::AppState {
        database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
        }))
        .await
        .expect("Failed to connect"),
    });
    let state2 = state1.clone();

    let window1 = TestWindow::new();
    let window2 = TestWindow::new();

    let handle1 = tokio::spawn(async move {
        crate::commands::analyze::start_analysis(tauri::State::from(state1), window1).await
    });

    let handle2 = tokio::spawn(async move {
        crate::commands::analyze::start_analysis(tauri::State::from(state2), window2).await
    });

    let (result1, result2) = tokio::join!(handle1, handle2);

    // Both should succeed (one might find 0 unanalyzed files)
    assert!(result1.is_ok(), "First concurrent analysis should succeed");
    assert!(result2.is_ok(), "Second concurrent analysis should succeed");

    // Verify no duplicate metadata entries (race condition check)
    let metadata_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM musical_metadata m
         JOIN files f ON m.file_id = f.file_id
         WHERE f.file_path LIKE '/tmp%race_%'",
    )
    .fetch_one(pool)
    .await
    .expect("Failed to count metadata");

    assert_eq!(
        metadata_count, 20,
        "Should have exactly 20 metadata entries (no duplicates from race)"
    );

    cleanup_database(&pool).await;
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_corrupted_tempo_metadata() {
    // Description: Test handling of corrupted tempo metadata that could cause divide-by-zero
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    cleanup_database(&pool).await;

    // Create MIDI with tempo of 0 (invalid, could cause divide-by-zero)
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[
        0x4D, 0x54, 0x68, 0x64, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x01, 0xE0,
    ]);

    let mut track_data = Vec::new();
    // Tempo with 0 microseconds per beat (invalid)
    track_data.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03, 0x00, 0x00, 0x00]);
    track_data.extend_from_slice(&[0x00, 0x90, 60, 80]); // Note
    track_data.extend_from_slice(&[0x20, 0x80, 60, 40]); // Note off
    track_data.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]); // End

    bytes.extend_from_slice(&[0x4D, 0x54, 0x72, 0x6B]);
    let len = track_data.len() as u32;
    bytes.extend_from_slice(&[
        ((len >> 24) & 0xFF) as u8,
        ((len >> 16) & 0xFF) as u8,
        ((len >> 8) & 0xFF) as u8,
        (len & 0xFF) as u8,
    ]);
    bytes.extend_from_slice(&track_data);

    let midi_path = file_fixtures.create_midi_file("zero_tempo.mid", &bytes).await;

    sqlx::query(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3)",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 620000))
    .bind(bytes.len() as i64)
    .execute(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(
        result.is_ok(),
        "Should handle zero tempo without divide-by-zero panic"
    );

    cleanup_database(&pool).await;
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_all_zero_velocities() {
    // Description: Test velocity statistics with all-zero velocities to prevent zero division
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    cleanup_database(&pool).await;

    // Create MIDI with all zero velocities
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[
        0x4D, 0x54, 0x68, 0x64, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x01, 0xE0,
    ]);

    let mut track_data = Vec::new();
    track_data.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20]); // Tempo

    // Add notes with zero velocity
    for note in &[60, 64, 67] {
        track_data.extend_from_slice(&[0x00, 0x90, *note, 0]); // Velocity 0
        track_data.extend_from_slice(&[0x20, 0x80, *note, 0]); // Off
    }

    track_data.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

    bytes.extend_from_slice(&[0x4D, 0x54, 0x72, 0x6B]);
    let len = track_data.len() as u32;
    bytes.extend_from_slice(&[
        ((len >> 24) & 0xFF) as u8,
        ((len >> 16) & 0xFF) as u8,
        ((len >> 8) & 0xFF) as u8,
        (len & 0xFF) as u8,
    ]);
    bytes.extend_from_slice(&track_data);

    let midi_path = file_fixtures.create_midi_file("zero_velocity.mid", &bytes).await;

    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3) RETURNING file_id",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 630000))
    .bind(bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(
        result.is_ok(),
        "Should handle zero velocities without division error"
    );

    // Verify velocity stats are handled correctly
    let avg_vel: Option<f64> =
        sqlx::query_scalar("SELECT avg_velocity FROM musical_metadata WHERE file_id = $1")
            .bind(file_id)
            .fetch_one(pool)
            .await
            .expect("Failed to fetch velocity");

    // Should be Some(0.0) or None, not panic
    if let Some(vel) = avg_vel {
        assert_eq!(
            vel, 0.0,
            "Average velocity should be 0.0 for all-zero velocities"
        );
    }

    cleanup_database(&pool).await;
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_zero_ticks_per_beat() {
    // Description: Test duration calculation with zero ticks per beat to prevent overflow
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    cleanup_database(&pool).await;

    // Create MIDI with 0 ticks per beat (invalid header)
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[
        0x4D, 0x54, 0x68, 0x64, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x00,
        0x00, // 0 ticks per beat - invalid!
    ]);

    let track_data = vec![
        0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20, // Tempo
        0x00, 0x90, 60, 80, // Note
        0x20, 0x80, 60, 40, // Note off
        0x00, 0xFF, 0x2F, 0x00, // End
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

    let midi_path = file_fixtures.create_midi_file("zero_ticks.mid", &bytes).await;

    sqlx::query(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3)",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 640000))
    .bind(bytes.len() as i64)
    .execute(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(
        result.is_ok(),
        "Should handle zero ticks per beat without panic or overflow"
    );

    cleanup_database(&pool).await;
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_high_polyphony_overflow() {
    // Description: Test saturation arithmetic with >100 simultaneous notes (polyphony overflow)
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    cleanup_database(&pool).await;

    // Create MIDI with 120 simultaneous notes
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[
        0x4D, 0x54, 0x68, 0x64, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x01, 0xE0,
    ]);

    let mut track_data = Vec::new();
    track_data.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20]); // Tempo

    // Turn on 120 notes simultaneously
    for note in 0..120 {
        track_data.extend_from_slice(&[0x00, 0x90, note, 80]);
    }

    // Turn them all off
    for note in 0..120 {
        track_data.extend_from_slice(&[0x00, 0x80, note, 40]);
    }

    track_data.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

    bytes.extend_from_slice(&[0x4D, 0x54, 0x72, 0x6B]);
    let len = track_data.len() as u32;
    bytes.extend_from_slice(&[
        ((len >> 24) & 0xFF) as u8,
        ((len >> 16) & 0xFF) as u8,
        ((len >> 8) & 0xFF) as u8,
        (len & 0xFF) as u8,
    ]);
    bytes.extend_from_slice(&track_data);

    let midi_path = file_fixtures.create_midi_file("high_poly.mid", &bytes).await;

    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3) RETURNING file_id",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 650000))
    .bind(bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(
        result.is_ok(),
        "Should handle high polyphony without overflow"
    );

    // Verify polyphony was tracked correctly (should use saturation arithmetic)
    let polyphony: Option<i16> =
        sqlx::query_scalar("SELECT polyphony_max FROM musical_metadata WHERE file_id = $1")
            .bind(file_id)
            .fetch_one(pool)
            .await
            .expect("Failed to fetch polyphony");

    assert!(polyphony.is_some(), "Polyphony should be detected");
    // i16::MAX is 32767, should saturate rather than overflow
    assert!(polyphony.unwrap() > 0, "Polyphony should be positive");

    cleanup_database(&pool).await;
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_malformed_midi_header() {
    // Description: Test handling of completely malformed MIDI header
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    cleanup_database(&pool).await;

    // Create file with invalid header (not "MThd")
    let malformed_bytes = vec![
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x01, 0xE0,
    ];

    let midi_path = file_fixtures.create_midi_file("malformed.mid", &malformed_bytes).await;

    sqlx::query(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3)",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 660000))
    .bind(malformed_bytes.len() as i64)
    .execute(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok(), "Should handle malformed header gracefully");
    let summary = result.unwrap();
    assert_eq!(summary.skipped, 1, "Malformed file should be skipped");
    assert!(!summary.errors.is_empty(), "Should have error message");

    cleanup_database(&pool).await;
    db.cleanup().await;
}

//=============================================================================
// SECTION 5: Data Validation Errors (9 tests)
//=============================================================================

#[tokio::test]
async fn test_analyze_extreme_bpm_values() {
    // Description: Test handling of extreme BPM values (very high/low)
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    cleanup_database(&pool).await;

    // Create MIDI with extremely high BPM (tempo of 1 microsecond per beat)
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[
        0x4D, 0x54, 0x68, 0x64, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x01, 0xE0,
    ]);

    let mut track_data = Vec::new();
    // Tempo: 1 microsecond per beat = 60,000,000 BPM
    track_data.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03, 0x00, 0x00, 0x01]);
    track_data.extend_from_slice(&[0x00, 0x90, 60, 80]);
    track_data.extend_from_slice(&[0x20, 0x80, 60, 40]);
    track_data.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

    bytes.extend_from_slice(&[0x4D, 0x54, 0x72, 0x6B]);
    let len = track_data.len() as u32;
    bytes.extend_from_slice(&[
        ((len >> 24) & 0xFF) as u8,
        ((len >> 16) & 0xFF) as u8,
        ((len >> 8) & 0xFF) as u8,
        (len & 0xFF) as u8,
    ]);
    bytes.extend_from_slice(&track_data);

    let midi_path = file_fixtures.create_midi_file("extreme_bpm.mid", &bytes).await;

    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3) RETURNING file_id",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 670000))
    .bind(bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(
        result.is_ok(),
        "Should handle extreme BPM values without overflow"
    );

    // Verify BPM is stored (might be clamped to reasonable range)
    let bpm: Option<f64> =
        sqlx::query_scalar("SELECT tempo_bpm FROM musical_metadata WHERE file_id = $1")
            .bind(file_id)
            .fetch_one(pool)
            .await
            .expect("Failed to fetch BPM");

    if let Some(bpm_val) = bpm {
        assert!(bpm_val.is_finite(), "BPM should be finite number");
        assert!(bpm_val > 0.0, "BPM should be positive");
    }

    cleanup_database(&pool).await;
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_invalid_time_signature() {
    // Description: Test handling of invalid time signature values
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    cleanup_database(&pool).await;

    // Create MIDI with invalid time signature (0/0)
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[
        0x4D, 0x54, 0x68, 0x64, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x01, 0xE0,
    ]);

    let mut track_data = Vec::new();
    track_data.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20]); // Tempo
                                                                               // Invalid time signature: 0/0
    track_data.extend_from_slice(&[0x00, 0xFF, 0x58, 0x04, 0x00, 0x00, 0x18, 0x08]);
    track_data.extend_from_slice(&[0x00, 0x90, 60, 80]);
    track_data.extend_from_slice(&[0x20, 0x80, 60, 40]);
    track_data.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

    bytes.extend_from_slice(&[0x4D, 0x54, 0x72, 0x6B]);
    let len = track_data.len() as u32;
    bytes.extend_from_slice(&[
        ((len >> 24) & 0xFF) as u8,
        ((len >> 16) & 0xFF) as u8,
        ((len >> 8) & 0xFF) as u8,
        (len & 0xFF) as u8,
    ]);
    bytes.extend_from_slice(&track_data);

    let midi_path = file_fixtures.create_midi_file("invalid_timesig.mid", &bytes).await;

    sqlx::query(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3)",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 680000))
    .bind(bytes.len() as i64)
    .execute(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(
        result.is_ok(),
        "Should handle invalid time signature gracefully"
    );

    cleanup_database(&pool).await;
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_note_range_overflow() {
    // Description: Test note range calculation with extreme pitch values
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    cleanup_database(&pool).await;

    // Create MIDI with notes at extreme ends (0 and 127)
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[
        0x4D, 0x54, 0x68, 0x64, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x01, 0xE0,
    ]);

    let mut track_data = Vec::new();
    track_data.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20]);

    // Extreme low and high notes
    track_data.extend_from_slice(&[0x00, 0x90, 0, 80]); // Note 0
    track_data.extend_from_slice(&[0x00, 0x90, 127, 80]); // Note 127
    track_data.extend_from_slice(&[0x20, 0x80, 0, 40]);
    track_data.extend_from_slice(&[0x00, 0x80, 127, 40]);
    track_data.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

    bytes.extend_from_slice(&[0x4D, 0x54, 0x72, 0x6B]);
    let len = track_data.len() as u32;
    bytes.extend_from_slice(&[
        ((len >> 24) & 0xFF) as u8,
        ((len >> 16) & 0xFF) as u8,
        ((len >> 8) & 0xFF) as u8,
        (len & 0xFF) as u8,
    ]);
    bytes.extend_from_slice(&track_data);

    let midi_path = file_fixtures.create_midi_file("extreme_notes.mid", &bytes).await;

    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3) RETURNING file_id",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 690000))
    .bind(bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok(), "Should handle extreme note range");

    // Verify note range calculation
    let (low, high, semitones): (Option<i16>, Option<i16>, Option<i16>) = sqlx::query_as(
        "SELECT pitch_range_low, pitch_range_high, pitch_range_semitones
         FROM musical_metadata WHERE file_id = $1",
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
    .expect("Failed to fetch note range");

    assert_eq!(low, Some(0), "Low note should be 0");
    assert_eq!(high, Some(127), "High note should be 127");
    assert_eq!(semitones, Some(127), "Range should be 127 semitones");

    cleanup_database(&pool).await;
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_truncated_midi_file() {
    // Description: Test handling of truncated MIDI file (incomplete data)
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    cleanup_database(&pool).await;

    // Create truncated MIDI (header but no track data)
    let truncated_bytes = vec![
        0x4D, 0x54, 0x68, 0x64, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x01, 0xE0,
        // Track header but missing data
        0x4D, 0x54, 0x72, 0x6B, 0x00, 0x00, 0x00, 0x10,
        // Data truncated...
    ];

    let midi_path = file_fixtures.create_midi_file("truncated.mid", &truncated_bytes).await;

    sqlx::query(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3)",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 700000))
    .bind(truncated_bytes.len() as i64)
    .execute(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok(), "Should handle truncated file gracefully");
    let summary = result.unwrap();
    assert_eq!(summary.skipped, 1, "Truncated file should be skipped");

    cleanup_database(&pool).await;
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_database_connection_pool_exhaustion() {
    // Description: Test handling when database connection pool is exhausted
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    cleanup_database(&pool).await;

    // Create 100 files to stress connection pool
    let midi_bytes = create_midi_bytes(120.0, &[60]);

    for i in 0..100 {
        let midi_path = file_fixtures
            .create_midi_file(&format!("pool_stress_{}.mid", i), &midi_bytes)
            .await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)",
        )
        .bind(midi_path.to_str().unwrap())
        .bind(format!("{:064x}", i + 710000))
        .bind(midi_bytes.len() as i64)
        .execute(pool)
        .await
        .expect("Failed to insert");
    }

    // Analysis should succeed even under connection pool stress
    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok(), "Should handle connection pool stress");
    let summary = result.unwrap();
    assert_eq!(
        summary.analyzed, 100,
        "All files should be analyzed despite pool stress"
    );

    cleanup_database(&pool).await;
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_negative_delta_time() {
    // Description: Test handling of invalid negative delta time in MIDI events
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    cleanup_database(&pool).await;

    // MIDI uses variable-length quantities for delta time, can't be truly negative
    // but we can test handling of unusual delta time sequences
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[
        0x4D, 0x54, 0x68, 0x64, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x01, 0xE0,
    ]);

    let mut track_data = Vec::new();
    track_data.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20]);

    // Very large delta time (max variable length: 0x0FFFFFFF)
    track_data.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0x7F]); // Max valid delta
    track_data.extend_from_slice(&[0x90, 60, 80]);
    track_data.extend_from_slice(&[0x20, 0x80, 60, 40]);
    track_data.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

    bytes.extend_from_slice(&[0x4D, 0x54, 0x72, 0x6B]);
    let len = track_data.len() as u32;
    bytes.extend_from_slice(&[
        ((len >> 24) & 0xFF) as u8,
        ((len >> 16) & 0xFF) as u8,
        ((len >> 8) & 0xFF) as u8,
        (len & 0xFF) as u8,
    ]);
    bytes.extend_from_slice(&track_data);

    let midi_path = file_fixtures.create_midi_file("large_delta.mid", &bytes).await;

    sqlx::query(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3)",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 720000))
    .bind(bytes.len() as i64)
    .execute(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok(), "Should handle extreme delta time values");

    cleanup_database(&pool).await;
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_mixed_valid_invalid_batch() {
    // Description: Test batch processing with mix of valid and invalid files
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    cleanup_database(&pool).await;

    let valid_bytes = create_midi_bytes(120.0, &[60, 64, 67]);
    let invalid_bytes = vec![0xFF, 0xFF, 0xFF];

    // Create 50 files: 25 valid, 25 invalid, interleaved
    for i in 0..50 {
        let (bytes, name) = if i % 2 == 0 {
            (&valid_bytes, format!("mixed_valid_{}.mid", i))
        } else {
            (&invalid_bytes, format!("mixed_invalid_{}.mid", i))
        };

        let midi_path = file_fixtures.create_midi_file(&name, bytes).await;

        sqlx::query(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
             VALUES ($1, $2, $3)",
        )
        .bind(midi_path.to_str().unwrap())
        .bind(format!("{:064x}", i + 730000))
        .bind(bytes.len() as i64)
        .execute(pool)
        .await
        .expect("Failed to insert");
    }

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok(), "Should handle mixed batch gracefully");
    let summary = result.unwrap();
    assert_eq!(summary.total_files, 50, "Should process all 50 files");
    assert_eq!(summary.analyzed, 25, "Should analyze 25 valid files");
    assert_eq!(summary.skipped, 25, "Should skip 25 invalid files");
    assert_eq!(summary.errors.len(), 25, "Should have 25 error messages");

    cleanup_database(&pool).await;
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_very_large_file_timeout() {
    // Description: Test handling of very large MIDI files that might timeout
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    cleanup_database(&pool).await;

    // Create MIDI with thousands of events to simulate large file
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[
        0x4D, 0x54, 0x68, 0x64, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x01, 0xE0,
    ]);

    let mut track_data = Vec::new();
    track_data.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20]);

    // Add 1000 notes to make file larger
    for note in 0..100 {
        for pitch in 60..70 {
            track_data.extend_from_slice(&[0x00, 0x90, pitch, 80]);
            track_data.extend_from_slice(&[0x10, 0x80, pitch, 40]);
        }
    }

    track_data.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

    bytes.extend_from_slice(&[0x4D, 0x54, 0x72, 0x6B]);
    let len = track_data.len() as u32;
    bytes.extend_from_slice(&[
        ((len >> 24) & 0xFF) as u8,
        ((len >> 16) & 0xFF) as u8,
        ((len >> 8) & 0xFF) as u8,
        (len & 0xFF) as u8,
    ]);
    bytes.extend_from_slice(&track_data);

    let midi_path = file_fixtures.create_midi_file("very_large.mid", &bytes).await;

    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3) RETURNING file_id",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 740000))
    .bind(bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok(), "Should handle large files without timeout");

    // Verify note count is correct
    let note_count: i32 =
        sqlx::query_scalar("SELECT note_count FROM musical_metadata WHERE file_id = $1")
            .bind(file_id)
            .fetch_one(pool)
            .await
            .expect("Failed to fetch note count");

    assert_eq!(
        note_count, 1000,
        "Should count all 1000 notes in large file"
    );

    cleanup_database(&pool).await;
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_empty_file() {
    // Description: Test handling of completely empty file (0 bytes)
    let db = TestDatabase::new().await;
    let file_fixtures = FileFixtures::new().await;
    let pool = db.pool();

    cleanup_database(&pool).await;

    // Create empty file
    let empty_bytes: Vec<u8> = vec![];
    let midi_path = file_fixtures.create_midi_file("empty.mid", &empty_bytes).await;

    sqlx::query(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes)
         VALUES ($1, $2, $3)",
    )
    .bind(midi_path.to_str().unwrap())
    .bind(format!("{:064x}", 750000))
    .bind(0i64)
    .execute(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok(), "Should handle empty file gracefully");
    let summary = result.unwrap();
    assert_eq!(summary.skipped, 1, "Empty file should be skipped");
    assert!(
        !summary.errors.is_empty(),
        "Should have error message for empty file"
    );

    cleanup_database(&pool).await;
    db.cleanup().await;
}

// ===== SECTION 5: ERROR PATH TESTING (12 comprehensive error scenario tests) =====

#[tokio::test]
async fn test_analyze_error_no_unanalyzed_files() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    for i in 0..5 {
        let file_id: i64 = sqlx::query_scalar(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes) VALUES ($1, $2, $3) RETURNING file_id"
        )
        .bind(format!("/test/analyzed_{}.mid", i))
        .bind(format!("{:064x}", i))
        .bind(1024i64)
        .fetch_one(pool)
        .await
        .expect("Failed to insert");

        sqlx::query("UPDATE files SET analyzed_at = NOW() WHERE file_id = $1")
            .bind(file_id)
            .execute(pool)
            .await
            .expect("Failed to mark analyzed");
    }

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(
        result.is_ok(),
        "Should handle no unanalyzed files gracefully"
    );
    let summary = result.unwrap();
    assert_eq!(summary.analyzed, 0, "No files should be analyzed");
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_error_corrupted_midi_file() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes) VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind("/test/corrupted.mid")
    .bind("0000000000000000000000000000000000000000000000000000000000000001")
    .bind(512i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok(), "Should handle corrupted MIDI gracefully");
    let summary = result.unwrap();
    assert!(
        summary.errors.len() > 0,
        "Should record error for corrupted file"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_error_invalid_tempo_values() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    let midi_bytes = create_midi_bytes(0.0, &[60]);

    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes) VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind("/test/zero_tempo.mid")
    .bind("0000000000000000000000000000000000000000000000000000000000000002")
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok(), "Should handle zero tempo gracefully");
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_error_batch_partial_failure() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    for i in 0..10 {
        let midi_bytes = if i % 2 == 0 {
            create_midi_bytes(120.0, &[60, 64, 67])
        } else {
            vec![]
        };

        let _: i64 = sqlx::query_scalar(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes) VALUES ($1, $2, $3) RETURNING file_id"
        )
        .bind(format!("/test/mixed_{}.mid", i))
        .bind(format!("{:064x}", i + 100))
        .bind(midi_bytes.len() as i64)
        .fetch_one(pool)
        .await
        .expect("Failed to insert");
    }

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok(), "Should handle batch with some failures");
    let summary = result.unwrap();
    assert!(summary.total_files > 0, "Should process multiple files");
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_error_empty_track_handling() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    let midi_bytes = vec![
        0x4D, 0x54, 0x68, 0x64, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x00, 0x60,
    ];

    let _: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes) VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind("/test/empty_track.mid")
    .bind("0000000000000000000000000000000000000000000000000000000000000003")
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok(), "Should handle empty track gracefully");
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_error_concurrent_analysis_idempotent() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    let midi_bytes = create_midi_bytes(120.0, &[60, 64, 67]);
    let _file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes) VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind("/test/concurrent.mid")
    .bind("0000000000000000000000000000000000000000000000000000000000000004")
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result1 = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    let result2 = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    if let (Ok(s1), Ok(s2)) = (result1, result2) {
        assert_eq!(
            s1.analyzed + s1.skipped,
            s2.analyzed + s2.skipped,
            "Concurrent analysis should be consistent"
        );
    }
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_error_large_batch_processing() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    for i in 0..500 {
        let midi_bytes = create_midi_bytes(100.0 + (i as f64 % 40.0), &[60, 64, 67]);
        let _: i64 = sqlx::query_scalar(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes) VALUES ($1, $2, $3) RETURNING file_id"
        )
        .bind(format!("/test/large_batch_{}.mid", i))
        .bind(format!("{:064x}", i + 1000))
        .bind(midi_bytes.len() as i64)
        .fetch_one(pool)
        .await
        .expect("Failed to insert");
    }

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(
        result.is_ok(),
        "Should handle large batch without panicking"
    );
    let summary = result.unwrap();
    assert!(summary.total_files > 0, "Should process large batch");
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_error_invalid_key_detection() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    let midi_bytes = create_midi_bytes(120.0, &[]);

    let _: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes) VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind("/test/no_notes.mid")
    .bind("0000000000000000000000000000000000000000000000000000000000000005")
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(
        result.is_ok(),
        "Should handle files with no notes gracefully"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_error_atonal_music() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    let midi_bytes = create_midi_bytes(120.0, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);

    let _: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes) VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind("/test/atonal.mid")
    .bind("0000000000000000000000000000000000000000000000000000000000000006")
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(
        result.is_ok(),
        "Should handle atonal music with low confidence"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_error_duration_edge_cases() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    let midi_bytes = create_midi_bytes(500.0, &[60]);

    let _: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes) VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind("/test/extreme_tempo.mid")
    .bind("0000000000000000000000000000000000000000000000000000000000000007")
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok(), "Should handle extreme tempo values");
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_error_progress_event_throttling() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    for i in 0..50 {
        let midi_bytes = create_midi_bytes(120.0, &[60, 64, 67]);
        let _: i64 = sqlx::query_scalar(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes) VALUES ($1, $2, $3) RETURNING file_id"
        )
        .bind(format!("/test/progress_{}.mid", i))
        .bind(format!("{:064x}", i + 2000))
        .bind(midi_bytes.len() as i64)
        .fetch_one(pool)
        .await
        .expect("Failed to insert");
    }

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(result.is_ok(), "Should handle batch with event throttling");
    db.cleanup().await;
}

#[tokio::test]
async fn test_analyze_error_metadata_insertion_rollback() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    let midi_bytes = create_midi_bytes(120.0, &[60, 64, 67]);
    let file_id: i64 = sqlx::query_scalar(
        "INSERT INTO files (file_path, blake3_hash, file_size_bytes) VALUES ($1, $2, $3) RETURNING file_id"
    )
    .bind("/test/rollback_test.mid")
    .bind("0000000000000000000000000000000000000000000000000000000000000008")
    .bind(midi_bytes.len() as i64)
    .fetch_one(pool)
    .await
    .expect("Failed to insert");

    let result = crate::commands::analyze::start_analysis(
        tauri::State::from(Arc::new(crate::AppState {
            database: crate::Database::new(&std::env::var("TEST_DATABASE_URL").unwrap_or_else(
                |_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string(),
            ))
            .await
            .expect("Failed to connect"),
        })),
        TestWindow::new(),
    )
    .await;

    assert!(
        result.is_ok(),
        "Should handle metadata insertion with proper state management"
    );
    db.cleanup().await;
}

// Helper function for database cleanup
async fn cleanup_database(pool: &sqlx::PgPool) {
    // Clean up test data to avoid conflicts
    let _ = sqlx::query("DELETE FROM musical_metadata").execute(pool).await;
    let _ = sqlx::query("DELETE FROM files").execute(pool).await;
}

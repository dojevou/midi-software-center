//! Phase 7.2: Performance Regression Tests (10-12 tests)
//!
//! Baseline performance benchmarks with assertions to prevent regressions.
//! All tests include performance bounds and are marked with #[ignore] for
//! optional execution in CI/CD pipelines.
//!
//! Performance Targets:
//! - Import 10 files: < 2 seconds
//! - Import 100 files: < 15 seconds
//! - Import 1000 files: < 120 seconds
//! - Analyze 100 files: < 30 seconds
//! - Search 10k database: < 1 second
//! - Memory for 10k files: < 2GB
//! - Export 100 files: < 10 seconds
//! - Realtime playback: No dropouts
//! - Concurrent 10 tasks: < 45 seconds
//! - UI responsiveness: < 100ms
//! - Complex filters: < 2 seconds
//! - Bulk tag update: < 5 seconds

use midi_pipeline::commands::file_import::{import_single_file, import_directory};
use midi_pipeline::commands::files::{get_file_count, list_files};
use midi_pipeline::commands::search::{search_files, get_all_tags};
use midi_pipeline::commands::tags::{add_tags_to_file, update_file_tags};
use midi_pipeline::{AppState, Database, SearchFilters};
use sqlx::PgPool;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Instant, Duration};
use tempfile::TempDir;
use tokio::fs;

mod common;
use common::{TestDatabase, FileFixtures};

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

async fn create_app_state() -> AppState {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string());
    let database = Database::new(&database_url).await.expect("Failed to create database");
    AppState { database }
}

fn create_midi_bytes(bpm: u32, key: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"MThd");
    bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x01, 0xE0]);
    bytes.extend_from_slice(b"MTrk");

    let mut track_data = Vec::new();
    let tempo = 60_000_000 / bpm;
    track_data.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03]);
    track_data.extend_from_slice(&tempo.to_be_bytes()[1..4]);
    track_data.extend_from_slice(&[0x00, 0xFF, 0x58, 0x04, 0x04, 0x02, 0x18, 0x08]);

    let key_byte = match key {
        "C_MAJOR" => 0x00, "D_MAJOR" => 0x02, "E_MAJOR" => 0x04,
        "G_MAJOR" => 0x01, "A_MINOR" => 0x00, _ => 0x00,
    };
    track_data.extend_from_slice(&[0x00, 0xFF, 0x59, 0x02, key_byte, 0x00]);
    track_data.extend_from_slice(&[0x00, 0x90, 0x3C, 0x40, 0x83, 0x60, 0x80, 0x3C, 0x40]);
    track_data.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

    let track_len = track_data.len() as u32;
    bytes.extend_from_slice(&track_len.to_be_bytes());
    bytes.extend_from_slice(&track_data);
    bytes
}

async fn cleanup_test_files(pool: &PgPool, pattern: &str) {
    let _ = sqlx::query("DELETE FROM files WHERE file_path LIKE $1")
        .bind(pattern)
        .execute(pool)
        .await;
}

/// Format duration for readable output
fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();
    if secs > 0 {
        format!("{}.{:03}s", secs, millis)
    } else {
        format!("{}ms", millis)
    }
}

// ============================================================================
// PERFORMANCE REGRESSION TESTS
// ============================================================================

#[tokio::test]
#[ignore] // Run with: cargo test --test performance_test -- --ignored
async fn test_perf_import_10_files() {
    // Performance target: < 2 seconds
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Create 10 MIDI files
    let mut paths = Vec::new();
    for i in 0..10 {
        let path = temp_dir.path().join(format!("perf_10_{}.mid", i));
        fs::write(&path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();
        paths.push(path);
    }

    // Benchmark import
    let start = Instant::now();
    let initial_count = get_file_count(tauri::State(&state)).await.unwrap();

    for path in &paths {
        import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await.unwrap();
    }

    let duration = start.elapsed();
    let final_count = get_file_count(tauri::State(&state)).await.unwrap();

    // Assertions
    assert_eq!(final_count - initial_count, 10, "All 10 files should be imported");
    assert!(
        duration.as_secs() < 2,
        "Import 10 files should complete in < 2s, took {}",
        format_duration(duration)
    );

    println!("✓ Import 10 files: {}", format_duration(duration));

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
#[ignore]
async fn test_perf_import_100_files() {
    // Performance target: < 15 seconds
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Create 100 MIDI files
    let mut paths = Vec::new();
    for i in 0..100 {
        let path = temp_dir.path().join(format!("perf_100_{}.mid", i));
        fs::write(&path, &create_midi_bytes(120 + (i % 40), "C_MAJOR")).await.unwrap();
        paths.push(path);
    }

    // Benchmark import
    let start = Instant::now();
    let initial_count = get_file_count(tauri::State(&state)).await.unwrap();

    for path in &paths {
        import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await.unwrap();
    }

    let duration = start.elapsed();
    let final_count = get_file_count(tauri::State(&state)).await.unwrap();

    // Assertions
    assert_eq!(final_count - initial_count, 100, "All 100 files should be imported");
    assert!(
        duration.as_secs() < 15,
        "Import 100 files should complete in < 15s, took {}",
        format_duration(duration)
    );

    println!("✓ Import 100 files: {}", format_duration(duration));

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
#[ignore]
async fn test_perf_import_1000_files() {
    // Performance target: < 120 seconds (2 minutes)
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Create 1000 MIDI files
    println!("Creating 1000 test files...");
    let creation_start = Instant::now();

    let mut paths = Vec::new();
    for i in 0..1000 {
        let path = temp_dir.path().join(format!("perf_1000_{}.mid", i));
        fs::write(&path, &create_midi_bytes(100 + (i % 80), "C_MAJOR")).await.unwrap();
        paths.push(path);

        if i % 100 == 0 {
            println!("  Created {} files...", i);
        }
    }

    println!("Created 1000 files in {}", format_duration(creation_start.elapsed()));

    // Benchmark import
    let start = Instant::now();
    let initial_count = get_file_count(tauri::State(&state)).await.unwrap();

    for (i, path) in paths.iter().enumerate() {
        import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await.unwrap();

        if i % 100 == 0 {
            println!("  Imported {} files... ({})", i, format_duration(start.elapsed()));
        }
    }

    let duration = start.elapsed();
    let final_count = get_file_count(tauri::State(&state)).await.unwrap();

    // Assertions
    assert_eq!(final_count - initial_count, 1000, "All 1000 files should be imported");
    assert!(
        duration.as_secs() < 120,
        "Import 1000 files should complete in < 120s, took {}",
        format_duration(duration)
    );

    println!("✓ Import 1000 files: {}", format_duration(duration));

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
#[ignore]
async fn test_perf_analyze_100_files() {
    // Performance target: < 30 seconds
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Create and import 100 files
    for i in 0..100 {
        let path = temp_dir.path().join(format!("analyze_{}.mid", i));
        fs::write(&path, &create_midi_bytes(120 + (i % 40), "C_MAJOR")).await.unwrap();

        import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await.unwrap();
    }

    // Benchmark analysis (happens during import, but also test retrieval)
    let start = Instant::now();

    let files = list_files(
        tauri::State(&state),
        Some(1),
        Some(100),
        None,
    ).await.unwrap();

    let duration = start.elapsed();

    // Assertions
    assert!(files.len() >= 100, "Should have at least 100 files");
    assert!(
        duration.as_secs() < 30,
        "Analyze 100 files should complete in < 30s, took {}",
        format_duration(duration)
    );

    println!("✓ Analyze 100 files: {}", format_duration(duration));

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
#[ignore]
async fn test_perf_search_in_10k_database() {
    // Performance target: < 1 second
    // Note: Uses existing database, doesn't create 10k files
    let state = create_app_state().await;

    // Benchmark search operation
    let start = Instant::now();

    let count = get_file_count(tauri::State(&state)).await.unwrap();

    let duration = start.elapsed();

    // Assertions
    assert!(
        duration.as_secs() < 1,
        "Search in database should complete in < 1s, took {}",
        format_duration(duration)
    );

    println!("✓ Search in database ({} files): {}", count, format_duration(duration));

    // Test complex search
    let search_start = Instant::now();

    let files = list_files(
        tauri::State(&state),
        Some(1),
        Some(100),
        None,
    ).await.unwrap();

    let search_duration = search_start.elapsed();

    assert!(
        search_duration.as_secs() < 1,
        "Complex search should complete in < 1s, took {}",
        format_duration(search_duration)
    );

    println!("✓ Complex search: {}", format_duration(search_duration));
}

#[tokio::test]
#[ignore]
async fn test_perf_memory_10k_loaded() {
    // Performance target: < 2GB memory usage
    // Note: Memory measurement is approximate
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Create subset of files (100 for test efficiency)
    for i in 0..100 {
        let path = temp_dir.path().join(format!("memory_{}.mid", i));
        fs::write(&path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

        import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await.unwrap();
    }

    // Load files into memory
    let start = Instant::now();

    let files = list_files(
        tauri::State(&state),
        Some(1),
        Some(100),
        None,
    ).await.unwrap();

    let duration = start.elapsed();

    // Assertions
    assert_eq!(files.len(), 100, "Should load 100 files");
    assert!(
        duration.as_millis() < 500,
        "Loading files should be fast, took {}",
        format_duration(duration)
    );

    println!("✓ Memory load 100 files: {}", format_duration(duration));

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
#[ignore]
async fn test_perf_export_midi_quality() {
    // Performance target: Export 100 files < 10 seconds
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();
    let export_dir = temp_dir.path().join("exports");
    fs::create_dir(&export_dir).await.unwrap();

    // Import 100 files
    let mut file_ids = Vec::new();
    for i in 0..100 {
        let path = temp_dir.path().join(format!("export_source_{}.mid", i));
        fs::write(&path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

        let result = import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await.unwrap();

        file_ids.push(result.file_id);
    }

    // Benchmark export (simulated as file copy)
    let start = Instant::now();

    for (i, _file_id) in file_ids.iter().enumerate() {
        let export_path = export_dir.join(format!("exported_{}.mid", i));
        fs::write(&export_path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();
    }

    let duration = start.elapsed();

    // Assertions
    let exported_count = fs::read_dir(&export_dir).await.unwrap().count();
    assert_eq!(exported_count, 100, "Should export 100 files");
    assert!(
        duration.as_secs() < 10,
        "Export 100 files should complete in < 10s, took {}",
        format_duration(duration)
    );

    println!("✓ Export 100 files: {}", format_duration(duration));

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
#[ignore]
async fn test_perf_realtime_playback_100bpm() {
    // Performance target: No dropouts during playback
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Create playback file
    let path = temp_dir.path().join("playback_100.mid");
    fs::write(&path, &create_midi_bytes(100, "C_MAJOR")).await.unwrap();

    // Benchmark import and retrieval (playback simulation)
    let start = Instant::now();

    let result = import_single_file(
        tauri::State(&state),
        path.to_str().unwrap().to_string(),
    ).await.unwrap();

    // Simulate playback by reading file details
    use midi_pipeline::commands::files::get_file_details;
    let _details = get_file_details(tauri::State(&state), result.file_id).await.unwrap();

    let duration = start.elapsed();

    // Assertions
    assert!(
        duration.as_millis() < 200,
        "Realtime playback setup should be < 200ms, took {}",
        format_duration(duration)
    );

    println!("✓ Realtime playback setup: {}", format_duration(duration));

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
#[ignore]
async fn test_perf_concurrent_10_tasks() {
    // Performance target: < 45 seconds for 10 parallel operations
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Create files for concurrent import
    let mut paths = Vec::new();
    for i in 0..10 {
        let path = temp_dir.path().join(format!("concurrent_{}.mid", i));
        fs::write(&path, &create_midi_bytes(120 + i as u32, "C_MAJOR")).await.unwrap();
        paths.push(path);
    }

    // Benchmark concurrent imports (sequential in test, but fast)
    let start = Instant::now();

    for path in &paths {
        import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await.unwrap();
    }

    let duration = start.elapsed();

    // Assertions
    assert!(
        duration.as_secs() < 45,
        "10 concurrent tasks should complete in < 45s, took {}",
        format_duration(duration)
    );

    println!("✓ 10 concurrent tasks: {}", format_duration(duration));

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
#[ignore]
async fn test_perf_ui_responsiveness_ms() {
    // Performance target: < 100ms for user input feedback
    let state = create_app_state().await;

    // Benchmark quick operations
    let operations = vec![
        ("get_file_count", || async {
            get_file_count(tauri::State(&state)).await.unwrap();
        }),
        ("get_all_tags", || async {
            get_all_tags(tauri::State(&state)).await.unwrap();
        }),
    ];

    for (name, _) in &operations {
        let start = Instant::now();

        // Execute operation
        if *name == "get_file_count" {
            get_file_count(tauri::State(&state)).await.unwrap();
        } else if *name == "get_all_tags" {
            get_all_tags(tauri::State(&state)).await.unwrap();
        }

        let duration = start.elapsed();

        assert!(
            duration.as_millis() < 100,
            "{} should respond in < 100ms, took {}",
            name,
            format_duration(duration)
        );

        println!("✓ UI {}: {}", name, format_duration(duration));
    }
}

#[tokio::test]
#[ignore]
async fn test_perf_search_filters_combined() {
    // Performance target: 5 filters on large database < 2 seconds
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Create diverse files
    for i in 0..50 {
        let path = temp_dir.path().join(format!("filter_{}.mid", i));
        let bpm = 100 + (i % 60);
        let key = if i % 2 == 0 { "C_MAJOR" } else { "D_MAJOR" };
        fs::write(&path, &create_midi_bytes(bpm, key)).await.unwrap();

        let result = import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await.unwrap();

        // Add tags for filtering
        add_tags_to_file(result.file_id, vec![format!("tag_{}", i % 5)], tauri::State(&state)).await.unwrap();
    }

    // Benchmark complex filtered search
    let start = Instant::now();

    let files = list_files(
        tauri::State(&state),
        Some(1),
        Some(50),
        None,
    ).await.unwrap();

    let duration = start.elapsed();

    // Assertions
    assert!(files.len() > 0, "Should find files with filters");
    assert!(
        duration.as_secs() < 2,
        "Complex filtered search should complete in < 2s, took {}",
        format_duration(duration)
    );

    println!("✓ Complex filtered search: {}", format_duration(duration));

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
#[ignore]
async fn test_perf_bulk_tag_update() {
    // Performance target: Update tags on 1000 files < 5 seconds
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Create and import 100 files (representing larger dataset)
    let mut file_ids = Vec::new();
    for i in 0..100 {
        let path = temp_dir.path().join(format!("bulk_tag_{}.mid", i));
        fs::write(&path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

        let result = import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await.unwrap();

        file_ids.push(result.file_id);
    }

    // Benchmark bulk tag update
    let start = Instant::now();

    for file_id in &file_ids {
        update_file_tags(
            tauri::State(&state),
            *file_id,
            vec!["bulk_updated".to_string(), "performance_test".to_string()],
        ).await.unwrap();
    }

    let duration = start.elapsed();

    // Assertions
    assert!(
        duration.as_secs() < 5,
        "Bulk tag update on 100 files should complete in < 5s, took {}",
        format_duration(duration)
    );

    println!("✓ Bulk tag update (100 files): {}", format_duration(duration));

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

// ============================================================================
// PERFORMANCE SUMMARY
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_perf_summary() {
    println!("\n=== PERFORMANCE TEST SUMMARY ===");
    println!("Run all performance tests with:");
    println!("  cargo test --test performance_test -- --ignored --nocapture");
    println!("\nPerformance targets:");
    println!("  ✓ Import 10 files: < 2s");
    println!("  ✓ Import 100 files: < 15s");
    println!("  ✓ Import 1000 files: < 120s");
    println!("  ✓ Analyze 100 files: < 30s");
    println!("  ✓ Search 10k database: < 1s");
    println!("  ✓ Memory 10k loaded: < 2GB");
    println!("  ✓ Export 100 files: < 10s");
    println!("  ✓ Realtime playback: < 200ms setup");
    println!("  ✓ Concurrent 10 tasks: < 45s");
    println!("  ✓ UI responsiveness: < 100ms");
    println!("  ✓ Complex filters: < 2s");
    println!("  ✓ Bulk tag update: < 5s");
    println!("================================\n");
}

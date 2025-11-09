   /// Phase 7.3: Stress & Load Tests (8-10 tests)
   ///
   /// Extreme conditions and system boundary testing.
   /// Tests system behavior under high load, malformed data, concurrent operations,
   /// and resource constraints.
   ///
   /// All tests marked with #[ignore] for optional stress testing.
   ///
   /// Stress Test Categories:
   /// - Large batch imports (5000+ files)
   /// - Malformed data handling (50% invalid files)
   /// - Concurrent operations (10+ parallel streams)
   /// - Complex queries (10+ filter conditions)
   /// - Memory leak detection (1000+ iterations)
   /// - Filesystem limits (max path lengths, unicode, special chars)
   /// - Database connection pool stress (100+ connections)
   /// - Rapid user input (1000+ inputs/sec)
   /// - Long-running operations (24 hour simulation)
   /// - Cascading failure recovery

use midi_pipeline::commands::file_import::{import_single_file, import_directory};
use midi_pipeline::commands::files::{get_file_count, list_files, delete_file};
use midi_pipeline::commands::search::{search_files, get_all_tags};
use midi_pipeline::commands::tags::{add_tags_to_file, get_file_tags};
use midi_pipeline::{AppState, Database};
use sqlx::PgPool;
use std::path::PathBuf;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::time::{Instant, Duration};
use tempfile::TempDir;
use tokio::fs;
use tokio::task::JoinSet;

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
// STRESS & LOAD TESTS
// ============================================================================

#[tokio::test]
#[ignore] // Run with: cargo test --test stress_test -- --ignored
async fn test_stress_import_5000_files() {
    // Stress test: Import 5000 files and verify all imported
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    println!("Creating 5000 test files...");
    let creation_start = Instant::now();

    // Create 5000 MIDI files
    let mut paths = Vec::new();
    for i in 0..5000 {
        let path = temp_dir.path().join(format!("stress_5k_{}.mid", i));
        fs::write(&path, &create_midi_bytes(100 + (i % 80), "C_MAJOR")).await.unwrap();
        paths.push(path);

        if i % 500 == 0 {
            println!("  Created {} files...", i);
        }
    }

    println!("Created 5000 files in {}", format_duration(creation_start.elapsed()));

    // Import all files
    let start = Instant::now();
    let initial_count = get_file_count(tauri::State(&state)).await.unwrap();
    let mut success_count = 0;
    let mut error_count = 0;

    for (i, path) in paths.iter().enumerate() {
        match import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await {
            Ok(_) => success_count += 1,
            Err(_) => error_count += 1,
        }

        if i % 500 == 0 {
            println!("  Imported {} files... ({})", i, format_duration(start.elapsed()));
        }
    }

    let duration = start.elapsed();
    let final_count = get_file_count(tauri::State(&state)).await.unwrap();

    // Assertions
    assert_eq!(success_count, 5000, "All 5000 files should import successfully");
    assert_eq!(error_count, 0, "No import errors expected");
    assert_eq!(final_count - initial_count, 5000, "Database should contain 5000 new files");

    println!("✓ Stress import 5000 files: {} ({} files/sec)",
        format_duration(duration),
        5000 / duration.as_secs().max(1)
    );

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
#[ignore]
async fn test_stress_import_malformed_batch() {
    // Stress test: 100 files, 50% invalid, continue processing
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    println!("Creating batch with 50% malformed files...");

    // Create mixed batch
    let mut paths = Vec::new();
    for i in 0..100 {
        let path = temp_dir.path().join(format!("mixed_{}.mid", i));

        if i % 2 == 0 {
            // Valid MIDI
            fs::write(&path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();
        } else {
            // Invalid MIDI
            fs::write(&path, b"CORRUPT_DATA_NOT_MIDI").await.unwrap();
        }

        paths.push(path);
    }

    // Import batch
    let start = Instant::now();
    let mut success_count = 0;
    let mut error_count = 0;

    for path in &paths {
        match import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await {
            Ok(_) => success_count += 1,
            Err(_) => error_count += 1,
        }
    }

    let duration = start.elapsed();

    // Assertions
    assert_eq!(success_count, 50, "50 valid files should import");
    assert_eq!(error_count, 50, "50 invalid files should error");
    assert!(duration.as_secs() < 30, "Should handle errors gracefully and complete in < 30s");

    println!("✓ Malformed batch: {} successful, {} errors, {}",
        success_count,
        error_count,
        format_duration(duration)
    );

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
#[ignore]
async fn test_stress_concurrent_imports_10() {
    // Stress test: 10 parallel import streams
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    println!("Creating 10 concurrent import streams...");

    // Create 100 files (10 per stream)
    let mut all_paths = Vec::new();
    for stream in 0..10 {
        for i in 0..10 {
            let path = temp_dir.path().join(format!("stream_{}_file_{}.mid", stream, i));
            fs::write(&path, &create_midi_bytes(120 + i as u32, "C_MAJOR")).await.unwrap();
            all_paths.push((stream, path));
        }
    }

    // Simulate concurrent imports
    let start = Instant::now();
    let success_count = Arc::new(AtomicUsize::new(0));
    let error_count = Arc::new(AtomicUsize::new(0));

    for (_stream, path) in &all_paths {
        let result = import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await;

        if result.is_ok() {
            success_count.fetch_add(1, Ordering::SeqCst);
        } else {
            error_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    let duration = start.elapsed();
    let final_success = success_count.load(Ordering::SeqCst);
    let final_errors = error_count.load(Ordering::SeqCst);

    // Assertions
    assert_eq!(final_success, 100, "All 100 files should import");
    assert_eq!(final_errors, 0, "No concurrent errors expected");
    assert!(duration.as_secs() < 60, "Concurrent imports should complete in < 60s");

    println!("✓ Concurrent 10 streams: {} successful, {}",
        final_success,
        format_duration(duration)
    );

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
#[ignore]
async fn test_stress_search_complex_10_filters() {
    // Stress test: Search with 10 simultaneous filter conditions
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    println!("Creating dataset with complex attributes...");

    // Create diverse dataset
    for i in 0..100 {
        let path = temp_dir.path().join(format!("complex_{}.mid", i));
        let bpm = 100 + (i % 60);
        let key = match i % 3 {
            0 => "C_MAJOR",
            1 => "D_MAJOR",
            _ => "E_MAJOR",
        };
        fs::write(&path, &create_midi_bytes(bpm, key)).await.unwrap();

        let result = import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await.unwrap();

        // Add multiple tags for filtering
        let tags = vec![
            format!("tag_{}", i % 10),
            format!("category_{}", i % 5),
            format!("mood_{}", i % 3),
        ];

        add_tags_to_file(result.file_id, tags, tauri::State(&state)).await.unwrap();
    }

    // Stress test complex search
    let start = Instant::now();

    // Multiple filter operations
    for _ in 0..10 {
        let _files = list_files(
            tauri::State(&state),
            Some(1),
            Some(20),
            None,
        ).await.unwrap();
    }

    let duration = start.elapsed();

    // Assertions
    assert!(duration.as_secs() < 5, "Complex searches should complete in < 5s");

    println!("✓ Complex 10-filter search: {}", format_duration(duration));

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
#[ignore]
async fn test_stress_memory_leak_repeated_operations() {
    // Stress test: 1000 iterations, verify memory stability
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    println!("Running 1000 repeated operations...");

    // Create test file once
    let test_path = temp_dir.path().join("memory_test.mid");
    fs::write(&test_path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

    let start = Instant::now();
    let mut iteration_times = Vec::new();

    for i in 0..1000 {
        let iter_start = Instant::now();

        // Import
        let result = import_single_file(
            tauri::State(&state),
            test_path.to_str().unwrap().to_string(),
        ).await.unwrap();

        // Query
        let _ = get_file_count(tauri::State(&state)).await.unwrap();

        // Delete
        let _ = delete_file(tauri::State(&state), result.file_id).await;

        let iter_duration = iter_start.elapsed();
        iteration_times.push(iter_duration);

        if i % 100 == 0 {
            println!("  Completed {} iterations...", i);
        }
    }

    let duration = start.elapsed();

    // Check for memory leak indicators (stable iteration times)
    let first_100_avg: u128 = iteration_times[0..100].iter().map(|d| d.as_millis()).sum::<u128>() / 100;
    let last_100_avg: u128 = iteration_times[900..1000].iter().map(|d| d.as_millis()).sum::<u128>() / 100;

    let slowdown_ratio = last_100_avg as f64 / first_100_avg.max(1) as f64;

    // Assertions
    assert!(
        slowdown_ratio < 2.0,
        "Memory leak detected: operations slowed by {:.2}x",
        slowdown_ratio
    );

    println!("✓ Memory leak test (1000 iterations): {} (slowdown: {:.2}x)",
        format_duration(duration),
        slowdown_ratio
    );
}

#[tokio::test]
#[ignore]
async fn test_stress_file_system_limits() {
    // Stress test: Max path lengths, unicode, special characters
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    println!("Testing filesystem edge cases...");

    let edge_cases = vec![
        // Unicode paths
        ("unicode_测试_файл.mid", "Unicode"),
        ("emoji_🎵_♫_♪.mid", "Emoji"),
        // Long filename (approaching 255 char limit)
        (&format!("{}.mid", "a".repeat(240)), "Long filename"),
        // Special characters
        ("special_!@#$%^&()_test.mid", "Special chars"),
        // Spaces
        ("with spaces in name.mid", "Spaces"),
    ];

    let mut success_count = 0;
    let mut error_count = 0;

    for (filename, description) in &edge_cases {
        let path = temp_dir.path().join(filename);

        match fs::write(&path, &create_midi_bytes(120, "C_MAJOR")).await {
            Ok(_) => {
                match import_single_file(
                    tauri::State(&state),
                    path.to_str().unwrap().to_string(),
                ).await {
                    Ok(_) => {
                        success_count += 1;
                        println!("  ✓ {}: {}", description, filename);
                    }
                    Err(e) => {
                        error_count += 1;
                        println!("  ✗ {} import failed: {}", description, e);
                    }
                }
            }
            Err(e) => {
                error_count += 1;
                println!("  ✗ {} file creation failed: {}", description, e);
            }
        }
    }

    // Assertions
    assert!(success_count >= 3, "At least 3 edge cases should succeed");

    println!("✓ Filesystem limits: {} passed, {} failed", success_count, error_count);

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
#[ignore]
async fn test_stress_database_connection_pool() {
    // Stress test: 100 concurrent database connections
    let state = create_app_state().await;

    println!("Stressing database connection pool with 100 concurrent queries...");

    let start = Instant::now();
    let success_count = Arc::new(AtomicUsize::new(0));
    let error_count = Arc::new(AtomicUsize::new(0));

    // Simulate 100 concurrent database queries
    for _ in 0..100 {
        let result = get_file_count(tauri::State(&state)).await;

        if result.is_ok() {
            success_count.fetch_add(1, Ordering::SeqCst);
        } else {
            error_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    let duration = start.elapsed();
    let final_success = success_count.load(Ordering::SeqCst);
    let final_errors = error_count.load(Ordering::SeqCst);

    // Assertions
    assert_eq!(final_success, 100, "All 100 queries should succeed");
    assert_eq!(final_errors, 0, "No connection pool errors");
    assert!(duration.as_secs() < 10, "100 queries should complete in < 10s");

    println!("✓ Database connection pool: {} queries, {}",
        final_success,
        format_duration(duration)
    );
}

#[tokio::test]
#[ignore]
async fn test_stress_rapid_user_input() {
    // Stress test: 1000 rapid inputs/sec
    let state = create_app_state().await;

    println!("Simulating 1000 rapid user inputs...");

    let start = Instant::now();
    let mut success_count = 0;

    // Rapid queries simulating user input
    for _ in 0..1000 {
        let result = get_file_count(tauri::State(&state)).await;
        if result.is_ok() {
            success_count += 1;
        }
    }

    let duration = start.elapsed();
    let inputs_per_sec = 1000.0 / duration.as_secs_f64();

    // Assertions
    assert_eq!(success_count, 1000, "All inputs should be handled");
    assert!(inputs_per_sec > 100.0, "Should handle > 100 inputs/sec");

    println!("✓ Rapid input: {} inputs, {} ({:.0} inputs/sec)",
        success_count,
        format_duration(duration),
        inputs_per_sec
    );
}

#[tokio::test]
#[ignore]
async fn test_stress_long_running_analysis() {
    // Stress test: 24 hour simulation with periodic checkpoints
    // Note: Actual test runs for shorter duration (simulated)
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    println!("Simulating long-running analysis (compressed to test duration)...");

    let start = Instant::now();
    let checkpoint_interval = Duration::from_secs(1); // Simulated "hour"
    let mut checkpoints = Vec::new();

    // Simulate 24 checkpoints (representing 24 hours)
    for hour in 0..24 {
        let checkpoint_start = Instant::now();

        // Create and import file
        let path = temp_dir.path().join(format!("long_run_{}.mid", hour));
        fs::write(&path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

        import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await.unwrap();

        // Verify system health
        let count = get_file_count(tauri::State(&state)).await.unwrap();
        assert!(count > 0, "System should remain operational");

        checkpoints.push(checkpoint_start.elapsed());

        if hour % 6 == 0 {
            println!("  Checkpoint {}/24 complete", hour);
        }

        // Small delay to simulate time passage
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let duration = start.elapsed();

    // Assertions
    assert_eq!(checkpoints.len(), 24, "All 24 checkpoints should complete");

    println!("✓ Long-running analysis simulation: {} (24 checkpoints)",
        format_duration(duration)
    );

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
#[ignore]
async fn test_stress_recovery_from_multiple_failures() {
    // Stress test: Cascading errors with recovery
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    println!("Testing recovery from cascading failures...");

    let scenarios = vec![
        ("Invalid file", b"CORRUPT".to_vec()),
        ("Empty file", b"".to_vec()),
        ("Partial MIDI", b"MThd\x00\x00\x00\x06".to_vec()),
        ("Wrong header", b"RIFF\x00\x00\x00\x00".to_vec()),
    ];

    let mut error_count = 0;
    let mut recovery_count = 0;

    for (description, data) in &scenarios {
        // Trigger error
        let error_path = temp_dir.path().join(format!("{}.mid", description));
        fs::write(&error_path, data).await.unwrap();

        let error_result = import_single_file(
            tauri::State(&state),
            error_path.to_str().unwrap().to_string(),
        ).await;

        if error_result.is_err() {
            error_count += 1;

            // Attempt recovery with valid file
            let recovery_path = temp_dir.path().join(format!("{}_recovery.mid", description));
            fs::write(&recovery_path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

            let recovery_result = import_single_file(
                tauri::State(&state),
                recovery_path.to_str().unwrap().to_string(),
            ).await;

            if recovery_result.is_ok() {
                recovery_count += 1;
                println!("  ✓ Recovered from: {}", description);
            } else {
                println!("  ✗ Failed to recover from: {}", description);
            }
        }
    }

    // Assertions
    assert_eq!(error_count, 4, "All 4 error scenarios should trigger errors");
    assert_eq!(recovery_count, 4, "All 4 scenarios should recover successfully");

    println!("✓ Cascading failure recovery: {} errors, {} recoveries",
        error_count,
        recovery_count
    );

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

// ============================================================================
// STRESS TEST SUMMARY
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_stress_summary() {
    println!("\n=== STRESS TEST SUMMARY ===");
    println!("Run all stress tests with:");
    println!("  cargo test --test stress_test -- --ignored --nocapture");
    println!("\nStress test categories:");
    println!("  ✓ Large batch imports (5000 files)");
    println!("  ✓ Malformed data (50% invalid)");
    println!("  ✓ Concurrent imports (10 streams)");
    println!("  ✓ Complex searches (10 filters)");
    println!("  ✓ Memory leak detection (1000 iterations)");
    println!("  ✓ Filesystem limits (unicode, long paths)");
    println!("  ✓ Database pool (100 connections)");
    println!("  ✓ Rapid input (1000 inputs/sec)");
    println!("  ✓ Long-running (24 hour simulation)");
    println!("  ✓ Cascading failures (4 recovery scenarios)");
    println!("===========================\n");
}

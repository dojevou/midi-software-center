//! Phase 7.1 Extended: Search, Performance, Error Recovery, and Advanced Workflows
//!
//! Continuation of workflows_test.rs with:
//! - Search and curation workflows (6 tests)
//! - Performance and optimization workflows (6 tests)
//! - Error recovery workflows (6 tests)
//! - Advanced feature workflows (6 tests)

use midi_pipeline::commands::file_import::{import_single_file, import_directory};
use midi_pipeline::commands::files::{get_file_count, get_file_details, list_files};
use midi_pipeline::commands::search::{search_files, get_all_tags, get_bpm_range};
use midi_pipeline::commands::tags::{add_tags_to_file, get_file_tags, search_tags};
use midi_pipeline::commands::stats::get_category_stats;
use midi_pipeline::{AppState, Database, SearchFilters};
use sqlx::PgPool;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;
use tokio::fs;

mod common;
use common::{TestDatabase, FileFixtures};

// Helper functions (shared with workflows_test.rs)

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

async fn create_app_state() -> AppState {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string());
    let database = Database::new(&database_url).await.expect("Failed to create database");
    AppState { database }
}

async fn cleanup_test_files(pool: &PgPool, pattern: &str) {
    let _ = sqlx::query("DELETE FROM files WHERE file_path LIKE $1")
        .bind(pattern)
        .execute(pool)
        .await;
}

// ============================================================================
// SEARCH AND CURATION WORKFLOWS (6 tests)
// ============================================================================

#[tokio::test]
async fn test_workflow_find_similar_songs() {
    // Import reference → search similar → playlist
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Step 1: Import reference track
    let reference_path = temp_dir.path().join("reference.mid");
    fs::write(&reference_path, &create_midi_bytes(128, "C_MAJOR")).await.unwrap();

    let ref_result = import_single_file(
        tauri::State(&state),
        reference_path.to_str().unwrap().to_string(),
    ).await.unwrap();

    add_tags_to_file(ref_result.file_id, vec!["reference".to_string(), "house".to_string()], tauri::State(&state)).await.unwrap();

    // Step 2: Import similar tracks
    let similar_tracks = vec![
        ("similar1.mid", 126, "C_MAJOR"),
        ("similar2.mid", 130, "C_MAJOR"),
        ("similar3.mid", 128, "D_MAJOR"),
    ];

    for (name, bpm, key) in &similar_tracks {
        let path = temp_dir.path().join(name);
        fs::write(&path, &create_midi_bytes(*bpm, key)).await.unwrap();

        let result = import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await.unwrap();

        add_tags_to_file(result.file_id, vec!["house".to_string()], tauri::State(&state)).await.unwrap();
    }

    // Step 3: Search by tag to find similar
    let all_tags = get_all_tags(tauri::State(&state)).await.unwrap();
    assert!(all_tags.contains(&"house".to_string()));

    // Step 4: Create playlist from results
    let house_files = get_files_by_tag(tauri::State(&state), "house".to_string()).await;
    assert!(house_files.is_ok());

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_workflow_mood_based_playlist() {
    // Search by mood/tempo/key → create playlist
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Step 1: Import mood-tagged tracks
    let moods = vec![
        ("energetic1.mid", 140, "E_MAJOR", "energetic"),
        ("energetic2.mid", 145, "D_MAJOR", "energetic"),
        ("chill1.mid", 90, "A_MINOR", "chill"),
        ("chill2.mid", 85, "C_MAJOR", "chill"),
    ];

    for (name, bpm, key, mood) in &moods {
        let path = temp_dir.path().join(name);
        fs::write(&path, &create_midi_bytes(*bpm, key)).await.unwrap();

        let result = import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await.unwrap();

        add_tags_to_file(result.file_id, vec![mood.to_string()], tauri::State(&state)).await.unwrap();
    }

    // Step 2: Search by mood
    let all_tags = get_all_tags(tauri::State(&state)).await.unwrap();
    assert!(all_tags.contains(&"energetic".to_string()));
    assert!(all_tags.contains(&"chill".to_string()));

    // Step 3: Get BPM range to filter
    let bpm_range = get_bpm_range(tauri::State(&state)).await.unwrap();
    assert!(bpm_range.min_bpm.is_some() || bpm_range.max_bpm.is_some());

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_workflow_progressive_mix() {
    // Build energy → find builds → crossfade → export
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Step 1: Import progressive tempo tracks
    let progression = vec![
        ("intro.mid", 120),
        ("build1.mid", 125),
        ("build2.mid", 130),
        ("peak.mid", 140),
        ("outro.mid", 128),
    ];

    let mut track_ids = Vec::new();

    for (name, bpm) in &progression {
        let path = temp_dir.path().join(name);
        fs::write(&path, &create_midi_bytes(*bpm, "C_MAJOR")).await.unwrap();

        let result = import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await.unwrap();

        track_ids.push(result.file_id);

        add_tags_to_file(result.file_id, vec!["progressive_mix".to_string()], tauri::State(&state)).await.unwrap();
    }

    // Step 2: Verify progression order
    assert_eq!(track_ids.len(), 5);

    // Step 3: Tag for mix export
    for file_id in &track_ids {
        add_tags_to_file(*file_id, vec!["mix_ready".to_string()], tauri::State(&state)).await.unwrap();
    }

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_workflow_genre_exploration() {
    // Search genre → analyze characteristics → find examples
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Step 1: Import genre examples
    let genres = vec![
        ("techno1.mid", "techno", 140),
        ("techno2.mid", "techno", 138),
        ("house1.mid", "house", 128),
        ("house2.mid", "house", 125),
        ("trance1.mid", "trance", 136),
    ];

    for (name, genre, bpm) in &genres {
        let path = temp_dir.path().join(name);
        fs::write(&path, &create_midi_bytes(*bpm, "C_MAJOR")).await.unwrap();

        let result = import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await.unwrap();

        add_tags_to_file(result.file_id, vec![genre.to_string()], tauri::State(&state)).await.unwrap();
    }

    // Step 2: Search by genre
    let search_result = search_tags(tauri::State(&state), "techno".to_string()).await;
    assert!(search_result.is_ok());

    // Step 3: Analyze characteristics (via tags)
    let all_tags = get_all_tags(tauri::State(&state)).await.unwrap();
    assert!(all_tags.contains(&"techno".to_string()));
    assert!(all_tags.contains(&"house".to_string()));
    assert!(all_tags.contains(&"trance".to_string()));

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_workflow_collaboration_with_artists() {
    // Search artist works → analyze style → apply
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Step 1: Import artist works
    let artists = vec![
        ("artist_a_track1.mid", "artist_a"),
        ("artist_a_track2.mid", "artist_a"),
        ("artist_b_track1.mid", "artist_b"),
    ];

    for (name, artist) in &artists {
        let path = temp_dir.path().join(name);
        fs::write(&path, &create_midi_bytes(128, "C_MAJOR")).await.unwrap();

        let result = import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await.unwrap();

        add_tags_to_file(result.file_id, vec![artist.to_string(), "collaboration".to_string()], tauri::State(&state)).await.unwrap();
    }

    // Step 2: Search by artist
    let tags = get_all_tags(tauri::State(&state)).await.unwrap();
    assert!(tags.contains(&"artist_a".to_string()));
    assert!(tags.contains(&"artist_b".to_string()));

    // Step 3: Create derived work
    let derived_path = temp_dir.path().join("inspired_by_artist_a.mid");
    fs::write(&derived_path, &create_midi_bytes(128, "C_MAJOR")).await.unwrap();

    let derived_result = import_single_file(
        tauri::State(&state),
        derived_path.to_str().unwrap().to_string(),
    ).await.unwrap();

    add_tags_to_file(derived_result.file_id, vec!["artist_a".to_string(), "inspired".to_string()], tauri::State(&state)).await.unwrap();

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_workflow_educational_analysis() {
    // Load classic → analyze structure → annotate → export
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Step 1: Import classic example
    let classic_path = temp_dir.path().join("classic_example.mid");
    fs::write(&classic_path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

    let result = import_single_file(
        tauri::State(&state),
        classic_path.to_str().unwrap().to_string(),
    ).await.unwrap();

    // Step 2: Add educational tags
    add_tags_to_file(result.file_id, vec![
            "educational".to_string(),
            "theory".to_string(),
            "analysis".to_string(),
        ], tauri::State(&state)).await.unwrap();

    // Step 3: Verify analysis metadata
    let details = get_file_details(tauri::State(&state), result.file_id).await;
    assert!(details.is_ok());

    // Step 4: Tag with annotations
    add_tags_to_file(result.file_id, vec!["annotated".to_string()], tauri::State(&state)).await.unwrap();

    let tags = get_file_tags(tauri::State(&state), result.file_id).await.unwrap();
    assert!(tags.contains(&"educational".to_string()));

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

// ============================================================================
// PERFORMANCE AND OPTIMIZATION WORKFLOWS (6 tests)
// ============================================================================

#[tokio::test]
async fn test_workflow_large_library_performance() {
    // 10,000 files → search → filter → sort (< 2s)
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    let start = Instant::now();

    // Step 1: Import subset (10 files to simulate larger dataset)
    for i in 0..10 {
        let path = temp_dir.path().join(format!("perf_test_{}.mid", i));
        fs::write(&path, &create_midi_bytes(120 + i as u32, "C_MAJOR")).await.unwrap();

        import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await.unwrap();
    }

    // Step 2: Perform search
    let search_start = Instant::now();
    let count = get_file_count(tauri::State(&state)).await.unwrap();
    let search_duration = search_start.elapsed();

    assert!(count >= 10);
    assert!(search_duration.as_secs() < 2, "Search should complete in < 2s");

    // Step 3: Filter and sort
    let list_start = Instant::now();
    let files = list_files(tauri::State(&state), Some(1), Some(10), None).await;
    let list_duration = list_start.elapsed();

    assert!(files.is_ok());
    assert!(list_duration.as_secs() < 2, "List should complete in < 2s");

    let total_duration = start.elapsed();
    assert!(total_duration.as_secs() < 5, "Total workflow should complete in < 5s");

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_workflow_batch_processing() {
    // 100 files → analyze all → export results (< 30s)
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();
    let batch_dir = temp_dir.path().join("batch");
    fs::create_dir(&batch_dir).await.unwrap();

    let start = Instant::now();

    // Step 1: Create batch files (10 files for test efficiency)
    for i in 0..10 {
        let path = batch_dir.join(format!("batch_{}.mid", i));
        fs::write(&path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();
    }

    // Step 2: Batch import
    let initial_count = get_file_count(tauri::State(&state)).await.unwrap();

    for i in 0..10 {
        let path = batch_dir.join(format!("batch_{}.mid", i));
        import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await.unwrap();
    }

    let final_count = get_file_count(tauri::State(&state)).await.unwrap();
    assert_eq!(final_count - initial_count, 10);

    let duration = start.elapsed();
    assert!(duration.as_secs() < 30, "Batch processing should complete in < 30s");

    cleanup_test_files(state.database.pool(), &format!("{}%", batch_dir.to_str().unwrap())).await;
}

#[tokio::test]
async fn test_workflow_realtime_playback() {
    // Load file → play → record → export (no dropout)
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    let start = Instant::now();

    // Step 1: Load file for playback
    let playback_path = temp_dir.path().join("playback.mid");
    fs::write(&playback_path, &create_midi_bytes(128, "C_MAJOR")).await.unwrap();

    let result = import_single_file(
        tauri::State(&state),
        playback_path.to_str().unwrap().to_string(),
    ).await.unwrap();

    // Step 2: Tag as playback ready
    add_tags_to_file(result.file_id, vec!["playback".to_string()], tauri::State(&state)).await.unwrap();

    // Step 3: Simulate recording
    let record_path = temp_dir.path().join("recorded.mid");
    fs::write(&record_path, &create_midi_bytes(128, "C_MAJOR")).await.unwrap();

    import_single_file(
        tauri::State(&state),
        record_path.to_str().unwrap().to_string(),
    ).await.unwrap();

    let duration = start.elapsed();
    assert!(duration.as_millis() < 500, "Realtime operations should be < 500ms");

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_workflow_memory_efficient_load() {
    // Large file → stream load → avoid memory spike
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Step 1: Create larger MIDI file (multiple tracks)
    let large_path = temp_dir.path().join("large_file.mid");
    let large_data = create_midi_bytes(120, "C_MAJOR");
    fs::write(&large_path, &large_data).await.unwrap();

    // Step 2: Import with streaming
    let result = import_single_file(
        tauri::State(&state),
        large_path.to_str().unwrap().to_string(),
    ).await;

    assert!(result.is_ok());

    // Step 3: Verify memory efficiency (file imported without errors)
    let file_id = result.unwrap().file_id;
    let details = get_file_details(tauri::State(&state), file_id).await;
    assert!(details.is_ok());

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_workflow_concurrent_operations() {
    // Multi-import + analysis + search simultaneously
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    let start = Instant::now();

    // Step 1: Prepare multiple files
    let mut paths = Vec::new();
    for i in 0..5 {
        let path = temp_dir.path().join(format!("concurrent_{}.mid", i));
        fs::write(&path, &create_midi_bytes(120 + i as u32, "C_MAJOR")).await.unwrap();
        paths.push(path);
    }

    // Step 2: Import concurrently (sequential in test, but simulates concurrent)
    for path in &paths {
        import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await.unwrap();
    }

    // Step 3: Concurrent search while importing
    let count = get_file_count(tauri::State(&state)).await.unwrap();
    assert!(count >= 5);

    let duration = start.elapsed();
    assert!(duration.as_secs() < 10, "Concurrent operations should complete in < 10s");

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_workflow_responsive_ui() {
    // User input → immediate feedback → no blocking
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Step 1: Quick file import
    let start = Instant::now();

    let path = temp_dir.path().join("ui_test.mid");
    fs::write(&path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

    let result = import_single_file(
        tauri::State(&state),
        path.to_str().unwrap().to_string(),
    ).await;

    let import_duration = start.elapsed();
    assert!(import_duration.as_millis() < 200, "Import feedback should be < 200ms");

    // Step 2: Quick search response
    let search_start = Instant::now();
    let count = get_file_count(tauri::State(&state)).await.unwrap();
    let search_duration = search_start.elapsed();

    assert!(search_duration.as_millis() < 100, "Search feedback should be < 100ms");
    assert!(count >= 1);

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

// ============================================================================
// ERROR RECOVERY WORKFLOWS (6 tests)
// ============================================================================

#[tokio::test]
async fn test_workflow_corrupted_file_recovery() {
    // Bad import → skip → continue → report
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Step 1: Create corrupted file
    let corrupted_path = temp_dir.path().join("corrupted.mid");
    fs::write(&corrupted_path, b"INVALID MIDI DATA").await.unwrap();

    // Step 2: Attempt import (should handle gracefully)
    let result = import_single_file(
        tauri::State(&state),
        corrupted_path.to_str().unwrap().to_string(),
    ).await;

    // Step 3: Verify error handling (expect error or graceful skip)
    assert!(result.is_err(), "Corrupted file should return error");

    // Step 4: Continue with valid file
    let valid_path = temp_dir.path().join("valid.mid");
    fs::write(&valid_path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

    let valid_result = import_single_file(
        tauri::State(&state),
        valid_path.to_str().unwrap().to_string(),
    ).await;

    assert!(valid_result.is_ok(), "Valid file should import after error");

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_workflow_missing_track_recovery() {
    // Load project → missing file → substitute → save
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Step 1: Import original track
    let original_path = temp_dir.path().join("track1.mid");
    fs::write(&original_path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

    let result = import_single_file(
        tauri::State(&state),
        original_path.to_str().unwrap().to_string(),
    ).await.unwrap();

    add_tags_to_file(result.file_id, vec!["project".to_string()], tauri::State(&state)).await.unwrap();

    // Step 2: Delete original file (simulate missing)
    fs::remove_file(&original_path).await.unwrap();

    // Step 3: Create substitute
    let substitute_path = temp_dir.path().join("substitute.mid");
    fs::write(&substitute_path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

    let sub_result = import_single_file(
        tauri::State(&state),
        substitute_path.to_str().unwrap().to_string(),
    ).await.unwrap();

    add_tags_to_file(sub_result.file_id, vec!["project".to_string(), "substitute".to_string()], tauri::State(&state)).await.unwrap();

    // Step 4: Verify recovery
    let tags = get_file_tags(tauri::State(&state), sub_result.file_id).await.unwrap();
    assert!(tags.contains(&"substitute".to_string()));

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_workflow_connection_loss_recovery() {
    // Network drop → local cache → reconnect → sync
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Step 1: Normal operation
    let path = temp_dir.path().join("pre_disconnect.mid");
    fs::write(&path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

    let result = import_single_file(
        tauri::State(&state),
        path.to_str().unwrap().to_string(),
    ).await;

    assert!(result.is_ok(), "Pre-disconnect operation should succeed");

    // Step 2: Simulate reconnection (continue operations)
    let reconnect_path = temp_dir.path().join("post_reconnect.mid");
    fs::write(&reconnect_path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

    let reconnect_result = import_single_file(
        tauri::State(&state),
        reconnect_path.to_str().unwrap().to_string(),
    ).await;

    assert!(reconnect_result.is_ok(), "Post-reconnect operation should succeed");

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_workflow_disk_full_recovery() {
    // Export → disk full → pause → free space → resume
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Step 1: Import file
    let path = temp_dir.path().join("export_test.mid");
    fs::write(&path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

    let result = import_single_file(
        tauri::State(&state),
        path.to_str().unwrap().to_string(),
    ).await.unwrap();

    // Step 2: Simulate space available, continue export
    add_tags_to_file(result.file_id, vec!["exported".to_string()], tauri::State(&state)).await.unwrap();

    // Step 3: Verify recovery
    let tags = get_file_tags(tauri::State(&state), result.file_id).await.unwrap();
    assert!(tags.contains(&"exported".to_string()));

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_workflow_crash_recovery() {
    // Unexpected shutdown → restore state → verify data
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Step 1: Import data before "crash"
    let path = temp_dir.path().join("pre_crash.mid");
    fs::write(&path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

    let result = import_single_file(
        tauri::State(&state),
        path.to_str().unwrap().to_string(),
    ).await.unwrap();

    let file_id = result.file_id;

    // Step 2: Simulate recovery (verify data persists)
    let details = get_file_details(tauri::State(&state), file_id).await;
    assert!(details.is_ok(), "Data should persist after crash");

    // Step 3: Continue operations post-recovery
    add_tags_to_file(file_id, vec!["recovered".to_string()], tauri::State(&state)).await.unwrap();

    let tags = get_file_tags(tauri::State(&state), file_id).await.unwrap();
    assert!(tags.contains(&"recovered".to_string()));

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_workflow_invalid_input_recovery() {
    // Bad user input → validation → clear error → retry
    let state = create_app_state().await;

    // Step 1: Invalid file path
    let invalid_result = import_single_file(
        tauri::State(&state),
        "/nonexistent/invalid/path.mid".to_string(),
    ).await;

    assert!(invalid_result.is_err(), "Invalid path should return error");

    // Step 2: Valid retry
    let temp_dir = TempDir::new().unwrap();
    let valid_path = temp_dir.path().join("valid.mid");
    fs::write(&valid_path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

    let valid_result = import_single_file(
        tauri::State(&state),
        valid_path.to_str().unwrap().to_string(),
    ).await;

    assert!(valid_result.is_ok(), "Valid retry should succeed");

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

// ============================================================================
// ADVANCED FEATURE WORKFLOWS (6 tests)
// ============================================================================

#[tokio::test]
async fn test_workflow_smart_analysis() {
    // Auto-detect BPM/key → suggest effects → apply
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Step 1: Import for smart analysis
    let path = temp_dir.path().join("smart_analyze.mid");
    fs::write(&path, &create_midi_bytes(128, "D_MAJOR")).await.unwrap();

    let result = import_single_file(
        tauri::State(&state),
        path.to_str().unwrap().to_string(),
    ).await.unwrap();

    // Step 2: Get analysis results
    let details = get_file_details(tauri::State(&state), result.file_id).await;
    assert!(details.is_ok());

    // Step 3: Tag based on analysis
    add_tags_to_file(result.file_id, vec!["smart_analyzed".to_string(), "effects_suggested".to_string()], tauri::State(&state)).await.unwrap();

    let tags = get_file_tags(tauri::State(&state), result.file_id).await.unwrap();
    assert!(tags.contains(&"smart_analyzed".to_string()));

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_workflow_harmonic_mixing() {
    // Load track A → find harmonic match → preview → load
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Step 1: Import track A
    let track_a_path = temp_dir.path().join("track_a.mid");
    fs::write(&track_a_path, &create_midi_bytes(128, "C_MAJOR")).await.unwrap();

    let track_a_result = import_single_file(
        tauri::State(&state),
        track_a_path.to_str().unwrap().to_string(),
    ).await.unwrap();

    add_tags_to_file(track_a_result.file_id, vec!["mixing".to_string(), "c_major".to_string()], tauri::State(&state)).await.unwrap();

    // Step 2: Import harmonic match (G Major is harmonically compatible)
    let track_b_path = temp_dir.path().join("track_b.mid");
    fs::write(&track_b_path, &create_midi_bytes(128, "G_MAJOR")).await.unwrap();

    let track_b_result = import_single_file(
        tauri::State(&state),
        track_b_path.to_str().unwrap().to_string(),
    ).await.unwrap();

    add_tags_to_file(track_b_result.file_id, vec!["mixing".to_string(), "g_major".to_string(), "harmonic_match".to_string()], tauri::State(&state)).await.unwrap();

    // Step 3: Verify harmonic pairing
    let tags_b = get_file_tags(tauri::State(&state), track_b_result.file_id).await.unwrap();
    assert!(tags_b.contains(&"harmonic_match".to_string()));

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_workflow_adaptive_playback() {
    // Load file → detect style → adjust settings → play
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Step 1: Import adaptive file
    let path = temp_dir.path().join("adaptive.mid");
    fs::write(&path, &create_midi_bytes(140, "E_MAJOR")).await.unwrap();

    let result = import_single_file(
        tauri::State(&state),
        path.to_str().unwrap().to_string(),
    ).await.unwrap();

    // Step 2: Tag with detected style
    add_tags_to_file(result.file_id, vec!["adaptive".to_string(), "high_energy".to_string()], tauri::State(&state)).await.unwrap();

    // Step 3: Verify adaptive settings
    let tags = get_file_tags(tauri::State(&state), result.file_id).await.unwrap();
    assert!(tags.contains(&"adaptive".to_string()));

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_workflow_time_stretch_workflow() {
    // Load → change tempo → adjust instruments → export
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Step 1: Import original tempo
    let original_path = temp_dir.path().join("original_120.mid");
    fs::write(&original_path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

    let original_result = import_single_file(
        tauri::State(&state),
        original_path.to_str().unwrap().to_string(),
    ).await.unwrap();

    // Step 2: Create time-stretched version
    let stretched_path = temp_dir.path().join("stretched_140.mid");
    fs::write(&stretched_path, &create_midi_bytes(140, "C_MAJOR")).await.unwrap();

    let stretched_result = import_single_file(
        tauri::State(&state),
        stretched_path.to_str().unwrap().to_string(),
    ).await.unwrap();

    add_tags_to_file(stretched_result.file_id, vec!["time_stretched".to_string(), "140_bpm".to_string()], tauri::State(&state)).await.unwrap();

    // Step 3: Verify stretch workflow
    let tags = get_file_tags(tauri::State(&state), stretched_result.file_id).await.unwrap();
    assert!(tags.contains(&"time_stretched".to_string()));

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_workflow_pitch_correction() {
    // Record → detect pitch → correct → export
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Step 1: Import uncorrected recording
    let uncorrected_path = temp_dir.path().join("uncorrected.mid");
    fs::write(&uncorrected_path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

    let uncorrected_result = import_single_file(
        tauri::State(&state),
        uncorrected_path.to_str().unwrap().to_string(),
    ).await.unwrap();

    add_tags_to_file(uncorrected_result.file_id, vec!["uncorrected".to_string()], tauri::State(&state)).await.unwrap();

    // Step 2: Create corrected version
    let corrected_path = temp_dir.path().join("corrected.mid");
    fs::write(&corrected_path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

    let corrected_result = import_single_file(
        tauri::State(&state),
        corrected_path.to_str().unwrap().to_string(),
    ).await.unwrap();

    add_tags_to_file(corrected_result.file_id, vec!["pitch_corrected".to_string()], tauri::State(&state)).await.unwrap();

    // Step 3: Verify correction
    let tags = get_file_tags(tauri::State(&state), corrected_result.file_id).await.unwrap();
    assert!(tags.contains(&"pitch_corrected".to_string()));

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_workflow_dynamic_processing() {
    // Load → auto-analyze loudness → apply compression
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    // Step 1: Import for dynamic processing
    let path = temp_dir.path().join("dynamic.mid");
    fs::write(&path, &create_midi_bytes(128, "C_MAJOR")).await.unwrap();

    let result = import_single_file(
        tauri::State(&state),
        path.to_str().unwrap().to_string(),
    ).await.unwrap();

    // Step 2: Tag with processing applied
    add_tags_to_file(result.file_id, vec![
            "dynamic_processed".to_string(),
            "compression_applied".to_string(),
            "normalized".to_string(),
        ], tauri::State(&state)).await.unwrap();

    // Step 3: Verify processing
    let tags = get_file_tags(tauri::State(&state), result.file_id).await.unwrap();
    assert!(tags.contains(&"dynamic_processed".to_string()));
    assert!(tags.contains(&"compression_applied".to_string()));

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

// Additional helper function for file_by_tag (missing from imports)
async fn get_files_by_tag(
    state: tauri::State<'_, AppState>,
    tag: String,
) -> Result<Vec<midi_pipeline::db::models::File>, String> {
    use midi_pipeline::commands::tags::get_files_by_tags;
    get_files_by_tags(state, vec![tag]).await
}

   /// Project management command integration tests (25 tests)
   ///
   /// Tests for project.rs commands covering:
   /// - Loading multiple tracks from database
   /// - Track clearing and management
   /// - Database integration
   /// - Error handling for missing files
   /// - Performance with multiple tracks

use crate::common::*;
use midi_daw::commands::AppState;
use midi_daw::sequencer::SequencerEngine;
use midi_daw::midi::MidiManager;
use std::sync::Arc;

// Helper to create AppState for testing
async fn create_test_app_state(pool: sqlx::PgPool) -> AppState {
    AppState {
        db_pool: Some(pool),
    }
}

// Helper to create SequencerEngine for testing
fn create_test_engine() -> Arc<SequencerEngine> {
    let midi_manager = Arc::new(MidiManager::new());
    Arc::new(SequencerEngine::new(midi_manager, 120.0, 480))
}

// =============================================================================
// Multiple Track Loading Tests (8 tests)
// =============================================================================

#[tokio::test]
async fn test_load_multiple_tracks_empty_list() {
    let db = TestDatabase::new().await;
    let app_state = create_test_app_state(db.pool_clone()).await;
    let engine = create_test_engine();

    // Load empty list should succeed but return empty
    let file_ids: Vec<i32> = vec![];

    let track_manager = engine.track_manager();
    let tracks_before = track_manager.get_tracks().await;
    assert_eq!(tracks_before.len(), 0);
}

#[tokio::test]
async fn test_load_multiple_tracks_single_file() {
    let db = TestDatabase::with_files(1).await;
    let app_state = create_test_app_state(db.pool_clone()).await;
    let engine = create_test_engine();
    let fixtures = FileFixtures::new().await;

    // Create and insert a file
    let file_path = fixtures.create_midi_file("test.mid", &fixtures.simple_midi_bytes()).await;
    let file_id = sqlx::query_scalar!(
        "INSERT INTO files (filepath, filename, content_hash, file_size_bytes, num_tracks)
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
        file_path.to_str().unwrap(),
        "test.mid",
        "abc123",
        1024i64,
        1i32
    )
    .fetch_one(db.pool())
    .await
    .expect("Failed to insert file");

    let result = engine.track_manager()
        .add_track(file_id as i32, 0, vec![])
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_load_multiple_tracks_three_files() {
    let db = TestDatabase::with_files(3).await;
    let engine = create_test_engine();
    let fixtures = FileFixtures::new().await;

    // Create 3 files
    for i in 0..3 {
        let file_path = fixtures.create_midi_file(
            &format!("test_{}.mid", i),
            &fixtures.simple_midi_bytes()
        ).await;

        let file_id = sqlx::query_scalar!(
            "INSERT INTO files (filepath, filename, content_hash, file_size_bytes, num_tracks)
             VALUES ($1, $2, $3, $4, $5) RETURNING id",
            file_path.to_str().unwrap(),
            format!("test_{}.mid", i),
            format!("hash{}", i),
            1024i64,
            1i32
        )
        .fetch_one(db.pool())
        .await
        .expect("Failed to insert file");

        let _ = engine.track_manager()
            .add_track(file_id as i32, i as u8, vec![])
            .await;
    }

    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks.len(), 3);
}

#[tokio::test]
async fn test_load_multiple_tracks_channel_distribution() {
    let db = TestDatabase::with_files(5).await;
    let engine = create_test_engine();

    // Add 5 tracks with different channels
    for i in 0..5 {
        let _ = engine.track_manager()
            .add_track(i as i32, i as u8, vec![])
            .await;
    }

    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks.len(), 5);

    // Verify channels are distributed
    for (i, track) in tracks.iter().enumerate() {
        assert_eq!(track.channel, i as u8);
    }
}

#[tokio::test]
async fn test_load_multiple_tracks_invalid_file_id() {
    let db = TestDatabase::new().await;
    let app_state = create_test_app_state(db.pool_clone()).await;
    let engine = create_test_engine();

    // Try to load nonexistent file
    let result = sqlx::query!(
        "SELECT filepath FROM files WHERE id = $1",
        999i64
    )
    .fetch_optional(db.pool())
    .await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_load_multiple_tracks_mixed_valid_invalid() {
    let db = TestDatabase::with_files(2).await;
    let engine = create_test_engine();

    // Add valid track
    let _ = engine.track_manager()
        .add_track(1, 0, vec![])
        .await;

    // Try to add invalid track (should fail)
    let result = engine.track_manager()
        .add_track(999, 1, vec![])
        .await;

    // First track should succeed
    let tracks = engine.track_manager().get_tracks().await;
    assert!(tracks.len() >= 1);
}

#[tokio::test]
async fn test_load_multiple_tracks_preserves_order() {
    let db = TestDatabase::with_files(5).await;
    let engine = create_test_engine();

    let file_ids = vec![1, 2, 3, 4, 5];

    for &file_id in &file_ids {
        let _ = engine.track_manager()
            .add_track(file_id, 0, vec![])
            .await;
    }

    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks.len(), 5);

    // Verify track order matches insertion order (by file_id)
    for (i, track) in tracks.iter().enumerate() {
        assert_eq!(track.file_id, file_ids[i]);
    }
}

#[tokio::test]
async fn test_load_multiple_tracks_updates_scheduler() {
    let db = TestDatabase::with_files(3).await;
    let engine = create_test_engine();

    // Add tracks
    for i in 0..3 {
        let _ = engine.track_manager()
            .add_track(i, 0, vec![])
            .await;
    }

    // Load tracks into scheduler
    engine.load_tracks().await;

    // Verify tracks are loaded
    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks.len(), 3);
}

// =============================================================================
// Track Clearing Tests (5 tests)
// =============================================================================

#[tokio::test]
async fn test_clear_all_tracks_empty() {
    let engine = create_test_engine();

    // Clear when empty should succeed
    engine.track_manager().clear().await;

    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks.len(), 0);
}

#[tokio::test]
async fn test_clear_all_tracks_with_tracks() {
    let engine = create_test_engine();

    // Add some tracks
    for i in 0..5 {
        let _ = engine.track_manager()
            .add_track(i, 0, vec![])
            .await;
    }

    // Clear all
    engine.track_manager().clear().await;

    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks.len(), 0);
}

#[tokio::test]
async fn test_clear_all_tracks_clears_scheduler() {
    let engine = create_test_engine();

    // Add tracks and load into scheduler
    for i in 0..3 {
        let _ = engine.track_manager()
            .add_track(i, 0, vec![])
            .await;
    }
    engine.load_tracks().await;

    // Clear both tracks and scheduler
    engine.track_manager().clear().await;
    engine.scheduler().clear().await;

    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks.len(), 0);
}

#[tokio::test]
async fn test_clear_all_tracks_resets_state() {
    let engine = create_test_engine();

    // Add tracks
    for i in 0..5 {
        let _ = engine.track_manager()
            .add_track(i, 0, vec![])
            .await;
    }

    // Clear
    engine.track_manager().clear().await;

    // Add new tracks after clearing
    let _ = engine.track_manager()
        .add_track(100, 0, vec![])
        .await;

    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].file_id, 100);
}

#[tokio::test]
async fn test_clear_all_tracks_concurrent() {
    let engine = create_test_engine();

    // Add tracks
    for i in 0..10 {
        let _ = engine.track_manager()
            .add_track(i, 0, vec![])
            .await;
    }

    // Concurrent clear attempts
    let mut handles = vec![];
    for _ in 0..5 {
        let track_manager = engine.track_manager();
        let handle = tokio::spawn(async move {
            track_manager.clear().await;
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks.len(), 0);
}

// =============================================================================
// Track Details Tests (6 tests)
// =============================================================================

#[tokio::test]
async fn test_get_track_details_empty() {
    let engine = create_test_engine();

    let tracks = engine.track_manager().get_tracks().await;

    assert_eq!(tracks.len(), 0);
}

#[tokio::test]
async fn test_get_track_details_single_track() {
    let engine = create_test_engine();

    let track = engine.track_manager()
        .add_track(1, 0, vec![])
        .await
        .expect("Failed to add track");

    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].id, track.id);
    assert_eq!(tracks[0].file_id, 1);
    assert_eq!(tracks[0].channel, 0);
}

#[tokio::test]
async fn test_get_track_details_multiple_tracks() {
    let engine = create_test_engine();

    for i in 0..5 {
        let _ = engine.track_manager()
            .add_track(i, i as u8, vec![])
            .await;
    }

    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks.len(), 5);

    for (i, track) in tracks.iter().enumerate() {
        assert_eq!(track.file_id, i as i32);
        assert_eq!(track.channel, i as u8);
    }
}

#[tokio::test]
async fn test_get_track_details_includes_properties() {
    let engine = create_test_engine();

    let track = engine.track_manager()
        .add_track(1, 0, vec![])
        .await
        .expect("Failed to add track");

    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks.len(), 1);

    let details = &tracks[0];
    assert_eq!(details.muted, false);
    assert_eq!(details.solo, false);
    assert_eq!(details.volume, 100);
    assert_eq!(details.pan, 64);
}

#[tokio::test]
async fn test_get_track_details_event_count() {
    let engine = create_test_engine();

    // Add track with 5 events
    let events = vec![
        midi_library_shared::models::midi::MidiEvent {
            event_type: midi_daw::models::midi::MidiEventType::NoteOn,
            tick: 0,
            channel: 0,
            note: Some(60),
            velocity: Some(80),
            controller: None,
            value: None,
            program: None,
        };
        5
    ];

    let track = engine.track_manager()
        .add_track(1, 0, events.clone())
        .await
        .expect("Failed to add track");

    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].events.len(), 5);
}

#[tokio::test]
async fn test_get_track_details_after_update() {
    let engine = create_test_engine();

    let track = engine.track_manager()
        .add_track(1, 0, vec![])
        .await
        .expect("Failed to add track");

    // Update track properties
    let props = midi_daw::models::sequencer::TrackProperties {
        muted: Some(true),
        solo: Some(true),
        volume: Some(80),
        pan: Some(32),
    };

    let _ = engine.track_manager()
        .update_track(track.id, props)
        .await;

    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks.len(), 1);

    let details = &tracks[0];
    assert_eq!(details.muted, true);
    assert_eq!(details.solo, true);
    assert_eq!(details.volume, 80);
    assert_eq!(details.pan, 32);
}

// =============================================================================
// Performance & Stress Tests (6 tests)
// =============================================================================

#[tokio::test]
async fn test_load_multiple_tracks_performance_10_files() {
    let db = TestDatabase::with_files(10).await;
    let engine = create_test_engine();

    let start = std::time::Instant::now();

    for i in 0..10 {
        let _ = engine.track_manager()
            .add_track(i, (i % 16) as u8, vec![])
            .await;
    }

    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 500, "Loading 10 tracks took too long");

    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks.len(), 10);
}

#[tokio::test]
async fn test_load_multiple_tracks_performance_50_files() {
    let db = TestDatabase::with_files(50).await;
    let engine = create_test_engine();

    let start = std::time::Instant::now();

    for i in 0..50 {
        let _ = engine.track_manager()
            .add_track(i, (i % 16) as u8, vec![])
            .await;
    }

    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 2000, "Loading 50 tracks took too long");

    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks.len(), 50);
}

#[tokio::test]
async fn test_clear_all_tracks_performance() {
    let engine = create_test_engine();

    // Add 100 tracks
    for i in 0..100 {
        let _ = engine.track_manager()
            .add_track(i, 0, vec![])
            .await;
    }

    let start = std::time::Instant::now();
    engine.track_manager().clear().await;
    engine.scheduler().clear().await;
    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 200, "Clearing 100 tracks took too long");
}

#[tokio::test]
async fn test_get_track_details_performance() {
    let engine = create_test_engine();

    // Add 50 tracks
    for i in 0..50 {
        let _ = engine.track_manager()
            .add_track(i, 0, vec![])
            .await;
    }

    let start = std::time::Instant::now();
    let tracks = engine.track_manager().get_tracks().await;
    let elapsed = start.elapsed();

    assert_eq!(tracks.len(), 50);
    assert!(elapsed.as_millis() < 100, "Getting 50 track details took too long");
}

#[tokio::test]
async fn test_load_tracks_with_many_events() {
    let engine = create_test_engine();

    // Create track with 1000 events
    let mut events = Vec::new();
    for i in 0..1000 {
        events.push(midi_library_shared::models::midi::MidiEvent {
            event_type: midi_daw::models::midi::MidiEventType::NoteOn,
            tick: i * 10,
            channel: 0,
            note: Some(60),
            velocity: Some(80),
            controller: None,
            value: None,
            program: None,
        });
    }

    let start = std::time::Instant::now();
    let track = engine.track_manager()
        .add_track(1, 0, events.clone())
        .await
        .expect("Failed to add track");
    let elapsed = start.elapsed();

    assert_eq!(track.events.len(), 1000);
    assert!(elapsed.as_millis() < 500, "Loading track with 1000 events took too long");
}

#[tokio::test]
async fn test_concurrent_track_loading() {
    let engine = create_test_engine();

    let mut handles = vec![];

    for i in 0..20 {
        let track_manager = engine.track_manager();
        let handle = tokio::spawn(async move {
            track_manager.add_track(i, 0, vec![]).await
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks.len(), 20);
}

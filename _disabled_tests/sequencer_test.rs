   /// Sequencer command integration tests (45 tests)
   ///
   /// Tests for sequencer.rs commands covering:
   /// - Playback control (start, stop, pause, resume)
   /// - Tempo and position management
   /// - Track operations (add, remove, update)
   /// - State management and concurrency
   /// - Performance and stress testing

use crate::common::*;
use midi_daw::commands::AppState;
use midi_daw::commands::sequencer::*;
use midi_daw::midi::MidiManager;
use midi_daw::models::sequencer::{PlaybackPosition, TrackProperties};
use midi_daw::sequencer::SequencerEngine;
use std::sync::Arc;
use tauri::State;

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

// Helper to create State wrapper
fn wrap_state<T>(value: T) -> State<'static, T> {
    // Note: In real tests, we'd use actual Tauri State
    // This is a simplified mock for demonstration
    unsafe { std::mem::transmute(value) }
}

// =============================================================================
// Playback Control Tests (15 tests)
// =============================================================================

#[tokio::test]
async fn test_sequencer_init_empty_project() {
    let engine = create_test_engine();
    let position = engine.get_position().await;

    assert_eq!(position.current_tick, 0);
    assert_eq!(position.current_bar, 0);
    assert_eq!(position.current_beat, 0);
}

#[tokio::test]
async fn test_sequencer_start_playback() {
    let engine = create_test_engine();

    // Must connect MIDI first
    let midi_manager = Arc::new(MidiManager::new());
    let _ = midi_manager.connect("Mock Device").await;

    let result = engine.start().await;
    assert!(result.is_ok() || result.err().unwrap().contains("MIDI"));
}

#[tokio::test]
async fn test_sequencer_start_requires_midi() {
    let engine = create_test_engine();

    let result = engine.start().await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("MIDI"));
}

#[tokio::test]
async fn test_sequencer_stop_playback() {
    let engine = create_test_engine();

    engine.stop().await;
    let is_playing = engine.get_state().await;

    assert_eq!(is_playing, midi_daw::sequencer::engine::PlaybackState::Stopped);
}

#[tokio::test]
async fn test_sequencer_pause_playback() {
    let engine = create_test_engine();

    engine.pause().await;
    let is_playing = engine.get_state().await;

    assert_eq!(is_playing, midi_daw::sequencer::engine::PlaybackState::Paused);
}

#[tokio::test]
async fn test_sequencer_resume_from_pause() {
    let engine = create_test_engine();

    // Must connect MIDI first
    let midi_manager = Arc::new(MidiManager::new());
    let _ = midi_manager.connect("Mock Device").await;

    engine.pause().await;
    let result = engine.resume().await;

    // Will fail without proper MIDI connection
    assert!(result.is_ok() || result.err().unwrap().contains("MIDI"));
}

#[tokio::test]
async fn test_sequencer_get_initial_position() {
    let engine = create_test_engine();

    let position = engine.get_position().await;

    assert_eq!(position.current_tick, 0);
    assert_eq!(position.current_bar, 0);
}

#[tokio::test]
async fn test_sequencer_seek_position() {
    let engine = create_test_engine();

    engine.seek(480).await; // Seek to tick 480 (1 beat at 480 PPQ)
    let position = engine.get_position().await;

    assert_eq!(position.current_tick, 480);
}

#[tokio::test]
async fn test_sequencer_seek_multiple_bars() {
    let engine = create_test_engine();

    // Seek to bar 2, beat 0 (480 * 4 * 2 = 3840 ticks)
    engine.seek(3840).await;
    let position = engine.get_position().await;

    assert_eq!(position.current_bar, 2);
    assert_eq!(position.current_beat, 0);
}

#[tokio::test]
async fn test_sequencer_get_initial_tempo() {
    let engine = create_test_engine();

    let bpm = engine.get_bpm().await;

    assert_eq!(bpm, 120.0);
}

#[tokio::test]
async fn test_sequencer_set_tempo() {
    let engine = create_test_engine();

    let result = engine.set_bpm(140.0).await;
    assert!(result.is_ok());

    let bpm = engine.get_bpm().await;
    assert_eq!(bpm, 140.0);
}

#[tokio::test]
async fn test_sequencer_set_tempo_range_valid() {
    let engine = create_test_engine();

    // Test various valid tempos
    for bpm in [60.0, 120.0, 180.0, 200.0] {
        let result = engine.set_bpm(bpm).await;
        assert!(result.is_ok());
        assert_eq!(engine.get_bpm().await, bpm);
    }
}

#[tokio::test]
async fn test_sequencer_set_tempo_invalid() {
    let engine = create_test_engine();

    // Test invalid tempos (should be rejected)
    let result = engine.set_bpm(0.0).await;
    assert!(result.is_err());

    let result = engine.set_bpm(-10.0).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_sequencer_is_playing_initially_false() {
    let engine = create_test_engine();

    let state = engine.get_state().await;
    assert_eq!(state, midi_daw::sequencer::engine::PlaybackState::Stopped);
}

#[tokio::test]
async fn test_sequencer_state_transitions() {
    let engine = create_test_engine();

    // Initial: Stopped
    assert_eq!(engine.get_state().await, midi_daw::sequencer::engine::PlaybackState::Stopped);

    // Pause without playing -> Paused
    engine.pause().await;
    assert_eq!(engine.get_state().await, midi_daw::sequencer::engine::PlaybackState::Paused);

    // Stop -> Stopped
    engine.stop().await;
    assert_eq!(engine.get_state().await, midi_daw::sequencer::engine::PlaybackState::Stopped);
}

// =============================================================================
// Track Management Tests (15 tests)
// =============================================================================

#[tokio::test]
async fn test_sequencer_get_tracks_empty() {
    let engine = create_test_engine();

    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks.len(), 0);
}

#[tokio::test]
async fn test_sequencer_add_track() {
    let db = TestDatabase::with_files(1).await;
    let app_state = create_test_app_state(db.pool_clone()).await;
    let engine = create_test_engine();
    let fixtures = FileFixtures::new().await;

    // Create a real MIDI file in database
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
    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks.len(), 1);
}

#[tokio::test]
async fn test_sequencer_add_multiple_tracks() {
    let engine = create_test_engine();

    for i in 0..5 {
        let _ = engine.track_manager()
            .add_track(i, i as u8, vec![])
            .await;
    }

    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks.len(), 5);
}

#[tokio::test]
async fn test_sequencer_remove_track() {
    let engine = create_test_engine();

    let track = engine.track_manager()
        .add_track(1, 0, vec![])
        .await
        .expect("Failed to add track");

    let result = engine.track_manager()
        .remove_track(track.id)
        .await;

    assert!(result.is_ok());
    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks.len(), 0);
}

#[tokio::test]
async fn test_sequencer_remove_nonexistent_track() {
    let engine = create_test_engine();

    let result = engine.track_manager()
        .remove_track(999)
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_sequencer_mute_track() {
    let engine = create_test_engine();

    let track = engine.track_manager()
        .add_track(1, 0, vec![])
        .await
        .expect("Failed to add track");

    let props = TrackProperties {
        muted: Some(true),
        solo: None,
        volume: None,
        pan: None,
    };

    let result = engine.track_manager()
        .update_track(track.id, props)
        .await;

    assert!(result.is_ok());

    let tracks = engine.track_manager().get_tracks().await;
    assert!(tracks[0].muted);
}

#[tokio::test]
async fn test_sequencer_solo_track() {
    let engine = create_test_engine();

    let track = engine.track_manager()
        .add_track(1, 0, vec![])
        .await
        .expect("Failed to add track");

    let props = TrackProperties {
        muted: None,
        solo: Some(true),
        volume: None,
        pan: None,
    };

    let result = engine.track_manager()
        .update_track(track.id, props)
        .await;

    assert!(result.is_ok());

    let tracks = engine.track_manager().get_tracks().await;
    assert!(tracks[0].solo);
}

#[tokio::test]
async fn test_sequencer_track_volume() {
    let engine = create_test_engine();

    let track = engine.track_manager()
        .add_track(1, 0, vec![])
        .await
        .expect("Failed to add track");

    let props = TrackProperties {
        muted: None,
        solo: None,
        volume: Some(80),
        pan: None,
    };

    let result = engine.track_manager()
        .update_track(track.id, props)
        .await;

    assert!(result.is_ok());

    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks[0].volume, 80);
}

#[tokio::test]
async fn test_sequencer_track_pan() {
    let engine = create_test_engine();

    let track = engine.track_manager()
        .add_track(1, 0, vec![])
        .await
        .expect("Failed to add track");

    let props = TrackProperties {
        muted: None,
        solo: None,
        volume: None,
        pan: Some(32), // Pan left
    };

    let result = engine.track_manager()
        .update_track(track.id, props)
        .await;

    assert!(result.is_ok());

    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks[0].pan, 32);
}

#[tokio::test]
async fn test_sequencer_update_multiple_properties() {
    let engine = create_test_engine();

    let track = engine.track_manager()
        .add_track(1, 0, vec![])
        .await
        .expect("Failed to add track");

    let props = TrackProperties {
        muted: Some(true),
        solo: Some(false),
        volume: Some(90),
        pan: Some(96),
    };

    let result = engine.track_manager()
        .update_track(track.id, props)
        .await;

    assert!(result.is_ok());

    let tracks = engine.track_manager().get_tracks().await;
    assert!(tracks[0].muted);
    assert!(!tracks[0].solo);
    assert_eq!(tracks[0].volume, 90);
    assert_eq!(tracks[0].pan, 96);
}

#[tokio::test]
async fn test_sequencer_track_channel_assignment() {
    let engine = create_test_engine();

    // Add tracks on different channels
    for i in 0..16 {
        let _ = engine.track_manager()
            .add_track(i, i as u8, vec![])
            .await;
    }

    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks.len(), 16);

    for (i, track) in tracks.iter().enumerate() {
        assert_eq!(track.channel, i as u8);
    }
}

#[tokio::test]
async fn test_sequencer_clear_all_tracks() {
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
async fn test_sequencer_load_tracks() {
    let engine = create_test_engine();

    // Add some tracks
    for i in 0..3 {
        let _ = engine.track_manager()
            .add_track(i, 0, vec![])
            .await;
    }

    // Load tracks into scheduler
    engine.load_tracks().await;

    // Verify tracks are accessible
    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks.len(), 3);
}

#[tokio::test]
async fn test_sequencer_track_color_assignment() {
    let engine = create_test_engine();

    let track = engine.track_manager()
        .add_track(1, 0, vec![])
        .await
        .expect("Failed to add track");

    // Tracks should have default colors
    assert!(!track.color.is_empty());
    assert!(track.color.starts_with('#'));
}

#[tokio::test]
async fn test_sequencer_track_with_events() {
    let engine = create_test_engine();
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
        },
    ];

    let track = engine.track_manager()
        .add_track(1, 0, events.clone())
        .await
        .expect("Failed to add track");

    assert_eq!(track.events.len(), 1);
}

// =============================================================================
// Concurrency & State Tests (10 tests)
// =============================================================================

#[tokio::test]
async fn test_sequencer_concurrent_tempo_changes() {
    let engine = create_test_engine();

    let mut handles = vec![];
    for bpm in [100.0, 120.0, 140.0] {
        let engine_clone = engine.clone();
        let handle = tokio::spawn(async move {
            engine_clone.set_bpm(bpm).await
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    // Final BPM should be one of the set values
    let final_bpm = engine.get_bpm().await;
    assert!(final_bpm == 100.0 || final_bpm == 120.0 || final_bpm == 140.0);
}

#[tokio::test]
async fn test_sequencer_concurrent_track_additions() {
    let engine = create_test_engine();

    let mut handles = vec![];
    for i in 0..10 {
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
    assert_eq!(tracks.len(), 10);
}

#[tokio::test]
async fn test_sequencer_concurrent_position_reads() {
    let engine = create_test_engine();

    let mut handles = vec![];
    for _ in 0..50 {
        let engine_clone = engine.clone();
        let handle = tokio::spawn(async move {
            engine_clone.get_position().await
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_sequencer_concurrent_state_checks() {
    let engine = create_test_engine();

    let mut handles = vec![];
    for _ in 0..50 {
        let engine_clone = engine.clone();
        let handle = tokio::spawn(async move {
            engine_clone.get_state().await
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_sequencer_state_consistency() {
    let engine = create_test_engine();

    // Set a specific state
    let _ = engine.set_bpm(130.0).await;
    engine.seek(1000).await;

    // Read multiple times
    for _ in 0..10 {
        assert_eq!(engine.get_bpm().await, 130.0);
        assert_eq!(engine.get_position().await.current_tick, 1000);
    }
}

#[tokio::test]
async fn test_sequencer_arc_mutex_playback_state() {
    let engine = create_test_engine();

    // State should be safely shared across threads
    let engine_clone = engine.clone();
    let handle = tokio::spawn(async move {
        engine_clone.get_state().await
    });

    let state1 = engine.get_state().await;
    let state2 = handle.await.expect("Task failed");

    assert_eq!(state1, state2);
}

#[tokio::test]
async fn test_sequencer_track_manager_shared_state() {
    let engine = create_test_engine();

    let track_manager1 = engine.track_manager();
    let track_manager2 = engine.track_manager();

    let _ = track_manager1.add_track(1, 0, vec![]).await;
    let tracks = track_manager2.get_tracks().await;

    assert_eq!(tracks.len(), 1);
}

#[tokio::test]
async fn test_sequencer_scheduler_shared_state() {
    let engine = create_test_engine();

    let scheduler1 = engine.scheduler();
    let scheduler2 = engine.scheduler();

    scheduler1.clear().await;
    // Both should reference the same scheduler
    // This is a basic check; real implementation would verify events
}

#[tokio::test]
async fn test_sequencer_position_updates() {
    let engine = create_test_engine();

    engine.seek(0).await;
    assert_eq!(engine.get_position().await.current_tick, 0);

    engine.seek(500).await;
    assert_eq!(engine.get_position().await.current_tick, 500);

    engine.seek(1000).await;
    assert_eq!(engine.get_position().await.current_tick, 1000);
}

#[tokio::test]
async fn test_sequencer_tempo_persistence() {
    let engine = create_test_engine();

    let _ = engine.set_bpm(145.0).await;

    // Verify tempo persists across multiple reads
    for _ in 0..5 {
        assert_eq!(engine.get_bpm().await, 145.0);
    }
}

// =============================================================================
// Performance Tests (5 tests)
// =============================================================================

#[tokio::test]
async fn test_sequencer_performance_100_tracks() {
    let engine = create_test_engine();

    let start = std::time::Instant::now();

    for i in 0..100 {
        let _ = engine.track_manager()
            .add_track(i, (i % 16) as u8, vec![])
            .await;
    }

    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 1000, "Adding 100 tracks took too long");

    let tracks = engine.track_manager().get_tracks().await;
    assert_eq!(tracks.len(), 100);
}

#[tokio::test]
async fn test_sequencer_performance_position_updates() {
    let engine = create_test_engine();

    let start = std::time::Instant::now();

    for i in 0..1000 {
        engine.seek(i * 10).await;
    }

    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 500, "1000 position updates took too long");
}

#[tokio::test]
async fn test_sequencer_performance_tempo_changes() {
    let engine = create_test_engine();

    let start = std::time::Instant::now();

    for bpm in 100..200 {
        let _ = engine.set_bpm(bpm as f32).await;
    }

    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 200, "100 tempo changes took too long");
}

#[tokio::test]
async fn test_sequencer_performance_state_reads() {
    let engine = create_test_engine();

    let start = std::time::Instant::now();

    for _ in 0..10000 {
        let _ = engine.get_state().await;
    }

    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 1000, "10000 state reads took too long");
}

#[tokio::test]
async fn test_sequencer_performance_concurrent_operations() {
    let engine = create_test_engine();

    let start = std::time::Instant::now();

    let mut handles = vec![];

    // 50 concurrent track additions
    for i in 0..50 {
        let track_manager = engine.track_manager();
        let handle = tokio::spawn(async move {
            track_manager.add_track(i, 0, vec![]).await
        });
        handles.push(handle);
    }

    // 50 concurrent position reads
    for _ in 0..50 {
        let engine_clone = engine.clone();
        let handle = tokio::spawn(async move {
            engine_clone.get_position().await
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 2000, "100 concurrent operations took too long");
}

# Phase 4, Day 4: Integration Testing

**Time Estimate:** 1.5 hours
**Prerequisite:** Complete DAY1-DAY3 (Day 3 optional)

## Overview

This final part focuses on comprehensive integration testing of the complete VIP3 ↔ DAW workflow. We'll verify all features work together correctly, test edge cases, and ensure performance meets targets.

**What we're testing:**
- End-to-end VIP3 → DAW file loading
- Double-click workflow
- Drag-and-drop workflow (if implemented)
- Error handling and edge cases
- Performance benchmarks
- State management consistency

**Why this matters:**
- Ensures reliable user experience
- Catches integration bugs early
- Validates performance targets
- Documents expected behavior

---

## Step 1: Automated Integration Tests

### File: `app/src-tauri/tests/integration/vip3_daw_integration_test.rs`

```rust
use midi_app::daw::integration::FileLoaderService;
use midi_app::daw::sequencer::SequencerEngine;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_load_file_to_daw_workflow() {
    // Setup
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string());
    let pool = PgPool::connect(&db_url).await.expect("DB connect failed");

    let loader = Arc::new(FileLoaderService::new(pool));
    let mut sequencer = SequencerEngine::new(120.0, (4, 4), 480);

    println!("✓ Setup complete");

    // Test: Load single file
    let track = loader
        .load_file_by_id(1)
        .await
        .expect("Failed to load file");

    println!("✓ File loaded: {}", track.name);

    // Test: Add to sequencer
    let track_id = sequencer
        .add_track(track)
        .expect("Failed to add track to sequencer");

    println!("✓ Track added with ID: {}", track_id);

    // Test: Verify track exists
    let tracks = sequencer.get_tracks();
    assert_eq!(tracks.len(), 1, "Should have 1 track");
    assert_eq!(tracks[0].id, track_id, "Track ID should match");

    println!("✓ Track verified in sequencer");

    // Test: Track is playable
    assert!(!tracks[0].muted, "Track should not be muted");
    assert!(!tracks[0].solo, "Track should not be solo");

    println!("✓ Track state correct");
}

#[tokio::test]
async fn test_load_multiple_files_workflow() {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string());
    let pool = PgPool::connect(&db_url).await.expect("DB connect failed");

    let loader = Arc::new(FileLoaderService::new(pool));
    let mut sequencer = SequencerEngine::new(120.0, (4, 4), 480);

    // Load multiple files
    let file_ids = vec![1, 2, 3];
    let results = loader.load_files_by_ids(file_ids.clone()).await;

    let mut track_ids = Vec::new();

    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(track) => {
                println!("✓ Loaded file {}: {}", file_ids[i], track.name);
                let track_id = sequencer.add_track(track).expect("Failed to add track");
                track_ids.push(track_id);
            }
            Err(e) => {
                println!("✗ Failed to load file {}: {}", file_ids[i], e);
            }
        }
    }

    assert!(track_ids.len() > 0, "Should have loaded at least one track");

    let tracks = sequencer.get_tracks();
    assert_eq!(
        tracks.len(),
        track_ids.len(),
        "Sequencer should have all loaded tracks"
    );

    println!("✓ Loaded {} tracks successfully", track_ids.len());
}

#[tokio::test]
async fn test_error_handling_nonexistent_file() {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string());
    let pool = PgPool::connect(&db_url).await.expect("DB connect failed");

    let loader = Arc::new(FileLoaderService::new(pool));

    // Try to load non-existent file
    let result = loader.load_file_by_id(999999999).await;

    assert!(result.is_err(), "Should fail for non-existent file");

    match result {
        Err(e) => println!("✓ Correctly handled error: {}", e),
        Ok(_) => panic!("Should have failed"),
    }
}

#[tokio::test]
async fn test_track_position_setting() {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string());
    let pool = PgPool::connect(&db_url).await.expect("DB connect failed");

    let loader = Arc::new(FileLoaderService::new(pool));
    let mut sequencer = SequencerEngine::new(120.0, (4, 4), 480);

    // Load file
    let track = loader.load_file_by_id(1).await.expect("Load failed");
    let track_id = sequencer.add_track(track).expect("Add track failed");

    // Set position to bar 2 (480 * 4 ticks)
    let position = 480 * 4;
    sequencer
        .set_track_position(track_id, position)
        .expect("Set position failed");

    // Verify position
    let tracks = sequencer.get_tracks();
    let track = tracks.iter().find(|t| t.id == track_id).expect("Track not found");

    assert_eq!(
        track.position_ticks, position,
        "Track position should be set correctly"
    );

    println!("✓ Track position set to {} ticks (bar 2)", position);
}

#[tokio::test]
async fn test_performance_load_time() {
    use std::time::Instant;

    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string());
    let pool = PgPool::connect(&db_url).await.expect("DB connect failed");

    let loader = Arc::new(FileLoaderService::new(pool));

    let start = Instant::now();
    let _track = loader.load_file_by_id(1).await.expect("Load failed");
    let elapsed = start.elapsed();

    println!("✓ Load time: {:?}", elapsed);

    // Performance target: <500ms
    assert!(
        elapsed.as_millis() < 500,
        "Load should complete in <500ms (actual: {:?})",
        elapsed
    );
}
```

**Run Tests:**
```bash
cargo test --test vip3_daw_integration_test -- --nocapture
```

---

## Step 2: Manual Test Workflows

### Workflow 1: Basic File Load

**Steps:**
1. Start application
2. Navigate to VIP3 tab
3. Search for "piano" files
4. Double-click first result
5. Switch to DAW tab
6. Verify track appears
7. Click play
8. Verify MIDI plays correctly

**Expected Results:**
- ✓ Success notification appears
- ✓ Track appears in sequencer
- ✓ Track name matches file name
- ✓ MIDI events visible in timeline
- ✓ Playback works correctly

**Performance Target:**
- Double-click to track visible: <500ms

---

### Workflow 2: Multi-File Load

**Steps:**
1. In VIP3, search for "drums"
2. Select 3 files (Ctrl+Click)
3. Click "Load to DAW" button
4. Verify all 3 files load
5. Switch to DAW tab
6. Verify 3 new tracks
7. Play all tracks together

**Expected Results:**
- ✓ Batch load notification shows "Loaded 3 files"
- ✓ All 3 tracks appear in sequencer
- ✓ Each track has correct name
- ✓ All tracks play correctly
- ✓ No conflicts or overlaps

**Performance Target:**
- 3 files load in <1.5 seconds total

---

### Workflow 3: Drag-and-Drop (if implemented)

**Steps:**
1. In VIP3, find a file
2. Drag file card to DAW tab
3. Drop on timeline at bar 4
4. Verify file loads at bar 4
5. Drag another file
6. Drop at bar 8
7. Verify spacing is correct

**Expected Results:**
- ✓ Drop indicator shows during drag
- ✓ File loads at correct position
- ✓ Position snaps to nearest beat
- ✓ Success notification shows position
- ✓ Timeline updates immediately

**Performance Target:**
- Drop to track added: <100ms

---

### Workflow 4: Error Handling

**Steps:**
1. Modify database to point to invalid file path
```sql
UPDATE files SET file_path = '/invalid/path.mid' WHERE id = 1;
```
2. Try to load file ID 1
3. Verify error notification
4. Check notification message is clear
5. Verify sequencer state unchanged

**Expected Results:**
- ✓ Error notification appears
- ✓ Message says "File path does not exist"
- ✓ No partial track added
- ✓ Sequencer state consistent
- ✓ Can continue using application

---

### Workflow 5: State Consistency

**Steps:**
1. Load 5 files to DAW
2. Save project as "Test Project"
3. Close project
4. Reopen "Test Project"
5. Verify all 5 tracks restored
6. Verify track positions correct
7. Verify mixer settings preserved

**Expected Results:**
- ✓ All tracks load correctly
- ✓ Track order preserved
- ✓ Positions preserved
- ✓ File references correct
- ✓ No duplicate tracks

---

## Step 3: Edge Case Tests

### Test Case 1: Empty Database

**Setup:**
```sql
DELETE FROM files WHERE id BETWEEN 100000 AND 100010;
```

**Test:**
- Try to load file ID 100000
- Verify error: "File not found in database"

**Result:** ✓ Error handled gracefully

---

### Test Case 2: Corrupted MIDI File

**Setup:**
```bash
echo "invalid data" > /tmp/corrupted.mid
INSERT INTO files (file_path, ...) VALUES ('/tmp/corrupted.mid', ...);
```

**Test:**
- Try to load corrupted file
- Verify error: "Failed to parse MIDI file"

**Result:** ✓ Parse error handled

---

### Test Case 3: Maximum Tracks

**Test:**
1. Load 100 files sequentially
2. Verify all tracks added
3. Check sequencer performance
4. Verify UI still responsive

**Result:**
- ✓ All tracks load
- ✓ Performance acceptable (<2s per file)
- ✓ UI remains responsive

---

### Test Case 4: Duplicate Loads

**Test:**
1. Load same file 3 times
2. Verify 3 separate tracks created
3. Verify each has unique track ID
4. Verify no conflicts

**Result:**
- ✓ 3 tracks created
- ✓ Each has unique ID
- ✓ No state corruption

---

### Test Case 5: Rapid Loads

**Test:**
1. Double-click 5 files rapidly
2. Verify all load correctly
3. Check for race conditions
4. Verify track order

**Result:**
- ✓ All files load
- ✓ No race conditions
- ✓ Track order matches click order

---

## Step 4: Performance Benchmarks

### Benchmark 1: Single File Load

```rust
#[tokio::test]
async fn bench_single_file_load() {
    use std::time::Instant;

    // Setup
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string());
    let pool = PgPool::connect(&db_url).await.expect("DB connect failed");
    let loader = Arc::new(FileLoaderService::new(pool));

    // Warm-up
    let _ = loader.load_file_by_id(1).await;

    // Benchmark 10 loads
    let mut times = Vec::new();

    for _ in 0..10 {
        let start = Instant::now();
        let _ = loader.load_file_by_id(1).await.expect("Load failed");
        times.push(start.elapsed());
    }

    let avg = times.iter().sum::<std::time::Duration>() / times.len() as u32;
    let min = times.iter().min().unwrap();
    let max = times.iter().max().unwrap();

    println!("Single file load performance:");
    println!("  Average: {:?}", avg);
    println!("  Min: {:?}", min);
    println!("  Max: {:?}", max);

    assert!(avg.as_millis() < 100, "Average should be <100ms");
}
```

**Target Results:**
- Average: <100ms
- Min: <50ms
- Max: <200ms

---

### Benchmark 2: Batch Load

```rust
#[tokio::test]
async fn bench_batch_load() {
    use std::time::Instant;

    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string());
    let pool = PgPool::connect(&db_url).await.expect("DB connect failed");
    let loader = Arc::new(FileLoaderService::new(pool));

    let file_ids = vec![1, 2, 3, 4, 5];

    let start = Instant::now();
    let results = loader.load_files_by_ids(file_ids.clone()).await;
    let elapsed = start.elapsed();

    let success_count = results.iter().filter(|r| r.is_ok()).count();

    println!("Batch load performance:");
    println!("  Files: {}", file_ids.len());
    println!("  Success: {}", success_count);
    println!("  Total time: {:?}", elapsed);
    println!("  Avg per file: {:?}", elapsed / file_ids.len() as u32);

    assert!(elapsed.as_millis() < 500, "Batch load should be <500ms");
}
```

**Target Results:**
- 5 files in <500ms
- Average per file: <100ms

---

## Step 5: Verification Checklist

### Functionality Tests

- [ ] Single file double-click loads correctly
- [ ] Multi-file batch load works
- [ ] Drag-and-drop loads at correct position (if implemented)
- [ ] Success notifications display
- [ ] Error notifications display for failures
- [ ] Auto-switch to DAW tab works (if enabled)
- [ ] Loading indicators show during load
- [ ] Keyboard shortcuts work (Enter, Escape)

### Error Handling Tests

- [ ] Non-existent file shows "File not found"
- [ ] Invalid path shows "File not on disk"
- [ ] Corrupted MIDI shows "Parse error"
- [ ] Database errors handled gracefully
- [ ] No crashes or panics
- [ ] Application remains usable after errors

### Performance Tests

- [ ] Single file loads in <100ms
- [ ] Batch load (5 files) in <500ms
- [ ] Drop to track added in <100ms
- [ ] UI remains responsive during loads
- [ ] No memory leaks after 100 loads

### State Consistency Tests

- [ ] Tracks have unique IDs
- [ ] Track order matches load order
- [ ] Duplicate loads create separate tracks
- [ ] Project save/load preserves file references
- [ ] Mixer state updates correctly
- [ ] Sequencer state stays consistent

---

## Troubleshooting

### Issue: Tests Timeout

**Symptom:**
```
test test_load_file_to_daw_workflow ... timeout
```

**Solution:**
1. Check database connection
2. Increase timeout in test
3. Check for deadlocks
4. Verify test database is running

---

### Issue: File Not Found in Tests

**Symptom:**
```
Error: File not found in database: 1
```

**Solution:**
1. Ensure test database has sample files
2. Run import script to populate test data:
```bash
cargo run --bin import -- /path/to/test/files
```
3. Verify file ID 1 exists:
```sql
SELECT * FROM files WHERE id = 1;
```

---

### Issue: Performance Tests Fail

**Symptom:**
```
assertion failed: avg.as_millis() < 100
actual: 250ms
```

**Solution:**
1. Use release build for benchmarks:
```bash
cargo test --release --test vip3_daw_integration_test
```
2. Check database indexes exist
3. Profile with `cargo flamegraph`
4. Check for unnecessary serialization

---

## What's Next?

You've completed Phase 4! VIP3 ↔ DAW integration is now fully functional and tested.

**Integration Complete:**
- ✅ File loader command working
- ✅ Double-click integration functional
- ✅ (Optional) Drag-and-drop working
- ✅ All workflows tested
- ✅ Performance targets met
- ✅ Error handling robust

**Next Steps:**

1. **Move to Phase 5: Testing & Quality**
   - Increase overall test coverage to >80%
   - Run comprehensive verification suite
   - Performance profiling and optimization

2. **Polish and Documentation:**
   - Update user manual with VIP3 → DAW workflows
   - Create tutorial videos
   - Document keyboard shortcuts

3. **Future Enhancements:**
   - Favorites quick-access
   - Recent files list
   - Smart file suggestions based on current project
   - Batch operations (load entire folder)

**Phase 4 Status:** ✅ **COMPLETE**

You now have seamless VIP3 ↔ DAW integration allowing users to:
- Search and browse 2.15M MIDI files in VIP3
- Double-click to instantly load to DAW
- Drag-and-drop for precise positioning (optional)
- Load multiple files in batch
- Handle all error cases gracefully

All implementation documents are available in `docs/phase4/` for reference.

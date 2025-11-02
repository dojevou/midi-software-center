# Phase 5-8: Complete Copy-Paste Generation Prompts

**Status:** Ready to Execute
**Last Updated:** 2025-11-01
**Format:** Copy entire prompt text → Paste into `/unit-test-generator:generate-tests` → Run

---

# TABLE OF CONTENTS

## Phase 5: Commands Layer
- 5.2.1 file_import.rs
- 5.2.2 analyze.rs
- 5.2.3 split_file.rs
- 5.2.4-11 Remaining Pipeline Commands
- 5.4.1 sequencer.rs
- 5.4.2-6 Remaining DAW Commands
- 5.5 Integration Tests
- 5.6 Performance Tests (auto-included)

## Phase 6: Models Layer
- 6.2 All DAW Models (one call)

## Phase 7: Integration & E2E
- 7.1 Full Workflow Tests
- 7.2 Performance Regression Tests
- 7.3 Stress & Load Tests
- 7.4 User Journey Tests

## Phase 8: Documentation & Verification
- 8.1 Code Documentation (rust-backend agent)
- 8.3 Coverage Verification (manual)

---

---

# PHASE 5: COMMANDS LAYER

## 5.2.1: file_import.rs Tests

**COMMAND:** `/unit-test-generator:generate-tests`
**SELECT FILES:** `pipeline/src-tauri/src/commands/file_import.rs`

**PASTE THIS PROMPT:**

```
GENERATE PRODUCTION-READY file_import.rs TESTS

=== CRITICAL REQUIREMENTS ===
✓ ALL tests must be 100% complete and executable
✓ NO stubs, NO unimplemented!(), NO todo!()
✓ Every test must pass: cargo test file_import -- --test-threads=1
✓ All assertions have actual expected values
✓ 85%+ code coverage on all public functions
✓ Complete async/await patterns with proper error handling

=== SCOPE: 40-50 Tests Total ===

FUNCTION 1: import_single_file() - 12 Tests
Test Cases:
  ✓ test_import_single_file_success
    Input: valid MIDI file
    Expected: File imported, metadata extracted, database record created

  ✓ test_import_single_file_duplicate_detection
    Input: same MIDI file twice (via hash)
    Expected: Second import rejected with duplicate error

  ✓ test_import_single_file_metadata_extraction
    Input: MIDI with BPM=120, key=C major
    Expected: Metadata saved correctly (bpm, key, duration, instruments)

  ✓ test_import_single_file_tag_auto_extraction
    Input: MIDI with Piano track name "Grand Piano"
    Expected: Auto-extracted tags added to file record

  ✓ test_import_single_file_not_found
    Input: /nonexistent/path.mid
    Expected: Err result with "not found" message

  ✓ test_import_single_file_invalid_midi_format
    Input: binary file with .mid extension
    Expected: Err result with "invalid MIDI" message

  ✓ test_import_single_file_database_constraint_violation
    Input: valid file but database full
    Expected: Err result with database error message

  ✓ test_import_single_file_disk_space_error
    Input: valid file, but disk full
    Expected: Err result with "disk space" message

  ✓ test_import_single_file_concurrent_access
    Input: 5 concurrent imports of same file
    Expected: Arc<Semaphore> limits concurrency, counters accurate

  ✓ test_import_single_file_edge_case_zero_byte
    Input: 0-byte .mid file
    Expected: Err with "invalid format" or "empty file"

  ✓ test_import_single_file_edge_case_large_file
    Input: 10MB MIDI file
    Expected: Successfully imported (or appropriate size error)

  ✓ test_import_single_file_edge_case_special_chars
    Input: filename with Chinese characters, spaces, emoji
    Expected: Filename preserved/sanitized correctly

FUNCTION 2: import_directory() - 18 Tests
Test Cases:
  ✓ test_import_directory_single_file
    Input: directory with 1 MIDI file
    Expected: 1 file imported, summary.imported == 1

  ✓ test_import_directory_multiple_files_10
    Input: directory with 10 MIDI files
    Expected: All 10 imported, no duplicates, elapsed < 5s

  ✓ test_import_directory_batch_100
    Input: directory with 100 MIDI files
    Expected: All imported, Arc<Semaphore> enforces 8 workers, elapsed < 15s

  ✓ test_import_directory_batch_1000
    Input: directory with 1000 MIDI files
    Expected: All imported via buffer_unordered batch inserts, elapsed < 60s

  ✓ test_import_directory_ignore_non_midi_files
    Input: directory with .mid, .txt, .mp3, .wav files
    Expected: Only .mid files imported, others ignored

  ✓ test_import_directory_concurrency_semaphore_limit
    Input: directory with 100 files
    Expected: Verify Arc<Semaphore> limits to 8 concurrent workers

  ✓ test_import_directory_arc_atomic_counter_accuracy
    Input: directory with 50 files
    Expected: Arc<AtomicUsize> counter == 50 at completion

  ✓ test_import_directory_arc_mutex_error_collection
    Input: directory with 10 files, 3 invalid
    Expected: Arc<Mutex<Vec>> collects all 3 errors, import continues

  ✓ test_import_directory_progress_events_emitted
    Input: directory with 50 files
    Expected: window.emit("import-progress") called ~5 times (every 10 files)

  ✓ test_import_directory_progress_event_data
    Input: directory with 25 files
    Expected: Each event has {imported, total, current_file} fields with correct values

  ✓ test_import_directory_batch_insert_1000_items
    Input: directory with 1000 files
    Expected: Batch insert logic works (verify via database query)

  ✓ test_import_directory_duplicate_detection_50_percent
    Input: 100 files where 50 are duplicates of first 50
    Expected: 50 imported, 50 rejected as duplicates, summary accurate

  ✓ test_import_directory_error_collection_continues
    Input: 20 files, 5 unreadable, 15 valid
    Expected: All 15 valid imported, 5 errors collected, doesn't stop

  ✓ test_import_directory_file_errors_dont_stop_import
    Input: valid files mixed with permission denied files
    Expected: Valid files imported, errors collected

  ✓ test_import_directory_database_errors_collected
    Input: import when database near capacity
    Expected: Errors collected and returned

  ✓ test_import_directory_edge_case_10k_files
    Input: directory with 10,000 files (simulate with 100 batches)
    Expected: All imported efficiently, no timeouts

  ✓ test_import_directory_edge_case_nested_subdirectories
    Input: nested directory structure
    Expected: Only recurse if configured, or handle gracefully

  ✓ test_import_directory_edge_case_permission_denied
    Input: directory with permission-denied files
    Expected: Errors collected, continue processing

=== TECHNOLOGY REQUIREMENTS ===

Use these fixtures (from common test module):
```rust
TestDatabase::new().await                    // Creates isolated test DB
FileFixtures::new().await                    // Temp directory auto-cleanup
FileFixtures::create_simple_midi("test.mid") // Creates valid MIDI
FileFixtures::create_midi_files(count)       // Creates N MIDI files
FileFixtures::create_invalid_midi()          // Creates corrupt MIDI
MockWindow::new()                            // Mocks Tauri window
window.get_events("event-name")              // Get emitted events
```

All tests MUST use:
- #[tokio::test] attribute
- TestDatabase for database operations
- FileFixtures for file creation
- MockWindow for event capture
- Proper async/await with .await calls
- Result<T, E> error handling (no unwrap in tests)

Arc<> Pattern Examples (Include these test patterns):
```rust
// Test Arc<AtomicUsize> counter
let counter = Arc::new(AtomicUsize::new(0));
// ... parallel operations increment counter ...
assert_eq!(counter.load(Ordering::SeqCst), expected_value);

// Test Arc<Mutex<Vec>> error collection
let errors = Arc::new(Mutex::new(Vec::new()));
// ... parallel operations add errors ...
let error_list = errors.lock().await;
assert_eq!(error_list.len(), expected_error_count);

// Test Arc<Semaphore> concurrency limiting
let semaphore = Arc::new(Semaphore::new(8));
// ... verify only 8 concurrent operations ...
```

=== EXPECTED OUTPUT ===

Paste complete, ready-to-run test file:
- Min 1200 lines
- All 40-50 tests fully implemented
- Every test executable
- All async patterns correct
- All Arc<> patterns tested
- All error scenarios covered
- Zero stubs or placeholders

Ready to run: cargo test file_import -- --test-threads=1
```

---

## 5.2.2: analyze.rs Tests

**COMMAND:** `/unit-test-generator:generate-tests`
**SELECT FILES:** `pipeline/src-tauri/src/commands/analyze.rs`

**PASTE THIS PROMPT:**

```
GENERATE PRODUCTION-READY analyze.rs TESTS

=== CRITICAL REQUIREMENTS ===
✓ ALL tests 100% complete and executable
✓ NO stubs, NO unimplemented!()
✓ Every test passes: cargo test analyze -- --test-threads=1
✓ Musical algorithm accuracy tested (±1%)
✓ 85%+ code coverage
✓ Complete async/concurrency patterns

=== SCOPE: 35-40 Tests Total ===

FUNCTION 1: start_analysis() - 15 Tests
Test Cases:
  ✓ test_start_analysis_single_file
    Input: 1 file in database
    Expected: Analyzed, Arc<AtomicUsize> counter == 1

  ✓ test_start_analysis_100_files
    Input: 100 files in database
    Expected: All analyzed, elapsed < 30s

  ✓ test_start_analysis_1000_files
    Input: 1000 files in database (batched)
    Expected: All analyzed via 32 worker pool, elapsed < 120s

  ✓ test_start_analysis_metadata_extracted
    Input: MIDI files with known BPM/key
    Expected: Metadata saved: bpm, key, instruments, duration

  ✓ test_start_analysis_batch_database_insert
    Input: 100 files analyzed
    Expected: Batch insert works (verify query execution)

  ✓ test_start_analysis_worker_pool_32
    Input: 100 files with monitoring
    Expected: Verify 32 concurrent workers active

  ✓ test_start_analysis_arc_atomic_counter
    Input: 50 files with concurrent analysis
    Expected: Arc<AtomicUsize> counter == 50 at completion

  ✓ test_start_analysis_arc_mutex_error_collection
    Input: 20 files, 3 unparseable
    Expected: Arc<Mutex<Vec>> collects errors, analysis continues

  ✓ test_start_analysis_progress_events
    Input: 100 files
    Expected: ~10 progress events emitted

  ✓ test_start_analysis_batching_1000_items
    Input: 1000 files
    Expected: Database batches verified (batch_size <= 1000)

  ✓ test_start_analysis_error_continuation
    Input: 20 files, 5 parse errors
    Expected: All 15 valid analyzed, errors collected

  ✓ test_start_analysis_database_error_handling
    Input: simulate database constraint violation
    Expected: Error collected and returned

  ✓ test_start_analysis_parse_error_handling
    Input: 5 invalid MIDI files mixed with valid
    Expected: Parse errors collected, valid ones analyzed

  ✓ test_start_analysis_edge_case_no_bpm
    Input: MIDI with no BPM events
    Expected: Default to 120 BPM, stored correctly

  ✓ test_start_analysis_edge_case_extremely_complex
    Input: MIDI with 100k+ events
    Expected: Analyzed successfully, no performance degradation

FUNCTION 2: detect_key() - 7 Tests
Test Cases:
  ✓ test_detect_key_c_major
    Input: MIDI with notes C E G (C major triad)
    Expected: Key == C major, confidence > 0.8

  ✓ test_detect_key_a_minor
    Input: MIDI with notes A C E (A minor triad)
    Expected: Key == A minor, confidence > 0.8

  ✓ test_detect_key_all_major_keys
    Input: Test all 12 major keys
    Expected: Each correctly identified

  ✓ test_detect_key_all_minor_keys
    Input: Test all 12 minor keys
    Expected: Each correctly identified

  ✓ test_detect_key_no_notes
    Input: MIDI with no note events
    Expected: Default key or Err with explanation

  ✓ test_detect_key_confidence_score
    Input: MIDI with ambiguous notes
    Expected: Confidence score between 0.0 and 1.0, realistic

  ✓ test_detect_key_single_note
    Input: MIDI with only single note played repeatedly
    Expected: Key detected or graceful fallback

FUNCTION 3: analyze_bpm() - 8 Tests
Test Cases:
  ✓ test_analyze_bpm_120
    Input: MIDI with tempo 120 BPM
    Expected: Detected BPM == 120.0 (±1%)

  ✓ test_analyze_bpm_variable_tempos
    Input: MIDI with tempos [120, 140, 100]
    Expected: Weighted average calculated, within range

  ✓ test_analyze_bpm_default_no_tempo
    Input: MIDI with no tempo events
    Expected: Default to 120 BPM

  ✓ test_analyze_bpm_invalid_midi
    Input: Corrupted MIDI data
    Expected: Err with explanation

  ✓ test_analyze_bpm_zero_tempo_events
    Input: MIDI with empty tempo list
    Expected: Default to 120

  ✓ test_analyze_bpm_1000_tempo_changes
    Input: MIDI with 1000+ tempo changes
    Expected: Algorithm handles correctly

  ✓ test_analyze_bpm_slow_tempo
    Input: Very slow: 30 BPM
    Expected: Correctly detected or error

  ✓ test_analyze_bpm_fast_tempo
    Input: Very fast: 300 BPM
    Expected: Correctly detected or error

FUNCTION 4: extract_instruments() - 5 Tests
Test Cases:
  ✓ test_extract_instruments_piano
    Input: MIDI with program change 0 (piano)
    Expected: Instruments vector includes "Grand Piano"

  ✓ test_extract_instruments_multiple
    Input: MIDI with tracks: Piano (0), Violin (40), Bass (33)
    Expected: All 3 detected correctly

  ✓ test_extract_instruments_drums
    Input: MIDI with drum track (channel 10)
    Expected: "Drums" or "Percussion" detected

  ✓ test_extract_instruments_no_program_changes
    Input: MIDI with no program changes
    Expected: Empty instruments or default

  ✓ test_extract_instruments_invalid_program
    Input: MIDI with invalid program numbers
    Expected: Graceful handling (skip or default)

=== TECHNOLOGY REQUIREMENTS ===

Use fixtures:
```rust
TestDatabase::new().await
FileFixtures::new().await
MidiFileBuilder::new()
  .tempo(120.0)
  .duration_ms(4000)
  .build()
MetadataBuilder::new()
  .bpm(120.0)
  .key("C major")
  .build()
```

All tests use:
- #[tokio::test]
- Accuracy assertions within tolerance (±1%)
- Algorithm validation (musical correctness)
- TestDatabase for persistence
- Proper error handling

=== EXPECTED OUTPUT ===

Complete test file:
- Min 1200 lines
- All 35-40 tests fully implemented
- All musical algorithms tested
- All accuracy requirements met
- All error cases covered
- Ready to run: cargo test analyze -- --test-threads=1
```

---

## 5.2.3: split_file.rs Tests

**COMMAND:** `/unit-test-generator:generate-tests`
**SELECT FILES:** `pipeline/src-tauri/src/commands/split_file.rs`

**PASTE THIS PROMPT:**

```
GENERATE PRODUCTION-READY split_file.rs TESTS

=== CRITICAL REQUIREMENTS ===
✓ ALL tests 100% complete and executable
✓ NO stubs, NO unimplemented!()
✓ Every test passes: cargo test split_file -- --test-threads=1
✓ File I/O tested and cleaned up automatically
✓ 80%+ code coverage
✓ Database integrity verified

=== SCOPE: 25-30 Tests Total ===

FUNCTION 1: split_and_import() - 10 Tests
Test Cases:
  ✓ test_split_and_import_multitrack_2_tracks
    Input: 2-track MIDI (Piano, Strings)
    Expected: 2 separate files created, parent-child links created

  ✓ test_split_and_import_multitrack_8_tracks
    Input: 8-track MIDI
    Expected: All 8 files created, parent IDs set correctly

  ✓ test_split_and_import_parent_child_relationships
    Input: 4-track MIDI
    Expected: All tracks have parent_id pointing to original file

  ✓ test_split_and_import_filename_sanitization
    Input: Track names with special chars: "Piano/Left", "Violin\Right"
    Expected: Filenames sanitized to safe names

  ✓ test_split_and_import_database_transaction
    Input: 4-track MIDI
    Expected: All tracks saved atomically or all rolled back

  ✓ test_split_and_import_file_not_found
    Input: /nonexistent/multitrack.mid
    Expected: Err with "not found" message

  ✓ test_split_and_import_invalid_midi_format
    Input: Text file with .mid extension
    Expected: Err with "invalid MIDI" message

  ✓ test_split_and_import_database_constraint_violation
    Input: Valid MIDI but database full
    Expected: Err with database error message

  ✓ test_split_and_import_single_track_edge_case
    Input: 1-track MIDI (nothing to split)
    Expected: Either skip or create 1 file

  ✓ test_split_and_import_64_track_edge_case
    Input: 64-track MIDI (max typical)
    Expected: All 64 files created successfully

FUNCTION 2: filename_generation() - 8 Tests
Test Cases:
  ✓ test_filename_sanitization_basic
    Input: "My Track"
    Expected: "My Track.mid" (unchanged)

  ✓ test_filename_sanitization_special_chars
    Input: "Track/With\\Slashes|And:Colons"
    Expected: "Track-With-Slashes-And-Colons.mid"

  ✓ test_filename_sanitization_path_traversal_prevention
    Input: "../../etc/passwd"
    Expected: "etc-passwd.mid" (no path traversal)

  ✓ test_filename_duplicate_counter
    Input: "Piano" appears twice
    Expected: "Piano.mid" and "Piano (1).mid"

  ✓ test_filename_truncation_long_name
    Input: 300-char filename
    Expected: Truncated to max (e.g., 200 chars)

  ✓ test_filename_unicode_handling
    Input: "Piano™ Ñiño 中文"
    Expected: Either preserved or converted to ASCII

  ✓ test_filename_null_bytes_prevention
    Input: "Piano\x00Track"
    Expected: Null bytes removed or error

  ✓ test_filename_spaces_and_case_preservation
    Input: "My Grand Piano", "DRUMS", "piZZiCato"
    Expected: Preserved as-is

FUNCTION 3: track_extraction() - 6 Tests
Test Cases:
  ✓ test_track_extraction_to_separate_midi
    Input: Track from multi-track MIDI
    Expected: Separate MIDI file with only that track

  ✓ test_track_extraction_preserve_events
    Input: Track with note on/off, CC, pitch bend, tempo
    Expected: All events preserved in extracted MIDI

  ✓ test_track_extraction_maintain_timing
    Input: Track with events at specific times
    Expected: Timing relationships maintained

  ✓ test_track_extraction_index_out_of_bounds
    Input: Request track 10 from 4-track MIDI
    Expected: Err with "track index out of bounds"

  ✓ test_track_extraction_empty_track
    Input: Track with no notes
    Expected: Still valid MIDI, just empty

  ✓ test_track_extraction_10k_notes
    Input: Track with 10,000+ notes
    Expected: All notes preserved correctly

FUNCTION 4: database_relationships() - 5 Tests
Test Cases:
  ✓ test_parent_child_link_creation
    Input: Split 4-track MIDI
    Expected: All child records have parent_id foreign key

  ✓ test_cascade_delete_works
    Input: Delete parent file
    Expected: All child files deleted (cascade)

  ✓ test_metadata_correctly_assigned
    Input: Split MIDI with metadata
    Expected: Metadata copied to all child files

  ✓ test_tags_inherited_from_parent
    Input: Parent file has tags, split it
    Expected: Child files inherit parent tags (or not - clarify business logic)

  ✓ test_orphaned_track_detection
    Input: Child file without valid parent_id
    Expected: Detection and warning (or repair)

=== TECHNOLOGY REQUIREMENTS ===

Use fixtures:
```rust
TestDatabase::new().await
FileFixtures::new().await
MidiFileBuilder::new()
  .add_track("Piano", notes_vec)
  .add_track("Strings", notes_vec)
  .build()
  .to_file("multitrack.mid").await
```

All tests use:
- #[tokio::test]
- TestDatabase for database ops
- FileFixtures for file I/O (auto-cleanup)
- MidiFileBuilder for test MIDI
- Proper async/await

=== EXPECTED OUTPUT ===

Complete test file:
- Min 900 lines
- All 25-30 tests fully implemented
- All file I/O tested
- All database integrity tests
- Ready to run: cargo test split_file -- --test-threads=1
```

---

## 5.2.4-11: Remaining Pipeline Commands

**COMMAND:** `/unit-test-generator:generate-tests`
**SELECT FILES:** ALL of these (6 files):
- `pipeline/src-tauri/src/commands/archive_import.rs`
- `pipeline/src-tauri/src/commands/search.rs`
- `pipeline/src-tauri/src/commands/files.rs`
- `pipeline/src-tauri/src/commands/tags.rs`
- `pipeline/src-tauri/src/commands/stats.rs`
- `pipeline/src-tauri/src/commands/progress.rs`

**PASTE THIS PROMPT:**

```
GENERATE PRODUCTION-READY PIPELINE REMAINING COMMANDS TESTS

=== CRITICAL REQUIREMENTS ===
✓ ALL tests 100% complete and executable
✓ NO stubs, NO unimplemented!()
✓ Every test passes: cargo test --test commands -- --test-threads=1
✓ 80%+ coverage on each command
✓ Complete 6 test files in single generation

=== GENERATE 6 SEPARATE TEST FILES ===

FILE 1: archive_import_test.rs (15-20 Tests)
Test: ZIP extraction, RAR extraction, 7z extraction, corrupted archives, path traversal prevention, symlink prevention, temp cleanup, concurrent extraction, error handling, integration with file_import

Expected outcomes:
- All archive formats extracted correctly
- Path traversal attempts blocked (security)
- Symlink bombs prevented (security)
- Temp directories cleaned up automatically
- Errors collected and reported
- Progress events emitted

FILE 2: search_test.rs (20-25 Tests)
Test: Full-text search, BPM range filter, key filter, manufacturer filter, combined filters, pagination, sorting, empty results, special characters, SQL injection prevention, tag retrieval, file filtering by tag

Expected outcomes:
- Search returns correct results
- Filters work individually and combined
- Pagination boundaries handled
- Sorting works (name, BPM, date)
- NULL values handled correctly
- SQL injection attempts blocked
- Helper commands all work (get_all_tags, get_files_by_tag, get_bpm_range, get_all_keys)

FILE 3: files_test.rs (20-25 Tests)
Test: Database connection test, file count accuracy, file details retrieval, single file get, list files with pagination, filter by category, recent files, delete file with cascade, NULL handling in JOINs

Expected outcomes:
- All CRUD operations work
- Pagination boundaries correct
- Sorting works
- Cascade delete removes metadata/tags/splits
- NULL values don't break queries
- Large result sets handled
- Corrupted metadata handled gracefully

FILE 4: tags_test.rs (15-20 Tests)
Test: Get all tags, get tags for file, search tags, filter by category, add tag to file, remove tag from file, create tag, delete tag, update tag, popular tags, UPSERT behavior, duplicate prevention

Expected outcomes:
- Tag CRUD operations complete
- File-tag associations correct
- Search works on tag names
- Popular tags calculated correctly
- UPSERT prevents duplicates
- Category filtering works
- NULL categories handled

FILE 5: stats_test.rs (12-15 Tests)
Test: File stats (count, size, duration), category stats, manufacturer stats, key distribution, database health, duplicate detection, empty database handling, large database performance

Expected outcomes:
- All aggregation queries return correct numbers
- Group by operations work
- Empty database returns 0s
- Large databases (<1 sec for 10k files)
- NULL values excluded correctly
- Statistics formulas correct

FILE 6: progress_test.rs (15-18 Tests)
Test: Start progress, update progress, get progress, complete progress, reset progress, duplicate counter, error counter, error collection, Arc<Mutex> concurrent access, event emission, percentage calculation, value validation

Expected outcomes:
- State transitions correct
- Counters accurate
- Events emitted on updates
- Percentage calculated correctly (0-100)
- Concurrent access safe (no race conditions)
- Negative values prevented
- Max values enforced

=== TECHNOLOGY REQUIREMENTS ===

All tests use:
- TestDatabase fixture for database operations
- FileFixtures for test file creation
- MockWindow for event verification
- #[tokio::test] for async
- Proper Result<> error handling
- Test transactions auto-rollback

=== EXPECTED OUTPUT ===

6 complete test files:
- archive_import_test.rs: 15-20 tests
- search_test.rs: 20-25 tests
- files_test.rs: 20-25 tests
- tags_test.rs: 15-20 tests
- stats_test.rs: 12-15 tests
- progress_test.rs: 15-18 tests
- Total: 97-123 tests

Each file:
- Min 600-900 lines
- All tests fully implemented
- All error cases covered
- Ready to run immediately

Ready to run: cargo test --test commands -- --test-threads=1
```

---

## 5.4.1: sequencer.rs Tests

**COMMAND:** `/unit-test-generator:generate-tests`
**SELECT FILES:** `daw/src-tauri/src/commands/sequencer.rs`

**PASTE THIS PROMPT:**

```
GENERATE PRODUCTION-READY sequencer.rs TESTS

=== CRITICAL REQUIREMENTS ===
✓ ALL tests 100% complete and executable
✓ NO stubs, NO unimplemented!()
✓ Every test passes: cargo test sequencer -- --test-threads=1
✓ Real-time timing constraints verified (<5ms latency)
✓ 85%+ code coverage
✓ Complete MIDI state machine testing

=== SCOPE: 35-40 Tests Total ===

FUNCTION GROUP 1: State Management (15 Tests)
Test Cases:
  ✓ test_sequencer_state_stopped_initial
    Input: New sequencer
    Expected: state() == PlaybackState::Stopped

  ✓ test_sequencer_play_transitions_to_playing
    Input: Stopped sequencer, call play()
    Expected: state() == PlaybackState::Playing

  ✓ test_sequencer_pause_transitions_to_paused
    Input: Playing sequencer, call pause()
    Expected: state() == PlaybackState::Paused

  ✓ test_sequencer_resume_from_paused
    Input: Paused sequencer, call resume()
    Expected: state() == PlaybackState::Playing, position preserved

  ✓ test_sequencer_stop_transitions_to_stopped
    Input: Playing sequencer, call stop()
    Expected: state() == PlaybackState::Stopped, position reset to 0

  ✓ test_sequencer_seek_to_position
    Input: Playing, seek(1000) (1 second in)
    Expected: position() == 1000ms

  ✓ test_sequencer_seek_bounds_clamp_minimum
    Input: Seek(-100)
    Expected: position() == 0 (clamped to minimum)

  ✓ test_sequencer_seek_bounds_clamp_maximum
    Input: Seek(999999) beyond duration
    Expected: position() == max_position (clamped to max)

  ✓ test_sequencer_seek_while_playing
    Input: Playing, seek() to middle
    Expected: Continues playing from new position

  ✓ test_sequencer_rapid_state_changes
    Input: play() → pause() → resume() → stop() → play()
    Expected: All transitions valid, no race conditions

  ✓ test_sequencer_tempo_change_updates_speed
    Input: set_tempo(140) while playing
    Expected: Playback speed increases, tempo reflected in MIDI output

  ✓ test_sequencer_time_signature_change
    Input: set_time_signature(3, 4)
    Expected: Grid updates, playback continues

  ✓ test_sequencer_concurrent_state_queries
    Input: Multiple threads query state() simultaneously
    Expected: All get consistent results

  ✓ test_sequencer_state_persistence_pause_resume
    Input: Play → Pause → Resume
    Expected: Position preserved during pause

  ✓ test_sequencer_invalid_state_transitions_rejected
    Input: Try resume() when Stopped
    Expected: Either no-op or error, no panic

FUNCTION GROUP 2: Track Management (10 Tests)
Test Cases:
  ✓ test_sequencer_add_track_increments_count
    Input: add_track()
    Expected: track_count() increases by 1

  ✓ test_sequencer_remove_track_decrements_count
    Input: add_track() then remove_track(id)
    Expected: track_count() decreases by 1

  ✓ test_sequencer_reorder_tracks
    Input: 3 tracks, reorder to [2, 0, 1]
    Expected: Sequence updated, MIDI output respects order

  ✓ test_sequencer_track_volume_range_0_100
    Input: set_volume(track, value) with 0, 50, 100, -1, 101
    Expected: 0 and 100 work, ±1 clamped

  ✓ test_sequencer_track_pan_range_minus100_to_100
    Input: set_pan(track, value) with -100, 0, 100, -101, 101
    Expected: Valid range works, out of range clamped

  ✓ test_sequencer_mute_track_silences_output
    Input: Play notes on muted track
    Expected: MIDI events NOT sent to output

  ✓ test_sequencer_solo_track_only_solo_plays
    Input: Multiple tracks with 1 soloed
    Expected: Only solo track events sent, others silent

  ✓ test_sequencer_track_limit_64_enforced
    Input: Try add 65th track
    Expected: Error or clamped to 64

  ✓ test_sequencer_delete_playing_track_handles_gracefully
    Input: Playing notes from track, delete it
    Expected: Notes stop, playback continues on other tracks

  ✓ test_sequencer_get_track_details
    Input: get_track(id)
    Expected: Returns volume, pan, mute, solo, notes, etc.

FUNCTION GROUP 3: Event Scheduling (10 Tests)
Test Cases:
  ✓ test_sequencer_add_note_on_scheduled
    Input: add_note_on(pitch=60, velocity=100, time=0)
    Expected: Scheduled for MIDI transmission at correct time

  ✓ test_sequencer_add_note_off_matches_note_on
    Input: add_note_on() then add_note_off()
    Expected: Note off cancels note on, sound stops

  ✓ test_sequencer_add_cc_control_change
    Input: add_cc(control=7, value=100, time=0)
    Expected: CC message sent at correct time

  ✓ test_sequencer_add_pitch_bend
    Input: add_pitch_bend(pitch_value, time_range)
    Expected: Pitch bend ramp generated

  ✓ test_sequencer_add_program_change
    Input: add_program_change(program=0, time=0)
    Expected: MIDI sent

  ✓ test_sequencer_event_ordering_maintained
    Input: Add events out of order (time 100, 0, 50)
    Expected: Transmitted in correct time order (0, 50, 100)

  ✓ test_sequencer_concurrent_events_no_conflicts
    Input: Multiple events at same tick
    Expected: All events transmitted without conflicts

  ✓ test_sequencer_events_same_tick_ordered
    Input: Note on and CC at same time
    Expected: Specific order maintained (typically note on first)

  ✓ test_sequencer_overlapping_notes_both_sound
    Input: Note 1 (100ms duration), Note 2 starts at 50ms
    Expected: Both notes play, overlap handled correctly

  ✓ test_sequencer_event_timing_accuracy_5ms
    Input: Events scheduled at precise times
    Expected: Transmitted within 5ms of scheduled time

FUNCTION GROUP 4: MIDI Output (5 Tests)
Test Cases:
  ✓ test_sequencer_note_on_transmission_correct_pitch
    Input: Add C4 note (pitch 60)
    Expected: MIDI note on sent with correct pitch

  ✓ test_sequencer_note_off_transmission_correct_velocity
    Input: Add note with velocity 100
    Expected: Note off sent with matching velocity

  ✓ test_sequencer_event_transmission_order_correct
    Input: Play multi-event sequence
    Expected: Events transmitted in correct order

  ✓ test_sequencer_tempo_change_sent_as_midi
    Input: Change tempo during playback
    Expected: MIDI meta message sent

  ✓ test_sequencer_program_change_sent_correctly
    Input: Set instrument via program change
    Expected: MIDI program change sent to correct channel

=== TECHNOLOGY REQUIREMENTS ===

Use fixtures:
```rust
Sequencer::new(MockMidiOutput::new())
MockMidiOutput tracks all transmitted MIDI events
TimingAssertions for <5ms accuracy verification
TestDatabase for any state persistence
```

All tests use:
- #[tokio::test]
- MockMidiOutput for MIDI verification
- TimingAssertions for latency validation
- Proper timing checks without actual audio
- No real MIDI hardware required

=== EXPECTED OUTPUT ===

Complete test file:
- Min 1400 lines
- All 35-40 tests fully implemented
- Real-time constraints verified
- MIDI output tested
- Ready to run: cargo test sequencer -- --test-threads=1
```

---

## 5.4.2-6: Remaining DAW Commands

**COMMAND:** `/unit-test-generator:generate-tests`
**SELECT FILES:** ALL of these (5 files):
- `daw/src-tauri/src/commands/midi.rs`
- `daw/src-tauri/src/commands/project.rs`
- `daw/src-tauri/src/commands/export.rs`
- `daw/src-tauri/src/commands/analysis.rs`
- `daw/src-tauri/src/commands/search.rs`

**PASTE THIS PROMPT:**

```
GENERATE PRODUCTION-READY DAW REMAINING COMMANDS TESTS

=== CRITICAL REQUIREMENTS ===
✓ ALL tests 100% complete and executable
✓ NO stubs, NO unimplemented!()
✓ Every test passes: cargo test --test commands -- --test-threads=1
✓ 80%+ coverage on each command
✓ Complete 5 test files in single generation

=== GENERATE 5 SEPARATE TEST FILES ===

FILE 1: midi_test.rs (20-25 Tests)
Test: Port enumeration, list inputs, list outputs, open input, open output, close input, close output, send MIDI note on/off/CC, receive MIDI input, concurrent I/O, port disconnection, permission denied, buffer overflow prevention, device reconnection, port caching, timing, error recovery

Expected outcomes:
- All MIDI I/O operations work
- Concurrency safe
- Hot-plug device support
- Error handling robust
- Event timestamps accurate

FILE 2: project_test.rs (15-20 Tests)
Test: Create project, open project, save project, auto-save, undo operation, redo operation, undo/redo history limits, file versioning, metadata (title/author/date), track persistence, region persistence, concurrent save, corrupted project recovery, delete project, project list enumeration

Expected outcomes:
- Projects created and saved
- All metadata persisted
- Undo/redo work correctly
- Concurrent saves safe
- Corrupted files handled

FILE 3: export_test.rs (10-15 Tests)
Test: Export to MIDI, export with all events, export with timing, export with metadata, export to audio format, quality settings, batch export, export with markers/cue points, performance acceptable, file format valid, overwrite protection, insufficient space, invalid format

Expected outcomes:
- Exports create valid files
- All data preserved
- Performance reasonable
- Batch operations work
- Error handling robust

FILE 4: analysis_test.rs (10-12 Tests)
Test: Analyze BPM current selection, detect key, detect instruments, count notes, measure duration, analyze polyphony, find unison notes, complexity score, progress reporting, large file handling

Expected outcomes:
- Analysis results accurate
- Progress events emitted
- Large files handled efficiently
- All metrics calculated correctly

FILE 5: search_test.rs (10-15 Tests)
Test: Search library by name, BPM range filter, key filter, tag filter, combined filters, pagination, sorting, recent files, favorites filter, duplicate exclusion, performance with large library

Expected outcomes:
- Search returns correct results
- Filters work individually and combined
- Pagination correct
- Large libraries fast (<500ms)
- Sorting works

=== TECHNOLOGY REQUIREMENTS ===

All tests use:
- TestDatabase for data persistence
- MockMidiOutput for MIDI verification
- FileFixtures for file creation
- TestProject builder
- #[tokio::test] for async
- Proper error handling

=== EXPECTED OUTPUT ===

5 complete test files:
- midi_test.rs: 20-25 tests
- project_test.rs: 15-20 tests
- export_test.rs: 10-15 tests
- analysis_test.rs: 10-12 tests
- search_test.rs: 10-15 tests
- Total: 65-87 tests

Ready to run: cargo test --test commands -- --test-threads=1
```

---

## 5.5: Integration Tests

**COMMAND:** `/integration-test-runner:run-integration`

**PASTE THIS PROMPT:**

```
GENERATE PRODUCTION-READY INTEGRATION TESTS

=== CRITICAL REQUIREMENTS ===
✓ ALL tests 100% complete and executable
✓ NO stubs, NO unimplemented!()
✓ Test full workflows across Pipeline AND DAW
✓ Every test passes: cargo test --test integration -- --test-threads=1
✓ Database consistency verified
✓ 80%+ coverage on integration flows

=== SCOPE: 15-20 Tests Total ===

WORKFLOW 1: import_to_daw_workflow (5-6 Tests)
File: tests/integration/import_to_daw_workflow_test.rs
Tests:
  ✓ test_import_single_file_then_load_in_daw
    Import file → Database → DAW loads it → Verify

  ✓ test_import_batch_then_search_then_load
    Import 20 files → Search → Load results → Play

  ✓ test_import_then_analyze_then_verify_in_daw
    Import → Analyze → Verify BPM/key in DB → Load in DAW

  ✓ test_import_then_tag_then_filter_in_daw
    Import → Add tags → Search by tag → Load in DAW

  ✓ test_concurrent_imports_and_daw_loads
    10 concurrent imports while DAW loads other files

  ✓ test_error_during_import_daw_recovers
    Import fails mid-operation → DAW continues working

WORKFLOW 2: search_workflow (4-5 Tests)
File: tests/integration/search_workflow_test.rs
Tests:
  ✓ test_search_by_bpm_range_load_and_play
    Load library → Search BPM 120-140 → Load → Play

  ✓ test_search_with_multiple_filters
    Search (key=C major, BPM=120, tag=loop) → Load

  ✓ test_search_pagination_load_results
    Search with limit → Paginate → Load page 2

  ✓ test_concurrent_searches_consistent_results
    5 concurrent searches → All get consistent results

  ✓ test_empty_search_graceful_handling
    Search returns no results → Handled gracefully

WORKFLOW 3: analysis_roundtrip (3-4 Tests)
File: tests/integration/analysis_roundtrip_test.rs
Tests:
  ✓ test_import_analyze_verify_metadata_in_daw
    Import → Analyze → Verify in DB → Load in DAW → Check BPM/key

  ✓ test_manual_analysis_in_daw_export_reimport
    Create project → Analyze notes → Export → Re-import → Verify

  ✓ test_metadata_extraction_persists_correctly
    Import with metadata → Verify saved → Load in DAW

  ✓ test_concurrent_analysis_and_imports
    Analysis running while files imported

WORKFLOW 4: export_roundtrip (3-4 Tests)
File: tests/integration/export_roundtrip_test.rs
Tests:
  ✓ test_load_modify_export_reimport_verify
    Load file → Modify in DAW → Export → Re-import → Verify

  ✓ test_export_to_archive_extract_reimport
    Export to ZIP → Extract → Re-import → Verify

  ✓ test_batch_export_to_archive_bulk_reimport
    Export 20 files to archive → Extract all → Import → Verify

  ✓ test_export_performance_100_files
    Export 100 files performance baseline

=== TECHNOLOGY REQUIREMENTS ===

Use fixtures:
```rust
TestDatabase::new().await                   // Real database operations
FileFixtures::new().await                   // Test MIDI creation
Full actual imports/searches/loads (no mocks)
```

All tests use:
- #[tokio::test]
- Real database queries (no mocks)
- Real file I/O (no mocks)
- TestDatabase transaction rollback
- Comprehensive data validation

=== EXPECTED OUTPUT ===

4 complete integration test files:
- import_to_daw_workflow_test.rs: 5-6 tests
- search_workflow_test.rs: 4-5 tests
- analysis_roundtrip_test.rs: 3-4 tests
- export_roundtrip_test.rs: 3-4 tests
- Total: 15-20 tests

Ready to run: cargo test --test integration -- --test-threads=1
```

---

## 5.6: Performance Tests

**ACTION:** Performance tests already included in Phase 5.2-5.4 marked with `#[ignore]`

**RUN:**
```bash
cargo test --test commands -- --ignored          # Run performance tests
cargo tarpaulin --test commands --timeout 600    # Measure coverage
```

No additional generation needed - performance benchmarks already embedded in test files.

---

# PHASE 6: MODELS LAYER

## 6.2: All DAW Models Tests

**COMMAND:** `/unit-test-generator:generate-tests`
**SELECT FILES:** ALL files in `daw/src-tauri/src/models/`

**PASTE THIS PROMPT:**

```
GENERATE PRODUCTION-READY DAW MODELS TESTS

=== CRITICAL REQUIREMENTS ===
✓ ALL tests 100% complete and executable
✓ NO stubs, NO unimplemented!()
✓ Every test passes: cargo test --lib models -- --test-threads=1
✓ 95%+ coverage on all models
✓ Complete 7 test files in single generation

=== GENERATE 7 SEPARATE TEST FILES ===

EACH FILE PATTERN (Repeat for all 7 models):

1. Creation Tests (2-3 per model):
   ✓ Create with valid parameters
   ✓ Create with default values
   ✓ Create from builder pattern

2. Serialization Tests (2-3 per model):
   ✓ Serialize to JSON (serde_json::to_string)
   ✓ Deserialize from JSON (serde_json::from_str)
   ✓ Round-trip: serialize → deserialize → equals original

3. Validation Tests (2-3 per model):
   ✓ Test field constraints (ranges, required fields)
   ✓ Test min/max values
   ✓ Test invalid inputs

4. Behavior Tests (2-3 per model):
   ✓ Clone produces independent copy
   ✓ Equality comparison works
   ✓ Display formatting correct

5. Property-Based Tests (1-2 per model):
   ✓ Random valid data round-trips correctly
   ✓ Hash consistency

=== MODEL-SPECIFIC TESTS ===

FILE 1: track_model_test.rs (10-12 Tests)
Model: Track
Fields to test: name, number, channel, volume (0-100), pan (-100 to 100), mute, solo, notes, program
Tests:
  ✓ create_track_basic
  ✓ track_serialize_deserialize
  ✓ track_volume_validation
  ✓ track_pan_validation
  ✓ track_mute_state
  ✓ track_solo_mutual_exclusivity
  ✓ track_clone_independence
  ✓ track_equality_comparison
  ✓ track_display_formatting
  ✓ track_property_based_random

FILE 2: clip_model_test.rs (10-12 Tests)
Model: Clip
Fields: track_id, start, end, offset, stretch (0.5-2.0), transpose (-24 to 24), color, locked, notes, duration
Tests:
  ✓ create_clip_basic
  ✓ clip_serialize_deserialize
  ✓ clip_duration_calculation
  ✓ clip_stretch_validation
  ✓ clip_transpose_validation
  ✓ clip_overlap_detection
  ✓ clip_clone_independence
  ✓ clip_equality_comparison
  ✓ clip_color_assignment
  ✓ clip_locked_state

FILE 3: project_model_test.rs (10-12 Tests)
Model: Project
Fields: title, author, tempo (20-300 BPM), time_signature, key, tracks, regions, version, file_path, modified
Tests:
  ✓ create_project_new
  ✓ project_serialize_deserialize
  ✓ project_tempo_validation
  ✓ project_time_signature_validation
  ✓ project_key_signature_validation
  ✓ project_metadata_set
  ✓ project_tracks_persistence
  ✓ project_modified_flag
  ✓ project_version_tracking
  ✓ project_clone_with_structure

FILE 4: region_model_test.rs (8-10 Tests)
Model: Region
Fields: start, end, loop_start, loop_end, color
Tests:
  ✓ create_region_basic
  ✓ region_serialize_deserialize
  ✓ region_duration_calculation
  ✓ region_overlap_detection
  ✓ region_loop_point_validation
  ✓ region_equality_comparison
  ✓ region_clone_independence
  ✓ region_color_assignment

FILE 5: marker_model_test.rs (8-10 Tests)
Model: Marker
Fields: time, label, color, custom_data
Tests:
  ✓ create_marker_basic
  ✓ marker_serialize_deserialize
  ✓ marker_time_positioning
  ✓ marker_label_text
  ✓ marker_custom_data_preservation
  ✓ marker_color_assignment
  ✓ marker_equality_comparison
  ✓ marker_clone_independence

FILE 6: settings_model_test.rs (8-10 Tests)
Model: Settings
Fields: theme (Dark/Light), buffer_size (128/256/512/1024), midi_device, ui_scale (0.5-2.0), folders
Tests:
  ✓ create_settings_defaults
  ✓ settings_serialize_deserialize
  ✓ settings_theme_options
  ✓ settings_buffer_size_validation
  ✓ settings_midi_device_selection
  ✓ settings_ui_scale_validation
  ✓ settings_folder_paths
  ✓ settings_persistence

FILE 7: transport_model_test.rs (8-10 Tests)
Model: Transport
Fields: position, state (Play/Pause/Stop), tempo, recording
Tests:
  ✓ create_transport_initial
  ✓ transport_serialize_deserialize
  ✓ transport_position_tracking
  ✓ transport_state_transitions
  ✓ transport_tempo_changes
  ✓ transport_recording_state
  ✓ transport_equality_comparison
  ✓ transport_clone_independence

=== TECHNOLOGY REQUIREMENTS ===

All tests use:
- #[test] (not tokio::test - models are sync)
- serde_json for serialization
- Property-based testing with proptest
- Custom builder patterns
- No database or file I/O

=== EXPECTED OUTPUT ===

7 complete model test files:
- track_model_test.rs: 10-12 tests
- clip_model_test.rs: 10-12 tests
- project_model_test.rs: 10-12 tests
- region_model_test.rs: 8-10 tests
- marker_model_test.rs: 8-10 tests
- settings_model_test.rs: 8-10 tests
- transport_model_test.rs: 8-10 tests
- Total: 62-76 tests

Each file:
- Min 400-600 lines
- All tests fully implemented
- 95%+ coverage achieved
- Ready to run: cargo test --lib models

Ready to run: cargo test --lib models -- --test-threads=1
```

---

# PHASE 7: INTEGRATION & E2E TESTS

## 7.1: Full Workflow Integration Tests

**COMMAND:** `/integration-test-runner:run-integration`
(Uses Phase 5.5 output as foundation, extended)

**PASTE THIS PROMPT:**

```
GENERATE PRODUCTION-READY PHASE 7 FULL WORKFLOW INTEGRATION TESTS

=== CRITICAL REQUIREMENTS ===
✓ ALL tests 100% complete and executable
✓ NO stubs, NO unimplemented!()
✓ Test complex multi-component workflows
✓ Every test passes: cargo test --test integration -- --test-threads=1
✓ 80%+ coverage on full workflows
✓ Complete 4 test files extending Phase 5.5

=== GENERATE 4 COMPREHENSIVE TEST FILES ===

FILE 1: full_import_pipeline_test.rs (12-15 Tests)
Tests:
  ✓ test_single_file_import_all_steps
    Import → Hash → Parse → Analyze → Store → Verify

  ✓ test_batch_100_files_concurrent
    Import 100 files concurrently → Verify all in DB → Search results correct

  ✓ test_batch_with_50_percent_duplicates
    Import 100 files where 50 are duplicates → Correct deduplication

  ✓ test_archive_extraction_and_import
    Extract ZIP → Import all MIDIs → Verify counts

  ✓ test_concurrent_imports_database_consistency
    5 concurrent import_directory operations → All data consistent

  ✓ test_error_recovery_partial_import
    Import 100 files, fail on #50 → Rollback partial or cleanup orphans

  ✓ test_progress_tracking_100_files
    Import 100 → Verify ~10 progress events with correct counters

  ✓ test_metadata_extraction_accuracy
    Import known MIDI → Verify BPM, key, duration, instruments correct

  ✓ test_tag_auto_extraction_from_tracks
    Import → Verify auto-extracted tags in database

  ✓ test_database_constraint_enforcement
    Verify foreign keys, cascades, unique constraints enforced

  ✓ test_file_cleanup_temporary_files
    Import → Verify no temp files remain after completion

  ✓ test_import_performance_baseline
    Import 100 files → Should complete in <60 seconds

FILE 2: search_to_playback_test.rs (10-12 Tests)
Tests:
  ✓ test_search_load_play_workflow
    Search library → Load result → Play in DAW → Verify playback

  ✓ test_search_bpm_filter_accuracy
    Search 120-140 BPM → Verify all results in range

  ✓ test_search_key_filter_accuracy
    Search "C major" → Verify all results match

  ✓ test_search_tag_filter
    Search tag:"loop" → Load tagged files → Verify

  ✓ test_search_combined_filters
    Search (BPM 100-130, key=C, tag=drum) → Verify results

  ✓ test_search_pagination_large_library
    Search in 10k file library → Load pages → Verify consistency

  ✓ test_search_sorting_multiple_orders
    Search → Sort by BPM → Sort by date → Verify order

  ✓ test_search_performance_10k_files
    Search 10k file library → Complete in <500ms

  ✓ test_search_empty_results_handling
    Search returns 0 files → Handled gracefully

  ✓ test_search_special_characters
    Search with regex, quotes, unicode → Handled correctly

  ✓ test_concurrent_searches_consistency
    5 threads search simultaneously → All get consistent results

  ✓ test_search_null_handling_metadata
    Files with NULL BPM/key → Search handles gracefully

FILE 3: daw_sequencer_workflow_test.rs (15-18 Tests)
Tests:
  ✓ test_create_project_add_tracks_play
    Create project → Add 4 tracks → Play → Verify MIDI output

  ✓ test_load_midi_parse_sequencer_play
    Load MIDI file → Parse tracks → Load in sequencer → Play

  ✓ test_add_notes_modify_playback
    Add notes to sequencer → Modify velocities → Play → Verify

  ✓ test_mute_track_silences_output
    Mute track → Play → Verify MIDI silent on that track

  ✓ test_solo_track_only_sound
    Solo 1 of 4 tracks → Play → Only solo track heard

  ✓ test_tempo_change_during_playback
    Playing → Change tempo 120→140 → Verify speed increases

  ✓ test_time_signature_change
    Playing in 4/4 → Change to 3/4 → Grid updates

  ✓ test_undo_operation_state_restored
    Make changes → Undo → Verify previous state

  ✓ test_redo_operation_forward_state
    Undo → Redo → Verify forward state restored

  ✓ test_export_midi_creates_valid_file
    Project → Export → Verify MIDI file valid

  ✓ test_concurrent_track_edits_no_race
    Modify tracks concurrently → All changes safe

  ✓ test_real_time_midi_output_events
    Play → Verify MIDI events transmitted in order

  ✓ test_state_persistence_save_load
    Create → Save → Load → Verify state identical

  ✓ test_64_tracks_10k_notes_performance
    Large project → Play smoothly without glitches

  ✓ test_memory_no_leaks_repeated_ops
    Load/play/unload repeated 100x → No memory growth

  ✓ test_real_time_constraints_met
    MIDI events within <5ms of scheduled time

  ✓ test_playback_position_accuracy
    Play → Seek → Verify position within 50ms

  ✓ test_hardware_device_hot_swap
    Play → Disconnect MIDI device → Reconnect → Continue

FILE 4: full_roundtrip_test.rs (8-10 Tests)
Tests:
  ✓ test_import_analyze_load_modify_export_reimport
    Full workflow: Import → Analyze → Load in DAW → Modify → Export → Re-import → Verify all metadata preserved

  ✓ test_create_in_daw_export_import_pipeline
    Create in DAW → Export → Import in Pipeline → Search → Verify

  ✓ test_batch_import_concurrent_daw_loads
    Import 50 files while DAW loads other files → All consistent

  ✓ test_search_load_modify_archive_extract_reimport
    Complex workflow: Search → Load → Modify → Archive → Extract → Re-import → Verify

  ✓ test_error_recovery_import_failure
    Import fails mid-operation → DAW continues working with existing data

  ✓ test_large_file_1mb_midi_full_workflow
    Large 1MB MIDI → Import → Load → Play → Export → Re-import

  ✓ test_performance_1000_file_library
    Library with 1000 files → All operations <5s latency

  ✓ test_data_integrity_end_to_end
    All metadata preserved: BPM, key, instruments, tags, custom fields → End-to-end

=== TECHNOLOGY REQUIREMENTS ===

All tests use:
- TestDatabase with real transactions
- FileFixtures for MIDI creation
- MockMidiOutput for playback verification
- Real file I/O and database queries
- Complete workflow testing (no mocking internal steps)
- Performance monitoring

=== EXPECTED OUTPUT ===

4 complete integration test files:
- full_import_pipeline_test.rs: 12-15 tests
- search_to_playback_test.rs: 10-12 tests
- daw_sequencer_workflow_test.rs: 15-18 tests
- full_roundtrip_test.rs: 8-10 tests
- Total: 45-55 tests

Ready to run: cargo test --test integration -- --test-threads=1
```

---

## 7.2: Performance Regression Tests

**COMMAND:** `performance-oracle` agent (1 call)

**CREATE FILE:** `tests/performance_regression_test.rs`

**PASTE THIS PROMPT FOR AGENT:**

```
Use the performance-oracle agent to create comprehensive performance regression test suite.

BENCHMARK TARGETS:
- MIDI parsing: 1MB file in <100ms
- BPM detection: 100 files in <30s
- Key detection: 100 files in <30s
- Batch database insert: 1000 records in <5s
- Search 10k library: <500ms
- Load in DAW: <5s
- Play 64 tracks: real-time capable (<5ms latency)

LOAD PATTERNS:
- Single operations (individual file import)
- Batch operations (100+ files)
- Concurrent operations (10 concurrent imports)
- Mixed operations (import + search + playback)

CREATE:
- Baseline measurements
- Performance regression tests
- Load test scenarios
- Memory profiling
- CPU profiling

OUTPUT: performance_regression_test.rs with 10-12 complete performance tests
```

---

## 7.3: Stress & Load Tests

**COMMAND:** `/test-orchestrator:orchestrate`

**CREATE FILE:** `tests/stress_load_test.rs`

**PASTE THIS PROMPT:**

```
Use the test-orchestrator to create stress and load testing scenarios.

LOAD SCENARIOS:
1. Database Growth:
   - Start: 1MB, 100 files
   - Middle: 100MB, 1000 files
   - End: 1GB, 10000 files
   - Verify performance at each level

2. Concurrent User Load:
   - 10 concurrent imports
   - 100 concurrent searches
   - 5 concurrent DAW instances

3. Recovery Testing:
   - Interrupted import (mid-operation)
   - Interrupted playback (sudden stop)
   - Database connection loss
   - Out of disk space
   - Corrupted database file

4. Resource Exhaustion:
   - Memory limits
   - File descriptor limits
   - Database connection pool exhaustion
   - Temporary file cleanup under stress

OUTPUT: stress_load_test.rs with 8-10 complete stress tests
```

---

## 7.4: End-to-End User Journey Tests

**COMMAND:** `/test-orchestrator:orchestrate`

**CREATE FILE:** `tests/user_journey_test.rs`

**PASTE THIS PROMPT:**

```
Use the test-orchestrator to create realistic end-to-end user journey tests.

JOURNEY 1: Producer Workflow (7 steps)
1. Import 50 MIDI files from external drive
2. Analyze full library (BPM, key, instruments)
3. Search for 120 BPM files in C major
4. Load top 5 results in DAW sequencer
5. Modify arrangement (reorder tracks, adjust tempo)
6. Export as high-quality MIDI collection
7. Archive to backup drive

JOURNEY 2: Sound Designer Workflow (6 steps)
1. Create new project
2. Load 20 library files as loops/samples
3. Build 8-track arrangement
4. Add automation (volume, pan, effects)
5. Export individual track stems
6. Re-import for mastering workflow

JOURNEY 3: Library Manager Workflow (5 steps)
1. Organize library with hierarchical tags
2. Create collections (Drums, Synths, Ambience)
3. Generate library statistics report
4. Export catalog for distribution
5. Create backup archive for safety

JOURNEY 4: Collaborative Workflow (4 steps)
1. Import shared collection
2. Search for specific BPM/key combinations
3. Create personal remix project
4. Export and share back

OUTPUT: user_journey_test.rs with 10-12 complete E2E tests
```

---

# PHASE 8: DOCUMENTATION & FINAL VERIFICATION

## 8.1: Code Documentation

**COMMAND:** Use `rust-backend` agent (1 call)

**PASTE THIS PROMPT:**

```
Use the rust-backend agent to add comprehensive rustdoc comments to all public APIs.

TARGET MODULES:
1. shared/rust/src/core/ (all functions)
   - MIDI parser functions
   - Analysis algorithms (BPM, key, chord detection)
   - Helper functions

2. shared/rust/src/db/repositories/ (all methods)
   - File repository
   - Tag repository
   - Metadata repository
   - Search repository

3. pipeline/src-tauri/src/commands/ (all functions)
   - file_import
   - analyze
   - split_file
   - search
   - All other command functions

4. daw/src-tauri/src/commands/ (all functions)
   - sequencer
   - midi
   - project
   - export
   - All other command functions

DOCUMENTATION REQUIREMENTS FOR EACH PUBLIC FUNCTION:
- Brief one-line summary
- Detailed explanation (if complex)
- # Arguments section with parameter descriptions
- # Returns section with return value description
- # Errors section (if Result<T, E>)
- # Examples section with runnable example
- # Panics section (if can panic)

ENSURE:
- All doc comments compile without warnings
- Examples are compilable (use cargo test --doc)
- No "missing documentation" warnings remain

OUTPUT: All public APIs have complete rustdoc comments
Ready to verify: cargo doc --no-deps
```

---

## 8.3: Coverage Verification

**ACTION:** Manual verification (no generation needed)

**RUN:**

```bash
# Generate coverage report
cargo tarpaulin --workspace --out Html --timeout 600

# Expected results:
# - shared/rust/src/core/: 100% (all Trusty Modules)
# - {pipeline,daw}/src-tauri/src/commands/: 85%+
# - {pipeline,daw}/src-tauri/src/models/: 95%+
# - shared/rust/src/db/: 85%+
# - Overall: 90%+

# If any module below 80%:
# 1. Identify uncovered lines in coverage/index.html
# 2. Add targeted tests for those lines
# 3. Re-run tarpaulin
```

---

# QUICK EXECUTION SUMMARY

## Phase 5 (Commands): 3 Main Prompts
```bash
# Prompt 1: Pipeline commands (5.2)
/unit-test-generator:generate-tests
# file_import + analyze + split_file + remaining 6

# Prompt 2: DAW commands (5.4)
/unit-test-generator:generate-tests
# sequencer + midi + project + export + analysis + search

# Prompt 3: Integration tests (5.5)
/integration-test-runner:run-integration
# Full workflow tests
```

## Phase 6 (Models): 1 Prompt
```bash
# All DAW models
/unit-test-generator:generate-tests
# 7 model test files
```

## Phase 7 (Integration & E2E): 4 Prompts
```bash
# Workflow tests
/integration-test-runner:run-integration

# Performance tests
performance-oracle agent

# Stress/load tests
/test-orchestrator:orchestrate

# User journey tests
/test-orchestrator:orchestrate
```

## Phase 8 (Documentation): 2 Tasks
```bash
# Documentation
rust-backend agent

# Coverage verification
cargo tarpaulin --workspace
```

---

## Total: ~14 generation calls → 600-700 tests → 90%+ coverage

**All tests 100% production-ready. No stubs. All executable immediately.**

**Time: ~27-32 hours to completion**

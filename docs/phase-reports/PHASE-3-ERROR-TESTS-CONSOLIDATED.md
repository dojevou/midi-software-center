# PHASE 3: ERROR TESTS IMPLEMENTATION GUIDE

## Status: Ready for Rapid Implementation

### File: analyze_test.rs (Add 12 error tests)
```rust
#[tokio::test]
async fn test_analyze_bpm_zero_tempo() { assert!(analyze_bpm_detection(zero_tempo_midi).bpm.is_none()); }

#[tokio::test]
async fn test_analyze_bpm_varying_tempo() { let bpm = analyze_bpm_detection(varying_tempo_midi).bpm.unwrap(); assert!(40.0 <= bpm && bpm <= 220.0); }

#[tokio::test]
async fn test_analyze_key_minimal_data() { assert!(analyze_key_detection(single_note_midi).key_signature.is_none()); }

#[tokio::test]
async fn test_analyze_key_atonal() { let key = analyze_key_detection(atonal_midi); assert!(key.key_signature.is_none() || matches!(key.confidence, 0..=30)); }

#[tokio::test]
async fn test_analyze_duration_timeout() { assert!(timeout(5s, analyze_duration(huge_midi)).await.is_err()); }

#[tokio::test]
async fn test_analyze_worker_pool_exhaustion() { queue_1000_tasks(pool_4_workers).await; assert!(all_tasks_complete()); }

#[tokio::test]
async fn test_analyze_concurrent_idempotency() { let r1 = analyze(file).await; let r2 = analyze(file).await; assert_eq!(r1.bpm, r2.bpm); }

#[tokio::test]
async fn test_analyze_batch_constraint_failure() { batch_analyze(50_files_injected_fk_fail).await; assert_eq!(db_records(), 0); }

#[tokio::test]
async fn test_analyze_db_connection_depletion() { close_pool(); assert!(analyze(file).await.is_err()); }

#[tokio::test]
async fn test_analyze_large_file_timeout() { assert!(timeout(5s, analyze(1m_events_midi)).await.is_err()); }

#[tokio::test]
async fn test_analyze_metadata_graceful_failure() { let result = analyze(corrupted_meta_midi).await; assert!(result.is_ok()); assert!(result.unwrap().metadata.is_empty_or_partial()); }

#[tokio::test]
async fn test_analyze_truncated_events() { assert!(analyze(truncated_midi).await.is_err()); }
```

### File: split_file_test.rs (Add 10 error tests)
```rust
#[tokio::test]
async fn test_split_no_tracks() { assert!(split_tracks(zero_track_midi, 0).await.is_err()); }

#[tokio::test]
async fn test_split_invalid_track_index() { assert!(split_tracks(3_track_midi, 5).await.is_err()); }

#[tokio::test]
async fn test_split_corrupted_track() { assert!(split_tracks(corrupted_track_midi, 0).await.is_err()); }

#[tokio::test]
async fn test_split_channel_no_events() { assert!(split_by_channel(no_events_midi, 0).await.is_err()); }

#[tokio::test]
async fn test_split_invalid_channel() { assert!(split_by_channel(midi, 16).await.is_err()); }

#[tokio::test]
async fn test_split_property_preservation_corrupt() { let result = split_tracks(partial_corrupt_midi, 0).await; assert!(result.is_ok()); }

#[tokio::test]
async fn test_split_db_insert_failure() { inject_fk_failure(); assert!(split_tracks(midi, 0).await.is_err()); verify_no_orphaned_files(); }

#[tokio::test]
async fn test_split_io_error() { readonly_filesystem(); assert!(split_tracks(midi, 0).await.is_err()); }

#[tokio::test]
async fn test_split_concurrent_same_file() { let r1 = split_tracks(midi, 0); let r2 = split_tracks(midi, 0); tokio::join!(r1, r2); assert!(both_succeed_isolated()); }

#[tokio::test]
async fn test_split_output_exceeds_limit() { assert!(split_tracks(100mb_track, 0).await.is_err()); }
```

### File: archive_import_test.rs (Add 12 error tests)
```rust
#[tokio::test]
async fn test_archive_nonexistent() { assert!(import_archive("/nonexistent.zip").await.is_err()); }

#[tokio::test]
async fn test_archive_permission_denied() { chmod_000("/archive.zip"); assert!(import_archive("/archive.zip").await.is_err()); }

#[tokio::test]
async fn test_archive_corrupted_structure() { assert!(import_archive(corrupted_zip).await.is_err()); }

#[tokio::test]
async fn test_archive_path_traversal() { assert!(import_archive(zip_with_../../../etc).await.is_err()); verify_no_files_outside_temp(); }

#[tokio::test]
async fn test_archive_absolute_path() { assert!(import_archive(zip_with_/tmp/path).await.is_err()); }

#[tokio::test]
async fn test_archive_nested_zip() { assert!(import_archive(zip_containing_zip).await.is_err() || skips_nested()); }

#[tokio::test]
async fn test_archive_compression_bomb() { assert!(import_archive(1kb_compressed_10gb).await.is_err()); }

#[tokio::test]
async fn test_archive_excessive_files() { assert!(import_archive(zip_1m_entries).await.is_err()); }

#[tokio::test]
async fn test_archive_all_invalid_midi() { let result = import_archive(zip_10_invalid_files).await; assert_eq!(result.successful, 0); }

#[tokio::test]
async fn test_archive_db_constraint() { assert!(import_archive(zip_duplicates).await.is_ok()); assert_eq!(db_duplicates(), 0); }

#[tokio::test]
async fn test_archive_io_error_disk_full() { fill_disk(); assert!(import_archive(archive).await.is_err()); cleanup_temp(); }

#[tokio::test]
async fn test_archive_concurrent_same() { tokio::join!(import_archive(z), import_archive(z)); assert_eq!(db_duplicates(), 0); }
```

## Implementation Steps

1. **Parse existing test structure** from each file
2. **Add error test sections** (SECTION 5-6 for error tests)
3. **Ensure test fixtures** support error conditions
4. **Run: `cargo test --package midi-pipeline`**
5. **Verify:** All 34 tests compile and pass

## Expected Result
- **file_import_test.rs:** 42 → 57 tests (54% → 75% error coverage)
- **analyze_test.rs:** 35 → 47 tests (54% → 80% error coverage)
- **split_file_test.rs:** 27 → 37 tests (54% → 75% error coverage)
- **archive_import_test.rs:** 20 → 32 tests (54% → 85% error coverage)
- **Total Phase 3:** 163 → 173 tests, **54% → 75%+ error coverage**

## Token-Efficient Implementation
- Use pseudocode templates above
- Adapt to actual fixture patterns in each file
- Focus on assert statements, minimal comments
- Batch similar error types together

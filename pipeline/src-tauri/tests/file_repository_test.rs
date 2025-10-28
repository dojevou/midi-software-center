//! Comprehensive tests for FileRepository
//! Coverage target: 80%+ (Trusty Module requirement)
//!
//! Test Categories:
//! 1. Insert Operations (basic, with all fields, edge cases)
//! 2. Find Operations (by ID, hash, path)
//! 3. Duplicate Detection
//! 4. Update Operations (mark_analyzed, metadata_fields)
//! 5. Delete Operations
//! 6. Count Operations
//! 7. List Operations (pagination, filtering)
//! 8. Edge Cases (not found, empty results, large data)
//! 9. Error Handling (invalid data, constraint violations)
//! 10. Performance Tests (batch operations, large datasets)

use midi_pipeline::db::models::{File, NewFile};
use midi_pipeline::db::repositories::FileRepository;
use sqlx::PgPool;

// Import test infrastructure
mod fixtures;
mod helpers;
mod common;

use fixtures::*;
use helpers::db::*;
use helpers::macros::*;
use common::*;

// ============================================================================
// SECTION 1: Insert Operations (12 tests)
// ============================================================================

#[tokio::test]
async fn test_insert_basic_file() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let new_file = NewFileBuilder::new()
        .filename("basic_test.mid")
        .filepath("/test/basic_test.mid")
        .content_hash(random_hash())
        .build();

    let file_id = FileRepository::insert(&pool, new_file)
        .await
        .expect("Insert failed");

    assert!(file_id > 0, "File ID should be positive");
    assert_file_exists(&pool, file_id).await;
    assert_file_count(&pool, 1).await;

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_insert_with_all_fields() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let new_file = NewFileBuilder::new()
        .filename("complete_test.mid")
        .filepath("/test/complete_test.mid")
        .original_filename("original_complete.mid")
        .content_hash(random_hash())
        .file_size_bytes(2048)
        .format(1)
        .num_tracks(4)
        .ticks_per_quarter_note(960)
        .duration_seconds(120.5)
        .duration_ticks(115200)
        .manufacturer("Roland")
        .collection_name("Test Collection")
        .folder_tags(vec!["drums".to_string(), "loops".to_string()])
        .import_batch_id(uuid::Uuid::new_v4())
        .build();

    let file_id = FileRepository::insert(&pool, new_file)
        .await
        .expect("Insert failed");

    let file = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");

    assert_eq!(file.filename, "complete_test.mid");
    assert_eq!(file.num_tracks, 4);
    assert_eq!(file.manufacturer, Some("Roland".to_string()));
    assert_eq!(file.collection_name, Some("Test Collection".to_string()));
    assert_eq!(file.folder_tags, Some(vec!["drums".to_string(), "loops".to_string()]));

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_insert_drum_loop_preset() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let drum_loop = Fixtures::drum_loop();
    let file_id = FileRepository::insert(&pool, drum_loop)
        .await
        .expect("Insert failed");

    let file = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");

    assert_eq!(file.manufacturer, Some("Ableton".to_string()));
    assert_eq!(file.num_tracks, 1);
    assert!(file.filename.contains("Drum"));

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_insert_piano_chord_preset() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let piano_chord = Fixtures::piano_chords();
    let file_id = FileRepository::insert(&pool, piano_chord)
        .await
        .expect("Insert failed");

    let file = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");

    assert_eq!(file.manufacturer, Some("Native Instruments".to_string()));
    assert!(file.filename.contains("Piano"));

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_insert_bass_line_preset() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let bass_line = Fixtures::bass_line();
    let file_id = FileRepository::insert(&pool, bass_line)
        .await
        .expect("Insert failed");

    assert!(file_id > 0);
    assert_file_exists(&pool, file_id).await;

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_insert_multiple_files() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let files = vec![
        Fixtures::drum_loop(),
        Fixtures::piano_chords(),
        Fixtures::bass_line(),
    ];

    for file in files {
        FileRepository::insert(&pool, file)
            .await
            .expect("Insert failed");
    }

    assert_file_count(&pool, 3).await;

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_insert_with_empty_optional_fields() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let new_file = NewFileBuilder::new()
        .filename("minimal.mid")
        .filepath("/test/minimal.mid")
        .content_hash(random_hash())
        // No manufacturer
        // No collection
        // No folder tags
        .build();

    let file_id = FileRepository::insert(&pool, new_file)
        .await
        .expect("Insert failed");

    let file = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");

    assert_eq!(file.manufacturer, None);
    assert_eq!(file.collection_name, None);
    assert_eq!(file.folder_tags, None);

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_insert_with_long_filename() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    // 200 character filename
    let long_name = "a".repeat(200) + ".mid";
    let new_file = NewFileBuilder::new()
        .filename(&long_name)
        .filepath(&format!("/test/{}", long_name))
        .content_hash(random_hash())
        .build();

    let file_id = FileRepository::insert(&pool, new_file)
        .await
        .expect("Insert failed");

    let file = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");

    assert_eq!(file.filename.len(), 204); // 200 + ".mid"

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_insert_with_unicode_filename() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let unicode_name = "测试_файл_🎹_音楽.mid";
    let new_file = NewFileBuilder::new()
        .filename(unicode_name)
        .filepath(&format!("/test/{}", unicode_name))
        .content_hash(random_hash())
        .build();

    let file_id = FileRepository::insert(&pool, new_file)
        .await
        .expect("Insert failed");

    let file = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");

    assert_eq!(file.filename, unicode_name);

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_insert_with_special_characters() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let special_name = "test-file_v2.0 (final) [edit].mid";
    let new_file = NewFileBuilder::new()
        .filename(special_name)
        .filepath(&format!("/test/{}", special_name))
        .content_hash(random_hash())
        .build();

    let file_id = FileRepository::insert(&pool, new_file)
        .await
        .expect("Insert failed");

    let file = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");

    assert_eq!(file.filename, special_name);

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_insert_returns_auto_incremented_id() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let file1 = NewFileBuilder::new()
        .filename("file1.mid")
        .filepath("/test/file1.mid")
        .content_hash(random_hash())
        .build();

    let file2 = NewFileBuilder::new()
        .filename("file2.mid")
        .filepath("/test/file2.mid")
        .content_hash(random_hash())
        .build();

    let id1 = FileRepository::insert(&pool, file1).await.expect("Insert failed");
    let id2 = FileRepository::insert(&pool, file2).await.expect("Insert failed");

    assert!(id2 > id1, "Second ID should be greater than first");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_insert_sets_timestamps() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let new_file = NewFileBuilder::new()
        .filename("timestamp_test.mid")
        .filepath("/test/timestamp_test.mid")
        .content_hash(random_hash())
        .build();

    let file_id = FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    let file = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");

    assert!(file.created_at.timestamp() > 0, "created_at should be set");
    assert!(file.updated_at.timestamp() > 0, "updated_at should be set");
    assert_eq!(file.analyzed_at, None, "analyzed_at should be NULL initially");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

// ============================================================================
// SECTION 2: Find Operations (15 tests)
// ============================================================================

#[tokio::test]
async fn test_find_by_id_existing() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let new_file = Fixtures::drum_loop();
    let file_id = FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    let found = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed");

    assert!(found.is_some(), "File should be found");
    let file = found.unwrap();
    assert_eq!(file.id, file_id);

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_find_by_id_not_found() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let found = FileRepository::find_by_id(&pool, 99999)
        .await
        .expect("Query should succeed");

    assert!(found.is_none(), "Non-existent file should return None");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_find_by_id_negative() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let found = FileRepository::find_by_id(&pool, -1)
        .await
        .expect("Query should succeed");

    assert!(found.is_none(), "Negative ID should return None");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_find_by_id_zero() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let found = FileRepository::find_by_id(&pool, 0)
        .await
        .expect("Query should succeed");

    assert!(found.is_none(), "Zero ID should return None");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_find_by_hash_existing() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let hash = random_hash();
    let new_file = NewFileBuilder::new()
        .filename("hash_test.mid")
        .filepath("/test/hash_test.mid")
        .content_hash(hash.clone())
        .build();

    FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    let found = FileRepository::find_by_hash(&pool, &hash)
        .await
        .expect("Find failed");

    assert!(found.is_some(), "File should be found by hash");
    let file = found.unwrap();
    assert_eq!(file.content_hash, hash);

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_find_by_hash_not_found() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let non_existent_hash = vec![255; 32];
    let found = FileRepository::find_by_hash(&pool, &non_existent_hash)
        .await
        .expect("Query should succeed");

    assert!(found.is_none(), "Non-existent hash should return None");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_find_by_hash_empty() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let empty_hash = vec![];
    let found = FileRepository::find_by_hash(&pool, &empty_hash)
        .await
        .expect("Query should succeed");

    assert!(found.is_none(), "Empty hash should return None");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_find_by_path_existing() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let filepath = "/test/unique_path_test.mid";
    let new_file = NewFileBuilder::new()
        .filename("unique_path_test.mid")
        .filepath(filepath)
        .content_hash(random_hash())
        .build();

    FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    let found = FileRepository::find_by_path(&pool, filepath)
        .await
        .expect("Find failed");

    assert!(found.is_some(), "File should be found by path");
    let file = found.unwrap();
    assert_eq!(file.filepath, filepath);

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_find_by_path_not_found() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let found = FileRepository::find_by_path(&pool, "/nonexistent/path.mid")
        .await
        .expect("Query should succeed");

    assert!(found.is_none(), "Non-existent path should return None");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_find_by_path_case_sensitive() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let filepath = "/Test/CaseSensitive.mid";
    let new_file = NewFileBuilder::new()
        .filename("CaseSensitive.mid")
        .filepath(filepath)
        .content_hash(random_hash())
        .build();

    FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    // PostgreSQL is case-sensitive for text comparison
    let found_exact = FileRepository::find_by_path(&pool, filepath)
        .await
        .expect("Find failed");
    assert!(found_exact.is_some(), "Exact case should match");

    let found_different = FileRepository::find_by_path(&pool, "/test/casesensitive.mid")
        .await
        .expect("Find failed");
    assert!(found_different.is_none(), "Different case should not match");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_find_by_path_with_spaces() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let filepath = "/test/file with spaces.mid";
    let new_file = NewFileBuilder::new()
        .filename("file with spaces.mid")
        .filepath(filepath)
        .content_hash(random_hash())
        .build();

    FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    let found = FileRepository::find_by_path(&pool, filepath)
        .await
        .expect("Find failed");

    assert!(found.is_some(), "Path with spaces should be found");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_find_by_path_with_unicode() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let filepath = "/测试/файл/音楽.mid";
    let new_file = NewFileBuilder::new()
        .filename("音楽.mid")
        .filepath(filepath)
        .content_hash(random_hash())
        .build();

    FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    let found = FileRepository::find_by_path(&pool, filepath)
        .await
        .expect("Find failed");

    assert!(found.is_some(), "Unicode path should be found");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_find_operations_return_complete_data() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let hash = random_hash();
    let new_file = NewFileBuilder::new()
        .filename("complete_data_test.mid")
        .filepath("/test/complete_data_test.mid")
        .content_hash(hash.clone())
        .num_tracks(4)
        .manufacturer("Roland")
        .build();

    let file_id = FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    // Test find_by_id returns all fields
    let by_id = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");

    assert_eq!(by_id.num_tracks, 4);
    assert_eq!(by_id.manufacturer, Some("Roland".to_string()));

    // Test find_by_hash returns all fields
    let by_hash = FileRepository::find_by_hash(&pool, &hash)
        .await
        .expect("Find failed")
        .expect("File not found");

    assert_eq!(by_hash.num_tracks, 4);
    assert_eq!(by_hash.manufacturer, Some("Roland".to_string()));

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_find_by_hash_with_limit_one() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    // Note: The function has LIMIT 1, so even if duplicates exist,
    // only one is returned (though duplicates shouldn't exist due to constraints)
    let hash = random_hash();
    let new_file = NewFileBuilder::new()
        .filename("limit_test.mid")
        .filepath("/test/limit_test.mid")
        .content_hash(hash.clone())
        .build();

    FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    let found = FileRepository::find_by_hash(&pool, &hash)
        .await
        .expect("Find failed");

    assert!(found.is_some(), "Should find exactly one file");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

// ============================================================================
// SECTION 3: Duplicate Detection (8 tests)
// ============================================================================

#[tokio::test]
async fn test_check_duplicate_not_exists() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let hash = random_hash();
    let is_duplicate = FileRepository::check_duplicate(&pool, &hash)
        .await
        .expect("Check failed");

    assert!(!is_duplicate, "New hash should not be duplicate");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_check_duplicate_exists() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let hash = random_hash();
    let new_file = NewFileBuilder::new()
        .filename("duplicate_test.mid")
        .filepath("/test/duplicate_test.mid")
        .content_hash(hash.clone())
        .build();

    FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    let is_duplicate = FileRepository::check_duplicate(&pool, &hash)
        .await
        .expect("Check failed");

    assert!(is_duplicate, "Existing hash should be duplicate");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_check_duplicate_workflow() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let hash = random_hash();

    // Step 1: Check - should not exist
    assert!(!FileRepository::check_duplicate(&pool, &hash).await.expect("Check failed"));

    // Step 2: Insert file
    let new_file = NewFileBuilder::new()
        .filename("workflow_test.mid")
        .filepath("/test/workflow_test.mid")
        .content_hash(hash.clone())
        .build();
    FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    // Step 3: Check again - should exist now
    assert!(FileRepository::check_duplicate(&pool, &hash).await.expect("Check failed"));

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_check_duplicate_empty_hash() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let empty_hash = vec![];
    let is_duplicate = FileRepository::check_duplicate(&pool, &empty_hash)
        .await
        .expect("Check should succeed");

    assert!(!is_duplicate, "Empty hash should not be duplicate");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_check_duplicate_different_hashes() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let hash1 = random_hash();
    let hash2 = random_hash();

    let file1 = NewFileBuilder::new()
        .filename("file1.mid")
        .filepath("/test/file1.mid")
        .content_hash(hash1.clone())
        .build();

    FileRepository::insert(&pool, file1).await.expect("Insert failed");

    // hash1 should be duplicate, hash2 should not
    assert!(FileRepository::check_duplicate(&pool, &hash1).await.expect("Check failed"));
    assert!(!FileRepository::check_duplicate(&pool, &hash2).await.expect("Check failed"));

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_check_duplicate_performance() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    // Insert 10 files with different hashes
    for i in 0..10 {
        let new_file = NewFileBuilder::new()
            .filename(&format!("perf_test_{}.mid", i))
            .filepath(&format!("/test/perf_test_{}.mid", i))
            .content_hash(random_hash())
            .build();
        FileRepository::insert(&pool, new_file).await.expect("Insert failed");
    }

    // Check duplicate should still be fast (indexed by hash)
    let test_hash = random_hash();
    let start = std::time::Instant::now();
    let _ = FileRepository::check_duplicate(&pool, &test_hash).await.expect("Check failed");
    let duration = start.elapsed();

    assert!(duration.as_millis() < 100, "Duplicate check should be fast (<100ms)");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_duplicate_detection_with_find_by_hash() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let hash = random_hash();

    // Check both methods agree before insert
    let check_result = FileRepository::check_duplicate(&pool, &hash).await.expect("Check failed");
    let find_result = FileRepository::find_by_hash(&pool, &hash).await.expect("Find failed");
    assert!(!check_result);
    assert!(find_result.is_none());

    // Insert file
    let new_file = NewFileBuilder::new()
        .filename("consistency_test.mid")
        .filepath("/test/consistency_test.mid")
        .content_hash(hash.clone())
        .build();
    FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    // Check both methods agree after insert
    let check_result = FileRepository::check_duplicate(&pool, &hash).await.expect("Check failed");
    let find_result = FileRepository::find_by_hash(&pool, &hash).await.expect("Find failed");
    assert!(check_result);
    assert!(find_result.is_some());

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_check_duplicate_with_32_byte_hash() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    // BLAKE3 produces 32-byte hashes
    let hash = vec![42u8; 32];
    let new_file = NewFileBuilder::new()
        .filename("blake3_test.mid")
        .filepath("/test/blake3_test.mid")
        .content_hash(hash.clone())
        .build();

    FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    let is_duplicate = FileRepository::check_duplicate(&pool, &hash)
        .await
        .expect("Check failed");

    assert!(is_duplicate, "32-byte hash should be found");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

// ============================================================================
// SECTION 4: Update Operations (12 tests)
// ============================================================================

#[tokio::test]
async fn test_mark_analyzed_success() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let new_file = Fixtures::drum_loop();
    let file_id = FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    // Initially analyzed_at should be NULL
    let before = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");
    assert_eq!(before.analyzed_at, None);

    // Mark as analyzed
    FileRepository::mark_analyzed(&pool, file_id)
        .await
        .expect("Mark analyzed failed");

    // Now analyzed_at should be set
    let after = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");
    assert!(after.analyzed_at.is_some(), "analyzed_at should be set");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_mark_analyzed_updates_timestamp() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let new_file = Fixtures::drum_loop();
    let file_id = FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    let before = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");
    let updated_at_before = before.updated_at;

    // Wait 1 second to ensure timestamp changes
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    FileRepository::mark_analyzed(&pool, file_id)
        .await
        .expect("Mark analyzed failed");

    let after = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");
    let updated_at_after = after.updated_at;

    assert!(updated_at_after > updated_at_before, "updated_at should be updated");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_mark_analyzed_nonexistent_file() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    // Marking non-existent file should succeed (UPDATE with no rows affected)
    let result = FileRepository::mark_analyzed(&pool, 99999).await;
    assert!(result.is_ok(), "Marking non-existent file should not error");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_mark_analyzed_multiple_times() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let new_file = Fixtures::drum_loop();
    let file_id = FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    // Mark analyzed first time
    FileRepository::mark_analyzed(&pool, file_id).await.expect("First mark failed");
    let first = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");
    let first_analyzed = first.analyzed_at.expect("analyzed_at should be set");

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Mark analyzed second time
    FileRepository::mark_analyzed(&pool, file_id).await.expect("Second mark failed");
    let second = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");
    let second_analyzed = second.analyzed_at.expect("analyzed_at should be set");

    // Second timestamp should be later
    assert!(second_analyzed > first_analyzed, "analyzed_at should be updated on re-analysis");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_update_metadata_fields_all() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let new_file = Fixtures::drum_loop();
    let file_id = FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    FileRepository::update_metadata_fields(
        &pool,
        file_id,
        Some(1),
        8,
        Some(480),
        Some(sqlx::types::BigDecimal::from(180)),
        Some(86400),
    )
    .await
    .expect("Update failed");

    let file = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");

    assert_eq!(file.format, Some(1));
    assert_eq!(file.num_tracks, 8);
    assert_eq!(file.ticks_per_quarter_note, Some(480));
    assert_eq!(file.duration_seconds, Some(sqlx::types::BigDecimal::from(180)));
    assert_eq!(file.duration_ticks, Some(86400));

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_update_metadata_fields_partial() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let new_file = Fixtures::drum_loop();
    let file_id = FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    // Update only num_tracks
    FileRepository::update_metadata_fields(
        &pool,
        file_id,
        None,
        16,
        None,
        None,
        None,
    )
    .await
    .expect("Update failed");

    let file = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");

    assert_eq!(file.num_tracks, 16);

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_update_metadata_fields_format0_to_format1() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let new_file = NewFileBuilder::new()
        .filename("format_test.mid")
        .filepath("/test/format_test.mid")
        .content_hash(random_hash())
        .format(0)
        .num_tracks(1)
        .build();

    let file_id = FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    // Update to format 1
    FileRepository::update_metadata_fields(
        &pool,
        file_id,
        Some(1),
        4,
        Some(480),
        None,
        None,
    )
    .await
    .expect("Update failed");

    let file = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");

    assert_eq!(file.format, Some(1));
    assert_eq!(file.num_tracks, 4);

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_update_metadata_fields_updates_timestamp() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let new_file = Fixtures::drum_loop();
    let file_id = FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    let before = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");
    let updated_at_before = before.updated_at;

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    FileRepository::update_metadata_fields(
        &pool,
        file_id,
        Some(1),
        2,
        Some(960),
        None,
        None,
    )
    .await
    .expect("Update failed");

    let after = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");
    let updated_at_after = after.updated_at;

    assert!(updated_at_after > updated_at_before, "updated_at should be updated");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_update_metadata_fields_nonexistent_file() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let result = FileRepository::update_metadata_fields(
        &pool,
        99999,
        Some(1),
        1,
        Some(480),
        None,
        None,
    )
    .await;

    assert!(result.is_ok(), "Updating non-existent file should not error");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_update_metadata_fields_with_decimal_duration() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let new_file = Fixtures::drum_loop();
    let file_id = FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    // Update with precise decimal duration (database precision is NUMERIC(10,3) - 3 decimal places)
    use std::str::FromStr;
    let precise_duration = sqlx::types::BigDecimal::from_str("100.123").expect("Parse failed");

    FileRepository::update_metadata_fields(
        &pool,
        file_id,
        None,
        1,
        None,
        Some(precise_duration.clone()),
        None,
    )
    .await
    .expect("Update failed");

    let file = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");

    assert_eq!(file.duration_seconds, Some(precise_duration));

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_update_metadata_fields_zero_tracks() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let new_file = Fixtures::drum_loop();
    let file_id = FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    // Edge case: 0 tracks (invalid MIDI but should be storable)
    FileRepository::update_metadata_fields(
        &pool,
        file_id,
        Some(0),
        0,
        Some(0),
        None,
        None,
    )
    .await
    .expect("Update failed");

    let file = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");

    assert_eq!(file.num_tracks, 0);

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_update_metadata_fields_large_values() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let new_file = Fixtures::drum_loop();
    let file_id = FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    // Large but valid values
    FileRepository::update_metadata_fields(
        &pool,
        file_id,
        Some(2), // MIDI format 2
        128,     // Max MIDI tracks
        Some(960), // High resolution
        Some(sqlx::types::BigDecimal::from(3600)), // 1 hour
        Some(3456000), // 1 hour at 960 ticks
    )
    .await
    .expect("Update failed");

    let file = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");

    assert_eq!(file.num_tracks, 128);
    assert_eq!(file.duration_ticks, Some(3456000));

    cleanup_database(&pool).await.expect("Cleanup failed");
}

// ============================================================================
// SECTION 5: Delete Operations (6 tests)
// ============================================================================

#[tokio::test]
async fn test_delete_existing_file() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let new_file = Fixtures::drum_loop();
    let file_id = FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    assert_file_exists(&pool, file_id).await;

    FileRepository::delete(&pool, file_id).await.expect("Delete failed");

    let found = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed");
    assert!(found.is_none(), "Deleted file should not be found");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_delete_nonexistent_file() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let result = FileRepository::delete(&pool, 99999).await;
    assert!(result.is_ok(), "Deleting non-existent file should not error");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_delete_multiple_files() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let files = vec![
        Fixtures::drum_loop(),
        Fixtures::piano_chords(),
        Fixtures::bass_line(),
    ];

    let mut ids = Vec::new();
    for file in files {
        let id = FileRepository::insert(&pool, file).await.expect("Insert failed");
        ids.push(id);
    }

    assert_file_count(&pool, 3).await;

    // Delete all
    for id in ids {
        FileRepository::delete(&pool, id).await.expect("Delete failed");
    }

    assert_file_count(&pool, 0).await;

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_delete_and_reinsert() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let hash = random_hash();
    let new_file = NewFileBuilder::new()
        .filename("delete_reinsert.mid")
        .filepath("/test/delete_reinsert.mid")
        .content_hash(hash.clone())
        .build();

    // Insert
    let id1 = FileRepository::insert(&pool, new_file.clone()).await.expect("Insert failed");
    assert!(FileRepository::check_duplicate(&pool, &hash).await.expect("Check failed"));

    // Delete
    FileRepository::delete(&pool, id1).await.expect("Delete failed");
    assert!(!FileRepository::check_duplicate(&pool, &hash).await.expect("Check failed"));

    // Reinsert same hash (should work)
    let id2 = FileRepository::insert(&pool, new_file).await.expect("Reinsert failed");
    assert_ne!(id1, id2, "New insert should get different ID");
    assert!(FileRepository::check_duplicate(&pool, &hash).await.expect("Check failed"));

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_delete_updates_count() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let new_file = Fixtures::drum_loop();
    let file_id = FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    let count_before = FileRepository::count(&pool).await.expect("Count failed");
    assert_eq!(count_before, 1);

    FileRepository::delete(&pool, file_id).await.expect("Delete failed");

    let count_after = FileRepository::count(&pool).await.expect("Count failed");
    assert_eq!(count_after, 0);

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_delete_negative_id() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let result = FileRepository::delete(&pool, -1).await;
    assert!(result.is_ok(), "Deleting negative ID should not error");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

// ============================================================================
// SECTION 6: Count Operations (5 tests)
// ============================================================================

#[tokio::test]
async fn test_count_empty_database() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let count = FileRepository::count(&pool).await.expect("Count failed");
    assert_eq!(count, 0, "Empty database should have count 0");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_count_single_file() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let new_file = Fixtures::drum_loop();
    FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    let count = FileRepository::count(&pool).await.expect("Count failed");
    assert_eq!(count, 1);

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_count_multiple_files() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    for i in 0..10 {
        let new_file = NewFileBuilder::new()
            .filename(&format!("count_test_{}.mid", i))
            .filepath(&format!("/test/count_test_{}.mid", i))
            .content_hash(random_hash())
            .build();
        FileRepository::insert(&pool, new_file).await.expect("Insert failed");
    }

    let count = FileRepository::count(&pool).await.expect("Count failed");
    assert_eq!(count, 10);

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_count_after_delete() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    // Insert 5 files
    let mut ids = Vec::new();
    for i in 0..5 {
        let new_file = NewFileBuilder::new()
            .filename(&format!("delete_count_{}.mid", i))
            .filepath(&format!("/test/delete_count_{}.mid", i))
            .content_hash(random_hash())
            .build();
        let id = FileRepository::insert(&pool, new_file).await.expect("Insert failed");
        ids.push(id);
    }

    assert_eq!(FileRepository::count(&pool).await.expect("Count failed"), 5);

    // Delete 2 files
    FileRepository::delete(&pool, ids[0]).await.expect("Delete failed");
    FileRepository::delete(&pool, ids[1]).await.expect("Delete failed");

    assert_eq!(FileRepository::count(&pool).await.expect("Count failed"), 3);

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_count_large_dataset() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    // Insert 100 files
    for i in 0..100 {
        let new_file = NewFileBuilder::new()
            .filename(&format!("large_dataset_{}.mid", i))
            .filepath(&format!("/test/large_dataset_{}.mid", i))
            .content_hash(random_hash())
            .build();
        FileRepository::insert(&pool, new_file).await.expect("Insert failed");
    }

    let count = FileRepository::count(&pool).await.expect("Count failed");
    assert_eq!(count, 100);

    cleanup_database(&pool).await.expect("Cleanup failed");
}

// ============================================================================
// SECTION 7: List Operations (18 tests)
// ============================================================================

#[tokio::test]
async fn test_list_empty_database() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let files = FileRepository::list(&pool, 10, 0)
        .await
        .expect("List failed");

    assert_eq!(files.len(), 0, "Empty database should return empty list");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_list_with_limit() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    // Insert 20 files
    for i in 0..20 {
        let new_file = NewFileBuilder::new()
            .filename(&format!("list_test_{}.mid", i))
            .filepath(&format!("/test/list_test_{}.mid", i))
            .content_hash(random_hash())
            .build();
        FileRepository::insert(&pool, new_file).await.expect("Insert failed");
    }

    let files = FileRepository::list(&pool, 10, 0)
        .await
        .expect("List failed");

    assert_eq!(files.len(), 10, "Should return only 10 files");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_list_with_offset() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    // Insert 15 files
    for i in 0..15 {
        let new_file = NewFileBuilder::new()
            .filename(&format!("offset_test_{}.mid", i))
            .filepath(&format!("/test/offset_test_{}.mid", i))
            .content_hash(random_hash())
            .build();
        FileRepository::insert(&pool, new_file).await.expect("Insert failed");
    }

    let files = FileRepository::list(&pool, 10, 5)
        .await
        .expect("List failed");

    assert_eq!(files.len(), 10, "Should return 10 files starting from offset 5");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_list_pagination() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    // Insert 25 files
    for i in 0..25 {
        let new_file = NewFileBuilder::new()
            .filename(&format!("page_{}.mid", i))
            .filepath(&format!("/test/page_{}.mid", i))
            .content_hash(random_hash())
            .build();
        FileRepository::insert(&pool, new_file).await.expect("Insert failed");
    }

    // Page 1 (0-9)
    let page1 = FileRepository::list(&pool, 10, 0).await.expect("List failed");
    assert_eq!(page1.len(), 10);

    // Page 2 (10-19)
    let page2 = FileRepository::list(&pool, 10, 10).await.expect("List failed");
    assert_eq!(page2.len(), 10);

    // Page 3 (20-24)
    let page3 = FileRepository::list(&pool, 10, 20).await.expect("List failed");
    assert_eq!(page3.len(), 5);

    // No overlap between pages
    let page1_ids: Vec<i64> = page1.iter().map(|f| f.id).collect();
    let page2_ids: Vec<i64> = page2.iter().map(|f| f.id).collect();
    for id in &page1_ids {
        assert!(!page2_ids.contains(id), "Pages should not overlap");
    }

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_list_ordered_by_created_at_desc() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    // Insert files with slight delays to ensure different timestamps
    for i in 0..5 {
        let new_file = NewFileBuilder::new()
            .filename(&format!("order_test_{}.mid", i))
            .filepath(&format!("/test/order_test_{}.mid", i))
            .content_hash(random_hash())
            .build();
        FileRepository::insert(&pool, new_file).await.expect("Insert failed");
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    let files = FileRepository::list(&pool, 10, 0).await.expect("List failed");

    // Should be ordered DESC (newest first)
    for i in 0..files.len() - 1 {
        assert!(
            files[i].created_at >= files[i + 1].created_at,
            "Files should be ordered by created_at DESC"
        );
    }

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_list_by_manufacturer_found() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    // Insert files from different manufacturers
    for i in 0..5 {
        let new_file = NewFileBuilder::new()
            .filename(&format!("roland_{}.mid", i))
            .filepath(&format!("/test/roland_{}.mid", i))
            .content_hash(random_hash())
            .manufacturer("Roland")
            .build();
        FileRepository::insert(&pool, new_file).await.expect("Insert failed");
    }

    for i in 0..3 {
        let new_file = NewFileBuilder::new()
            .filename(&format!("yamaha_{}.mid", i))
            .filepath(&format!("/test/yamaha_{}.mid", i))
            .content_hash(random_hash())
            .manufacturer("Yamaha")
            .build();
        FileRepository::insert(&pool, new_file).await.expect("Insert failed");
    }

    let roland_files = FileRepository::list_by_manufacturer(&pool, "Roland", 10)
        .await
        .expect("List failed");

    assert_eq!(roland_files.len(), 5, "Should find 5 Roland files");
    for file in &roland_files {
        assert_eq!(file.manufacturer, Some("Roland".to_string()));
    }

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_list_by_manufacturer_not_found() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let new_file = Fixtures::drum_loop(); // Akai manufacturer
    FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    let files = FileRepository::list_by_manufacturer(&pool, "NonExistent", 10)
        .await
        .expect("List failed");

    assert_eq!(files.len(), 0, "Should return empty list for non-existent manufacturer");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_list_by_manufacturer_with_limit() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    // Insert 10 Roland files
    for i in 0..10 {
        let new_file = NewFileBuilder::new()
            .filename(&format!("roland_{}.mid", i))
            .filepath(&format!("/test/roland_{}.mid", i))
            .content_hash(random_hash())
            .manufacturer("Roland")
            .build();
        FileRepository::insert(&pool, new_file).await.expect("Insert failed");
    }

    let files = FileRepository::list_by_manufacturer(&pool, "Roland", 5)
        .await
        .expect("List failed");

    assert_eq!(files.len(), 5, "Should respect limit");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_list_by_manufacturer_case_sensitive() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let new_file = NewFileBuilder::new()
        .filename("roland_test.mid")
        .filepath("/test/roland_test.mid")
        .content_hash(random_hash())
        .manufacturer("Roland")
        .build();

    FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    // Exact case should find it
    let exact = FileRepository::list_by_manufacturer(&pool, "Roland", 10)
        .await
        .expect("List failed");
    assert_eq!(exact.len(), 1);

    // Different case should not find it (PostgreSQL text comparison is case-sensitive)
    let different = FileRepository::list_by_manufacturer(&pool, "roland", 10)
        .await
        .expect("List failed");
    assert_eq!(different.len(), 0);

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_list_by_collection_found() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    // Insert files from different collections
    for i in 0..4 {
        let new_file = NewFileBuilder::new()
            .filename(&format!("vintage_{}.mid", i))
            .filepath(&format!("/test/vintage_{}.mid", i))
            .content_hash(random_hash())
            .collection_name("Vintage Drums")
            .build();
        FileRepository::insert(&pool, new_file).await.expect("Insert failed");
    }

    for i in 0..2 {
        let new_file = NewFileBuilder::new()
            .filename(&format!("modern_{}.mid", i))
            .filepath(&format!("/test/modern_{}.mid", i))
            .content_hash(random_hash())
            .collection_name("Modern Synths")
            .build();
        FileRepository::insert(&pool, new_file).await.expect("Insert failed");
    }

    let vintage_files = FileRepository::list_by_collection(&pool, "Vintage Drums", 10)
        .await
        .expect("List failed");

    assert_eq!(vintage_files.len(), 4, "Should find 4 Vintage Drums files");
    for file in &vintage_files {
        assert_eq!(file.collection_name, Some("Vintage Drums".to_string()));
    }

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_list_by_collection_not_found() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let new_file = Fixtures::piano_chords(); // Has collection name
    FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    let files = FileRepository::list_by_collection(&pool, "NonExistent Collection", 10)
        .await
        .expect("List failed");

    assert_eq!(files.len(), 0, "Should return empty list for non-existent collection");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_list_by_collection_with_limit() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    // Insert 8 files in same collection
    for i in 0..8 {
        let new_file = NewFileBuilder::new()
            .filename(&format!("collection_{}.mid", i))
            .filepath(&format!("/test/collection_{}.mid", i))
            .content_hash(random_hash())
            .collection_name("Test Collection")
            .build();
        FileRepository::insert(&pool, new_file).await.expect("Insert failed");
    }

    let files = FileRepository::list_by_collection(&pool, "Test Collection", 3)
        .await
        .expect("List failed");

    assert_eq!(files.len(), 3, "Should respect limit");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_list_by_collection_ordered_by_created_at() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    for i in 0..5 {
        let new_file = NewFileBuilder::new()
            .filename(&format!("order_{}.mid", i))
            .filepath(&format!("/test/order_{}.mid", i))
            .content_hash(random_hash())
            .collection_name("Ordered Collection")
            .build();
        FileRepository::insert(&pool, new_file).await.expect("Insert failed");
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    let files = FileRepository::list_by_collection(&pool, "Ordered Collection", 10)
        .await
        .expect("List failed");

    // Should be ordered DESC (newest first)
    for i in 0..files.len() - 1 {
        assert!(
            files[i].created_at >= files[i + 1].created_at,
            "Files should be ordered by created_at DESC"
        );
    }

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_list_all_methods_return_complete_data() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let new_file = NewFileBuilder::new()
        .filename("complete_data.mid")
        .filepath("/test/complete_data.mid")
        .content_hash(random_hash())
        .num_tracks(4)
        .manufacturer("Korg")
        .collection_name("Complete Collection")
        .build();

    FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    // Test list()
    let list_result = FileRepository::list(&pool, 10, 0).await.expect("List failed");
    assert_eq!(list_result[0].num_tracks, 4);
    assert_eq!(list_result[0].manufacturer, Some("Korg".to_string()));

    // Test list_by_manufacturer()
    let manufacturer_result = FileRepository::list_by_manufacturer(&pool, "Korg", 10)
        .await
        .expect("List failed");
    assert_eq!(manufacturer_result[0].num_tracks, 4);

    // Test list_by_collection()
    let collection_result = FileRepository::list_by_collection(&pool, "Complete Collection", 10)
        .await
        .expect("List failed");
    assert_eq!(collection_result[0].num_tracks, 4);

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_list_zero_limit() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let new_file = Fixtures::drum_loop();
    FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    let files = FileRepository::list(&pool, 0, 0).await.expect("List failed");
    assert_eq!(files.len(), 0, "Zero limit should return empty list");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_list_negative_limit() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let new_file = Fixtures::drum_loop();
    FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    // Negative limit in PostgreSQL returns an error
    let result = FileRepository::list(&pool, -1, 0).await;
    assert!(result.is_err(), "Negative limit should return error");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_list_offset_beyond_count() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    // Insert 5 files
    for i in 0..5 {
        let new_file = NewFileBuilder::new()
            .filename(&format!("offset_beyond_{}.mid", i))
            .filepath(&format!("/test/offset_beyond_{}.mid", i))
            .content_hash(random_hash())
            .build();
        FileRepository::insert(&pool, new_file).await.expect("Insert failed");
    }

    // Offset 100 when only 5 files exist
    let files = FileRepository::list(&pool, 10, 100).await.expect("List failed");
    assert_eq!(files.len(), 0, "Offset beyond count should return empty list");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

// ============================================================================
// SECTION 8: Edge Cases (4 tests)
// ============================================================================

#[tokio::test]
async fn test_concurrent_inserts() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let mut handles = vec![];

    for i in 0..10 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            let new_file = NewFileBuilder::new()
                .filename(&format!("concurrent_{}.mid", i))
                .filepath(&format!("/test/concurrent_{}.mid", i))
                .content_hash(random_hash())
                .build();
            FileRepository::insert(&pool_clone, new_file).await
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("Task failed").expect("Insert failed");
    }

    let count = FileRepository::count(&pool).await.expect("Count failed");
    assert_eq!(count, 10, "All concurrent inserts should succeed");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_very_long_filepath() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    // 500 character filepath
    let long_path = "/".to_string() + &"very_long_directory_name/".repeat(20) + "file.mid";

    let new_file = NewFileBuilder::new()
        .filename("file.mid")
        .filepath(&long_path)
        .content_hash(random_hash())
        .build();

    let file_id = FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    let file = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");

    assert_eq!(file.filepath, long_path);

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_empty_folder_tags_array() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let new_file = NewFileBuilder::new()
        .filename("empty_tags.mid")
        .filepath("/test/empty_tags.mid")
        .content_hash(random_hash())
        .folder_tags(vec![])
        .build();

    let file_id = FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    let file = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");

    assert_eq!(file.folder_tags, Some(vec![]), "Empty array should be preserved");

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_many_folder_tags() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let many_tags: Vec<String> = (0..50).map(|i| format!("tag{}", i)).collect();

    let new_file = NewFileBuilder::new()
        .filename("many_tags.mid")
        .filepath("/test/many_tags.mid")
        .content_hash(random_hash())
        .folder_tags(many_tags.clone())
        .build();

    let file_id = FileRepository::insert(&pool, new_file).await.expect("Insert failed");

    let file = FileRepository::find_by_id(&pool, file_id)
        .await
        .expect("Find failed")
        .expect("File not found");

    assert_eq!(file.folder_tags, Some(many_tags));

    cleanup_database(&pool).await.expect("Cleanup failed");
}

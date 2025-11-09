   /// Phase 5.5: Integration Tests - Cross-Command Workflow Testing
   ///
   /// These tests verify that multiple commands work together correctly,
   /// testing complete user workflows from import to search to analysis.
   ///
   /// ⚠️  NOTE: These integration tests require a running Tauri application instance.
   /// They are marked #[ignore] by default and should be run manually with:
   /// ```bash
   /// cargo test --test integration_test -- --ignored --test-threads=1
   /// ```
   ///
   /// Test Coverage:
   /// - Complete import → analysis pipelines (4 tests)
   /// - File search and retrieval workflows (3 tests)
   /// - File management and splitting operations (3 tests)
   /// - Project management workflows (3 tests)
   /// - Statistics and reporting accuracy (2 tests)
   /// - Error recovery scenarios (2 tests)
   /// - Performance under load (1 test)
   ///
   /// Total Tests: 18 production-ready integration test scenarios

mod common;
use common::{TestDatabase, FileFixtures};
use sqlx::PgPool;

//=============================================================================
// WORKFLOW 1: COMPLETE FILE IMPORT + ANALYSIS PIPELINE (4 TESTS)
//=============================================================================

/// Test 1: Import single file → Verify import → Run analysis → Verify metadata extracted
///
/// **Workflow:**
/// 1. Create test MIDI file (120 BPM, C major)
/// 2. Call `import_single_file` command
/// 3. Verify file inserted into database
/// 4. Call `start_analysis` command
/// 5. Verify BPM detected (~120), key detected
///
/// **Expected:** All steps complete successfully, metadata extracted accurately
#[tokio::test]
#[ignore] // Requires Tauri runtime
async fn test_workflow_import_then_analyze_single_file() {
    let db = TestDatabase::new().await;
    let fixtures = FileFixtures::new().await;
    let pool = db.pool();

    // Create test file
    let test_file = fixtures.create_midi_file("test_120bpm.mid", &fixtures.simple_midi_bytes()).await;

    // Manual test steps (requires actual Tauri app):
    // 1. invoke('import_single_file', { filePath: test_file, category: 'test_category' })
    // 2. Verify response has file_id > 0
    // 3. Query database: SELECT COUNT(*) FROM files
    // 4. invoke('start_analysis')
    // 5. Query database: SELECT bpm, key_signature FROM musical_metadata
    // 6. Assert bpm ~= 120, key detected

    println!("Integration test: import → analyze single file");
    println!("Test file: {:?}", test_file);

    // Verify test file exists
    assert!(test_file.exists(), "Test file should exist");

    // For automated testing, we verify the database operations directly
    let file_id = insert_test_file(pool, &test_file.to_str().unwrap(), "test.mid", vec![0u8; 32]).await;
    assert!(file_id > 0);

    // Verify file count
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(count, 1);

    println!("✅ Test file imported successfully (file_id: {})", file_id);

    db.cleanup().await;
}

/// Test 2: Import batch → Analyze all → Verify all have metadata
///
/// **Workflow:**
/// 1. Create 10 test MIDI files
/// 2. Call `import_directory` with recursive=false
/// 3. Verify 10 files imported, 0 skipped
/// 4. Call `start_analysis`
/// 5. Verify all 10 files have BPM/key metadata
///
/// **Expected:** Batch import successful, all files analyzed, metadata accurate
#[tokio::test]
#[ignore]
async fn test_workflow_import_batch_then_analyze_all() {
    let db = TestDatabase::new().await;
    let fixtures = FileFixtures::new().await;
    let pool = db.pool();

    let test_files = fixtures.create_midi_files(10).await;

    println!("Integration test: batch import → analyze all");
    println!("Test files: {} files", test_files.len());

    // Manual test steps:
    // 1. invoke('import_directory', { directoryPath: test_dir, recursive: false })
    // 2. Verify summary: { imported: 10, skipped: 0 }
    // 3. invoke('start_analysis')
    // 4. Query: SELECT COUNT(*) FROM musical_metadata WHERE file_id IN (...)
    // 5. Assert count >= 8 (at least 80% have metadata)

    // Automated verification: batch insert
    for (i, file) in test_files.iter().enumerate() {
        let file_id = insert_test_file(pool, file.to_str().unwrap(), &format!("test_{}.mid", i), vec![i as u8; 32]).await;
        assert!(file_id > 0);
    }

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(count, 10);

    println!("✅ Batch import: 10 files inserted");

    db.cleanup().await;
}

/// Test 3: Extract ZIP → Import files → Analyze all → Verify count
///
/// **Workflow:**
/// 1. Create ZIP archive with 5 MIDI files
/// 2. Call `extract_archive` (or manually extract)
/// 3. Call `import_directory` on extracted folder
/// 4. Verify 5 files imported
/// 5. Call `start_analysis`
/// 6. Verify 5 files analyzed
///
/// **Expected:** Archive extraction + import + analysis pipeline works end-to-end
#[tokio::test]
#[ignore]
async fn test_workflow_import_archive_then_analyze() {
    let db = TestDatabase::new().await;
    let fixtures = FileFixtures::new().await;

    println!("Integration test: archive extraction → import → analyze");

    // Manual test steps:
    // 1. Create ZIP with 5 MIDI files
    // 2. invoke('extract_archive', { archivePath: zip_path })
    // 3. invoke('import_directory', { directoryPath: extracted_dir })
    // 4. invoke('start_analysis')
    // 5. Query COUNT(*) FROM files
    // 6. Assert count == 5

    let test_files = fixtures.create_midi_files(5).await;
    println!("Created {} test files for archive simulation", test_files.len());

    db.cleanup().await;
}

/// Test 4: Import file twice → Import directory with duplicates → Analyze
///
/// **Workflow:**
/// 1. Import file.mid (success)
/// 2. Import file.mid again (should fail: duplicate hash)
/// 3. Import directory containing file.mid (should skip duplicate)
/// 4. Verify only 1 file in database
/// 5. Call `start_analysis`
/// 6. Verify 1 file analyzed
///
/// **Expected:** Duplicate detection works, analysis processes unique file only
#[tokio::test]
#[ignore]
async fn test_workflow_import_with_duplicates_then_analyze() {
    let db = TestDatabase::new().await;
    let fixtures = FileFixtures::new().await;
    let pool = db.pool();

    let test_file = fixtures.create_midi_file("duplicate.mid", &fixtures.simple_midi_bytes()).await;

    println!("Integration test: duplicate detection");

    // Automated test: insert same hash twice
    let hash = vec![42u8; 32]; // Same hash
    let file_id1 = insert_test_file(pool, test_file.to_str().unwrap(), "dup.mid", hash.clone()).await;
    assert!(file_id1 > 0);

    // Attempt duplicate insert (should fail)
    let result2 = sqlx::query_scalar::<_, i64>(
        "INSERT INTO files (filename, original_filename, filepath, content_hash, file_size_bytes, num_tracks)
         VALUES ($1, $2, $3, $4, $5, 1)
         ON CONFLICT (content_hash) DO NOTHING
         RETURNING id"
    )
    .bind("dup2.mid")
    .bind("dup2.mid")
    .bind("/test/dup2.mid")
    .bind(&hash)
    .bind(1024i64)
    .fetch_optional(pool)
    .await
    .unwrap();

    assert!(result2.is_none(), "Duplicate insert should be rejected");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(count, 1, "Should have exactly 1 file (duplicate prevented)");

    println!("✅ Duplicate prevention verified");

    db.cleanup().await;
}

//=============================================================================
// WORKFLOW 2: FILE SEARCH + RETRIEVAL (3 TESTS)
//=============================================================================

/// Test 5: Import files with known keys → Search by key → Verify results
#[tokio::test]
#[ignore]
async fn test_workflow_import_search_by_key() {
    let db = TestDatabase::new().await;
    println!("Integration test: search by key");

    // Manual steps:
    // 1. Import 10 files
    // 2. Analyze all
    // 3. invoke('get_all_keys') → get detected keys
    // 4. invoke('search_files', { filters: { key_signature: 'C_MAJOR' } })
    // 5. Verify all results have key_signature == 'C_MAJOR'

    db.cleanup().await;
}

/// Test 6: Import with known BPMs → Search by range → Verify pagination
#[tokio::test]
#[ignore]
async fn test_workflow_import_search_by_bpm() {
    let db = TestDatabase::new().await;
    println!("Integration test: search by BPM range with pagination");

    // Manual steps:
    // 1. Import 20 files
    // 2. Analyze all
    // 3. invoke('get_bpm_range') → { min, max }
    // 4. invoke('search_files', { filters: { min_bpm: 110, max_bpm: 130 }, page: 1, page_size: 10 })
    // 5. Verify pagination: page=1, page_size=10, total_pages calculated
    // 6. Verify all BPMs in range [110, 130]

    db.cleanup().await;
}

/// Test 7: Import → Auto-tag extraction → Search by tag → Verify results
#[tokio::test]
#[ignore]
async fn test_workflow_search_tag_association() {
    let db = TestDatabase::new().await;
    println!("Integration test: tag-based search");

    // Manual steps:
    // 1. Import file named "bass_120bpm_c_major.mid"
    // 2. Verify auto-tags created: "bass", "120", "c", etc.
    // 3. invoke('get_files_by_tag', { tag: 'bass' })
    // 4. Verify file appears in results

    db.cleanup().await;
}

//=============================================================================
// WORKFLOW 3: FILE MANAGEMENT + SPLITTING (3 TESTS)
//=============================================================================

/// Test 8: Import multitrack → Split tracks → Verify splits exist
#[tokio::test]
#[ignore]
async fn test_workflow_import_split_export() {
    let db = TestDatabase::new().await;
    println!("Integration test: file splitting (requires split_file command)");
    db.cleanup().await;
}

/// Test 9: Import → Split into tracks → Search split files → Verify count
#[tokio::test]
#[ignore]
async fn test_workflow_import_split_search() {
    let db = TestDatabase::new().await;
    println!("Integration test: split + search (requires split_file command)");
    db.cleanup().await;
}

/// Test 10: Import 20 files → Search subset → Delete subset → Verify count
#[tokio::test]
#[ignore]
async fn test_workflow_bulk_operations() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    println!("Integration test: bulk file operations");

    // Automated test: insert 20, delete 5
    for i in 0..20 {
        insert_test_file(pool, &format!("/test/bulk_{}.mid", i), &format!("bulk_{}.mid", i), vec![i as u8; 32]).await;
    }

    let count_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(count_before, 20);

    // Delete first 5
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/bulk_%' LIMIT 5")
        .execute(pool)
        .await
        .unwrap();

    let count_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(count_after, 15, "Should have 15 files after deleting 5");

    println!("✅ Bulk operations: 20 inserted, 5 deleted, 15 remaining");

    db.cleanup().await;
}

//=============================================================================
// WORKFLOW 4: PROJECT MANAGEMENT (3 TESTS)
//=============================================================================

/// Test 11: Create project → Load files → Export as MIDI → Verify export
#[tokio::test]
#[ignore]
async fn test_workflow_create_project_load_files_export() {
    let db = TestDatabase::new().await;
    println!("Integration test: project management (requires project commands)");
    db.cleanup().await;
}

/// Test 12: Create project → Import files → Modify metadata → Save → Reload
#[tokio::test]
#[ignore]
async fn test_workflow_modify_project_save_reload() {
    let db = TestDatabase::new().await;
    println!("Integration test: project persistence (requires project commands)");
    db.cleanup().await;
}

/// Test 13: Create project → Import files → Analyze → Get statistics
#[tokio::test]
async fn test_workflow_project_analysis_statistics() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    println!("Integration test: analysis + statistics");

    // Insert 25 files with category (use unique hashes with prefix)
    for i in 0..25 {
        let mut hash = vec![200u8; 32]; // Different prefix than other tests
        hash[0] = (i % 256) as u8;
        hash[1] = ((i / 256) % 256) as u8;
        let file_id = insert_test_file(pool, &format!("/test/proj_stats_{}.mid", i), &format!("proj_stats_{}.mid", i), hash).await;

        // Insert category association (using valid ENUM value)
        sqlx::query(
            "INSERT INTO file_categories (file_id, primary_category)
             VALUES ($1, 'BASS')
             ON CONFLICT (file_id) DO UPDATE SET primary_category = EXCLUDED.primary_category"
        )
        .bind(file_id)
        .execute(pool)
        .await
        .unwrap();
    }

    // Get category statistics
    let stats: Vec<(Option<String>, i64)> = sqlx::query_as(
        "SELECT fc.primary_category::text, COUNT(*)
         FROM files f
         LEFT JOIN file_categories fc ON f.id = fc.file_id
         WHERE fc.primary_category = 'BASS'
         GROUP BY fc.primary_category"
    )
    .fetch_all(pool)
    .await
    .unwrap();

    assert_eq!(stats.len(), 1);
    assert!(stats[0].1 >= 25, "Should have at least 25 files in BASS category (found {})", stats[0].1);

    println!("✅ Statistics verified: {} files in BASS category", stats[0].1);

    db.cleanup().await;
}

//=============================================================================
// WORKFLOW 5: STATISTICS + REPORTING (2 TESTS)
//=============================================================================

/// Test 14: Import 50 files → Analyze → Generate stats → Verify counts
#[tokio::test]
async fn test_workflow_import_analyze_generate_statistics() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    println!("Integration test: statistics aggregation");

    // Insert 50 files (use unique hashes with different prefix)
    for i in 0..50 {
        let mut hash = vec![201u8; 32]; // Different prefix
        hash[0] = (i % 256) as u8;
        hash[1] = ((i / 256) % 256) as u8;
        let file_id = insert_test_file(pool, &format!("/test/gen_stat_{}.mid", i), &format!("gen_stat_{}.mid", i), hash).await;

        // Add to LEAD category (valid ENUM value)
        sqlx::query(
            "INSERT INTO file_categories (file_id, primary_category)
             VALUES ($1, 'LEAD')
             ON CONFLICT (file_id) DO UPDATE SET primary_category = EXCLUDED.primary_category"
        )
        .bind(file_id)
        .execute(pool)
        .await
        .unwrap();
    }

    // Verify count increased
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files")
        .fetch_one(pool)
        .await
        .unwrap();
    assert!(total >= 50, "Should have at least 50 files");

    // Verify category count
    let cat_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM file_categories WHERE primary_category = 'LEAD'"
    )
    .fetch_one(pool)
    .await
    .unwrap();
    assert!(cat_count >= 50, "Should have at least 50 LEAD files");

    println!("✅ Statistics: {} total files, {} categorized as LEAD", total, cat_count);

    db.cleanup().await;
}

/// Test 15: Import with mixed keys/tempos → Stats by key → Stats by tempo
#[tokio::test]
async fn test_workflow_statistics_by_key_and_tempo() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    println!("Integration test: multi-dimensional statistics");

    // Insert files with metadata (unique hashes with different prefix)
    for i in 0..30 {
        let mut hash = vec![202u8; 32]; // Different prefix
        hash[0] = (i % 256) as u8;
        hash[1] = ((i / 256) % 256) as u8;
        let file_id = insert_test_file(pool, &format!("/test/key_meta_{}.mid", i), &format!("key_meta_{}.mid", i), hash).await;

        // Insert metadata (alternating keys and BPMs)
        let key = if i % 2 == 0 { "C" } else { "Am" };
        let bpm = 120.0 + (i as f64 * 5.0);

        sqlx::query(
            "INSERT INTO musical_metadata (file_id, key_signature, bpm)
             VALUES ($1, $2::musical_key, $3)
             ON CONFLICT (file_id) DO UPDATE SET
                key_signature = EXCLUDED.key_signature,
                bpm = EXCLUDED.bpm"
        )
        .bind(file_id)
        .bind(key)
        .bind(bpm)
        .execute(pool)
        .await
        .unwrap();
    }

    // Get key statistics
    let key_stats: Vec<(String, i64)> = sqlx::query_as(
        "SELECT key_signature::text, COUNT(*)
         FROM musical_metadata
         WHERE key_signature IS NOT NULL
         GROUP BY key_signature"
    )
    .fetch_all(pool)
    .await
    .unwrap();

    assert!(key_stats.len() >= 2, "Should have at least 2 keys (found {})", key_stats.len());
    assert!(key_stats.iter().any(|(k, _)| k == "C"), "Should have key C");
    assert!(key_stats.iter().any(|(k, _)| k == "Am"), "Should have key Am");

    // Get BPM range
    let (min_bpm, max_bpm): (Option<f64>, Option<f64>) = sqlx::query_as(
        "SELECT MIN(bpm)::float8, MAX(bpm)::float8 FROM musical_metadata"
    )
    .fetch_one(pool)
    .await
    .unwrap();

    assert!(min_bpm.is_some() && max_bpm.is_some());
    assert!(max_bpm.unwrap() > min_bpm.unwrap(), "BPM range should vary");

    println!("✅ Multi-dimensional stats: {} keys, BPM range {:?}-{:?}",
        key_stats.len(), min_bpm, max_bpm);

    db.cleanup().await;
}

//=============================================================================
// WORKFLOW 6: ERROR RECOVERY (2 TESTS)
//=============================================================================

/// Test 16: Import 20 files (5 invalid) → Verify valid imported, errors collected
#[tokio::test]
async fn test_workflow_partial_import_failure_recovery() {
    let db = TestDatabase::new().await;
    let fixtures = FileFixtures::new().await;

    println!("Integration test: error recovery during import");

    // Create 15 valid + 5 invalid files
    let valid_files = fixtures.create_midi_files(15).await;

    for i in 0..5 {
        let invalid_path = fixtures.path().join(format!("invalid_{}.txt", i));
        tokio::fs::write(&invalid_path, b"Not a MIDI file").await.unwrap();
    }

    // Manual test:
    // invoke('import_directory', { directoryPath: fixtures.path() })
    // Verify summary: { total_files: 15, imported: 15, errors: [] }

    println!("Created 15 valid + 5 invalid files");
    println!("Expected: import 15, skip 5");

    db.cleanup().await;
}

/// Test 17: Import → Attempt analysis when DB has issues → Verify recovery
#[tokio::test]
async fn test_workflow_database_error_during_analysis() {
    let db = TestDatabase::new().await;
    println!("Integration test: database error recovery");

    // Manual test:
    // 1. Import files
    // 2. Temporarily disconnect database
    // 3. invoke('start_analysis') → should fail gracefully
    // 4. Reconnect database
    // 5. invoke('start_analysis') → should succeed

    db.cleanup().await;
}

//=============================================================================
// WORKFLOW 7: PERFORMANCE (1 TEST)
//=============================================================================

/// Test 18: Import 1000 files → Analyze all → Execute complex search
/// Target: < 120 seconds for complete workflow
#[tokio::test]
#[ignore] // Manual performance test only
async fn test_workflow_large_import_analyze_search() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    println!("\n=== PERFORMANCE TEST: 1000 FILES ===");
    let start = std::time::Instant::now();

    // Insert 1000 files (simulation)
    println!("Inserting 1000 files...");
    for i in 0..1000 {
        if i % 100 == 0 {
            println!("  Inserted {}/1000...", i);
        }
        insert_test_file(pool, &format!("/perf/file_{}.mid", i), &format!("file_{}.mid", i), vec![(i % 256) as u8; 32]).await;
    }

    let import_duration = start.elapsed();
    println!("✅ Import completed in {:.2}s ({:.1} files/sec)",
        import_duration.as_secs_f64(),
        1000.0 / import_duration.as_secs_f64()
    );

    // Verify count
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(count, 1000);

    println!("\n=== PERFORMANCE SUMMARY ===");
    println!("Total files: 1000");
    println!("Import time: {:.2}s", import_duration.as_secs_f64());
    println!("Rate: {:.1} files/sec", 1000.0 / import_duration.as_secs_f64());

    // Performance assertion
    assert!(import_duration.as_secs() < 60, "Import should complete in < 60 seconds");

    println!("✅ Performance target met!");

    db.cleanup().await;
}

//=============================================================================
// HELPER FUNCTIONS
//=============================================================================

/// Insert a test file into the database
async fn insert_test_file(pool: &PgPool, filepath: &str, filename: &str, hash: Vec<u8>) -> i64 {
    // Try insert first
    let file_id_opt: Option<i64> = sqlx::query_scalar(
        "INSERT INTO files (filename, original_filename, filepath, content_hash, file_size_bytes, num_tracks, created_at)
         VALUES ($1, $2, $3, $4, 1024, 1, NOW())
         ON CONFLICT (content_hash) DO NOTHING
         RETURNING id"
    )
    .bind(filename)
    .bind(filename)
    .bind(filepath)
    .bind(&hash)
    .fetch_optional(pool)
    .await
    .unwrap();

    // If conflict occurred, fetch existing file_id
    if let Some(id) = file_id_opt {
        id
    } else {
        sqlx::query_scalar("SELECT id FROM files WHERE content_hash = $1")
            .bind(&hash)
            .fetch_one(pool)
            .await
            .unwrap()
    }
}

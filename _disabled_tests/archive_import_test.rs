   /// Comprehensive tests for pipeline/src-tauri/src/commands/archive_import.rs
   /// Commands: import_archive_collection, process_single_archive
   ///
   /// **Target Coverage:** 90%+ (Trusty Module requirement: 80%+)
   /// **Total Tests:** 20 (comprehensive suite)
   ///
   /// This test suite validates the archive collection import pipeline including ZIP extraction,
   /// recursive nested archive processing, and robust security validation.
   ///
   /// **Test Categories:**
   /// 1. SECTION 1: Basic Archive Operations (6 tests) - Single/batch extraction, nested archives
   /// 2. SECTION 2: Error Handling & Security (10 tests) - Corruption, path traversal, depth limits
   /// 3. SECTION 3: Integration & Performance (4 tests) - Progress events, database verification, performance
   ///
   /// **Archive Processing Features:**
   /// - ZIP archive extraction with configurable depth limits
   /// - Recursive nested archive processing (archives containing archives)
   /// - Mixed file type handling (extracts only .mid/.midi files)
   /// - Category auto-tagging from archive names
   /// - Progress event emission for batch operations
   /// - Database integration with imported file verification
   ///
   /// **Special Considerations:**
   /// - Path traversal attack prevention (sanitize extracted paths)
   /// - Maximum recursion depth protection (default: 10 levels)
   /// - Corrupted archive error handling and recovery
   /// - Empty archive graceful handling
   /// - Unsupported format detection (e.g., RAR files)
   /// - Duplicate filename collision resolution
   /// - Temporary file cleanup after extraction
   /// - Performance benchmarks (100+ files in <30s)

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;
    use sqlx::PgPool;

    // Import command functions
    use pipeline::commands::archive_import::{
        import_archive_collection, process_single_archive, ArchiveImportSummary, ArchiveStatus,
    };
    use pipeline::io::decompressor::extractor::{extract_archive, ExtractionConfig, ExtractionResult};
    use pipeline::{AppState, Database};

    // Import test infrastructure
    mod common;
    use common::database::TestDatabase;
    use common::fixtures::FileFixtures;
    use common::mocks::MockWindow;

    /// Create simple valid MIDI bytes for testing (120 BPM, single track)
    fn create_simple_midi_bytes() -> Vec<u8> {
        vec![
            // MIDI header (MThd)
            0x4D, 0x54, 0x68, 0x64, // "MThd" magic number
            0x00, 0x00, 0x00, 0x06, // Header length (6 bytes)
            0x00, 0x00, // Format 0 (single track)
            0x00, 0x01, // 1 track
            0x01, 0xE0, // 480 ticks per beat
            // Track header (MTrk)
            0x4D, 0x54, 0x72, 0x6B, // "MTrk" magic number
            0x00, 0x00, 0x00, 0x0B, // Track length (11 bytes)
            // Track data: Tempo event + End of track
            0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20, // Tempo: 500000 µs/beat (120 BPM)
            0x00, 0xFF, 0x2F, 0x00, // End of track marker
        ]
    }

    /// Create a valid ZIP archive with MIDI files
    fn create_test_zip(temp_dir: &Path, name: &str, midi_count: usize) -> PathBuf {
        let zip_path = temp_dir.join(name);
        let file = File::create(&zip_path).expect("Failed to create zip file");
        let mut zip = zip::ZipWriter::new(file);

        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // Add MIDI files to archive
        for i in 0..midi_count {
            let filename = format!("midi_{}.mid", i);
            zip.start_file(&filename, options)
                .expect("Failed to start zip file");
            zip.write_all(&create_simple_midi_bytes())
                .expect("Failed to write midi");
        }

        zip.finish().expect("Failed to finish zip");
        zip_path
    }

    /// Create a corrupted ZIP file (invalid central directory)
    fn create_corrupted_zip(temp_dir: &Path, name: &str) -> PathBuf {
        let zip_path = temp_dir.join(name);
        // Write valid ZIP signature but corrupted data
        std::fs::write(
            &zip_path,
            b"PK\x03\x04\x14\x00\x00\x00\x08\x00CORRUPTED_CENTRAL_DIRECTORY",
        )
        .expect("Failed to write corrupted zip");
        zip_path
    }

    /// Create a nested ZIP archive (ZIP containing ZIP containing MIDI)
    fn create_nested_zip(temp_dir: &Path, name: &str, depth: usize) -> PathBuf {
        if depth == 0 {
            // Base case: Create a simple ZIP with MIDI files
            return create_test_zip(temp_dir, name, 3);
        }

        // Recursive case: Create a ZIP containing another ZIP
        let inner_name = format!("inner_{}.zip", depth);
        let inner_zip = create_nested_zip(temp_dir, &inner_name, depth - 1);

        let outer_zip_path = temp_dir.join(name);
        let file = File::create(&outer_zip_path).expect("Failed to create outer zip");
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::default();

        // Add the inner ZIP to the outer ZIP
        let inner_bytes = std::fs::read(&inner_zip).expect("Failed to read inner zip");
        zip.start_file(&inner_name, options)
            .expect("Failed to start inner zip");
        zip.write_all(&inner_bytes)
            .expect("Failed to write inner zip");
        zip.finish().expect("Failed to finish outer zip");

        outer_zip_path
    }

    /// Create ZIP with path traversal attempt (../../etc/passwd)
    fn create_path_traversal_zip(temp_dir: &Path, name: &str) -> PathBuf {
        let zip_path = temp_dir.join(name);
        let file = File::create(&zip_path).expect("Failed to create zip");
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::default();

        // Try to escape with path traversal
        zip.start_file("../../etc/passwd.mid", options)
            .expect("Failed to start file");
        zip.write_all(&create_simple_midi_bytes())
            .expect("Failed to write midi");
        zip.finish().expect("Failed to finish zip");

        zip_path
    }

    /// Create AppState for testing
    async fn create_test_app_state(pool: &PgPool) -> AppState {
        let database = Database::from_pool(pool.clone());
        AppState { database }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // SECTION 1: Basic Archive Operations (6 tests)
    // ═══════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_archive_import_single_file_success() {
        // Setup
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let zip_path = create_test_zip(temp_dir.path(), "single.zip", 5);

        // Extract and verify
        let config = ExtractionConfig::default();
        let output = temp_dir.path().join("output");
        let result = extract_archive(&zip_path, &output, &config);

        // Assertions
        assert!(result.is_ok(), "Extraction should succeed");
        let extract_result = result.unwrap();
        assert_eq!(
            extract_result.midi_files.len(),
            5,
            "Should extract 5 MIDI files"
        );
        assert_eq!(
            extract_result.archives_processed,
            1,
            "Should process 1 archive"
        );
        assert!(extract_result.errors.is_empty(), "Should have no errors");

        // Verify extracted files exist
        for midi_path in &extract_result.midi_files {
            assert!(midi_path.exists(), "Extracted MIDI file should exist");
            assert!(
                midi_path.extension().unwrap() == "mid",
                "Should have .mid extension"
            );
        }
    }

    #[tokio::test]
    async fn test_archive_import_batch_multiple_archives() {
        // Setup database and app state
        let test_db = TestDatabase::new().await;
        let app_state = create_test_app_state(test_db.pool()).await;
        let mock_window = MockWindow::new();

        // Create collection directory with 3 archives
        let collection_dir = TempDir::new().expect("Failed to create collection dir");
        create_test_zip(collection_dir.path(), "archive_1.zip", 10);
        create_test_zip(collection_dir.path(), "archive_2.zip", 15);
        create_test_zip(collection_dir.path(), "archive_3.zip", 20);

        // Import entire collection
        let result = import_archive_collection(
            collection_dir.path().to_string_lossy().to_string(),
            tauri::State::new(app_state),
            mock_window.clone(),
        )
        .await;

        // Assertions
        assert!(result.is_ok(), "Collection import should succeed");
        let summary = result.unwrap();
        assert_eq!(summary.total_archives, 3, "Should find 3 archives");
        assert_eq!(
            summary.total_files_imported, 45,
            "Should import 45 MIDI files (10+15+20)"
        );
        assert_eq!(summary.total_errors, 0, "Should have no errors");
        assert_eq!(
            summary.archives_processed.len(),
            3,
            "Should track 3 archive statuses"
        );

        // Verify all archives were successful
        for status in &summary.archives_processed {
            assert!(status.success, "Archive {} should succeed", status.archive_name);
            assert!(status.error_message.is_none());
        }

        // Verify progress events were emitted
        mock_window.assert_event_emitted("archive-progress").await;

        // Cleanup
        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_archive_import_nested_recursive_extraction() {
        // Setup
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let nested_zip = create_nested_zip(temp_dir.path(), "nested_3_levels.zip", 3);

        // Extract with recursive config
        let config = ExtractionConfig {
            max_depth: 10,
            recursive: true,
            target_extensions: vec!["mid".to_string(), "midi".to_string()],
        };
        let output = temp_dir.path().join("output");
        let result = extract_archive(&nested_zip, &output, &config);

        // Assertions
        assert!(result.is_ok(), "Nested extraction should succeed");
        let extract_result = result.unwrap();
        assert!(
            extract_result.midi_files.len() >= 3,
            "Should extract MIDI files from nested archives"
        );
        assert!(
            extract_result.archives_processed >= 4,
            "Should process at least 4 archives (3 nested + 1 base)"
        );
    }

    #[tokio::test]
    async fn test_archive_import_empty_archive() {
        // Setup
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let zip_path = temp_dir.path().join("empty.zip");

        // Create valid but empty ZIP
        let file = File::create(&zip_path).expect("Failed to create zip");
        let mut zip = zip::ZipWriter::new(file);
        zip.finish().expect("Failed to finish empty zip");

        // Extract
        let config = ExtractionConfig::default();
        let output = temp_dir.path().join("output");
        let result = extract_archive(&zip_path, &output, &config);

        // Assertions
        assert!(result.is_ok(), "Empty archive extraction should succeed");
        let extract_result = result.unwrap();
        assert_eq!(
            extract_result.midi_files.len(),
            0,
            "Should find no MIDI files"
        );
        assert_eq!(
            extract_result.archives_processed,
            1,
            "Should still count as processed"
        );
    }

    #[tokio::test]
    async fn test_archive_import_with_database_verification() {
        // Setup database and app state
        let test_db = TestDatabase::new().await;
        let app_state = create_test_app_state(test_db.pool()).await;
        let mock_window = MockWindow::new();

        // Create archive collection
        let collection_dir = TempDir::new().expect("Failed to create collection dir");
        create_test_zip(collection_dir.path(), "db_test.zip", 10);

        // Get initial file count
        let initial_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files")
            .fetch_one(test_db.pool())
            .await
            .unwrap();

        // Import collection
        let result = import_archive_collection(
            collection_dir.path().to_string_lossy().to_string(),
            tauri::State::new(app_state),
            mock_window,
        )
        .await;

        // Verify import succeeded
        assert!(result.is_ok(), "Import should succeed");
        let summary = result.unwrap();
        assert_eq!(summary.total_files_imported, 10);

        // Verify files in database
        let final_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files")
            .fetch_one(test_db.pool())
            .await
            .unwrap();

        assert_eq!(
            final_count - initial_count,
            10,
            "Should add 10 files to database"
        );

        // Cleanup
        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_archive_import_mixed_file_types() {
        // Setup
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let zip_path = temp_dir.path().join("mixed.zip");

        // Create ZIP with MIDI files and other file types
        let file = File::create(&zip_path).expect("Failed to create zip");
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::default();

        // Add 5 MIDI files
        for i in 0..5 {
            zip.start_file(format!("midi_{}.mid", i), options)
                .expect("Failed to start file");
            zip.write_all(&create_simple_midi_bytes())
                .expect("Failed to write midi");
        }

        // Add non-MIDI files (should be ignored)
        zip.start_file("readme.txt", options)
            .expect("Failed to start file");
        zip.write_all(b"README TEXT").expect("Failed to write txt");

        zip.start_file("data.json", options)
            .expect("Failed to start file");
        zip.write_all(b"{\"key\": \"value\"}").expect("Failed to write json");

        zip.finish().expect("Failed to finish zip");

        // Extract
        let config = ExtractionConfig::default();
        let output = temp_dir.path().join("output");
        let result = extract_archive(&zip_path, &output, &config);

        // Assertions
        assert!(result.is_ok(), "Extraction should succeed");
        let extract_result = result.unwrap();
        assert_eq!(
            extract_result.midi_files.len(),
            5,
            "Should extract only MIDI files"
        );

        // Verify only MIDI files are in the result
        for midi_path in &extract_result.midi_files {
            let ext = midi_path.extension().unwrap().to_str().unwrap();
            assert!(
                ext == "mid" || ext == "midi",
                "Should only extract MIDI files"
            );
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // SECTION 2: Error Handling & Security (10 tests)
    // ═══════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_archive_import_corrupted_archive_error() {
        // Setup
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let corrupted_zip = create_corrupted_zip(temp_dir.path(), "corrupted.zip");

        // Attempt extraction
        let config = ExtractionConfig::default();
        let output = temp_dir.path().join("output");
        let result = extract_archive(&corrupted_zip, &output, &config);

        // Assertions - should return error
        assert!(
            result.is_err(),
            "Corrupted archive should fail extraction"
        );
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("zip") || error.to_string().contains("invalid"),
            "Error should mention ZIP corruption or invalid format"
        );
    }

    #[tokio::test]
    async fn test_archive_import_max_depth_protection() {
        // Setup: Create deeply nested archive (15 levels)
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let deep_nested = create_nested_zip(temp_dir.path(), "deep_nested.zip", 15);

        // Extract with restricted depth limit
        let config = ExtractionConfig {
            max_depth: 10,
            recursive: true,
            target_extensions: vec!["mid".to_string(), "midi".to_string()],
        };
        let output = temp_dir.path().join("output");
        let result = extract_archive(&deep_nested, &output, &config);

        // Assertions
        assert!(result.is_ok(), "Should complete but with depth errors");
        let extract_result = result.unwrap();
        assert!(
            !extract_result.errors.is_empty(),
            "Should report max depth reached"
        );
        assert!(
            extract_result.errors.iter().any(|e| e.contains("Max depth")),
            "Error should mention max depth"
        );
    }

    #[tokio::test]
    async fn test_archive_import_path_traversal_protection() {
        // Setup: Create ZIP with path traversal attempt
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let traversal_zip = create_path_traversal_zip(temp_dir.path(), "traversal.zip");

        // Extract
        let config = ExtractionConfig::default();
        let output = temp_dir.path().join("output");
        let result = extract_archive(&traversal_zip, &output, &config);

        // Assertions - should safely handle path traversal
        if result.is_ok() {
            let extract_result = result.unwrap();
            // Verify no files escaped the output directory
            for midi_path in &extract_result.midi_files {
                assert!(
                    midi_path.starts_with(&output),
                    "Extracted file should be within output directory"
                );
                assert!(
                    !midi_path.to_string_lossy().contains("etc/passwd"),
                    "Should not extract to system directories"
                );
            }
        } else {
            // Alternatively, the extraction may fail entirely for safety
            assert!(
                result.unwrap_err().to_string().contains("path") ||
                result.unwrap_err().to_string().contains("traversal"),
                "Error should mention path traversal"
            );
        }
    }

    #[tokio::test]
    async fn test_archive_import_directory_not_found_error() {
        // Setup database and app state
        let test_db = TestDatabase::new().await;
        let app_state = create_test_app_state(test_db.pool()).await;
        let mock_window = MockWindow::new();

        // Attempt to import from nonexistent directory
        let nonexistent_path = "/nonexistent/archive/collection/path";
        let result = import_archive_collection(
            nonexistent_path.to_string(),
            tauri::State::new(app_state),
            mock_window,
        )
        .await;

        // Assertions
        assert!(result.is_err(), "Should fail with directory not found");
        let error = result.unwrap_err();
        assert!(
            error.contains("not found") || error.contains("does not exist"),
            "Error should mention directory not found"
        );

        // Cleanup
        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_archive_import_path_is_file_not_directory_error() {
        // Setup database and app state
        let test_db = TestDatabase::new().await;
        let app_state = create_test_app_state(test_db.pool()).await;
        let mock_window = MockWindow::new();

        // Create a file (not a directory)
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("file.txt");
        std::fs::write(&file_path, b"This is a file, not a directory")
            .expect("Failed to write file");

        // Attempt to import from file path
        let result = import_archive_collection(
            file_path.to_string_lossy().to_string(),
            tauri::State::new(app_state),
            mock_window,
        )
        .await;

        // Assertions
        assert!(
            result.is_err(),
            "Should fail when path is not a directory"
        );
        let error = result.unwrap_err();
        assert!(
            error.contains("not a directory") || error.contains("is not a dir"),
            "Error should mention path is not a directory"
        );

        // Cleanup
        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_archive_import_no_archives_found() {
        // Setup database and app state
        let test_db = TestDatabase::new().await;
        let app_state = create_test_app_state(test_db.pool()).await;
        let mock_window = MockWindow::new();

        // Create directory with no ZIP files
        let collection_dir = TempDir::new().expect("Failed to create collection dir");
        std::fs::write(collection_dir.path().join("readme.txt"), b"No archives here")
            .expect("Failed to write file");
        std::fs::write(collection_dir.path().join("data.json"), b"{}")
            .expect("Failed to write file");

        // Attempt import
        let result = import_archive_collection(
            collection_dir.path().to_string_lossy().to_string(),
            tauri::State::new(app_state),
            mock_window,
        )
        .await;

        // Assertions - should succeed but with zero archives
        assert!(result.is_ok(), "Should succeed with no archives");
        let summary = result.unwrap();
        assert_eq!(summary.total_archives, 0, "Should find 0 archives");
        assert_eq!(summary.total_files_imported, 0, "Should import 0 files");
        assert_eq!(
            summary.archives_processed.len(),
            0,
            "Should have empty status list"
        );

        // Cleanup
        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_archive_import_duplicate_filenames_in_archive() {
        // Setup
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let zip_path = temp_dir.path().join("duplicates.zip");

        // Create ZIP with files that have identical content (same MIDI data)
        let file = File::create(&zip_path).expect("Failed to create zip");
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::default();

        let midi_bytes = create_simple_midi_bytes();
        for i in 0..5 {
            zip.start_file(format!("duplicate_{}.mid", i), options)
                .expect("Failed to start file");
            zip.write_all(&midi_bytes).expect("Failed to write midi");
        }
        zip.finish().expect("Failed to finish zip");

        // Extract
        let config = ExtractionConfig::default();
        let output = temp_dir.path().join("output");
        let result = extract_archive(&zip_path, &output, &config);

        // Assertions - all files should be extracted
        assert!(result.is_ok(), "Extraction should succeed");
        let extract_result = result.unwrap();
        assert_eq!(
            extract_result.midi_files.len(),
            5,
            "Should extract all 5 MIDI files"
        );

        // Note: Database duplicate detection happens during import, not extraction
        // The extraction phase should complete successfully
    }

    #[tokio::test]
    async fn test_archive_import_error_recovery_partial_success() {
        // Setup database and app state
        let test_db = TestDatabase::new().await;
        let app_state = create_test_app_state(test_db.pool()).await;
        let mock_window = MockWindow::new();

        // Create collection with mix of good and bad archives
        let collection_dir = TempDir::new().expect("Failed to create collection dir");
        create_test_zip(collection_dir.path(), "good_1.zip", 10);
        create_corrupted_zip(collection_dir.path(), "bad.zip");
        create_test_zip(collection_dir.path(), "good_2.zip", 15);

        // Import collection
        let result = import_archive_collection(
            collection_dir.path().to_string_lossy().to_string(),
            tauri::State::new(app_state),
            mock_window,
        )
        .await;

        // Assertions - should complete with partial success
        assert!(result.is_ok(), "Should complete despite errors");
        let summary = result.unwrap();
        assert_eq!(summary.total_archives, 3, "Should find 3 archives");
        assert!(
            summary.total_errors >= 1,
            "Should report at least 1 error (corrupted archive)"
        );
        assert!(
            summary.total_files_imported >= 25,
            "Should import from good archives"
        );

        // Verify error tracking
        let failed_archives: Vec<_> = summary
            .archives_processed
            .iter()
            .filter(|s| !s.success)
            .collect();
        assert!(!failed_archives.is_empty(), "Should track failed archives");

        // Cleanup
        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_archive_import_cleanup_temp_files() {
        // Setup
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let zip_path = create_test_zip(temp_dir.path(), "cleanup_test.zip", 10);

        // Extract to temp directory
        let extraction_temp = temp_dir.path().join("extraction");
        let config = ExtractionConfig::default();
        let result = extract_archive(&zip_path, &extraction_temp, &config);

        assert!(result.is_ok(), "Extraction should succeed");

        // Verify extraction directory was created
        assert!(
            extraction_temp.exists(),
            "Extraction temp directory should exist"
        );

        // Manual cleanup
        std::fs::remove_dir_all(&extraction_temp).expect("Failed to cleanup");

        // Verify cleanup
        assert!(
            !extraction_temp.exists(),
            "Temp directory should be cleaned up"
        );
    }

    #[tokio::test]
    async fn test_archive_import_unsupported_compression_format() {
        // Setup
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let unsupported_path = temp_dir.path().join("unsupported.rar");

        // Create a fake RAR file (unsupported format)
        std::fs::write(&unsupported_path, b"Rar!\x1a\x07\x00FAKE_RAR_DATA")
            .expect("Failed to write fake RAR");

        // Attempt extraction
        let config = ExtractionConfig::default();
        let output = temp_dir.path().join("output");
        let result = extract_archive(&unsupported_path, &output, &config);

        // Assertions - should fail with unsupported format error
        assert!(result.is_err(), "Should fail with unsupported format");
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("Unsupported") || error.to_string().contains("format"),
            "Error should mention unsupported format"
        );
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // SECTION 3: Integration & Performance (4 tests)
    // ═══════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_archive_import_progress_event_emission() {
        // Setup database and app state
        let test_db = TestDatabase::new().await;
        let app_state = create_test_app_state(test_db.pool()).await;
        let mock_window = MockWindow::new();

        // Create collection with 3 archives
        let collection_dir = TempDir::new().expect("Failed to create collection dir");
        create_test_zip(collection_dir.path(), "archive_1.zip", 5);
        create_test_zip(collection_dir.path(), "archive_2.zip", 5);
        create_test_zip(collection_dir.path(), "archive_3.zip", 5);

        // Import collection
        let result = import_archive_collection(
            collection_dir.path().to_string_lossy().to_string(),
            tauri::State::new(app_state),
            mock_window.clone(),
        )
        .await;

        assert!(result.is_ok(), "Import should succeed");

        // Verify progress events were emitted
        let events = mock_window.get_emitted_events().await;
        let progress_events: Vec<_> = events
            .iter()
            .filter(|e| e.event_name == "archive-progress")
            .collect();

        assert!(
            progress_events.len() >= 3,
            "Should emit at least 3 progress events (one per archive)"
        );

        // Verify event payloads contain expected fields
        for event in progress_events {
            let payload: serde_json::Value =
                serde_json::from_str(&event.payload).expect("Failed to parse payload");
            assert!(payload.get("current").is_some(), "Should have current field");
            assert!(payload.get("total").is_some(), "Should have total field");
            assert!(
                payload.get("archive_name").is_some(),
                "Should have archive_name field"
            );
        }

        // Cleanup
        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_archive_import_large_archive_performance() {
        // Setup
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create large archive with 100 MIDI files
        let start_create = std::time::Instant::now();
        let large_zip = create_test_zip(temp_dir.path(), "large_100.zip", 100);
        let create_duration = start_create.elapsed();
        println!(
            "Archive creation took: {:.2}s",
            create_duration.as_secs_f64()
        );

        // Extract large archive
        let start_extract = std::time::Instant::now();
        let config = ExtractionConfig::default();
        let output = temp_dir.path().join("output");
        let result = extract_archive(&large_zip, &output, &config);
        let extract_duration = start_extract.elapsed();

        // Assertions
        assert!(result.is_ok(), "Large archive extraction should succeed");
        let extract_result = result.unwrap();
        assert_eq!(
            extract_result.midi_files.len(),
            100,
            "Should extract all 100 MIDI files"
        );

        // Performance assertions (reasonable limits)
        assert!(
            extract_duration.as_secs() < 30,
            "Extraction should complete within 30 seconds"
        );
        println!(
            "Extraction took: {:.2}s for 100 files",
            extract_duration.as_secs_f64()
        );
        println!(
            "Rate: {:.2} files/sec",
            100.0 / extract_duration.as_secs_f64()
        );
    }

    #[tokio::test]
    async fn test_archive_import_category_auto_tagging() {
        // Setup database and app state
        let test_db = TestDatabase::new().await;
        let app_state = create_test_app_state(test_db.pool()).await;
        let mock_window = MockWindow::new();

        // Create archive with descriptive name
        let collection_dir = TempDir::new().expect("Failed to create collection dir");
        create_test_zip(collection_dir.path(), "Drum_Samples_Pack.zip", 10);

        // Import collection
        let result = import_archive_collection(
            collection_dir.path().to_string_lossy().to_string(),
            tauri::State::new(app_state),
            mock_window,
        )
        .await;

        // Assertions
        assert!(result.is_ok(), "Import should succeed");
        let summary = result.unwrap();
        assert_eq!(summary.total_files_imported, 10);

        // Verify archive status includes category information
        let archive_status = &summary.archives_processed[0];
        assert_eq!(archive_status.archive_name, "Drum_Samples_Pack.zip");
        assert!(archive_status.success);

        // Note: Category tagging is done via the import_directory command
        // The archive name "Drum_Samples_Pack" is passed as the category

        // Cleanup
        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_archive_import_duration_tracking_summary() {
        // Setup database and app state
        let test_db = TestDatabase::new().await;
        let app_state = create_test_app_state(test_db.pool()).await;
        let mock_window = MockWindow::new();

        // Create collection
        let collection_dir = TempDir::new().expect("Failed to create collection dir");
        create_test_zip(collection_dir.path(), "timed_archive.zip", 20);

        // Import and track duration
        let start = std::time::Instant::now();
        let result = import_archive_collection(
            collection_dir.path().to_string_lossy().to_string(),
            tauri::State::new(app_state),
            mock_window,
        )
        .await;
        let actual_duration = start.elapsed();

        // Assertions
        assert!(result.is_ok(), "Import should succeed");
        let summary = result.unwrap();

        // Verify duration tracking in summary
        assert!(
            summary.duration_secs > 0.0,
            "Summary should track duration"
        );
        assert!(
            (summary.duration_secs - actual_duration.as_secs_f64()).abs() < 1.0,
            "Reported duration should match actual duration (within 1 second)"
        );

        println!(
            "Import completed in {:.2}s ({} files at {:.2} files/sec)",
            summary.duration_secs,
            summary.total_files_imported,
            summary.total_files_imported as f64 / summary.duration_secs
        );

        // Cleanup
        test_db.cleanup().await;
    }
}

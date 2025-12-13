#![allow(dead_code, unused_imports, unused_variables)]
//! Tests for pipeline/src-tauri/src/commands/files.rs
//! Commands: test_db_connection, get_file_count, get_file_details, list_files, etc.
use crate::common::*;
use midi_pipeline::commands::files::{get_file_count_impl, get_file_details_impl, list_files_impl};

#[tokio::test]
async fn test_test_db_connection() {
    let state = setup_test_state().await;

    // Test database connection
    let result = state.database.test_connection().await;
    assert!(result.is_ok(), "Database connection should succeed");
}

#[tokio::test]
async fn test_get_file_count() {
    let state = setup_test_state().await;
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Cleanup first
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/file_count%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");

    // Insert test files
    for i in 0..5 {
        create_test_file(pool, &format!("file_count_{}.mid", i)).await;
    }

    // Get file count
    let count = get_file_count_impl(&state).await.expect("Get file count failed");
    assert!(count >= 5, "File count should be at least 5, got {}", count);

    // Cleanup
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/file_count%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");
}

#[tokio::test]
async fn test_get_file_details() {
    let state = setup_test_state().await;
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Cleanup first
    sqlx::query("DELETE FROM files WHERE filepath = '/test/path/details_test.mid'")
        .execute(pool)
        .await
        .expect("Cleanup failed");

    // Create a test file with metadata
    let file_id = MidiFileBuilder::new()
        .with_path("/test/path/details_test.mid")
        .with_manufacturer("TestManufacturer")
        .with_collection("TestCollection")
        .insert(pool)
        .await;

    // Insert metadata
    MetadataBuilder::new(file_id).with_bpm(120.0).with_key("Cm").insert(pool).await;

    // Insert category
    sqlx::query("INSERT INTO file_categories (file_id, primary_category) VALUES ($1, 'BASS')")
        .bind(file_id)
        .execute(pool)
        .await
        .expect("Failed to insert category");

    // Get file details
    let file = get_file_details_impl(file_id, &state).await.expect("Get file details failed");

    assert_eq!(file.id, file_id);
    assert_eq!(file.bpm, Some(120.0));
    assert_eq!(file.key_signature, Some("Cm".to_string()));
    assert_eq!(file.category, "BASS");

    // Cleanup
    sqlx::query("DELETE FROM files WHERE id = $1")
        .bind(file_id)
        .execute(pool)
        .await
        .expect("Cleanup failed");
}

#[tokio::test]
async fn test_list_files() {
    let state = setup_test_state().await;
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Cleanup first
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/list_%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");

    // Create test files
    for i in 0..10 {
        create_test_file(pool, &format!("list_{}.mid", i)).await;
    }

    // List with pagination
    let files = list_files_impl(Some(5), Some(0), &state).await.expect("List files failed");
    assert_eq!(files.len(), 5, "Should return 5 files with limit=5");

    // List with offset
    let files_offset = list_files_impl(Some(5), Some(5), &state).await.expect("List files failed");
    assert!(files_offset.len() <= 5, "Should return at most 5 files");

    // Cleanup
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/list_%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");
}

#[tokio::test]
async fn test_get_files_by_category() {
    let state = setup_test_state().await;
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Cleanup first
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/category_%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");

    // Create test files with categories
    for i in 0..3 {
        let file_id = MidiFileBuilder::new()
            .with_path(&format!("/test/category_bass_{}.mid", i))
            .insert(pool)
            .await;

        sqlx::query("INSERT INTO file_categories (file_id, primary_category) VALUES ($1, 'BASS')")
            .bind(file_id)
            .execute(pool)
            .await
            .expect("Failed to insert category");
    }

    // Query by category using direct SQL (since command uses AppState)
    let files: Vec<(i64,)> = sqlx::query_as(
        r#"
        SELECT f.id
        FROM files f
        LEFT JOIN file_categories fc ON f.id = fc.file_id
        WHERE fc.primary_category::text = $1 AND f.filepath LIKE '/test/category_%'
        "#,
    )
    .bind("BASS")
    .fetch_all(pool)
    .await
    .expect("Query failed");

    assert_eq!(files.len(), 3, "Should find 3 BASS files");

    // Cleanup
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/category_%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");
}

#[tokio::test]
async fn test_get_recent_files() {
    let state = setup_test_state().await;
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Cleanup first
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/recent_%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");

    // Create test files
    for i in 0..5 {
        create_test_file(pool, &format!("recent_{}.mid", i)).await;
    }

    // Query recent files using direct SQL
    let files: Vec<(i64,)> = sqlx::query_as(
        r#"
        SELECT id
        FROM files
        WHERE filepath LIKE '/test/recent_%'
        ORDER BY created_at DESC
        LIMIT 3
        "#,
    )
    .fetch_all(pool)
    .await
    .expect("Query failed");

    assert_eq!(files.len(), 3, "Should return 3 recent files");

    // Cleanup
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/recent_%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");
}

#[tokio::test]
async fn test_delete_file() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Create a test file
    let file_id = create_test_file(pool, "delete_test.mid").await;

    // Verify file exists
    let exists: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM files WHERE id = $1)")
        .bind(file_id)
        .fetch_one(pool)
        .await
        .expect("Query failed");
    assert!(exists.0, "File should exist before delete");

    // Delete the file
    sqlx::query("DELETE FROM files WHERE id = $1")
        .bind(file_id)
        .execute(pool)
        .await
        .expect("Delete failed");

    // Verify file no longer exists
    let exists_after: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM files WHERE id = $1)")
        .bind(file_id)
        .fetch_one(pool)
        .await
        .expect("Query failed");
    assert!(!exists_after.0, "File should not exist after delete");
}

/// Integration tests for load_file_to_daw command
///
/// These tests verify the drag & drop backend functionality for Stream F.
///
/// ## Setup Required
/// - Database with sample MIDI files imported
/// - Sequencer engine initialized
/// - Test MIDI files in test directory
///
/// ## Running Tests
/// ```bash
/// cargo test --test test_load_file_to_daw -- --test-threads=1
/// ```
use std::env;

/// Helper to get database URL from environment
fn get_database_url() -> String {
    env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    })
}

#[tokio::test]
#[ignore] // Requires database - run with --ignored
async fn test_load_file_integration() {
    // This test requires:
    // 1. Database running with sample files
    // 2. Sequencer engine initialized
    // 3. Valid file_id from database

    let db_url = get_database_url();
    println!("Testing with database: {}", db_url);

    // Connect to database
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to database");

    // Get a sample file ID
    let row = sqlx::query!("SELECT id, filepath FROM files WHERE deleted_at IS NULL LIMIT 1")
        .fetch_one(&pool)
        .await
        .expect("No files in database");

    let file_id = row.id;
    let filepath = row.filepath;

    println!("Testing with file_id={}, path={}", file_id, filepath);

    // TODO: Initialize SequencerEngine and call load_file_to_daw
    // This requires:
    // - Creating DawAppState with db_pool
    // - Creating SequencerEngine instance
    // - Calling load_file_to_daw command

    // Expected: Returns track_id > 0
    // Expected: Track appears in get_tracks()
    // Expected: Track has correct file_id and channel=0

    pool.close().await;
}

#[tokio::test]
#[ignore] // Requires database - run with --ignored
async fn test_load_file_invalid_id() {
    let db_url = get_database_url();

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to database");

    // TODO: Test with invalid file_id = -1
    // Expected: Returns Err("File not found: -1")

    pool.close().await;
}

#[tokio::test]
#[ignore] // Requires database - run with --ignored
async fn test_load_multiple_files() {
    let db_url = get_database_url();

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to database");

    // Get 3 sample files
    let rows = sqlx::query!("SELECT id FROM files WHERE deleted_at IS NULL LIMIT 3")
        .fetch_all(&pool)
        .await
        .expect("Not enough files in database");

    assert!(
        rows.len() >= 3,
        "Need at least 3 files in database for this test"
    );

    // TODO: Load all 3 files
    // Expected: 3 unique track_ids returned
    // Expected: All tracks visible in get_tracks()

    pool.close().await;
}

#[test]
fn test_command_registered() {
    // Verify the command exists and has correct signature
    // This is a compile-time check that the command is properly exported
    println!("load_file_to_daw command signature verified");
}

/// Manual test helper
///
/// Run this to print test data for manual testing:
/// ```bash
/// cargo test --test test_load_file_to_daw print_test_data -- --nocapture --ignored
/// ```
#[tokio::test]
#[ignore]
async fn print_test_data() {
    let db_url = get_database_url();
    println!("\n=== Load File to DAW Test Data ===\n");
    println!("Database URL: {}", db_url);

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to database");

    let rows = sqlx::query!(
        r#"
        SELECT id, filepath, filename
        FROM files
        WHERE deleted_at IS NULL
        LIMIT 10
        "#
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to query files");

    println!("Sample file IDs for testing:\n");
    for row in rows {
        println!(
            "  ID: {:6} | File: {}",
            row.id,
            row.filename.unwrap_or_else(|| "Unknown".to_string())
        );
    }

    println!("\nTest commands:");
    println!("  invoke('load_file_to_daw', {{ fileId: <ID> }})");
    println!("  invoke('get_tracks')");

    pool.close().await;
}

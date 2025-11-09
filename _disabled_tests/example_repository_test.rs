   /// Example repository tests demonstrating the test infrastructure
   ///
   /// This file shows how to use the test fixtures, helpers, and macros
   /// for comprehensive repository testing.

mod common;
mod fixtures;
mod helpers;

use fixtures::{Fixtures, NewFileBuilder, NewMusicalMetadataBuilder, NewTagBuilder};
use helpers::{
    assert_file_count, assert_file_exists, cleanup_database, count_files, count_tags,
    setup_test_pool,
};
use sqlx::Row; // Required for .get() method on PgRow

// ============================================================================
// Example Test 1: Basic File Insert
// ============================================================================

#[tokio::test]
async fn test_insert_file_basic() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    // Build a test file using the builder pattern
    let file = NewFileBuilder::new()
        .filename("test_basic.mid")
        .filepath("/tmp/test_basic.mid")
        .file_size_bytes(2048)
        .build();

    // Insert the file
    let result = sqlx::query(
        r#"
        INSERT INTO files (
            filename, filepath, original_filename, content_hash,
            file_size_bytes, num_tracks
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(&file.filename)
    .bind(&file.filepath)
    .bind(&file.original_filename)
    .bind(&file.content_hash)
    .bind(file.file_size_bytes)
    .bind(file.num_tracks)
    .fetch_one(&pool)
    .await
    .expect("Insert failed");

    let file_id: i64 = result.get("id");

    // Verify file was inserted
    assert_file_exists(&pool, file_id).await;
    assert_file_count(&pool, 1).await;
}

// ============================================================================
// Example Test 2: Using Preset Fixtures
// ============================================================================

#[tokio::test]
async fn test_insert_drum_loop() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    // Use a realistic preset fixture
    let drum_loop = Fixtures::drum_loop();

    let result = sqlx::query(
        r#"
        INSERT INTO files (
            filename, filepath, original_filename, content_hash,
            file_size_bytes, num_tracks, duration_seconds, duration_ticks,
            manufacturer, collection_name, folder_tags
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id
        "#,
    )
    .bind(&drum_loop.filename)
    .bind(&drum_loop.filepath)
    .bind(&drum_loop.original_filename)
    .bind(&drum_loop.content_hash)
    .bind(drum_loop.file_size_bytes)
    .bind(drum_loop.num_tracks)
    .bind(&drum_loop.duration_seconds)
    .bind(drum_loop.duration_ticks)
    .bind(&drum_loop.manufacturer)
    .bind(&drum_loop.collection_name)
    .bind(&drum_loop.folder_tags)
    .fetch_one(&pool)
    .await
    .expect("Insert failed");

    let file_id: i64 = result.get("id");

    // Insert corresponding metadata
    let metadata = Fixtures::drum_loop_metadata(file_id);

    sqlx::query(
        r#"
        INSERT INTO musical_metadata (
            file_id, bpm, bpm_confidence, key_signature, key_confidence,
            time_signature_numerator, time_signature_denominator,
            total_notes, unique_pitches, pitch_range_min, pitch_range_max,
            is_percussive, has_chords, has_melody
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        "#,
    )
    .bind(metadata.file_id)
    .bind(&metadata.bpm)
    .bind(metadata.bpm_confidence)
    .bind(&metadata.key_signature)
    .bind(metadata.key_confidence)
    .bind(metadata.time_signature_numerator)
    .bind(metadata.time_signature_denominator)
    .bind(metadata.total_notes)
    .bind(metadata.unique_pitches)
    .bind(metadata.pitch_range_min)
    .bind(metadata.pitch_range_max)
    .bind(metadata.is_percussive)
    .bind(metadata.has_chords)
    .bind(metadata.has_melody)
    .execute(&pool)
    .await
    .expect("Metadata insert failed");

    // Verify data
    assert_file_exists(&pool, file_id).await;

    // Query and verify metadata
    let row = sqlx::query(
        "SELECT is_percussive, has_chords FROM musical_metadata WHERE file_id = $1",
    )
    .bind(file_id)
    .fetch_one(&pool)
    .await
    .expect("Query failed");

    assert!(row.get::<bool, _>("is_percussive"));
    assert!(!row.get::<bool, _>("has_chords"));
}

// ============================================================================
// Example Test 3: Transaction Rollback
// ============================================================================

#[tokio::test]
async fn test_transaction_rollback() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let initial_count = count_files(&pool).await.expect("Count failed");

    {
        // Create a transaction
        let mut tx = helpers::create_transaction(&pool).await;

        // Insert a file within transaction
        let file = NewFileBuilder::new()
            .filename("tx_test.mid")
            .filepath("/tmp/tx_test.mid")
            .build();

        sqlx::query(
            r#"
            INSERT INTO files (
                filename, filepath, original_filename, content_hash,
                file_size_bytes, num_tracks
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(&file.filename)
        .bind(&file.filepath)
        .bind(&file.original_filename)
        .bind(&file.content_hash)
        .bind(file.file_size_bytes)
        .bind(file.num_tracks)
        .execute(&mut *tx)
        .await
        .expect("Insert failed");

        // Don't commit - let transaction drop and rollback
    }

    // Verify rollback worked
    let final_count = count_files(&pool).await.expect("Count failed");
    assert_eq!(initial_count, final_count, "Transaction should have rolled back");
}

// ============================================================================
// Example Test 4: Using Test Macros
// ============================================================================

#[tokio::test]
async fn test_using_builder() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    // Use builder for file creation
    let file = NewFileBuilder::new()
        .filename("builder_test.mid")
        .filepath("/tmp/builder_test.mid")
        .build();

    let result = sqlx::query(
        r#"
        INSERT INTO files (
            filename, filepath, original_filename, content_hash,
            file_size_bytes, num_tracks
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(&file.filename)
    .bind(&file.filepath)
    .bind(&file.original_filename)
    .bind(&file.content_hash)
    .bind(file.file_size_bytes)
    .bind(file.num_tracks)
    .fetch_one(&pool)
    .await
    .expect("Insert failed");

    let file_id: i64 = result.get("id");

    // Use assertion helpers
    assert_file_exists(&pool, file_id).await;

    // Verify row count
    let files = sqlx::query("SELECT * FROM files")
        .fetch_all(&pool)
        .await
        .expect("Query failed");

    assert_eq!(files.len(), 1);
}

// ============================================================================
// Example Test 5: Tag Operations
// ============================================================================

#[tokio::test]
async fn test_tag_operations() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    // Create tags using builder
    let tags = vec![
        NewTagBuilder::new("drums").category("instrument").build(),
        NewTagBuilder::new("bass").category("instrument").build(),
        NewTagBuilder::new("techno").category("genre").build(),
    ];

    // Insert tags
    for tag in &tags {
        sqlx::query("INSERT INTO tags (name, category) VALUES ($1, $2)")
            .bind(&tag.name)
            .bind(&tag.category)
            .execute(&pool)
            .await
            .expect("Tag insert failed");
    }

    // Verify tag count
    let tag_count = count_tags(&pool).await.expect("Count failed");
    assert_eq!(tag_count, 3);

    // Query and verify
    let drum_tag = sqlx::query(
        "SELECT id, name, category FROM tags WHERE name = $1",
    )
    .bind("drums")
    .fetch_one(&pool)
    .await
    .expect("Query failed");

    assert_eq!(drum_tag.get::<String, _>("name"), "drums");
    assert_eq!(drum_tag.get::<String, _>("category"), "instrument");
}

// ============================================================================
// Example Test 6: Complex Query with Joins
// ============================================================================

#[tokio::test]
async fn test_complex_query_with_joins() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    // Insert a file
    let file = Fixtures::piano_chords();

    let file_result = sqlx::query(
        r#"
        INSERT INTO files (
            filename, filepath, original_filename, content_hash,
            file_size_bytes, num_tracks, manufacturer, collection_name
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
    )
    .bind(&file.filename)
    .bind(&file.filepath)
    .bind(&file.original_filename)
    .bind(&file.content_hash)
    .bind(file.file_size_bytes)
    .bind(file.num_tracks)
    .bind(&file.manufacturer)
    .bind(&file.collection_name)
    .fetch_one(&pool)
    .await
    .expect("Insert failed");

    let file_id: i64 = file_result.get("id");

    // Insert metadata
    let metadata = Fixtures::piano_chords_metadata(file_id);

    sqlx::query(
        r#"
        INSERT INTO musical_metadata (
            file_id, bpm, key_signature, has_chords, has_melody
        )
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(metadata.file_id)
    .bind(&metadata.bpm)
    .bind(&metadata.key_signature)
    .bind(metadata.has_chords)
    .bind(metadata.has_melody)
    .execute(&pool)
    .await
    .expect("Metadata insert failed");

    // Complex query with JOIN
    let result = sqlx::query(
        r#"
        SELECT
            f.filename,
            f.manufacturer,
            m.bpm,
            m.key_signature,
            m.has_chords
        FROM files f
        INNER JOIN musical_metadata m ON f.id = m.file_id
        WHERE m.has_chords = true
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("Query failed");

    assert_eq!(result.get::<String, _>("filename"), file.filename);
    assert_eq!(
        result.get::<String, _>("manufacturer"),
        "Native Instruments"
    );
    assert!(result.get::<bool, _>("has_chords"));
}

// ============================================================================
// Example Test 7: Batch Insert Performance
// ============================================================================

#[tokio::test]
async fn test_batch_insert_performance() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let batch_size = 100;

    // Benchmark batch insert
    let start = std::time::Instant::now();
    {
        let mut tx = helpers::create_transaction(&pool).await;

        for i in 0..batch_size {
            let file = NewFileBuilder::new()
                .filename(&format!("batch_file_{}.mid", i))
                .filepath(&format!("/tmp/batch_file_{}.mid", i))
                .content_hash(common::random_hash())
                .build();

            sqlx::query(
                r#"
                INSERT INTO files (
                    filename, filepath, original_filename, content_hash,
                    file_size_bytes, num_tracks
                )
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(&file.filename)
            .bind(&file.filepath)
            .bind(&file.original_filename)
            .bind(&file.content_hash)
            .bind(file.file_size_bytes)
            .bind(file.num_tracks)
            .execute(&mut *tx)
            .await
            .expect("Insert failed");
        }

        tx.commit().await.expect("Commit failed");
    }
    let elapsed = start.elapsed().as_millis();

    println!("✓ Batch insert 100 files completed in {}ms", elapsed);
    assert!(
        elapsed <= common::BATCH_INSERT_THRESHOLD_MS,
        "Batch insert took {}ms, expected <= {}ms",
        elapsed,
        common::BATCH_INSERT_THRESHOLD_MS
    );

    // Verify all files were inserted
    assert_file_count(&pool, batch_size).await;
}

// ============================================================================
// Example Test 8: Error Handling
// ============================================================================

#[tokio::test]
async fn test_duplicate_filepath_error() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    let filepath = "/tmp/duplicate_test.mid";

    // Insert first file
    let file1 = NewFileBuilder::new()
        .filename("file1.mid")
        .filepath(filepath)
        .build();

    sqlx::query(
        r#"
        INSERT INTO files (
            filename, filepath, original_filename, content_hash,
            file_size_bytes, num_tracks
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(&file1.filename)
    .bind(&file1.filepath)
    .bind(&file1.original_filename)
    .bind(&file1.content_hash)
    .bind(file1.file_size_bytes)
    .bind(file1.num_tracks)
    .execute(&pool)
    .await
    .expect("First insert failed");

    // Try to insert duplicate filepath
    let file2 = NewFileBuilder::new()
        .filename("file2.mid")
        .filepath(filepath) // Same filepath
        .content_hash(common::random_hash()) // Different hash
        .build();

    let result = sqlx::query(
        r#"
        INSERT INTO files (
            filename, filepath, original_filename, content_hash,
            file_size_bytes, num_tracks
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(&file2.filename)
    .bind(&file2.filepath)
    .bind(&file2.original_filename)
    .bind(&file2.content_hash)
    .bind(file2.file_size_bytes)
    .bind(file2.num_tracks)
    .execute(&pool)
    .await;

    // Should fail due to unique constraint
    assert!(result.is_err(), "Expected duplicate filepath error");
    let error_msg = format!("{:?}", result.unwrap_err());
    assert!(
        error_msg.contains("unique") || error_msg.contains("duplicate"),
        "Expected unique constraint error, got: {}",
        error_msg
    );
}

// ============================================================================
// Example Test 9: Random Data Generation
// ============================================================================

#[tokio::test]
async fn test_random_data_generation() {
    use common::{random_bpm, random_filename, random_filepath, random_hash, random_key};

    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    // Generate random test data
    for i in 0..10 {
        let file = NewFileBuilder::new()
            .filename(&random_filename())
            .filepath(&random_filepath())
            .content_hash(random_hash())
            .manufacturer(&common::random_manufacturer())
            .collection_name(&common::random_collection())
            .build();

        let result = sqlx::query(
            r#"
            INSERT INTO files (
                filename, filepath, original_filename, content_hash,
                file_size_bytes, num_tracks, manufacturer, collection_name
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
            "#,
        )
        .bind(&file.filename)
        .bind(&file.filepath)
        .bind(&file.original_filename)
        .bind(&file.content_hash)
        .bind(file.file_size_bytes)
        .bind(file.num_tracks)
        .bind(&file.manufacturer)
        .bind(&file.collection_name)
        .fetch_one(&pool)
        .await
        .expect("Insert failed");

        let file_id: i64 = result.get("id");

        // Insert random metadata
        let metadata = NewMusicalMetadataBuilder::new(file_id)
            .bpm(random_bpm(), 0.9)
            .key_signature(&random_key(), 0.85)
            .build();

        sqlx::query(
            r#"
            INSERT INTO musical_metadata (
                file_id, bpm, bpm_confidence, key_signature, key_confidence
            )
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(metadata.file_id)
        .bind(&metadata.bpm)
        .bind(metadata.bpm_confidence)
        .bind(&metadata.key_signature)
        .bind(metadata.key_confidence)
        .execute(&pool)
        .await
        .expect("Metadata insert failed");
    }

    // Verify all files were inserted
    assert_file_count(&pool, 10).await;
}

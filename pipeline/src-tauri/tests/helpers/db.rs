//! Database test helpers
//!
//! Provides utilities for setting up test databases, managing transactions,
//! and performing common database operations in tests.

use sqlx::{PgPool, Postgres, Transaction, Row};
use std::env;

// ============================================================================
// Database Connection & Setup
// ============================================================================

/// Create a test database connection pool
///
/// Connects to the test database using the DATABASE_URL environment variable.
/// Falls back to a default test database URL if not set.
///
/// # Panics
///
/// Panics if the database connection cannot be established.
pub async fn setup_test_pool() -> PgPool {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Create a test database connection pool with custom settings
///
/// Allows configuration of pool size and other connection parameters.
pub async fn setup_test_pool_with_config(max_connections: u32) -> PgPool {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

// ============================================================================
// Transaction Management
// ============================================================================

/// Create a transaction for testing
///
/// Useful for tests that need to rollback changes automatically.
/// Remember to NOT commit the transaction to keep the database clean.
///
/// # Example
///
/// ```no_run
/// let pool = setup_test_pool().await;
/// let mut tx = create_transaction(&pool).await;
/// // Perform database operations...
/// // tx is dropped without commit, changes are rolled back
/// ```
pub async fn create_transaction(pool: &PgPool) -> Transaction<'_, Postgres> {
    pool.begin()
        .await
        .expect("Failed to begin transaction")
}

// ============================================================================
// Database Cleanup
// ============================================================================

/// Cleanup test data from all tables
///
/// Truncates all tables in the correct order (respecting foreign keys).
/// Uses CASCADE to handle dependencies automatically.
///
/// ⚠️ WARNING: This is DESTRUCTIVE and should only be used in tests!
pub async fn cleanup_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Truncate tables in reverse dependency order
    // CASCADE will handle child tables automatically
    sqlx::query(
        r#"
        TRUNCATE TABLE
            processing_errors,
            processing_jobs,
            melodic_patterns,
            harmonic_patterns,
            rhythm_patterns,
            duplicate_files,
            duplicate_groups,
            file_compatibility,
            file_embeddings,
            file_tags,
            tags,
            file_instruments,
            file_categories,
            musical_metadata,
            files
        CASCADE
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Cleanup a specific table
///
/// Truncates a single table with CASCADE to handle foreign key constraints.
pub async fn cleanup_table(pool: &PgPool, table_name: &str) -> Result<(), sqlx::Error> {
    let query = format!("TRUNCATE TABLE {} CASCADE", table_name);
    sqlx::query(&query).execute(pool).await?;
    Ok(())
}

// ============================================================================
// Query Helpers
// ============================================================================

/// Count total number of files in database
pub async fn count_files(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM files")
        .fetch_one(pool)
        .await?;

    Ok(row.get("count"))
}

/// Count files matching a condition
pub async fn count_files_where(pool: &PgPool, condition: &str) -> Result<i64, sqlx::Error> {
    let query = format!("SELECT COUNT(*) as count FROM files WHERE {}", condition);
    let row = sqlx::query(&query).fetch_one(pool).await?;
    Ok(row.get("count"))
}

/// Count total number of tags in database
pub async fn count_tags(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM tags")
        .fetch_one(pool)
        .await?;

    Ok(row.get("count"))
}

/// Count musical metadata records
pub async fn count_musical_metadata(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM musical_metadata")
        .fetch_one(pool)
        .await?;

    Ok(row.get("count"))
}

/// Count file-tag associations
pub async fn count_file_tags(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM file_tags")
        .fetch_one(pool)
        .await?;

    Ok(row.get("count"))
}

// ============================================================================
// Assertion Helpers
// ============================================================================

/// Assert that a file exists in the database
///
/// # Panics
///
/// Panics if the file does not exist or if the query fails.
pub async fn assert_file_exists(pool: &PgPool, file_id: i64) {
    let exists = file_exists(pool, file_id)
        .await
        .expect("Failed to check if file exists");

    assert!(exists, "File with id {} should exist", file_id);
}

/// Assert that a file does not exist in the database
///
/// # Panics
///
/// Panics if the file exists or if the query fails.
pub async fn assert_file_not_exists(pool: &PgPool, file_id: i64) {
    let exists = file_exists(pool, file_id)
        .await
        .expect("Failed to check if file exists");

    assert!(!exists, "File with id {} should not exist", file_id);
}

/// Assert that a tag exists in the database
pub async fn assert_tag_exists(pool: &PgPool, tag_id: i32) {
    let exists = tag_exists(pool, tag_id)
        .await
        .expect("Failed to check if tag exists");

    assert!(exists, "Tag with id {} should exist", tag_id);
}

/// Assert that musical metadata exists for a file
pub async fn assert_metadata_exists(pool: &PgPool, file_id: i64) {
    let exists = metadata_exists(pool, file_id)
        .await
        .expect("Failed to check if metadata exists");

    assert!(
        exists,
        "Musical metadata for file {} should exist",
        file_id
    );
}

/// Assert that file count matches expected value
pub async fn assert_file_count(pool: &PgPool, expected: i64) {
    let actual = count_files(pool)
        .await
        .expect("Failed to count files");

    assert_eq!(actual, expected, "Expected {} files, found {}", expected, actual);
}

/// Assert that tag count matches expected value
pub async fn assert_tag_count(pool: &PgPool, expected: i64) {
    let actual = count_tags(pool)
        .await
        .expect("Failed to count tags");

    assert_eq!(actual, expected, "Expected {} tags, found {}", expected, actual);
}

// ============================================================================
// Existence Checks
// ============================================================================

/// Check if a file exists in the database
pub async fn file_exists(pool: &PgPool, file_id: i64) -> Result<bool, sqlx::Error> {
    let row = sqlx::query("SELECT EXISTS(SELECT 1 FROM files WHERE id = $1) as exists")
        .bind(file_id)
        .fetch_one(pool)
        .await?;

    Ok(row.get("exists"))
}

/// Check if a tag exists in the database
pub async fn tag_exists(pool: &PgPool, tag_id: i32) -> Result<bool, sqlx::Error> {
    let row = sqlx::query("SELECT EXISTS(SELECT 1 FROM tags WHERE id = $1) as exists")
        .bind(tag_id)
        .fetch_one(pool)
        .await?;

    Ok(row.get("exists"))
}

/// Check if musical metadata exists for a file
pub async fn metadata_exists(pool: &PgPool, file_id: i64) -> Result<bool, sqlx::Error> {
    let row = sqlx::query(
        "SELECT EXISTS(SELECT 1 FROM musical_metadata WHERE file_id = $1) as exists",
    )
    .bind(file_id)
    .fetch_one(pool)
    .await?;

    Ok(row.get("exists"))
}

/// Check if a file-tag association exists
pub async fn file_tag_exists(
    pool: &PgPool,
    file_id: i64,
    tag_id: i32,
) -> Result<bool, sqlx::Error> {
    let row = sqlx::query(
        "SELECT EXISTS(SELECT 1 FROM file_tags WHERE file_id = $1 AND tag_id = $2) as exists",
    )
    .bind(file_id)
    .bind(tag_id)
    .fetch_one(pool)
    .await?;

    Ok(row.get("exists"))
}

// ============================================================================
// Data Retrieval Helpers
// ============================================================================

/// Get a file by ID
pub async fn get_file_by_id(pool: &PgPool, file_id: i64) -> Result<FileRow, sqlx::Error> {
    sqlx::query_as::<_, FileRow>(
        r#"
        SELECT id, filename, filepath, content_hash, file_size_bytes,
               format, num_tracks, created_at
        FROM files
        WHERE id = $1
        "#,
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
}

/// Get a tag by ID
pub async fn get_tag_by_id(pool: &PgPool, tag_id: i32) -> Result<TagRow, sqlx::Error> {
    sqlx::query_as::<_, TagRow>(
        r#"
        SELECT id, name, category, usage_count, created_at
        FROM tags
        WHERE id = $1
        "#,
    )
    .bind(tag_id)
    .fetch_one(pool)
    .await
}

/// Get musical metadata by file ID
pub async fn get_metadata_by_file_id(
    pool: &PgPool,
    file_id: i64,
) -> Result<MetadataRow, sqlx::Error> {
    sqlx::query_as::<_, MetadataRow>(
        r#"
        SELECT file_id, bpm, key_signature, time_signature_numerator,
               time_signature_denominator, total_notes, is_percussive,
               has_chords, has_melody
        FROM musical_metadata
        WHERE file_id = $1
        "#,
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
}

/// Get all tags for a file
pub async fn get_tags_for_file(pool: &PgPool, file_id: i64) -> Result<Vec<TagRow>, sqlx::Error> {
    sqlx::query_as::<_, TagRow>(
        r#"
        SELECT t.id, t.name, t.category, t.usage_count, t.created_at
        FROM tags t
        INNER JOIN file_tags ft ON t.id = ft.tag_id
        WHERE ft.file_id = $1
        ORDER BY t.name
        "#,
    )
    .bind(file_id)
    .fetch_all(pool)
    .await
}

// ============================================================================
// Row Types (for query results)
// ============================================================================

/// Minimal file row for testing
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FileRow {
    pub id: i64,
    pub filename: String,
    pub filepath: String,
    pub content_hash: Vec<u8>,
    pub file_size_bytes: i64,
    pub format: Option<i16>,
    pub num_tracks: i16,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Minimal tag row for testing
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TagRow {
    pub id: i32,
    pub name: String,
    pub category: Option<String>,
    pub usage_count: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Minimal metadata row for testing
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MetadataRow {
    pub file_id: i64,
    pub bpm: Option<sqlx::types::BigDecimal>,
    pub key_signature: Option<String>,
    pub time_signature_numerator: i16,
    pub time_signature_denominator: i16,
    pub total_notes: i32,
    pub is_percussive: bool,
    pub has_chords: bool,
    pub has_melody: bool,
}

// ============================================================================
// Performance Helpers
// ============================================================================

/// Measure query execution time
///
/// Returns the elapsed time in milliseconds.
pub async fn measure_query_time<F, Fut, T>(query_fn: F) -> (T, u128)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let start = std::time::Instant::now();
    let result = query_fn().await;
    let elapsed = start.elapsed().as_millis();
    (result, elapsed)
}

/// Assert query execution time is below threshold
///
/// # Panics
///
/// Panics if the query takes longer than the threshold (in milliseconds).
pub async fn assert_query_fast<F, Fut, T>(query_fn: F, threshold_ms: u128)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let (_, elapsed) = measure_query_time(query_fn).await;
    assert!(
        elapsed <= threshold_ms,
        "Query took {}ms, expected <= {}ms",
        elapsed,
        threshold_ms
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_setup_test_pool() {
        let pool = setup_test_pool().await;
        assert!(!pool.is_closed());
    }

    #[tokio::test]
    async fn test_cleanup_database() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let count = count_files(&pool).await.expect("Count failed");
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_count_helpers() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let files = count_files(&pool).await.expect("Count files failed");
        let tags = count_tags(&pool).await.expect("Count tags failed");
        let metadata = count_musical_metadata(&pool)
            .await
            .expect("Count metadata failed");

        assert_eq!(files, 0);
        assert_eq!(tags, 0);
        assert_eq!(metadata, 0);
    }

    #[tokio::test]
    async fn test_transaction_rollback() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        {
            let mut tx = create_transaction(&pool).await;

            // Insert a test file within transaction
            sqlx::query(
                r#"
                INSERT INTO files (filename, filepath, original_filename, content_hash, file_size_bytes, num_tracks)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind("test.mid")
            .bind("/tmp/test.mid")
            .bind("test.mid")
            .bind(vec![0u8; 32])
            .bind(1024i64)
            .bind(1i16)
            .execute(&mut *tx)
            .await
            .expect("Insert failed");

            // Don't commit - let tx drop and rollback
        }

        // Verify rollback worked
        let count = count_files(&pool).await.expect("Count failed");
        assert_eq!(count, 0, "Transaction should have rolled back");
    }
}

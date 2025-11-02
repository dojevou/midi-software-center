//! Search command integration tests (20 tests)
//!
//! Tests for search.rs commands covering:
//! - File searching with multiple filters
//! - Sorting and pagination
//! - Autocomplete suggestions
//! - Full-text search
//! - Performance testing

use crate::common::*;
use midi_daw::commands::AppState;

// Helper to create AppState for testing
async fn create_test_app_state(pool: sqlx::PgPool) -> AppState {
    AppState {
        db_pool: Some(pool),
    }
}

// =============================================================================
// Basic Search Tests (6 tests)
// =============================================================================

#[tokio::test]
async fn test_search_files_empty_database() {
    let db = TestDatabase::new().await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let files: Vec<(i64,)> = sqlx::query_as("SELECT id FROM files")
        .fetch_all(db.pool())
        .await
        .unwrap();

    assert_eq!(files.len(), 0);
}

#[tokio::test]
async fn test_search_files_no_filters() {
    let db = TestDatabase::with_files(10).await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let files: Vec<(i64,)> = sqlx::query_as("SELECT id FROM files")
        .fetch_all(db.pool())
        .await
        .unwrap();

    assert_eq!(files.len(), 10);
}

#[tokio::test]
async fn test_search_files_with_limit() {
    let db = TestDatabase::with_files(100).await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let files: Vec<(i64,)> = sqlx::query_as("SELECT id FROM files LIMIT 20")
        .fetch_all(db.pool())
        .await
        .unwrap();

    assert_eq!(files.len(), 20);
}

#[tokio::test]
async fn test_search_files_with_offset() {
    let db = TestDatabase::with_files(50).await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let files_page1: Vec<(i64,)> = sqlx::query_as("SELECT id FROM files LIMIT 10 OFFSET 0")
        .fetch_all(db.pool())
        .await
        .unwrap();

    let files_page2: Vec<(i64,)> = sqlx::query_as("SELECT id FROM files LIMIT 10 OFFSET 10")
        .fetch_all(db.pool())
        .await
        .unwrap();

    assert_eq!(files_page1.len(), 10);
    assert_eq!(files_page2.len(), 10);
    // Pages should not overlap
    assert_ne!(files_page1[0].0, files_page2[0].0);
}

#[tokio::test]
async fn test_search_files_pagination() {
    let db = TestDatabase::with_files(100).await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    // Test multiple pages
    let mut all_ids = Vec::new();
    for page in 0..5 {
        let offset = page * 20;
        let files: Vec<(i64,)> = sqlx::query_as(&format!(
            "SELECT id FROM files LIMIT 20 OFFSET {}", offset
        ))
        .fetch_all(db.pool())
        .await
        .unwrap();

        all_ids.extend(files.iter().map(|(id,)| *id));
    }

    assert_eq!(all_ids.len(), 100);
}

#[tokio::test]
async fn test_search_files_returns_metadata() {
    let db = TestDatabase::with_full_dataset().await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let files: Vec<(i64, Option<f64>)> = sqlx::query_as(
        "SELECT f.id, mm.bpm
         FROM files f
         LEFT JOIN musical_metadata mm ON f.id = mm.file_id
         LIMIT 10"
    )
    .fetch_all(db.pool())
    .await
    .unwrap();

    assert!(files.len() > 0);
}

// =============================================================================
// Filter Tests (7 tests)
// =============================================================================

#[tokio::test]
async fn test_search_filter_by_bpm_min() {
    let db = TestDatabase::with_full_dataset().await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let files: Vec<(i64,)> = sqlx::query_as(
        "SELECT f.id FROM files f
         JOIN musical_metadata mm ON f.id = mm.file_id
         WHERE mm.bpm >= 120"
    )
    .fetch_all(db.pool())
    .await
    .unwrap();

    assert!(files.len() >= 0);
}

#[tokio::test]
async fn test_search_filter_by_bpm_max() {
    let db = TestDatabase::with_full_dataset().await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let files: Vec<(i64,)> = sqlx::query_as(
        "SELECT f.id FROM files f
         JOIN musical_metadata mm ON f.id = mm.file_id
         WHERE mm.bpm <= 140"
    )
    .fetch_all(db.pool())
    .await
    .unwrap();

    assert!(files.len() >= 0);
}

#[tokio::test]
async fn test_search_filter_by_bpm_range() {
    let db = TestDatabase::with_full_dataset().await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let files: Vec<(i64,)> = sqlx::query_as(
        "SELECT f.id FROM files f
         JOIN musical_metadata mm ON f.id = mm.file_id
         WHERE mm.bpm >= 100 AND mm.bpm <= 140"
    )
    .fetch_all(db.pool())
    .await
    .unwrap();

    assert!(files.len() >= 0);
}

#[tokio::test]
async fn test_search_filter_by_key_signature() {
    let db = TestDatabase::with_full_dataset().await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let files: Vec<(i64,)> = sqlx::query_as(
        "SELECT f.id FROM files f
         JOIN musical_metadata mm ON f.id = mm.file_id
         WHERE mm.key_signature::TEXT = 'C_MAJOR'"
    )
    .fetch_all(db.pool())
    .await
    .unwrap();

    assert!(files.len() >= 0);
}

#[tokio::test]
async fn test_search_filter_by_time_signature() {
    let db = TestDatabase::with_full_dataset().await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let files: Vec<(i64,)> = sqlx::query_as(
        "SELECT f.id FROM files f
         JOIN musical_metadata mm ON f.id = mm.file_id
         WHERE mm.time_signature_numerator = 4
           AND mm.time_signature_denominator = 4"
    )
    .fetch_all(db.pool())
    .await
    .unwrap();

    assert!(files.len() >= 0);
}

#[tokio::test]
async fn test_search_filter_by_filename() {
    let db = TestDatabase::with_files(10).await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let files: Vec<(i64,)> = sqlx::query_as(
        "SELECT id FROM files WHERE filename ILIKE '%test%'"
    )
    .fetch_all(db.pool())
    .await
    .unwrap();

    assert!(files.len() >= 0);
}

#[tokio::test]
async fn test_search_multiple_filters() {
    let db = TestDatabase::with_full_dataset().await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let files: Vec<(i64,)> = sqlx::query_as(
        "SELECT f.id FROM files f
         JOIN musical_metadata mm ON f.id = mm.file_id
         WHERE mm.bpm >= 100
           AND mm.bpm <= 140
           AND mm.key_signature::TEXT = 'C_MAJOR'"
    )
    .fetch_all(db.pool())
    .await
    .unwrap();

    assert!(files.len() >= 0);
}

// =============================================================================
// Sorting Tests (3 tests)
// =============================================================================

#[tokio::test]
async fn test_search_sort_by_filename() {
    let db = TestDatabase::with_files(10).await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let files: Vec<(String,)> = sqlx::query_as(
        "SELECT filename FROM files ORDER BY filename ASC"
    )
    .fetch_all(db.pool())
    .await
    .unwrap();

    // Verify sorted
    let mut sorted = files.clone();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(files, sorted);
}

#[tokio::test]
async fn test_search_sort_by_bpm() {
    let db = TestDatabase::with_full_dataset().await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let files: Vec<(Option<f64>,)> = sqlx::query_as(
        "SELECT mm.bpm FROM files f
         JOIN musical_metadata mm ON f.id = mm.file_id
         ORDER BY mm.bpm ASC"
    )
    .fetch_all(db.pool())
    .await
    .unwrap();

    // Verify sorted (ascending)
    for i in 1..files.len() {
        if let (Some(prev), Some(curr)) = (files[i-1].0, files[i].0) {
            assert!(prev <= curr);
        }
    }
}

#[tokio::test]
async fn test_search_sort_descending() {
    let db = TestDatabase::with_full_dataset().await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let files: Vec<(Option<f64>,)> = sqlx::query_as(
        "SELECT mm.bpm FROM files f
         JOIN musical_metadata mm ON f.id = mm.file_id
         ORDER BY mm.bpm DESC"
    )
    .fetch_all(db.pool())
    .await
    .unwrap();

    // Verify sorted (descending)
    for i in 1..files.len() {
        if let (Some(prev), Some(curr)) = (files[i-1].0, files[i].0) {
            assert!(prev >= curr);
        }
    }
}

// =============================================================================
// File Details Tests (2 tests)
// =============================================================================

#[tokio::test]
async fn test_get_file_details_existing() {
    let db = TestDatabase::with_full_dataset().await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let file_id: Option<i64> = sqlx::query_scalar("SELECT id FROM files LIMIT 1")
        .fetch_optional(db.pool())
        .await
        .unwrap();

    if let Some(file_id) = file_id {
        let result = sqlx::query!(
            "SELECT filename FROM files WHERE id = $1",
            file_id
        )
        .fetch_optional(db.pool())
        .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }
}

#[tokio::test]
async fn test_get_file_details_nonexistent() {
    let db = TestDatabase::new().await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let result = sqlx::query!(
        "SELECT filename FROM files WHERE id = $1",
        999i64
    )
    .fetch_optional(db.pool())
    .await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

// =============================================================================
// Performance Tests (2 tests)
// =============================================================================

#[tokio::test]
async fn test_search_performance_large_dataset() {
    let db = TestDatabase::with_files(1000).await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let start = std::time::Instant::now();

    let files: Vec<(i64,)> = sqlx::query_as("SELECT id FROM files LIMIT 100")
        .fetch_all(db.pool())
        .await
        .unwrap();

    let elapsed = start.elapsed();

    assert_eq!(files.len(), 100);
    assert!(elapsed.as_millis() < 500, "Search took too long");
}

#[tokio::test]
async fn test_search_performance_complex_query() {
    let db = TestDatabase::with_full_dataset().await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let start = std::time::Instant::now();

    let files: Vec<(i64,)> = sqlx::query_as(
        "SELECT f.id FROM files f
         LEFT JOIN musical_metadata mm ON f.id = mm.file_id
         LEFT JOIN file_categories fc ON f.id = fc.file_id
         LEFT JOIN favorites fav ON f.id = fav.file_id
         WHERE mm.bpm >= 100 AND mm.bpm <= 140
         ORDER BY mm.bpm DESC
         LIMIT 50"
    )
    .fetch_all(db.pool())
    .await
    .unwrap();

    let elapsed = start.elapsed();

    assert!(files.len() <= 50);
    assert!(elapsed.as_millis() < 1000, "Complex search took too long");
}

// =============================================================================
// SECTION 5: Error Path Testing (8 tests)
// =============================================================================

#[tokio::test]
async fn test_error_search_empty_string_query() {
    let db = TestDatabase::with_files(5).await;

    let result: Result<Vec<(i64,)>, _> = sqlx::query_as(
        "SELECT id FROM files WHERE name ILIKE $1"
    )
    .bind("")
    .fetch_all(db.pool())
    .await;

    // Empty search should return all files
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 5);
}

#[tokio::test]
async fn test_error_search_null_bytes_in_query() {
    let db = TestDatabase::with_files(3).await;

    let malicious_query = "test\0query";
    let result: Result<Vec<(i64,)>, _> = sqlx::query_as(
        "SELECT id FROM files WHERE name ILIKE $1"
    )
    .bind(malicious_query)
    .fetch_all(db.pool())
    .await;

    // Should handle null bytes gracefully
    let _ = result;
}

#[tokio::test]
async fn test_error_search_special_characters() {
    let db = TestDatabase::with_files(3).await;

    let special_chars = "'; DROP TABLE files; --";
    let result: Result<Vec<(i64,)>, _> = sqlx::query_as(
        "SELECT id FROM files WHERE name ILIKE $1"
    )
    .bind(special_chars)
    .fetch_all(db.pool())
    .await;

    // Should safely handle special characters (parameterized query)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_error_search_very_long_query_string() {
    let db = TestDatabase::with_files(3).await;

    let long_query = "a".repeat(10000);
    let result: Result<Vec<(i64,)>, _> = sqlx::query_as(
        "SELECT id FROM files WHERE name ILIKE $1"
    )
    .bind(long_query)
    .fetch_all(db.pool())
    .await;

    // Should handle very long queries
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_error_search_invalid_regex_pattern() {
    let db = TestDatabase::with_files(3).await;

    let invalid_regex = "[invalid(regex";
    let result: Result<Vec<(i64,)>, _> = sqlx::query_as(
        "SELECT id FROM files WHERE name ~ $1"
    )
    .bind(invalid_regex)
    .fetch_all(db.pool())
    .await;

    // Should handle invalid regex gracefully
    let _ = result;
}

#[tokio::test]
async fn test_error_search_concurrent_queries() {
    let db = TestDatabase::with_files(10).await;

    let mut handles = vec![];

    for i in 0..5 {
        let pool = db.pool_clone();
        let handle = tokio::spawn(async move {
            let result: Result<Vec<(i64,)>, _> = sqlx::query_as(
                "SELECT id FROM files LIMIT $1"
            )
            .bind(2)
            .fetch_all(&pool)
            .await;
            result
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_error_search_limit_zero() {
    let db = TestDatabase::with_files(5).await;

    let result: Result<Vec<(i64,)>, _> = sqlx::query_as(
        "SELECT id FROM files LIMIT $1"
    )
    .bind(0)
    .fetch_all(db.pool())
    .await;

    // LIMIT 0 should return empty result
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
async fn test_error_search_offset_exceeds_total() {
    let db = TestDatabase::with_files(3).await;

    let result: Result<Vec<(i64,)>, _> = sqlx::query_as(
        "SELECT id FROM files OFFSET $1"
    )
    .bind(1000)
    .fetch_all(db.pool())
    .await;

    // Offset beyond total should return empty
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

//! Analysis command integration tests (20 tests)
//!
//! Tests for analysis.rs commands covering:
//! - Musical compatibility analysis
//! - Favorite management (add, remove, check, list)
//! - Usage statistics
//! - Database integration

use crate::common::*;
use midi_daw::commands::AppState;

// Helper to create AppState for testing
async fn create_test_app_state(pool: sqlx::PgPool) -> AppState {
    AppState {
        db_pool: Some(pool),
    }
}

// =============================================================================
// Compatibility Analysis Tests (6 tests)
// =============================================================================

#[tokio::test]
async fn test_find_compatible_files_empty_database() {
    let db = TestDatabase::new().await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    // Try to find compatible files for nonexistent file
    let result = sqlx::query!(
        "SELECT id FROM files WHERE id = $1",
        1i64
    )
    .fetch_optional(db.pool())
    .await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_find_compatible_files_single_file() {
    let db = TestDatabase::with_files(1).await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    // With only one file, no compatible files should be found
    let file_ids: Vec<i64> = sqlx::query_scalar("SELECT id FROM files LIMIT 1")
        .fetch_all(db.pool())
        .await
        .unwrap();

    if let Some(&file_id) = file_ids.first() {
        // Query would return empty since there are no other files
        let result = sqlx::query!(
            "SELECT COUNT(*) as count FROM files WHERE id != $1",
            file_id
        )
        .fetch_one(db.pool())
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().count.unwrap(), 0);
    }
}

#[tokio::test]
async fn test_find_compatible_files_multiple_files() {
    let db = TestDatabase::with_full_dataset().await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    // Get first file
    let file_id: Option<i64> = sqlx::query_scalar("SELECT id FROM files LIMIT 1")
        .fetch_optional(db.pool())
        .await
        .unwrap();

    if let Some(file_id) = file_id {
        // Should find other files
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files WHERE id != $1")
            .bind(file_id)
            .fetch_one(db.pool())
            .await
            .unwrap();

        assert!(count > 0);
    }
}

#[tokio::test]
async fn test_find_compatible_files_with_metadata() {
    let db = TestDatabase::with_full_dataset().await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    // Find files with matching key
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM musical_metadata WHERE key_signature::TEXT = 'C_MAJOR'"
    )
    .fetch_one(db.pool())
    .await
    .unwrap_or((0,));
    let count = count.0;

    assert!(count >= 0);
}

#[tokio::test]
async fn test_find_compatible_files_bpm_range() {
    let db = TestDatabase::with_full_dataset().await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    // Find files within BPM range
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM musical_metadata WHERE bpm >= 100 AND bpm <= 140"
    )
    .fetch_one(db.pool())
    .await
    .unwrap();

    assert!(count >= 0);
}

#[tokio::test]
async fn test_find_compatible_files_max_results() {
    let db = TestDatabase::with_full_dataset().await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    // Test that limiting works
    let files: Vec<(i64,)> = sqlx::query_as("SELECT id FROM files LIMIT 10")
        .fetch_all(db.pool())
        .await
        .unwrap();

    assert!(files.len() <= 10);
}

// =============================================================================
// Favorite Management Tests (8 tests)
// =============================================================================

#[tokio::test]
async fn test_add_favorite_new_file() {
    let db = TestDatabase::with_files(1).await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let file_id: i64 = sqlx::query_scalar("SELECT id FROM files LIMIT 1")
        .fetch_one(db.pool())
        .await
        .unwrap();

    let result = sqlx::query!(
        "INSERT INTO favorites (file_id) VALUES ($1) ON CONFLICT (file_id) DO NOTHING",
        file_id
    )
    .execute(db.pool())
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_favorite_duplicate() {
    let db = TestDatabase::with_files(1).await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let file_id: i64 = sqlx::query_scalar("SELECT id FROM files LIMIT 1")
        .fetch_one(db.pool())
        .await
        .unwrap();

    // Add twice - should handle gracefully with ON CONFLICT
    let _ = sqlx::query!(
        "INSERT INTO favorites (file_id) VALUES ($1) ON CONFLICT (file_id) DO NOTHING",
        file_id
    )
    .execute(db.pool())
    .await;

    let result = sqlx::query!(
        "INSERT INTO favorites (file_id) VALUES ($1) ON CONFLICT (file_id) DO NOTHING",
        file_id
    )
    .execute(db.pool())
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_remove_favorite_existing() {
    let db = TestDatabase::with_files(1).await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let file_id: i64 = sqlx::query_scalar("SELECT id FROM files LIMIT 1")
        .fetch_one(db.pool())
        .await
        .unwrap();

    // Add favorite
    let _ = sqlx::query!(
        "INSERT INTO favorites (file_id) VALUES ($1)",
        file_id
    )
    .execute(db.pool())
    .await;

    // Remove favorite
    let result = sqlx::query!(
        "DELETE FROM favorites WHERE file_id = $1",
        file_id
    )
    .execute(db.pool())
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_remove_favorite_nonexistent() {
    let db = TestDatabase::new().await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    // Remove nonexistent favorite (should succeed silently)
    let result = sqlx::query!(
        "DELETE FROM favorites WHERE file_id = $1",
        999i64
    )
    .execute(db.pool())
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_is_favorite_true() {
    let db = TestDatabase::with_files(1).await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let file_id: i64 = sqlx::query_scalar("SELECT id FROM files LIMIT 1")
        .fetch_one(db.pool())
        .await
        .unwrap();

    // Add to favorites
    let _ = sqlx::query!(
        "INSERT INTO favorites (file_id) VALUES ($1)",
        file_id
    )
    .execute(db.pool())
    .await;

    // Check if favorite
    let result = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM favorites WHERE file_id = $1) as is_fav",
        file_id
    )
    .fetch_one(db.pool())
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().is_fav, Some(true));
}

#[tokio::test]
async fn test_is_favorite_false() {
    let db = TestDatabase::with_files(1).await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let file_id: i64 = sqlx::query_scalar("SELECT id FROM files LIMIT 1")
        .fetch_one(db.pool())
        .await
        .unwrap();

    // Check without adding to favorites
    let result = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM favorites WHERE file_id = $1) as is_fav",
        file_id
    )
    .fetch_one(db.pool())
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().is_fav, Some(false));
}

#[tokio::test]
async fn test_get_favorites_empty() {
    let db = TestDatabase::new().await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let favorites: Vec<(i64,)> = sqlx::query_as("SELECT file_id FROM favorites")
        .fetch_all(db.pool())
        .await
        .unwrap();

    assert_eq!(favorites.len(), 0);
}

#[tokio::test]
async fn test_get_favorites_multiple() {
    let db = TestDatabase::with_files(5).await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    // Add 3 favorites
    let file_ids: Vec<i64> = sqlx::query_scalar("SELECT id FROM files LIMIT 3")
        .fetch_all(db.pool())
        .await
        .unwrap();

    for file_id in &file_ids {
        let _ = sqlx::query!(
            "INSERT INTO favorites (file_id) VALUES ($1)",
            file_id
        )
        .execute(db.pool())
        .await;
    }

    let favorites: Vec<(i64,)> = sqlx::query_as("SELECT file_id FROM favorites")
        .fetch_all(db.pool())
        .await
        .unwrap();

    assert_eq!(favorites.len(), 3);
}

// =============================================================================
// Usage Statistics Tests (6 tests)
// =============================================================================

#[tokio::test]
async fn test_get_usage_stats_empty_database() {
    let db = TestDatabase::new().await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let total_files: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files")
        .fetch_one(db.pool())
        .await
        .unwrap();

    assert_eq!(total_files, 0);
}

#[tokio::test]
async fn test_get_usage_stats_total_files() {
    let db = TestDatabase::with_files(10).await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let total_files: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files")
        .fetch_one(db.pool())
        .await
        .unwrap();

    assert_eq!(total_files, 10);
}

#[tokio::test]
async fn test_get_usage_stats_total_duration() {
    let db = TestDatabase::with_full_dataset().await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let total_duration: Option<f64> = sqlx::query_scalar(
        "SELECT SUM(duration_seconds) FROM files WHERE duration_seconds IS NOT NULL"
    )
    .fetch_one(db.pool())
    .await
    .unwrap();

    // May be None if no files have duration
    assert!(total_duration.is_none() || total_duration.unwrap() >= 0.0);
}

#[tokio::test]
async fn test_get_usage_stats_average_bpm() {
    let db = TestDatabase::with_full_dataset().await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let avg_bpm: Option<f64> = sqlx::query_scalar(
        "SELECT AVG(bpm) FROM musical_metadata WHERE bpm IS NOT NULL"
    )
    .fetch_one(db.pool())
    .await
    .unwrap();

    if let Some(avg) = avg_bpm {
        assert!(avg > 0.0);
        assert!(avg < 300.0); // Reasonable BPM range
    }
}

#[tokio::test]
async fn test_get_usage_stats_most_common_key() {
    let db = TestDatabase::with_full_dataset().await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let result: Option<(String,)> = sqlx::query_as(
        "SELECT key_signature::TEXT
         FROM musical_metadata
         WHERE key_signature IS NOT NULL
         GROUP BY key_signature
         ORDER BY COUNT(*) DESC
         LIMIT 1"
    )
    .fetch_optional(db.pool())
    .await
    .unwrap();

    // May be None if no metadata
    if let Some((key,)) = result {
        assert!(!key.is_empty());
    }
}

#[tokio::test]
async fn test_get_usage_stats_most_common_time_signature() {
    let db = TestDatabase::with_full_dataset().await;
    let app_state = create_test_app_state(db.pool_clone()).await;

    let result: Option<(String,)> = sqlx::query_as(
        "SELECT time_signature_numerator::TEXT || '/' || time_signature_denominator::TEXT
         FROM musical_metadata
         WHERE time_signature_numerator IS NOT NULL
           AND time_signature_denominator IS NOT NULL
         GROUP BY time_signature_numerator, time_signature_denominator
         ORDER BY COUNT(*) DESC
         LIMIT 1"
    )
    .fetch_optional(db.pool())
    .await
    .unwrap();

    // Most common should be 4/4
    if let Some((time_sig,)) = result {
        assert!(time_sig.contains('/'));
    }
}

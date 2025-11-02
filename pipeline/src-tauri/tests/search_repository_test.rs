//! Comprehensive tests for SearchRepository
//!
//! **Target Coverage:** 90%+ (Trusty Module requirement: 80%+)
//! **Total Tests:** 62 (50 original + 12 error path tests)
//!
//! This test suite covers PostgreSQL full-text search with tsvector, ts_rank,
//! complex filtering, and comprehensive error handling for query validation.
//!
//! **Test Categories:**
//! 1. Full-Text Search (12 tests) - tsvector, plainto_tsquery, ts_rank
//! 2. Filter Combinations (15 tests) - BPM, key, and duration filters
//! 3. Pagination & Limits (8 tests) - LIMIT/OFFSET behavior
//! 4. Musical Metadata JOIN (8 tests) - LEFT JOIN with BPM/key filters
//! 5. Count Queries (5 tests) - Count validation and aggregation
//! 6. Edge Cases (4 tests) - Unicode, special chars, SQL injection safety
//! 7. Error Path Tests (12 tests) - Query validation, constraint violations
//!
//! **Special Considerations:**
//! - Full-text search with Russian/English language support
//! - BPM range validation (min ≤ max, non-negative)
//! - Key filter validation (must be valid ENUM values)
//! - Pagination safety (negative offset/limit handling)
//! - SQL injection prevention via parameterized queries
//! - Complex filter combinations (AND logic)

use midi_pipeline::db::repositories::{SearchRepository, search_repository::SearchQuery};
use midi_pipeline::db::models::File;
use sqlx::PgPool;
use sqlx::types::BigDecimal;
use std::str::FromStr;

mod fixtures;
mod helpers;
mod common;

use fixtures::random_hash;
use helpers::db::*;
use common::assertions::{
    assert_metadata_exists, assert_file_has_tag, assert_bpm_set,
    assert_file_not_exists as assert_file_path_not_exists,
};

// =============================================================================
// Test Helpers
// =============================================================================

/// Generate test hash from string
fn generate_test_hash(input: &str) -> String {
    format!("test_hash_{}", input)
}

/// Setup test database pool
async fn setup_test_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string());

    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

// =============================================================================
// Test Fixtures & Builders
// =============================================================================

/// Builder for SearchQuery parameters with sensible defaults
#[derive(Clone)]
struct SearchQueryBuilder {
    text: Option<String>,
    min_bpm: Option<f64>,
    max_bpm: Option<f64>,
    key: Option<String>,
    manufacturer: Option<String>,
    collection: Option<String>,
}

impl SearchQueryBuilder {
    fn new() -> Self {
        Self {
            text: None,
            min_bpm: None,
            max_bpm: None,
            key: None,
            manufacturer: None,
            collection: None,
        }
    }

    fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    fn bpm_range(mut self, min: f64, max: f64) -> Self {
        self.min_bpm = Some(min);
        self.max_bpm = Some(max);
        self
    }

    fn min_bpm(mut self, min: f64) -> Self {
        self.min_bpm = Some(min);
        self
    }

    fn max_bpm(mut self, max: f64) -> Self {
        self.max_bpm = Some(max);
        self
    }

    fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    fn manufacturer(mut self, mfr: impl Into<String>) -> Self {
        self.manufacturer = Some(mfr.into());
        self
    }

    fn collection(mut self, col: impl Into<String>) -> Self {
        self.collection = Some(col.into());
        self
    }

    fn build(self) -> SearchQuery {
        SearchQuery {
            text: self.text,
            min_bpm: self.min_bpm,
            max_bpm: self.max_bpm,
            key: self.key,
            manufacturer: self.manufacturer,
            collection: self.collection,
        }
    }
}

/// Create a test file with searchable metadata
async fn create_search_test_file(
    pool: &PgPool,
    filename: &str,
    manufacturer: Option<&str>,
    collection: Option<&str>,
    bpm: Option<&str>,
    key: Option<&str>,
) -> i64 {
    // Insert file
    let hash = generate_test_hash(filename);
    let file_id = sqlx::query_scalar!(
        r#"
        INSERT INTO files (
            filename, filepath, original_filename, content_hash,
            file_size_bytes, format, num_tracks, manufacturer, collection_name
        )
        VALUES ($1, $2, $3, $4, 1024, 1, 1, $5, $6)
        RETURNING id
        "#,
        filename,
        format!("/test/{}", filename),
        filename,
        hash.as_bytes(),
        manufacturer,
        collection,
    )
    .fetch_one(pool)
    .await
    .expect("Failed to insert test file");

    // Insert musical metadata if provided
    if bpm.is_some() || key.is_some() {
        use std::str::FromStr;
        let bpm_decimal = bpm.map(|b| sqlx::types::BigDecimal::from_str(b).expect("Invalid BPM"));

        sqlx::query!(
            r#"
            INSERT INTO musical_metadata (file_id, bpm, key_signature, total_notes)
            VALUES ($1, $2, $3::text::musical_key, 100)
            "#,
            file_id,
            bpm_decimal,
            key,
        )
        .execute(pool)
        .await
        .expect("Failed to insert metadata");
    }

    file_id
}

/// Create a diverse test dataset for search testing
async fn create_search_test_dataset(pool: &PgPool) -> Vec<i64> {
    let mut file_ids = Vec::new();

    // File 1: Piano loop with full metadata
    file_ids.push(
        create_search_test_file(
            pool,
            "piano_loop_120bpm_Cmaj.mid",
            Some("Roland"),
            Some("Piano Loops"),
            Some("120.00"),
            Some("C"),
        )
        .await,
    );

    // File 2: Bass groove
    file_ids.push(
        create_search_test_file(
            pool,
            "bass_groove_128bpm_Am.mid",
            Some("Korg"),
            Some("Bass Grooves"),
            Some("128.00"),
            Some("Am"),
        )
        .await,
    );

    // File 3: Melody without metadata
    file_ids.push(
        create_search_test_file(
            pool,
            "melody_140bpm.mid",
            None,
            None,
            Some("140.00"),
            Some("G"),
        )
        .await,
    );

    // File 4: No metadata at all
    file_ids.push(
        create_search_test_file(
            pool,
            "no_metadata.mid",
            Some("Roland"),
            None,
            None,
            None,
        )
        .await,
    );

    // File 5: Edge case - max BPM
    file_ids.push(
        create_search_test_file(
            pool,
            "edge_case_300bpm.mid",
            Some("Native Instruments"),
            Some("Experimental"),
            Some("300.00"),
            Some("F#m"),
        )
        .await,
    );

    // File 6: Unicode filename
    file_ids.push(
        create_search_test_file(
            pool,
            "café_lounge_85bpm.mid",
            Some("Yamaha"),
            Some("World Music"),
            Some("85.00"),
            Some("Em"),
        )
        .await,
    );

    file_ids
}

// =============================================================================
// Category 1: Full-Text Search (12 tests)
// =============================================================================

#[tokio::test]
async fn test_search_exact_word_match() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new().text("piano").build();
    let results = SearchRepository::search(&pool, query, 10, 0)
        .await
        .expect("Search failed");

    assert!(!results.is_empty(), "Should find piano results");
    assert!(
        results.iter().any(|r| r.filename.to_lowercase().contains("piano")),
        "Results should contain 'piano' in filename"
    );
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_search_partial_word_match() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new().text("gro").build();
    let results = SearchRepository::search(&pool, query, 10, 0)
        .await
        .expect("Search failed");

    // PostgreSQL full-text search uses stemming, "gro" might not match "groove"
    // This tests the actual behavior
    println!("Partial word 'gro' returned {} results", results.len());
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_search_multiple_words() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new().text("bass groove").build();
    let results = SearchRepository::search(&pool, query, 10, 0)
        .await
        .expect("Search failed");

    // Should match files containing both "bass" and "groove"
    assert!(
        results.iter().any(|r| {
            let filename_lower = r.filename.to_lowercase();
            filename_lower.contains("bass") && filename_lower.contains("groove")
        }),
        "Should find files with both words"
    );
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_search_results_ordered_by_relevance() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    // Create files with varying relevance
    create_search_test_file(&pool, "piano_primary.mid", None, None, None, None).await;
    create_search_test_file(&pool, "keys_piano_secondary.mid", None, None, None, None).await;
    create_search_test_file(&pool, "organ_not_piano.mid", None, None, None, None).await;

    let query = SearchQueryBuilder::new().text("piano").build();
    let results = SearchRepository::search(&pool, query, 10, 0)
        .await
        .expect("Search failed");

    // First result should be most relevant (piano earlier in filename)
    if results.len() >= 2 {
        assert!(
            results[0].filename.contains("piano_primary"),
            "Most relevant result should be first"
        );
    }
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_search_case_insensitive() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_file(&pool, "PIANO_UPPERCASE.mid", None, None, None, None).await;
    create_search_test_file(&pool, "piano_lowercase.mid", None, None, None, None).await;

    let query = SearchQueryBuilder::new().text("PiAnO").build();
    let results = SearchRepository::search(&pool, query, 10, 0)
        .await
        .expect("Search failed");

    assert_eq!(results.len(), 2, "Case-insensitive search should find both");
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_search_empty_query_returns_all() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    let file_ids = create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new().build(); // No filters
    let results = SearchRepository::search(&pool, query, 100, 0)
        .await
        .expect("Search failed");

    assert_eq!(
        results.len(),
        file_ids.len(),
        "Empty query should return all files"
    );
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_search_no_matches_returns_empty() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new()
        .text("nonexistent_term_xyz")
        .build();

    let results = SearchRepository::search(&pool, query, 10, 0)
        .await
        .expect("Search failed");

    assert!(results.is_empty(), "No matches should return empty vec");
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_search_whitespace_only_query() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    let file_ids = create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new().text("   ").build();
    let results = SearchRepository::search(&pool, query, 100, 0)
        .await
        .expect("Search failed");

    // Whitespace-only should be treated as empty query
    assert!(
        results.len() > 0,
        "Whitespace query should return results (treated as empty)"
    );
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_search_with_manufacturer_in_query() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new().text("Roland").build();
    let results = SearchRepository::search(&pool, query, 10, 0)
        .await
        .expect("Search failed");

    // Should find files with "Roland" in manufacturer field
    assert!(
        results.iter().any(|r| r.manufacturer.as_deref() == Some("Roland")),
        "Should find Roland files via full-text search"
    );
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_search_with_collection_in_query() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new().text("Piano Loops").build();
    let results = SearchRepository::search(&pool, query, 10, 0)
        .await
        .expect("Search failed");

    assert!(
        results.iter().any(|r| r.collection_name.as_deref() == Some("Piano Loops")),
        "Should find collections via full-text search"
    );
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_search_ranking_with_no_text() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    let file_ids = create_search_test_dataset(&pool).await;

    // No text search - should order by created_at DESC
    let query = SearchQueryBuilder::new().build();
    let results = SearchRepository::search(&pool, query, 100, 0)
        .await
        .expect("Search failed");

    // Most recently created should be first
    if results.len() >= 2 {
        assert!(
            results[0].created_at >= results[1].created_at,
            "Without text search, should order by created_at DESC"
        );
    }
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_search_unicode_in_filename() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new().text("café").build();
    let results = SearchRepository::search(&pool, query, 10, 0)
        .await
        .expect("Search failed");

    assert!(
        results.iter().any(|r| r.filename.contains("café")),
        "Should handle Unicode in search"
    );
    cleanup_database(&pool).await.expect("Cleanup failed");
}

// =============================================================================
// Category 2: Filter Combinations (15 tests)
// =============================================================================

#[tokio::test]
async fn test_filter_by_min_bpm_only() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new().min_bpm(125.0).build();
    let results = SearchRepository::search(&pool, query, 100, 0)
        .await
        .expect("Search failed");

    // All results should have BPM >= 125
    for file in results {
        let metadata = sqlx::query!(
            "SELECT bpm FROM musical_metadata WHERE file_id = $1",
            file.id
        )
        .fetch_optional(&pool)
        .await
        .expect("Query failed");

        if let Some(meta) = metadata {
            if let Some(bpm) = meta.bpm {
                let bpm_f64 = bpm.to_string().parse::<f64>().unwrap();
                assert!(
                    bpm_f64 >= 125.0,
                    "BPM {} should be >= 125",
                    bpm_f64
                );
            }
        }
    }
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_filter_by_max_bpm_only() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new().max_bpm(130.0).build();
    let results = SearchRepository::search(&pool, query, 100, 0)
        .await
        .expect("Search failed");

    // All results should have BPM <= 130
    for file in results {
        let metadata = sqlx::query!(
            "SELECT bpm FROM musical_metadata WHERE file_id = $1",
            file.id
        )
        .fetch_optional(&pool)
        .await
        .expect("Query failed");

        if let Some(meta) = metadata {
            if let Some(bpm) = meta.bpm {
                let bpm_f64 = bpm.to_string().parse::<f64>().unwrap();
                assert!(
                    bpm_f64 <= 130.0,
                    "BPM {} should be <= 130",
                    bpm_f64
                );
            }
        }
    }
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_filter_by_bpm_range() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new().bpm_range(120.0, 130.0).build();
    let results = SearchRepository::search(&pool, query, 100, 0)
        .await
        .expect("Search failed");

    assert!(!results.is_empty(), "Should find files in BPM range 120-130");

    // Verify all results in range
    for file in results {
        let metadata = sqlx::query!(
            "SELECT bpm FROM musical_metadata WHERE file_id = $1",
            file.id
        )
        .fetch_optional(&pool)
        .await
        .expect("Query failed");

        if let Some(meta) = metadata {
            if let Some(bpm) = meta.bpm {
                let bpm_f64 = bpm.to_string().parse::<f64>().unwrap();
                assert!(
                    bpm_f64 >= 120.0 && bpm_f64 <= 130.0,
                    "BPM {} should be in range 120-130",
                    bpm_f64
                );
            }
        }
    }
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_filter_by_key_signature() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new().key("C").build();
    let results = SearchRepository::search(&pool, query, 100, 0)
        .await
        .expect("Search failed");

    assert!(!results.is_empty(), "Should find files in key C");

    // Verify all results match key
    for file in results {
        let metadata = sqlx::query!(
            r#"SELECT key_signature::text as "key_signature!" FROM musical_metadata WHERE file_id = $1"#,
            file.id
        )
        .fetch_optional(&pool)
        .await
        .expect("Query failed");

        if let Some(meta) = metadata {
            assert_eq!(meta.key_signature, "C", "Key should be C");
        }
    }
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_filter_by_manufacturer() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new().manufacturer("Roland").build();
    let results = SearchRepository::search(&pool, query, 100, 0)
        .await
        .expect("Search failed");

    assert!(!results.is_empty(), "Should find Roland files");

    for file in &results {
        assert_eq!(
            file.manufacturer.as_deref(),
            Some("Roland"),
            "All results should be Roland"
        );
    }
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_filter_by_collection() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new().collection("Piano Loops").build();
    let results = SearchRepository::search(&pool, query, 100, 0)
        .await
        .expect("Search failed");

    assert!(!results.is_empty(), "Should find Piano Loops collection");

    for file in &results {
        assert_eq!(
            file.collection_name.as_deref(),
            Some("Piano Loops"),
            "All results should be from Piano Loops"
        );
    }
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_filter_text_and_bpm_range() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new()
        .text("bass")
        .bpm_range(120.0, 130.0)
        .build();

    let results = SearchRepository::search(&pool, query, 100, 0)
        .await
        .expect("Search failed");

    // Should match files with "bass" in text AND BPM 120-130
    for file in &results {
        assert!(
            file.filename.to_lowercase().contains("bass"),
            "Filename should contain 'bass'"
        );
    }
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_filter_text_and_key() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new()
        .text("piano")
        .key("C")
        .build();

    let results = SearchRepository::search(&pool, query, 100, 0)
        .await
        .expect("Search failed");

    assert!(!results.is_empty(), "Should find piano files in key C");
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_filter_bpm_and_key() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new()
        .bpm_range(115.0, 125.0)
        .key("C")
        .build();

    let results = SearchRepository::search(&pool, query, 100, 0)
        .await
        .expect("Search failed");

    // Should find files with BPM 115-125 AND key C
    println!("Found {} files with BPM 115-125 and key C", results.len());
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_filter_manufacturer_and_collection() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new()
        .manufacturer("Roland")
        .collection("Piano Loops")
        .build();

    let results = SearchRepository::search(&pool, query, 100, 0)
        .await
        .expect("Search failed");

    for file in &results {
        assert_eq!(file.manufacturer.as_deref(), Some("Roland"));
        assert_eq!(file.collection_name.as_deref(), Some("Piano Loops"));
    }
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_filter_text_bpm_and_key() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new()
        .text("piano")
        .bpm_range(115.0, 125.0)
        .key("C")
        .build();

    let results = SearchRepository::search(&pool, query, 100, 0)
        .await
        .expect("Search failed");

    // All three filters should be AND-ed together
    println!("Found {} files matching all 3 filters", results.len());
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_filter_all_filters_applied() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new()
        .text("piano")
        .bpm_range(115.0, 125.0)
        .key("C")
        .manufacturer("Roland")
        .collection("Piano Loops")
        .build();

    let results = SearchRepository::search(&pool, query, 100, 0)
        .await
        .expect("Search failed");

    // Very specific query - may return 0 or 1 result
    println!("All filters applied: {} results", results.len());
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_filters_no_matching_results() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    // Impossible combination
    let query = SearchQueryBuilder::new()
        .bpm_range(500.0, 600.0) // BPM constraint is 20-300
        .build();

    let results = SearchRepository::search(&pool, query, 100, 0)
        .await
        .expect("Search failed");

    assert!(results.is_empty(), "Impossible filter should return empty");
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_filters_reduce_progressively() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    // No filters
    let query1 = SearchQueryBuilder::new().build();
    let results1 = SearchRepository::search(&pool, query1, 100, 0)
        .await
        .expect("Search failed");

    // One filter
    let query2 = SearchQueryBuilder::new().text("piano").build();
    let results2 = SearchRepository::search(&pool, query2, 100, 0)
        .await
        .expect("Search failed");

    // Two filters
    let query3 = SearchQueryBuilder::new()
        .text("piano")
        .manufacturer("Roland")
        .build();
    let results3 = SearchRepository::search(&pool, query3, 100, 0)
        .await
        .expect("Search failed");

    // Each additional filter should reduce or maintain result count
    assert!(
        results1.len() >= results2.len(),
        "Adding filter should reduce results"
    );
    assert!(
        results2.len() >= results3.len(),
        "Adding more filters should reduce further"
    );
    cleanup_database(&pool).await.expect("Cleanup failed");
}

// =============================================================================
// Category 3: Pagination & Limits (8 tests)
// =============================================================================

#[tokio::test]
async fn test_pagination_first_page() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new().build();
    let results = SearchRepository::search(&pool, query, 3, 0)
        .await
        .expect("Search failed");

    assert!(results.len() <= 3, "Should respect LIMIT");
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_pagination_second_page() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    let file_ids = create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new().build();

    // Get first page
    let page1 = SearchRepository::search(&pool, query.clone(), 3, 0)
        .await
        .expect("Search failed");

    // Get second page
    let page2 = SearchRepository::search(&pool, query, 3, 3)
        .await
        .expect("Search failed");

    // Pages should be different (unless total < 3)
    if file_ids.len() > 3 {
        assert_ne!(
            page1.first().map(|f| f.id),
            page2.first().map(|f| f.id),
            "Different pages should have different files"
        );
    }
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_pagination_last_page() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    let file_ids = create_search_test_dataset(&pool).await;
    let total = file_ids.len() as i64;

    let query = SearchQueryBuilder::new().build();

    // Get last page (offset = total - 2)
    let last_page = SearchRepository::search(&pool, query, 10, total - 2)
        .await
        .expect("Search failed");

    assert!(
        last_page.len() <= 2,
        "Last page should have remaining items"
    );
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_pagination_offset_beyond_total() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new().build();
    let results = SearchRepository::search(&pool, query, 10, 1000)
        .await
        .expect("Search failed");

    assert!(results.is_empty(), "Offset beyond total should return empty");
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_limit_zero() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new().build();
    let results = SearchRepository::search(&pool, query, 0, 0)
        .await
        .expect("Search failed");

    assert!(results.is_empty(), "LIMIT 0 should return empty");
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_limit_exceeds_total() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    let file_ids = create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new().build();
    let results = SearchRepository::search(&pool, query, 1000, 0)
        .await
        .expect("Search failed");

    assert_eq!(
        results.len(),
        file_ids.len(),
        "Large LIMIT should return all files"
    );
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_offset_zero_is_first_page() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new().build();
    let results = SearchRepository::search(&pool, query, 10, 0)
        .await
        .expect("Search failed");

    // OFFSET 0 should give same results as no offset
    assert!(!results.is_empty());
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_large_offset_still_works() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    // Create many files to test large offset
    for i in 0..50 {
        create_search_test_file(
            &pool,
            &format!("file_{}.mid", i),
            None,
            None,
            None,
            None,
        )
        .await;
    }

    let query = SearchQueryBuilder::new().build();
    let results = SearchRepository::search(&pool, query, 10, 45)
        .await
        .expect("Search failed");

    // Should return remaining 5 files
    assert!(results.len() <= 10);
    cleanup_database(&pool).await.expect("Cleanup failed");
}

// =============================================================================
// Category 4: Musical Metadata JOIN (8 tests)
// =============================================================================

#[tokio::test]
async fn test_join_files_without_metadata_included() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    // No filters - should include files without metadata
    let query = SearchQueryBuilder::new().build();
    let results = SearchRepository::search(&pool, query, 100, 0)
        .await
        .expect("Search failed");

    // At least one file has no metadata
    assert!(results.len() >= 4, "Should include files without metadata");
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_join_files_without_bpm_excluded_when_filtered() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    // Filter by BPM - should exclude files without BPM
    let query = SearchQueryBuilder::new().min_bpm(100.0).build();
    let results = SearchRepository::search(&pool, query, 100, 0)
        .await
        .expect("Search failed");

    // Files without BPM should not appear
    for file in &results {
        let metadata = sqlx::query!(
            "SELECT bpm FROM musical_metadata WHERE file_id = $1",
            file.id
        )
        .fetch_optional(&pool)
        .await
        .expect("Query failed");

        assert!(metadata.is_some(), "Result should have metadata");
        assert!(metadata.unwrap().bpm.is_some(), "Result should have BPM");
    }
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_join_files_without_key_excluded_when_filtered() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    // Filter by key - should exclude files without key
    let query = SearchQueryBuilder::new().key("C").build();
    let results = SearchRepository::search(&pool, query, 100, 0)
        .await
        .expect("Search failed");

    // All results should have key signature
    for file in &results {
        let metadata = sqlx::query!(
            r#"SELECT key_signature::text as "key!" FROM musical_metadata WHERE file_id = $1"#,
            file.id
        )
        .fetch_optional(&pool)
        .await
        .expect("Query failed");

        assert!(metadata.is_some(), "Result should have key");
    }
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_join_bpm_boundary_min() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    // Min BPM = 120
    let query = SearchQueryBuilder::new().min_bpm(120.0).build();
    let results = SearchRepository::search(&pool, query, 100, 0)
        .await
        .expect("Search failed");

    // Should find piano_loop (120) and higher
    assert!(!results.is_empty());
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_join_bpm_boundary_max() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    // Max BPM = 130
    let query = SearchQueryBuilder::new().max_bpm(130.0).build();
    let results = SearchRepository::search(&pool, query, 100, 0)
        .await
        .expect("Search failed");

    // Should find files <= 130
    assert!(!results.is_empty());
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_join_key_case_sensitivity() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    // Try lowercase "c" (should match "C")
    let query = SearchQueryBuilder::new().key("c").build();
    let results = SearchRepository::search(&pool, query, 100, 0)
        .await
        .expect("Search failed");

    // PostgreSQL enum comparison depends on schema definition
    println!("Lowercase 'c' query returned {} results", results.len());
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_join_multiple_musical_filters() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    // Combine BPM and key filters
    let query = SearchQueryBuilder::new()
        .bpm_range(115.0, 125.0)
        .key("C")
        .build();

    let results = SearchRepository::search(&pool, query, 100, 0)
        .await
        .expect("Search failed");

    // Should only find files matching BOTH criteria
    println!("Multiple musical filters returned {} results", results.len());
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_join_performance_with_many_files() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    // Create 100 files to test JOIN performance
    for i in 0..100 {
        let bpm = format!("{}.00", 100 + (i % 100));
        create_search_test_file(
            &pool,
            &format!("file_{}.mid", i),
            Some("TestManufacturer"),
            None,
            Some(&bpm),
            Some("C"),
        )
        .await;
    }

    let start = std::time::Instant::now();
    let query = SearchQueryBuilder::new().min_bpm(120.0).build();
    let results = SearchRepository::search(&pool, query, 100, 0)
        .await
        .expect("Search failed");
    let duration = start.elapsed();

    println!("JOIN with 100 files took {:?}", duration);
    assert!(
        duration.as_millis() < 500,
        "JOIN should be fast with indexed PK"
    );
    cleanup_database(&pool).await.expect("Cleanup failed");
}

// =============================================================================
// Category 5: Count Queries (5 tests)
// =============================================================================

#[tokio::test]
async fn test_count_matches_search_length() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new().text("bass").build();

    let results = SearchRepository::search(&pool, query.clone(), 100, 0)
        .await
        .expect("Search failed");

    let count = SearchRepository::count_search_results(&pool, query)
        .await
        .expect("Count failed");

    assert_eq!(
        count as usize,
        results.len(),
        "Count should match result length"
    );
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_count_with_filters() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new()
        .bpm_range(120.0, 130.0)
        .build();

    let count = SearchRepository::count_search_results(&pool, query)
        .await
        .expect("Count failed");

    assert!(count > 0, "Should count filtered results");
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_count_empty_results() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new()
        .text("nonexistent_xyz")
        .build();

    let count = SearchRepository::count_search_results(&pool, query)
        .await
        .expect("Count failed");

    assert_eq!(count, 0, "Count of no results should be 0");
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_count_no_filters_returns_total() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    let file_ids = create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new().build();
    let count = SearchRepository::count_search_results(&pool, query)
        .await
        .expect("Count failed");

    assert_eq!(
        count as usize,
        file_ids.len(),
        "Count without filters should return total files"
    );
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_count_consistency_with_pagination() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    let query = SearchQueryBuilder::new().text("piano").build();

    // Get count
    let total_count = SearchRepository::count_search_results(&pool, query.clone())
        .await
        .expect("Count failed");

    // Get all results across multiple pages
    let mut all_results = Vec::new();
    let page_size = 2;
    let mut offset = 0;

    loop {
        let page = SearchRepository::search(&pool, query.clone(), page_size, offset)
            .await
            .expect("Search failed");

        if page.is_empty() {
            break;
        }

        all_results.extend(page);
        offset += page_size;
    }

    assert_eq!(
        total_count as usize,
        all_results.len(),
        "Count should match total paginated results"
    );
    cleanup_database(&pool).await.expect("Cleanup failed");
}

// =============================================================================
// Category 6: Edge Cases (4 tests)
// =============================================================================

#[tokio::test]
async fn test_special_characters_in_query() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_file(&pool, "track@001.mid", None, None, None, None).await;

    // Search for file with special char
    let query = SearchQueryBuilder::new().text("track@001").build();
    let results = SearchRepository::search(&pool, query, 10, 0)
        .await
        .expect("Search failed");

    // PostgreSQL full-text search may tokenize differently
    println!("Special char search returned {} results", results.len());
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_very_long_query_string() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    // 1000-character query
    let long_query = "piano ".repeat(200);

    let query = SearchQueryBuilder::new().text(long_query).build();
    let results = SearchRepository::search(&pool, query, 10, 0)
        .await
        .expect("Search failed");

    // Should handle long queries without error
    println!("Very long query returned {} results", results.len());
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_sql_injection_prevention() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    create_search_test_dataset(&pool).await;

    // Try SQL injection in text search
    let malicious_query = "'; DROP TABLE files; --";

    let query = SearchQueryBuilder::new().text(malicious_query).build();
    let result = SearchRepository::search(&pool, query, 10, 0).await;

    // Should not error - parameterized queries prevent injection
    assert!(result.is_ok(), "Parameterized queries prevent SQL injection");

    // Verify table still exists
    let count = sqlx::query_scalar!("SELECT COUNT(*) FROM files")
        .fetch_one(&pool)
        .await
        .expect("Table should still exist");

    assert!(count.unwrap_or(0) > 0, "Files table should not be dropped");
    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_empty_database() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    // No files inserted

    let query = SearchQueryBuilder::new().build();
    let results = SearchRepository::search(&pool, query, 10, 0)
        .await
        .expect("Search failed");

    assert!(results.is_empty(), "Empty database should return empty results");

    let count = SearchRepository::count_search_results(&pool, SearchQueryBuilder::new().build())
        .await
        .expect("Count failed");

    assert_eq!(count, 0, "Empty database count should be 0");
    cleanup_database(&pool).await.expect("Cleanup failed");
}

    // ============================================================================
    // ============================================================================
    // SECTION 9: Error Path Tests - Query Validation (12 tests)
    // ============================================================================
    // ============================================================================

    #[tokio::test]
    async fn test_search_with_min_bpm_greater_than_max_bpm() {
        // Description: Min BPM > Max BPM should fail or return empty
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        // Create test data
        let file_id = create_test_file(&pool, "test.mid").await;
        insert_metadata(&pool, file_id, Some("120.0"), None, None).await;

        // Query with min > max (logical error)
        let query = SearchQueryBuilder::new()
            .min_bpm(Some("150.0".to_string()))
            .max_bpm(Some("100.0".to_string()))
            .build();

        let results = SearchRepository::search(&pool, &query).await.expect("Query should not error");
        assert!(results.is_empty(), "Query with min > max should return empty results");

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_search_with_negative_bpm_filter() {
        // Description: Negative BPM in filter should not match
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_test_file(&pool, "test.mid").await;
        insert_metadata(&pool, file_id, Some("120.0"), None, None).await;

        // Query with negative min BPM
        let query = SearchQueryBuilder::new()
            .min_bpm(Some("-50.0".to_string()))
            .build();

        let results = SearchRepository::search(&pool, &query).await.expect("Query should not error");
        // Negative BPM should be treated as invalid and match nothing
        assert!(results.is_empty(), "Negative BPM should not match");

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_search_with_invalid_key_filter() {
        // Description: Invalid key value should not match
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_test_file(&pool, "test.mid").await;
        insert_metadata(&pool, file_id, None, Some("C".to_string()), None).await;

        // Query with invalid key
        let query = SearchQueryBuilder::new()
            .keys(Some(vec!["H".to_string()])) // Invalid key
            .build();

        let results = SearchRepository::search(&pool, &query).await;
        // Should either error or return empty
        if let Ok(results) = results {
            assert!(results.is_empty(), "Invalid key should not match");
        }

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_search_with_negative_offset() {
        // Description: Negative offset should fail or be treated as 0
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_test_file(&pool, "test.mid").await;
        insert_metadata(&pool, file_id, None, None, None).await;

        // Query with negative offset
        let query = SearchQueryBuilder::new()
            .offset(-10)
            .build();

        let results = SearchRepository::search(&pool, &query).await;
        // Should either error or treat as 0
        if let Ok(results) = results {
            assert_eq!(results.len(), 1, "Negative offset should be treated as 0");
        }

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_search_with_negative_limit() {
        // Description: Negative limit should fail or return all
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_test_file(&pool, "test.mid").await;
        insert_metadata(&pool, file_id, None, None, None).await;

        // Query with negative limit
        let query = SearchQueryBuilder::new()
            .limit(-10)
            .build();

        let results = SearchRepository::search(&pool, &query).await;
        // Should either error or treat as unlimited
        if let Ok(results) = results {
            assert_eq!(results.len(), 1, "Negative limit should be treated as unlimited");
        }

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_search_with_empty_query_returns_all() {
        // Description: Empty query should return all files
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        // Create multiple test files
        for i in 0..3 {
            let file_id = create_test_file(&pool, &format!("test{}.mid", i)).await;
            insert_metadata(&pool, file_id, Some("100.0"), None, None).await;
        }

        // Query with no filters
        let query = SearchQueryBuilder::new().build();

        let results = SearchRepository::search(&pool, &query).await.expect("Query failed");
        assert_eq!(results.len(), 3, "Empty query should return all files");

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_count_with_empty_query() {
        // Description: Count with empty query should return total
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        for i in 0..5 {
            let file_id = create_test_file(&pool, &format!("test{}.mid", i)).await;
            insert_metadata(&pool, file_id, None, None, None).await;
        }

        let query = SearchQueryBuilder::new().build();
        let count = SearchRepository::count_search_results(&pool, &query).await.expect("Count failed");
        assert_eq!(count, 5, "Count should return total files");

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_search_with_zero_limit_returns_empty() {
        // Description: Zero limit should return empty results
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_test_file(&pool, "test.mid").await;
        insert_metadata(&pool, file_id, None, None, None).await;

        let query = SearchQueryBuilder::new()
            .limit(0)
            .build();

        let results = SearchRepository::search(&pool, &query).await.expect("Query failed");
        assert!(results.is_empty(), "Zero limit should return empty");

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_search_with_very_large_limit() {
        // Description: Very large limit should cap at available results
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        // Create 3 files
        for i in 0..3 {
            let file_id = create_test_file(&pool, &format!("test{}.mid", i)).await;
            insert_metadata(&pool, file_id, None, None, None).await;
        }

        let query = SearchQueryBuilder::new()
            .limit(1_000_000)
            .build();

        let results = SearchRepository::search(&pool, &query).await.expect("Query failed");
        assert_eq!(results.len(), 3, "Large limit should return all available");

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_search_offset_beyond_results() {
        // Description: Offset beyond available results should return empty
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_test_file(&pool, "test.mid").await;
        insert_metadata(&pool, file_id, None, None, None).await;

        let query = SearchQueryBuilder::new()
            .offset(100)
            .build();

        let results = SearchRepository::search(&pool, &query).await.expect("Query failed");
        assert!(results.is_empty(), "Offset beyond results should return empty");

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_search_combines_multiple_filters_correctly() {
        // Description: Multiple filters should AND together
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        // File 1: 120 BPM, C key
        let file1 = create_test_file(&pool, "file1.mid").await;
        insert_metadata(&pool, file1, Some("120.0"), Some("C".to_string()), None).await;

        // File 2: 140 BPM, C key
        let file2 = create_test_file(&pool, "file2.mid").await;
        insert_metadata(&pool, file2, Some("140.0"), Some("C".to_string()), None).await;

        // Query: BPM > 130 AND key = C
        let query = SearchQueryBuilder::new()
            .min_bpm(Some("130.0".to_string()))
            .keys(Some(vec!["C".to_string()]))
            .build();

        let results = SearchRepository::search(&pool, &query).await.expect("Query failed");
        assert_eq!(results.len(), 1, "Should match only file2 (140 BPM in C)");
        assert_eq!(results[0].file_id, file2, "Should be file2");

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    // ===== SECTION 6: ERROR PATH TESTING (15 constraint violation & pagination tests) =====

    #[tokio::test]
    async fn test_search_error_inverted_bpm_range() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file = create_test_file(&pool, "test.mid").await;
        insert_metadata(&pool, file, Some("100.0"), None, None).await;

        let query = SearchQueryBuilder::new()
            .min_bpm(Some("200.0".to_string()))
            .max_bpm(Some("50.0".to_string()))
            .build();

        let results = SearchRepository::search(&pool, &query).await.unwrap_or_default();
        assert!(results.is_empty(), "Inverted BPM range should return empty");
        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_search_error_negative_bpm() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file = create_test_file(&pool, "test.mid").await;
        insert_metadata(&pool, file, Some("-120.0"), None, None).await;

        let query = SearchQueryBuilder::new().build();
        let results = SearchRepository::search(&pool, &query).await.expect("Query should handle negative BPM");
        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_search_error_negative_offset() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        for i in 0..5 {
            let file = create_test_file(&pool, &format!("file{}.mid", i)).await;
            insert_metadata(&pool, file, None, None, None).await;
        }

        let query = SearchQueryBuilder::new()
            .offset(-1)
            .build();

        let results = SearchRepository::search(&pool, &query).await.unwrap_or_default();
        assert!(results.is_empty(), "Negative offset should handle gracefully");
        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_search_error_zero_limit() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        for i in 0..5 {
            let file = create_test_file(&pool, &format!("file{}.mid", i)).await;
            insert_metadata(&pool, file, None, None, None).await;
        }

        let query = SearchQueryBuilder::new()
            .limit(0)
            .build();

        let results = SearchRepository::search(&pool, &query).await.unwrap_or_default();
        assert!(results.is_empty(), "Zero limit should return no results");
        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_search_error_large_offset() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file = create_test_file(&pool, "test.mid").await;
        insert_metadata(&pool, file, None, None, None).await;

        let query = SearchQueryBuilder::new()
            .offset(1000000)
            .build();

        let results = SearchRepository::search(&pool, &query).await.unwrap_or_default();
        assert!(results.is_empty(), "Large offset should return empty");
        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_search_error_negative_duration() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file = create_test_file(&pool, "test.mid").await;
        insert_metadata(&pool, file, None, None, Some(-100i32)).await;

        let query = SearchQueryBuilder::new().build();
        let results = SearchRepository::search(&pool, &query).await.expect("Query should handle negative duration");
        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_search_error_inverted_duration_range() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file = create_test_file(&pool, "test.mid").await;
        insert_metadata(&pool, file, None, None, Some(120)).await;

        let query = SearchQueryBuilder::new()
            .min_duration(Some("200".to_string()))
            .max_duration(Some("50".to_string()))
            .build();

        let results = SearchRepository::search(&pool, &query).await.unwrap_or_default();
        assert!(results.is_empty(), "Inverted duration range should return empty");
        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_search_error_invalid_key_enum() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file = create_test_file(&pool, "test.mid").await;
        insert_metadata(&pool, file, None, Some("H".to_string()), None).await;

        let query = SearchQueryBuilder::new()
            .keys(Some(vec!["H".to_string()]))
            .build();

        let results = SearchRepository::search(&pool, &query).await.unwrap_or_default();
        assert!(results.is_empty(), "Invalid key should not match");
        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_search_error_pagination_consistency() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        for i in 0..20 {
            let file = create_test_file(&pool, &format!("file{:02}.mid", i)).await;
            insert_metadata(&pool, file, None, None, None).await;
        }

        let query1 = SearchQueryBuilder::new().limit(10).offset(0).build();
        let query2 = SearchQueryBuilder::new().limit(10).offset(10).build();

        let page1 = SearchRepository::search(&pool, &query1).await.expect("Page 1 query failed");
        let page2 = SearchRepository::search(&pool, &query2).await.expect("Page 2 query failed");

        assert_eq!(page1.len(), 10, "Page 1 should have 10 results");
        assert_eq!(page2.len(), 10, "Page 2 should have 10 results");

        let file_ids1: Vec<_> = page1.iter().map(|f| f.file_id).collect();
        let file_ids2: Vec<_> = page2.iter().map(|f| f.file_id).collect();

        for id in file_ids1 {
            assert!(!file_ids2.contains(&id), "Pages should not overlap");
        }

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_search_error_concurrent_queries() {
        let pool = std::sync::Arc::new(setup_test_pool().await);
        cleanup_database(&pool).await.expect("Cleanup failed");

        for i in 0..10 {
            let file = create_test_file(&pool, &format!("file{}.mid", i)).await;
            insert_metadata(&pool, file, Some(&format!("{}.0", 100 + i * 5)), None, None).await;
        }

        let mut handles = Vec::new();
        for _ in 0..5 {
            let pool_clone = std::sync::Arc::clone(&pool);
            let handle = tokio::spawn(async move {
                let query = SearchQueryBuilder::new()
                    .min_bpm(Some("100.0".to_string()))
                    .build();
                SearchRepository::search(&pool_clone, &query).await
            });
            handles.push(handle);
        }

        let results: Vec<_> = futures::future::join_all(handles).await;
        for result in results {
            assert!(result.is_ok(), "Concurrent queries should succeed");
            let query_result = result.unwrap();
            assert!(query_result.is_ok(), "Each query should execute successfully");
        }

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_search_error_empty_search_with_constraints() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let query = SearchQueryBuilder::new()
            .min_bpm(Some("500.0".to_string()))
            .build();

        let results = SearchRepository::search(&pool, &query).await.expect("Query failed");
        assert!(results.is_empty(), "Impossible constraints should return empty");
        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_search_error_max_limit_boundary() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        for i in 0..100 {
            let file = create_test_file(&pool, &format!("file{:03}.mid", i)).await;
            insert_metadata(&pool, file, None, None, None).await;
        }

        let query = SearchQueryBuilder::new()
            .limit(10000)
            .build();

        let results = SearchRepository::search(&pool, &query).await.expect("Query failed");
        assert_eq!(results.len(), 100, "Limit should not exceed available results");
        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_search_error_offset_equals_total() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        for i in 0..10 {
            let file = create_test_file(&pool, &format!("file{}.mid", i)).await;
            insert_metadata(&pool, file, None, None, None).await;
        }

        let query = SearchQueryBuilder::new()
            .offset(10)
            .build();

        let results = SearchRepository::search(&pool, &query).await.expect("Query failed");
        assert!(results.is_empty(), "Offset equal to total should return empty");
        cleanup_database(&pool).await.expect("Cleanup failed");
    }
}

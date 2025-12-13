#[allow(dead_code, unused_imports, unused_variables)]

/// Tests for pipeline/src-tauri/src/commands/search.rs
/// Commands: search_files, get_all_tags, get_files_by_tag, get_bpm_range, get_all_keys
use crate::common::*;
use midi_pipeline::commands::search::{
    get_all_tags_impl, get_bpm_range_impl, search_files_impl, SearchFilters,
};

#[tokio::test]
async fn test_search_files_empty_query() {
    let state = setup_test_state().await;
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Cleanup first
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/search_empty%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");

    // Create test files
    for i in 0..5 {
        create_test_file(pool, &format!("search_empty_{}.mid", i)).await;
    }

    // Search with empty query (should return all files with our test prefix)
    let filters =
        SearchFilters { category: None, min_bpm: None, max_bpm: None, key_signature: None };

    let results = search_files_impl("".to_string(), filters, 1, 100, &state)
        .await
        .expect("Search failed");

    assert!(results.total_count >= 5, "Should find at least 5 files");

    // Cleanup
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/search_empty%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");
}

#[tokio::test]
async fn test_search_files_with_filters() {
    let state = setup_test_state().await;
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Cleanup first
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/search_filter%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");

    // Create test files with metadata
    for i in 0..3 {
        let file_id = MidiFileBuilder::new()
            .with_path(&format!("/test/search_filter_{}.mid", i))
            .insert(pool)
            .await;

        // Insert metadata with BPM
        MetadataBuilder::new(file_id)
            .with_bpm(120.0 + (i as f64) * 10.0)
            .with_key("Cm")
            .insert(pool)
            .await;

        // Insert category
        sqlx::query("INSERT INTO file_categories (file_id, primary_category) VALUES ($1, 'BASS')")
            .bind(file_id)
            .execute(pool)
            .await
            .expect("Failed to insert category");
    }

    // Search with BPM filter
    let filters = SearchFilters {
        category: Some("BASS".to_string()),
        min_bpm: Some(115.0),
        max_bpm: Some(135.0),
        key_signature: Some("Cm".to_string()),
    };

    let results = search_files_impl("".to_string(), filters, 1, 100, &state)
        .await
        .expect("Search failed");

    assert!(
        results.items.len() >= 2,
        "Should find at least 2 files matching filters"
    );

    // Cleanup
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/search_filter%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");
}

#[tokio::test]
async fn test_search_files_pagination() {
    let state = setup_test_state().await;
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Cleanup first
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/search_page%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");

    // Create 10 test files
    for i in 0..10 {
        create_test_file(pool, &format!("search_page_{}.mid", i)).await;
    }

    let filters =
        SearchFilters { category: None, min_bpm: None, max_bpm: None, key_signature: None };

    // Get first page (5 items)
    let page1 = search_files_impl("".to_string(), filters.clone(), 1, 5, &state)
        .await
        .expect("Search failed");

    assert!(page1.items.len() <= 5, "Page 1 should have at most 5 items");
    assert!(page1.total_count >= 10, "Total count should be at least 10");

    // Get second page
    let page2 = search_files_impl("".to_string(), filters, 2, 5, &state)
        .await
        .expect("Search failed");

    assert!(page2.items.len() <= 5, "Page 2 should have at most 5 items");

    // Cleanup
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/search_page%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");
}

#[tokio::test]
async fn test_get_all_tags() {
    let state = setup_test_state().await;
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Cleanup first
    sqlx::query("DELETE FROM file_tags WHERE tag_name LIKE 'test_tag_%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");

    // Insert test tags
    let file_id = create_test_file(pool, "tagged_file.mid").await;

    for i in 0..3 {
        sqlx::query(
            "INSERT INTO file_tags (file_id, tag_name) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(file_id)
        .bind(format!("test_tag_{}", i))
        .execute(pool)
        .await
        .expect("Failed to insert tag");
    }

    // Get all tags
    let tags = get_all_tags_impl(&state).await.expect("Get tags failed");

    // Should include our test tags
    let test_tags: Vec<_> = tags.iter().filter(|t| t.starts_with("test_tag_")).collect();
    assert!(test_tags.len() >= 3, "Should find at least 3 test tags");

    // Cleanup
    sqlx::query("DELETE FROM file_tags WHERE tag_name LIKE 'test_tag_%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");
    sqlx::query("DELETE FROM files WHERE id = $1")
        .bind(file_id)
        .execute(pool)
        .await
        .expect("Cleanup failed");
}

#[tokio::test]
async fn test_get_files_by_tag() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Cleanup first
    sqlx::query("DELETE FROM file_tags WHERE tag_name = 'test_specific_tag'")
        .execute(pool)
        .await
        .expect("Cleanup failed");

    // Create files with specific tag
    let mut file_ids = Vec::new();
    for i in 0..2 {
        let file_id = create_test_file(pool, &format!("tagged_specific_{}.mid", i)).await;
        file_ids.push(file_id);

        sqlx::query(
            "INSERT INTO file_tags (file_id, tag_name) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(file_id)
        .bind("test_specific_tag")
        .execute(pool)
        .await
        .expect("Failed to insert tag");
    }

    // Query files by tag
    let files: Vec<(i64,)> = sqlx::query_as(
        r#"
        SELECT f.id
        FROM files f
        INNER JOIN file_tags ft ON f.id = ft.file_id
        WHERE ft.tag_name = $1
        "#,
    )
    .bind("test_specific_tag")
    .fetch_all(pool)
    .await
    .expect("Query failed");

    assert_eq!(files.len(), 2, "Should find 2 files with the tag");

    // Cleanup
    sqlx::query("DELETE FROM file_tags WHERE tag_name = 'test_specific_tag'")
        .execute(pool)
        .await
        .expect("Cleanup failed");
    for file_id in file_ids {
        sqlx::query("DELETE FROM files WHERE id = $1")
            .bind(file_id)
            .execute(pool)
            .await
            .expect("Cleanup failed");
    }
}

#[tokio::test]
async fn test_get_bpm_range() {
    let state = setup_test_state().await;
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Cleanup first
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/bpm_range%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");

    // Create files with different BPMs
    for bpm in &[80.0, 120.0, 160.0] {
        let file_id = MidiFileBuilder::new()
            .with_path(&format!("/test/bpm_range_{}.mid", bpm))
            .insert(pool)
            .await;

        MetadataBuilder::new(file_id).with_bpm(*bpm).insert(pool).await;
    }

    // Get BPM range
    let range = get_bpm_range_impl(&state).await.expect("Get BPM range failed");

    // Range should include our test values
    assert!(range.min <= 80.0, "Min BPM should be <= 80");
    assert!(range.max >= 160.0, "Max BPM should be >= 160");

    // Cleanup
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/bpm_range%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");
}

#[tokio::test]
async fn test_get_all_keys() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Cleanup first
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/keys_%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");

    // Create files with different keys
    for key in &["C", "Dm", "Em"] {
        let file_id = MidiFileBuilder::new()
            .with_path(&format!("/test/keys_{}.mid", key))
            .insert(pool)
            .await;

        MetadataBuilder::new(file_id).with_key(key).insert(pool).await;
    }

    // Get all keys
    let keys: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT DISTINCT key_signature::text
        FROM musical_metadata
        WHERE key_signature IS NOT NULL
        ORDER BY key_signature ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .expect("Query failed");

    let key_names: Vec<String> = keys.into_iter().map(|(k,)| k).collect();
    assert!(key_names.contains(&"C".to_string()), "Should include C");
    assert!(key_names.contains(&"Dm".to_string()), "Should include Dm");
    assert!(key_names.contains(&"Em".to_string()), "Should include Em");

    // Cleanup
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/keys_%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");
}

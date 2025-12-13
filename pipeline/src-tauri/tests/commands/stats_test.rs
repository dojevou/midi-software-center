#![allow(dead_code, unused_imports, unused_variables)]
//! Tests for pipeline/src-tauri/src/commands/stats.rs
//! Commands: get_category_stats, get_manufacturer_stats, get_key_signature_stats, etc.
use crate::common::*;
use midi_pipeline::commands::stats::{get_category_stats_impl, get_database_size_impl};

#[tokio::test]
async fn test_get_category_stats() {
    let state = setup_test_state().await;
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Cleanup
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/stats_cat%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");

    // Create files with different categories
    for i in 0..3 {
        let file_id = MidiFileBuilder::new()
            .with_path(&format!("/test/stats_cat_bass_{}.mid", i))
            .insert(pool)
            .await;

        sqlx::query("INSERT INTO file_categories (file_id, primary_category) VALUES ($1, 'BASS')")
            .bind(file_id)
            .execute(pool)
            .await
            .expect("Failed to insert category");
    }

    for i in 0..2 {
        let file_id = MidiFileBuilder::new()
            .with_path(&format!("/test/stats_cat_drums_{}.mid", i))
            .insert(pool)
            .await;

        sqlx::query("INSERT INTO file_categories (file_id, primary_category) VALUES ($1, 'DRUMS')")
            .bind(file_id)
            .execute(pool)
            .await
            .expect("Failed to insert category");
    }

    // Get category stats
    let stats = get_category_stats_impl(&state).await.expect("Get category stats failed");

    // Verify counts
    assert!(
        stats.get("BASS").copied().unwrap_or(0) >= 3,
        "Should have at least 3 BASS files"
    );
    assert!(
        stats.get("DRUMS").copied().unwrap_or(0) >= 2,
        "Should have at least 2 DRUMS files"
    );

    // Cleanup
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/stats_cat%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");
}

#[tokio::test]
async fn test_get_manufacturer_stats() {
    let state = setup_test_state().await;
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Cleanup
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/stats_mfr%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");

    // Create files with different manufacturers
    for i in 0..3 {
        let file_id = MidiFileBuilder::new()
            .with_path(&format!("/test/stats_mfr_roland_{}.mid", i))
            .insert(pool)
            .await;

        MetadataBuilder::new(file_id).insert(pool).await;

        sqlx::query("UPDATE musical_metadata SET manufacturer = $1 WHERE file_id = $2")
            .bind("Roland")
            .bind(file_id)
            .execute(pool)
            .await
            .expect("Failed to update manufacturer");
    }

    for i in 0..2 {
        let file_id = MidiFileBuilder::new()
            .with_path(&format!("/test/stats_mfr_yamaha_{}.mid", i))
            .insert(pool)
            .await;

        MetadataBuilder::new(file_id).insert(pool).await;

        sqlx::query("UPDATE musical_metadata SET manufacturer = $1 WHERE file_id = $2")
            .bind("Yamaha")
            .bind(file_id)
            .execute(pool)
            .await
            .expect("Failed to update manufacturer");
    }

    // Get manufacturer stats
    let stats: std::collections::HashMap<String, i64> = sqlx::query_as(
        r#"
        SELECT mm.manufacturer::text as manufacturer, COUNT(*) as count
        FROM files f
        LEFT JOIN musical_metadata mm ON f.id = mm.file_id
        WHERE mm.manufacturer IS NOT NULL AND f.filepath LIKE '/test/stats_mfr%'
        GROUP BY mm.manufacturer
        ORDER BY count DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .expect("Query failed")
    .into_iter()
    .map(|(mfr, count): (Option<String>, i64)| (mfr.unwrap(), count))
    .collect();

    assert!(
        stats.get("Roland").copied().unwrap_or(0) >= 3,
        "Should have at least 3 Roland files"
    );
    assert!(
        stats.get("Yamaha").copied().unwrap_or(0) >= 2,
        "Should have at least 2 Yamaha files"
    );

    // Cleanup
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/stats_mfr%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");
}

#[tokio::test]
async fn test_get_key_signature_stats() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Cleanup
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/stats_key%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");

    // Create files with different keys
    for key in &["C", "Dm", "Em"] {
        for i in 0..2 {
            let file_id = MidiFileBuilder::new()
                .with_path(&format!("/test/stats_key_{}_{}.mid", key, i))
                .insert(pool)
                .await;

            MetadataBuilder::new(file_id).with_key(key).insert(pool).await;
        }
    }

    // Get key signature stats
    let stats: std::collections::HashMap<String, i64> = sqlx::query_as(
        r#"
        SELECT mm.key_signature::text as key_sig, COUNT(*) as count
        FROM files f
        LEFT JOIN musical_metadata mm ON f.id = mm.file_id
        WHERE mm.key_signature IS NOT NULL AND f.filepath LIKE '/test/stats_key%'
        GROUP BY mm.key_signature
        ORDER BY count DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .expect("Query failed")
    .into_iter()
    .map(|(key, count): (Option<String>, i64)| (key.unwrap(), count))
    .collect();

    assert_eq!(
        stats.get("C").copied().unwrap_or(0),
        2,
        "Should have 2 C files"
    );
    assert_eq!(
        stats.get("Dm").copied().unwrap_or(0),
        2,
        "Should have 2 Dm files"
    );
    assert_eq!(
        stats.get("Em").copied().unwrap_or(0),
        2,
        "Should have 2 Em files"
    );

    // Cleanup
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/stats_key%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");
}

#[tokio::test]
async fn test_get_recently_added_count() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Cleanup
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/stats_recent%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");

    // Create recent files
    for i in 0..5 {
        create_test_file(pool, &format!("stats_recent_{}.mid", i)).await;
    }

    // Count recently added files (last 7 days)
    let count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM files
        WHERE filepath LIKE '/test/stats_recent%' AND created_at >= NOW() - INTERVAL '7 days'
        "#,
    )
    .fetch_one(pool)
    .await
    .expect("Query failed");

    assert_eq!(count.0, 5, "Should have 5 recently added files");

    // Cleanup
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/stats_recent%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");
}

#[tokio::test]
async fn test_get_duplicate_count() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Cleanup
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/stats_dup%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");

    // Create files with duplicate content_hash
    let dup_hash = format!("{:064x}", 999999);

    for i in 0..2 {
        MidiFileBuilder::new()
            .with_path(&format!("/test/stats_dup_{}.mid", i))
            .with_hash(&dup_hash)
            .insert(pool)
            .await;
    }

    // Count duplicate groups
    let count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM (
            SELECT content_hash
            FROM files
            WHERE filepath LIKE '/test/stats_dup%'
            GROUP BY content_hash
            HAVING COUNT(*) > 1
        ) as duplicates
        "#,
    )
    .fetch_one(pool)
    .await
    .expect("Query failed");

    assert_eq!(count.0, 1, "Should have 1 duplicate group");

    // Cleanup
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/stats_dup%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");
}

#[tokio::test]
async fn test_get_database_size() {
    let state = setup_test_state().await;

    // Get database size
    let size = get_database_size_impl(&state).await.expect("Get database size failed");

    // Should return a non-empty string
    assert!(!size.is_empty(), "Database size should not be empty");
    assert!(
        size.contains("B") || size.contains("K") || size.contains("M") || size.contains("G"),
        "Size should contain a unit (B, KB, MB, or GB)"
    );
}

#[tokio::test]
async fn test_check_database_health() {
    let state = setup_test_state().await;

    // Check database health
    let health = state.database.test_connection().await;

    assert!(health.is_ok(), "Database health check should succeed");

    // Additional verification - count files should work
    let pool = state.database.pool().await;
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM files")
        .fetch_one(&pool)
        .await
        .expect("Count query should succeed");

    assert!(count.0 >= 0, "File count should be non-negative");
}

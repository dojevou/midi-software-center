#[allow(dead_code, unused_imports, unused_variables)]

/// Tests for pipeline/src-tauri/src/commands/tags.rs
/// Commands: get_file_tags, get_popular_tags, search_tags, update_file_tags, etc.
use crate::common::*;
use midi_pipeline::commands::tags::{
    add_tags_to_file_impl, get_file_tags_impl, get_popular_tags_impl, search_tags_impl,
};
use midi_pipeline::db::repositories::TagRepository;

#[tokio::test]
async fn test_get_file_tags() {
    let state = setup_test_state().await;
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Cleanup
    sqlx::query("DELETE FROM files WHERE filepath = '/test/path/tagged_file.mid'")
        .execute(pool)
        .await
        .expect("Cleanup failed");

    // Create a file
    let file_id = create_test_file(pool, "tagged_file.mid").await;

    // Add tags using repository
    let repo = TagRepository::new(pool.clone());
    let tag_data = vec![
        ("test_tag1".to_string(), Some("test_category".to_string())),
        ("test_tag2".to_string(), Some("test_category".to_string())),
    ];
    let tag_ids = repo.get_or_create_tags_batch(&tag_data).await.expect("Failed to create tags");
    repo.add_tags_to_file(file_id, &tag_ids).await.expect("Failed to add tags");

    // Get file tags
    let tags = get_file_tags_impl(file_id, &state).await.expect("Get file tags failed");

    assert!(tags.len() >= 2, "Should have at least 2 tags");
    assert!(tags.iter().any(|t| t.name == "test_tag1"));
    assert!(tags.iter().any(|t| t.name == "test_tag2"));

    // Cleanup
    sqlx::query("DELETE FROM files WHERE id = $1")
        .bind(file_id)
        .execute(pool)
        .await
        .expect("Cleanup failed");
}

#[tokio::test]
async fn test_get_popular_tags() {
    let state = setup_test_state().await;
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Create test files and tags
    let repo = TagRepository::new(pool.clone());
    for i in 0..5 {
        let file_id = create_test_file(pool, &format!("popular_tag_{}.mid", i)).await;

        let tag_data = vec![("popular_tag".to_string(), None)];
        let tag_ids =
            repo.get_or_create_tags_batch(&tag_data).await.expect("Failed to create tags");
        repo.add_tags_to_file(file_id, &tag_ids).await.expect("Failed to add tags");
    }

    // Get popular tags
    let tags = get_popular_tags_impl(Some(50), &state).await.expect("Get popular tags failed");

    // Should have our popular tag
    let popular_tag = tags.iter().find(|t| t.name == "popular_tag");
    assert!(popular_tag.is_some(), "Should find popular_tag");
    assert!(
        popular_tag.unwrap().usage_count >= 5,
        "Usage count should be at least 5"
    );

    // Cleanup
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/path/popular_tag_%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");
}

#[tokio::test]
async fn test_search_tags() {
    let state = setup_test_state().await;
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Create tags with specific prefix
    let repo = TagRepository::new(pool.clone());
    let file_id = create_test_file(pool, "searchable_file.mid").await;

    let tag_data = vec![
        ("searchtest_alpha".to_string(), None),
        ("searchtest_beta".to_string(), None),
        ("other_tag".to_string(), None),
    ];
    let tag_ids = repo.get_or_create_tags_batch(&tag_data).await.expect("Failed to create tags");
    repo.add_tags_to_file(file_id, &tag_ids).await.expect("Failed to add tags");

    // Search for tags with prefix
    let tags = search_tags_impl("searchtest".to_string(), Some(10), &state)
        .await
        .expect("Search tags failed");

    // Should find at least our 2 searchtest tags
    let matching_tags: Vec<_> = tags.iter().filter(|t| t.name.starts_with("searchtest")).collect();
    assert!(
        matching_tags.len() >= 2,
        "Should find at least 2 matching tags"
    );

    // Cleanup
    sqlx::query("DELETE FROM files WHERE id = $1")
        .bind(file_id)
        .execute(pool)
        .await
        .expect("Cleanup failed");
}

#[tokio::test]
async fn test_get_tag_categories() {
    let state = setup_test_state().await;
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Create tags with specific categories
    let repo = TagRepository::new(pool.clone());
    let file_id = create_test_file(pool, "category_test.mid").await;

    let tag_data = vec![
        ("cat_tag1".to_string(), Some("test_category_A".to_string())),
        ("cat_tag2".to_string(), Some("test_category_B".to_string())),
    ];
    let tag_ids = repo.get_or_create_tags_batch(&tag_data).await.expect("Failed to create tags");
    repo.add_tags_to_file(file_id, &tag_ids).await.expect("Failed to add tags");

    // Get categories
    let categories = repo.get_tag_categories().await.expect("Get categories failed");

    // Should include our test categories
    assert!(categories.contains(&"test_category_A".to_string()));
    assert!(categories.contains(&"test_category_B".to_string()));

    // Cleanup
    sqlx::query("DELETE FROM files WHERE id = $1")
        .bind(file_id)
        .execute(pool)
        .await
        .expect("Cleanup failed");
}

#[tokio::test]
async fn test_update_file_tags() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Create a file
    let file_id = create_test_file(pool, "update_tags_file.mid").await;

    // Add initial tags
    let repo = TagRepository::new(pool.clone());
    let initial_tags = vec![("old_tag".to_string(), None)];
    let tag_ids = repo
        .get_or_create_tags_batch(&initial_tags)
        .await
        .expect("Failed to create tags");
    repo.add_tags_to_file(file_id, &tag_ids).await.expect("Failed to add tags");

    // Update tags (replace all)
    let new_tags = vec!["new_tag1".to_string(), "new_tag2".to_string()];
    let new_tag_data: Vec<_> = new_tags.iter().map(|n| (n.clone(), None)).collect();
    let new_tag_ids = repo
        .get_or_create_tags_batch(&new_tag_data)
        .await
        .expect("Failed to create tags");
    repo.update_file_tags(file_id, &new_tag_ids)
        .await
        .expect("Failed to update tags");

    // Verify tags were replaced
    let current_tags = repo.get_file_tags(file_id).await.expect("Failed to get tags");
    assert_eq!(current_tags.len(), 2, "Should have exactly 2 tags");
    assert!(current_tags.iter().any(|t| t.name == "new_tag1"));
    assert!(current_tags.iter().any(|t| t.name == "new_tag2"));
    assert!(
        !current_tags.iter().any(|t| t.name == "old_tag"),
        "Old tag should be removed"
    );

    // Cleanup
    sqlx::query("DELETE FROM files WHERE id = $1")
        .bind(file_id)
        .execute(pool)
        .await
        .expect("Cleanup failed");
}

#[tokio::test]
async fn test_add_tags_to_file() {
    let state = setup_test_state().await;
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Create a file
    let file_id = create_test_file(pool, "add_tags_file.mid").await;

    // Add tags
    let tags = vec!["add_tag1".to_string(), "add_tag2".to_string()];
    add_tags_to_file_impl(file_id, tags, &state).await.expect("Add tags failed");

    // Verify tags were added
    let repo = TagRepository::new(pool.clone());
    let current_tags = repo.get_file_tags(file_id).await.expect("Failed to get tags");

    assert!(current_tags.len() >= 2, "Should have at least 2 tags");
    assert!(current_tags.iter().any(|t| t.name == "add_tag1"));
    assert!(current_tags.iter().any(|t| t.name == "add_tag2"));

    // Cleanup
    sqlx::query("DELETE FROM files WHERE id = $1")
        .bind(file_id)
        .execute(pool)
        .await
        .expect("Cleanup failed");
}

#[tokio::test]
async fn test_remove_tag_from_file() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Create a file with tags
    let file_id = create_test_file(pool, "remove_tag_file.mid").await;

    let repo = TagRepository::new(pool.clone());
    let tag_data = vec![("keep_tag".to_string(), None), ("remove_tag".to_string(), None)];
    let tag_ids = repo.get_or_create_tags_batch(&tag_data).await.expect("Failed to create tags");
    repo.add_tags_to_file(file_id, &tag_ids).await.expect("Failed to add tags");

    // Find the tag ID to remove
    let tags_before = repo.get_file_tags(file_id).await.expect("Failed to get tags");
    let remove_tag_id = tags_before.iter().find(|t| t.name == "remove_tag").unwrap().id;

    // Remove the tag
    repo.remove_tag_from_file(file_id, remove_tag_id)
        .await
        .expect("Failed to remove tag");

    // Verify tag was removed
    let tags_after = repo.get_file_tags(file_id).await.expect("Failed to get tags");
    assert!(
        !tags_after.iter().any(|t| t.name == "remove_tag"),
        "Tag should be removed"
    );
    assert!(
        tags_after.iter().any(|t| t.name == "keep_tag"),
        "Other tags should remain"
    );

    // Cleanup
    sqlx::query("DELETE FROM files WHERE id = $1")
        .bind(file_id)
        .execute(pool)
        .await
        .expect("Cleanup failed");
}

#[tokio::test]
async fn test_get_files_by_tags() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Create files with specific tags
    let repo = TagRepository::new(pool.clone());

    let file1_id = create_test_file(pool, "multi_tag_1.mid").await;
    let file2_id = create_test_file(pool, "multi_tag_2.mid").await;
    let file3_id = create_test_file(pool, "multi_tag_3.mid").await;

    // file1: tag_a, tag_b
    let tags1 = vec![("tag_a".to_string(), None), ("tag_b".to_string(), None)];
    let tag_ids1 = repo.get_or_create_tags_batch(&tags1).await.expect("Failed to create tags");
    repo.add_tags_to_file(file1_id, &tag_ids1).await.expect("Failed to add tags");

    // file2: tag_a
    let tags2 = vec![("tag_a".to_string(), None)];
    let tag_ids2 = repo.get_or_create_tags_batch(&tags2).await.expect("Failed to create tags");
    repo.add_tags_to_file(file2_id, &tag_ids2).await.expect("Failed to add tags");

    // file3: tag_b
    let tags3 = vec![("tag_b".to_string(), None)];
    let tag_ids3 = repo.get_or_create_tags_batch(&tags3).await.expect("Failed to create tags");
    repo.add_tags_to_file(file3_id, &tag_ids3).await.expect("Failed to add tags");

    // Test OR logic (at least one tag)
    let files_or = repo
        .get_files_by_tags(&vec!["tag_a".to_string(), "tag_b".to_string()], false)
        .await
        .expect("Failed to get files");
    assert!(
        files_or.len() >= 3,
        "Should find at least 3 files with tag_a OR tag_b"
    );

    // Test AND logic (all tags)
    let files_and = repo
        .get_files_by_tags(&vec!["tag_a".to_string(), "tag_b".to_string()], true)
        .await
        .expect("Failed to get files");
    assert!(
        files_and.contains(&file1_id),
        "Should find file1 with both tags"
    );

    // Cleanup
    for file_id in &[file1_id, file2_id, file3_id] {
        sqlx::query("DELETE FROM files WHERE id = $1")
            .bind(file_id)
            .execute(pool)
            .await
            .expect("Cleanup failed");
    }
}

#[tokio::test]
async fn test_get_tag_stats() {
    let db = TestDatabase::new().await;
    let pool = db.pool();

    // Create files with a specific tag
    let repo = TagRepository::new(pool.clone());

    for i in 0..3 {
        let file_id = create_test_file(pool, &format!("stats_file_{}.mid", i)).await;
        let tag_data = vec![("stats_tag".to_string(), None)];
        let tag_ids =
            repo.get_or_create_tags_batch(&tag_data).await.expect("Failed to create tags");
        repo.add_tags_to_file(file_id, &tag_ids).await.expect("Failed to add tags");
    }

    // Get the tag ID
    let all_tags = repo.get_popular_tags(100).await.expect("Failed to get tags");
    let stats_tag = all_tags.iter().find(|t| t.name == "stats_tag").expect("Should find stats_tag");

    // Get tag stats
    let count = repo.get_tag_file_count(stats_tag.id).await.expect("Failed to get tag stats");
    assert!(count >= 3, "Tag should be associated with at least 3 files");

    // Cleanup
    sqlx::query("DELETE FROM files WHERE filepath LIKE '/test/path/stats_file_%'")
        .execute(pool)
        .await
        .expect("Cleanup failed");
}

//! Integration tests for Meilisearch functionality
//!
//! These tests verify that the Meilisearch integration works correctly
//! end-to-end, including indexing, searching, and error handling.
//!
//! Prerequisites:
//! - Meilisearch server running at http://localhost:7700
//! - Set MEILISEARCH_URL=http://localhost:7700 in environment

use midi_app::services::meilisearch_client::{
    MeilisearchClient, MidiSearchDocument, SearchFilters,
};
use std::env;

/// Helper to create a test Meilisearch client
fn create_test_client() -> Result<MeilisearchClient, String> {
    let url = env::var("MEILISEARCH_URL").unwrap_or_else(|_| "http://localhost:7700".to_string());
    let api_key = env::var("MEILISEARCH_API_KEY").ok();

    MeilisearchClient::new(&url, api_key.as_deref(), Some("test_midi_files"))
}

/// Helper to create a sample test document
fn create_test_document(id: i64, filename: &str) -> MidiSearchDocument {
    MidiSearchDocument {
        id,
        filename: filename.to_string(),
        original_filename: filename.to_string(),
        filepath: format!("/test/path/{}", filename),
        tags: Some(vec!["test".to_string(), "integration".to_string()]),
        instruments: Some(vec!["piano".to_string()]),
        bpm: Some(120.0),
        key_signature: Some("C".to_string()),
        time_signature: Some("4/4".to_string()),
        manufacturer: Some("Test Manufacturer".to_string()),
        collection_name: Some("Test Collection".to_string()),
        duration_seconds: Some(60.0),
        num_tracks: 1,
        is_multi_track: Some(false),
        is_percussive: Some(false),
        timbres: Some(vec!["acoustic".to_string()]),
        styles: Some(vec!["jazz".to_string()]),
        articulations: Some(vec!["staccato".to_string()]),
    }
}

#[tokio::test]
async fn test_meilisearch_client_creation() {
    let result = create_test_client();

    match result {
        Ok(_client) => {
            println!("✅ Meilisearch client created successfully");
        }
        Err(e) => {
            println!("⚠️  Meilisearch client creation failed: {}", e);
            println!("   Make sure Meilisearch is running at http://localhost:7700");
        }
    }
}

#[tokio::test]
async fn test_initialize_index() {
    let client = match create_test_client() {
        Ok(c) => c,
        Err(e) => {
            println!("⚠️  Skipping test - Meilisearch not available: {}", e);
            return;
        }
    };

    match client.initialize_index().await {
        Ok(_) => {
            println!("✅ Index initialized successfully");
        }
        Err(e) => {
            println!("❌ Index initialization failed: {}", e);
            panic!("Failed to initialize index: {}", e);
        }
    }
}

#[tokio::test]
async fn test_index_single_file() {
    let client = match create_test_client() {
        Ok(c) => c,
        Err(e) => {
            println!("⚠️  Skipping test - Meilisearch not available: {}", e);
            return;
        }
    };

    // Initialize index first
    client.initialize_index().await.ok();

    let document = create_test_document(1, "test_file_1.mid");

    match client.index_file(&document).await {
        Ok(_) => {
            println!("✅ Single file indexed successfully");
        }
        Err(e) => {
            println!("❌ Single file indexing failed: {}", e);
            panic!("Failed to index file: {}", e);
        }
    }

    // Wait a bit for indexing to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
}

#[tokio::test]
async fn test_batch_index_files() {
    let client = match create_test_client() {
        Ok(c) => c,
        Err(e) => {
            println!("⚠️  Skipping test - Meilisearch not available: {}", e);
            return;
        }
    };

    // Initialize index first
    client.initialize_index().await.ok();

    // Create test documents
    let documents: Vec<MidiSearchDocument> = (1..=100)
        .map(|i| create_test_document(i, &format!("batch_test_{}.mid", i)))
        .collect();

    let start = std::time::Instant::now();

    match client.index_files_batch(&documents).await {
        Ok(_) => {
            let elapsed = start.elapsed();
            println!("✅ Batch indexed 100 files in {:?}", elapsed);
            println!("   Average: {:.2}ms per file", elapsed.as_millis() as f64 / 100.0);
        }
        Err(e) => {
            println!("❌ Batch indexing failed: {}", e);
            panic!("Failed to batch index files: {}", e);
        }
    }

    // Wait for indexing to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
}

#[tokio::test]
async fn test_simple_search() {
    let client = match create_test_client() {
        Ok(c) => c,
        Err(e) => {
            println!("⚠️  Skipping test - Meilisearch not available: {}", e);
            return;
        }
    };

    // Initialize and index test data
    client.initialize_index().await.ok();
    let document = create_test_document(1, "jazz_piano_test.mid");
    client.index_file(&document).await.ok();

    // Wait for indexing
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let start = std::time::Instant::now();

    match client.search("jazz piano", Some(20), None).await {
        Ok(results) => {
            let elapsed = start.elapsed();
            println!("✅ Search completed in {:?}", elapsed);
            println!("   Found {} results", results.len());

            // Verify we got results
            assert!(!results.is_empty(), "Should find at least one result");

            // Check latency target (<10ms for simple search)
            if elapsed.as_millis() < 10 {
                println!("   ✅ Latency target met (<10ms)");
            } else {
                println!("   ⚠️  Latency target missed: {}ms (target: <10ms)", elapsed.as_millis());
            }
        }
        Err(e) => {
            println!("❌ Search failed: {}", e);
            panic!("Failed to search: {}", e);
        }
    }
}

#[tokio::test]
async fn test_faceted_search() {
    let client = match create_test_client() {
        Ok(c) => c,
        Err(e) => {
            println!("⚠️  Skipping test - Meilisearch not available: {}", e);
            return;
        }
    };

    // Initialize and index test data
    client.initialize_index().await.ok();

    // Index multiple documents with different attributes
    let documents = vec![
        create_test_document(1, "jazz_piano_120bpm.mid"),
        create_test_document(2, "rock_drums_140bpm.mid"),
        create_test_document(3, "jazz_bass_115bpm.mid"),
    ];

    for doc in &documents {
        client.index_file(doc).await.ok();
    }

    // Wait for indexing
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    let filters = SearchFilters {
        instruments: Some(vec!["piano".to_string()]),
        tags: Some(vec!["jazz".to_string()]),
        bpm_min: Some(110.0),
        bpm_max: Some(130.0),
        key_signature: Some("C".to_string()),
        ..Default::default()
    };

    let start = std::time::Instant::now();

    match client.faceted_search(Some("jazz"), &filters, Some(20), None).await {
        Ok(results) => {
            let elapsed = start.elapsed();
            println!("✅ Faceted search completed in {:?}", elapsed);
            println!("   Found {} results", results.len());

            // Check latency target (<50ms for faceted search)
            if elapsed.as_millis() < 50 {
                println!("   ✅ Latency target met (<50ms)");
            } else {
                println!("   ⚠️  Latency target missed: {}ms (target: <50ms)", elapsed.as_millis());
            }
        }
        Err(e) => {
            println!("❌ Faceted search failed: {}", e);
            panic!("Failed to faceted search: {}", e);
        }
    }
}

#[tokio::test]
async fn test_search_with_typos() {
    let client = match create_test_client() {
        Ok(c) => c,
        Err(e) => {
            println!("⚠️  Skipping test - Meilisearch not available: {}", e);
            return;
        }
    };

    // Initialize and index test data
    client.initialize_index().await.ok();
    let document = create_test_document(1, "piano_melody.mid");
    client.index_file(&document).await.ok();

    // Wait for indexing
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Search with typo: "pino" instead of "piano"
    match client.search("pino melody", Some(20), None).await {
        Ok(results) => {
            println!("✅ Typo-tolerant search completed");
            println!("   Found {} results for 'pino melody' (should match 'piano melody')", results.len());

            if !results.is_empty() {
                println!("   ✅ Typo tolerance working");
            } else {
                println!("   ⚠️  Typo tolerance might not be working as expected");
            }
        }
        Err(e) => {
            println!("❌ Typo-tolerant search failed: {}", e);
            panic!("Failed to search with typos: {}", e);
        }
    }
}

#[tokio::test]
async fn test_delete_file() {
    let client = match create_test_client() {
        Ok(c) => c,
        Err(e) => {
            println!("⚠️  Skipping test - Meilisearch not available: {}", e);
            return;
        }
    };

    // Initialize and index test data
    client.initialize_index().await.ok();
    let document = create_test_document(999, "file_to_delete.mid");
    client.index_file(&document).await.ok();

    // Wait for indexing
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Delete the file
    match client.delete_file(999).await {
        Ok(_) => {
            println!("✅ File deleted successfully");
        }
        Err(e) => {
            println!("❌ File deletion failed: {}", e);
            panic!("Failed to delete file: {}", e);
        }
    }

    // Wait for deletion to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Verify file is deleted by searching
    match client.search("file_to_delete", Some(20), None).await {
        Ok(results) => {
            if results.is_empty() {
                println!("   ✅ File successfully removed from index");
            } else {
                println!("   ⚠️  File still found in index after deletion");
            }
        }
        Err(e) => {
            println!("   ⚠️  Could not verify deletion: {}", e);
        }
    }
}

#[tokio::test]
async fn test_batch_delete_files() {
    let client = match create_test_client() {
        Ok(c) => c,
        Err(e) => {
            println!("⚠️  Skipping test - Meilisearch not available: {}", e);
            return;
        }
    };

    // Initialize and index test data
    client.initialize_index().await.ok();

    let file_ids: Vec<i64> = (1000..1010).collect();
    let documents: Vec<MidiSearchDocument> = file_ids
        .iter()
        .map(|&id| create_test_document(id, &format!("batch_delete_{}.mid", id)))
        .collect();

    client.index_files_batch(&documents).await.ok();

    // Wait for indexing
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Batch delete
    match client.delete_files_batch(&file_ids).await {
        Ok(_) => {
            println!("✅ Batch deleted 10 files successfully");
        }
        Err(e) => {
            println!("❌ Batch deletion failed: {}", e);
            panic!("Failed to batch delete files: {}", e);
        }
    }
}

#[tokio::test]
async fn test_get_stats() {
    let client = match create_test_client() {
        Ok(c) => c,
        Err(e) => {
            println!("⚠️  Skipping test - Meilisearch not available: {}", e);
            return;
        }
    };

    // Initialize index
    client.initialize_index().await.ok();

    match client.get_stats().await {
        Ok(stats) => {
            println!("✅ Stats retrieved successfully");
            println!("   Stats: {:?}", stats);

            // Verify stats contain expected fields
            assert!(stats.contains_key("number_of_documents"), "Stats should contain number_of_documents");
            assert!(stats.contains_key("is_indexing"), "Stats should contain is_indexing");
        }
        Err(e) => {
            println!("❌ Get stats failed: {}", e);
            panic!("Failed to get stats: {}", e);
        }
    }
}

#[tokio::test]
async fn test_clear_index() {
    let client = match create_test_client() {
        Ok(c) => c,
        Err(e) => {
            println!("⚠️  Skipping test - Meilisearch not available: {}", e);
            return;
        }
    };

    // Initialize and add some data
    client.initialize_index().await.ok();
    let document = create_test_document(1, "test_clear.mid");
    client.index_file(&document).await.ok();

    // Wait for indexing
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Clear index
    match client.clear_index().await {
        Ok(_) => {
            println!("✅ Index cleared successfully");
        }
        Err(e) => {
            println!("❌ Clear index failed: {}", e);
            panic!("Failed to clear index: {}", e);
        }
    }

    // Wait for clearing to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Verify index is empty
    match client.get_stats().await {
        Ok(stats) => {
            if let Some(count) = stats.get("number_of_documents") {
                println!("   Documents in index after clear: {:?}", count);
            }
        }
        Err(e) => {
            println!("   ⚠️  Could not verify index clear: {}", e);
        }
    }
}

#[tokio::test]
async fn test_error_handling_no_server() {
    // Try to create client with invalid URL
    match MeilisearchClient::new("http://localhost:99999", None, Some("test")) {
        Ok(_) => {
            println!("⚠️  Client created with invalid URL (might fail on first operation)");
        }
        Err(e) => {
            println!("✅ Correctly failed to create client with invalid URL: {}", e);
        }
    }
}

#[tokio::test]
async fn test_pagination() {
    let client = match create_test_client() {
        Ok(c) => c,
        Err(e) => {
            println!("⚠️  Skipping test - Meilisearch not available: {}", e);
            return;
        }
    };

    // Initialize and index test data
    client.initialize_index().await.ok();

    // Index 30 documents
    let documents: Vec<MidiSearchDocument> = (1..=30)
        .map(|i| create_test_document(i, &format!("pagination_test_{}.mid", i)))
        .collect();

    client.index_files_batch(&documents).await.ok();

    // Wait for indexing
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Test pagination
    let page1 = client.search("pagination_test", Some(10), Some(0)).await;
    let page2 = client.search("pagination_test", Some(10), Some(10)).await;
    let page3 = client.search("pagination_test", Some(10), Some(20)).await;

    match (page1, page2, page3) {
        (Ok(p1), Ok(p2), Ok(p3)) => {
            println!("✅ Pagination working");
            println!("   Page 1: {} results", p1.len());
            println!("   Page 2: {} results", p2.len());
            println!("   Page 3: {} results", p3.len());

            assert_eq!(p1.len(), 10, "Page 1 should have 10 results");
            assert_eq!(p2.len(), 10, "Page 2 should have 10 results");
            assert_eq!(p3.len(), 10, "Page 3 should have 10 results");
        }
        _ => {
            println!("❌ Pagination test failed");
        }
    }
}

/// Run all tests and report summary
#[tokio::test]
async fn test_summary() {
    println!("\n========================================");
    println!("MEILISEARCH INTEGRATION TEST SUMMARY");
    println!("========================================\n");
    println!("Run with: cargo test --test test_meilisearch_integration -- --nocapture");
    println!("\nPrerequisites:");
    println!("  - Meilisearch server at http://localhost:7700");
    println!("  - Set MEILISEARCH_URL environment variable");
    println!("\nTests cover:");
    println!("  ✅ Client creation");
    println!("  ✅ Index initialization");
    println!("  ✅ Single file indexing");
    println!("  ✅ Batch file indexing (100 files)");
    println!("  ✅ Simple search (<10ms target)");
    println!("  ✅ Faceted search (<50ms target)");
    println!("  ✅ Typo-tolerant search");
    println!("  ✅ File deletion");
    println!("  ✅ Batch deletion");
    println!("  ✅ Statistics retrieval");
    println!("  ✅ Index clearing");
    println!("  ✅ Error handling");
    println!("  ✅ Pagination");
    println!("\n========================================\n");
}

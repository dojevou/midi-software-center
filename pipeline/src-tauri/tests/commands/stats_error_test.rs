/// Stats Command Error Path Tests
/// Tests error handling, edge cases, and boundary conditions for stats functionality
use midi_pipeline::commands::stats::{get_category_stats_impl, get_database_size_impl};
use midi_pipeline::{database::Database, AppState};

#[tokio::test]
async fn test_category_stats_empty_database() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    // This tests behavior with potentially zero categorized files
    let result = get_category_stats_impl(&state).await;

    // Should succeed even with empty database
    assert!(result.is_ok());
    let stats = result.unwrap();

    // Empty database should return a HashMap (possibly empty)
    // The HashMap maps category names to counts
    for count in stats.values() {
        assert!(*count >= 0, "Category count should be non-negative");
    }
}

#[tokio::test]
async fn test_database_size_query() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    let result = get_database_size_impl(&state).await;

    assert!(result.is_ok());
    let size = result.unwrap();

    // Size should be a human-readable string like "125.4 MB" or "Unknown"
    assert!(!size.is_empty(), "Database size should not be empty");
}

#[tokio::test]
async fn test_category_stats_concurrent_requests() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    // Send 10 concurrent stats requests
    let mut handles = vec![];
    for _ in 0..10 {
        let state_clone = AppState { database: state.database.clone() };
        let handle = tokio::spawn(async move { get_category_stats_impl(&state_clone).await });
        handles.push(handle);
    }

    // All should succeed
    for handle in handles {
        let result = handle.await.expect("Task panicked");
        assert!(result.is_ok(), "Concurrent stats request failed");
    }
}

#[tokio::test]
async fn test_category_stats_consistency() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    // Get stats twice
    let result1 = get_category_stats_impl(&state).await;
    let result2 = get_category_stats_impl(&state).await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    let stats1 = result1.unwrap();
    let stats2 = result2.unwrap();

    // Without intervening writes, stats should be identical
    // (unless there are concurrent imports happening)
    assert_eq!(
        stats1.len(),
        stats2.len(),
        "Category count should be consistent"
    );
}

#[tokio::test]
async fn test_database_size_format() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    let result = get_database_size_impl(&state).await;

    assert!(result.is_ok());
    let size = result.unwrap();

    // PostgreSQL pg_size_pretty returns formats like:
    // "8192 bytes", "125 kB", "1234 MB", "1 GB", etc.
    // Should contain at least one character
    assert!(!size.is_empty());

    // Should not be an error message (should contain size units or "Unknown")
    let is_valid_format = size == "Unknown"
        || size.contains("bytes")
        || size.contains("kB")
        || size.contains("MB")
        || size.contains("GB")
        || size.contains("TB");

    assert!(
        is_valid_format,
        "Database size '{}' should be in valid format",
        size
    );
}

/// Search Command Error Path Tests
/// Tests error handling, edge cases, and boundary conditions for search functionality
use midi_pipeline::commands::search::{search_files_impl, SearchFilters};
use midi_pipeline::{database::Database, AppState};

// Note: SearchResults has fields: items, total_count, page, page_size, total_pages

#[tokio::test]
async fn test_search_empty_query() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    let result = search_files_impl(
        "".to_string(),
        SearchFilters { category: None, min_bpm: None, max_bpm: None, key_signature: None },
        1,
        20,
        &state,
    )
    .await;

    // Empty query should still work (returns all files)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_search_no_results() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    let result = search_files_impl(
        "NONEXISTENT_FILE_XYZABC123".to_string(),
        SearchFilters { category: None, min_bpm: None, max_bpm: None, key_signature: None },
        1,
        20,
        &state,
    )
    .await;

    assert!(result.is_ok());
    let results = result.unwrap();
    assert_eq!(results.items.len(), 0);
    assert_eq!(results.total_count, 0);
}

#[tokio::test]
async fn test_search_negative_page() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    let result = search_files_impl(
        "test".to_string(),
        SearchFilters { category: None, min_bpm: None, max_bpm: None, key_signature: None },
        -1, // Invalid page
        20,
        &state,
    )
    .await;

    // Should handle gracefully or error
    assert!(result.is_err() || result.unwrap().page >= 1);
}

#[tokio::test]
async fn test_search_zero_page() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    let result = search_files_impl(
        "test".to_string(),
        SearchFilters { category: None, min_bpm: None, max_bpm: None, key_signature: None },
        0, // Invalid page
        20,
        &state,
    )
    .await;

    assert!(result.is_err() || result.unwrap().page >= 1);
}

#[tokio::test]
async fn test_search_negative_page_size() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    let result = search_files_impl(
        "test".to_string(),
        SearchFilters { category: None, min_bpm: None, max_bpm: None, key_signature: None },
        1,
        -10, // Invalid page size
        &state,
    )
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_search_excessive_page_size() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    let result = search_files_impl(
        "test".to_string(),
        SearchFilters { category: None, min_bpm: None, max_bpm: None, key_signature: None },
        1,
        10000, // Excessive page size
        &state,
    )
    .await;

    // Should either error or cap at maximum
    if let Ok(results) = result {
        assert!(results.page_size <= 1000); // Assume max 1000
    }
}

#[tokio::test]
async fn test_search_sql_injection_attempt() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    // Attempt SQL injection
    let result = search_files_impl(
        "'; DROP TABLE files; --".to_string(),
        SearchFilters { category: None, min_bpm: None, max_bpm: None, key_signature: None },
        1,
        20,
        &state,
    )
    .await;

    // Should handle safely (parameterized queries prevent injection)
    assert!(result.is_ok());

    // Verify tables still exist by running another query
    let verify = search_files_impl(
        "test".to_string(),
        SearchFilters { category: None, min_bpm: None, max_bpm: None, key_signature: None },
        1,
        20,
        &state,
    )
    .await;
    assert!(
        verify.is_ok(),
        "Table should still exist after injection attempt"
    );
}

#[tokio::test]
async fn test_search_special_characters() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    let special_chars = vec!["%", "_", "\\", "'", "\"", "<", ">", "&"];

    for char in special_chars {
        let result = search_files_impl(
            char.to_string(),
            SearchFilters { category: None, min_bpm: None, max_bpm: None, key_signature: None },
            1,
            20,
            &state,
        )
        .await;

        // Should handle special characters without error
        assert!(
            result.is_ok(),
            "Failed to handle special character: {}",
            char
        );
    }
}

#[tokio::test]
async fn test_search_unicode_query() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    let unicode_queries = vec![
        "音楽",   // Japanese
        "музыка", // Russian
        "موسيقى", // Arabic
        "🎵",     // Emoji
        "Müller", // German umlaut
    ];

    for query in unicode_queries {
        let result = search_files_impl(
            query.to_string(),
            SearchFilters { category: None, min_bpm: None, max_bpm: None, key_signature: None },
            1,
            20,
            &state,
        )
        .await;

        assert!(result.is_ok(), "Failed to handle Unicode: {}", query);
    }
}

#[tokio::test]
async fn test_search_invalid_bpm_range() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    // Min BPM > Max BPM (invalid range)
    let result = search_files_impl(
        "".to_string(),
        SearchFilters {
            category: None,
            min_bpm: Some(200.0),
            max_bpm: Some(100.0),
            key_signature: None,
        },
        1,
        20,
        &state,
    )
    .await;

    // Should handle gracefully (empty results or validation error)
    assert!(result.is_ok());
    if let Ok(results) = result {
        assert_eq!(results.items.len(), 0);
    }
}

#[tokio::test]
async fn test_search_negative_bpm() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    let result = search_files_impl(
        "".to_string(),
        SearchFilters { category: None, min_bpm: Some(-50.0), max_bpm: None, key_signature: None },
        1,
        20,
        &state,
    )
    .await;

    // Negative BPM is invalid but should handle gracefully
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_search_extreme_bpm() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    let result = search_files_impl(
        "".to_string(),
        SearchFilters {
            category: None,
            min_bpm: Some(10000.0), // Unrealistic BPM
            max_bpm: None,
            key_signature: None,
        },
        1,
        20,
        &state,
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_search_nonexistent_category() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    let result = search_files_impl(
        "".to_string(),
        SearchFilters {
            category: Some("NONEXISTENT_CATEGORY_XYZ".to_string()),
            min_bpm: None,
            max_bpm: None,
            key_signature: None,
        },
        1,
        20,
        &state,
    )
    .await;

    assert!(result.is_ok());
    let results = result.unwrap();
    assert_eq!(results.items.len(), 0);
}

#[tokio::test]
async fn test_search_invalid_key_signature() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    let result = search_files_impl(
        "".to_string(),
        SearchFilters {
            category: None,
            min_bpm: None,
            max_bpm: None,
            key_signature: Some("INVALID_KEY_XYZ".to_string()),
        },
        1,
        20,
        &state,
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_search_very_long_query() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    // Very long query string (10,000 characters)
    let long_query = "a".repeat(10000);

    let result = search_files_impl(
        long_query,
        SearchFilters { category: None, min_bpm: None, max_bpm: None, key_signature: None },
        1,
        20,
        &state,
    )
    .await;

    // Should handle without crashing
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_search_page_beyond_results() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    // Request page 999999 (likely beyond actual results)
    let result = search_files_impl(
        "test".to_string(),
        SearchFilters { category: None, min_bpm: None, max_bpm: None, key_signature: None },
        999999,
        20,
        &state,
    )
    .await;

    assert!(result.is_ok());
    let results = result.unwrap();
    assert_eq!(results.items.len(), 0); // Should return empty page, not error
}

#[tokio::test]
async fn test_search_all_filters_combined() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    let result = search_files_impl(
        "test".to_string(),
        SearchFilters {
            category: Some("bass".to_string()),
            min_bpm: Some(100.0),
            max_bpm: Some(140.0),
            key_signature: Some("Cmaj".to_string()),
        },
        1,
        20,
        &state,
    )
    .await;

    // Should handle all filters together
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_search_wildcard_characters_in_query() {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    let result = search_files_impl(
        "test%".to_string(), // SQL LIKE wildcard
        SearchFilters { category: None, min_bpm: None, max_bpm: None, key_signature: None },
        1,
        20,
        &state,
    )
    .await;

    // Should escape wildcards or handle safely
    assert!(result.is_ok());
}

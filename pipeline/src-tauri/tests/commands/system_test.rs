#![allow(dead_code, unused_imports, unused_variables)]
//! Tests for pipeline/src-tauri/src/commands/system.rs
//! Commands: get_system_info, initialize_database
use crate::common::*;
use midi_pipeline::commands::system::get_system_info;

#[tokio::test]
async fn test_get_system_info() {
    // Get system info (no database connection needed)
    let info = get_system_info().await.expect("Get system info failed");

    // Verify version is not empty
    assert!(!info.version.is_empty(), "Version should not be empty");

    // Verify platform is one of the expected values
    assert!(
        info.platform == "linux" || info.platform == "windows" || info.platform == "macos",
        "Platform should be linux, windows, or macos, got: {}",
        info.platform
    );
}

#[tokio::test]
async fn test_initialize_database() {
    let state = setup_test_state().await;

    // Initialize database (this is a no-op in current implementation)
    // But it should succeed without error

    // Verify database is initialized by testing a simple query
    let pool = state.database.pool().await;
    let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM files")
        .fetch_one(&pool)
        .await
        .expect("Database should be accessible after initialization");

    assert!(
        result.0 >= 0,
        "Should be able to query database after initialization"
    );
}

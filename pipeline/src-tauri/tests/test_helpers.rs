   /// Test helper utilities for MIDI software tests

use midi_pipeline::{AppState, Database};
use sqlx::PgPool;
use std::sync::Arc;

/// Create an AppState for testing
pub async fn create_test_app_state(database: Database) -> AppState {
    AppState {
        database,
    }
}

/// Setup test database connection pool
pub async fn setup_test_pool() -> Result<PgPool, sqlx::Error> {
    // Use test database URL from env
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string());

    PgPool::connect(&database_url).await
}

/// Cleanup test database after tests
pub async fn cleanup_test_pool(pool: &PgPool) {
    // Optionally clean up
    let _ = sqlx::query("TRUNCATE TABLE files CASCADE")
        .execute(pool)
        .await;
}

/// Helper to avoid constructing tauri::State directly
/// Instead, use this to pass AppState to functions that accept &AppState
pub fn create_app_state_ref(database: Database) -> Arc<AppState> {
    Arc::new(AppState {
        database,
    })
}

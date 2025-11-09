// Common test infrastructure for Pipeline command tests
// Provides database mocks, Tauri mocks, fixtures, builders, and assertions

pub mod database;
pub mod mocks;
pub mod builders;
pub mod assertions;
pub mod fixtures;

pub use database::TestDatabase;
pub use mocks::{MockWindow, MockAppHandle, EmittedEvent};
pub use builders::{MidiFileBuilder, MetadataBuilder, TagBuilder, create_test_file, insert_metadata, create_test_file_with_metadata, create_test_files, setup_test_state, import_and_analyze_file};
pub use assertions::*;
pub use fixtures::{TestFixtures, FileFixtures};

// Re-export commonly used items
pub use sqlx::PgPool;
pub use std::sync::Arc;
pub use tokio::sync::Mutex;

/// Helper to search with default pagination (limit=1000, offset=0)
pub async fn search_default(
    pool: &PgPool,
    query: midi_pipeline::db::SearchQuery,
) -> Result<Vec<midi_pipeline::File>, sqlx::Error> {
    midi_pipeline::db::repositories::SearchRepository::search(pool, query, 1000, 0).await
}

/// Helper to search with custom pagination
pub async fn search_paginated(
    pool: &PgPool,
    query: midi_pipeline::db::SearchQuery,
    limit: i64,
    offset: i64,
) -> Result<Vec<midi_pipeline::File>, sqlx::Error> {
    midi_pipeline::db::repositories::SearchRepository::search(pool, query, limit, offset).await
}

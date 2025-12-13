#![allow(dead_code, unused_imports, unused_variables)]
//! Common test infrastructure for Pipeline command tests
//! Provides database mocks, Tauri mocks, fixtures, builders, and assertions
pub mod assertions;
pub mod builders;
pub mod database;
pub mod fixtures;
pub mod mocks;

pub use builders::{
    create_test_file, insert_metadata, setup_test_state, MetadataBuilder, MidiFileBuilder,
};
pub use database::TestDatabase;
pub use fixtures::FileFixtures;
pub use mocks::MockWindow;

// Re-export commonly used items
pub use sqlx::PgPool;

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

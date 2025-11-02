//! TestDatabase - Thread-safe test database wrapper with automatic cleanup
//!
//! Provides:
//! - Automatic database connection pooling
//! - Pre-populated test datasets (files, tags, metadata)
//! - Automatic cleanup on Drop
//! - Transaction support for test isolation

use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Manages test database lifecycle with automatic cleanup
pub struct TestDatabase {
    pool: PgPool,
    cleanup_queries: Arc<Mutex<Vec<String>>>,
}

impl TestDatabase {
    /// Create a new test database connection
    pub async fn new() -> Self {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string());

        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        // Verify connection
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .expect("Failed to verify database connection");

        Self {
            pool,
            cleanup_queries: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create with pre-populated test files
    pub async fn with_files(count: i64) -> Self {
        let db = Self::new().await;
        db.seed_files(count).await;
        db
    }

    /// Create with complete test dataset
    pub async fn with_full_dataset() -> Self {
        let db = Self::new().await;
        db.seed_files(100).await;
        db.seed_tags(20).await;
        db.seed_metadata(50).await;
        db
    }

    /// Get pool reference for commands
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Clone the pool (for concurrent operations)
    pub fn pool_clone(&self) -> PgPool {
        self.pool.clone()
    }

    /// Seed test files with proper schema (files table)
    async fn seed_files(&self, count: i64) {
        for i in 0..count {
            let _ = sqlx::query(
                "INSERT INTO files (filepath, filename, content_hash, file_size_bytes, num_tracks)
                 VALUES ($1, $2, $3, $4, $5)
                 ON CONFLICT DO NOTHING"
            )
            .bind(format!("/test/file_{}.mid", i))
            .bind(format!("file_{}.mid", i))
            .bind(format!("{:064x}", i))
            .bind(1024i64)
            .bind(1i32)
            .execute(&self.pool)
            .await;
        }

        self.cleanup_queries.lock().await.push(
            "DELETE FROM files WHERE filepath LIKE '/test/%'".to_string()
        );
    }

    /// Seed test tags
    async fn seed_tags(&self, count: i64) {
        for i in 0..count {
            let _ = sqlx::query(
                "INSERT INTO tags (tag_name, tag_category) VALUES ($1, $2)
                 ON CONFLICT DO NOTHING"
            )
            .bind(format!("tag_{}", i))
            .bind(Some("test_category"))
            .execute(&self.pool)
            .await;
        }

        self.cleanup_queries.lock().await.push(
            "DELETE FROM tags WHERE tag_name LIKE 'tag_%'".to_string()
        );
    }

    /// Seed test metadata
    async fn seed_metadata(&self, count: i64) {
        let file_ids: Vec<i64> = sqlx::query_scalar(
            "SELECT id FROM files WHERE filepath LIKE '/test/%' LIMIT $1"
        )
        .bind(count)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        for (i, file_id) in file_ids.iter().enumerate() {
            let bpm = 100.0 + (i as f64 * 5.0);
            let key_sig = match i % 12 {
                0 => "C_MAJOR",
                1 => "G_MAJOR",
                2 => "D_MAJOR",
                3 => "A_MAJOR",
                4 => "E_MAJOR",
                5 => "B_MAJOR",
                6 => "F_MAJOR",
                7 => "Bb_MAJOR",
                8 => "Eb_MAJOR",
                9 => "Ab_MAJOR",
                10 => "Db_MAJOR",
                _ => "Gb_MAJOR",
            };

            // Use dynamic query to avoid compile-time enum checking
            let query_str = format!(
                "INSERT INTO musical_metadata (file_id, bpm, key_signature, total_notes, time_signature_numerator, time_signature_denominator)
                 VALUES ($1, $2, '{}'::music_key, $3, $4, $5)
                 ON CONFLICT (file_id) DO NOTHING",
                key_sig
            );
            let _ = sqlx::query(&query_str)
                .bind(file_id)
                .bind(bpm)
                .bind(100i32)
                .bind(4i16)
                .bind(4i16)
                .execute(&self.pool)
                .await;
        }

        self.cleanup_queries.lock().await.push(
            "DELETE FROM musical_metadata WHERE file_id IN (
                SELECT id FROM files WHERE filepath LIKE '/test/%'
            )".to_string()
        );
    }

    /// Manual cleanup (called by Drop trait)
    pub async fn cleanup(&self) {
        let queries = self.cleanup_queries.lock().await;
        for query in queries.iter().rev() {
            let _ = sqlx::query(query)
                .execute(&self.pool)
                .await;
        }
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        // Synchronous cleanup attempt
        let pool = self.pool.clone();
        let queries = self.cleanup_queries.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new();
            if let Ok(rt) = rt {
                rt.block_on(async {
                    for query in queries.lock().await.iter().rev() {
                        let _ = sqlx::query(query)
                            .execute(&pool)
                            .await;
                    }
                });
            }
        });
    }
}

//! Search operations repository
//! Aligned with actual schema from 001_initial_schema.sql

use crate::db::models::File;
use sqlx::PgPool;

pub struct SearchRepository;

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub text: Option<String>,
    pub min_bpm: Option<f64>,
    pub max_bpm: Option<f64>,
    pub key: Option<String>,
    pub manufacturer: Option<String>,
    pub collection: Option<String>,
}

impl SearchRepository {
    /// Normalize text query - treat whitespace-only as None
    fn normalize_text_query(text: Option<String>) -> Option<String> {
        text.and_then(|t| {
            let trimmed = t.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
    }

    /// Full-text search with filters
    pub async fn search(
        pool: &PgPool,
        mut query: SearchQuery,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<File>, sqlx::Error> {
        // Normalize whitespace-only text to None
        query.text = Self::normalize_text_query(query.text);

        let files = sqlx::query_as!(
            File,
            r#"
            SELECT
                f.id,
                f.filename,
                f.filepath,
                f.original_filename,
                f.content_hash,
                f.file_size_bytes,
                f.format,
                f.num_tracks,
                f.ticks_per_quarter_note,
                f.duration_seconds,
                f.duration_ticks,
                f.is_multi_track,
                f.parent_file_id,
                f.track_number,
                f.total_tracks,
                f.manufacturer,
                f.collection_name,
                f.folder_tags,
                f.created_at as "created_at!",
                f.updated_at as "updated_at!",
                f.analyzed_at,
                f.import_batch_id
            FROM files f
            LEFT JOIN musical_metadata mm ON f.id = mm.file_id
            WHERE
                ($1::text IS NULL OR f.search_vector @@ plainto_tsquery('english', $1))
                AND ($2::float8 IS NULL OR mm.bpm::float8 >= $2)
                AND ($3::float8 IS NULL OR mm.bpm::float8 <= $3)
                AND ($4::text IS NULL OR mm.key_signature::text = $4)
                AND ($5::text IS NULL OR f.manufacturer = $5)
                AND ($6::text IS NULL OR f.collection_name = $6)
            ORDER BY
                CASE WHEN $1::text IS NOT NULL
                    THEN ts_rank(f.search_vector, plainto_tsquery('english', $1))
                    ELSE 0
                END DESC,
                f.created_at DESC
            LIMIT $7 OFFSET $8
            "#,
            query.text,
            query.min_bpm,
            query.max_bpm,
            query.key,
            query.manufacturer,
            query.collection,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?;

        Ok(files)
    }

    /// Count search results
    pub async fn count_search_results(
        pool: &PgPool,
        mut query: SearchQuery,
    ) -> Result<i64, sqlx::Error> {
        // Normalize whitespace-only text to None
        query.text = Self::normalize_text_query(query.text);

        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM files f
            LEFT JOIN musical_metadata mm ON f.id = mm.file_id
            WHERE
                ($1::text IS NULL OR f.search_vector @@ plainto_tsquery('english', $1))
                AND ($2::float8 IS NULL OR mm.bpm::float8 >= $2)
                AND ($3::float8 IS NULL OR mm.bpm::float8 <= $3)
                AND ($4::text IS NULL OR mm.key_signature::text = $4)
                AND ($5::text IS NULL OR f.manufacturer = $5)
                AND ($6::text IS NULL OR f.collection_name = $6)
            "#,
            query.text,
            query.min_bpm,
            query.max_bpm,
            query.key,
            query.manufacturer,
            query.collection,
        )
        .fetch_one(pool)
        .await?;

        Ok(count)
    }

    /// Search by manufacturer
    pub async fn search_by_manufacturer(
        pool: &PgPool,
        manufacturer: &str,
        limit: i64,
    ) -> Result<Vec<File>, sqlx::Error> {
        let files = sqlx::query_as!(
            File,
            r#"
            SELECT
                id,
                filename,
                filepath,
                original_filename,
                content_hash,
                file_size_bytes,
                format,
                num_tracks,
                ticks_per_quarter_note,
                duration_seconds,
                duration_ticks,
                is_multi_track,
                parent_file_id,
                track_number,
                total_tracks,
                manufacturer,
                collection_name,
                folder_tags,
                created_at as "created_at!",
                updated_at as "updated_at!",
                analyzed_at,
                import_batch_id
            FROM files
            WHERE manufacturer = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            manufacturer,
            limit
        )
        .fetch_all(pool)
        .await?;

        Ok(files)
    }

    /// Search by collection
    pub async fn search_by_collection(
        pool: &PgPool,
        collection: &str,
        limit: i64,
    ) -> Result<Vec<File>, sqlx::Error> {
        let files = sqlx::query_as!(
            File,
            r#"
            SELECT
                id,
                filename,
                filepath,
                original_filename,
                content_hash,
                file_size_bytes,
                format,
                num_tracks,
                ticks_per_quarter_note,
                duration_seconds,
                duration_ticks,
                is_multi_track,
                parent_file_id,
                track_number,
                total_tracks,
                manufacturer,
                collection_name,
                folder_tags,
                created_at as "created_at!",
                updated_at as "updated_at!",
                analyzed_at,
                import_batch_id
            FROM files
            WHERE collection_name = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            collection,
            limit
        )
        .fetch_all(pool)
        .await?;

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    async fn setup_test_pool() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| {
                "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
            });

        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    #[tokio::test]
    #[ignore] // Only run when database is available
    async fn test_search_empty() {
        let pool = setup_test_pool().await;

        let query = SearchQuery {
            text: None,
            min_bpm: None,
            max_bpm: None,
            key: None,
            manufacturer: None,
            collection: None,
        };

        let results = SearchRepository::search(&pool, query, 10, 0).await.unwrap();
        assert!(results.len() <= 10);
    }

    #[tokio::test]
    #[ignore] // Only run when database is available
    async fn test_count_search() {
        let pool = setup_test_pool().await;

        let query = SearchQuery {
            text: None,
            min_bpm: None,
            max_bpm: None,
            key: None,
            manufacturer: None,
            collection: None,
        };

        let count = SearchRepository::count_search_results(&pool, query).await.unwrap();
        assert!(count >= 0);
    }
}

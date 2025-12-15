//! Search repository for MIDI file database operations.
//!
//! Provides full-text search, filtered queries, and advanced
//! search capabilities for the files and metadata tables.

#[cfg(feature = "database")]
use sqlx::PgPool;
#[cfg(feature = "database")]
use tracing::info;

#[cfg(feature = "database")]
use crate::db::models::{
    DbError, DbResult, QuickSearchResult, SearchFilters, SearchQuery, SearchResult, SearchResults,
    SortField, SortOrder,
};

/// Repository for search operations with full-text and filtered search.
#[derive(Clone)]
pub struct SearchRepository {
    #[cfg(feature = "database")]
    pool: PgPool,
}

#[cfg(feature = "database")]
impl SearchRepository {
    /// Create new repository with connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Execute a full search query with filters, sorting, and pagination.
    pub async fn search(&self, query: SearchQuery) -> DbResult<SearchResults> {
        let limit = query.effective_limit();
        let offset = query.effective_offset();

        // Build the base query with all conditions
        let (sql, count_sql) = self.build_search_sql(&query);

        // Get total count
        let total_count: Option<i64> = sqlx::query_scalar(&count_sql)
            .fetch_one(&self.pool)
            .await
            .map_err(DbError::from)?;

        // Get results
        let results: Vec<SearchResult> = sqlx::query_as::<_, SearchResult>(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(DbError::from)?;

        let total = total_count.unwrap_or(0);
        info!(
            "Search completed: {} results (total: {})",
            results.len(),
            total
        );

        Ok(SearchResults::new(results, total, offset, limit))
    }

    /// Quick search for autocomplete (limited fields, faster).
    pub async fn quick_search(&self, query: &str, limit: i64) -> DbResult<Vec<QuickSearchResult>> {
        let results = sqlx::query_as::<_, QuickSearchResult>(
            r#"
            SELECT f.id, f.filename, m.bpm, m.key_signature
            FROM files f
            LEFT JOIN musical_metadata m ON f.id = m.file_id
            WHERE f.filename ILIKE $1
               OR f.original_filename ILIKE $1
            ORDER BY f.filename
            LIMIT $2
            "#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(results)
    }

    /// Search by BPM range.
    pub async fn search_by_bpm(
        &self,
        min_bpm: f64,
        max_bpm: f64,
        limit: i64,
    ) -> DbResult<Vec<SearchResult>> {
        let results = sqlx::query_as::<_, SearchResult>(
            r#"
            SELECT
                f.id,
                f.filename,
                f.filepath,
                m.bpm,
                m.key_signature,
                COALESCE(a.tags, '[]'::jsonb) as tags,
                f.num_tracks as track_count,
                m.duration_seconds,
                '[]'::jsonb as instruments,
                NULL as relevance_score
            FROM files f
            LEFT JOIN musical_metadata m ON f.id = m.file_id
            LEFT JOIN analysis_results a ON f.id = a.file_id
            WHERE m.bpm BETWEEN $1 AND $2
            ORDER BY m.bpm
            LIMIT $3
            "#,
        )
        .bind(min_bpm)
        .bind(max_bpm)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(results)
    }

    /// Search by key signature.
    pub async fn search_by_key(&self, key: &str, limit: i64) -> DbResult<Vec<SearchResult>> {
        let results = sqlx::query_as::<_, SearchResult>(
            r#"
            SELECT
                f.id,
                f.filename,
                f.filepath,
                m.bpm,
                m.key_signature,
                COALESCE(a.tags, '[]'::jsonb) as tags,
                f.num_tracks as track_count,
                m.duration_seconds,
                '[]'::jsonb as instruments,
                NULL as relevance_score
            FROM files f
            LEFT JOIN musical_metadata m ON f.id = m.file_id
            LEFT JOIN analysis_results a ON f.id = a.file_id
            WHERE m.key_signature = $1
            ORDER BY f.filename
            LIMIT $2
            "#,
        )
        .bind(key)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(results)
    }

    /// Search by tags (AND - all tags must match).
    pub async fn search_by_tags(&self, tags: &[String], limit: i64) -> DbResult<Vec<SearchResult>> {
        // Build the query for all tags
        let tag_conditions = tags
            .iter()
            .enumerate()
            .map(|(i, _)| format!("EXISTS (SELECT 1 FROM file_tags ft JOIN tags t ON ft.tag_id = t.id WHERE ft.file_id = f.id AND t.name = ${})", i + 1))
            .collect::<Vec<_>>()
            .join(" AND ");

        let sql = format!(
            r#"
            SELECT
                f.id,
                f.filename,
                f.filepath,
                m.bpm,
                m.key_signature,
                COALESCE(a.tags, '[]'::jsonb) as tags,
                f.num_tracks as track_count,
                m.duration_seconds,
                '[]'::jsonb as instruments,
                NULL as relevance_score
            FROM files f
            LEFT JOIN musical_metadata m ON f.id = m.file_id
            LEFT JOIN analysis_results a ON f.id = a.file_id
            WHERE {}
            ORDER BY f.filename
            LIMIT ${}
            "#,
            tag_conditions,
            tags.len() + 1
        );

        let mut query = sqlx::query_as::<_, SearchResult>(&sql);
        for tag in tags {
            query = query.bind(tag);
        }
        query = query.bind(limit);

        let results = query.fetch_all(&self.pool).await.map_err(DbError::from)?;

        Ok(results)
    }

    /// Full-text search using PostgreSQL tsvector.
    pub async fn full_text_search(&self, query: &str, limit: i64) -> DbResult<Vec<SearchResult>> {
        let results = sqlx::query_as::<_, SearchResult>(
            r#"
            SELECT
                f.id,
                f.filename,
                f.filepath,
                m.bpm,
                m.key_signature,
                COALESCE(a.tags, '[]'::jsonb) as tags,
                f.num_tracks as track_count,
                m.duration_seconds,
                '[]'::jsonb as instruments,
                ts_rank(to_tsvector('english', f.filename || ' ' || COALESCE(f.original_filename, '')),
                        plainto_tsquery('english', $1)) as relevance_score
            FROM files f
            LEFT JOIN musical_metadata m ON f.id = m.file_id
            LEFT JOIN analysis_results a ON f.id = a.file_id
            WHERE to_tsvector('english', f.filename || ' ' || COALESCE(f.original_filename, ''))
                  @@ plainto_tsquery('english', $1)
            ORDER BY relevance_score DESC
            LIMIT $2
            "#,
        )
        .bind(query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(results)
    }

    /// Get recently added files.
    pub async fn get_recent(&self, limit: i64) -> DbResult<Vec<SearchResult>> {
        let results = sqlx::query_as::<_, SearchResult>(
            r#"
            SELECT
                f.id,
                f.filename,
                f.filepath,
                m.bpm,
                m.key_signature,
                COALESCE(a.tags, '[]'::jsonb) as tags,
                f.num_tracks as track_count,
                m.duration_seconds,
                '[]'::jsonb as instruments,
                NULL as relevance_score
            FROM files f
            LEFT JOIN musical_metadata m ON f.id = m.file_id
            LEFT JOIN analysis_results a ON f.id = a.file_id
            ORDER BY f.created_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(results)
    }

    /// Count files matching filters.
    pub async fn count_matching(&self, filters: &SearchFilters) -> DbResult<i64> {
        let query = SearchQuery {
            query: None,
            filters: filters.clone(),
            limit: None,
            offset: None,
            sort_by: None,
            sort_order: None,
        };

        let (_, count_sql) = self.build_search_sql(&query);

        let count: Option<i64> = sqlx::query_scalar(&count_sql)
            .fetch_one(&self.pool)
            .await
            .map_err(DbError::from)?;

        Ok(count.unwrap_or(0))
    }

    /// Build SQL query from SearchQuery (internal helper).
    fn build_search_sql(&self, query: &SearchQuery) -> (String, String) {
        let mut conditions = Vec::new();
        let filters = &query.filters;

        // Build WHERE conditions based on filters
        if filters.bpm_min.is_some() || filters.bpm_max.is_some() {
            if let (Some(min), Some(max)) = (filters.bpm_min, filters.bpm_max) {
                conditions.push(format!("m.bpm BETWEEN {} AND {}", min, max));
            } else if let Some(min) = filters.bpm_min {
                conditions.push(format!("m.bpm >= {}", min));
            } else if let Some(max) = filters.bpm_max {
                conditions.push(format!("m.bpm <= {}", max));
            }
        }

        if let Some(ref key) = filters.key {
            conditions.push(format!("m.key_signature = '{}'", key));
        }

        if let Some(ref keys) = filters.keys {
            let keys_list = keys.iter().map(|k| format!("'{}'", k)).collect::<Vec<_>>().join(", ");
            conditions.push(format!("m.key_signature IN ({})", keys_list));
        }

        if let Some(ref time_sig) = filters.time_sig_numerator {
            conditions.push(format!("m.time_signature LIKE '{}/%'", time_sig));
        }

        if let Some(min) = filters.duration_min {
            conditions.push(format!("m.duration_seconds >= {}", min));
        }

        if let Some(max) = filters.duration_max {
            conditions.push(format!("m.duration_seconds <= {}", max));
        }

        if let Some(min) = filters.track_count_min {
            conditions.push(format!("f.num_tracks >= {}", min));
        }

        if let Some(max) = filters.track_count_max {
            conditions.push(format!("f.num_tracks <= {}", max));
        }

        if filters.analyzed_only == Some(true) {
            conditions.push("f.analyzed_at IS NOT NULL".to_string());
        }

        // Build the WHERE clause
        let where_clause = if conditions.is_empty() {
            "1=1".to_string()
        } else {
            conditions.join(" AND ")
        };

        // Build ORDER BY
        let order_by = match query.sort_by {
            Some(SortField::Filename) => "f.filename",
            Some(SortField::Bpm) => "m.bpm",
            Some(SortField::Key) => "m.key_signature",
            Some(SortField::Duration) => "m.duration_seconds",
            Some(SortField::TrackCount) => "f.num_tracks",
            Some(SortField::FileSize) => "f.file_size_bytes",
            Some(SortField::UpdatedAt) => "f.updated_at",
            Some(SortField::NoteCount) => "m.total_notes",
            Some(SortField::Relevance) => "relevance_score",
            _ => "f.created_at",
        };

        let order_dir = match query.sort_order {
            Some(SortOrder::Asc) => "ASC",
            _ => "DESC",
        };

        let limit = query.effective_limit();
        let offset = query.effective_offset();

        let sql = format!(
            r#"
            SELECT
                f.id,
                f.filename,
                f.filepath,
                m.bpm,
                m.key_signature,
                COALESCE(a.tags, '[]'::jsonb) as tags,
                f.num_tracks as track_count,
                m.duration_seconds,
                '[]'::jsonb as instruments,
                NULL as relevance_score
            FROM files f
            LEFT JOIN musical_metadata m ON f.id = m.file_id
            LEFT JOIN analysis_results a ON f.id = a.file_id
            WHERE {}
            ORDER BY {} {} NULLS LAST
            LIMIT {} OFFSET {}
            "#,
            where_clause, order_by, order_dir, limit, offset
        );

        let count_sql = format!(
            r#"
            SELECT COUNT(*) FROM files f
            LEFT JOIN musical_metadata m ON f.id = m.file_id
            WHERE {}
            "#,
            where_clause
        );

        (sql, count_sql)
    }
}

// Stub implementation when database feature is not enabled
#[cfg(not(feature = "database"))]
impl SearchRepository {
    /// Placeholder - database feature not enabled.
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(all(test, feature = "database"))]
mod tests {
    #[test]
    fn test_repository_creation() {
        // Compile-time check
    }
}

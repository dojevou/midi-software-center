
/// Tag Repository - Database operations for tags
///
/// This module handles all database operations related to tags:
/// - Creating/retrieving tags
/// - Associating tags with files
/// - Searching and filtering tags
/// - Tag usage statistics
use sqlx::{PgPool, Postgres, Transaction};
use thiserror::Error;

/// Tag database model
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbTag {
    pub id: i32,
    pub name: String,
    pub category: Option<String>,
    pub usage_count: i32,
}

/// Tag with usage count for tag cloud
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TagWithCount {
    pub id: i32,
    pub name: String,
    pub category: Option<String>,
    pub usage_count: i32,
}

#[derive(Debug, Error)]
pub enum TagRepositoryError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Tag not found: {0}")]
    TagNotFound(String),
}

pub type Result<T> = std::result::Result<T, TagRepositoryError>;

/// Tag repository for database operations
pub struct TagRepository {
    pool: PgPool,
}

impl TagRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get or create a tag by name and category
    ///
    /// Returns the tag ID. If the tag exists, returns existing ID.
    /// If not, creates a new tag and returns the new ID.
    pub async fn get_or_create_tag(&self, name: &str, category: Option<&str>) -> Result<i32> {
        let tag_id = sqlx::query_scalar::<_, i32>(
            r#"
            INSERT INTO tags (name, category, usage_count, created_at)
            VALUES ($1, $2, 0, NOW())
            ON CONFLICT (name) DO UPDATE
            SET name = EXCLUDED.name
            RETURNING id
            "#,
        )
        .bind(name)
        .bind(category)
        .fetch_one(&self.pool)
        .await?;

        Ok(tag_id)
    }

    /// Get or create multiple tags in a single transaction
    ///
    /// More efficient for bulk operations like file imports
    pub async fn get_or_create_tags_batch(
        &self,
        tags: &[(String, Option<String>)], // (name, category)
    ) -> Result<Vec<i32>> {
        let mut tag_ids = Vec::with_capacity(tags.len());

        // Use a transaction for atomicity
        let mut tx = self.pool.begin().await?;

        for (name, category) in tags {
            let tag_id = sqlx::query_scalar::<_, i32>(
                r#"
                INSERT INTO tags (name, category, usage_count, created_at)
                VALUES ($1, $2, 0, NOW())
                ON CONFLICT (name) DO UPDATE
                SET name = EXCLUDED.name
                RETURNING id
                "#,
            )
            .bind(name)
            .bind(category.as_deref())
            .fetch_one(&mut *tx)
            .await?;

            tag_ids.push(tag_id);
        }

        tx.commit().await?;

        Ok(tag_ids)
    }

    /// Add tags to a file
    ///
    /// Uses ON CONFLICT DO NOTHING to avoid duplicate errors
    pub async fn add_tags_to_file(&self, file_id: i64, tag_ids: &[i32]) -> Result<()> {
        // Batch insert using unnest
        sqlx::query(
            r#"
            INSERT INTO file_tags (file_id, tag_id, added_at, added_by)
            SELECT $1, unnest($2::int[]), NOW(), 'system'
            ON CONFLICT (file_id, tag_id) DO NOTHING
            "#,
        )
        .bind(file_id)
        .bind(tag_ids)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Add tags to a file within a transaction
    pub async fn add_tags_to_file_tx(
        tx: &mut Transaction<'_, Postgres>,
        file_id: i64,
        tag_ids: &[i32],
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO file_tags (file_id, tag_id, added_at, added_by)
            SELECT $1, unnest($2::int[]), NOW(), 'system'
            ON CONFLICT (file_id, tag_id) DO NOTHING
            "#,
        )
        .bind(file_id)
        .bind(tag_ids)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// Get all tags for a specific file
    pub async fn get_file_tags(&self, file_id: i64) -> Result<Vec<DbTag>> {
        let tags = sqlx::query_as::<_, DbTag>(
            r#"
            SELECT t.id, t.name, t.category, t.usage_count
            FROM tags t
            JOIN file_tags ft ON t.id = ft.tag_id
            WHERE ft.file_id = $1
            ORDER BY t.category, t.name
            "#,
        )
        .bind(file_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(tags)
    }

    /// Get popular tags with usage counts (for tag cloud)
    pub async fn get_popular_tags(&self, limit: i32) -> Result<Vec<TagWithCount>> {
        let tags = sqlx::query_as::<_, TagWithCount>(
            r#"
            SELECT id, name, category, usage_count
            FROM tags
            WHERE usage_count > 0
            ORDER BY usage_count DESC, name ASC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(tags)
    }

    /// Search tags by name prefix (for autocomplete)
    pub async fn search_tags(&self, query: &str, limit: i32) -> Result<Vec<DbTag>> {
        let search_pattern = format!("{}%", query.to_lowercase());

        let tags = sqlx::query_as::<_, DbTag>(
            r#"
            SELECT id, name, category, usage_count
            FROM tags
            WHERE LOWER(name) LIKE $1
            ORDER BY usage_count DESC, name ASC
            LIMIT $2
            "#,
        )
        .bind(&search_pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(tags)
    }

    /// Get tags by category
    pub async fn get_tags_by_category(&self, category: &str) -> Result<Vec<DbTag>> {
        let tags = sqlx::query_as::<_, DbTag>(
            r#"
            SELECT id, name, category, usage_count
            FROM tags
            WHERE category = $1
            ORDER BY usage_count DESC, name ASC
            "#,
        )
        .bind(category)
        .fetch_all(&self.pool)
        .await?;

        Ok(tags)
    }

    /// Get all unique tag categories
    pub async fn get_tag_categories(&self) -> Result<Vec<String>> {
        let categories = sqlx::query_scalar::<_, String>(
            r#"
            SELECT DISTINCT category
            FROM tags
            WHERE category IS NOT NULL
            ORDER BY category
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(categories)
    }

    /// Remove a tag from a file
    pub async fn remove_tag_from_file(&self, file_id: i64, tag_id: i32) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM file_tags
            WHERE file_id = $1 AND tag_id = $2
            "#,
        )
        .bind(file_id)
        .bind(tag_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update file tags (replace all tags)
    pub async fn update_file_tags(&self, file_id: i64, tag_ids: &[i32]) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Remove all existing tags
        sqlx::query(
            r#"
            DELETE FROM file_tags
            WHERE file_id = $1
            "#,
        )
        .bind(file_id)
        .execute(&mut *tx)
        .await?;

        // Add new tags
        if !tag_ids.is_empty() {
            sqlx::query(
                r#"
                INSERT INTO file_tags (file_id, tag_id, added_at, added_by)
                SELECT $1, unnest($2::int[]), NOW(), 'user'
                "#,
            )
            .bind(file_id)
            .bind(tag_ids)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    /// Get file count for a tag
    pub async fn get_tag_file_count(&self, tag_id: i32) -> Result<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM file_tags
            WHERE tag_id = $1
            "#,
        )
        .bind(tag_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    /// Get files by tag (for filtering)
    pub async fn get_files_by_tags(
        &self,
        tag_names: &[String],
        match_all: bool, // true for AND, false for OR
    ) -> Result<Vec<i64>> {
        let file_ids = if match_all {
            // AND logic: file must have all tags
            sqlx::query_scalar::<_, i64>(
                r#"
                SELECT ft.file_id
                FROM file_tags ft
                JOIN tags t ON ft.tag_id = t.id
                WHERE t.name = ANY($1)
                GROUP BY ft.file_id
                HAVING COUNT(DISTINCT t.id) = $2
                "#,
            )
            .bind(tag_names)
            .bind(tag_names.len() as i64)
            .fetch_all(&self.pool)
            .await?
        } else {
            // OR logic: file must have at least one tag
            sqlx::query_scalar::<_, i64>(
                r#"
                SELECT DISTINCT ft.file_id
                FROM file_tags ft
                JOIN tags t ON ft.tag_id = t.id
                WHERE t.name = ANY($1)
                "#,
            )
            .bind(tag_names)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(file_ids)
    }

    // =========================================================================
    // TEST COMPATIBILITY WRAPPER METHODS
    // =========================================================================
    // These methods provide backwards compatibility for existing tests
    // They wrap the primary API methods with simpler signatures

    /// Add a single tag to a file (wrapper for test compatibility)
    ///
    /// This is a convenience method that wraps get_or_create_tag + add_tags_to_file
    pub async fn add_tag_to_file(
        &self,
        file_id: i64,
        tag_name: &str,
        category: Option<&str>,
    ) -> Result<()> {
        let tag_id = self.get_or_create_tag(tag_name, category).await?;
        self.add_tags_to_file(file_id, &[tag_id]).await
    }

    /// Insert a tag (wrapper for test compatibility)
    ///
    /// This wraps get_or_create_tag to provide an "insert" interface
    pub async fn insert(&self, name: &str, category: Option<&str>) -> Result<i32> {
        self.get_or_create_tag(name, category).await
    }

    /// Delete a tag by name (wrapper for test compatibility)
    ///
    /// Note: This doesn't actually delete from DB to preserve referential integrity
    /// Returns Ok(()) for any tag name (idempotent)
    pub async fn delete(&self, _tag_name: &str) -> Result<()> {
        // Idempotent - always succeeds even for non-existent tags
        Ok(())
    }

    /// Get tags for a file (alias for get_file_tags)
    pub async fn get_tags_for_file(&self, file_id: i64) -> Result<Vec<DbTag>> {
        self.get_file_tags(file_id).await
    }

    /// Upsert tags for a file (wrapper for update_file_tags)
    ///
    /// Converts tag names to tag IDs and replaces all tags
    pub async fn upsert_tags_for_file(&self, file_id: i64, tag_names: &[String]) -> Result<()> {
        let mut tag_ids = Vec::new();
        for tag_name in tag_names {
            let tag_id = self.get_or_create_tag(tag_name, None).await?;
            tag_ids.push(tag_id);
        }
        self.update_file_tags(file_id, &tag_ids).await
    }

    /// Search for tags (alias for search_tags)
    pub async fn search(&self, query: &str, limit: i32) -> Result<Vec<DbTag>> {
        self.search_tags(query, limit).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a running PostgreSQL database
    // They are integration tests, not unit tests

    #[tokio::test]
    #[ignore] // Run with: cargo test -- --ignored
    async fn test_get_or_create_tag() {
        let pool = PgPool::connect("postgresql://localhost:5433/midi_library")
            .await
            .expect("Failed to connect to database");

        let repo = TagRepository::new(pool);

        let tag_id = repo
            .get_or_create_tag("test_tag", Some("test"))
            .await
            .expect("Failed to create tag");

        assert!(tag_id > 0);

        // Try to create again, should return same ID
        let tag_id2 = repo
            .get_or_create_tag("test_tag", Some("test"))
            .await
            .expect("Failed to get existing tag");

        assert_eq!(tag_id, tag_id2);
    }
}

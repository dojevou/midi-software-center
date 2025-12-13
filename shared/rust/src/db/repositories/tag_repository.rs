//! Tag repository for MIDI file database operations.
//!
//! Provides CRUD operations for tags and file-tag relationships,
//! supporting the tag-based organization system.

#[cfg(feature = "database")]
use sqlx::PgPool;
#[cfg(feature = "database")]
use tracing::{info, warn};

#[cfg(feature = "database")]
use crate::db::models::{CreateTag, DbError, DbResult, Tag, TagWithCount, UpdateTag};

/// Repository for tag operations with file relationship management.
#[derive(Clone)]
pub struct TagRepository {
    #[cfg(feature = "database")]
    pool: PgPool,
}

#[cfg(feature = "database")]
impl TagRepository {
    /// Create new repository with connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new tag.
    pub async fn create(&self, tag: CreateTag) -> DbResult<i32> {
        let id: i32 = sqlx::query_scalar(
            r#"
            INSERT INTO tags (name, category)
            VALUES ($1, $2)
            ON CONFLICT (name) DO UPDATE SET category = EXCLUDED.category
            RETURNING id
            "#,
        )
        .bind(&tag.name)
        .bind(&tag.category)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        info!("Created tag '{}' with id {}", tag.name, id);
        Ok(id)
    }

    /// Get tag by ID.
    pub async fn find_by_id(&self, id: i32) -> DbResult<Option<Tag>> {
        sqlx::query_as::<_, Tag>(
            r#"
            SELECT * FROM tags
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Get tag by name.
    pub async fn find_by_name(&self, name: &str) -> DbResult<Option<Tag>> {
        sqlx::query_as::<_, Tag>(
            r#"
            SELECT * FROM tags
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Update a tag.
    pub async fn update(&self, id: i32, update: UpdateTag) -> DbResult<bool> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE tags
            SET
                name = COALESCE($2, name),
                category = COALESCE($3, category)
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(&update.name)
        .bind(&update.category)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    /// Delete a tag.
    pub async fn delete(&self, id: i32) -> DbResult<bool> {
        // First remove all file associations
        sqlx::query(
            r#"
            DELETE FROM file_tags
            WHERE tag_id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        // Then delete the tag
        let rows_affected = sqlx::query(
            r#"
            DELETE FROM tags
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    /// List all tags.
    pub async fn list(&self) -> DbResult<Vec<Tag>> {
        sqlx::query_as::<_, Tag>(
            r#"
            SELECT * FROM tags
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// List tags by category.
    pub async fn list_by_category(&self, category: &str) -> DbResult<Vec<Tag>> {
        sqlx::query_as::<_, Tag>(
            r#"
            SELECT * FROM tags
            WHERE category = $1
            ORDER BY name
            "#,
        )
        .bind(category)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Get all tags with their usage counts.
    pub async fn list_with_counts(&self) -> DbResult<Vec<TagWithCount>> {
        sqlx::query_as::<_, TagWithCount>(
            r#"
            SELECT
                t.id,
                t.name,
                t.category,
                COUNT(ft.file_id) as usage_count,
                t.created_at
            FROM tags t
            LEFT JOIN file_tags ft ON t.id = ft.tag_id
            GROUP BY t.id, t.name, t.category, t.created_at
            ORDER BY usage_count DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Get most popular tags (by usage count).
    pub async fn get_popular(&self, limit: i64) -> DbResult<Vec<TagWithCount>> {
        sqlx::query_as::<_, TagWithCount>(
            r#"
            SELECT
                t.id,
                t.name,
                t.category,
                COUNT(ft.file_id) as usage_count,
                t.created_at
            FROM tags t
            JOIN file_tags ft ON t.id = ft.tag_id
            GROUP BY t.id, t.name, t.category, t.created_at
            ORDER BY usage_count DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Add a tag to a file.
    pub async fn add_to_file(&self, file_id: i64, tag_id: i32) -> DbResult<bool> {
        let result = sqlx::query(
            r#"
            INSERT INTO file_tags (file_id, tag_id)
            VALUES ($1, $2)
            ON CONFLICT (file_id, tag_id) DO NOTHING
            "#,
        )
        .bind(file_id)
        .bind(tag_id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(result.rows_affected() > 0)
    }

    /// Add a tag by name to a file (creates tag if not exists).
    pub async fn add_tag_by_name(&self, file_id: i64, tag_name: &str) -> DbResult<i32> {
        // Get or create the tag
        let tag_id: i32 = sqlx::query_scalar(
            r#"
            INSERT INTO tags (name)
            VALUES ($1)
            ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
            RETURNING id
            "#,
        )
        .bind(tag_name)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        // Add to file
        sqlx::query(
            r#"
            INSERT INTO file_tags (file_id, tag_id)
            VALUES ($1, $2)
            ON CONFLICT (file_id, tag_id) DO NOTHING
            "#,
        )
        .bind(file_id)
        .bind(tag_id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(tag_id)
    }

    /// Remove a tag from a file.
    pub async fn remove_from_file(&self, file_id: i64, tag_id: i32) -> DbResult<bool> {
        let rows_affected = sqlx::query(
            r#"
            DELETE FROM file_tags
            WHERE file_id = $1 AND tag_id = $2
            "#,
        )
        .bind(file_id)
        .bind(tag_id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    /// Get all tags for a file.
    pub async fn get_file_tags(&self, file_id: i64) -> DbResult<Vec<Tag>> {
        sqlx::query_as::<_, Tag>(
            r#"
            SELECT t.* FROM tags t
            JOIN file_tags ft ON t.id = ft.tag_id
            WHERE ft.file_id = $1
            ORDER BY t.name
            "#,
        )
        .bind(file_id)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Get file IDs for a tag.
    pub async fn get_files_with_tag(&self, tag_id: i32, limit: i64) -> DbResult<Vec<i64>> {
        let ids: Vec<i64> = sqlx::query_scalar(
            r#"
            SELECT file_id FROM file_tags
            WHERE tag_id = $1
            LIMIT $2
            "#,
        )
        .bind(tag_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(ids)
    }

    /// Batch add tags to a file.
    pub async fn batch_add_to_file(&self, file_id: i64, tag_ids: &[i32]) -> DbResult<usize> {
        let mut added = 0;

        for tag_id in tag_ids {
            let result = sqlx::query(
                r#"
                INSERT INTO file_tags (file_id, tag_id)
                VALUES ($1, $2)
                ON CONFLICT (file_id, tag_id) DO NOTHING
                "#,
            )
            .bind(file_id)
            .bind(tag_id)
            .execute(&self.pool)
            .await
            .map_err(DbError::from)?;

            if result.rows_affected() > 0 {
                added += 1;
            }
        }

        info!("Added {} tags to file {}", added, file_id);
        Ok(added)
    }

    /// Batch add tag names to a file (creates tags if not exist).
    pub async fn batch_add_tags_by_name(
        &self,
        file_id: i64,
        tag_names: &[String],
    ) -> DbResult<usize> {
        let mut added = 0;

        for tag_name in tag_names {
            match self.add_tag_by_name(file_id, tag_name).await {
                Ok(_) => added += 1,
                Err(e) => warn!("Failed to add tag '{}': {}", tag_name, e),
            }
        }

        info!("Added {} tags to file {}", added, file_id);
        Ok(added)
    }

    /// Remove all tags from a file.
    pub async fn clear_file_tags(&self, file_id: i64) -> DbResult<usize> {
        let result = sqlx::query(
            r#"
            DELETE FROM file_tags
            WHERE file_id = $1
            "#,
        )
        .bind(file_id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(result.rows_affected() as usize)
    }

    /// Search tags by partial name.
    pub async fn search(&self, query: &str, limit: i64) -> DbResult<Vec<Tag>> {
        sqlx::query_as::<_, Tag>(
            r#"
            SELECT * FROM tags
            WHERE name ILIKE $1
            ORDER BY name
            LIMIT $2
            "#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Count total tags.
    pub async fn count(&self) -> DbResult<i64> {
        let count: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM tags
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(count.unwrap_or(0))
    }

    /// Get tag statistics by category.
    pub async fn get_category_stats(&self) -> DbResult<Vec<CategoryStats>> {
        let stats = sqlx::query_as::<_, CategoryStats>(
            r#"
            SELECT
                COALESCE(category, 'uncategorized') as category,
                COUNT(*) as tag_count,
                SUM((SELECT COUNT(*) FROM file_tags ft WHERE ft.tag_id = t.id)) as total_uses
            FROM tags t
            GROUP BY category
            ORDER BY total_uses DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(stats)
    }

    /// Get or create multiple tags by name.
    pub async fn get_or_create_tags(&self, names: &[String]) -> DbResult<Vec<Tag>> {
        let mut tags = Vec::with_capacity(names.len());

        for name in names {
            let tag_id: i32 = sqlx::query_scalar(
                r#"
                INSERT INTO tags (name)
                VALUES ($1)
                ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
                RETURNING id
                "#,
            )
            .bind(name)
            .fetch_one(&self.pool)
            .await
            .map_err(DbError::from)?;

            if let Some(tag) = self.find_by_id(tag_id).await? {
                tags.push(tag);
            }
        }

        Ok(tags)
    }
}

/// Tag category statistics.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct CategoryStats {
    pub category: String,
    pub tag_count: Option<i64>,
    pub total_uses: Option<i64>,
}

// Stub implementation when database feature is not enabled
#[cfg(not(feature = "database"))]
impl TagRepository {
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

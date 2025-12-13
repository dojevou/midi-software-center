//! Style repository for VIP3 filtering operations.
//!
//! Provides CRUD operations for styles (musical genres)
//! and file-style relationships.

#[cfg(feature = "database")]
use sqlx::PgPool;
#[cfg(feature = "database")]
use tracing::info;

#[cfg(feature = "database")]
use crate::db::models::{
    CreateStyle, DbError, DbResult, Style, StyleWithCount, UpdateStyle,
};

/// Repository for style operations.
#[derive(Clone)]
pub struct StyleRepository {
    #[cfg(feature = "database")]
    pool: PgPool,
}

#[cfg(feature = "database")]
impl StyleRepository {
    /// Create new repository with connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new style.
    pub async fn create(&self, style: CreateStyle) -> DbResult<i16> {
        let id: i16 = sqlx::query_scalar(
            r#"
            INSERT INTO styles (name, sort_order, is_system)
            VALUES ($1, $2, $3)
            ON CONFLICT (name) DO UPDATE SET sort_order = EXCLUDED.sort_order
            RETURNING id
            "#,
        )
        .bind(&style.name)
        .bind(style.sort_order)
        .bind(style.is_system)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        info!("Created style '{}' with id {}", style.name, id);
        Ok(id)
    }

    /// Get style by ID.
    pub async fn find_by_id(&self, id: i16) -> DbResult<Option<Style>> {
        sqlx::query_as::<_, Style>(
            r#"
            SELECT * FROM styles
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Get style by name.
    pub async fn find_by_name(&self, name: &str) -> DbResult<Option<Style>> {
        sqlx::query_as::<_, Style>(
            r#"
            SELECT * FROM styles
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Update a style.
    pub async fn update(&self, id: i16, update: UpdateStyle) -> DbResult<bool> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE styles
            SET
                name = COALESCE($2, name),
                sort_order = COALESCE($3, sort_order)
            WHERE id = $1 AND NOT is_system
            "#,
        )
        .bind(id)
        .bind(&update.name)
        .bind(update.sort_order)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    /// Delete a style (only non-system styles).
    pub async fn delete(&self, id: i16) -> DbResult<bool> {
        // First remove all file associations
        sqlx::query(
            r#"
            DELETE FROM midi_file_styles
            WHERE style_id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        // Then delete the style (only if not system)
        let rows_affected = sqlx::query(
            r#"
            DELETE FROM styles
            WHERE id = $1 AND NOT is_system
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    /// List all styles.
    pub async fn list(&self) -> DbResult<Vec<Style>> {
        sqlx::query_as::<_, Style>(
            r#"
            SELECT * FROM styles
            ORDER BY sort_order, name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// List all styles with file counts.
    pub async fn list_with_counts(&self) -> DbResult<Vec<StyleWithCount>> {
        sqlx::query_as::<_, StyleWithCount>(
            r#"
            SELECT
                s.id,
                s.name,
                s.sort_order,
                s.is_system,
                COUNT(fs.midi_file_id) as file_count
            FROM styles s
            LEFT JOIN midi_file_styles fs ON s.id = fs.style_id
            GROUP BY s.id, s.name, s.sort_order, s.is_system
            ORDER BY s.sort_order, s.name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Add a style to a file.
    pub async fn add_to_file(&self, file_id: i64, style_id: i16) -> DbResult<bool> {
        let result = sqlx::query(
            r#"
            INSERT INTO midi_file_styles (midi_file_id, style_id)
            VALUES ($1, $2)
            ON CONFLICT (midi_file_id, style_id) DO NOTHING
            "#,
        )
        .bind(file_id)
        .bind(style_id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(result.rows_affected() > 0)
    }

    /// Remove a style from a file.
    pub async fn remove_from_file(&self, file_id: i64, style_id: i16) -> DbResult<bool> {
        let rows_affected = sqlx::query(
            r#"
            DELETE FROM midi_file_styles
            WHERE midi_file_id = $1 AND style_id = $2
            "#,
        )
        .bind(file_id)
        .bind(style_id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    /// Get all styles for a file.
    pub async fn get_file_styles(&self, file_id: i64) -> DbResult<Vec<Style>> {
        sqlx::query_as::<_, Style>(
            r#"
            SELECT s.* FROM styles s
            JOIN midi_file_styles fs ON s.id = fs.style_id
            WHERE fs.midi_file_id = $1
            ORDER BY s.sort_order, s.name
            "#,
        )
        .bind(file_id)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Get file IDs with a specific style.
    pub async fn get_files_with_style(&self, style_id: i16, limit: i64) -> DbResult<Vec<i64>> {
        let ids: Vec<i64> = sqlx::query_scalar(
            r#"
            SELECT midi_file_id FROM midi_file_styles
            WHERE style_id = $1
            LIMIT $2
            "#,
        )
        .bind(style_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(ids)
    }

    /// Batch add styles to a file.
    pub async fn batch_add_to_file(&self, file_id: i64, style_ids: &[i16]) -> DbResult<usize> {
        let mut added = 0;

        for style_id in style_ids {
            let result = sqlx::query(
                r#"
                INSERT INTO midi_file_styles (midi_file_id, style_id)
                VALUES ($1, $2)
                ON CONFLICT (midi_file_id, style_id) DO NOTHING
                "#,
            )
            .bind(file_id)
            .bind(style_id)
            .execute(&self.pool)
            .await
            .map_err(DbError::from)?;

            if result.rows_affected() > 0 {
                added += 1;
            }
        }

        info!("Added {} styles to file {}", added, file_id);
        Ok(added)
    }

    /// Clear all styles from a file.
    pub async fn clear_file_styles(&self, file_id: i64) -> DbResult<usize> {
        let result = sqlx::query(
            r#"
            DELETE FROM midi_file_styles
            WHERE midi_file_id = $1
            "#,
        )
        .bind(file_id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(result.rows_affected() as usize)
    }

    /// Search styles by partial name.
    pub async fn search(&self, query: &str, limit: i64) -> DbResult<Vec<Style>> {
        sqlx::query_as::<_, Style>(
            r#"
            SELECT * FROM styles
            WHERE name ILIKE $1
            ORDER BY sort_order, name
            LIMIT $2
            "#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Count total styles.
    pub async fn count(&self) -> DbResult<i64> {
        let count: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM styles
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(count.unwrap_or(0))
    }

    /// Get or create a style by name.
    pub async fn get_or_create(&self, name: &str) -> DbResult<i16> {
        let id: i16 = sqlx::query_scalar(
            r#"
            INSERT INTO styles (name, is_system)
            VALUES ($1, FALSE)
            ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
            RETURNING id
            "#,
        )
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(id)
    }

    /// Update file counts for all styles.
    pub async fn refresh_file_counts(&self) -> DbResult<()> {
        sqlx::query(
            r#"
            UPDATE styles s
            SET file_count = (
                SELECT COUNT(*) FROM midi_file_styles fs
                WHERE fs.style_id = s.id
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        info!("Refreshed style file counts");
        Ok(())
    }
}

// Stub implementation when database feature is not enabled
#[cfg(not(feature = "database"))]
impl StyleRepository {
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

//! Timbre repository for VIP3 filtering operations.
//!
//! Provides CRUD operations for timbres (sound character categories)
//! and file-timbre relationships.

#[cfg(feature = "database")]
use sqlx::PgPool;
#[cfg(feature = "database")]
use tracing::info;

#[cfg(feature = "database")]
use crate::db::models::{CreateTimbre, DbError, DbResult, Timbre, TimbreWithCount, UpdateTimbre};

/// Repository for timbre operations.
#[derive(Clone)]
pub struct TimbreRepository {
    #[cfg(feature = "database")]
    pool: PgPool,
}

#[cfg(feature = "database")]
impl TimbreRepository {
    /// Create new repository with connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new timbre.
    pub async fn create(&self, timbre: CreateTimbre) -> DbResult<i16> {
        let id: i16 = sqlx::query_scalar(
            r#"
            INSERT INTO timbres (name, sort_order, is_system)
            VALUES ($1, $2, $3)
            ON CONFLICT (name) DO UPDATE SET sort_order = EXCLUDED.sort_order
            RETURNING id
            "#,
        )
        .bind(&timbre.name)
        .bind(timbre.sort_order)
        .bind(timbre.is_system)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        info!("Created timbre '{}' with id {}", timbre.name, id);
        Ok(id)
    }

    /// Get timbre by ID.
    pub async fn find_by_id(&self, id: i16) -> DbResult<Option<Timbre>> {
        sqlx::query_as::<_, Timbre>(
            r#"
            SELECT * FROM timbres
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Get timbre by name.
    pub async fn find_by_name(&self, name: &str) -> DbResult<Option<Timbre>> {
        sqlx::query_as::<_, Timbre>(
            r#"
            SELECT * FROM timbres
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Update a timbre.
    pub async fn update(&self, id: i16, update: UpdateTimbre) -> DbResult<bool> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE timbres
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

    /// Delete a timbre (only non-system timbres).
    pub async fn delete(&self, id: i16) -> DbResult<bool> {
        // First remove all file associations
        sqlx::query(
            r#"
            DELETE FROM midi_file_timbres
            WHERE timbre_id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        // Then delete the timbre (only if not system)
        let rows_affected = sqlx::query(
            r#"
            DELETE FROM timbres
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

    /// List all timbres.
    pub async fn list(&self) -> DbResult<Vec<Timbre>> {
        sqlx::query_as::<_, Timbre>(
            r#"
            SELECT * FROM timbres
            ORDER BY sort_order, name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// List all timbres with file counts.
    pub async fn list_with_counts(&self) -> DbResult<Vec<TimbreWithCount>> {
        sqlx::query_as::<_, TimbreWithCount>(
            r#"
            SELECT
                t.id,
                t.name,
                t.sort_order,
                t.is_system,
                COUNT(ft.midi_file_id) as file_count
            FROM timbres t
            LEFT JOIN midi_file_timbres ft ON t.id = ft.timbre_id
            GROUP BY t.id, t.name, t.sort_order, t.is_system
            ORDER BY t.sort_order, t.name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Add a timbre to a file.
    pub async fn add_to_file(&self, file_id: i64, timbre_id: i16) -> DbResult<bool> {
        let result = sqlx::query(
            r#"
            INSERT INTO midi_file_timbres (midi_file_id, timbre_id)
            VALUES ($1, $2)
            ON CONFLICT (midi_file_id, timbre_id) DO NOTHING
            "#,
        )
        .bind(file_id)
        .bind(timbre_id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(result.rows_affected() > 0)
    }

    /// Remove a timbre from a file.
    pub async fn remove_from_file(&self, file_id: i64, timbre_id: i16) -> DbResult<bool> {
        let rows_affected = sqlx::query(
            r#"
            DELETE FROM midi_file_timbres
            WHERE midi_file_id = $1 AND timbre_id = $2
            "#,
        )
        .bind(file_id)
        .bind(timbre_id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    /// Get all timbres for a file.
    pub async fn get_file_timbres(&self, file_id: i64) -> DbResult<Vec<Timbre>> {
        sqlx::query_as::<_, Timbre>(
            r#"
            SELECT t.* FROM timbres t
            JOIN midi_file_timbres ft ON t.id = ft.timbre_id
            WHERE ft.midi_file_id = $1
            ORDER BY t.sort_order, t.name
            "#,
        )
        .bind(file_id)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Get file IDs with a specific timbre.
    pub async fn get_files_with_timbre(&self, timbre_id: i16, limit: i64) -> DbResult<Vec<i64>> {
        let ids: Vec<i64> = sqlx::query_scalar(
            r#"
            SELECT midi_file_id FROM midi_file_timbres
            WHERE timbre_id = $1
            LIMIT $2
            "#,
        )
        .bind(timbre_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(ids)
    }

    /// Batch add timbres to a file.
    pub async fn batch_add_to_file(&self, file_id: i64, timbre_ids: &[i16]) -> DbResult<usize> {
        let mut added = 0;

        for timbre_id in timbre_ids {
            let result = sqlx::query(
                r#"
                INSERT INTO midi_file_timbres (midi_file_id, timbre_id)
                VALUES ($1, $2)
                ON CONFLICT (midi_file_id, timbre_id) DO NOTHING
                "#,
            )
            .bind(file_id)
            .bind(timbre_id)
            .execute(&self.pool)
            .await
            .map_err(DbError::from)?;

            if result.rows_affected() > 0 {
                added += 1;
            }
        }

        info!("Added {} timbres to file {}", added, file_id);
        Ok(added)
    }

    /// Clear all timbres from a file.
    pub async fn clear_file_timbres(&self, file_id: i64) -> DbResult<usize> {
        let result = sqlx::query(
            r#"
            DELETE FROM midi_file_timbres
            WHERE midi_file_id = $1
            "#,
        )
        .bind(file_id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(result.rows_affected() as usize)
    }

    /// Search timbres by partial name.
    pub async fn search(&self, query: &str, limit: i64) -> DbResult<Vec<Timbre>> {
        sqlx::query_as::<_, Timbre>(
            r#"
            SELECT * FROM timbres
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

    /// Count total timbres.
    pub async fn count(&self) -> DbResult<i64> {
        let count: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM timbres
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(count.unwrap_or(0))
    }

    /// Get or create a timbre by name.
    pub async fn get_or_create(&self, name: &str) -> DbResult<i16> {
        let id: i16 = sqlx::query_scalar(
            r#"
            INSERT INTO timbres (name, is_system)
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

    /// Update file counts for all timbres.
    pub async fn refresh_file_counts(&self) -> DbResult<()> {
        sqlx::query(
            r#"
            UPDATE timbres t
            SET file_count = (
                SELECT COUNT(*) FROM midi_file_timbres ft
                WHERE ft.timbre_id = t.id
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        info!("Refreshed timbre file counts");
        Ok(())
    }
}

// Stub implementation when database feature is not enabled
#[cfg(not(feature = "database"))]
impl TimbreRepository {
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

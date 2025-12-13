//! Articulation repository for VIP3 filtering operations.
//!
//! Provides CRUD operations for articulations (playing styles/pattern types)
//! and file-articulation relationships.

#[cfg(feature = "database")]
use sqlx::PgPool;
#[cfg(feature = "database")]
use tracing::info;

#[cfg(feature = "database")]
use crate::db::models::{
    Articulation, ArticulationWithCount, CreateArticulation, DbError, DbResult,
    UpdateArticulation,
};

/// Repository for articulation operations.
#[derive(Clone)]
pub struct ArticulationRepository {
    #[cfg(feature = "database")]
    pool: PgPool,
}

#[cfg(feature = "database")]
impl ArticulationRepository {
    /// Create new repository with connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new articulation.
    pub async fn create(&self, articulation: CreateArticulation) -> DbResult<i16> {
        let id: i16 = sqlx::query_scalar(
            r#"
            INSERT INTO articulations (name, sort_order, is_system)
            VALUES ($1, $2, $3)
            ON CONFLICT (name) DO UPDATE SET sort_order = EXCLUDED.sort_order
            RETURNING id
            "#,
        )
        .bind(&articulation.name)
        .bind(articulation.sort_order)
        .bind(articulation.is_system)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        info!("Created articulation '{}' with id {}", articulation.name, id);
        Ok(id)
    }

    /// Get articulation by ID.
    pub async fn find_by_id(&self, id: i16) -> DbResult<Option<Articulation>> {
        sqlx::query_as::<_, Articulation>(
            r#"
            SELECT * FROM articulations
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Get articulation by name.
    pub async fn find_by_name(&self, name: &str) -> DbResult<Option<Articulation>> {
        sqlx::query_as::<_, Articulation>(
            r#"
            SELECT * FROM articulations
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Update an articulation.
    pub async fn update(&self, id: i16, update: UpdateArticulation) -> DbResult<bool> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE articulations
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

    /// Delete an articulation (only non-system articulations).
    pub async fn delete(&self, id: i16) -> DbResult<bool> {
        // First remove all file associations
        sqlx::query(
            r#"
            DELETE FROM midi_file_articulations
            WHERE articulation_id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        // Then delete the articulation (only if not system)
        let rows_affected = sqlx::query(
            r#"
            DELETE FROM articulations
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

    /// List all articulations.
    pub async fn list(&self) -> DbResult<Vec<Articulation>> {
        sqlx::query_as::<_, Articulation>(
            r#"
            SELECT * FROM articulations
            ORDER BY sort_order, name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// List all articulations with file counts.
    pub async fn list_with_counts(&self) -> DbResult<Vec<ArticulationWithCount>> {
        sqlx::query_as::<_, ArticulationWithCount>(
            r#"
            SELECT
                a.id,
                a.name,
                a.sort_order,
                a.is_system,
                COUNT(fa.midi_file_id) as file_count
            FROM articulations a
            LEFT JOIN midi_file_articulations fa ON a.id = fa.articulation_id
            GROUP BY a.id, a.name, a.sort_order, a.is_system
            ORDER BY a.sort_order, a.name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Add an articulation to a file.
    pub async fn add_to_file(&self, file_id: i64, articulation_id: i16) -> DbResult<bool> {
        let result = sqlx::query(
            r#"
            INSERT INTO midi_file_articulations (midi_file_id, articulation_id)
            VALUES ($1, $2)
            ON CONFLICT (midi_file_id, articulation_id) DO NOTHING
            "#,
        )
        .bind(file_id)
        .bind(articulation_id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(result.rows_affected() > 0)
    }

    /// Remove an articulation from a file.
    pub async fn remove_from_file(&self, file_id: i64, articulation_id: i16) -> DbResult<bool> {
        let rows_affected = sqlx::query(
            r#"
            DELETE FROM midi_file_articulations
            WHERE midi_file_id = $1 AND articulation_id = $2
            "#,
        )
        .bind(file_id)
        .bind(articulation_id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    /// Get all articulations for a file.
    pub async fn get_file_articulations(&self, file_id: i64) -> DbResult<Vec<Articulation>> {
        sqlx::query_as::<_, Articulation>(
            r#"
            SELECT a.* FROM articulations a
            JOIN midi_file_articulations fa ON a.id = fa.articulation_id
            WHERE fa.midi_file_id = $1
            ORDER BY a.sort_order, a.name
            "#,
        )
        .bind(file_id)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Get file IDs with a specific articulation.
    pub async fn get_files_with_articulation(
        &self,
        articulation_id: i16,
        limit: i64,
    ) -> DbResult<Vec<i64>> {
        let ids: Vec<i64> = sqlx::query_scalar(
            r#"
            SELECT midi_file_id FROM midi_file_articulations
            WHERE articulation_id = $1
            LIMIT $2
            "#,
        )
        .bind(articulation_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(ids)
    }

    /// Batch add articulations to a file.
    pub async fn batch_add_to_file(
        &self,
        file_id: i64,
        articulation_ids: &[i16],
    ) -> DbResult<usize> {
        let mut added = 0;

        for articulation_id in articulation_ids {
            let result = sqlx::query(
                r#"
                INSERT INTO midi_file_articulations (midi_file_id, articulation_id)
                VALUES ($1, $2)
                ON CONFLICT (midi_file_id, articulation_id) DO NOTHING
                "#,
            )
            .bind(file_id)
            .bind(articulation_id)
            .execute(&self.pool)
            .await
            .map_err(DbError::from)?;

            if result.rows_affected() > 0 {
                added += 1;
            }
        }

        info!("Added {} articulations to file {}", added, file_id);
        Ok(added)
    }

    /// Clear all articulations from a file.
    pub async fn clear_file_articulations(&self, file_id: i64) -> DbResult<usize> {
        let result = sqlx::query(
            r#"
            DELETE FROM midi_file_articulations
            WHERE midi_file_id = $1
            "#,
        )
        .bind(file_id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(result.rows_affected() as usize)
    }

    /// Search articulations by partial name.
    pub async fn search(&self, query: &str, limit: i64) -> DbResult<Vec<Articulation>> {
        sqlx::query_as::<_, Articulation>(
            r#"
            SELECT * FROM articulations
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

    /// Count total articulations.
    pub async fn count(&self) -> DbResult<i64> {
        let count: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM articulations
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(count.unwrap_or(0))
    }

    /// Get or create an articulation by name.
    pub async fn get_or_create(&self, name: &str) -> DbResult<i16> {
        let id: i16 = sqlx::query_scalar(
            r#"
            INSERT INTO articulations (name, is_system)
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

    /// Update file counts for all articulations.
    pub async fn refresh_file_counts(&self) -> DbResult<()> {
        sqlx::query(
            r#"
            UPDATE articulations a
            SET file_count = (
                SELECT COUNT(*) FROM midi_file_articulations fa
                WHERE fa.articulation_id = a.id
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        info!("Refreshed articulation file counts");
        Ok(())
    }

    /// Get loop-type articulations.
    pub async fn get_loop_types(&self) -> DbResult<Vec<Articulation>> {
        sqlx::query_as::<_, Articulation>(
            r#"
            SELECT * FROM articulations
            WHERE LOWER(name) IN ('loop', 'groove', 'pattern', 'sequence')
            ORDER BY sort_order, name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Get one-shot type articulations.
    pub async fn get_oneshot_types(&self) -> DbResult<Vec<Articulation>> {
        sqlx::query_as::<_, Articulation>(
            r#"
            SELECT * FROM articulations
            WHERE LOWER(name) IN ('one-shot', 'stab', 'hit')
            ORDER BY sort_order, name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }
}

// Stub implementation when database feature is not enabled
#[cfg(not(feature = "database"))]
impl ArticulationRepository {
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

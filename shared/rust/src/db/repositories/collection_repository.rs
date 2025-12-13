//! Collection repository for user playlist/collection operations.
//!
//! Provides CRUD operations for collections and file-collection relationships.

#[cfg(feature = "database")]
use sqlx::PgPool;
#[cfg(feature = "database")]
use tracing::info;

#[cfg(feature = "database")]
use crate::db::models::{
    Collection, CollectionFile, CollectionWithCount, CreateCollection, DbError,
    DbResult, UpdateCollection,
};

/// Repository for collection operations.
#[derive(Clone)]
pub struct CollectionRepository {
    #[cfg(feature = "database")]
    pool: PgPool,
}

#[cfg(feature = "database")]
impl CollectionRepository {
    /// Create new repository with connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new collection.
    pub async fn create(&self, collection: CreateCollection) -> DbResult<i64> {
        let id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO collections (name, description, icon, color)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
        )
        .bind(&collection.name)
        .bind(&collection.description)
        .bind(&collection.icon)
        .bind(&collection.color)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        info!("Created collection '{}' with id {}", collection.name, id);
        Ok(id)
    }

    /// Get collection by ID.
    pub async fn find_by_id(&self, id: i64) -> DbResult<Option<Collection>> {
        sqlx::query_as::<_, Collection>(
            r#"
            SELECT * FROM collections
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Get collection by name.
    pub async fn find_by_name(&self, name: &str) -> DbResult<Option<Collection>> {
        sqlx::query_as::<_, Collection>(
            r#"
            SELECT * FROM collections
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Update a collection.
    pub async fn update(&self, id: i64, update: UpdateCollection) -> DbResult<bool> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE collections
            SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                icon = COALESCE($4, icon),
                color = COALESCE($5, color),
                modified_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(&update.name)
        .bind(&update.description)
        .bind(&update.icon)
        .bind(&update.color)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    /// Delete a collection.
    pub async fn delete(&self, id: i64) -> DbResult<bool> {
        // First remove all file associations
        sqlx::query(
            r#"
            DELETE FROM collection_files
            WHERE collection_id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        // Then delete the collection
        let rows_affected = sqlx::query(
            r#"
            DELETE FROM collections
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

    /// List all collections.
    pub async fn list(&self) -> DbResult<Vec<Collection>> {
        sqlx::query_as::<_, Collection>(
            r#"
            SELECT * FROM collections
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// List all collections with file counts.
    pub async fn list_with_counts(&self) -> DbResult<Vec<CollectionWithCount>> {
        sqlx::query_as::<_, CollectionWithCount>(
            r#"
            SELECT
                c.id,
                c.name,
                c.description,
                c.icon,
                c.color,
                COUNT(cf.midi_file_id) as file_count,
                c.created_at,
                c.modified_at
            FROM collections c
            LEFT JOIN collection_files cf ON c.id = cf.collection_id
            GROUP BY c.id, c.name, c.description, c.icon, c.color, c.created_at, c.modified_at
            ORDER BY c.name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Add a file to a collection.
    pub async fn add_file(&self, collection_id: i64, file_id: i64) -> DbResult<bool> {
        // Get the next sort order
        let max_order: Option<i32> = sqlx::query_scalar(
            r#"
            SELECT MAX(sort_order) FROM collection_files
            WHERE collection_id = $1
            "#,
        )
        .bind(collection_id)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        let sort_order = max_order.unwrap_or(0) + 1;

        let result = sqlx::query(
            r#"
            INSERT INTO collection_files (collection_id, midi_file_id, sort_order)
            VALUES ($1, $2, $3)
            ON CONFLICT (collection_id, midi_file_id) DO NOTHING
            "#,
        )
        .bind(collection_id)
        .bind(file_id)
        .bind(sort_order)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        // Update modified_at
        if result.rows_affected() > 0 {
            sqlx::query(
                r#"
                UPDATE collections SET modified_at = NOW() WHERE id = $1
                "#,
            )
            .bind(collection_id)
            .execute(&self.pool)
            .await
            .map_err(DbError::from)?;
        }

        Ok(result.rows_affected() > 0)
    }

    /// Remove a file from a collection.
    pub async fn remove_file(&self, collection_id: i64, file_id: i64) -> DbResult<bool> {
        let rows_affected = sqlx::query(
            r#"
            DELETE FROM collection_files
            WHERE collection_id = $1 AND midi_file_id = $2
            "#,
        )
        .bind(collection_id)
        .bind(file_id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?
        .rows_affected();

        // Update modified_at
        if rows_affected > 0 {
            sqlx::query(
                r#"
                UPDATE collections SET modified_at = NOW() WHERE id = $1
                "#,
            )
            .bind(collection_id)
            .execute(&self.pool)
            .await
            .map_err(DbError::from)?;
        }

        Ok(rows_affected > 0)
    }

    /// Get all file IDs in a collection (ordered).
    pub async fn get_file_ids(&self, collection_id: i64) -> DbResult<Vec<i64>> {
        let ids: Vec<i64> = sqlx::query_scalar(
            r#"
            SELECT midi_file_id FROM collection_files
            WHERE collection_id = $1
            ORDER BY sort_order, added_at
            "#,
        )
        .bind(collection_id)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(ids)
    }

    /// Get all collection-file relationships for a collection.
    pub async fn get_collection_files(&self, collection_id: i64) -> DbResult<Vec<CollectionFile>> {
        sqlx::query_as::<_, CollectionFile>(
            r#"
            SELECT * FROM collection_files
            WHERE collection_id = $1
            ORDER BY sort_order, added_at
            "#,
        )
        .bind(collection_id)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Get collections containing a specific file.
    pub async fn get_file_collections(&self, file_id: i64) -> DbResult<Vec<Collection>> {
        sqlx::query_as::<_, Collection>(
            r#"
            SELECT c.* FROM collections c
            JOIN collection_files cf ON c.id = cf.collection_id
            WHERE cf.midi_file_id = $1
            ORDER BY c.name
            "#,
        )
        .bind(file_id)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Batch add files to a collection.
    pub async fn batch_add_files(
        &self,
        collection_id: i64,
        file_ids: &[i64],
    ) -> DbResult<usize> {
        let mut added = 0;

        for file_id in file_ids {
            if self.add_file(collection_id, *file_id).await? {
                added += 1;
            }
        }

        info!("Added {} files to collection {}", added, collection_id);
        Ok(added)
    }

    /// Clear all files from a collection.
    pub async fn clear_files(&self, collection_id: i64) -> DbResult<usize> {
        let result = sqlx::query(
            r#"
            DELETE FROM collection_files
            WHERE collection_id = $1
            "#,
        )
        .bind(collection_id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        // Update modified_at
        sqlx::query(
            r#"
            UPDATE collections SET modified_at = NOW() WHERE id = $1
            "#,
        )
        .bind(collection_id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(result.rows_affected() as usize)
    }

    /// Reorder files in a collection.
    pub async fn reorder_files(
        &self,
        collection_id: i64,
        file_ids_in_order: &[i64],
    ) -> DbResult<()> {
        for (idx, file_id) in file_ids_in_order.iter().enumerate() {
            sqlx::query(
                r#"
                UPDATE collection_files
                SET sort_order = $3
                WHERE collection_id = $1 AND midi_file_id = $2
                "#,
            )
            .bind(collection_id)
            .bind(file_id)
            .bind(idx as i32)
            .execute(&self.pool)
            .await
            .map_err(DbError::from)?;
        }

        // Update modified_at
        sqlx::query(
            r#"
            UPDATE collections SET modified_at = NOW() WHERE id = $1
            "#,
        )
        .bind(collection_id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        info!("Reordered {} files in collection {}", file_ids_in_order.len(), collection_id);
        Ok(())
    }

    /// Count total collections.
    pub async fn count(&self) -> DbResult<i64> {
        let count: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM collections
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(count.unwrap_or(0))
    }

    /// Count files in a collection.
    pub async fn count_files(&self, collection_id: i64) -> DbResult<i64> {
        let count: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM collection_files
            WHERE collection_id = $1
            "#,
        )
        .bind(collection_id)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(count.unwrap_or(0))
    }

    /// Search collections by name.
    pub async fn search(&self, query: &str, limit: i64) -> DbResult<Vec<Collection>> {
        sqlx::query_as::<_, Collection>(
            r#"
            SELECT * FROM collections
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

    /// Duplicate a collection.
    pub async fn duplicate(&self, id: i64, new_name: &str) -> DbResult<i64> {
        // Get the original collection
        let original = self.find_by_id(id).await?.ok_or_else(|| {
            DbError::NotFound(format!("Collection {} not found", id))
        })?;

        // Create the new collection
        let new_id = self
            .create(CreateCollection {
                name: new_name.to_string(),
                description: original.description,
                icon: original.icon,
                color: original.color,
            })
            .await?;

        // Copy all file associations
        sqlx::query(
            r#"
            INSERT INTO collection_files (collection_id, midi_file_id, sort_order)
            SELECT $2, midi_file_id, sort_order FROM collection_files
            WHERE collection_id = $1
            "#,
        )
        .bind(id)
        .bind(new_id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        info!("Duplicated collection {} as {} (id: {})", id, new_name, new_id);
        Ok(new_id)
    }
}

// Stub implementation when database feature is not enabled
#[cfg(not(feature = "database"))]
impl CollectionRepository {
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

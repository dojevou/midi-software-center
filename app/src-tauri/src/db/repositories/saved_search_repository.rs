//! Saved search repository for VIP3 filtering operations.
//!
//! Provides CRUD operations for saved searches (user filter presets)
//! and recent searches (search history).

#[cfg(feature = "database")]
use sqlx::PgPool;
#[cfg(feature = "database")]
use tracing::info;

#[cfg(feature = "database")]
use crate::db::models::{
    CreateRecentSearch, CreateSavedSearch, DbError, DbResult, RecentSearch, SavedSearch,
    UpdateSavedSearch,
};

/// Repository for saved search operations.
#[derive(Clone)]
pub struct SavedSearchRepository {
    #[cfg(feature = "database")]
    pool: PgPool,
}

#[cfg(feature = "database")]
impl SavedSearchRepository {
    /// Create new repository with connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ========================================================================
    // Saved Search Operations
    // ========================================================================

    /// Create a new saved search.
    pub async fn create(&self, saved_search: CreateSavedSearch) -> DbResult<i64> {
        let id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO saved_searches (name, filters, sort_by, sort_order)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
        )
        .bind(&saved_search.name)
        .bind(&saved_search.filters)
        .bind(&saved_search.sort_by)
        .bind(&saved_search.sort_order)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        info!(
            "Created saved search '{}' with id {}",
            saved_search.name, id
        );
        Ok(id)
    }

    /// Get saved search by ID.
    pub async fn find_by_id(&self, id: i64) -> DbResult<Option<SavedSearch>> {
        sqlx::query_as::<_, SavedSearch>(
            r#"
            SELECT * FROM saved_searches
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Get saved search by name.
    pub async fn find_by_name(&self, name: &str) -> DbResult<Option<SavedSearch>> {
        sqlx::query_as::<_, SavedSearch>(
            r#"
            SELECT * FROM saved_searches
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Update a saved search.
    pub async fn update(&self, id: i64, update: UpdateSavedSearch) -> DbResult<bool> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE saved_searches
            SET
                name = COALESCE($2, name),
                filters = COALESCE($3, filters),
                sort_by = COALESCE($4, sort_by),
                sort_order = COALESCE($5, sort_order)
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(&update.name)
        .bind(&update.filters)
        .bind(&update.sort_by)
        .bind(&update.sort_order)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    /// Delete a saved search.
    pub async fn delete(&self, id: i64) -> DbResult<bool> {
        let rows_affected = sqlx::query(
            r#"
            DELETE FROM saved_searches
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

    /// List all saved searches.
    pub async fn list(&self) -> DbResult<Vec<SavedSearch>> {
        sqlx::query_as::<_, SavedSearch>(
            r#"
            SELECT * FROM saved_searches
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// List saved searches ordered by creation time (most recent first).
    pub async fn list_by_creation(&self, limit: i64) -> DbResult<Vec<SavedSearch>> {
        sqlx::query_as::<_, SavedSearch>(
            r#"
            SELECT * FROM saved_searches
            ORDER BY created_at DESC NULLS LAST, name
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Search saved searches by name.
    pub async fn search(&self, query: &str, limit: i64) -> DbResult<Vec<SavedSearch>> {
        sqlx::query_as::<_, SavedSearch>(
            r#"
            SELECT * FROM saved_searches
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

    /// Count total saved searches.
    pub async fn count(&self) -> DbResult<i64> {
        let count: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM saved_searches
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(count.unwrap_or(0))
    }

    /// Duplicate a saved search.
    pub async fn duplicate(&self, id: i64, new_name: &str) -> DbResult<i64> {
        let original = self
            .find_by_id(id)
            .await?
            .ok_or_else(|| DbError::NotFound(format!("Saved search {} not found", id)))?;

        let new_id = self
            .create(CreateSavedSearch {
                name: new_name.to_string(),
                filters: original.filters,
                sort_by: original.sort_by,
                sort_order: original.sort_order,
            })
            .await?;

        info!(
            "Duplicated saved search {} as {} (id: {})",
            id, new_name, new_id
        );
        Ok(new_id)
    }

    // ========================================================================
    // Recent Search Operations
    // ========================================================================

    /// Add a recent search.
    pub async fn add_recent(&self, recent: CreateRecentSearch) -> DbResult<i64> {
        let id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO recent_searches (query, filters)
            VALUES ($1, $2)
            RETURNING id
            "#,
        )
        .bind(&recent.query)
        .bind(&recent.filters)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(id)
    }

    /// Get recent searches.
    pub async fn get_recent(&self, limit: i64) -> DbResult<Vec<RecentSearch>> {
        sqlx::query_as::<_, RecentSearch>(
            r#"
            SELECT * FROM recent_searches
            ORDER BY searched_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Clear all recent searches.
    pub async fn clear_recent(&self) -> DbResult<usize> {
        let result = sqlx::query(
            r#"
            DELETE FROM recent_searches
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        info!("Cleared {} recent searches", result.rows_affected());
        Ok(result.rows_affected() as usize)
    }

    /// Delete a specific recent search.
    pub async fn delete_recent(&self, id: i64) -> DbResult<bool> {
        let rows_affected = sqlx::query(
            r#"
            DELETE FROM recent_searches
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

    /// Prune old recent searches, keeping only the most recent N.
    pub async fn prune_recent(&self, keep_count: i64) -> DbResult<usize> {
        let result = sqlx::query(
            r#"
            DELETE FROM recent_searches
            WHERE id NOT IN (
                SELECT id FROM recent_searches
                ORDER BY searched_at DESC
                LIMIT $1
            )
            "#,
        )
        .bind(keep_count)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        if result.rows_affected() > 0 {
            info!("Pruned {} old recent searches", result.rows_affected());
        }
        Ok(result.rows_affected() as usize)
    }

    /// Search recent searches by query text.
    pub async fn search_recent(&self, query: &str, limit: i64) -> DbResult<Vec<RecentSearch>> {
        sqlx::query_as::<_, RecentSearch>(
            r#"
            SELECT * FROM recent_searches
            WHERE query ILIKE $1
            ORDER BY searched_at DESC
            LIMIT $2
            "#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Count recent searches.
    pub async fn count_recent(&self) -> DbResult<i64> {
        let count: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM recent_searches
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(count.unwrap_or(0))
    }

    /// Convert a recent search to a saved search.
    pub async fn save_recent(&self, recent_id: i64, name: &str) -> DbResult<i64> {
        let recent = sqlx::query_as::<_, RecentSearch>(
            r#"
            SELECT * FROM recent_searches
            WHERE id = $1
            "#,
        )
        .bind(recent_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)?
        .ok_or_else(|| DbError::NotFound(format!("Recent search {} not found", recent_id)))?;

        let saved_id = self
            .create(CreateSavedSearch {
                name: name.to_string(),
                filters: recent.filters.unwrap_or_default(),
                sort_by: None,
                sort_order: None,
            })
            .await?;

        info!(
            "Converted recent search {} to saved search {} (id: {})",
            recent_id, name, saved_id
        );
        Ok(saved_id)
    }
}

// Stub implementation when database feature is not enabled
#[cfg(not(feature = "database"))]
impl SavedSearchRepository {
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

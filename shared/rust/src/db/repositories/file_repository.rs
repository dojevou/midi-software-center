//! File repository for MIDI file database operations.
//!
//! Provides CRUD operations, batch inserts, deduplication,
//! and efficient querying for the files table.

#[cfg(feature = "database")]
use sqlx::PgPool;
#[cfg(feature = "database")]
use tracing::{info, warn};
#[cfg(feature = "database")]
use uuid::Uuid;

#[cfg(feature = "database")]
use crate::db::models::{CreateMidiFile, DbError, DbResult, MidiFile, UpdateMidiFile};

/// Repository for file operations with deduplication and batch insert support.
#[derive(Clone)]
pub struct FileRepository {
    #[cfg(feature = "database")]
    pool: PgPool,
}

#[cfg(feature = "database")]
impl FileRepository {
    /// Create new repository with connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Insert single file with deduplication check.
    pub async fn insert(&self, new_file: CreateMidiFile) -> DbResult<i64> {
        let mut tx = self.pool.begin().await.map_err(DbError::from)?;

        // Check for duplicate by content hash
        let duplicate_id: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT id FROM files
            WHERE content_hash = $1
            LIMIT 1
            "#,
        )
        .bind(&new_file.content_hash)
        .fetch_optional(&mut *tx)
        .await
        .map_err(DbError::from)?;

        if let Some(id) = duplicate_id {
            warn!(
                "Duplicate file found with content hash: {}",
                hex::encode(&new_file.content_hash)
            );
            tx.rollback().await.map_err(DbError::from)?;
            return Err(DbError::Duplicate(format!(
                "File already exists with id: {}",
                id
            )));
        }

        // Check for duplicate by filepath
        let duplicate_path_id: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT id FROM files
            WHERE filepath = $1
            LIMIT 1
            "#,
        )
        .bind(&new_file.filepath)
        .fetch_optional(&mut *tx)
        .await
        .map_err(DbError::from)?;

        if duplicate_path_id.is_some() {
            warn!("Duplicate filepath found: {}", new_file.filepath);
            tx.rollback().await.map_err(DbError::from)?;
            return Err(DbError::Duplicate(format!(
                "Filepath already exists: {}",
                new_file.filepath
            )));
        }

        // Insert the file
        let file_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO files (
                filename, filepath, original_filename, content_hash,
                file_size_bytes, format, num_tracks, ticks_per_quarter_note,
                import_batch_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id
            "#,
        )
        .bind(&new_file.filename)
        .bind(&new_file.filepath)
        .bind(&new_file.original_filename)
        .bind(&new_file.content_hash)
        .bind(new_file.file_size_bytes)
        .bind(new_file.format)
        .bind(new_file.num_tracks)
        .bind(new_file.ticks_per_quarter_note)
        .bind(new_file.import_batch_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(DbError::from)?;

        tx.commit().await.map_err(DbError::from)?;
        info!("Inserted file {} with id {}", new_file.filename, file_id);

        Ok(file_id)
    }

    /// Batch insert files with transactional safety.
    pub async fn batch_insert(&self, files: Vec<CreateMidiFile>) -> DbResult<Vec<i64>> {
        let mut tx = self.pool.begin().await.map_err(DbError::from)?;
        let mut inserted_ids = Vec::with_capacity(files.len());

        for file in files {
            let duplicate_id: Option<i64> = sqlx::query_scalar(
                r#"
                SELECT id FROM files
                WHERE content_hash = $1
                LIMIT 1
                "#,
            )
            .bind(&file.content_hash)
            .fetch_optional(&mut *tx)
            .await
            .map_err(DbError::from)?;

            if duplicate_id.is_some() {
                warn!(
                    "Skipping duplicate file with hash: {}",
                    hex::encode(&file.content_hash)
                );
                continue;
            }

            let file_id: Option<i64> = sqlx::query_scalar(
                r#"
                INSERT INTO files (
                    filename, filepath, original_filename, content_hash,
                    file_size_bytes, format, num_tracks, ticks_per_quarter_note,
                    import_batch_id
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ON CONFLICT (content_hash) DO NOTHING
                RETURNING id
                "#,
            )
            .bind(&file.filename)
            .bind(&file.filepath)
            .bind(&file.original_filename)
            .bind(&file.content_hash)
            .bind(file.file_size_bytes)
            .bind(file.format)
            .bind(file.num_tracks)
            .bind(file.ticks_per_quarter_note)
            .bind(file.import_batch_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(DbError::from)?;

            if let Some(id) = file_id {
                inserted_ids.push(id);
            }
        }

        tx.commit().await.map_err(DbError::from)?;
        info!("Batch inserted {} files", inserted_ids.len());

        Ok(inserted_ids)
    }

    /// Get file by ID.
    pub async fn find_by_id(&self, id: i64) -> DbResult<Option<MidiFile>> {
        sqlx::query_as::<_, MidiFile>(
            r#"
            SELECT * FROM files
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Get file by content hash.
    pub async fn find_by_hash(&self, hash: &[u8]) -> DbResult<Option<MidiFile>> {
        sqlx::query_as::<_, MidiFile>(
            r#"
            SELECT * FROM files
            WHERE content_hash = $1
            "#,
        )
        .bind(hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Get file by filepath.
    pub async fn find_by_filepath(&self, filepath: &str) -> DbResult<Option<MidiFile>> {
        sqlx::query_as::<_, MidiFile>(
            r#"
            SELECT * FROM files
            WHERE filepath = $1
            "#,
        )
        .bind(filepath)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Update file metadata.
    pub async fn update(&self, id: i64, update: UpdateMidiFile) -> DbResult<bool> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE files
            SET
                filename = COALESCE($2, filename),
                filepath = COALESCE($3, filepath),
                format = COALESCE($4, format),
                num_tracks = COALESCE($5, num_tracks),
                ticks_per_quarter_note = COALESCE($6, ticks_per_quarter_note),
                duration_seconds = COALESCE($7, duration_seconds),
                duration_ticks = COALESCE($8, duration_ticks),
                manufacturer = COALESCE($9, manufacturer),
                collection_name = COALESCE($10, collection_name),
                folder_tags = COALESCE($11, folder_tags),
                analyzed_at = COALESCE($12, analyzed_at),
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(&update.filename)
        .bind(&update.filepath)
        .bind(update.format)
        .bind(update.num_tracks)
        .bind(update.ticks_per_quarter_note)
        .bind(update.duration_seconds)
        .bind(update.duration_ticks)
        .bind(&update.manufacturer)
        .bind(&update.collection_name)
        .bind(&update.folder_tags)
        .bind(update.analyzed_at)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    /// Delete file by ID.
    pub async fn delete(&self, id: i64) -> DbResult<bool> {
        let rows_affected = sqlx::query(
            r#"
            DELETE FROM files
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

    /// Get paginated list of files.
    pub async fn list(&self, limit: i64, offset: i64) -> DbResult<Vec<MidiFile>> {
        sqlx::query_as::<_, MidiFile>(
            r#"
            SELECT * FROM files
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Count total files.
    pub async fn count(&self) -> DbResult<i64> {
        let count: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM files
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(count.unwrap_or(0))
    }

    /// Mark file as analyzed.
    pub async fn mark_analyzed(&self, id: i64) -> DbResult<bool> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE files
            SET analyzed_at = NOW(), updated_at = NOW()
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

    /// Get files by import batch.
    pub async fn get_by_import_batch(&self, batch_id: Uuid) -> DbResult<Vec<MidiFile>> {
        sqlx::query_as::<_, MidiFile>(
            r#"
            SELECT * FROM files
            WHERE import_batch_id = $1
            ORDER BY filename
            "#,
        )
        .bind(batch_id)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Check if file exists by hash.
    pub async fn exists_by_hash(&self, hash: &[u8]) -> DbResult<bool> {
        let count: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM files
            WHERE content_hash = $1
            "#,
        )
        .bind(hash)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(count.unwrap_or(0) > 0)
    }

    /// Search files by partial filename.
    pub async fn search_by_name(&self, query: &str, limit: i64) -> DbResult<Vec<MidiFile>> {
        sqlx::query_as::<_, MidiFile>(
            r#"
            SELECT * FROM files
            WHERE filename ILIKE $1 OR original_filename ILIKE $1
            ORDER BY filename
            LIMIT $2
            "#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Get files not yet analyzed.
    pub async fn get_unanalyzed(&self, limit: i64) -> DbResult<Vec<MidiFile>> {
        sqlx::query_as::<_, MidiFile>(
            r#"
            SELECT * FROM files
            WHERE analyzed_at IS NULL
            ORDER BY created_at ASC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Get child tracks for a multi-track parent file.
    pub async fn get_split_tracks(&self, parent_id: i64) -> DbResult<Vec<MidiFile>> {
        sqlx::query_as::<_, MidiFile>(
            r#"
            SELECT * FROM files
            WHERE parent_file_id = $1
            ORDER BY track_number
            "#,
        )
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }
}

// Stub implementation when database feature is not enabled
#[cfg(not(feature = "database"))]
impl FileRepository {
    /// Placeholder - database feature not enabled.
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(all(test, feature = "database"))]
mod tests {
    // Integration tests would go here with a test database connection
    // For now, we just verify the struct compiles
    #[test]
    fn test_repository_creation() {
        // This is a compile-time check that the struct is valid
        // Full integration tests require a database connection
    }
}

/// File repository - CRUD operations for files table
/// Aligned with actual schema from 001_initial_schema.sql
use crate::db::models::{File, NewFile};
use sqlx::PgPool;

pub struct FileRepository;

impl FileRepository {
    /// Inserts a new file and returns its ID
    pub async fn insert(pool: &PgPool, new_file: NewFile) -> Result<i64, sqlx::Error> {
        let file_id = sqlx::query_scalar!(
            r#"
            INSERT INTO files (
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
                manufacturer,
                collection_name,
                folder_tags,
                import_batch_id,
                parent_folder,
                filename_bpm,
                filename_key,
                filename_genres,
                structure_tags,
                metadata_source,
                track_names,
                copyright,
                instrument_names_text,
                markers,
                lyrics
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25)
            RETURNING id
            "#,
            new_file.filename,
            new_file.filepath,
            new_file.original_filename,
            &new_file.content_hash[..],
            new_file.file_size_bytes,
            new_file.format,
            new_file.num_tracks,
            new_file.ticks_per_quarter_note,
            new_file.duration_seconds,
            new_file.duration_ticks,
            new_file.manufacturer,
            new_file.collection_name,
            new_file.folder_tags.as_deref(),
            new_file.import_batch_id,
            new_file.parent_folder,
            new_file.filename_bpm,
            new_file.filename_key,
            new_file.filename_genres.as_deref(),
            new_file.structure_tags.as_deref(),
            new_file.metadata_source,
            new_file.track_names.as_deref(),
            new_file.copyright,
            new_file.instrument_names_text.as_deref(),
            new_file.markers.as_deref(),
            new_file.lyrics.as_deref(),
        )
        .fetch_one(pool)
        .await?;

        Ok(file_id)
    }

    /// Finds file by ID
    pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<File>, sqlx::Error> {
        let file = sqlx::query_as!(
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
                import_batch_id,
                parent_folder,
                filename_bpm,
                filename_key,
                filename_genres,
                structure_tags,
                metadata_source,
                track_names,
                copyright,
                instrument_names_text,
                markers,
                lyrics
            FROM files WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(file)
    }

    /// Checks if file with hash already exists
    pub async fn check_duplicate(pool: &PgPool, content_hash: &[u8]) -> Result<bool, sqlx::Error> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM files
            WHERE content_hash = $1
            "#,
            content_hash
        )
        .fetch_one(pool)
        .await?;

        Ok(count > 0)
    }

    /// Finds file by hash
    pub async fn find_by_hash(
        pool: &PgPool,
        content_hash: &[u8],
    ) -> Result<Option<File>, sqlx::Error> {
        let file = sqlx::query_as!(
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
                import_batch_id,
                parent_folder,
                filename_bpm,
                filename_key,
                filename_genres,
                structure_tags,
                metadata_source,
                track_names,
                copyright,
                instrument_names_text,
                markers,
                lyrics
            FROM files WHERE content_hash = $1 LIMIT 1
            "#,
            content_hash
        )
        .fetch_optional(pool)
        .await?;

        Ok(file)
    }

    /// Finds file by filepath
    pub async fn find_by_path(pool: &PgPool, filepath: &str) -> Result<Option<File>, sqlx::Error> {
        let file = sqlx::query_as!(
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
                import_batch_id,
                parent_folder,
                filename_bpm,
                filename_key,
                filename_genres,
                structure_tags,
                metadata_source,
                track_names,
                copyright,
                instrument_names_text,
                markers,
                lyrics
            FROM files WHERE filepath = $1
            "#,
            filepath
        )
        .fetch_optional(pool)
        .await?;

        Ok(file)
    }

    /// Updates file's analyzed_at timestamp
    pub async fn mark_analyzed(pool: &PgPool, file_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE files
            SET analyzed_at = NOW(), updated_at = NOW()
            WHERE id = $1
            "#,
            file_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Updates file metadata fields
    pub async fn update_metadata_fields(
        pool: &PgPool,
        file_id: i64,
        format: Option<i16>,
        num_tracks: i16,
        ticks_per_quarter_note: Option<i32>,
        duration_seconds: Option<sqlx::types::BigDecimal>,
        duration_ticks: Option<i64>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE files
            SET
                format = $2,
                num_tracks = $3,
                ticks_per_quarter_note = $4,
                duration_seconds = $5,
                duration_ticks = $6,
                updated_at = NOW()
            WHERE id = $1
            "#,
            file_id,
            format,
            num_tracks,
            ticks_per_quarter_note,
            duration_seconds,
            duration_ticks
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Deletes file by ID
    pub async fn delete(pool: &PgPool, file_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM files WHERE id = $1", file_id).execute(pool).await?;

        Ok(())
    }

    /// Gets file count
    pub async fn count(pool: &PgPool) -> Result<i64, sqlx::Error> {
        let count = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM files"#)
            .fetch_one(pool)
            .await?;

        Ok(count)
    }

    /// Lists files with pagination
    pub async fn list(pool: &PgPool, limit: i64, offset: i64) -> Result<Vec<File>, sqlx::Error> {
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
                import_batch_id,
                parent_folder,
                filename_bpm,
                filename_key,
                filename_genres,
                structure_tags,
                metadata_source,
                track_names,
                copyright,
                instrument_names_text,
                markers,
                lyrics
            FROM files
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?;

        Ok(files)
    }

    /// Lists files by manufacturer
    pub async fn list_by_manufacturer(
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
                import_batch_id,
                parent_folder,
                filename_bpm,
                filename_key,
                filename_genres,
                structure_tags,
                metadata_source,
                track_names,
                copyright,
                instrument_names_text,
                markers,
                lyrics
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

    /// Lists files by collection
    pub async fn list_by_collection(
        pool: &PgPool,
        collection_name: &str,
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
                import_batch_id,
                parent_folder,
                filename_bpm,
                filename_key,
                filename_genres,
                structure_tags,
                metadata_source,
                track_names,
                copyright,
                instrument_names_text,
                markers,
                lyrics
            FROM files
            WHERE collection_name = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            collection_name,
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
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
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
    async fn test_insert_and_find() {
        let pool = setup_test_pool().await;

        let new_file = NewFile {
            filename: "test_file.mid".to_string(),
            filepath: "/test/test_file.mid".to_string(),
            original_filename: "test_file.mid".to_string(),
            content_hash: vec![1, 2, 3, 4, 5, 6, 7, 8],
            file_size_bytes: 1024,
            format: Some(1),
            num_tracks: 1,
            ticks_per_quarter_note: Some(480),
            duration_seconds: None,
            duration_ticks: None,
            manufacturer: None,
            collection_name: None,
            folder_tags: None,
            import_batch_id: None,
            parent_folder: None,
            filename_bpm: None,
            filename_key: None,
            filename_genres: None,
            structure_tags: None,
            metadata_source: None,
            track_names: None,
            copyright: None,
            instrument_names_text: None,
            markers: None,
            lyrics: None,
        };

        let file_id = FileRepository::insert(&pool, new_file).await.unwrap();
        assert!(file_id > 0);

        let found = FileRepository::find_by_id(&pool, file_id).await.unwrap();
        assert!(found.is_some());

        let file = found.unwrap();
        assert_eq!(file.id, file_id);
        assert_eq!(file.filename, "test_file.mid");
    }

    #[tokio::test]
    #[ignore] // Only run when database is available
    async fn test_check_duplicate() {
        let pool = setup_test_pool().await;

        let hash = vec![9, 9, 9, 9, 9, 9, 9, 9];

        // Should not exist initially
        let exists = FileRepository::check_duplicate(&pool, &hash).await.unwrap();
        assert!(!exists);

        // Insert file
        let new_file = NewFile {
            filename: "dup_test.mid".to_string(),
            filepath: "/test/dup_test.mid".to_string(),
            original_filename: "dup_test.mid".to_string(),
            content_hash: hash.clone(),
            file_size_bytes: 512,
            format: Some(1),
            num_tracks: 1,
            ticks_per_quarter_note: Some(480),
            duration_seconds: None,
            duration_ticks: None,
            manufacturer: None,
            collection_name: None,
            folder_tags: None,
            import_batch_id: None,
            parent_folder: None,
            filename_bpm: None,
            filename_key: None,
            filename_genres: None,
            structure_tags: None,
            metadata_source: None,
            track_names: None,
            copyright: None,
            instrument_names_text: None,
            markers: None,
            lyrics: None,
        };

        FileRepository::insert(&pool, new_file).await.unwrap();

        // Should exist now
        let exists = FileRepository::check_duplicate(&pool, &hash).await.unwrap();
        assert!(exists);
    }
}

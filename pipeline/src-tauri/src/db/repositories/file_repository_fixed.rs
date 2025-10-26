//! Fixed file repository - avoids custom types

use crate::db::models::{File, NewFile};
use sqlx::PgPool;

pub struct FileRepository;

impl FileRepository {
    /// Inserts a new file and returns its ID
    pub async fn insert(pool: &PgPool, new_file: NewFile) -> Result<i64, sqlx::Error> {
        let file_id = sqlx::query_scalar!(
            r#"
            INSERT INTO files (
                original_path,
                current_path,
                original_filename,
                new_filename,
                content_hash,
                file_size,
                file_modified,
                manufacturer,
                collection_name,
                folder_tags,
                import_batch_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id
            "#,
            new_file.original_path,
            new_file.current_path,
            new_file.original_filename,
            new_file.new_filename,
            &new_file.content_hash[..],
            new_file.file_size,
            new_file.file_modified,
            new_file.manufacturer,
            new_file.collection_name,
            new_file.folder_tags.as_deref(),
            new_file.import_batch_id,
        )
        .fetch_one(pool)
        .await?;

        Ok(file_id)
    }

    /// Finds file by ID using manual mapping
    pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<File>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT 
                id,
                original_path,
                current_path,
                original_filename,
                new_filename,
                content_hash,
                file_size as file_size_bytes,
                file_modified,
                is_multi_track,
                parent_file_id,
                track_number,
                total_tracks,
                manufacturer,
                collection_name,
                folder_tags,
                category::text,
                subcategory,
                auto_tags,
                user_tags,
                analyzed_at,
                import_batch_id,
                created_at,
                updated_at
            FROM files WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| File {
            id: r.id,
            original_path: r.original_path,
            current_path: r.current_path,
            original_filename: r.original_filename,
            new_filename: r.new_filename,
            content_hash: r.content_hash,
            file_size_bytes: r.file_size_bytes,
            file_modified: r.file_modified,
            is_multi_track: r.is_multi_track,
            parent_file_id: r.parent_file_id,
            track_number: r.track_number,
            total_tracks: r.total_tracks,
            manufacturer: r.manufacturer,
            collection_name: r.collection_name,
            folder_tags: r.folder_tags,
            category: r.category,
            subcategory: r.subcategory,
            auto_tags: r.auto_tags,
            user_tags: r.user_tags,
            analyzed_at: r.analyzed_at,
            import_batch_id: r.import_batch_id,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
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
}

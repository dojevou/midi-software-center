#[allow(dead_code, unused_imports, unused_variables)]
#[allow(dead_code, unused_imports, unused_variables)]
/// Custom assertion helpers for common test validations
use sqlx::PgPool;

/// Assert file exists in database
pub async fn assert_file_exists(pool: &PgPool, file_path: &str) {
    let exists: bool =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM files WHERE file_path = $1)")
            .bind(file_path)
            .fetch_one(pool)
            .await
            .expect("Failed to check file existence");

    assert!(exists, "File '{}' not found in database", file_path);
}

/// Assert file does not exist
pub async fn assert_file_not_exists(pool: &PgPool, file_path: &str) {
    let exists: bool =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM files WHERE file_path = $1)")
            .bind(file_path)
            .fetch_one(pool)
            .await
            .expect("Failed to check file existence");

    assert!(!exists, "File '{}' should not exist in database", file_path);
}

/// Assert file count matches expected
pub async fn assert_file_count(pool: &PgPool, expected: i64) {
    let actual: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM files")
        .fetch_one(pool)
        .await
        .expect("Failed to count files");

    assert_eq!(
        actual, expected,
        "Expected {} files, found {}",
        expected, actual
    );
}

/// Assert metadata exists for file
pub async fn assert_metadata_exists(pool: &PgPool, file_id: i64) {
    let exists: bool = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM musical_metadata WHERE file_id = $1)",
    )
    .bind(file_id)
    .fetch_one(pool)
    .await
    .expect("Failed to check metadata existence");

    assert!(exists, "Metadata not found for file {}", file_id);
}

/// Assert BPM is set
pub async fn assert_bpm_set(pool: &PgPool, file_id: i64) {
    let bpm: Option<f64> =
        sqlx::query_scalar::<_, Option<f64>>("SELECT bpm FROM musical_metadata WHERE file_id = $1")
            .bind(file_id)
            .fetch_one(pool)
            .await
            .expect("Failed to fetch BPM");

    assert!(bpm.is_some(), "BPM not set for file {}", file_id);
}

/// Assert tag is associated with file
pub async fn assert_file_has_tag(pool: &PgPool, file_id: i64, tag_name: &str) {
    let exists: bool = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
            SELECT 1 FROM file_tags ft
            JOIN tags t ON ft.tag_id = t.tag_id
            WHERE ft.file_id = $1 AND t.tag_name = $2
        )",
    )
    .bind(file_id)
    .bind(tag_name)
    .fetch_one(pool)
    .await
    .expect("Failed to check file-tag association");

    assert!(exists, "File {} should have tag '{}'", file_id, tag_name);
}

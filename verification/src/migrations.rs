//! Migration Verification Module
//!
//! Verifies database schema consistency and migration state.

use sqlx::PgPool;
use std::collections::HashSet;

/// Migration verification errors
#[derive(Debug, Clone)]
pub struct MigrationError {
    pub category: String,
    pub message: String,
    pub severity: MigrationErrorSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationErrorSeverity {
    Critical,
    Warning,
    Info,
}

/// Verifies database migrations and schema consistency
pub struct MigrationVerifier;

impl MigrationVerifier {
    /// Verify current schema matches expected structure
    pub async fn verify_current_schema(pool: &PgPool) -> Result<(), Vec<MigrationError>> {
        let mut errors = Vec::new();

        // Check if all expected tables exist
        let expected_tables = vec![
            "files",
            "musical_metadata",
            "tags",
            "file_tags",
            "file_instruments",
            "file_categories",
            "harmonic_patterns",
            "melodic_patterns",
            "rhythm_patterns",
        ];

        for table in expected_tables {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = $1)"
            )
            .bind(table)
            .fetch_one(pool)
            .await
            .unwrap_or(false);

            if !exists {
                errors.push(MigrationError {
                    category: "schema".to_string(),
                    message: format!("Table '{}' is missing", table),
                    severity: MigrationErrorSeverity::Critical,
                });
            }
        }

        // Verify files table columns
        let file_columns = Self::get_table_columns(pool, "files").await?;
        let required_file_columns = vec![
            "id",
            "filepath",
            "filename",
            "content_hash",
            "file_size_bytes",
            "created_at",
            "updated_at",
        ];

        for col in required_file_columns {
            if !file_columns.contains(col) {
                errors.push(MigrationError {
                    category: "schema".to_string(),
                    message: format!("Column 'files.{}' is missing", col),
                    severity: MigrationErrorSeverity::Critical,
                });
            }
        }

        // Verify musical_metadata columns
        let metadata_columns = Self::get_table_columns(pool, "musical_metadata").await?;
        let required_metadata_columns = vec![
            "file_id",
            "bpm",
            "key_signature",
            "time_signature_numerator",
            "time_signature_denominator",
        ];

        for col in required_metadata_columns {
            if !metadata_columns.contains(col) {
                errors.push(MigrationError {
                    category: "schema".to_string(),
                    message: format!("Column 'musical_metadata.{}' is missing", col),
                    severity: MigrationErrorSeverity::Critical,
                });
            }
        }

        // Verify row counts make sense
        let file_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files")
            .fetch_one(pool)
            .await
            .unwrap_or(0);

        if file_count == 0 {
            errors.push(MigrationError {
                category: "data".to_string(),
                message: "No files in database - migration may have failed or import not run"
                    .to_string(),
                severity: MigrationErrorSeverity::Warning,
            });
        }

        // Check for orphaned records
        let orphan_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM musical_metadata m
             LEFT JOIN files f ON m.file_id = f.id
             WHERE f.id IS NULL",
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        if orphan_count > 0 {
            errors.push(MigrationError {
                category: "integrity".to_string(),
                message: format!("Found {} orphaned metadata records", orphan_count),
                severity: MigrationErrorSeverity::Warning,
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get all column names for a table
    async fn get_table_columns(
        pool: &PgPool,
        table_name: &str,
    ) -> Result<HashSet<String>, Vec<MigrationError>> {
        let columns: Vec<String> = sqlx::query_scalar(
            "SELECT column_name FROM information_schema.columns
             WHERE table_schema = 'public' AND table_name = $1",
        )
        .bind(table_name)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            vec![MigrationError {
                category: "database".to_string(),
                message: format!("Failed to query columns for {}: {}", table_name, e),
                severity: MigrationErrorSeverity::Critical,
            }]
        })?;

        Ok(columns.into_iter().collect())
    }

    /// Verify all required indexes exist
    pub async fn verify_indexes(pool: &PgPool) -> Result<(), Vec<MigrationError>> {
        let mut errors = Vec::new();

        let required_indexes = vec![
            ("files", "idx_files_filepath"),
            ("files", "idx_files_content_hash"),
            ("files", "idx_files_filename_lower"),
            ("musical_metadata", "musical_metadata_pkey"),
            ("musical_metadata", "idx_metadata_bpm"),
            ("file_tags", "idx_file_tags_file"),
            ("file_tags", "idx_file_tags_tag"),
        ];

        for (table, index) in required_indexes {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT FROM pg_indexes WHERE schemaname = 'public' AND indexname = $1)"
            )
            .bind(index)
            .fetch_one(pool)
            .await
            .unwrap_or(false);

            if !exists {
                errors.push(MigrationError {
                    category: "performance".to_string(),
                    message: format!("Index '{}' on table '{}' is missing", index, table),
                    severity: MigrationErrorSeverity::Warning,
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Verify foreign key constraints
    pub async fn verify_foreign_keys(pool: &PgPool) -> Result<(), Vec<MigrationError>> {
        let mut errors = Vec::new();

        let expected_fks = vec![
            ("musical_metadata", "file_id", "files", "id"),
            ("file_tags", "file_id", "files", "id"),
            ("file_tags", "tag_id", "tags", "id"),
            ("file_instruments", "file_id", "files", "id"),
            ("file_categories", "file_id", "files", "id"),
        ];

        for (table, column, ref_table, ref_column) in expected_fks {
            let exists: bool = sqlx::query_scalar(
                r#"
                SELECT EXISTS(
                    SELECT 1 FROM information_schema.table_constraints tc
                    JOIN information_schema.key_column_usage kcu
                        ON tc.constraint_name = kcu.constraint_name
                    JOIN information_schema.constraint_column_usage ccu
                        ON tc.constraint_name = ccu.constraint_name
                    WHERE tc.constraint_type = 'FOREIGN KEY'
                    AND tc.table_name = $1
                    AND kcu.column_name = $2
                    AND ccu.table_name = $3
                    AND ccu.column_name = $4
                )
                "#,
            )
            .bind(table)
            .bind(column)
            .bind(ref_table)
            .bind(ref_column)
            .fetch_one(pool)
            .await
            .unwrap_or(false);

            if !exists {
                errors.push(MigrationError {
                    category: "integrity".to_string(),
                    message: format!(
                        "Foreign key {}.{} -> {}.{} is missing",
                        table, column, ref_table, ref_column
                    ),
                    severity: MigrationErrorSeverity::Warning,
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get migration verification summary
    pub async fn get_summary(pool: &PgPool) -> MigrationSummary {
        let schema_valid = Self::verify_current_schema(pool).await.is_ok();
        let indexes_valid = Self::verify_indexes(pool).await.is_ok();
        let foreign_keys_valid = Self::verify_foreign_keys(pool).await.is_ok();

        MigrationSummary { schema_valid, indexes_valid, foreign_keys_valid }
    }
}

/// Summary of migration state
#[derive(Debug, Clone)]
pub struct MigrationSummary {
    pub schema_valid: bool,
    pub indexes_valid: bool,
    pub foreign_keys_valid: bool,
}

impl MigrationSummary {
    pub fn all_valid(&self) -> bool {
        self.schema_valid && self.indexes_valid && self.foreign_keys_valid
    }
}

impl std::fmt::Display for MigrationSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Migration Summary:\n  Schema: {}\n  Indexes: {}\n  Foreign Keys: {}",
            if self.schema_valid { "OK" } else { "FAILED" },
            if self.indexes_valid { "OK" } else { "FAILED" },
            if self.foreign_keys_valid {
                "OK"
            } else {
                "FAILED"
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_error_severity() {
        let error = MigrationError {
            category: "test".to_string(),
            message: "test error".to_string(),
            severity: MigrationErrorSeverity::Critical,
        };
        assert_eq!(error.severity, MigrationErrorSeverity::Critical);
    }

    #[test]
    fn test_migration_summary_display() {
        let summary =
            MigrationSummary { schema_valid: true, indexes_valid: true, foreign_keys_valid: false };
        let display = format!("{}", summary);
        assert!(display.contains("Schema: OK"));
        assert!(display.contains("Foreign Keys: FAILED"));
    }

    #[test]
    fn test_migration_summary_all_valid() {
        let summary =
            MigrationSummary { schema_valid: true, indexes_valid: true, foreign_keys_valid: true };
        assert!(summary.all_valid());

        let partial =
            MigrationSummary { schema_valid: true, indexes_valid: false, foreign_keys_valid: true };
        assert!(!partial.all_valid());
    }
}

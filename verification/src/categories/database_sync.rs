//! Database Synchronization Verification
//!
//! Verifies database schema matches code expectations and data integrity.

use crate::shared::{CheckStatus, ExpectedSchema, VerificationLevel, VerificationResult};
use sqlx::PgPool;
use std::collections::HashMap;

/// Database Sync Verifier
pub struct DatabaseSyncVerifier;

impl DatabaseSyncVerifier {
    /// Verify database schema matches expected schema
    pub async fn verify_schema_sync(pool: &PgPool) -> VerificationResult {
        let expected = ExpectedSchema::midi_software_center_v1();
        let mut mismatches = Vec::new();

        for expected_table in &expected.tables {
            // Check if table exists
            let table_exists: Result<Option<(String,)>, _> = sqlx::query_as(
                "SELECT table_name FROM information_schema.tables
                 WHERE table_schema = 'public' AND table_name = $1",
            )
            .bind(&expected_table.name)
            .fetch_optional(pool)
            .await;

            match table_exists {
                Ok(Some(_)) => {
                    // Table exists, check columns
                    let columns: Result<Vec<(String, String, String)>, _> = sqlx::query_as(
                        "SELECT column_name, data_type, is_nullable
                         FROM information_schema.columns
                         WHERE table_schema = 'public' AND table_name = $1",
                    )
                    .bind(&expected_table.name)
                    .fetch_all(pool)
                    .await;

                    if let Ok(actual_columns) = columns {
                        let actual_map: HashMap<String, (String, bool)> = actual_columns
                            .into_iter()
                            .map(|(name, dtype, nullable)| (name, (dtype, nullable == "YES")))
                            .collect();

                        for expected_col in &expected_table.columns {
                            match actual_map.get(&expected_col.name) {
                                Some((actual_type, actual_nullable)) => {
                                    // Check type (simplified comparison)
                                    if !types_compatible(&expected_col.data_type, actual_type) {
                                        mismatches.push(format!(
                                            "Table {}: column {} has type '{}' but expected '{}'",
                                            expected_table.name,
                                            expected_col.name,
                                            actual_type,
                                            expected_col.data_type
                                        ));
                                    }
                                    // Check nullability
                                    if *actual_nullable != expected_col.nullable {
                                        mismatches.push(format!(
                                            "Table {}: column {} nullable={} but expected nullable={}",
                                            expected_table.name,
                                            expected_col.name,
                                            actual_nullable,
                                            expected_col.nullable
                                        ));
                                    }
                                },
                                None => {
                                    mismatches.push(format!(
                                        "Table {}: missing column '{}'",
                                        expected_table.name, expected_col.name
                                    ));
                                },
                            }
                        }
                    }
                },
                Ok(None) => {
                    mismatches.push(format!("Missing table: {}", expected_table.name));
                },
                Err(e) => {
                    mismatches.push(format!(
                        "Error checking table {}: {}",
                        expected_table.name, e
                    ));
                },
            }
        }

        let status = if mismatches.is_empty() {
            CheckStatus::Pass
        } else if mismatches.len() <= 3 {
            CheckStatus::Warning
        } else {
            CheckStatus::Fail
        };

        VerificationResult {
            category: "database_sync".to_string(),
            check_name: "schema_sync".to_string(),
            level: VerificationLevel::Critical,
            status,
            message: format!("Schema sync: {} mismatch(es) found", mismatches.len()),
            details: mismatches,
            suggestion: Some("Run database migrations to sync schema".to_string()),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Verify all expected indexes exist
    pub async fn verify_indexes(pool: &PgPool) -> VerificationResult {
        let expected = ExpectedSchema::midi_software_center_v1();
        let mut missing_indexes = Vec::new();

        // Get all existing indexes
        let indexes: Result<Vec<(String, String)>, _> = sqlx::query_as(
            "SELECT indexname, tablename FROM pg_indexes WHERE schemaname = 'public'",
        )
        .fetch_all(pool)
        .await;

        let existing_indexes: Vec<String> = match indexes {
            Ok(idx) => idx.into_iter().map(|(name, _)| name).collect(),
            Err(e) => {
                return VerificationResult {
                    category: "database_sync".to_string(),
                    check_name: "indexes".to_string(),
                    level: VerificationLevel::High,
                    status: CheckStatus::Error,
                    message: format!("Failed to query indexes: {}", e),
                    details: vec![],
                    suggestion: None,
                    timestamp: chrono::Utc::now(),
                };
            },
        };

        for table in &expected.tables {
            for expected_idx in &table.indexes {
                if !existing_indexes.contains(&expected_idx.name) {
                    missing_indexes.push(format!(
                        "Missing index: {} on {} (columns: {:?})",
                        expected_idx.name, table.name, expected_idx.columns
                    ));
                }
            }
        }

        let status = if missing_indexes.is_empty() {
            CheckStatus::Pass
        } else if missing_indexes.len() <= 2 {
            CheckStatus::Warning
        } else {
            CheckStatus::Fail
        };

        VerificationResult {
            category: "database_sync".to_string(),
            check_name: "indexes".to_string(),
            level: VerificationLevel::High,
            status,
            message: format!("Index check: {} missing index(es)", missing_indexes.len()),
            details: missing_indexes,
            suggestion: Some("Create missing indexes for query performance".to_string()),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Verify foreign key constraints
    pub async fn verify_foreign_keys(pool: &PgPool) -> VerificationResult {
        let mut issues = Vec::new();

        // Check that file_tags references valid files
        let orphan_file_tags: Result<(i64,), _> = sqlx::query_as(
            "SELECT COUNT(*) FROM file_tags ft
             WHERE NOT EXISTS (SELECT 1 FROM files f WHERE f.id = ft.file_id)",
        )
        .fetch_one(pool)
        .await;

        if let Ok((count,)) = orphan_file_tags {
            if count > 0 {
                issues.push(format!(
                    "file_tags has {} orphan records (missing files)",
                    count
                ));
            }
        }

        // Check that file_tags references valid tags
        let orphan_tags: Result<(i64,), _> = sqlx::query_as(
            "SELECT COUNT(*) FROM file_tags ft
             WHERE NOT EXISTS (SELECT 1 FROM tags t WHERE t.id = ft.tag_id)",
        )
        .fetch_one(pool)
        .await;

        if let Ok((count,)) = orphan_tags {
            if count > 0 {
                issues.push(format!(
                    "file_tags has {} orphan records (missing tags)",
                    count
                ));
            }
        }

        // Check that musical_metadata references valid files
        let orphan_metadata: Result<(i64,), _> = sqlx::query_as(
            "SELECT COUNT(*) FROM musical_metadata m
             WHERE NOT EXISTS (SELECT 1 FROM files f WHERE f.id = m.file_id)",
        )
        .fetch_one(pool)
        .await;

        if let Ok((count,)) = orphan_metadata {
            if count > 0 {
                issues.push(format!("musical_metadata has {} orphan records", count));
            }
        }

        let status = if issues.is_empty() {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        };

        VerificationResult {
            category: "database_sync".to_string(),
            check_name: "foreign_keys".to_string(),
            level: VerificationLevel::Critical,
            status,
            message: format!("Foreign key integrity: {} issue(s) found", issues.len()),
            details: issues,
            suggestion: Some("Clean up orphan records or add missing foreign keys".to_string()),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Verify data consistency
    pub async fn verify_data_consistency(pool: &PgPool) -> VerificationResult {
        let mut issues = Vec::new();

        // Check for duplicate files by hash
        let dup_hashes: Result<Vec<(String, i64)>, _> = sqlx::query_as(
            "SELECT hash, COUNT(*) as cnt FROM files
             WHERE hash IS NOT NULL
             GROUP BY hash HAVING COUNT(*) > 1
             LIMIT 10",
        )
        .fetch_all(pool)
        .await;

        if let Ok(dups) = dup_hashes {
            if !dups.is_empty() {
                issues.push(format!(
                    "Found {} duplicate hashes (first 10 shown)",
                    dups.len()
                ));
                for (hash, count) in dups.iter().take(5) {
                    issues.push(format!("  Hash {} has {} duplicates", &hash[..8], count));
                }
            }
        }

        // Check for files with missing metadata
        let missing_metadata: Result<(i64,), _> = sqlx::query_as(
            "SELECT COUNT(*) FROM files f
             WHERE NOT EXISTS (SELECT 1 FROM musical_metadata m WHERE m.file_id = f.id)",
        )
        .fetch_one(pool)
        .await;

        if let Ok((count,)) = missing_metadata {
            let total: Result<(i64,), _> =
                sqlx::query_as("SELECT COUNT(*) FROM files").fetch_one(pool).await;
            if let Ok((total_count,)) = total {
                let pct = if total_count > 0 {
                    count as f64 / total_count as f64 * 100.0
                } else {
                    0.0
                };
                if pct > 10.0 {
                    issues.push(format!(
                        "{} files ({:.1}%) missing musical metadata",
                        count, pct
                    ));
                }
            }
        }

        // Check for invalid BPM values
        let invalid_bpm: Result<(i64,), _> = sqlx::query_as(
            "SELECT COUNT(*) FROM musical_metadata
             WHERE bpm IS NOT NULL AND (bpm < 20 OR bpm > 400)",
        )
        .fetch_one(pool)
        .await;

        if let Ok((count,)) = invalid_bpm {
            if count > 0 {
                issues.push(format!(
                    "{} records with invalid BPM (< 20 or > 400)",
                    count
                ));
            }
        }

        let status = if issues.is_empty() {
            CheckStatus::Pass
        } else if issues.len() <= 2 {
            CheckStatus::Warning
        } else {
            CheckStatus::Fail
        };

        VerificationResult {
            category: "database_sync".to_string(),
            check_name: "data_consistency".to_string(),
            level: VerificationLevel::High,
            status,
            message: format!("Data consistency: {} issue(s) found", issues.len()),
            details: issues,
            suggestion: Some("Run data cleanup scripts".to_string()),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Verify row counts are within expected ranges
    pub async fn verify_row_counts(pool: &PgPool) -> VerificationResult {
        let expected = ExpectedSchema::midi_software_center_v1();
        let mut issues = Vec::new();
        let mut details = Vec::new();

        for table in &expected.tables {
            let count_result: Result<(i64,), _> =
                sqlx::query_as(&format!("SELECT COUNT(*) FROM {}", table.name))
                    .fetch_one(pool)
                    .await;

            if let Ok((count,)) = count_result {
                details.push(format!("{}: {} rows", table.name, count));

                if let Some(min) = table.expected_row_count_min {
                    if count < min {
                        issues.push(format!(
                            "{}: {} rows (expected at least {})",
                            table.name, count, min
                        ));
                    }
                }
                if let Some(max) = table.expected_row_count_max {
                    if count > max {
                        issues.push(format!(
                            "{}: {} rows (expected at most {})",
                            table.name, count, max
                        ));
                    }
                }
            }
        }

        let status = if issues.is_empty() {
            CheckStatus::Pass
        } else {
            CheckStatus::Warning // Row counts are informational
        };

        VerificationResult {
            category: "database_sync".to_string(),
            check_name: "row_counts".to_string(),
            level: VerificationLevel::Low,
            status,
            message: format!("Row count check: {} out of expected range", issues.len()),
            details: if issues.is_empty() { details } else { issues },
            suggestion: Some("Review data import status".to_string()),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Get full database sync verification summary
    pub async fn get_summary(pool: &PgPool) -> DatabaseSyncSummary {
        let schema = Self::verify_schema_sync(pool).await;
        let indexes = Self::verify_indexes(pool).await;
        let foreign_keys = Self::verify_foreign_keys(pool).await;
        let consistency = Self::verify_data_consistency(pool).await;
        let row_counts = Self::verify_row_counts(pool).await;

        DatabaseSyncSummary {
            schema_synced: schema.status == CheckStatus::Pass,
            indexes_present: indexes.status == CheckStatus::Pass
                || indexes.status == CheckStatus::Warning,
            foreign_keys_valid: foreign_keys.status == CheckStatus::Pass,
            data_consistent: consistency.status == CheckStatus::Pass
                || consistency.status == CheckStatus::Warning,
            row_counts_valid: row_counts.status == CheckStatus::Pass,
            results: vec![schema, indexes, foreign_keys, consistency, row_counts],
        }
    }
}

/// Summary of database sync verification
#[derive(Debug)]
pub struct DatabaseSyncSummary {
    pub schema_synced: bool,
    pub indexes_present: bool,
    pub foreign_keys_valid: bool,
    pub data_consistent: bool,
    pub row_counts_valid: bool,
    pub results: Vec<VerificationResult>,
}

impl DatabaseSyncSummary {
    pub fn all_synced(&self) -> bool {
        self.schema_synced
            && self.indexes_present
            && self.foreign_keys_valid
            && self.data_consistent
    }
}

/// Check if two PostgreSQL types are compatible
fn types_compatible(expected: &str, actual: &str) -> bool {
    let expected_lower = expected.to_lowercase();
    let actual_lower = actual.to_lowercase();

    // Direct match
    if expected_lower == actual_lower {
        return true;
    }

    // Common type aliases
    let aliases: Vec<(&str, &[&str])> = vec![
        ("bigint", &["bigint", "int8", "bigserial"]),
        ("integer", &["integer", "int", "int4", "serial"]),
        ("text", &["text", "character varying", "varchar"]),
        ("double precision", &["double precision", "float8", "real"]),
        (
            "timestamp with time zone",
            &["timestamp with time zone", "timestamptz"],
        ),
        ("boolean", &["boolean", "bool"]),
    ];

    for (canonical, variations) in &aliases {
        if (expected_lower.contains(canonical) || variations.contains(&expected_lower.as_str()))
            && (actual_lower.contains(canonical) || variations.contains(&actual_lower.as_str()))
        {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_types_compatible() {
        assert!(types_compatible("bigint", "bigint"));
        assert!(types_compatible("bigint", "int8"));
        assert!(types_compatible("text", "character varying"));
        assert!(types_compatible("double precision", "float8"));
        assert!(!types_compatible("bigint", "text"));
    }

    #[test]
    fn test_expected_schema() {
        let schema = ExpectedSchema::midi_software_center_v1();
        assert!(!schema.tables.is_empty());
        assert!(schema.tables.iter().any(|t| t.name == "files"));
    }
}

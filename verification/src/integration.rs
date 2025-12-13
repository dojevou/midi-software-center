//! Integration Verification Module
//!
//! Verifies end-to-end workflow integration between components.

use sqlx::PgPool;

/// Type alias for metadata query result to reduce type complexity
type MetadataQueryResult = Result<Option<(i64, Option<f64>, Option<String>)>, sqlx::Error>;

/// Integration verification error
#[derive(Debug, Clone)]
pub struct IntegrationError {
    pub workflow: String,
    pub step: String,
    pub message: String,
    pub severity: IntegrationErrorSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrationErrorSeverity {
    Critical,
    Warning,
    Info,
}

/// Search filters for testing
#[derive(Debug, Clone, Default)]
pub struct TestSearchFilters {
    pub query: Option<String>,
    pub bpm_range: Option<(f64, f64)>,
    pub key: Option<String>,
    pub tags: Option<Vec<String>>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

/// Search result for testing
#[derive(Debug, Clone)]
pub struct TestSearchResult {
    pub files: Vec<TestFileMetadata>,
    pub total: i64,
}

/// File metadata for testing
#[derive(Debug, Clone)]
pub struct TestFileMetadata {
    pub id: i64,
    pub filename: String,
    pub filepath: String,
}

/// Verifies integration between components
pub struct IntegrationVerifier;

impl IntegrationVerifier {
    /// Verify full workflow integration
    pub async fn verify_full_workflow(pool: &PgPool) -> Result<(), Vec<IntegrationError>> {
        let mut errors = Vec::new();

        // Test 1: Database search workflow
        if let Err(e) = Self::test_search_workflow(pool).await {
            errors.push(IntegrationError {
                workflow: "search".to_string(),
                step: "database".to_string(),
                message: e,
                severity: IntegrationErrorSeverity::Critical,
            });
        }

        // Test 2: Metadata retrieval workflow
        if let Err(e) = Self::test_metadata_workflow(pool).await {
            errors.push(IntegrationError {
                workflow: "metadata".to_string(),
                step: "retrieval".to_string(),
                message: e,
                severity: IntegrationErrorSeverity::Warning,
            });
        }

        // Test 3: Tag system workflow
        if let Err(e) = Self::test_tagging_workflow(pool).await {
            errors.push(IntegrationError {
                workflow: "tagging".to_string(),
                step: "database".to_string(),
                message: e,
                severity: IntegrationErrorSeverity::Warning,
            });
        }

        // Test 4: File path resolution
        if let Err(e) = Self::test_file_path_resolution(pool).await {
            errors.push(IntegrationError {
                workflow: "file_paths".to_string(),
                step: "resolution".to_string(),
                message: e,
                severity: IntegrationErrorSeverity::Warning,
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Test database search workflow
    async fn test_search_workflow(pool: &PgPool) -> Result<(), String> {
        println!("Testing search workflow...");

        // Search for files with a common term
        let query = "drum";
        let result: Result<Vec<(i64, String, String)>, _> = sqlx::query_as(
            "SELECT id, filename, filepath FROM files WHERE filename ILIKE $1 LIMIT 10",
        )
        .bind(format!("%{}%", query))
        .fetch_all(pool)
        .await;

        match result {
            Ok(files) => {
                if files.is_empty() {
                    return Err("Search returned no results for 'drum'".to_string());
                }

                println!(
                    "Search workflow: Found {} files matching '{}'",
                    files.len(),
                    query
                );

                // Verify file metadata is populated
                for (id, filename, filepath) in &files {
                    if filename.is_empty() {
                        return Err(format!("File {} has empty filename", id));
                    }
                    if filepath.is_empty() {
                        return Err(format!("File {} has empty filepath", id));
                    }
                }

                Ok(())
            },
            Err(e) => Err(format!("Search query failed: {}", e)),
        }
    }

    /// Test metadata retrieval workflow
    async fn test_metadata_workflow(pool: &PgPool) -> Result<(), String> {
        println!("Testing metadata workflow...");

        // Get a file with metadata (cast numeric bpm to float8, enum key_signature to text)
        let result: MetadataQueryResult = sqlx::query_as(
            "SELECT f.id, m.bpm::float8, m.key_signature::text
             FROM files f
             JOIN musical_metadata m ON f.id = m.file_id
             WHERE m.bpm IS NOT NULL
             LIMIT 1",
        )
        .fetch_optional(pool)
        .await;

        match result {
            Ok(Some((id, bpm, key))) => {
                println!(
                    "Metadata workflow: File {} has BPM={:?}, Key={:?}",
                    id, bpm, key
                );

                // Validate BPM range
                if let Some(bpm_val) = bpm {
                    if !(20.0..=400.0).contains(&bpm_val) {
                        return Err(format!("Invalid BPM value: {}", bpm_val));
                    }
                }

                Ok(())
            },
            Ok(None) => Err("No files with metadata found".to_string()),
            Err(e) => Err(format!("Metadata query failed: {}", e)),
        }
    }

    /// Test tagging system workflow
    async fn test_tagging_workflow(pool: &PgPool) -> Result<(), String> {
        println!("Testing tagging workflow...");

        // Check tags exist
        let tag_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tags")
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Tag count query failed: {}", e))?;

        if tag_count == 0 {
            return Err("No tags defined in database".to_string());
        }

        println!("Tagging workflow: {} tags defined", tag_count);

        // Check file-tag relationships
        let relationship_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM file_tags")
            .fetch_one(pool)
            .await
            .map_err(|e| format!("File-tag count query failed: {}", e))?;

        if relationship_count == 0 && tag_count > 0 {
            return Err("Tags exist but no files are tagged".to_string());
        }

        println!(
            "Tagging workflow: {} file-tag relationships",
            relationship_count
        );

        // Test tag search
        let tagged_files: i64 = sqlx::query_scalar("SELECT COUNT(DISTINCT file_id) FROM file_tags")
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Tagged files count query failed: {}", e))?;

        println!("Tagging workflow: {} files have tags", tagged_files);

        Ok(())
    }

    /// Test file path resolution
    async fn test_file_path_resolution(pool: &PgPool) -> Result<(), String> {
        println!("Testing file path resolution...");

        // Get sample file paths
        let paths: Vec<String> = sqlx::query_scalar("SELECT filepath FROM files LIMIT 5")
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Path query failed: {}", e))?;

        if paths.is_empty() {
            return Err("No file paths in database".to_string());
        }

        let mut accessible = 0;
        let mut missing = 0;

        for path in &paths {
            if std::path::Path::new(path).exists() {
                accessible += 1;
            } else {
                missing += 1;
            }
        }

        println!("File paths: {} accessible, {} missing", accessible, missing);

        // Warn if too many files are missing
        if missing > accessible {
            return Err(format!(
                "More files missing ({}) than accessible ({}) - paths may be incorrect",
                missing, accessible
            ));
        }

        Ok(())
    }

    /// Verify command invocation integration
    pub fn verify_command_integration() -> Result<(), Vec<IntegrationError>> {
        let mut errors = Vec::new();

        // Check that the command registry exists and has expected commands
        let expected_commands = vec![
            "search_files",
            "get_file_metadata",
            "start_sequencer",
            "stop_sequencer",
            "set_tempo",
        ];

        // Check the Commands registry file
        let commands_file = "./app/src/lib/api/commands.ts";
        if !std::path::Path::new(commands_file).exists() {
            errors.push(IntegrationError {
                workflow: "commands".to_string(),
                step: "registry".to_string(),
                message: "Command registry file not found".to_string(),
                severity: IntegrationErrorSeverity::Critical,
            });
        } else if let Ok(content) = std::fs::read_to_string(commands_file) {
            for command in expected_commands {
                if !content.contains(command) {
                    errors.push(IntegrationError {
                        workflow: "commands".to_string(),
                        step: command.to_string(),
                        message: format!("Expected command '{}' not found in registry", command),
                        severity: IntegrationErrorSeverity::Warning,
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get integration verification summary
    pub async fn get_summary(pool: &PgPool) -> IntegrationSummary {
        let workflow_ok = Self::verify_full_workflow(pool).await.is_ok();
        let commands_ok = Self::verify_command_integration().is_ok();

        IntegrationSummary { workflows_integrated: workflow_ok, commands_integrated: commands_ok }
    }
}

/// Integration verification summary
#[derive(Debug, Clone)]
pub struct IntegrationSummary {
    pub workflows_integrated: bool,
    pub commands_integrated: bool,
}

impl IntegrationSummary {
    pub fn all_integrated(&self) -> bool {
        self.workflows_integrated && self.commands_integrated
    }
}

impl std::fmt::Display for IntegrationSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Integration Summary:\n  Workflows: {}\n  Commands: {}",
            if self.workflows_integrated {
                "OK"
            } else {
                "FAILED"
            },
            if self.commands_integrated {
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
    fn test_integration_error_severity() {
        let error = IntegrationError {
            workflow: "test".to_string(),
            step: "step1".to_string(),
            message: "Test message".to_string(),
            severity: IntegrationErrorSeverity::Critical,
        };
        assert_eq!(error.severity, IntegrationErrorSeverity::Critical);
    }

    #[test]
    fn test_integration_summary_display() {
        let summary = IntegrationSummary { workflows_integrated: true, commands_integrated: false };
        let display = format!("{}", summary);
        assert!(display.contains("Workflows: OK"));
        assert!(display.contains("Commands: FAILED"));
    }

    #[test]
    fn test_integration_summary_all_integrated() {
        let summary = IntegrationSummary { workflows_integrated: true, commands_integrated: true };
        assert!(summary.all_integrated());

        let partial = IntegrationSummary { workflows_integrated: true, commands_integrated: false };
        assert!(!partial.all_integrated());
    }
}

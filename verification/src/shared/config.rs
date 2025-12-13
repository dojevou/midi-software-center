//! Verification Configuration
//!
//! Configuration for the verification system.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Verification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationConfig {
    pub database_url: Option<String>,
    pub strict_mode: bool,
    pub skip_categories: HashSet<String>,
    pub output_format: OutputFormat,
    pub verbose: bool,
    pub ci_mode: bool,
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL").ok(),
            strict_mode: false,
            skip_categories: HashSet::new(),
            output_format: OutputFormat::Console,
            verbose: false,
            ci_mode: false,
        }
    }
}

impl VerificationConfig {
    pub fn with_database_url(mut self, url: &str) -> Self {
        self.database_url = Some(url.to_string());
        self
    }

    pub fn with_strict_mode(mut self) -> Self {
        self.strict_mode = true;
        self
    }

    pub fn with_ci_mode(mut self) -> Self {
        self.ci_mode = true;
        self.strict_mode = true;
        self
    }

    pub fn skip_category(mut self, category: &str) -> Self {
        self.skip_categories.insert(category.to_string());
        self
    }

    pub fn should_run_category(&self, category: &str) -> bool {
        !self.skip_categories.contains(category)
    }
}

/// Output format for reports
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputFormat {
    Console,
    Json,
    Html,
}

/// Expected database schema configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedSchema {
    pub tables: Vec<ExpectedTable>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedTable {
    pub name: String,
    pub columns: Vec<ExpectedColumn>,
    pub indexes: Vec<ExpectedIndex>,
    pub expected_row_count_min: Option<i64>,
    pub expected_row_count_max: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedColumn {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedIndex {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

impl ExpectedSchema {
    /// Define the expected schema for MIDI Software Center
    pub fn midi_software_center_v1() -> Self {
        Self {
            version: "1.0.0".to_string(),
            tables: vec![
                ExpectedTable {
                    name: "files".to_string(),
                    expected_row_count_min: Some(1_000_000),
                    expected_row_count_max: Some(5_000_000),
                    columns: vec![
                        ExpectedColumn {
                            name: "id".to_string(),
                            data_type: "bigint".to_string(),
                            nullable: false,
                        },
                        ExpectedColumn {
                            name: "filename".to_string(),
                            data_type: "text".to_string(),
                            nullable: false,
                        },
                        ExpectedColumn {
                            name: "filepath".to_string(),
                            data_type: "text".to_string(),
                            nullable: false,
                        },
                        ExpectedColumn {
                            name: "content_hash".to_string(),
                            data_type: "bytea".to_string(),
                            nullable: false,
                        },
                        ExpectedColumn {
                            name: "file_size_bytes".to_string(),
                            data_type: "bigint".to_string(),
                            nullable: false,
                        },
                        ExpectedColumn {
                            name: "created_at".to_string(),
                            data_type: "timestamp with time zone".to_string(),
                            nullable: true,
                        },
                    ],
                    indexes: vec![
                        ExpectedIndex {
                            name: "idx_files_filepath".to_string(),
                            columns: vec!["filepath".to_string()],
                            unique: false,
                        },
                        ExpectedIndex {
                            name: "idx_files_filename_lower".to_string(),
                            columns: vec!["filename".to_string()],
                            unique: false,
                        },
                        ExpectedIndex {
                            name: "idx_files_content_hash".to_string(),
                            columns: vec!["content_hash".to_string()],
                            unique: false,
                        },
                    ],
                },
                ExpectedTable {
                    name: "musical_metadata".to_string(),
                    expected_row_count_min: Some(500_000),
                    expected_row_count_max: Some(5_000_000),
                    columns: vec![
                        ExpectedColumn {
                            name: "file_id".to_string(),
                            data_type: "bigint".to_string(),
                            nullable: false,
                        },
                        ExpectedColumn {
                            name: "bpm".to_string(),
                            data_type: "numeric".to_string(),
                            nullable: true,
                        },
                        ExpectedColumn {
                            name: "key_signature".to_string(),
                            data_type: "user-defined".to_string(),
                            nullable: true,
                        },
                        ExpectedColumn {
                            name: "time_signature_numerator".to_string(),
                            data_type: "smallint".to_string(),
                            nullable: true,
                        },
                        ExpectedColumn {
                            name: "time_signature_denominator".to_string(),
                            data_type: "smallint".to_string(),
                            nullable: true,
                        },
                    ],
                    indexes: vec![
                        ExpectedIndex {
                            name: "musical_metadata_pkey".to_string(),
                            columns: vec!["file_id".to_string()],
                            unique: true,
                        },
                        ExpectedIndex {
                            name: "idx_metadata_bpm".to_string(),
                            columns: vec!["bpm".to_string()],
                            unique: false,
                        },
                    ],
                },
                ExpectedTable {
                    name: "tags".to_string(),
                    expected_row_count_min: Some(100),
                    expected_row_count_max: Some(50_000),
                    columns: vec![
                        ExpectedColumn {
                            name: "id".to_string(),
                            data_type: "bigint".to_string(),
                            nullable: false,
                        },
                        ExpectedColumn {
                            name: "name".to_string(),
                            data_type: "text".to_string(),
                            nullable: false,
                        },
                    ],
                    indexes: vec![ExpectedIndex {
                        name: "tags_name_key".to_string(),
                        columns: vec!["name".to_string()],
                        unique: true,
                    }],
                },
                ExpectedTable {
                    name: "file_tags".to_string(),
                    expected_row_count_min: Some(1_000_000),
                    expected_row_count_max: Some(50_000_000),
                    columns: vec![
                        ExpectedColumn {
                            name: "file_id".to_string(),
                            data_type: "bigint".to_string(),
                            nullable: false,
                        },
                        ExpectedColumn {
                            name: "tag_id".to_string(),
                            data_type: "bigint".to_string(),
                            nullable: false,
                        },
                    ],
                    indexes: vec![
                        ExpectedIndex {
                            name: "idx_file_tags_file".to_string(),
                            columns: vec!["file_id".to_string()],
                            unique: false,
                        },
                        ExpectedIndex {
                            name: "idx_file_tags_tag".to_string(),
                            columns: vec!["tag_id".to_string()],
                            unique: false,
                        },
                    ],
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = VerificationConfig::default();
        assert!(!config.strict_mode);
        assert!(!config.ci_mode);
        assert!(config.skip_categories.is_empty());
    }

    #[test]
    fn test_config_skip_category() {
        let config = VerificationConfig::default()
            .skip_category("security")
            .skip_category("performance");

        assert!(!config.should_run_category("security"));
        assert!(!config.should_run_category("performance"));
        assert!(config.should_run_category("database"));
    }

    #[test]
    fn test_expected_schema() {
        let schema = ExpectedSchema::midi_software_center_v1();
        assert_eq!(schema.tables.len(), 4);
        assert!(schema.tables.iter().any(|t| t.name == "files"));
    }
}

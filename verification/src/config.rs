//! Configuration Validation Module
//!
//! Validates application configuration and environment variables.

use std::env;
use std::fs;
use std::path::Path;

/// Configuration validation error
#[derive(Debug, Clone)]
pub struct ConfigError {
    pub category: String,
    pub key: String,
    pub message: String,
    pub severity: ConfigErrorSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigErrorSeverity {
    Critical, // Application won't start
    Warning,  // Application may have issues
    Info,     // Informational only
}

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

impl DatabaseConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let url = env::var("DATABASE_URL").map_err(|_| ConfigError {
            category: "database".to_string(),
            key: "DATABASE_URL".to_string(),
            message: "DATABASE_URL environment variable not set".to_string(),
            severity: ConfigErrorSeverity::Critical,
        })?;

        let max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "34".to_string())
            .parse()
            .unwrap_or(34);

        let timeout_seconds = env::var("DATABASE_TIMEOUT")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);

        Ok(Self { url, max_connections, timeout_seconds })
    }

    pub fn validate(&self) -> Vec<ConfigError> {
        let mut errors = Vec::new();

        // Validate URL format
        if !self.url.starts_with("postgresql://") && !self.url.starts_with("postgres://") {
            errors.push(ConfigError {
                category: "database".to_string(),
                key: "DATABASE_URL".to_string(),
                message: "DATABASE_URL must start with postgresql:// or postgres://".to_string(),
                severity: ConfigErrorSeverity::Critical,
            });
        }

        // Validate connection count
        if self.max_connections < 1 || self.max_connections > 100 {
            errors.push(ConfigError {
                category: "database".to_string(),
                key: "max_connections".to_string(),
                message: format!(
                    "max_connections ({}) should be between 1 and 100",
                    self.max_connections
                ),
                severity: ConfigErrorSeverity::Warning,
            });
        }

        // Validate timeout
        if self.timeout_seconds < 1 || self.timeout_seconds > 300 {
            errors.push(ConfigError {
                category: "database".to_string(),
                key: "timeout_seconds".to_string(),
                message: format!(
                    "timeout_seconds ({}) should be between 1 and 300",
                    self.timeout_seconds
                ),
                severity: ConfigErrorSeverity::Warning,
            });
        }

        errors
    }
}

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database: Option<DatabaseConfig>,
    pub log_level: String,
    pub environment: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            database: DatabaseConfig::from_env().ok(),
            log_level: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            environment: env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()),
        }
    }
}

/// Configuration verifier
pub struct ConfigVerifier;

impl ConfigVerifier {
    /// Validate all configuration
    pub fn validate_all() -> Result<(), Vec<ConfigError>> {
        let mut errors = Vec::new();

        // Validate environment variables
        errors.extend(Self::validate_environment_variables());

        // Validate config files
        errors.extend(Self::validate_config_files());

        // Validate Tauri configuration
        errors.extend(Self::validate_tauri_config());

        // Validate Cargo configuration
        errors.extend(Self::validate_cargo_config());

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate required environment variables
    pub fn validate_environment_variables() -> Vec<ConfigError> {
        let mut errors = Vec::new();

        // Required for database operations
        if env::var("DATABASE_URL").is_err() {
            errors.push(ConfigError {
                category: "environment".to_string(),
                key: "DATABASE_URL".to_string(),
                message: "DATABASE_URL environment variable not set".to_string(),
                severity: ConfigErrorSeverity::Critical,
            });
        }

        // Optional but recommended
        if env::var("RUST_LOG").is_err() {
            errors.push(ConfigError {
                category: "environment".to_string(),
                key: "RUST_LOG".to_string(),
                message: "RUST_LOG not set, defaulting to 'info'".to_string(),
                severity: ConfigErrorSeverity::Info,
            });
        }

        errors
    }

    /// Validate configuration files exist and are valid
    pub fn validate_config_files() -> Vec<ConfigError> {
        let mut errors = Vec::new();

        let config_files = vec![
            (
                "./app/src-tauri/tauri.conf.json",
                true,
                "Tauri configuration",
            ),
            ("./app/tsconfig.json", true, "TypeScript configuration"),
            ("./app/vite.config.ts", true, "Vite configuration"),
            ("./rustfmt.toml", false, "Rust formatter configuration"),
        ];

        for (path, required, description) in config_files {
            if !Path::new(path).exists() {
                if required {
                    errors.push(ConfigError {
                        category: "files".to_string(),
                        key: path.to_string(),
                        message: format!("{} is missing", description),
                        severity: ConfigErrorSeverity::Critical,
                    });
                } else {
                    errors.push(ConfigError {
                        category: "files".to_string(),
                        key: path.to_string(),
                        message: format!("{} is missing (optional)", description),
                        severity: ConfigErrorSeverity::Info,
                    });
                }
            }
        }

        errors
    }

    /// Validate Tauri configuration
    pub fn validate_tauri_config() -> Vec<ConfigError> {
        let mut errors = Vec::new();
        let tauri_config_path = "./app/src-tauri/tauri.conf.json";

        if let Ok(content) = fs::read_to_string(tauri_config_path) {
            // Try to parse as JSON
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(config) => {
                    // Check required fields
                    if config.get("productName").is_none() {
                        errors.push(ConfigError {
                            category: "tauri".to_string(),
                            key: "productName".to_string(),
                            message: "productName not set in tauri.conf.json".to_string(),
                            severity: ConfigErrorSeverity::Warning,
                        });
                    }

                    if config.get("identifier").is_none() {
                        errors.push(ConfigError {
                            category: "tauri".to_string(),
                            key: "identifier".to_string(),
                            message: "identifier not set in tauri.conf.json".to_string(),
                            severity: ConfigErrorSeverity::Critical,
                        });
                    }

                    // Check build configuration
                    if let Some(build) = config.get("build") {
                        if build.get("devUrl").is_none() {
                            errors.push(ConfigError {
                                category: "tauri".to_string(),
                                key: "build.devUrl".to_string(),
                                message: "devUrl not set in tauri.conf.json".to_string(),
                                severity: ConfigErrorSeverity::Warning,
                            });
                        }
                    }
                },
                Err(e) => {
                    errors.push(ConfigError {
                        category: "tauri".to_string(),
                        key: tauri_config_path.to_string(),
                        message: format!("Invalid JSON in tauri.conf.json: {}", e),
                        severity: ConfigErrorSeverity::Critical,
                    });
                },
            }
        }

        errors
    }

    /// Validate Cargo workspace configuration
    pub fn validate_cargo_config() -> Vec<ConfigError> {
        let mut errors = Vec::new();
        let cargo_toml_path = "./Cargo.toml";

        if let Ok(content) = fs::read_to_string(cargo_toml_path) {
            // Check for workspace members
            if !content.contains("[workspace]") {
                errors.push(ConfigError {
                    category: "cargo".to_string(),
                    key: "workspace".to_string(),
                    message: "Cargo.toml does not define a workspace".to_string(),
                    severity: ConfigErrorSeverity::Warning,
                });
            }

            // Check for required members
            let required_members =
                vec!["app/src-tauri", "pipeline/src-tauri", "daw/src-tauri", "shared/rust"];
            for member in required_members {
                if !content.contains(member) {
                    errors.push(ConfigError {
                        category: "cargo".to_string(),
                        key: format!("workspace.members.{}", member),
                        message: format!("Workspace member '{}' not found in Cargo.toml", member),
                        severity: ConfigErrorSeverity::Warning,
                    });
                }
            }
        }

        errors
    }

    /// Get configuration summary
    pub fn get_summary() -> ConfigSummary {
        let env_errors = Self::validate_environment_variables();
        let tauri_errors = Self::validate_tauri_config();
        let cargo_errors = Self::validate_cargo_config();
        let config_errors = Self::validate_config_files();

        // Environment is valid if no critical errors
        let environment_valid =
            !env_errors.iter().any(|e| e.severity == ConfigErrorSeverity::Critical);

        // Tauri is valid if no critical errors
        let tauri_valid = !tauri_errors.iter().any(|e| e.severity == ConfigErrorSeverity::Critical);

        // Cargo is valid if no critical errors
        let cargo_valid = !cargo_errors.iter().any(|e| e.severity == ConfigErrorSeverity::Critical);

        // Database is valid if DATABASE_URL is set and config files exist
        let database_valid = env::var("DATABASE_URL").is_ok()
            && !config_errors.iter().any(|e| e.severity == ConfigErrorSeverity::Critical);

        ConfigSummary { environment_valid, tauri_valid, cargo_valid, database_valid }
    }
}

/// Configuration summary
#[derive(Debug, Clone)]
pub struct ConfigSummary {
    pub environment_valid: bool,
    pub tauri_valid: bool,
    pub cargo_valid: bool,
    pub database_valid: bool,
}

impl ConfigSummary {
    pub fn all_valid(&self) -> bool {
        self.environment_valid && self.tauri_valid && self.cargo_valid && self.database_valid
    }
}

impl std::fmt::Display for ConfigSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Configuration Summary:\n  Environment: {}\n  Tauri: {}\n  Cargo: {}\n  Database: {}",
            if self.environment_valid {
                "OK"
            } else {
                "FAILED"
            },
            if self.tauri_valid { "OK" } else { "FAILED" },
            if self.cargo_valid { "OK" } else { "FAILED" },
            if self.database_valid { "OK" } else { "FAILED" },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_severity() {
        let error = ConfigError {
            category: "test".to_string(),
            key: "TEST_KEY".to_string(),
            message: "Test message".to_string(),
            severity: ConfigErrorSeverity::Critical,
        };
        assert_eq!(error.severity, ConfigErrorSeverity::Critical);
    }

    #[test]
    fn test_database_config_validation() {
        let config = DatabaseConfig {
            url: "postgresql://localhost/test".to_string(),
            max_connections: 10,
            timeout_seconds: 30,
        };
        let errors = config.validate();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_invalid_database_url() {
        let config = DatabaseConfig {
            url: "mysql://localhost/test".to_string(), // Wrong protocol
            max_connections: 10,
            timeout_seconds: 30,
        };
        let errors = config.validate();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_config_summary_display() {
        let summary = ConfigSummary {
            environment_valid: true,
            tauri_valid: true,
            cargo_valid: true,
            database_valid: false,
        };
        let display = format!("{}", summary);
        assert!(display.contains("Environment: OK"));
        assert!(display.contains("Database: FAILED"));
    }

    #[test]
    fn test_config_summary_all_valid() {
        let summary = ConfigSummary {
            environment_valid: true,
            tauri_valid: true,
            cargo_valid: true,
            database_valid: true,
        };
        assert!(summary.all_valid());

        let partial = ConfigSummary {
            environment_valid: true,
            tauri_valid: false,
            cargo_valid: true,
            database_valid: true,
        };
        assert!(!partial.all_valid());
    }
}

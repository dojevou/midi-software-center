//! MIDI Software Center Verification Library
//!
//! Comprehensive verification system for the MIDI Software Center project.
//! Verifies database schema, migrations, paths, configuration, dependencies,
//! build environment, health checks, and integration.
//!
//! ## Architecture
//!
//! The verification system is organized into:
//! - **Core modules**: Basic verification (build, config, dependencies, health, etc.)
//! - **Categories**: Advanced verification (api_commands, security, performance, database_sync)
//! - **Shared**: Common types and configuration

// Core modules
pub mod build;
pub mod config;
pub mod dependencies;
pub mod health;
pub mod integration;
pub mod migrations;
pub mod paths;

// Advanced verification categories
pub mod categories;
pub mod shared;

// Re-export core types
pub use build::{BuildError, BuildErrorSeverity, BuildSummary, BuildVerifier};
pub use config::{ConfigError, ConfigErrorSeverity, ConfigSummary, ConfigVerifier};
pub use dependencies::{
    DependencyError, DependencyErrorSeverity, DependencySummary, DependencyVerifier,
};
pub use health::{HealthError, HealthErrorSeverity, HealthSummary, HealthVerifier};
pub use integration::{
    IntegrationError, IntegrationErrorSeverity, IntegrationSummary, IntegrationVerifier,
};
pub use migrations::{MigrationError, MigrationErrorSeverity, MigrationSummary, MigrationVerifier};
pub use paths::{PathError, PathErrorType, PathSummary, PathVerifier};

// Re-export category verifiers
pub use categories::{
    ApiCommandsSummary, ApiCommandsVerifier, DatabaseSyncSummary, DatabaseSyncVerifier,
    PerformanceSummary, PerformanceVerifier, SecuritySummary, SecurityVerifier,
};

// Re-export shared types
pub use shared::{
    CheckStatus, EnvironmentInfo, ExpectedColumn, ExpectedIndex, ExpectedSchema, ExpectedTable,
    OutputFormat, VerificationConfig, VerificationLevel, VerificationReport,
    VerificationResult as SharedVerificationResult, VerificationSummary,
};

/// Overall verification result
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub migrations: MigrationSummary,
    pub paths: PathSummary,
    pub config: ConfigSummary,
    pub dependencies: DependencySummary,
    pub build: BuildSummary,
    pub health: HealthSummary,
    pub integration: IntegrationSummary,
}

impl VerificationResult {
    /// Check if all verifications passed
    pub fn all_passed(&self) -> bool {
        self.migrations.all_valid()
            && self.paths.all_valid()
            && self.config.all_valid()
            && self.dependencies.all_valid()
            && self.build.all_valid()
            && self.health.all_healthy()
            && self.integration.all_integrated()
    }

    /// Get count of failed verification categories
    pub fn failed_count(&self) -> usize {
        let mut count = 0;
        if !self.migrations.all_valid() {
            count += 1;
        }
        if !self.paths.all_valid() {
            count += 1;
        }
        if !self.config.all_valid() {
            count += 1;
        }
        if !self.dependencies.all_valid() {
            count += 1;
        }
        if !self.build.all_valid() {
            count += 1;
        }
        if !self.health.all_healthy() {
            count += 1;
        }
        if !self.integration.all_integrated() {
            count += 1;
        }
        count
    }
}

impl std::fmt::Display for VerificationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== MIDI Software Center Verification Results ===")?;
        writeln!(f)?;
        writeln!(f, "{}", self.migrations)?;
        writeln!(f)?;
        writeln!(f, "{}", self.paths)?;
        writeln!(f)?;
        writeln!(f, "{}", self.config)?;
        writeln!(f)?;
        writeln!(f, "{}", self.dependencies)?;
        writeln!(f)?;
        writeln!(f, "{}", self.build)?;
        writeln!(f)?;
        writeln!(f, "{}", self.health)?;
        writeln!(f)?;
        writeln!(f, "{}", self.integration)?;
        writeln!(f)?;
        writeln!(
            f,
            "=== Overall Status: {} ===",
            if self.all_passed() {
                "PASSED"
            } else {
                "FAILED"
            }
        )
    }
}

/// Verification options
#[derive(Debug, Clone, Default)]
pub struct VerificationOptions {
    /// Skip migration verification
    pub skip_migrations: bool,
    /// Skip path verification
    pub skip_paths: bool,
    /// Skip configuration verification
    pub skip_config: bool,
    /// Skip dependency verification
    pub skip_dependencies: bool,
    /// Skip build verification
    pub skip_build: bool,
    /// Skip health verification
    pub skip_health: bool,
    /// Skip integration verification
    pub skip_integration: bool,
    /// Run tests as part of build verification
    pub run_tests: bool,
    /// Verbose output
    pub verbose: bool,
}

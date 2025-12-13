//! Shared Types for Verification System
//!
//! Common types used across all verification categories.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Verification severity level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum VerificationLevel {
    Critical, // Blocks deployment
    High,     // Should be fixed before release
    Medium,   // Nice to have fixed
    Low,      // Optional improvements
}

/// Check status result
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Fail,
    Warning,
    Skipped,
    Error, // Verification itself failed
}

/// Individual verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub category: String,
    pub check_name: String,
    pub level: VerificationLevel,
    pub status: CheckStatus,
    pub message: String,
    pub details: Vec<String>,
    pub suggestion: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl VerificationResult {
    pub fn pass(category: &str, check_name: &str, level: VerificationLevel, message: &str) -> Self {
        Self {
            category: category.to_string(),
            check_name: check_name.to_string(),
            level,
            status: CheckStatus::Pass,
            message: message.to_string(),
            details: vec![],
            suggestion: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn fail(category: &str, check_name: &str, level: VerificationLevel, message: &str) -> Self {
        Self {
            category: category.to_string(),
            check_name: check_name.to_string(),
            level,
            status: CheckStatus::Fail,
            message: message.to_string(),
            details: vec![],
            suggestion: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn warning(
        category: &str,
        check_name: &str,
        level: VerificationLevel,
        message: &str,
    ) -> Self {
        Self {
            category: category.to_string(),
            check_name: check_name.to_string(),
            level,
            status: CheckStatus::Warning,
            message: message.to_string(),
            details: vec![],
            suggestion: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_details(mut self, details: Vec<String>) -> Self {
        self.details = details;
        self
    }

    pub fn with_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestion = Some(suggestion.to_string());
        self
    }

    pub fn is_critical_failure(&self) -> bool {
        self.status == CheckStatus::Fail && self.level == VerificationLevel::Critical
    }
}

/// Full verification report
#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationReport {
    pub project_name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub git_commit: String,
    pub environment: EnvironmentInfo,
    pub results: Vec<VerificationResult>,
    pub summary: VerificationSummary,
    pub metadata: HashMap<String, String>,
}

impl VerificationReport {
    pub fn new(project_name: &str) -> Self {
        Self {
            project_name: project_name.to_string(),
            timestamp: chrono::Utc::now(),
            git_commit: get_git_commit(),
            environment: EnvironmentInfo::detect(),
            results: Vec::new(),
            summary: VerificationSummary::default(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_result(&mut self, result: VerificationResult) {
        self.results.push(result);
    }

    pub fn calculate_summary(&mut self) {
        let total = self.results.len();
        let passed = self.results.iter().filter(|r| r.status == CheckStatus::Pass).count();
        let failed = self.results.iter().filter(|r| r.status == CheckStatus::Fail).count();
        let warnings = self.results.iter().filter(|r| r.status == CheckStatus::Warning).count();
        let skipped = self.results.iter().filter(|r| r.status == CheckStatus::Skipped).count();
        let errors = self.results.iter().filter(|r| r.status == CheckStatus::Error).count();
        let critical_failures = self.results.iter().filter(|r| r.is_critical_failure()).count();

        self.summary = VerificationSummary {
            total_checks: total,
            passed,
            failed,
            warnings,
            skipped,
            errors,
            critical_failures,
            success_rate: if total > 0 {
                passed as f64 / total as f64 * 100.0
            } else {
                0.0
            },
            deployment_blocked: critical_failures > 0 || (failed as f64 / total as f64) > 0.1,
        };
    }

    pub fn format_summary(&self) -> String {
        format!(
            "Verification Summary:\n\
             Total: {} | Passed: {} | Failed: {} | Warnings: {} | Skipped: {}\n\
             Success Rate: {:.1}%\n\
             Critical Failures: {}\n\
             Deployment: {}",
            self.summary.total_checks,
            self.summary.passed,
            self.summary.failed,
            self.summary.warnings,
            self.summary.skipped,
            self.summary.success_rate,
            self.summary.critical_failures,
            if self.summary.deployment_blocked {
                "BLOCKED"
            } else {
                "ALLOWED"
            }
        )
    }
}

/// Verification summary statistics
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct VerificationSummary {
    pub total_checks: usize,
    pub passed: usize,
    pub failed: usize,
    pub warnings: usize,
    pub skipped: usize,
    pub errors: usize,
    pub critical_failures: usize,
    pub success_rate: f64,
    pub deployment_blocked: bool,
}

/// Environment information
#[derive(Debug, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    pub rust_version: String,
    pub node_version: String,
    pub os: String,
    pub database_version: String,
    pub tauri_version: String,
}

impl EnvironmentInfo {
    pub fn detect() -> Self {
        Self {
            rust_version: get_command_output("rustc", &["--version"]),
            node_version: get_command_output("node", &["--version"]),
            os: std::env::consts::OS.to_string(),
            database_version: "PostgreSQL 16".to_string(),
            tauri_version: "2.7".to_string(),
        }
    }
}

fn get_git_commit() -> String {
    get_command_output("git", &["rev-parse", "--short", "HEAD"])
}

fn get_command_output(cmd: &str, args: &[&str]) -> String {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verification_result_pass() {
        let result = VerificationResult::pass("test", "check", VerificationLevel::High, "OK");
        assert_eq!(result.status, CheckStatus::Pass);
        assert!(!result.is_critical_failure());
    }

    #[test]
    fn test_verification_result_critical_failure() {
        let result =
            VerificationResult::fail("test", "check", VerificationLevel::Critical, "Failed");
        assert!(result.is_critical_failure());
    }

    #[test]
    fn test_verification_report_summary() {
        let mut report = VerificationReport::new("test");
        report.add_result(VerificationResult::pass(
            "a",
            "1",
            VerificationLevel::High,
            "ok",
        ));
        report.add_result(VerificationResult::fail(
            "b",
            "2",
            VerificationLevel::Medium,
            "fail",
        ));
        report.calculate_summary();

        assert_eq!(report.summary.total_checks, 2);
        assert_eq!(report.summary.passed, 1);
        assert_eq!(report.summary.failed, 1);
    }
}

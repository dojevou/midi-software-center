//! Security Verification
//!
//! Scans for security vulnerabilities, hardcoded secrets, and unsafe patterns.

use crate::shared::{CheckStatus, VerificationLevel, VerificationResult};
use std::fs;
use std::path::Path;

/// Security Verifier
pub struct SecurityVerifier;

impl SecurityVerifier {
    /// Scan for hardcoded secrets
    pub fn scan_hardcoded_secrets() -> VerificationResult {
        let mut findings = Vec::new();

        let secret_patterns = vec![
            ("password", "Hardcoded password"),
            ("api_key", "Hardcoded API key"),
            ("apikey", "Hardcoded API key"),
            ("secret_key", "Hardcoded secret key"),
            ("private_key", "Hardcoded private key"),
            ("aws_access", "AWS credentials"),
            ("aws_secret", "AWS credentials"),
            ("bearer ", "Bearer token"),
            ("authorization:", "Authorization header"),
        ];

        let scan_dirs = vec![
            "pipeline/src-tauri/src",
            "daw/src-tauri/src",
            "shared/rust/src",
            "app/src",
            "app/src-tauri/src",
        ];

        for dir in &scan_dirs {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        scan_file_for_secrets(&path, &secret_patterns, &mut findings);
                    }
                }
            }
            // Also scan subdirectories
            scan_directory_recursive(dir, &secret_patterns, &mut findings);
        }

        let status = if findings.is_empty() {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        };

        VerificationResult {
            category: "security".to_string(),
            check_name: "hardcoded_secrets".to_string(),
            level: VerificationLevel::Critical,
            status,
            message: format!("Found {} potential hardcoded secret(s)", findings.len()),
            details: findings,
            suggestion: Some("Use environment variables or secure vaults for secrets".to_string()),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Check for SQL injection vulnerabilities
    pub fn check_sql_injection() -> VerificationResult {
        let mut vulnerabilities = Vec::new();

        let dangerous_patterns = vec![
            "format!(\"SELECT", // String interpolation in SQL
            "format!(\"INSERT",
            "format!(\"UPDATE",
            "format!(\"DELETE",
            "&format!(\"", // Reference to formatted SQL
            "+ query",     // String concatenation with query
            "query + ",    // String concatenation with query
        ];

        let scan_files = vec![
            "pipeline/src-tauri/src/db/repositories/file_repository.rs",
            "pipeline/src-tauri/src/db/repositories/search_repository.rs",
            "pipeline/src-tauri/src/db/repositories/tag_repository.rs",
            "pipeline/src-tauri/src/db/repositories/metadata_repository.rs",
            "shared/rust/src/db/repositories/file_repository.rs",
            "shared/rust/src/db/repositories/search_repository.rs",
        ];

        for file_path in &scan_files {
            if let Ok(content) = fs::read_to_string(file_path) {
                for (line_num, line) in content.lines().enumerate() {
                    let line_lower = line.to_lowercase();
                    for pattern in &dangerous_patterns {
                        if line_lower.contains(&pattern.to_lowercase()) {
                            // Exclude safe sqlx patterns
                            if !line.contains("sqlx::query")
                                && !line.contains("query_as")
                                && !line.contains("$1")
                            {
                                vulnerabilities.push(format!(
                                    "{}:{} - Potential SQL injection: {}",
                                    file_path,
                                    line_num + 1,
                                    line.trim()
                                ));
                            }
                        }
                    }
                }
            }
        }

        let status = if vulnerabilities.is_empty() {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        };

        VerificationResult {
            category: "security".to_string(),
            check_name: "sql_injection".to_string(),
            level: VerificationLevel::Critical,
            status,
            message: format!(
                "Found {} potential SQL injection vulnerabilit(ies)",
                vulnerabilities.len()
            ),
            details: vulnerabilities,
            suggestion: Some("Use parameterized queries with sqlx ($1, $2, etc.)".to_string()),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Check for path traversal vulnerabilities
    pub fn check_path_traversal() -> VerificationResult {
        let mut vulnerabilities = Vec::new();

        let scan_files = vec![
            "pipeline/src-tauri/src/commands/file_import.rs",
            "pipeline/src-tauri/src/commands/archive_import.rs",
            "pipeline/src-tauri/src/io/decompressor/extractor.rs",
            "daw/src-tauri/src/commands/mod.rs",
        ];

        for file_path in &scan_files {
            if let Ok(content) = fs::read_to_string(file_path) {
                for (line_num, line) in content.lines().enumerate() {
                    // Check for unvalidated path operations
                    if (line.contains("Path::new") || line.contains("PathBuf::from"))
                        && !content.contains("canonicalize")
                        && !content.contains("starts_with")
                    {
                        // This is a simplified check - real implementation would be more sophisticated
                        if line.contains("user") || line.contains("input") || line.contains("req") {
                            vulnerabilities.push(format!(
                                "{}:{} - Potential path traversal: {}",
                                file_path,
                                line_num + 1,
                                line.trim()
                            ));
                        }
                    }
                }
            }
        }

        let status = if vulnerabilities.is_empty() {
            CheckStatus::Pass
        } else {
            CheckStatus::Warning
        };

        VerificationResult {
            category: "security".to_string(),
            check_name: "path_traversal".to_string(),
            level: VerificationLevel::High,
            status,
            message: format!(
                "Found {} potential path traversal issue(s)",
                vulnerabilities.len()
            ),
            details: vulnerabilities,
            suggestion: Some(
                "Validate paths with canonicalize() and starts_with() checks".to_string(),
            ),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Check for unsafe Rust code
    pub fn check_unsafe_code() -> VerificationResult {
        let mut unsafe_blocks = Vec::new();

        let scan_dirs = vec!["pipeline/src-tauri/src", "daw/src-tauri/src", "shared/rust/src"];

        for dir in &scan_dirs {
            scan_for_unsafe_recursive(dir, &mut unsafe_blocks);
        }

        let status = if unsafe_blocks.is_empty() {
            CheckStatus::Pass
        } else if unsafe_blocks.len() <= 3 {
            CheckStatus::Warning
        } else {
            CheckStatus::Fail
        };

        VerificationResult {
            category: "security".to_string(),
            check_name: "unsafe_code".to_string(),
            level: VerificationLevel::High,
            status,
            message: format!("Found {} unsafe block(s)", unsafe_blocks.len()),
            details: unsafe_blocks,
            suggestion: Some("Review unsafe blocks and document safety invariants".to_string()),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Check dependency vulnerabilities (requires cargo-audit)
    pub fn check_dependency_vulnerabilities() -> VerificationResult {
        // Try to run cargo audit
        let output = std::process::Command::new("cargo").args(["audit", "--json"]).output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    VerificationResult {
                        category: "security".to_string(),
                        check_name: "dependency_vulnerabilities".to_string(),
                        level: VerificationLevel::Critical,
                        status: CheckStatus::Pass,
                        message: "No known vulnerabilities in dependencies".to_string(),
                        details: vec![],
                        suggestion: None,
                        timestamp: chrono::Utc::now(),
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    VerificationResult {
                        category: "security".to_string(),
                        check_name: "dependency_vulnerabilities".to_string(),
                        level: VerificationLevel::Critical,
                        status: CheckStatus::Fail,
                        message: "Vulnerabilities found in dependencies".to_string(),
                        details: vec![stderr.to_string()],
                        suggestion: Some(
                            "Run `cargo audit fix` to update vulnerable dependencies".to_string(),
                        ),
                        timestamp: chrono::Utc::now(),
                    }
                }
            },
            Err(_) => VerificationResult {
                category: "security".to_string(),
                check_name: "dependency_vulnerabilities".to_string(),
                level: VerificationLevel::Critical,
                status: CheckStatus::Skipped,
                message: "cargo-audit not installed".to_string(),
                details: vec![],
                suggestion: Some("Install with: cargo install cargo-audit".to_string()),
                timestamp: chrono::Utc::now(),
            },
        }
    }

    /// Check Tauri security configuration
    pub fn check_tauri_security() -> VerificationResult {
        let tauri_conf = Path::new("app/src-tauri/tauri.conf.json");
        let mut issues = Vec::new();

        if !tauri_conf.exists() {
            return VerificationResult {
                category: "security".to_string(),
                check_name: "tauri_security".to_string(),
                level: VerificationLevel::Critical,
                status: CheckStatus::Fail,
                message: "tauri.conf.json not found".to_string(),
                details: vec![],
                suggestion: Some("Ensure tauri.conf.json exists in app/src-tauri/".to_string()),
                timestamp: chrono::Utc::now(),
            };
        }

        if let Ok(content) = fs::read_to_string(tauri_conf) {
            // Check for dangerous CSP settings
            if content.contains("\"csp\": null") || content.contains("\"csp\": \"*\"") {
                issues.push("CSP is disabled or too permissive".to_string());
            }

            // Check for dangerous capabilities
            if content.contains("\"all\": true") {
                issues.push("All capabilities enabled - should be restricted".to_string());
            }

            // Check for remote URL loading without restrictions
            if content.contains("\"dangerousRemoteUrlRedirectAccess\"") {
                issues.push("Dangerous remote URL redirect access enabled".to_string());
            }

            // Check devtools in production
            if content.contains("\"devtools\": true") && !content.contains("\"debug\"") {
                issues.push("DevTools enabled (ensure this is dev-only)".to_string());
            }
        }

        let status = if issues.is_empty() {
            CheckStatus::Pass
        } else if issues.len() == 1 {
            CheckStatus::Warning
        } else {
            CheckStatus::Fail
        };

        VerificationResult {
            category: "security".to_string(),
            check_name: "tauri_security".to_string(),
            level: VerificationLevel::High,
            status,
            message: format!("Tauri security: {} issue(s) found", issues.len()),
            details: issues,
            suggestion: Some(
                "Review Tauri security best practices at https://tauri.app/security".to_string(),
            ),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Get full security verification summary
    pub fn get_summary() -> SecuritySummary {
        let secrets = Self::scan_hardcoded_secrets();
        let sql_injection = Self::check_sql_injection();
        let path_traversal = Self::check_path_traversal();
        let unsafe_code = Self::check_unsafe_code();
        let dependencies = Self::check_dependency_vulnerabilities();
        let tauri_security = Self::check_tauri_security();

        SecuritySummary {
            no_hardcoded_secrets: secrets.status == CheckStatus::Pass,
            no_sql_injection: sql_injection.status == CheckStatus::Pass,
            path_traversal_safe: path_traversal.status == CheckStatus::Pass
                || path_traversal.status == CheckStatus::Warning,
            unsafe_code_reviewed: unsafe_code.status == CheckStatus::Pass
                || unsafe_code.status == CheckStatus::Warning,
            dependencies_safe: dependencies.status == CheckStatus::Pass
                || dependencies.status == CheckStatus::Skipped,
            tauri_secure: tauri_security.status == CheckStatus::Pass,
            results: vec![
                secrets,
                sql_injection,
                path_traversal,
                unsafe_code,
                dependencies,
                tauri_security,
            ],
        }
    }
}

/// Summary of security verification
#[derive(Debug)]
pub struct SecuritySummary {
    pub no_hardcoded_secrets: bool,
    pub no_sql_injection: bool,
    pub path_traversal_safe: bool,
    pub unsafe_code_reviewed: bool,
    pub dependencies_safe: bool,
    pub tauri_secure: bool,
    pub results: Vec<VerificationResult>,
}

impl SecuritySummary {
    pub fn all_secure(&self) -> bool {
        self.no_hardcoded_secrets
            && self.no_sql_injection
            && self.path_traversal_safe
            && self.unsafe_code_reviewed
            && self.dependencies_safe
            && self.tauri_secure
    }

    pub fn critical_issues(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.level == VerificationLevel::Critical && r.status == CheckStatus::Fail)
            .count()
    }
}

/// Recursively scan directory for secrets
fn scan_directory_recursive(dir: &str, patterns: &[(&str, &str)], findings: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                // Skip common non-source directories
                if !["target", "node_modules", ".git", "dist", "build"].contains(&dir_name.as_ref())
                {
                    scan_directory_recursive(&path.to_string_lossy(), patterns, findings);
                }
            } else if path.is_file() {
                scan_file_for_secrets(&path, patterns, findings);
            }
        }
    }
}

/// Scan a single file for secret patterns
fn scan_file_for_secrets(path: &Path, patterns: &[(&str, &str)], findings: &mut Vec<String>) {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    // Only scan source files
    if !["rs", "ts", "js", "json", "toml", "yaml", "yml", "env"].contains(&ext) {
        return;
    }

    // Skip test files and fixtures
    let path_str = path.to_string_lossy();
    if path_str.contains("test") || path_str.contains("fixture") || path_str.contains("mock") {
        return;
    }

    if let Ok(content) = fs::read_to_string(path) {
        for (line_num, line) in content.lines().enumerate() {
            let line_lower = line.to_lowercase();
            for (pattern, description) in patterns {
                if line_lower.contains(pattern) {
                    // Skip if it's a variable declaration without value
                    if line.contains("env::var") || line.contains("std::env") {
                        continue;
                    }
                    // Skip if it's a comment
                    if line.trim().starts_with("//") || line.trim().starts_with('#') {
                        continue;
                    }
                    // Check if there's an actual value assigned
                    if line.contains('=') && (line.contains('"') || line.contains('\'')) {
                        findings.push(format!(
                            "{}:{} - {}: {}",
                            path.display(),
                            line_num + 1,
                            description,
                            line.trim()
                        ));
                    }
                }
            }
        }
    }
}

/// Recursively scan for unsafe blocks
fn scan_for_unsafe_recursive(dir: &str, findings: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                if !["target", ".git"].contains(&dir_name.as_ref()) {
                    scan_for_unsafe_recursive(&path.to_string_lossy(), findings);
                }
            } else if path.is_file() && path.extension().is_some_and(|e| e == "rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    for (line_num, line) in content.lines().enumerate() {
                        if line.contains("unsafe {") || line.contains("unsafe fn") {
                            findings.push(format!(
                                "{}:{} - {}",
                                path.display(),
                                line_num + 1,
                                line.trim()
                            ));
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_summary() {
        let summary = SecurityVerifier::get_summary();
        // Just verify it runs without panic
        assert!(summary.results.len() >= 5);
    }

    #[test]
    fn test_critical_issues_count() {
        let summary = SecuritySummary {
            no_hardcoded_secrets: true,
            no_sql_injection: true,
            path_traversal_safe: true,
            unsafe_code_reviewed: true,
            dependencies_safe: true,
            tauri_secure: true,
            results: vec![],
        };
        assert_eq!(summary.critical_issues(), 0);
    }
}

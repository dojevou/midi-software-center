//! Dependency Verification Module
//!
//! Verifies cross-component version compatibility and dependency consistency.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Dependency verification error
#[derive(Debug, Clone)]
pub struct DependencyError {
    pub category: String,
    pub package: String,
    pub message: String,
    pub severity: DependencyErrorSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyErrorSeverity {
    Critical,
    Warning,
    Info,
}

/// Cargo package information
#[derive(Debug, Clone)]
pub struct CargoPackage {
    pub name: String,
    pub version: String,
    pub edition: String,
}

/// Verifies workspace dependencies
pub struct DependencyVerifier;

impl DependencyVerifier {
    /// Verify workspace consistency across all crates
    pub fn verify_workspace_consistency() -> Result<(), Vec<DependencyError>> {
        let mut errors = Vec::new();

        // Unified architecture: only the main app and helper scripts
        let crates = vec![
            ("./app/src-tauri", "midi-software-center"),
            ("./scripts/import-tool", "import-tool"),
            ("./scripts/test-midi-files", "test-midi-files"),
            ("./verification", "verification"),
        ];

        let mut versions: HashMap<String, String> = HashMap::new();
        let mut editions: HashMap<String, String> = HashMap::new();

        for (path, name) in &crates {
            let cargo_path = format!("{}/Cargo.toml", path);
            if let Ok(content) = fs::read_to_string(&cargo_path) {
                if let Some(version) = Self::extract_version(&content) {
                    versions.insert(name.to_string(), version);
                }
                if let Some(edition) = Self::extract_edition(&content) {
                    editions.insert(name.to_string(), edition);
                }
            } else {
                errors.push(DependencyError {
                    category: "workspace".to_string(),
                    package: name.to_string(),
                    message: format!("Cannot read Cargo.toml at {}", cargo_path),
                    severity: DependencyErrorSeverity::Critical,
                });
            }
        }

        // Check Rust edition consistency
        let unique_editions: std::collections::HashSet<&String> = editions.values().collect();
        if unique_editions.len() > 1 {
            errors.push(DependencyError {
                category: "edition".to_string(),
                package: "workspace".to_string(),
                message: format!("Mixed Rust editions detected: {:?}", editions),
                severity: DependencyErrorSeverity::Warning,
            });
        }

        // Unified architecture - no need to check shared library version consistency
        // All code is now in midi-software-center

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Verify frontend dependencies
    pub fn verify_frontend_dependencies() -> Result<(), Vec<DependencyError>> {
        let mut errors = Vec::new();

        let package_json_path = "./app/package.json";
        if !Path::new(package_json_path).exists() {
            errors.push(DependencyError {
                category: "frontend".to_string(),
                package: "package.json".to_string(),
                message: "app/package.json missing".to_string(),
                severity: DependencyErrorSeverity::Critical,
            });
            return Err(errors);
        }

        if let Ok(content) = fs::read_to_string(package_json_path) {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(package) => {
                    // Check for required dependencies
                    let required_deps =
                        vec![("@tauri-apps/api", "Tauri API"), ("svelte", "Svelte framework")];

                    if let Some(deps) = package.get("dependencies") {
                        for (dep, name) in &required_deps {
                            if deps.get(*dep).is_none() {
                                errors.push(DependencyError {
                                    category: "frontend".to_string(),
                                    package: dep.to_string(),
                                    message: format!("{} is not listed in dependencies", name),
                                    severity: DependencyErrorSeverity::Critical,
                                });
                            }
                        }
                    }

                    // Check for required dev dependencies
                    let required_dev_deps = vec![
                        ("typescript", "TypeScript"),
                        ("vite", "Vite bundler"),
                        ("@sveltejs/vite-plugin-svelte", "Svelte Vite plugin"),
                    ];

                    if let Some(dev_deps) = package.get("devDependencies") {
                        for (dep, name) in &required_dev_deps {
                            if dev_deps.get(*dep).is_none() {
                                errors.push(DependencyError {
                                    category: "frontend".to_string(),
                                    package: dep.to_string(),
                                    message: format!("{} is not listed in devDependencies", name),
                                    severity: DependencyErrorSeverity::Warning,
                                });
                            }
                        }
                    }
                },
                Err(e) => {
                    errors.push(DependencyError {
                        category: "frontend".to_string(),
                        package: "package.json".to_string(),
                        message: format!("Invalid JSON in package.json: {}", e),
                        severity: DependencyErrorSeverity::Critical,
                    });
                },
            }
        }

        // Check for lock file
        if !Path::new("./app/pnpm-lock.yaml").exists() {
            errors.push(DependencyError {
                category: "frontend".to_string(),
                package: "pnpm-lock.yaml".to_string(),
                message: "Lock file missing - run 'pnpm install'".to_string(),
                severity: DependencyErrorSeverity::Warning,
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Verify Tauri version compatibility
    pub fn verify_tauri_compatibility() -> Result<(), Vec<DependencyError>> {
        let mut errors = Vec::new();

        // Get Tauri Rust version
        let tauri_rust_version =
            Self::get_cargo_dependency_version("./app/src-tauri/Cargo.toml", "tauri");

        // Get Tauri JS version
        let tauri_js_version =
            Self::get_npm_dependency_version("./app/package.json", "@tauri-apps/api");

        if let (Some(rust_ver), Some(js_ver)) = (tauri_rust_version, tauri_js_version) {
            // Extract major versions
            let rust_major = rust_ver.split('.').next().unwrap_or("0");
            let js_major = js_ver.split('.').next().unwrap_or("0");

            if rust_major != js_major {
                errors.push(DependencyError {
                    category: "tauri".to_string(),
                    package: "tauri".to_string(),
                    message: format!(
                        "Tauri version mismatch: Rust tauri {} vs JS @tauri-apps/api {}",
                        rust_ver, js_ver
                    ),
                    severity: DependencyErrorSeverity::Critical,
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Extract version from Cargo.toml content
    fn extract_version(content: &str) -> Option<String> {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("version") {
                if let Some(value) = line.split('=').nth(1) {
                    return Some(value.trim().trim_matches('"').to_string());
                }
            }
        }
        None
    }

    /// Extract edition from Cargo.toml content
    fn extract_edition(content: &str) -> Option<String> {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("edition") {
                if let Some(value) = line.split('=').nth(1) {
                    return Some(value.trim().trim_matches('"').to_string());
                }
            }
        }
        None
    }

    /// Get dependency version from a crate
    #[allow(dead_code)]
    fn get_dependency_version(crate_name: &str, dep_name: &str) -> Option<String> {
        let paths = vec![
            format!("./app/src-tauri/Cargo.toml"),
            format!("./pipeline/src-tauri/Cargo.toml"),
            format!("./daw/src-tauri/Cargo.toml"),
        ];

        for path in paths {
            if let Ok(content) = fs::read_to_string(&path) {
                if content.contains(crate_name) {
                    // Find the dependency
                    let mut in_deps = false;
                    for line in content.lines() {
                        if line.starts_with("[dependencies]") {
                            in_deps = true;
                            continue;
                        }
                        if line.starts_with('[') && !line.starts_with("[dependencies") {
                            in_deps = false;
                        }
                        if in_deps && line.contains(dep_name) {
                            if let Some(version) = line.split('=').nth(1) {
                                return Some(version.trim().trim_matches('"').to_string());
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Get Cargo dependency version from a specific file
    fn get_cargo_dependency_version(cargo_path: &str, dep_name: &str) -> Option<String> {
        if let Ok(content) = fs::read_to_string(cargo_path) {
            let mut in_deps = false;
            for line in content.lines() {
                if line.starts_with("[dependencies]") {
                    in_deps = true;
                    continue;
                }
                if line.starts_with('[') && !line.starts_with("[dependencies") {
                    in_deps = false;
                }
                if in_deps && line.contains(dep_name) && line.contains('=') {
                    // Handle both simple and table formats
                    if line.contains("version") {
                        // Table format: tauri = { version = "2.0", features = [...] }
                        if let Some(start) = line.find("version") {
                            let rest = &line[start..];
                            if let Some(eq) = rest.find('=') {
                                let value_start = &rest[eq + 1..];
                                let value = value_start.trim().trim_matches(|c| {
                                    c == '"' || c == '\'' || c == ' ' || c == ','
                                });
                                if let Some(end) = value.find(['"', '\'', ',', '}']) {
                                    return Some(value[..end].to_string());
                                }
                                return Some(value.to_string());
                            }
                        }
                    } else {
                        // Simple format: tauri = "2.0"
                        if let Some(version) = line.split('=').nth(1) {
                            return Some(version.trim().trim_matches('"').to_string());
                        }
                    }
                }
            }
        }
        None
    }

    /// Get NPM dependency version
    fn get_npm_dependency_version(package_path: &str, dep_name: &str) -> Option<String> {
        if let Ok(content) = fs::read_to_string(package_path) {
            if let Ok(package) = serde_json::from_str::<serde_json::Value>(&content) {
                // Check dependencies
                if let Some(deps) = package.get("dependencies") {
                    if let Some(version) = deps.get(dep_name) {
                        return version.as_str().map(|s| s.trim_start_matches('^').to_string());
                    }
                }
                // Check devDependencies
                if let Some(deps) = package.get("devDependencies") {
                    if let Some(version) = deps.get(dep_name) {
                        return version.as_str().map(|s| s.trim_start_matches('^').to_string());
                    }
                }
            }
        }
        None
    }

    /// Get dependency summary
    pub fn get_summary() -> DependencySummary {
        let workspace_valid = Self::verify_workspace_consistency().is_ok();
        let frontend_valid = Self::verify_frontend_dependencies().is_ok();
        let tauri_valid = Self::verify_tauri_compatibility().is_ok();

        DependencySummary {
            workspace_consistent: workspace_valid,
            frontend_valid,
            tauri_compatible: tauri_valid,
        }
    }
}

/// Dependency verification summary
#[derive(Debug, Clone)]
pub struct DependencySummary {
    pub workspace_consistent: bool,
    pub frontend_valid: bool,
    pub tauri_compatible: bool,
}

impl DependencySummary {
    pub fn all_valid(&self) -> bool {
        self.workspace_consistent && self.frontend_valid && self.tauri_compatible
    }
}

impl std::fmt::Display for DependencySummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Dependency Summary:\n  Workspace: {}\n  Frontend: {}\n  Tauri: {}",
            if self.workspace_consistent {
                "OK"
            } else {
                "FAILED"
            },
            if self.frontend_valid { "OK" } else { "FAILED" },
            if self.tauri_compatible {
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
    fn test_extract_version() {
        let content = r#"
[package]
name = "test"
version = "1.0.0"
edition = "2021"
"#;
        assert_eq!(
            DependencyVerifier::extract_version(content),
            Some("1.0.0".to_string())
        );
    }

    #[test]
    fn test_extract_edition() {
        let content = r#"
[package]
name = "test"
version = "1.0.0"
edition = "2021"
"#;
        assert_eq!(
            DependencyVerifier::extract_edition(content),
            Some("2021".to_string())
        );
    }

    #[test]
    fn test_dependency_summary_display() {
        let summary = DependencySummary {
            workspace_consistent: true,
            frontend_valid: true,
            tauri_compatible: false,
        };
        let display = format!("{}", summary);
        assert!(display.contains("Workspace: OK"));
        assert!(display.contains("Tauri: FAILED"));
    }
}

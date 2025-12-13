//! Build Verification Module
//!
//! Verifies build environment and compilation status.

use std::process::Command;

/// Build verification error
#[derive(Debug, Clone)]
pub struct BuildError {
    pub category: String,
    pub tool: String,
    pub message: String,
    pub severity: BuildErrorSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildErrorSeverity {
    Critical,
    Warning,
    Info,
}

/// Verifies build environment and compilation
pub struct BuildVerifier;

impl BuildVerifier {
    /// Verify build environment has all required tools
    pub fn verify_build_environment() -> Result<(), Vec<BuildError>> {
        let mut errors = Vec::new();

        // Check Rust toolchain
        match Command::new("rustc").arg("--version").output() {
            Ok(output) => {
                let version = String::from_utf8_lossy(&output.stdout);
                println!("Rust version: {}", version.trim());

                // Extract version number
                if let Some(ver) = version.split_whitespace().nth(1) {
                    let parts: Vec<&str> = ver.split('.').collect();
                    if parts.len() >= 2 {
                        if let (Ok(major), Ok(minor)) =
                            (parts[0].parse::<u32>(), parts[1].parse::<u32>())
                        {
                            if major < 1 || (major == 1 && minor < 70) {
                                errors.push(BuildError {
                                    category: "toolchain".to_string(),
                                    tool: "rustc".to_string(),
                                    message: format!("Rust version {} is below minimum 1.70", ver),
                                    severity: BuildErrorSeverity::Critical,
                                });
                            }
                        }
                    }
                }
            },
            Err(_) => {
                errors.push(BuildError {
                    category: "toolchain".to_string(),
                    tool: "rustc".to_string(),
                    message: "Rust compiler not found".to_string(),
                    severity: BuildErrorSeverity::Critical,
                });
            },
        }

        // Check Cargo
        match Command::new("cargo").arg("--version").output() {
            Ok(output) => {
                let version = String::from_utf8_lossy(&output.stdout);
                println!("Cargo version: {}", version.trim());
            },
            Err(_) => {
                errors.push(BuildError {
                    category: "toolchain".to_string(),
                    tool: "cargo".to_string(),
                    message: "Cargo not found".to_string(),
                    severity: BuildErrorSeverity::Critical,
                });
            },
        }

        // Check Node.js
        match Command::new("node").arg("--version").output() {
            Ok(output) => {
                let version = String::from_utf8_lossy(&output.stdout);
                println!("Node.js version: {}", version.trim());

                // Check minimum version (v18+)
                let ver = version.trim().trim_start_matches('v');
                if let Ok(major) = ver.split('.').next().unwrap_or("0").parse::<u32>() {
                    if major < 18 {
                        errors.push(BuildError {
                            category: "toolchain".to_string(),
                            tool: "node".to_string(),
                            message: format!("Node.js version {} is below minimum v18", ver),
                            severity: BuildErrorSeverity::Warning,
                        });
                    }
                }
            },
            Err(_) => {
                errors.push(BuildError {
                    category: "toolchain".to_string(),
                    tool: "node".to_string(),
                    message: "Node.js not found".to_string(),
                    severity: BuildErrorSeverity::Critical,
                });
            },
        }

        // Check pnpm
        match Command::new("pnpm").arg("--version").output() {
            Ok(output) => {
                let version = String::from_utf8_lossy(&output.stdout);
                println!("pnpm version: {}", version.trim());
            },
            Err(_) => {
                errors.push(BuildError {
                    category: "toolchain".to_string(),
                    tool: "pnpm".to_string(),
                    message: "pnpm not found - install with 'npm install -g pnpm'".to_string(),
                    severity: BuildErrorSeverity::Critical,
                });
            },
        }

        // Check optional tools (Info severity - don't fail verification)
        Self::check_optional_tool("wasm-pack", "WebAssembly support");
        Self::check_optional_tool("cargo-tarpaulin", "Code coverage");

        // Only fail on Critical errors
        let critical_errors: Vec<_> = errors
            .into_iter()
            .filter(|e| e.severity == BuildErrorSeverity::Critical)
            .collect();

        if critical_errors.is_empty() {
            Ok(())
        } else {
            Err(critical_errors)
        }
    }

    /// Check for optional tools (just prints info, doesn't add errors)
    fn check_optional_tool(tool: &str, purpose: &str) {
        if Command::new(tool).arg("--version").output().is_err() {
            println!("  Optional: {} not found - needed for {}", tool, purpose);
        }
    }

    /// Verify workspace crates compile
    pub fn verify_crate_compilation() -> Result<(), Vec<BuildError>> {
        let mut errors = Vec::new();

        println!("Verifying workspace compilation...");

        // Run cargo check on workspace
        match Command::new("cargo").args(["check", "--workspace"]).output() {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    errors.push(BuildError {
                        category: "compilation".to_string(),
                        tool: "cargo".to_string(),
                        message: format!("Workspace compilation failed:\n{}", stderr),
                        severity: BuildErrorSeverity::Critical,
                    });
                } else {
                    println!("Workspace compilation: OK");
                }
            },
            Err(e) => {
                errors.push(BuildError {
                    category: "compilation".to_string(),
                    tool: "cargo".to_string(),
                    message: format!("Failed to run cargo check: {}", e),
                    severity: BuildErrorSeverity::Critical,
                });
            },
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Verify frontend builds
    pub fn verify_frontend_build() -> Result<(), Vec<BuildError>> {
        let mut errors = Vec::new();

        println!("Verifying frontend build...");

        // Check if node_modules exists
        if !std::path::Path::new("./app/node_modules").exists() {
            errors.push(BuildError {
                category: "frontend".to_string(),
                tool: "pnpm".to_string(),
                message: "node_modules not found - run 'pnpm install' in app directory".to_string(),
                severity: BuildErrorSeverity::Warning,
            });
        }

        // Run TypeScript check
        match Command::new("pnpm").args(["run", "check"]).current_dir("./app").output() {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    errors.push(BuildError {
                        category: "frontend".to_string(),
                        tool: "svelte-check".to_string(),
                        message: format!("TypeScript check failed:\n{}{}", stdout, stderr),
                        severity: BuildErrorSeverity::Critical,
                    });
                } else {
                    println!("Frontend TypeScript check: OK");
                }
            },
            Err(e) => {
                errors.push(BuildError {
                    category: "frontend".to_string(),
                    tool: "pnpm".to_string(),
                    message: format!("Failed to run type check: {}", e),
                    severity: BuildErrorSeverity::Warning,
                });
            },
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Verify tests pass
    pub fn verify_tests() -> Result<(), Vec<BuildError>> {
        let mut errors = Vec::new();

        println!("Running workspace tests...");

        match Command::new("cargo")
            .args(["test", "--workspace", "--lib", "--", "--test-threads=1"])
            .output()
        {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    errors.push(BuildError {
                        category: "tests".to_string(),
                        tool: "cargo".to_string(),
                        message: format!("Tests failed:\n{}", stderr),
                        severity: BuildErrorSeverity::Critical,
                    });
                } else {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    // Extract test summary
                    if let Some(summary_line) = stdout.lines().find(|l| l.contains("test result:"))
                    {
                        println!("Tests: {}", summary_line);
                    } else {
                        println!("Tests: OK");
                    }
                }
            },
            Err(e) => {
                errors.push(BuildError {
                    category: "tests".to_string(),
                    tool: "cargo".to_string(),
                    message: format!("Failed to run tests: {}", e),
                    severity: BuildErrorSeverity::Critical,
                });
            },
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Verify code formatting
    pub fn verify_formatting() -> Result<(), Vec<BuildError>> {
        let mut errors = Vec::new();

        println!("Checking code formatting...");

        // Check Rust formatting
        match Command::new("cargo").args(["fmt", "--", "--check"]).output() {
            Ok(output) => {
                if !output.status.success() {
                    errors.push(BuildError {
                        category: "formatting".to_string(),
                        tool: "rustfmt".to_string(),
                        message: "Rust code is not formatted - run 'cargo fmt'".to_string(),
                        severity: BuildErrorSeverity::Warning,
                    });
                } else {
                    println!("Rust formatting: OK");
                }
            },
            Err(e) => {
                errors.push(BuildError {
                    category: "formatting".to_string(),
                    tool: "rustfmt".to_string(),
                    message: format!("Failed to check formatting: {}", e),
                    severity: BuildErrorSeverity::Info,
                });
            },
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get build verification summary
    pub fn get_summary() -> BuildSummary {
        let environment_ok = Self::verify_build_environment().is_ok();
        let compilation_ok = Self::verify_crate_compilation().is_ok();
        let frontend_ok = Self::verify_frontend_build().is_ok();
        let formatting_ok = Self::verify_formatting().is_ok();

        BuildSummary {
            environment_valid: environment_ok,
            compilation_passes: compilation_ok,
            frontend_builds: frontend_ok,
            formatting_correct: formatting_ok,
        }
    }
}

/// Build verification summary
#[derive(Debug, Clone)]
pub struct BuildSummary {
    pub environment_valid: bool,
    pub compilation_passes: bool,
    pub frontend_builds: bool,
    pub formatting_correct: bool,
}

impl BuildSummary {
    pub fn all_valid(&self) -> bool {
        self.environment_valid && self.compilation_passes && self.frontend_builds
    }
}

impl std::fmt::Display for BuildSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Build Summary:\n  Environment: {}\n  Compilation: {}\n  Frontend: {}\n  Formatting: {}",
            if self.environment_valid { "OK" } else { "FAILED" },
            if self.compilation_passes { "OK" } else { "FAILED" },
            if self.frontend_builds { "OK" } else { "FAILED" },
            if self.formatting_correct { "OK" } else { "NEEDS FMT" },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_error_severity() {
        let error = BuildError {
            category: "test".to_string(),
            tool: "test-tool".to_string(),
            message: "Test message".to_string(),
            severity: BuildErrorSeverity::Critical,
        };
        assert_eq!(error.severity, BuildErrorSeverity::Critical);
    }

    #[test]
    fn test_build_summary_display() {
        let summary = BuildSummary {
            environment_valid: true,
            compilation_passes: true,
            frontend_builds: false,
            formatting_correct: true,
        };
        let display = format!("{}", summary);
        assert!(display.contains("Environment: OK"));
        assert!(display.contains("Frontend: FAILED"));
    }

    #[test]
    fn test_build_summary_all_valid() {
        let summary = BuildSummary {
            environment_valid: true,
            compilation_passes: true,
            frontend_builds: true,
            formatting_correct: false, // formatting doesn't affect all_valid
        };
        assert!(summary.all_valid());
    }
}

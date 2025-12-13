//! Path Verification Module
//!
//! Verifies file system paths and directory structure.

use std::fs;
use std::path::Path;

/// Path verification error
#[derive(Debug, Clone)]
pub struct PathError {
    pub path: String,
    pub error_type: PathErrorType,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathErrorType {
    Missing,
    NotAccessible,
    WrongType, // Expected dir got file or vice versa
    Empty,
}

/// Verifies application and data paths
pub struct PathVerifier;

impl PathVerifier {
    /// Verify all required application directories exist
    pub fn verify_application_paths() -> Result<(), Vec<PathError>> {
        let mut errors = Vec::new();

        // Required directories for the application
        let required_dirs = vec![
            ("./app", "Frontend application directory"),
            ("./app/src-tauri", "Tauri backend directory"),
            ("./app/src", "Frontend source directory"),
            ("./app/src/lib", "Frontend library directory"),
            ("./pipeline", "Pipeline application directory"),
            ("./pipeline/src-tauri", "Pipeline backend directory"),
            ("./daw", "DAW application directory"),
            ("./daw/src-tauri", "DAW backend directory"),
            ("./shared", "Shared libraries directory"),
            ("./shared/rust", "Shared Rust library"),
            ("./database", "Database migrations directory"),
            ("./database/migrations", "SQL migrations"),
            ("./verification", "Verification scripts directory"),
        ];

        for (dir, description) in required_dirs {
            let path = Path::new(dir);
            if !path.exists() {
                errors.push(PathError {
                    path: dir.to_string(),
                    error_type: PathErrorType::Missing,
                    message: format!("{} is missing", description),
                });
            } else if !path.is_dir() {
                errors.push(PathError {
                    path: dir.to_string(),
                    error_type: PathErrorType::WrongType,
                    message: format!("{} exists but is not a directory", description),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Verify critical source files exist
    pub fn verify_critical_files() -> Result<(), Vec<PathError>> {
        let mut errors = Vec::new();

        let critical_files = vec![
            // Rust configuration
            ("./Cargo.toml", "Workspace Cargo.toml"),
            ("./rustfmt.toml", "Rust formatter config"),
            // App package
            ("./app/src-tauri/src/main.rs", "App main entry point"),
            ("./app/src-tauri/src/lib.rs", "App library entry point"),
            ("./app/src-tauri/Cargo.toml", "App Cargo.toml"),
            ("./app/src-tauri/tauri.conf.json", "Tauri configuration"),
            ("./app/package.json", "Frontend package.json"),
            ("./app/vite.config.ts", "Vite configuration"),
            ("./app/tsconfig.json", "TypeScript configuration"),
            // Frontend API
            ("./app/src/lib/api.ts", "Frontend API layer"),
            ("./app/src/lib/api/commands.ts", "Command registry"),
            ("./app/src/lib/types.ts", "TypeScript type definitions"),
            // Pipeline package
            ("./pipeline/src-tauri/src/lib.rs", "Pipeline library"),
            ("./pipeline/src-tauri/Cargo.toml", "Pipeline Cargo.toml"),
            // DAW package
            ("./daw/src-tauri/src/main.rs", "DAW main entry point"),
            ("./daw/src-tauri/Cargo.toml", "DAW Cargo.toml"),
            // Shared library
            ("./shared/rust/src/lib.rs", "Shared library entry point"),
            ("./shared/rust/Cargo.toml", "Shared Cargo.toml"),
            // Documentation
            ("./CLAUDE.md", "Project documentation"),
        ];

        for (file, description) in critical_files {
            let path = Path::new(file);
            if !path.exists() {
                errors.push(PathError {
                    path: file.to_string(),
                    error_type: PathErrorType::Missing,
                    message: format!("{} is missing", description),
                });
            } else if !path.is_file() {
                errors.push(PathError {
                    path: file.to_string(),
                    error_type: PathErrorType::WrongType,
                    message: format!("{} exists but is not a file", description),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Verify data directories exist and are accessible
    pub fn verify_data_paths() -> Result<(), Vec<PathError>> {
        let mut errors = Vec::new();

        // Check MIDI splits directory
        let splits_dir = Path::new("/home/dojevou/tmp/midi_splits_fast");
        if splits_dir.exists() {
            match fs::read_dir(splits_dir) {
                Ok(entries) => {
                    let count = entries.count();
                    if count == 0 {
                        errors.push(PathError {
                            path: splits_dir.to_string_lossy().to_string(),
                            error_type: PathErrorType::Empty,
                            message: "MIDI splits directory exists but is empty".to_string(),
                        });
                    }
                },
                Err(e) => {
                    errors.push(PathError {
                        path: splits_dir.to_string_lossy().to_string(),
                        error_type: PathErrorType::NotAccessible,
                        message: format!("Cannot read MIDI splits directory: {}", e),
                    });
                },
            }
        } else {
            errors.push(PathError {
                path: splits_dir.to_string_lossy().to_string(),
                error_type: PathErrorType::Missing,
                message: "MIDI splits directory does not exist".to_string(),
            });
        }

        // Check MIDI library base directory
        let library_dir = Path::new("/home/dojevou/projects/midi-software-center/midi-library");
        if library_dir.exists() && !library_dir.is_dir() {
            errors.push(PathError {
                path: library_dir.to_string_lossy().to_string(),
                error_type: PathErrorType::WrongType,
                message: "midi-library path exists but is not a directory".to_string(),
            });
        }
        // Note: midi-library is optional, so we don't error if missing

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Verify build output directories
    pub fn verify_build_outputs() -> Result<(), Vec<PathError>> {
        let errors = Vec::new();

        let build_dirs = vec![
            ("./target", "Rust build output"),
            ("./app/dist", "Frontend build output"),
            ("./app/node_modules", "Node.js dependencies"),
        ];

        for (dir, description) in build_dirs {
            let path = Path::new(dir);
            if !path.exists() {
                // Build outputs are warnings, not errors
                println!(
                    "Warning: {} ({}) not found - run build first",
                    description, dir
                );
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Check if a path is within allowed directories (security check)
    pub fn is_path_allowed(path: &Path) -> bool {
        let allowed_prefixes =
            ["/home/dojevou/projects/midi-software-center", "/home/dojevou/tmp", "/tmp"];

        let canonical = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => return false,
        };

        allowed_prefixes.iter().any(|prefix| canonical.starts_with(prefix))
    }

    /// Get path verification summary
    pub fn get_summary() -> PathSummary {
        let app_paths = Self::verify_application_paths().is_ok();
        let critical_files = Self::verify_critical_files().is_ok();
        let data_paths = Self::verify_data_paths().is_ok();
        let build_outputs = Self::verify_build_outputs().is_ok();

        PathSummary {
            application_paths_valid: app_paths,
            critical_files_present: critical_files,
            data_paths_accessible: data_paths,
            build_outputs_present: build_outputs,
        }
    }
}

/// Summary of path verification results
#[derive(Debug, Clone)]
pub struct PathSummary {
    pub application_paths_valid: bool,
    pub critical_files_present: bool,
    pub data_paths_accessible: bool,
    pub build_outputs_present: bool,
}

impl PathSummary {
    pub fn all_valid(&self) -> bool {
        self.application_paths_valid && self.critical_files_present && self.data_paths_accessible
    }
}

impl std::fmt::Display for PathSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Path Summary:\n  Application Paths: {}\n  Critical Files: {}\n  Data Paths: {}\n  Build Outputs: {}",
            if self.application_paths_valid { "OK" } else { "FAILED" },
            if self.critical_files_present { "OK" } else { "FAILED" },
            if self.data_paths_accessible { "OK" } else { "FAILED" },
            if self.build_outputs_present { "OK" } else { "MISSING" },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_error_types() {
        let error = PathError {
            path: "/test/path".to_string(),
            error_type: PathErrorType::Missing,
            message: "Test error".to_string(),
        };
        assert_eq!(error.error_type, PathErrorType::Missing);
    }

    #[test]
    fn test_is_path_allowed() {
        // Test with /tmp which always exists
        assert!(PathVerifier::is_path_allowed(Path::new("/tmp")));
        // Test with a path that exists but is not allowed
        assert!(!PathVerifier::is_path_allowed(Path::new("/etc")));
    }

    #[test]
    fn test_path_summary_display() {
        let summary = PathSummary {
            application_paths_valid: true,
            critical_files_present: true,
            data_paths_accessible: false,
            build_outputs_present: true,
        };
        let display = format!("{}", summary);
        assert!(display.contains("Application Paths: OK"));
        assert!(display.contains("Data Paths: FAILED"));
    }
}

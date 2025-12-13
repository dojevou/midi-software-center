//! API Commands Verification
//!
//! Verifies that all Tauri commands are properly registered and documented.

use crate::shared::{CheckStatus, VerificationLevel, VerificationResult};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// API Commands Verifier
pub struct ApiCommandsVerifier;

impl ApiCommandsVerifier {
    /// Verify all API commands are properly registered
    pub fn verify_command_registration() -> VerificationResult {
        let mut declared_commands = HashSet::new();
        let mut registered_commands = HashSet::new();
        let mut details = Vec::new();

        // Scan for declared commands in source files
        let command_files = vec![
            "pipeline/src-tauri/src/commands/mod.rs",
            "pipeline/src-tauri/src/commands/file_import.rs",
            "pipeline/src-tauri/src/commands/analyze.rs",
            "pipeline/src-tauri/src/commands/search.rs",
            "pipeline/src-tauri/src/commands/stats.rs",
            "pipeline/src-tauri/src/commands/tags.rs",
            "pipeline/src-tauri/src/commands/system.rs",
            "pipeline/src-tauri/src/commands/archive_import.rs",
            "pipeline/src-tauri/src/commands/split_file.rs",
            "pipeline/src-tauri/src/commands/progress.rs",
            "daw/src-tauri/src/commands/mod.rs",
        ];

        for file_path in &command_files {
            if let Ok(content) = fs::read_to_string(file_path) {
                // Find #[tauri::command] annotated functions
                for line in content.lines() {
                    if line.contains("pub async fn") || line.contains("pub fn") {
                        if let Some(name) = extract_function_name(line) {
                            declared_commands.insert(name);
                        }
                    }
                }
            }
        }

        // Check lib.rs for registered commands
        let lib_files = vec![
            "pipeline/src-tauri/src/lib.rs",
            "daw/src-tauri/src/lib.rs",
            "app/src-tauri/src/lib.rs",
        ];

        for lib_path in &lib_files {
            if let Ok(content) = fs::read_to_string(lib_path) {
                // Find .invoke_handler(tauri::generate_handler![...])
                if let Some(start) = content.find("generate_handler![") {
                    if let Some(end) = content[start..].find(']') {
                        let handler_block = &content[start..start + end];
                        for cmd in handler_block.split(',') {
                            let cmd = cmd
                                .trim()
                                .replace("generate_handler![", "")
                                .replace("commands::", "");
                            if !cmd.is_empty() && !cmd.contains("//") {
                                registered_commands.insert(cmd);
                            }
                        }
                    }
                }
            }
        }

        // Check for unregistered commands
        let unregistered: Vec<_> =
            declared_commands.difference(&registered_commands).cloned().collect();

        if !unregistered.is_empty() {
            details.push(format!("Unregistered commands: {:?}", unregistered));
        }

        let status = if unregistered.is_empty() {
            CheckStatus::Pass
        } else if unregistered.len() > 5 {
            CheckStatus::Fail
        } else {
            CheckStatus::Warning
        };

        VerificationResult {
            category: "api_commands".to_string(),
            check_name: "command_registration".to_string(),
            level: VerificationLevel::High,
            status,
            message: format!(
                "Found {} declared commands, {} registered",
                declared_commands.len(),
                registered_commands.len()
            ),
            details,
            suggestion: if !unregistered.is_empty() {
                Some("Add missing commands to generate_handler![] in lib.rs".to_string())
            } else {
                None
            },
            timestamp: chrono::Utc::now(),
        }
    }

    /// Verify command documentation
    pub fn verify_command_documentation() -> VerificationResult {
        let mut undocumented = Vec::new();
        let mut total_commands = 0;

        let command_files = vec![
            "pipeline/src-tauri/src/commands/file_import.rs",
            "pipeline/src-tauri/src/commands/analyze.rs",
            "pipeline/src-tauri/src/commands/search.rs",
            "pipeline/src-tauri/src/commands/stats.rs",
            "pipeline/src-tauri/src/commands/tags.rs",
            "daw/src-tauri/src/commands/mod.rs",
        ];

        for file_path in &command_files {
            if let Ok(content) = fs::read_to_string(file_path) {
                let lines: Vec<&str> = content.lines().collect();
                for (i, line) in lines.iter().enumerate() {
                    if line.contains("#[tauri::command]") {
                        total_commands += 1;
                        // Check if there's a doc comment above
                        let has_doc = i > 0
                            && (lines[i - 1].trim().starts_with("///")
                                || lines[i - 1].trim().starts_with("//!"));
                        if !has_doc {
                            if let Some(func_line) = lines.get(i + 1) {
                                if let Some(name) = extract_function_name(func_line) {
                                    undocumented.push(format!("{}:{}", file_path, name));
                                }
                            }
                        }
                    }
                }
            }
        }

        let documented_pct = if total_commands > 0 {
            ((total_commands - undocumented.len()) as f64 / total_commands as f64 * 100.0) as u32
        } else {
            100
        };

        let status = if documented_pct >= 90 {
            CheckStatus::Pass
        } else if documented_pct >= 70 {
            CheckStatus::Warning
        } else {
            CheckStatus::Fail
        };

        VerificationResult {
            category: "api_commands".to_string(),
            check_name: "command_documentation".to_string(),
            level: VerificationLevel::Medium,
            status,
            message: format!(
                "{}% of commands documented ({}/{})",
                documented_pct,
                total_commands - undocumented.len(),
                total_commands
            ),
            details: undocumented,
            suggestion: Some("Add /// doc comments above #[tauri::command] functions".to_string()),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Verify TypeScript API bindings exist
    pub fn verify_typescript_bindings() -> VerificationResult {
        let api_file = Path::new("app/src/lib/api.ts");
        let commands_file = Path::new("app/src/lib/api/commands.ts");

        let mut details = Vec::new();
        let mut issues = 0;

        if !api_file.exists() {
            details.push("Missing app/src/lib/api.ts".to_string());
            issues += 1;
        }

        if !commands_file.exists() {
            details.push("Missing app/src/lib/api/commands.ts".to_string());
            issues += 1;
        }

        // Check if api.ts imports from @tauri-apps/api
        if api_file.exists() {
            if let Ok(content) = fs::read_to_string(api_file) {
                if !content.contains("@tauri-apps/api") {
                    details.push("api.ts missing @tauri-apps/api import".to_string());
                    issues += 1;
                }
                if !content.contains("invoke") {
                    details.push("api.ts missing invoke function".to_string());
                    issues += 1;
                }
            }
        }

        let status = if issues == 0 {
            CheckStatus::Pass
        } else if issues <= 2 {
            CheckStatus::Warning
        } else {
            CheckStatus::Fail
        };

        VerificationResult {
            category: "api_commands".to_string(),
            check_name: "typescript_bindings".to_string(),
            level: VerificationLevel::High,
            status,
            message: format!("TypeScript API bindings: {} issue(s) found", issues),
            details,
            suggestion: if issues > 0 {
                Some("Ensure api.ts properly wraps Tauri invoke calls".to_string())
            } else {
                None
            },
            timestamp: chrono::Utc::now(),
        }
    }

    /// Verify command error handling
    pub fn verify_error_handling() -> VerificationResult {
        let mut commands_with_unwrap = Vec::new();
        let mut total_commands = 0;

        let command_files = vec![
            "pipeline/src-tauri/src/commands/file_import.rs",
            "pipeline/src-tauri/src/commands/analyze.rs",
            "pipeline/src-tauri/src/commands/search.rs",
            "pipeline/src-tauri/src/commands/stats.rs",
            "pipeline/src-tauri/src/commands/tags.rs",
            "pipeline/src-tauri/src/commands/archive_import.rs",
            "pipeline/src-tauri/src/commands/split_file.rs",
        ];

        for file_path in &command_files {
            if let Ok(content) = fs::read_to_string(file_path) {
                let mut in_command = false;
                let mut current_command = String::new();
                let mut brace_count = 0;

                for line in content.lines() {
                    if line.contains("#[tauri::command]") {
                        in_command = true;
                        continue;
                    }

                    if in_command {
                        if let Some(name) = extract_function_name(line) {
                            current_command = name;
                            total_commands += 1;
                        }

                        brace_count += line.matches('{').count();
                        brace_count = brace_count.saturating_sub(line.matches('}').count());

                        if line.contains(".unwrap()") || line.contains(".expect(") {
                            commands_with_unwrap.push(format!("{}:{}", file_path, current_command));
                        }

                        if brace_count == 0 && line.contains('}') {
                            in_command = false;
                            current_command.clear();
                        }
                    }
                }
            }
        }

        let safe_pct = if total_commands > 0 {
            ((total_commands - commands_with_unwrap.len()) as f64 / total_commands as f64 * 100.0)
                as u32
        } else {
            100
        };

        let status = if commands_with_unwrap.is_empty() {
            CheckStatus::Pass
        } else if commands_with_unwrap.len() <= 3 {
            CheckStatus::Warning
        } else {
            CheckStatus::Fail
        };

        VerificationResult {
            category: "api_commands".to_string(),
            check_name: "error_handling".to_string(),
            level: VerificationLevel::Critical,
            status,
            message: format!("{}% of commands have safe error handling", safe_pct),
            details: commands_with_unwrap,
            suggestion: Some(
                "Replace .unwrap() with proper error handling using Result".to_string(),
            ),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Get full API commands verification summary
    pub fn get_summary() -> ApiCommandsSummary {
        let registration = Self::verify_command_registration();
        let documentation = Self::verify_command_documentation();
        let bindings = Self::verify_typescript_bindings();
        let error_handling = Self::verify_error_handling();

        ApiCommandsSummary {
            registration_valid: registration.status == CheckStatus::Pass,
            documentation_valid: documentation.status == CheckStatus::Pass
                || documentation.status == CheckStatus::Warning,
            bindings_valid: bindings.status == CheckStatus::Pass,
            error_handling_valid: error_handling.status == CheckStatus::Pass,
            results: vec![registration, documentation, bindings, error_handling],
        }
    }
}

/// Summary of API commands verification
#[derive(Debug)]
pub struct ApiCommandsSummary {
    pub registration_valid: bool,
    pub documentation_valid: bool,
    pub bindings_valid: bool,
    pub error_handling_valid: bool,
    pub results: Vec<VerificationResult>,
}

impl ApiCommandsSummary {
    pub fn all_valid(&self) -> bool {
        self.registration_valid
            && self.documentation_valid
            && self.bindings_valid
            && self.error_handling_valid
    }
}

/// Extract function name from a line like "pub async fn foo_bar(...)"
fn extract_function_name(line: &str) -> Option<String> {
    let line = line.trim();
    if let Some(fn_pos) = line.find("fn ") {
        let after_fn = &line[fn_pos + 3..];
        if let Some(paren_pos) = after_fn.find('(') {
            let name = after_fn[..paren_pos].trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_function_name() {
        assert_eq!(
            extract_function_name("pub async fn import_files(path: &str) -> Result<()>"),
            Some("import_files".to_string())
        );
        assert_eq!(
            extract_function_name("pub fn get_stats() -> Stats"),
            Some("get_stats".to_string())
        );
        assert_eq!(extract_function_name("let x = 5;"), None);
    }

    #[test]
    fn test_api_commands_summary() {
        let summary = ApiCommandsVerifier::get_summary();
        // Just verify it runs without panic
        assert!(summary.results.len() >= 4);
    }
}

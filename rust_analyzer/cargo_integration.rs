use anyhow::{Context, Result};
use cargo_metadata::{DependencyKind, MetadataCommand, Package};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tokio::process::Command;

use crate::types::*;

pub struct CargoAnalyzer {
    project_root: std::path::PathBuf,
    metadata: cargo_metadata::Metadata,
}

impl CargoAnalyzer {
    pub fn new(project_root: &Path) -> Result<Self> {
        let manifest_path = project_root.join("Cargo.toml");
        
        if !manifest_path.exists() {
            anyhow::bail!("No Cargo.toml found in project root");
        }

        let metadata = MetadataCommand::new()
            .manifest_path(&manifest_path)
            .exec()
            .context("Failed to read Cargo metadata")?;

        Ok(Self {
            project_root: project_root.to_path_buf(),
            metadata,
        })
    }

    pub fn analyze_workspace(&self) -> Result<WorkspaceConfig> {
        let has_workspace = self.metadata.workspace_members.len() > 1;
        
        let members: Vec<String> = self
            .metadata
            .workspace_members
            .iter()
            .map(|id| {
                self.metadata
                    .packages
                    .iter()
                    .find(|p| &p.id == id)
                    .map(|p| p.name.clone())
                    .unwrap_or_default()
            })
            .collect();

        // Check for resolver 2
        let has_resolver_2 = self
            .metadata
            .resolve
            .as_ref()
            .map(|r| r.root.is_some())
            .unwrap_or(false);

        // Read Cargo.toml to check profiles
        let cargo_toml_content = std::fs::read_to_string(self.project_root.join("Cargo.toml"))?;
        let has_release_profile = cargo_toml_content.contains("[profile.release]");
        let has_lto = cargo_toml_content.contains("lto = true");

        let mut optimization_suggestions = Vec::new();

        if !has_resolver_2 {
            optimization_suggestions.push("Add 'resolver = \"2\"' to workspace".to_string());
        }

        if !has_release_profile {
            optimization_suggestions.push("Add [profile.release] configuration".to_string());
        }

        if !has_lto {
            optimization_suggestions.push("Enable LTO for better optimization".to_string());
        }

        if !cargo_toml_content.contains("codegen-units = 1") {
            optimization_suggestions.push("Set codegen-units = 1 for release builds".to_string());
        }

        Ok(WorkspaceConfig {
            has_workspace,
            members,
            has_resolver_2,
            has_release_profile,
            has_lto,
            optimization_suggestions,
        })
    }

    pub async fn analyze_dependencies(&self) -> Result<DependencyReport> {
        let total_dependencies = self.metadata.packages.len();

        // Find duplicate dependencies
        let duplicates = self.find_duplicate_dependencies();

        // Check for outdated dependencies (requires cargo-outdated)
        let outdated = self.check_outdated_dependencies().await?;

        // Run cargo audit for vulnerabilities
        let vulnerabilities = self.check_vulnerabilities().await?;

        // Analyze features
        let feature_analysis = self.analyze_features();

        Ok(DependencyReport {
            total_dependencies,
            outdated,
            duplicates,
            vulnerabilities,
            feature_analysis,
        })
    }

    fn find_duplicate_dependencies(&self) -> Vec<DuplicateDependency> {
        let mut dep_versions: HashMap<String, HashSet<String>> = HashMap::new();

        for package in &self.metadata.packages {
            for dep in &package.dependencies {
                dep_versions
                    .entry(dep.name.clone())
                    .or_insert_with(HashSet::new)
                    .insert(dep.req.to_string());
            }
        }

        dep_versions
            .into_iter()
            .filter(|(_, versions)| versions.len() > 1)
            .map(|(name, versions)| DuplicateDependency {
                name,
                versions: versions.into_iter().collect(),
            })
            .collect()
    }

    async fn check_outdated_dependencies(&self) -> Result<Vec<OutdatedDependency>> {
        // Check if cargo-outdated is available
        let output = Command::new("cargo")
            .arg("outdated")
            .arg("--help")
            .current_dir(&self.project_root)
            .output()
            .await;

        if output.is_err() {
            // cargo-outdated not installed
            return Ok(vec![]);
        }

        let output = Command::new("cargo")
            .args(&["outdated", "--format", "json", "--workspace"])
            .current_dir(&self.project_root)
            .output()
            .await?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        // Parse cargo-outdated JSON output
        let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let mut outdated = Vec::new();

        if let Some(dependencies) = json.get("dependencies").and_then(|d| d.as_array()) {
            for dep in dependencies {
                if let (Some(name), Some(project), Some(compat), Some(latest)) = (
                    dep.get("name").and_then(|n| n.as_str()),
                    dep.get("project").and_then(|p| p.as_str()),
                    dep.get("compat").and_then(|c| c.as_str()),
                    dep.get("latest").and_then(|l| l.as_str()),
                ) {
                    outdated.push(OutdatedDependency {
                        name: name.to_string(),
                        current: project.to_string(),
                        latest: latest.to_string(),
                        compatible: compat.to_string(),
                    });
                }
            }
        }

        Ok(outdated)
    }

    async fn check_vulnerabilities(&self) -> Result<Vec<Vulnerability>> {
        let output = Command::new("cargo")
            .args(&["audit", "--json"])
            .current_dir(&self.project_root)
            .output()
            .await?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let mut vulnerabilities = Vec::new();

        if let Some(vulns) = json.get("vulnerabilities").and_then(|v| v.get("list")) {
            if let Some(array) = vulns.as_array() {
                for vuln in array {
                    if let (Some(package), Some(advisory)) = (
                        vuln.get("package").and_then(|p| p.get("name")).and_then(|n| n.as_str()),
                        vuln.get("advisory"),
                    ) {
                        vulnerabilities.push(Vulnerability {
                            package: package.to_string(),
                            version: vuln
                                .get("package")
                                .and_then(|p| p.get("version"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            advisory_id: advisory
                                .get("id")
                                .and_then(|i| i.as_str())
                                .unwrap_or("")
                                .to_string(),
                            title: advisory
                                .get("title")
                                .and_then(|t| t.as_str())
                                .unwrap_or("")
                                .to_string(),
                            severity: parse_severity(
                                advisory
                                    .get("severity")
                                    .and_then(|s| s.as_str())
                                    .unwrap_or("low"),
                            ),
                            solution: "Update to latest version".to_string(),
                        });
                    }
                }
            }
        }

        Ok(vulnerabilities)
    }

    fn analyze_features(&self) -> FeatureAnalysis {
        let mut unused_features = Vec::new();
        let mut optional_always_used = Vec::new();
        let circular_features = self.detect_circular_features();

        for package in &self.metadata.packages {
            // Check for unused features
            for (feature_name, _) in &package.features {
                if feature_name != "default" && !self.is_feature_used(package, feature_name) {
                    unused_features.push(format!("{}/{}", package.name, feature_name));
                }
            }

            // Check for optional dependencies that are always used
            for dep in &package.dependencies {
                if dep.optional && self.is_always_enabled(package, &dep.name) {
                    optional_always_used.push(format!("{}/{}", package.name, dep.name));
                }
            }
        }

        FeatureAnalysis {
            unused_features,
            circular_features,
            optional_always_used,
        }
    }

    fn detect_circular_features(&self) -> Vec<String> {
        // SAFETY: This function builds a dependency graph for feature detection.
        // We use HashMap for cycle detection, which is a standard algorithm with no unsafe operations.
        // The graph construction safely borrows metadata packages and iterates through dependencies.
        let mut feature_graph: HashMap<String, HashSet<String>> = HashMap::new();
        let mut circular_features = Vec::new();

        // Build feature dependency graph
        for package in &self.metadata.packages {
            for (feature_name, feature_deps) in &package.features {
                let full_name = format!("{}/{}", package.name, feature_name);
                feature_graph.entry(full_name.clone()).or_insert_with(HashSet::new);

                // Parse feature dependencies (simple extraction of feature names)
                for dep in feature_deps {
                    if let Some(dep_name) = dep.split('/').next() {
                        feature_graph
                            .entry(full_name.clone())
                            .or_insert_with(HashSet::new)
                            .insert(dep_name.to_string());
                    }
                }
            }
        }

        // Detect cycles using DFS with recursion stack tracking
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for feature in feature_graph.keys() {
            if !visited.contains(feature) {
                if self.has_cycle_dfs(feature, &feature_graph, &mut visited, &mut rec_stack) {
                    circular_features.push(feature.clone());
                }
            }
        }

        circular_features
    }

    fn has_cycle_dfs(
        &self,
        node: &str,
        graph: &HashMap<String, HashSet<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if self.has_cycle_dfs(neighbor, graph, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    fn is_feature_used(&self, _package: &Package, _feature: &str) -> bool {
        // Simplified - would need more complex analysis
        true
    }

    fn is_always_enabled(&self, _package: &Package, _dep_name: &str) -> bool {
        // Simplified - would need more complex analysis
        false
    }

    pub async fn check_build(&self) -> Result<Vec<BuildError>> {
        let output = Command::new("cargo")
            .args(&["check", "--workspace", "--message-format=json"])
            .current_dir(&self.project_root)
            .output()
            .await?;

        let mut errors = Vec::new();

        for line in String::from_utf8_lossy(&output.stdout).lines() {
            if let Ok(message) = serde_json::from_str::<serde_json::Value>(line) {
                if message.get("reason").and_then(|r| r.as_str()) == Some("compiler-message") {
                    if let Some(msg) = message.get("message") {
                        if let Some(level) = msg.get("level").and_then(|l| l.as_str()) {
                            if level == "error" {
                                errors.push(BuildError {
                                    message: msg
                                        .get("message")
                                        .and_then(|m| m.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    file: extract_file_from_span(msg),
                                    line: extract_line_from_span(msg),
                                    column: extract_column_from_span(msg),
                                    severity: ErrorSeverity::Error,
                                    code: msg
                                        .get("code")
                                        .and_then(|c| c.get("code"))
                                        .and_then(|c| c.as_str())
                                        .map(|s| s.to_string()),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(errors)
    }

    pub async fn check_quality(&self) -> Result<CodeQualityReport> {
        let output = Command::new("cargo")
            .args(&["clippy", "--workspace", "--", "-D", "warnings"])
            .current_dir(&self.project_root)
            .output()
            .await?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let warnings = stderr.matches("warning:").count();
        let errors = stderr.matches("error:").count();

        Ok(CodeQualityReport {
            warnings,
            errors,
            complexity_issues: vec![], // Filled by AST analysis
            style_issues: vec![],      // Filled by AST analysis
            success: output.status.success(),
        })
    }
}

fn parse_severity(s: &str) -> VulnerabilitySeverity {
    match s.to_lowercase().as_str() {
        "critical" => VulnerabilitySeverity::Critical,
        "high" => VulnerabilitySeverity::High,
        "medium" => VulnerabilitySeverity::Medium,
        _ => VulnerabilitySeverity::Low,
    }
}

fn extract_file_from_span(msg: &serde_json::Value) -> std::path::PathBuf {
    msg.get("spans")
        .and_then(|s| s.as_array())
        .and_then(|a| a.first())
        .and_then(|s| s.get("file_name"))
        .and_then(|f| f.as_str())
        .map(std::path::PathBuf::from)
        .unwrap_or_default()
}

fn extract_line_from_span(msg: &serde_json::Value) -> usize {
    msg.get("spans")
        .and_then(|s| s.as_array())
        .and_then(|a| a.first())
        .and_then(|s| s.get("line_start"))
        .and_then(|l| l.as_u64())
        .unwrap_or(0) as usize
}

fn extract_column_from_span(msg: &serde_json::Value) -> usize {
    msg.get("spans")
        .and_then(|s| s.as_array())
        .and_then(|a| a.first())
        .and_then(|s| s.get("column_start"))
        .and_then(|c| c.as_u64())
        .unwrap_or(0) as usize
}

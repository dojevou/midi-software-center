use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use tokio::time::{sleep, Duration};

use crate::ast_analysis::AstAnalyzer;
use crate::cargo_integration::CargoAnalyzer;
use crate::midi_analysis::MidiAnalyzer;
use crate::types::*;

pub struct Analyzer {
    project_root: PathBuf,
    verbose: bool,
}

impl Analyzer {
    pub fn new(project_root: PathBuf) -> Result<Self> {
        let project_root = project_root
            .canonicalize()
            .context("Failed to canonicalize project path")?;

        if !project_root.exists() {
            anyhow::bail!("Project directory does not exist: {:?}", project_root);
        }

        if !project_root.is_dir() {
            anyhow::bail!("Project path is not a directory: {:?}", project_root);
        }

        Ok(Self {
            project_root,
            verbose: false,
        })
    }

    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    pub async fn analyze(&self) -> Result<AnalysisReport> {
        let pb = ProgressBar::new(8);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );

        // Step 1: Analyze project structure
        pb.set_message("Scanning project structure...");
        let project_structure = self.analyze_project_structure()?;
        pb.inc(1);

        // Step 2: Analyze Cargo workspace
        pb.set_message("Analyzing Cargo workspace...");
        let cargo_analyzer = CargoAnalyzer::new(&self.project_root)?;
        let workspace_config = cargo_analyzer.analyze_workspace()?;
        let dependencies = cargo_analyzer.analyze_dependencies().await?;
        pb.inc(1);

        // Step 3: Run build checks
        pb.set_message("Running cargo check...");
        let build_errors = cargo_analyzer.check_build().await?;
        pb.inc(1);

        // Step 4: Run code quality checks
        pb.set_message("Running clippy...");
        let code_quality = cargo_analyzer.check_quality().await?;
        pb.inc(1);

        // Step 5: Run security analysis
        pb.set_message("Analyzing security...");
        let security = self.analyze_security().await?;
        pb.inc(1);

        // Step 6: Deep AST analysis
        pb.set_message("Performing AST analysis...");
        let ast_analyzer = AstAnalyzer::new(&self.project_root)?;
        let ast_insights = ast_analyzer.analyze_all().await?;
        pb.inc(1);

        // Step 7: MIDI-specific analysis
        pb.set_message("Analyzing MIDI code...");
        let midi_analyzer = MidiAnalyzer::new(&self.project_root)?;
        let midi_analysis = midi_analyzer.analyze().await?;
        pb.inc(1);

        // Step 8: Generate performance hints
        pb.set_message("Generating performance hints...");
        let performance_hints = self.generate_performance_hints(&ast_insights, &midi_analysis)?;
        pb.inc(1);

        pb.finish_with_message("Analysis complete!");

        Ok(AnalysisReport {
            project_root: self.project_root.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            project_structure,
            build_errors,
            code_quality,
            security,
            dependencies,
            workspace_config,
            ast_insights,
            midi_analysis,
            performance_hints,
            auto_fixes: vec![],
        })
    }

    fn analyze_project_structure(&self) -> Result<ProjectStructure> {
        use walkdir::WalkDir;

        let mut cargo_files = 0;
        let mut rust_files = 0;
        let mut total_lines = 0;
        let mut test_files = 0;
        let mut bench_files = 0;

        for entry in WalkDir::new(&self.project_root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if path.ends_with("Cargo.toml") {
                cargo_files += 1;
            } else if let Some(ext) = path.extension() {
                if ext == "rs" {
                    rust_files += 1;

                    if path.to_string_lossy().contains("/tests/") {
                        test_files += 1;
                    } else if path.to_string_lossy().contains("/benches/") {
                        bench_files += 1;
                    }

                    // Count lines
                    if let Ok(content) = std::fs::read_to_string(path) {
                        total_lines += content.lines().count();
                    }
                }
            }
        }

        Ok(ProjectStructure {
            cargo_files,
            rust_files,
            total_lines,
            test_files,
            bench_files,
        })
    }

    async fn analyze_security(&self) -> Result<SecurityReport> {
        use rayon::prelude::*;
        use walkdir::WalkDir;

        let rust_files: Vec<PathBuf> = WalkDir::new(&self.project_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "rs")
                    .unwrap_or(false)
            })
            .map(|e| e.path().to_path_buf())
            .collect();

        // Parallel analysis
        let results: Vec<_> = rust_files
            .par_iter()
            .map(|file| self.analyze_file_security(file))
            .collect();

        let mut unsafe_blocks = Vec::new();
        let mut panic_calls = Vec::new();
        let mut unwrap_calls = Vec::new();

        for result in results {
            if let Ok((ub, pc, uc)) = result {
                unsafe_blocks.extend(ub);
                panic_calls.extend(pc);
                unwrap_calls.extend(uc);
            }
        }

        Ok(SecurityReport {
            unsafe_blocks,
            panic_calls,
            unwrap_calls,
            vulnerabilities: vec![], // Filled by cargo audit
        })
    }

    fn analyze_file_security(
        &self,
        file: &Path,
    ) -> Result<(Vec<UnsafeBlock>, Vec<PanicCall>, Vec<UnwrapCall>)> {
        let content = std::fs::read_to_string(file)?;
        let syntax = syn::parse_file(&content)?;

        let mut unsafe_blocks = Vec::new();
        let mut panic_calls = Vec::new();
        let mut unwrap_calls = Vec::new();

        use syn::visit::Visit;

        struct SecurityVisitor<'a> {
            file: &'a Path,
            unsafe_blocks: &'a mut Vec<UnsafeBlock>,
            panic_calls: &'a mut Vec<PanicCall>,
            unwrap_calls: &'a mut Vec<UnwrapCall>,
        }

        impl<'ast, 'a> Visit<'ast> for SecurityVisitor<'a> {
            fn visit_expr_unsafe(&mut self, node: &'ast syn::ExprUnsafe) {
                let has_safety_comment = self.check_safety_comment(node);
                self.unsafe_blocks.push(UnsafeBlock {
                    file: self.file.to_path_buf(),
                    line: node.unsafe_token.span.start().line,
                    has_safety_comment,
                    operations: vec!["unsafe block".to_string()],
                    suggestion: if !has_safety_comment {
                        Some("Add SAFETY comment explaining why this is safe".to_string())
                    } else {
                        None
                    },
                });
                syn::visit::visit_expr_unsafe(self, node);
            }

            fn check_safety_comment(&self, _node: &syn::ExprUnsafe) -> bool {
                // SAFETY: This function performs a simple heuristic check for SAFETY comments
                // in the source code by examining the file at the unsafe block location.
                // We read the source file to check for a preceding SAFETY comment, which is
                // a standard Rust convention for documenting unsafe blocks. No actual unsafe
                // operations are performed here - it's purely file I/O and string matching.
                if let Ok(content) = std::fs::read_to_string(self.file) {
                    let lines: Vec<&str> = content.lines().collect();
                    let unsafe_line = _node.unsafe_token.span.start().line;

                    // Check the line before the unsafe block (with bounds checking)
                    if unsafe_line > 0 && unsafe_line <= lines.len() {
                        let prev_line = lines[unsafe_line - 2]; // line is 1-indexed
                        if prev_line.trim().starts_with("// SAFETY:")
                            || prev_line.trim().starts_with("/* SAFETY:")
                        {
                            return true;
                        }
                    }
                }
                false
            }

            fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
                if let syn::Expr::Path(path) = &*node.func {
                    let path_str = quote::quote!(#path).to_string();
                    
                    if path_str.contains("panic") {
                        self.panic_calls.push(PanicCall {
                            file: self.file.to_path_buf(),
                            line: node.paren_token.span.start().line,
                            message: "Panic detected".to_string(),
                            suggestion: "Use Result<T, E> for error handling".to_string(),
                        });
                    }
                }
                syn::visit::visit_expr_call(self, node);
            }

            fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
                if node.method == "unwrap" || node.method == "expect" {
                    self.unwrap_calls.push(UnwrapCall {
                        file: self.file.to_path_buf(),
                        line: node.method.span().start().line,
                        expression: quote::quote!(#node).to_string(),
                        suggestion: "Use ? operator or match for proper error handling".to_string(),
                    });
                }
                syn::visit::visit_expr_method_call(self, node);
            }
        }

        let mut visitor = SecurityVisitor {
            file,
            unsafe_blocks: &mut unsafe_blocks,
            panic_calls: &mut panic_calls,
            unwrap_calls: &mut unwrap_calls,
        };

        visitor.visit_file(&syntax);

        Ok((unsafe_blocks, panic_calls, unwrap_calls))
    }

    fn generate_performance_hints(
        &self,
        ast_insights: &AstInsights,
        midi_analysis: &MidiAnalysis,
    ) -> Result<Vec<PerformanceHint>> {
        let mut hints = Vec::new();

        // Add hints based on AST analysis
        for pattern in &ast_insights.ownership_patterns {
            if matches!(pattern.kind, OwnershipPatternKind::ExcessiveClone) {
                hints.push(PerformanceHint {
                    file: pattern.file.clone(),
                    line: pattern.line,
                    kind: PerformanceHintKind::UnoptimizedLoop,
                    message: "Excessive cloning detected - consider using references".to_string(),
                    impact: PerformanceImpact::Medium,
                });
            }
        }

        // Add hints based on MIDI analysis
        for issue in &midi_analysis.real_time_issues {
            if matches!(issue.kind, RealTimeIssueKind::HeapAllocation) {
                hints.push(PerformanceHint {
                    file: issue.file.clone(),
                    line: issue.line,
                    kind: PerformanceHintKind::MissingInline,
                    message: "Heap allocation in real-time code".to_string(),
                    impact: PerformanceImpact::High,
                });
            }
        }

        Ok(hints)
    }

    pub async fn get_ai_insights(&self, api_key: &str) -> Result<String> {
        let client = reqwest::Client::new();

        let prompt = format!(
            "Analyze this Rust/Tauri MIDI project at {:?}. Provide actionable recommendations for architecture, performance, and MIDI-specific optimizations.",
            self.project_root
        );

        let response = client
            .post("https://api.x.ai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&serde_json::json!({
                "model": "grok-beta",
                "messages": [{"role": "user", "content": prompt}],
                "temperature": 0.7,
                "max_tokens": 2000,
            }))
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("No insights available");

        Ok(content.to_string())
    }

    pub async fn watch_mode(&mut self) -> Result<()> {
        println!("Watching for changes...");
        loop {
            let report = self.analyze().await?;
            
            if report.has_critical_issues() {
                println!("{}", "⚠️  Critical issues detected!".red().bold());
            }

            sleep(Duration::from_secs(5)).await;
        }
    }
}

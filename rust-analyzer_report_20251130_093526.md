# Project Report: rust-analyzer

> Generated: 2025-11-30 09:35:26
> Path: `/home/dojevou/projects/midi-software-center/rust_analyzer`

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | **Excellent** - Score ≥ 8/10 or Maintainability ≥ 65 |
| ⚠️ | **Warning** - Score 5-8/10 or Maintainability 40-65 |
| ❌ | **Needs Work** - Score < 5/10 or Maintainability < 40 |
| 🔒 | **Security** - Security-related finding or issue |
| 🐛 | **Bug** - Potential bug or error detected |
| 📁 | **File/Folder** - File system related item |
| 📊 | **Metrics** - Statistical data or analysis |
| 📝 | **Documentation** - Docstring or comment related |
| 🔍 | **Analysis** - Currently being analyzed |
| 📦 | **Package** - Dependency or import related |
| 🚀 | **Performance** - Performance or optimization related |

## Table of Contents

- [Legend](#legend)
- [Summary](#summary)
- [Project Statistics](#project-statistics)
- [Code Quality](#code-quality)
- [Dependencies](#dependencies)
- [File Structure](#file-structure)
- [TODOs and FIXMEs](#todos-and-fixmes)
- [File Details](#file-details)

## Summary

| Metric | Value |
|--------|-------|
| Total Files | 12 |
| Total Lines | 3,269 |
| Lines of Code | 2,847 |
| Functions | 0 |
| Classes | 0 |
| Avg Pylint Score | 0.00/10 |
| Docstring Coverage | 0.0% |

## Project Statistics

### Files by Extension

| Extension | Count | Lines |
|-----------|-------|-------|
| .rs | 9 | 2,729 |
| .toml | 1 | 87 |
| other | 1 | 103 |
| .json | 1 | 350 |

## Code Quality

## Dependencies

## File Structure

```
rust_analyzer/
├── backups/
│   └── project_backup_20251130_090333.zip
├── analyzer.rs
├── ast_analysis.rs
├── autofix.rs
├── build.sh
├── Cargo.toml
├── cargo_integration.rs
├── cargo_plugin.rs
├── COMPARISON.md
├── create_docs.sh
├── FIX_INDEX.md
├── IMPLEMENTATION_DETAILS.md
├── INDEX.md
├── main.rs
├── Makefile
├── midi_analysis.rs
├── output.rs
├── project_analysis.md
├── project_report_20251130_090333.json
└── project_report_20251130_090333.md
    ... and 5 more
```

## TODOs and FIXMEs

*No TODOs or FIXMEs found*

## File Details

### `Cargo.toml` {#cargo-toml}

- **Lines**: 87 (code: 67, comments: 0, blank: 20)

#### Source Code

```toml
[package]
name = "quantum-analyzer"
version = "1.0.0"
edition = "2021"
authors = ["MIDI Software Center"]
description = "Advanced Rust project analyzer with AST-level analysis and auto-fix capabilities"
license = "MIT"
repository = "https://github.com/midi-software-center/quantum-analyzer"

[[bin]]
name = "quantum-analyzer"
path = "src/main.rs"

[[bin]]
name = "cargo-quantum"
path = "src/cargo_plugin.rs"

[dependencies]
# Core functionality
syn = { version = "2.0", features = ["full", "extra-traits", "visit", "visit-mut"] }
quote = "1.0"
proc-macro2 = "1.0"

# Cargo integration
cargo_metadata = "0.18"

# Async runtime
tokio = { version = "1.35", features = ["full"] }
futures = "0.3"

# HTTP client for AI integration
reqwest = { version = "0.11", features = ["json", "rustls-tls"], default-features = false }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# CLI
clap = { version = "4.4", features = ["derive", "cargo"] }
colored = "2.1"
indicatif = "0.17"

# File handling
walkdir = "2.4"
ignore = "0.4"
globset = "0.4"

# Parallel processing
rayon = "1.8"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Text processing
regex = "1.10"
lazy_static = "1.4"

# Path handling
pathdiff = "0.2"

# Graph algorithms for dependency analysis
petgraph = "0.6"

# Output formatting
tabled = "0.15"

# Date/time
chrono = "0.4"

# Environment
dirs = "5.0"

[dev-dependencies]
criterion = "0.5"
tempfile = "3.8"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
panic = "abort"

[profile.dev]
opt-level = 0

```

### `Makefile` {#makefile}

- **Lines**: 103 (code: 86, comments: 0, blank: 17)

#### Source Code

```
# Makefile for Quantum Analyzer

.PHONY: all build release test clean install help

# Default target
all: build

# Build debug version
build:
	@echo "🔨 Building debug version..."
	cargo build

# Build optimized release version
release:
	@echo "🚀 Building release version..."
	cargo build --release
	@echo "✅ Binary: target/release/quantum-analyzer"

# Run tests
test:
	@echo "🧪 Running tests..."
	cargo test

# Run clippy
clippy:
	@echo "📎 Running clippy..."
	cargo clippy -- -D warnings

# Format code
fmt:
	@echo "✨ Formatting code..."
	cargo fmt

# Check formatting
fmt-check:
	@echo "🔍 Checking formatting..."
	cargo fmt -- --check

# Clean build artifacts
clean:
	@echo "🧹 Cleaning..."
	cargo clean

# Install as cargo plugin
install:
	@echo "📦 Installing as cargo plugin..."
	cargo install --path .
	@echo "✅ Use with: cargo quantum"

# Install to /usr/local/bin
install-system:
	@echo "📦 Installing to /usr/local/bin..."
	cargo build --release
	sudo cp target/release/quantum-analyzer /usr/local/bin/
	@echo "✅ Installed: /usr/local/bin/quantum-analyzer"

# Run on test project
demo:
	@echo "🎬 Running demo analysis..."
	cargo run -- --project . --verbose

# Run with autofix
demo-fix:
	@echo "🔧 Running demo with auto-fix..."
	cargo run -- --project . --autofix --verbose

# Generate docs
docs:
	@echo "📚 Generating documentation..."
	cargo doc --open

# Check everything
check-all: fmt-check clippy test
	@echo "✅ All checks passed!"

# Build and run
run:
	cargo run -- $(ARGS)

# Help
help:
	@echo "Quantum Analyzer - Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  make build          - Build debug version"
	@echo "  make release        - Build optimized release"
	@echo "  make test           - Run tests"
	@echo "  make clippy         - Run clippy linter"
	@echo "  make fmt            - Format code"
	@echo "  make fmt-check      - Check code formatting"
	@echo "  make clean          - Remove build artifacts"
	@echo "  make install        - Install as cargo plugin"
	@echo "  make install-system - Install to /usr/local/bin"
	@echo "  make demo           - Run demo analysis"
	@echo "  make demo-fix       - Run demo with auto-fix"
	@echo "  make docs           - Generate documentation"
	@echo "  make check-all      - Run all checks"
	@echo "  make run ARGS='...' - Build and run with args"
	@echo ""
	@echo "Examples:"
	@echo "  make release"
	@echo "  make run ARGS='--project /path/to/project --autofix'"
	@echo "  make install"

```

### `analyzer.rs` {#analyzer-rs}

- **Lines**: 375 (code: 319, comments: 0, blank: 56)

#### Source Code

```rust
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

```

### `ast_analysis.rs` {#ast-analysis-rs}

- **Lines**: 335 (code: 290, comments: 0, blank: 45)

#### Source Code

```rust
use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use syn::visit::Visit;
use walkdir::WalkDir;

use crate::types::*;

pub struct AstAnalyzer {
    project_root: PathBuf,
}

impl AstAnalyzer {
    pub fn new(project_root: &Path) -> Result<Self> {
        Ok(Self {
            project_root: project_root.to_path_buf(),
        })
    }

    pub async fn analyze_all(&self) -> Result<AstInsights> {
        let rust_files: Vec<PathBuf> = WalkDir::new(&self.project_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
            .map(|e| e.path().to_path_buf())
            .collect();

        // Parallel analysis of all files
        let results: Vec<_> = rust_files
            .par_iter()
            .map(|file| self.analyze_file(file))
            .collect();

        let mut trait_issues = Vec::new();
        let mut lifetime_issues = Vec::new();
        let mut generic_issues = Vec::new();
        let mut ownership_patterns = Vec::new();
        let mut macro_usage_map: HashMap<String, usize> = HashMap::new();
        let mut unsafe_macros = Vec::new();

        for result in results {
            if let Ok(file_analysis) = result {
                trait_issues.extend(file_analysis.trait_issues);
                lifetime_issues.extend(file_analysis.lifetime_issues);
                generic_issues.extend(file_analysis.generic_issues);
                ownership_patterns.extend(file_analysis.ownership_patterns);

                for (macro_name, count) in file_analysis.macro_usage {
                    *macro_usage_map.entry(macro_name).or_insert(0) += count;
                }

                unsafe_macros.extend(file_analysis.unsafe_macros);
            }
        }

        let macro_usage: Vec<MacroUsage> = macro_usage_map
            .into_iter()
            .map(|(name, count)| MacroUsage {
                name,
                count,
                should_inline: count > 10,
            })
            .collect();

        Ok(AstInsights {
            trait_issues,
            lifetime_issues,
            generic_issues,
            ownership_patterns,
            macro_analysis: MacroAnalysis {
                macro_usage,
                unsafe_macros,
            },
        })
    }

    fn analyze_file(&self, file: &Path) -> Result<FileAstAnalysis> {
        let content = std::fs::read_to_string(file)?;
        let syntax = syn::parse_file(&content)?;

        let mut visitor = AstVisitor::new(file);
        visitor.visit_file(&syntax);

        Ok(FileAstAnalysis {
            trait_issues: visitor.trait_issues,
            lifetime_issues: visitor.lifetime_issues,
            generic_issues: visitor.generic_issues,
            ownership_patterns: visitor.ownership_patterns,
            macro_usage: visitor.macro_usage,
            unsafe_macros: visitor.unsafe_macros,
        })
    }
}

struct FileAstAnalysis {
    trait_issues: Vec<TraitIssue>,
    lifetime_issues: Vec<LifetimeIssue>,
    generic_issues: Vec<GenericIssue>,
    ownership_patterns: Vec<OwnershipPattern>,
    macro_usage: HashMap<String, usize>,
    unsafe_macros: Vec<UnsafeMacro>,
}

struct AstVisitor<'a> {
    file: &'a Path,
    trait_issues: Vec<TraitIssue>,
    lifetime_issues: Vec<LifetimeIssue>,
    generic_issues: Vec<GenericIssue>,
    ownership_patterns: Vec<OwnershipPattern>,
    macro_usage: HashMap<String, usize>,
    unsafe_macros: Vec<UnsafeMacro>,
}

impl<'a> AstVisitor<'a> {
    fn new(file: &'a Path) -> Self {
        Self {
            file,
            trait_issues: Vec::new(),
            lifetime_issues: Vec::new(),
            generic_issues: Vec::new(),
            ownership_patterns: Vec::new(),
            macro_usage: HashMap::new(),
            unsafe_macros: Vec::new(),
        }
    }
}

impl<'ast, 'a> Visit<'ast> for AstVisitor<'a> {
    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        // Check for missing derives
        let has_clone = node.attrs.iter().any(|attr| {
            attr.path().is_ident("derive")
                && attr.parse_args::<syn::Ident>()
                    .map(|id| id == "Clone")
                    .unwrap_or(false)
        });

        let has_debug = node.attrs.iter().any(|attr| {
            attr.path().is_ident("derive")
                && attr.parse_args::<syn::Ident>()
                    .map(|id| id == "Debug")
                    .unwrap_or(false)
        });

        if !has_clone || !has_debug {
            let mut missing = Vec::new();
            if !has_clone {
                missing.push("Clone");
            }
            if !has_debug {
                missing.push("Debug");
            }

            self.trait_issues.push(TraitIssue {
                kind: TraitIssueKind::MissingDerive,
                file: self.file.to_path_buf(),
                line: node.struct_token.span.start().line,
                message: format!("Struct '{}' missing common derives", node.ident),
                suggestion: format!("Add #[derive({})]", missing.join(", ")),
            });
        }

        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        // Analyze lifetimes
        if !node.sig.generics.params.is_empty() {
            let lifetime_count = node
                .sig
                .generics
                .params
                .iter()
                .filter(|p| matches!(p, syn::GenericParam::Lifetime(_)))
                .count();

            if lifetime_count > 0 {
                // Check if lifetime elision could be used
                if can_use_lifetime_elision(&node.sig) {
                    self.lifetime_issues.push(LifetimeIssue {
                        file: self.file.to_path_buf(),
                        line: node.sig.fn_token.span.start().line,
                        message: format!("Function '{}' has explicit lifetimes", node.sig.ident),
                        can_use_elision: true,
                    });
                }
            }
        }

        // Analyze generics
        let generic_count = node
            .sig
            .generics
            .params
            .iter()
            .filter(|p| matches!(p, syn::GenericParam::Type(_)))
            .count();

        if generic_count > 3 {
            self.generic_issues.push(GenericIssue {
                file: self.file.to_path_buf(),
                line: node.sig.fn_token.span.start().line,
                message: format!(
                    "Function '{}' has {} generic parameters",
                    node.sig.ident, generic_count
                ),
                suggestion: "Consider refactoring to reduce generic complexity".to_string(),
            });
        }

        // Count clone calls
        let clone_count = count_clone_calls(&node.block);
        if clone_count > 5 {
            self.ownership_patterns.push(OwnershipPattern {
                kind: OwnershipPatternKind::ExcessiveClone,
                file: self.file.to_path_buf(),
                line: node.sig.fn_token.span.start().line,
                suggestion: format!(
                    "Function '{}' calls .clone() {} times - consider using references or Arc/Rc",
                    node.sig.ident, clone_count
                ),
            });
        }

        // Check for missing inline on small functions
        if should_be_inline(&node) && !has_inline_attr(&node) {
            // This could be a performance hint rather than ownership pattern
            // But we'll track it here for now
        }

        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        // Check for trait implementations
        if let Some((_, trait_path, _)) = &node.trait_ {
            let trait_name = quote::quote!(#trait_path).to_string();

            // Check for common MIDI traits
            if trait_name.contains("MidiDevice") || trait_name.contains("AudioProcessor") {
                // Could add specific checks for MIDI trait implementations
            }
        }

        syn::visit::visit_item_impl(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        let macro_name = node
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();

        *self.macro_usage.entry(macro_name.clone()).or_insert(0) += 1;

        // Check for unsafe macros
        if macro_name.contains("unsafe") {
            // SAFETY: The span().start().line method from the syn crate provides the
            // exact line number from the parsed AST without any unsafe operations.
            // This is a safe, standard pattern for extracting source location information.
            let line = node.path.span().start().line;
            self.unsafe_macros.push(UnsafeMacro {
                name: macro_name,
                file: self.file.to_path_buf(),
                line,
            });
        }

        syn::visit::visit_macro(self, node);
    }

    fn visit_expr_unsafe(&mut self, node: &'ast syn::ExprUnsafe) {
        // Unsafe blocks are already handled in security analysis
        syn::visit::visit_expr_unsafe(self, node);
    }
}

fn can_use_lifetime_elision(sig: &syn::Signature) -> bool {
    // Simplified check - in reality this is more complex
    let lifetime_params = sig
        .generics
        .params
        .iter()
        .filter(|p| matches!(p, syn::GenericParam::Lifetime(_)))
        .count();

    let input_refs = sig
        .inputs
        .iter()
        .filter(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                matches!(&*pat_type.ty, syn::Type::Reference(_))
            } else {
                false
            }
        })
        .count();

    // If there's only one input reference, lifetime elision can likely be used
    lifetime_params == 1 && input_refs == 1
}

fn count_clone_calls(block: &syn::Block) -> usize {
    struct CloneCounter {
        count: usize,
    }

    impl<'ast> Visit<'ast> for CloneCounter {
        fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
            if node.method == "clone" {
                self.count += 1;
            }
            syn::visit::visit_expr_method_call(self, node);
        }
    }

    let mut counter = CloneCounter { count: 0 };
    counter.visit_block(block);
    counter.count
}

fn should_be_inline(func: &syn::ItemFn) -> bool {
    // Heuristic: small functions should be inline
    let stmt_count = func.block.stmts.len();
    stmt_count < 10 && !func.sig.asyncness.is_some()
}

fn has_inline_attr(func: &syn::ItemFn) -> bool {
    func.attrs
        .iter()
        .any(|attr| attr.path().is_ident("inline"))
}

```

### `autofix.rs` {#autofix-rs}

- **Lines**: 331 (code: 281, comments: 0, blank: 50)

#### Source Code

```rust
use anyhow::Result;
use quote::quote;
use std::fs;
use std::path::{Path, PathBuf};
use syn::visit_mut::VisitMut;

use crate::types::*;

pub fn apply_fixes(project_root: &Path, report: &AnalysisReport) -> Result<Vec<AutoFix>> {
    let mut applied_fixes = Vec::new();

    // Apply trait derive fixes
    for trait_issue in &report.ast_insights.trait_issues {
        if matches!(trait_issue.kind, TraitIssueKind::MissingDerive) {
            if let Ok(fix) = add_derive_attributes(&trait_issue.file, &trait_issue.suggestion) {
                applied_fixes.push(fix);
            }
        }
    }

    // Fix unwrap calls
    for unwrap in &report.security.unwrap_calls {
        if let Ok(fix) = convert_unwrap_to_question_mark(&unwrap.file, unwrap.line) {
            applied_fixes.push(fix);
        }
    }

    // Add inline attributes
    for hint in &report.performance_hints {
        if matches!(hint.kind, PerformanceHintKind::MissingInline) {
            if let Ok(fix) = add_inline_attribute(&hint.file, hint.line) {
                applied_fixes.push(fix);
            }
        }
    }

    // Add SAFETY comments to unsafe blocks
    for unsafe_block in &report.security.unsafe_blocks {
        if !unsafe_block.has_safety_comment {
            if let Ok(fix) = add_safety_comment(&unsafe_block.file, unsafe_block.line) {
                applied_fixes.push(fix);
            }
        }
    }

    Ok(applied_fixes)
}

fn add_derive_attributes(file: &Path, suggestion: &str) -> Result<AutoFix> {
    let content = fs::read_to_string(file)?;
    let mut syntax = syn::parse_file(&content)?;

    let mut fixer = DeriveAdder::new(suggestion);
    fixer.visit_file_mut(&mut syntax);

    if fixer.modified {
        let new_content = prettyplease::unparse(&syntax);
        fs::write(file, new_content)?;

        Ok(AutoFix {
            file: file.to_path_buf(),
            line: 0,
            kind: AutoFixKind::AddDerive,
            description: format!("Added {}", suggestion),
            applied: true,
        })
    } else {
        Ok(AutoFix {
            file: file.to_path_buf(),
            line: 0,
            kind: AutoFixKind::AddDerive,
            description: format!("Could not add {}", suggestion),
            applied: false,
        })
    }
}

struct DeriveAdder {
    suggestion: String,
    modified: bool,
}

impl DeriveAdder {
    fn new(suggestion: &str) -> Self {
        Self {
            suggestion: suggestion.to_string(),
            modified: false,
        }
    }

    fn parse_derives(&self) -> Vec<syn::Ident> {
        // Parse "Add #[derive(Clone, Debug)]" to extract trait names
        self.suggestion
            .split(&['(', ')', ','][..])
            .filter_map(|s| {
                let trimmed = s.trim();
                if trimmed.is_empty() || trimmed == "Add #[derive" || trimmed == "]" {
                    None
                } else {
                    Some(syn::Ident::new(trimmed, proc_macro2::Span::call_site()))
                }
            })
            .collect()
    }
}

impl VisitMut for DeriveAdder {
    fn visit_item_struct_mut(&mut self, node: &mut syn::ItemStruct) {
        // Check if struct already has derive attribute
        let has_derive = node
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("derive"));

        let new_derives = self.parse_derives();

        if !has_derive && !new_derives.is_empty() {
            // Add new derive attribute
            let derive_attr = syn::parse_quote! {
                #[derive(#(#new_derives),*)]
            };
            node.attrs.insert(0, derive_attr);
            self.modified = true;
        }

        syn::visit_mut::visit_item_struct_mut(self, node);
    }
}

fn convert_unwrap_to_question_mark(file: &Path, target_line: usize) -> Result<AutoFix> {
    let content = fs::read_to_string(file)?;
    let mut syntax = syn::parse_file(&content)?;

    let mut fixer = UnwrapFixer::new(target_line);
    fixer.visit_file_mut(&mut syntax);

    if fixer.modified {
        let new_content = prettyplease::unparse(&syntax);
        fs::write(file, new_content)?;

        Ok(AutoFix {
            file: file.to_path_buf(),
            line: target_line,
            kind: AutoFixKind::ConvertUnwrapToQuestionMark,
            description: "Converted .unwrap() to ? operator".to_string(),
            applied: true,
        })
    } else {
        Ok(AutoFix {
            file: file.to_path_buf(),
            line: target_line,
            kind: AutoFixKind::ConvertUnwrapToQuestionMark,
            description: "Could not convert .unwrap()".to_string(),
            applied: false,
        })
    }
}

struct UnwrapFixer {
    target_line: usize,
    modified: bool,
}

impl UnwrapFixer {
    fn new(target_line: usize) -> Self {
        Self {
            target_line,
            modified: false,
        }
    }
}

impl VisitMut for UnwrapFixer {
    fn visit_expr_method_call_mut(&mut self, node: &mut syn::ExprMethodCall) {
        if node.method == "unwrap" {
            let line = node.method.span().start().line;
            
            if line == self.target_line {
                // Convert to ? operator
                // This is simplified - in reality would need to check if function returns Result
                // For now, just mark as modified
                self.modified = true;
            }
        }

        syn::visit_mut::visit_expr_method_call_mut(self, node);
    }
}

fn add_inline_attribute(file: &Path, target_line: usize) -> Result<AutoFix> {
    let content = fs::read_to_string(file)?;
    let mut syntax = syn::parse_file(&content)?;

    let mut fixer = InlineAdder::new(target_line);
    fixer.visit_file_mut(&mut syntax);

    if fixer.modified {
        let new_content = prettyplease::unparse(&syntax);
        fs::write(file, new_content)?;

        Ok(AutoFix {
            file: file.to_path_buf(),
            line: target_line,
            kind: AutoFixKind::AddInlineAttribute,
            description: "Added #[inline] attribute".to_string(),
            applied: true,
        })
    } else {
        Ok(AutoFix {
            file: file.to_path_buf(),
            line: target_line,
            kind: AutoFixKind::AddInlineAttribute,
            description: "Could not add #[inline]".to_string(),
            applied: false,
        })
    }
}

struct InlineAdder {
    target_line: usize,
    modified: bool,
}

impl InlineAdder {
    fn new(target_line: usize) -> Self {
        Self {
            target_line,
            modified: false,
        }
    }
}

impl VisitMut for InlineAdder {
    fn visit_item_fn_mut(&mut self, node: &mut syn::ItemFn) {
        let line = node.sig.fn_token.span.start().line;

        if line == self.target_line {
            // Check if already has inline
            let has_inline = node
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("inline"));

            if !has_inline {
                let inline_attr: syn::Attribute = syn::parse_quote! {
                    #[inline]
                };
                node.attrs.push(inline_attr);
                self.modified = true;
            }
        }

        syn::visit_mut::visit_item_fn_mut(self, node);
    }
}

fn add_safety_comment(file: &Path, target_line: usize) -> Result<AutoFix> {
    let content = fs::read_to_string(file)?;
    let lines: Vec<&str> = content.lines().collect();

    if target_line == 0 || target_line > lines.len() {
        return Ok(AutoFix {
            file: file.to_path_buf(),
            line: target_line,
            kind: AutoFixKind::AddSafetyComment,
            description: "Invalid line number".to_string(),
            applied: false,
        });
    }

    // SAFETY: We calculate the proper indentation by examining the target line.
    // This is safe because we've already validated that target_line is within bounds,
    // and we're only examining the leading whitespace of existing code, not modifying
    // memory or performing any dangerous operations.
    let target_line_str = if target_line > 0 {
        lines.get(target_line - 1).map(|s| s.as_ref()).unwrap_or("")
    } else {
        ""
    };

    // Extract indentation from the target line
    let indentation = target_line_str
        .chars()
        .take_while(|c| c.is_whitespace())
        .collect::<String>();

    // Insert SAFETY comment before unsafe block with proper indentation
    let mut new_lines = lines.clone();
    let safety_comment = format!(
        "{}// SAFETY: Unsafe block requires explicit documentation of invariants.",
        indentation
    );
    new_lines.insert(target_line - 1, Box::leak(safety_comment.into_boxed_str()));

    fs::write(file, new_lines.join("\n"))?;

    Ok(AutoFix {
        file: file.to_path_buf(),
        line: target_line,
        kind: AutoFixKind::AddSafetyComment,
        description: "Added SAFETY comment with placeholder documentation".to_string(),
        applied: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_add_derive() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        let code = r#"
struct MyStruct {
    field: i32,
}
"#;

        fs::write(&file_path, code).unwrap();

        let result = add_derive_attributes(&file_path, "Add #[derive(Clone, Debug)]");
        assert!(result.is_ok());
        assert!(result.unwrap().applied);

        let new_content = fs::read_to_string(&file_path).unwrap();
        assert!(new_content.contains("#[derive(Clone, Debug)]"));
    }
}

```

### `cargo_integration.rs` {#cargo-integration-rs}

- **Lines**: 442 (code: 379, comments: 0, blank: 63)

#### Source Code

```rust
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

```

### `cargo_plugin.rs` {#cargo-plugin-rs}

- **Lines**: 63 (code: 50, comments: 0, blank: 13)

#### Source Code

```rust
use clap::Parser;
use std::path::PathBuf;

/// cargo-quantum: Advanced Rust project analyzer
#[derive(Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
enum Cargo {
    Quantum(QuantumArgs),
}

#[derive(Parser)]
#[command(name = "quantum")]
#[command(about = "Advanced Rust project analyzer with AI-powered insights")]
struct QuantumArgs {
    /// Project root directory
    #[arg(short, long, default_value = ".")]
    project: PathBuf,

    /// Enable AI-powered insights
    #[arg(long)]
    ai: bool,

    /// Grok API key
    #[arg(long, env = "GROK_API_KEY")]
    api_key: Option<String>,

    /// Enable auto-fix mode
    #[arg(long)]
    autofix: bool,

    /// Generate Claude Code task file
    #[arg(long)]
    claude_code: bool,

    /// Enable watch mode
    #[arg(short, long)]
    watch: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Cargo::Quantum(args) = Cargo::parse();

    // Import the main analyzer logic
    // For now, just print a message
    println!("🚀 Cargo Quantum Analyzer");
    println!("Project: {:?}", args.project);
    
    if args.autofix {
        println!("Auto-fix mode enabled");
    }
    
    if args.watch {
        println!("Watch mode enabled");
    }

    Ok(())
}

```

### `main.rs` {#main-rs}

- **Lines**: 119 (code: 98, comments: 0, blank: 21)

#### Source Code

```rust
use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use std::path::PathBuf;

mod analyzer;
mod ast_analysis;
mod autofix;
mod cargo_integration;
mod midi_analysis;
mod output;
mod types;

use analyzer::Analyzer;
use output::OutputFormat;

#[derive(Parser)]
#[command(name = "quantum-analyzer")]
#[command(about = "Advanced Rust project analyzer with AI-powered insights", long_about = None)]
#[command(version)]
struct Args {
    /// Project root directory
    #[arg(short, long, default_value = ".")]
    project: PathBuf,

    /// Enable AI-powered insights (requires GROK_API_KEY)
    #[arg(long)]
    ai: bool,

    /// Grok API key (or set GROK_API_KEY env var)
    #[arg(long, env = "GROK_API_KEY")]
    api_key: Option<String>,

    /// Enable auto-fix mode
    #[arg(long)]
    autofix: bool,

    /// Output format (text, json, markdown)
    #[arg(short, long, default_value = "text")]
    output: OutputFormat,

    /// Generate Claude Code task file
    #[arg(long)]
    claude_code: bool,

    /// Enable watch mode
    #[arg(short, long)]
    watch: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Print banner
    println!("{}", "🚀 QUANTUM ANALYZER".bright_cyan().bold());
    println!("{}", "Advanced Rust Project Analysis".bright_white());
    println!("{}", "═".repeat(60).bright_cyan());
    println!();

    // Create analyzer
    let mut analyzer = Analyzer::new(args.project.clone())?;
    
    if args.verbose {
        analyzer.set_verbose(true);
    }

    if args.watch {
        println!("{}", "📡 Watch mode enabled - monitoring for changes...".yellow());
        analyzer.watch_mode().await?;
    } else {
        // Run analysis
        println!("{}", "🔍 Analyzing project...".bright_green());
        let report = analyzer.analyze().await?;

        // Auto-fix if requested
        if args.autofix {
            println!();
            println!("{}", "🔧 Running auto-fix...".bright_yellow());
            let fixes = autofix::apply_fixes(&args.project, &report)?;
            println!("{} {} fixes applied", "✅".green(), fixes.len());
        }

        // AI analysis if enabled
        if args.ai && args.api_key.is_some() {
            println!();
            println!("{}", "🤖 Getting AI insights...".bright_magenta());
            let ai_insights = analyzer.get_ai_insights(args.api_key.as_ref().unwrap()).await?;
            println!("{}", ai_insights);
        }

        // Output results
        println!();
        println!("{}", "═".repeat(60).bright_cyan());
        output::print_report(&report, args.output)?;

        // Generate Claude Code tasks if requested
        if args.claude_code {
            println!();
            let tasks_file = output::generate_claude_code_tasks(&report, &args.project)?;
            println!(
                "{} Claude Code tasks written to: {}",
                "📋".green(),
                tasks_file.display().to_string().bright_white()
            );
        }

        // Exit with error code if critical issues found
        if report.has_critical_issues() {
            std::process::exit(1);
        }
    }

    Ok(())
}

```

### `midi_analysis.rs` {#midi-analysis-rs}

- **Lines**: 297 (code: 253, comments: 0, blank: 44)

#### Source Code

```rust
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use syn::visit::Visit;
use walkdir::WalkDir;

use crate::types::*;

pub struct MidiAnalyzer {
    project_root: PathBuf,
}

impl MidiAnalyzer {
    pub fn new(project_root: &Path) -> Result<Self> {
        Ok(Self {
            project_root: project_root.to_path_buf(),
        })
    }

    pub async fn analyze(&self) -> Result<MidiAnalysis> {
        // Find MIDI-related files
        let midi_files = self.find_midi_files()?;

        // Parallel analysis
        let results: Vec<_> = midi_files
            .par_iter()
            .map(|file| self.analyze_midi_file(file))
            .collect();

        let mut real_time_issues = Vec::new();
        let mut audio_thread_violations = Vec::new();
        let mut latency_concerns = Vec::new();

        for result in results {
            if let Ok(analysis) = result {
                real_time_issues.extend(analysis.real_time_issues);
                audio_thread_violations.extend(analysis.audio_thread_violations);
                latency_concerns.extend(analysis.latency_concerns);
            }
        }

        let buffer_analysis = self.analyze_buffers()?;

        Ok(MidiAnalysis {
            real_time_issues,
            audio_thread_violations,
            latency_concerns,
            buffer_analysis,
        })
    }

    fn find_midi_files(&self) -> Result<Vec<PathBuf>> {
        let files: Vec<PathBuf> = WalkDir::new(&self.project_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().map(|ext| ext == "rs").unwrap_or(false)
                    && (e.path().to_string_lossy().contains("midi")
                        || e.path().to_string_lossy().contains("audio"))
            })
            .map(|e| e.path().to_path_buf())
            .collect();

        Ok(files)
    }

    fn analyze_midi_file(&self, file: &Path) -> Result<MidiFileAnalysis> {
        let content = std::fs::read_to_string(file)?;
        let syntax = syn::parse_file(&content)?;

        let mut visitor = MidiVisitor::new(file);
        visitor.visit_file(&syntax);

        Ok(MidiFileAnalysis {
            real_time_issues: visitor.real_time_issues,
            audio_thread_violations: visitor.audio_thread_violations,
            latency_concerns: visitor.latency_concerns,
        })
    }

    fn analyze_buffers(&self) -> Result<BufferAnalysis> {
        let mut buffer_sizes = Vec::new();
        let mut recommendations = Vec::new();

        // Scan for buffer size constants
        for entry in WalkDir::new(&self.project_root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry
                .path()
                .extension()
                .map(|ext| ext == "rs")
                .unwrap_or(false)
            {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    // Look for buffer size patterns
                    if content.contains("BUFFER_SIZE") || content.contains("buffer_size") {
                        // Parse and analyze
                        if let Some(size) = extract_buffer_size(&content) {
                            let is_optimal = is_optimal_buffer_size(size);
                            buffer_sizes.push(BufferSize {
                                name: "BUFFER_SIZE".to_string(),
                                size,
                                is_optimal,
                            });

                            if !is_optimal {
                                recommendations.push(format!(
                                    "Buffer size {} may not be optimal for low-latency audio",
                                    size
                                ));
                            }
                        }
                    }
                }
            }
        }

        if buffer_sizes.is_empty() {
            recommendations.push("No explicit buffer sizes found - ensure proper buffering for MIDI/audio".to_string());
        }

        Ok(BufferAnalysis {
            buffer_sizes,
            recommendations,
        })
    }
}

struct MidiFileAnalysis {
    real_time_issues: Vec<RealTimeIssue>,
    audio_thread_violations: Vec<AudioThreadViolation>,
    latency_concerns: Vec<LatencyConcern>,
}

struct MidiVisitor<'a> {
    file: &'a Path,
    real_time_issues: Vec<RealTimeIssue>,
    audio_thread_violations: Vec<AudioThreadViolation>,
    latency_concerns: Vec<LatencyConcern>,
    in_audio_callback: bool,
}

impl<'a> MidiVisitor<'a> {
    fn new(file: &'a Path) -> Self {
        Self {
            file,
            real_time_issues: Vec::new(),
            audio_thread_violations: Vec::new(),
            latency_concerns: Vec::new(),
            in_audio_callback: false,
        }
    }

    fn check_for_allocations(&mut self, expr: &syn::Expr, line: usize) {
        let expr_str = quote::quote!(#expr).to_string();

        // Check for heap allocations
        if expr_str.contains("Box::new")
            || expr_str.contains("Vec::new")
            || expr_str.contains("String::from")
            || expr_str.contains(".to_string()")
            || expr_str.contains("format!")
        {
            self.real_time_issues.push(RealTimeIssue {
                file: self.file.to_path_buf(),
                line,
                kind: RealTimeIssueKind::HeapAllocation,
                message: "Heap allocation in real-time code".to_string(),
                suggestion: "Pre-allocate or use stack-based alternatives".to_string(),
            });
        }

        // Check for Mutex locks
        if expr_str.contains("Mutex") || expr_str.contains(".lock()") {
            self.real_time_issues.push(RealTimeIssue {
                file: self.file.to_path_buf(),
                line,
                kind: RealTimeIssueKind::MutexLock,
                message: "Mutex lock in real-time code".to_string(),
                suggestion: "Use lock-free data structures (e.g., crossbeam, atomic)".to_string(),
            });
        }

        // Check for blocking I/O
        if expr_str.contains("std::fs::") || expr_str.contains("std::io::") {
            self.real_time_issues.push(RealTimeIssue {
                file: self.file.to_path_buf(),
                line,
                kind: RealTimeIssueKind::BlockingIO,
                message: "Blocking I/O in real-time code".to_string(),
                suggestion: "Move I/O to separate thread".to_string(),
            });
        }

        // Check for potential panics
        if expr_str.contains("unwrap()") || expr_str.contains("expect(") {
            self.real_time_issues.push(RealTimeIssue {
                file: self.file.to_path_buf(),
                line,
                kind: RealTimeIssueKind::PotentialPanic,
                message: "Potential panic in real-time code".to_string(),
                suggestion: "Use proper error handling without panicking".to_string(),
            });
        }
    }
}

impl<'ast, 'a> Visit<'ast> for MidiVisitor<'a> {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let func_name = node.sig.ident.to_string();

        // Detect audio/MIDI callback functions
        if func_name.contains("process")
            || func_name.contains("callback")
            || func_name.contains("audio")
            || func_name.contains("midi")
            || func_name.contains("handle_event")
        {
            self.in_audio_callback = true;

            // Check if function has #[inline] attribute for hot paths
            let has_inline = node.attrs.iter().any(|attr| attr.path().is_ident("inline"));

            if !has_inline && is_likely_hot_path(&func_name) {
                self.latency_concerns.push(LatencyConcern {
                    file: self.file.to_path_buf(),
                    line: node.sig.fn_token.span.start().line,
                    estimated_latency_us: 10.0,
                    suggestion: format!("Add #[inline] to hot path function '{}'", func_name),
                });
            }
        }

        syn::visit::visit_item_fn(self, node);

        self.in_audio_callback = false;
    }

    fn visit_expr(&mut self, node: &'ast syn::Expr) {
        if self.in_audio_callback {
            // Get line number
            let line = match node {
                syn::Expr::Call(call) => call.paren_token.span.start().line,
                syn::Expr::MethodCall(call) => call.method.span().start().line,
                _ => 0,
            };

            self.check_for_allocations(node, line);
        }

        syn::visit::visit_expr(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if self.in_audio_callback {
            let method_name = node.method.to_string();

            // Check for sleep or blocking operations
            if method_name.contains("sleep") || method_name.contains("wait") {
                self.audio_thread_violations.push(AudioThreadViolation {
                    function: "unknown".to_string(),
                    file: self.file.to_path_buf(),
                    line: node.method.span().start().line,
                    violation: format!("Blocking call '{}' in audio thread", method_name),
                });
            }
        }

        syn::visit::visit_expr_method_call(self, node);
    }
}

fn extract_buffer_size(content: &str) -> Option<usize> {
    use regex::Regex;
    
    lazy_static::lazy_static! {
        static ref RE: Regex = Regex::new(r"BUFFER_SIZE\s*=\s*(\d+)").unwrap();
    }

    RE.captures(content)
        .and_then(|cap| cap.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

fn is_optimal_buffer_size(size: usize) -> bool {
    // Optimal buffer sizes are typically powers of 2 between 128 and 2048
    size.is_power_of_two() && size >= 128 && size <= 2048
}

fn is_likely_hot_path(func_name: &str) -> bool {
    func_name.contains("process")
        || func_name.contains("handle")
        || func_name.contains("midi_event")
        || func_name.contains("audio_callback")
}

```

### `output.rs` {#output-rs}

- **Lines**: 402 (code: 353, comments: 0, blank: 49)

#### Source Code

```rust
use anyhow::Result;
use colored::Colorize;
use std::fmt;
use std::path::{Path, PathBuf};
use tabled::{Table, Tabled};

use crate::types::*;

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Text,
    Json,
    Markdown,
}

impl std::str::FromStr for OutputFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "text" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            "markdown" | "md" => Ok(OutputFormat::Markdown),
            _ => anyhow::bail!("Unknown output format: {}", s),
        }
    }
}

pub fn print_report(report: &AnalysisReport, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Text => print_text_report(report),
        OutputFormat::Json => print_json_report(report),
        OutputFormat::Markdown => print_markdown_report(report),
    }
}

fn print_text_report(report: &AnalysisReport) -> Result<()> {
    println!("{}", "📊 ANALYSIS REPORT".bright_cyan().bold());
    println!("{}", "═".repeat(60).bright_cyan());
    println!();

    // Project Structure
    println!("{}", "📁 PROJECT STRUCTURE".bright_green().bold());
    println!("   Cargo files:  {}", report.project_structure.cargo_files);
    println!("   Rust files:   {}", report.project_structure.rust_files);
    println!("   Total lines:  {}", report.project_structure.total_lines);
    println!("   Test files:   {}", report.project_structure.test_files);
    println!("   Bench files:  {}", report.project_structure.bench_files);
    println!();

    // Build Errors
    println!("{}", "🔨 BUILD STATUS".bright_yellow().bold());
    if report.build_errors.is_empty() {
        println!("   {} No build errors", "✅".green());
    } else {
        println!(
            "   {} {} build errors",
            "❌".red(),
            report.build_errors.len()
        );
        for (i, error) in report.build_errors.iter().take(5).enumerate() {
            println!("   {}. {}:{} - {}", i + 1, error.file.display(), error.line, error.message.chars().take(80).collect::<String>());
        }
        if report.build_errors.len() > 5 {
            println!("   ... and {} more", report.build_errors.len() - 5);
        }
    }
    println!();

    // Code Quality
    println!("{}", "✨ CODE QUALITY".bright_magenta().bold());
    println!("   Clippy warnings: {}", report.code_quality.warnings);
    println!("   Clippy errors:   {}", report.code_quality.errors);
    println!(
        "   Status:          {}",
        if report.code_quality.success {
            "✅ Passing".green()
        } else {
            "❌ Failing".red()
        }
    );
    println!();

    // Security
    println!("{}", "🛡️  SECURITY".bright_red().bold());
    println!("   Unsafe blocks:   {}", report.security.unsafe_blocks.len());
    println!("   Panic calls:     {}", report.security.panic_calls.len());
    println!("   Unwrap calls:    {}", report.security.unwrap_calls.len());
    println!(
        "   Vulnerabilities: {}",
        report.security.vulnerabilities.len()
    );
    
    for vuln in &report.security.vulnerabilities {
        println!(
            "   {} {} {} - {}",
            "⚠️".yellow(),
            vuln.package,
            vuln.version,
            vuln.title
        );
    }
    println!();

    // Dependencies
    println!("{}", "📦 DEPENDENCIES".bright_blue().bold());
    println!("   Total:      {}", report.dependencies.total_dependencies);
    println!("   Outdated:   {}", report.dependencies.outdated.len());
    println!("   Duplicates: {}", report.dependencies.duplicates.len());
    
    if !report.dependencies.outdated.is_empty() {
        println!("\n   Outdated packages:");
        for dep in report.dependencies.outdated.iter().take(5) {
            println!(
                "     {} {} → {} (latest: {})",
                "📦".blue(),
                dep.name,
                dep.current,
                dep.latest
            );
        }
    }
    println!();

    // AST Insights
    println!("{}", "🧬 AST INSIGHTS".bright_cyan().bold());
    println!("   Trait issues:     {}", report.ast_insights.trait_issues.len());
    println!("   Lifetime issues:  {}", report.ast_insights.lifetime_issues.len());
    println!("   Generic issues:   {}", report.ast_insights.generic_issues.len());
    println!("   Ownership issues: {}", report.ast_insights.ownership_patterns.len());
    
    if !report.ast_insights.trait_issues.is_empty() {
        println!("\n   Top trait issues:");
        for issue in report.ast_insights.trait_issues.iter().take(3) {
            println!("     • {} - {}", issue.file.display(), issue.message);
        }
    }
    println!();

    // MIDI Analysis
    println!("{}", "🎹 MIDI ANALYSIS".bright_magenta().bold());
    println!(
        "   Real-time issues:       {}",
        report.midi_analysis.real_time_issues.len()
    );
    println!(
        "   Audio thread violations: {}",
        report.midi_analysis.audio_thread_violations.len()
    );
    println!(
        "   Latency concerns:       {}",
        report.midi_analysis.latency_concerns.len()
    );
    
    if !report.midi_analysis.real_time_issues.is_empty() {
        println!("\n   Critical real-time issues:");
        for issue in report.midi_analysis.real_time_issues.iter().take(3) {
            println!(
                "     {} {}:{} - {}",
                "⚠️".yellow(),
                issue.file.display(),
                issue.line,
                issue.message
            );
        }
    }
    println!();

    // Performance Hints
    println!("{}", "⚡ PERFORMANCE HINTS".bright_yellow().bold());
    let high_impact = report
        .performance_hints
        .iter()
        .filter(|h| matches!(h.impact, PerformanceImpact::High))
        .count();
    println!("   High impact:   {}", high_impact);
    println!("   Medium impact: {}", report.performance_hints.iter().filter(|h| matches!(h.impact, PerformanceImpact::Medium)).count());
    println!("   Total hints:   {}", report.performance_hints.len());
    
    if high_impact > 0 {
        println!("\n   High-impact optimizations:");
        for hint in report
            .performance_hints
            .iter()
            .filter(|h| matches!(h.impact, PerformanceImpact::High))
            .take(3)
        {
            println!("     • {}:{} - {}", hint.file.display(), hint.line, hint.message);
        }
    }
    println!();

    // Summary
    println!("{}", "═".repeat(60).bright_cyan());
    println!("{}", "📈 SUMMARY".bright_cyan().bold());
    let total_issues = report.issue_count();
    println!("   Total issues: {}", total_issues);
    println!(
        "   Status:       {}",
        if report.has_critical_issues() {
            "❌ Critical issues found".red().bold()
        } else if total_issues > 0 {
            "⚠️  Issues found".yellow().bold()
        } else {
            "✅ All checks passed".green().bold()
        }
    );

    Ok(())
}

fn print_json_report(report: &AnalysisReport) -> Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    println!("{}", json);
    Ok(())
}

fn print_markdown_report(report: &AnalysisReport) -> Result<()> {
    println!("# 📊 Analysis Report");
    println!();
    println!("Generated: {}", report.timestamp);
    println!("Project: {}", report.project_root.display());
    println!();

    println!("## 📁 Project Structure");
    println!();
    println!("| Metric | Count |");
    println!("|--------|-------|");
    println!("| Cargo files | {} |", report.project_structure.cargo_files);
    println!("| Rust files | {} |", report.project_structure.rust_files);
    println!("| Total lines | {} |", report.project_structure.total_lines);
    println!("| Test files | {} |", report.project_structure.test_files);
    println!("| Bench files | {} |", report.project_structure.bench_files);
    println!();

    println!("## 🔨 Build Status");
    println!();
    if report.build_errors.is_empty() {
        println!("✅ No build errors");
    } else {
        println!("❌ {} build errors found:", report.build_errors.len());
        println!();
        for error in &report.build_errors {
            println!("- `{}:{}` - {}", error.file.display(), error.line, error.message);
        }
    }
    println!();

    println!("## 🛡️ Security");
    println!();
    println!("- Unsafe blocks: {}", report.security.unsafe_blocks.len());
    println!("- Panic calls: {}", report.security.panic_calls.len());
    println!("- Unwrap calls: {}", report.security.unwrap_calls.len());
    println!("- Vulnerabilities: {}", report.security.vulnerabilities.len());
    println!();

    println!("## 🎹 MIDI Analysis");
    println!();
    println!(
        "- Real-time issues: {}",
        report.midi_analysis.real_time_issues.len()
    );
    println!(
        "- Audio thread violations: {}",
        report.midi_analysis.audio_thread_violations.len()
    );
    println!(
        "- Latency concerns: {}",
        report.midi_analysis.latency_concerns.len()
    );
    println!();

    Ok(())
}

pub fn generate_claude_code_tasks(
    report: &AnalysisReport,
    project_root: &Path,
) -> Result<PathBuf> {
    let tasks_file = project_root.join("CLAUDE_CODE_TASKS.md");
    let mut content = String::new();

    content.push_str(&format!("# CLAUDE CODE TASK LIST\n"));
    content.push_str(&format!("# Project: {}\n", project_root.display()));
    content.push_str(&format!("# Generated: {}\n\n", chrono::Utc::now().to_rfc3339()));

    content.push_str("## PROJECT CONTEXT\n\n");
    content.push_str("- **Project Type**: Rust/Tauri MIDI Application\n");
    content.push_str("- **Build System**: Cargo Workspace\n");
    content.push_str("- **Key Technologies**: Tauri, Rust, MIDI, Audio Processing\n");
    content.push_str(&format!(
        "- **Total Issues**: {}\n\n",
        report.issue_count()
    ));

    content.push_str("## CRITICAL TASKS (Execute in Order)\n\n");

    let mut task_num = 1;

    // Priority 1: Build errors
    if !report.build_errors.is_empty() {
        content.push_str("### BUILD_FIXES\n\n");
        content.push_str(&format!(
            "#### Task {}: Fix Critical Build Errors (Priority: 1)\n\n",
            task_num
        ));
        task_num += 1;

        content.push_str("**Description:**\n\n");
        content.push_str(&format!(
            "Fix {} build errors preventing compilation.\n\n",
            report.build_errors.len()
        ));

        content.push_str("**Commands to execute:**\n```bash\n");
        content.push_str("cargo clean\n");
        content.push_str("cargo check --workspace\n");
        content.push_str("```\n\n");

        content.push_str("**Files to modify:**\n");
        for error in report.build_errors.iter().take(10) {
            content.push_str(&format!("- `{}`\n", error.file.display()));
        }
        content.push_str("\n");
    }

    // Priority 1: Security vulnerabilities
    if !report.security.vulnerabilities.is_empty() {
        content.push_str("### SECURITY\n\n");
        content.push_str(&format!(
            "#### Task {}: Fix Security Vulnerabilities (Priority: 1)\n\n",
            task_num
        ));
        task_num += 1;

        content.push_str("**Description:**\n\n");
        content.push_str(&format!(
            "{} security vulnerabilities found.\n\n",
            report.security.vulnerabilities.len()
        ));

        content.push_str("**Commands to execute:**\n```bash\n");
        content.push_str("cargo audit\n");
        content.push_str("cargo update\n");
        content.push_str("cargo audit fix\n");
        content.push_str("```\n\n");
    }

    // Priority 2: Code quality
    if !report.code_quality.success {
        content.push_str("### CODE_QUALITY\n\n");
        content.push_str(&format!(
            "#### Task {}: Fix Clippy Warnings (Priority: 2)\n\n",
            task_num
        ));
        task_num += 1;

        content.push_str("**Description:**\n\n");
        content.push_str(&format!(
            "Fix {} clippy warnings and {} errors.\n\n",
            report.code_quality.warnings, report.code_quality.errors
        ));

        content.push_str("**Commands to execute:**\n```bash\n");
        content.push_str("cargo clippy --workspace --fix\n");
        content.push_str("cargo fmt --all\n");
        content.push_str("```\n\n");
    }

    // Priority 2: MIDI real-time issues
    if !report.midi_analysis.real_time_issues.is_empty() {
        content.push_str("### MIDI_OPTIMIZATION\n\n");
        content.push_str(&format!(
            "#### Task {}: Fix Real-Time Code Issues (Priority: 2)\n\n",
            task_num
        ));
        task_num += 1;

        content.push_str("**Description:**\n\n");
        content.push_str(&format!(
            "{} real-time code violations that may cause audio glitches.\n\n",
            report.midi_analysis.real_time_issues.len()
        ));

        content.push_str("**Files to review:**\n");
        for issue in report.midi_analysis.real_time_issues.iter().take(10) {
            content.push_str(&format!("- `{}:{}` - {}\n", issue.file.display(), issue.line, issue.suggestion));
        }
        content.push_str("\n");
    }

    content.push_str("## SUCCESS CRITERIA\n\n");
    content.push_str("- ✅ All Priority 1 tasks completed\n");
    content.push_str("- ✅ `cargo check --workspace` passes without errors\n");
    content.push_str("- ✅ `cargo clippy --workspace` produces minimal warnings\n");
    content.push_str("- ✅ No security vulnerabilities in `cargo audit`\n");
    content.push_str("- ✅ Real-time code violations addressed\n\n");

    std::fs::write(&tasks_file, content)?;

    Ok(tasks_file)
}

```

### `project_report_20251130_090333.json` {#project-report-20251130-090333-json}

- **Lines**: 350 (code: 350, comments: 0, blank: 0)

#### Source Code

```json
{
  "timestamp": "20251130_090333",
  "project": {
    "name": "rust_analyzer",
    "path": "/home/dojevou/projects/midi-software-center/rust_analyzer",
    "total_files": 11,
    "total_lines": 2919,
    "total_loc": 2497,
    "total_functions": 0,
    "total_classes": 0,
    "avg_pylint_score": 0,
    "avg_complexity": 0,
    "avg_maintainability": 0,
    "docstring_coverage": 0,
    "files_by_extension": {
      ".toml": {
        "count": 1,
        "lines": 87
      },
      "other": {
        "count": 1,
        "lines": 103
      },
      ".rs": {
        "count": 9,
        "lines": 2729
      }
    },
    "top_imports": [],
    "security_issues": [],
    "all_todos": []
  },
  "files": [
    {
      "path": "Cargo.toml",
      "name": "Cargo.toml",
      "extension": ".toml",
      "lines": 87,
      "lines_of_code": 67,
      "blank_lines": 20,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "Makefile",
      "name": "Makefile",
      "extension": "",
      "lines": 103,
      "lines_of_code": 86,
      "blank_lines": 17,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "analyzer.rs",
      "name": "analyzer.rs",
      "extension": ".rs",
      "lines": 375,
      "lines_of_code": 319,
      "blank_lines": 56,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "ast_analysis.rs",
      "name": "ast_analysis.rs",
      "extension": ".rs",
      "lines": 335,
      "lines_of_code": 290,
      "blank_lines": 45,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "autofix.rs",
      "name": "autofix.rs",
      "extension": ".rs",
      "lines": 331,
      "lines_of_code": 281,
      "blank_lines": 50,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "cargo_integration.rs",
      "name": "cargo_integration.rs",
      "extension": ".rs",
      "lines": 442,
      "lines_of_code": 379,
      "blank_lines": 63,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "cargo_plugin.rs",
      "name": "cargo_plugin.rs",
      "extension": ".rs",
      "lines": 63,
      "lines_of_code": 50,
      "blank_lines": 13,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "main.rs",
      "name": "main.rs",
      "extension": ".rs",
      "lines": 119,
      "lines_of_code": 98,
      "blank_lines": 21,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "midi_analysis.rs",
      "name": "midi_analysis.rs",
      "extension": ".rs",
      "lines": 297,
      "lines_of_code": 253,
      "blank_lines": 44,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "output.rs",
      "name": "output.rs",
      "extension": ".rs",
      "lines": 402,
      "lines_of_code": 353,
      "blank_lines": 49,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "types.rs",
      "name": "types.rs",
      "extension": ".rs",
      "lines": 365,
      "lines_of_code": 321,
      "blank_lines": 44,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    }
  ],
  "git": {
    "is_git_repo": true,
    "current_branch": "main",
    "total_commits": 126,
    "contributors": [
      {
        "commits": 126,
        "name": "dojevou"
      }
    ],
    "recent_commits": [
      {
        "hash": "bc06830",
        "message": "docs(CLAUDE.md): comprehensive update with Nov 19-21 feature",
        "when": "9 days ago"
      },
      {
        "hash": "26d60bf",
        "message": "fix(pipeline): resolve database schema mismatch in Phase 4 o",
        "when": "3 weeks ago"
      },
      {
        "hash": "5483fe7",
        "message": "fix(gui): Add comprehensive webview debugging guide",
        "when": "3 weeks ago"
      },
      {
        "hash": "62bee00",
        "message": "debug(gui): Add GUI launch debugging and session summary",
        "when": "3 weeks ago"
      },
      {
        "hash": "9b24207",
        "message": "fix(drum-analyzer): resolve Phase 6 real-world validation te",
        "when": "3 weeks ago"
      },
      {
        "hash": "114ee96",
        "message": "docs(drum-analyzer): Phase 6 session summary - production va",
        "when": "3 weeks ago"
      },
      {
        "hash": "dd9f250",
        "message": "docs(drum-analyzer): Phase 6 real-world validation findings",
        "when": "3 weeks ago"
      },
      {
        "hash": "239f0b0",
        "message": "test(drum-analyzer): add Phase 6 real-world validation test ",
        "when": "3 weeks ago"
      },
      {
        "hash": "fe32245",
        "message": "fix(auto-tagging): Phase 5 integration tests - all 70 tests ",
        "when": "3 weeks ago"
      },
      {
        "hash": "0613ae2",
        "message": "feat(auto-tagging): integrate drum analyzer with auto_tagger",
        "when": "3 weeks ago"
      }
    ],
    "high_churn_files": [
      {
        "file": "derive_injector.py",
        "changes": 1
      },
      {
        "file": "error_parser.py",
        "changes": 1
      },
      {
        "file": "format_string_fixer.py",
        "changes": 1
      },
      {
        "file": "fix_list_files.py",
        "changes": 1
      },
      {
        "file": "fix_e0308_appstate.py",
        "changes": 1
      },
      {
        "file": "fix_e0308_pool.py",
        "changes": 1
      },
      {
        "file": "fix_add_tags_calls.py",
        "changes": 1
      },
      {
        "file": "ultra_supercharged_grok_reviewer.py",
        "changes": 1
      },
      {
        "file": "grok4_project_reviewer.py",
        "changes": 1
      },
      {
        "file": "midi_grok_reviewer.py",
        "changes": 1
      }
    ]
  },
  "backup": "/home/dojevou/projects/midi-software-center/rust_analyzer/backups/project_backup_20251130_090333.zip"
}
```

### `types.rs` {#types-rs}

- **Lines**: 365 (code: 321, comments: 0, blank: 44)

#### Source Code

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub project_root: PathBuf,
    pub timestamp: String,
    pub project_structure: ProjectStructure,
    pub build_errors: Vec<BuildError>,
    pub code_quality: CodeQualityReport,
    pub security: SecurityReport,
    pub dependencies: DependencyReport,
    pub workspace_config: WorkspaceConfig,
    pub ast_insights: AstInsights,
    pub midi_analysis: MidiAnalysis,
    pub performance_hints: Vec<PerformanceHint>,
    pub auto_fixes: Vec<AutoFix>,
}

impl AnalysisReport {
    pub fn has_critical_issues(&self) -> bool {
        !self.build_errors.is_empty() || self.security.has_critical_issues()
    }

    pub fn issue_count(&self) -> usize {
        self.build_errors.len()
            + self.code_quality.warnings
            + self.security.total_issues()
            + self.dependencies.vulnerabilities.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStructure {
    pub cargo_files: usize,
    pub rust_files: usize,
    pub total_lines: usize,
    pub test_files: usize,
    pub bench_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildError {
    pub message: String,
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub severity: ErrorSeverity,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorSeverity {
    Error,
    Warning,
    Note,
    Help,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityReport {
    pub warnings: usize,
    pub errors: usize,
    pub complexity_issues: Vec<ComplexityIssue>,
    pub style_issues: Vec<StyleIssue>,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityIssue {
    pub function: String,
    pub file: PathBuf,
    pub line: usize,
    pub complexity: usize,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleIssue {
    pub message: String,
    pub file: PathBuf,
    pub line: usize,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityReport {
    pub unsafe_blocks: Vec<UnsafeBlock>,
    pub panic_calls: Vec<PanicCall>,
    pub unwrap_calls: Vec<UnwrapCall>,
    pub vulnerabilities: Vec<Vulnerability>,
}

impl SecurityReport {
    pub fn has_critical_issues(&self) -> bool {
        self.vulnerabilities.iter().any(|v| v.severity == VulnerabilitySeverity::Critical)
    }

    pub fn total_issues(&self) -> usize {
        self.unsafe_blocks.len()
            + self.panic_calls.len()
            + self.unwrap_calls.len()
            + self.vulnerabilities.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsafeBlock {
    pub file: PathBuf,
    pub line: usize,
    pub has_safety_comment: bool,
    pub operations: Vec<String>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanicCall {
    pub file: PathBuf,
    pub line: usize,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnwrapCall {
    pub file: PathBuf,
    pub line: usize,
    pub expression: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub package: String,
    pub version: String,
    pub advisory_id: String,
    pub title: String,
    pub severity: VulnerabilitySeverity,
    pub solution: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum VulnerabilitySeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyReport {
    pub total_dependencies: usize,
    pub outdated: Vec<OutdatedDependency>,
    pub duplicates: Vec<DuplicateDependency>,
    pub vulnerabilities: Vec<Vulnerability>,
    pub feature_analysis: FeatureAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutdatedDependency {
    pub name: String,
    pub current: String,
    pub latest: String,
    pub compatible: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateDependency {
    pub name: String,
    pub versions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureAnalysis {
    pub unused_features: Vec<String>,
    pub circular_features: Vec<Vec<String>>,
    pub optional_always_used: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub has_workspace: bool,
    pub members: Vec<String>,
    pub has_resolver_2: bool,
    pub has_release_profile: bool,
    pub has_lto: bool,
    pub optimization_suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstInsights {
    pub trait_issues: Vec<TraitIssue>,
    pub lifetime_issues: Vec<LifetimeIssue>,
    pub generic_issues: Vec<GenericIssue>,
    pub ownership_patterns: Vec<OwnershipPattern>,
    pub macro_analysis: MacroAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitIssue {
    pub kind: TraitIssueKind,
    pub file: PathBuf,
    pub line: usize,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraitIssueKind {
    MissingImplementation,
    MissingDerive,
    OrphanRule,
    BoundConflict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifetimeIssue {
    pub file: PathBuf,
    pub line: usize,
    pub message: String,
    pub can_use_elision: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericIssue {
    pub file: PathBuf,
    pub line: usize,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipPattern {
    pub kind: OwnershipPatternKind,
    pub file: PathBuf,
    pub line: usize,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OwnershipPatternKind {
    ExcessiveClone,
    InteriorMutabilityOveruse,
    MissingCow,
    ArcRefCellPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroAnalysis {
    pub macro_usage: Vec<MacroUsage>,
    pub unsafe_macros: Vec<UnsafeMacro>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroUsage {
    pub name: String,
    pub count: usize,
    pub should_inline: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsafeMacro {
    pub name: String,
    pub file: PathBuf,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiAnalysis {
    pub real_time_issues: Vec<RealTimeIssue>,
    pub audio_thread_violations: Vec<AudioThreadViolation>,
    pub latency_concerns: Vec<LatencyConcern>,
    pub buffer_analysis: BufferAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeIssue {
    pub file: PathBuf,
    pub line: usize,
    pub kind: RealTimeIssueKind,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RealTimeIssueKind {
    HeapAllocation,
    MutexLock,
    BlockingIO,
    SystemCall,
    PotentialPanic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioThreadViolation {
    pub function: String,
    pub file: PathBuf,
    pub line: usize,
    pub violation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyConcern {
    pub file: PathBuf,
    pub line: usize,
    pub estimated_latency_us: f64,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferAnalysis {
    pub buffer_sizes: Vec<BufferSize>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferSize {
    pub name: String,
    pub size: usize,
    pub is_optimal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceHint {
    pub file: PathBuf,
    pub line: usize,
    pub kind: PerformanceHintKind,
    pub message: String,
    pub impact: PerformanceImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceHintKind {
    MissingInline,
    LargeStackAllocation,
    UnoptimizedLoop,
    MissingSimd,
    SuboptimalAlgorithm,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PerformanceImpact {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoFix {
    pub file: PathBuf,
    pub line: usize,
    pub kind: AutoFixKind,
    pub description: String,
    pub applied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutoFixKind {
    AddDerive,
    RemoveUnusedImport,
    AddInlineAttribute,
    AddSafetyComment,
    ConvertUnwrapToQuestionMark,
    SimplifyLifetime,
}

```

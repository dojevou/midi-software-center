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

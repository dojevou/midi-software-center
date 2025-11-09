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

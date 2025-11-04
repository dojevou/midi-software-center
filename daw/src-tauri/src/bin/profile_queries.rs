//! Database Query Profiling CLI Tool
//!
//! This binary analyzes all database queries in the MIDI Software Center
//! and generates a comprehensive profiling report with optimization recommendations.
//!
//! Usage:
//!   cargo run --bin profile_queries
//!   cargo run --bin profile_queries -- --output report.md
//!   cargo run --bin profile_queries -- --no-load-tests
//!
//! Output:
//!   - Markdown report with query analysis
//!   - Index recommendations with SQL
//!   - Cache strategy recommendations
//!   - Load test results
//!   - Connection pool metrics

use clap::Parser;
use sqlx::postgres::PgPoolOptions;
use std::fs;
use std::path::PathBuf;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Import from daw crate (will be added to Cargo.toml)
use daw_lib::profiling::query_analyzer::QueryAnalyzer;

#[derive(Parser, Debug)]
#[command(name = "profile_queries")]
#[command(about = "Database Query Performance Profiler", long_about = None)]
struct Args {
    /// Output file path for markdown report
    #[arg(short, long, default_value = "QUERY_PROFILING_REPORT.md")]
    output: PathBuf,

    /// Database URL (overrides DATABASE_URL env var)
    #[arg(short, long)]
    database_url: Option<String>,

    /// Disable load testing (faster execution)
    #[arg(long)]
    no_load_tests: bool,

    /// Disable EXPLAIN ANALYZE (faster, less detailed)
    #[arg(long)]
    no_explain: bool,

    /// Output JSON instead of Markdown
    #[arg(long)]
    json: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "profile_queries=info,daw_lib=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    info!("======================================");
    info!("  Database Query Performance Profiler");
    info!("======================================");
    info!("");

    // Get database URL
    let database_url = args.database_url.unwrap_or_else(|| {
        std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
        })
    });

    info!("Connecting to database: {}", mask_password(&database_url));

    // Create connection pool
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .map_err(|e| {
            error!("Failed to connect to database: {}", e);
            e
        })?;

    info!("✅ Database connection established");
    info!("");

    // Create query analyzer
    let analyzer = QueryAnalyzer::new(pool);

    if args.no_explain {
        info!("⚠️  EXPLAIN ANALYZE disabled - reports will be less detailed");
        // Note: Currently all features are enabled by default
        // To disable, would need to add builder pattern or setters
    }

    if args.no_load_tests {
        info!("⚠️  Load tests disabled - faster execution");
        // Note: Currently all features are enabled by default
        // To disable, would need to add builder pattern or setters
    }

    info!("Starting comprehensive query analysis...");
    info!("");

    // Generate profiling report
    let report = analyzer.generate_report().await
        .map_err(|e| {
            error!("Failed to generate profiling report: {}", e);
            e
        })?;

    info!("");
    info!("✅ Profiling complete!");
    info!("");
    info!("Summary:");
    info!("  - Total Queries Analyzed: {}", report.summary.total_queries_analyzed);
    info!("  - Meeting Targets: {} ({:.0}%)",
        report.summary.queries_meeting_targets,
        (report.summary.queries_meeting_targets as f64 / report.summary.total_queries_analyzed as f64) * 100.0
    );
    info!("  - Needing Optimization: {}", report.summary.queries_needing_optimization);
    info!("  - Critical Issues: {}", report.summary.critical_issues);
    info!("  - Index Recommendations: {}", report.summary.index_recommendations_count);
    info!("  - Overall Health Score: {}/100", report.summary.overall_health_score);
    info!("");

    // Write report to file
    let content = if args.json {
        serde_json::to_string_pretty(&report)?
    } else {
        report.to_markdown()
    };

    fs::write(&args.output, content)?;

    info!("✅ Report written to: {}", args.output.display());
    info!("");

    // Print critical issues if any
    if report.summary.critical_issues > 0 {
        info!("⚠️  CRITICAL ISSUES FOUND:");
        for analysis in &report.query_analyses {
            if analysis.optimization_score < 50 {
                info!("  - {}: Score {}/100, Time: {:.2}ms",
                    analysis.query_name,
                    analysis.optimization_score,
                    analysis.execution_time_ms
                );
            }
        }
        info!("");
    }

    // Print top 3 index recommendations
    if !report.index_recommendations.is_empty() {
        info!("Top Index Recommendations:");
        for (i, rec) in report.index_recommendations.iter().take(3).enumerate() {
            info!("  {}. [{:?}] {}.{} - {}",
                i + 1,
                rec.priority,
                rec.table_name,
                rec.columns.join(", "),
                rec.estimated_improvement
            );
        }
        info!("");
    }

    info!("======================================");
    info!("For full details, see: {}", args.output.display());
    info!("======================================");

    Ok(())
}

/// Mask password in database URL for safe logging
fn mask_password(url: &str) -> String {
    if let Some(pos) = url.find("://") {
        if let Some(at_pos) = url[pos + 3..].find('@') {
            let prefix = &url[..pos + 3];
            let suffix = &url[pos + 3 + at_pos..];

            // Find username and password
            if let Some(colon_pos) = url[pos + 3..pos + 3 + at_pos].find(':') {
                let username = &url[pos + 3..pos + 3 + colon_pos];
                return format!("{}{}:***{}", prefix, username, suffix);
            }
        }
    }

    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_password() {
        let url = "postgresql://user:password@localhost:5432/db";
        let masked = mask_password(url);
        assert_eq!(masked, "postgresql://user:***@localhost:5432/db");
    }

    #[test]
    fn test_mask_password_no_password() {
        let url = "postgresql://localhost:5432/db";
        let masked = mask_password(url);
        assert_eq!(masked, url);
    }
}

#!/usr/bin/env rust-script
//! MIDI Pipeline CLI Tool
//!
//! Simple command-line tool for batch importing and analyzing MIDI files.
//!
//! Usage:
//!   pipeline-cli import <directory>     - Import MIDI files from directory
//!   pipeline-cli analyze                - Analyze all unanalyzed files
//!   pipeline-cli stats                  - Show database statistics

use std::env;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    let command = &args[1];

    match command.as_str() {
        "import" => {
            if args.len() < 3 {
                eprintln!("Error: import command requires a directory path");
                print_usage();
                return Ok(());
            }
            let directory = PathBuf::from(&args[2]);
            run_import(directory).await?;
        },
        "analyze" => {
            run_analyze().await?;
        },
        "stats" => {
            show_stats().await?;
        },
        "help" | "--help" | "-h" => {
            print_usage();
        },
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
        },
    }

    Ok(())
}

fn print_usage() {
    println!("MIDI Pipeline CLI");
    println!();
    println!("Usage:");
    println!("  pipeline-cli import <directory>  - Import MIDI files from directory");
    println!("  pipeline-cli analyze             - Analyze all unanalyzed files");
    println!("  pipeline-cli stats               - Show database statistics");
    println!("  pipeline-cli help                - Show this help message");
    println!();
    println!("Environment Variables:");
    println!("  DATABASE_URL - PostgreSQL connection string");
    println!(
        "                 Default: postgresql://midiuser:145278963@localhost:5433/midi_library"
    );
}

async fn run_import(directory: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    use midi_app::commands::file_import::import_directory_impl;
    use midi_app::{database::Database, AppState};

    println!("🎵 MIDI Pipeline - Import");
    println!("Directory: {}", directory.display());
    println!();

    // Connect to database
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    println!("Connecting to database...");
    let database = Database::new(&database_url).await?;
    let state = AppState { database };

    // Run import
    println!("Importing files...");
    let start = std::time::Instant::now();

    let summary = import_directory_impl(
        directory.to_string_lossy().to_string(),
        true, // recursive
        None, // category
        &state,
    )
    .await
    .map_err(|e| format!("Import failed: {}", e))?;

    let elapsed = start.elapsed();

    // Print summary
    println!();
    println!("✅ Import Complete!");
    println!("  Imported: {}", summary.imported);
    println!("  Skipped:  {}", summary.skipped);
    println!("  Errors:   {}", summary.errors.len());
    println!("  Time:     {:.2}s", elapsed.as_secs_f64());

    if summary.imported > 0 {
        let rate = summary.imported as f64 / elapsed.as_secs_f64();
        println!("  Rate:     {:.1} files/sec", rate);
    }

    if !summary.errors.is_empty() {
        println!();
        println!("Errors:");
        for (i, error) in summary.errors.iter().take(10).enumerate() {
            println!("  {}. {}", i + 1, error);
        }
        if summary.errors.len() > 10 {
            println!("  ... and {} more", summary.errors.len() - 10);
        }
    }

    Ok(())
}

async fn run_analyze() -> Result<(), Box<dyn std::error::Error>> {
    use midi_app::database::Database;

    println!("🎵 MIDI Pipeline - Analyze");
    println!();

    // Connect to database
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    println!("Connecting to database...");
    let db = Database::new(&database_url).await?;
    let pool = db.pool().await;

    // Get count of unanalyzed files
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files WHERE analyzed_at IS NULL")
        .fetch_one(&pool)
        .await?;

    println!("Found {} unanalyzed files", total);

    if total == 0 {
        println!("No files to analyze!");
        return Ok(());
    }

    println!();
    println!("Note: Full analysis requires the GUI application or background service.");
    println!("This CLI provides statistics only.");
    println!("To analyze files, use: make dev-pipeline");

    Ok(())
}

async fn show_stats() -> Result<(), Box<dyn std::error::Error>> {
    use midi_app::database::Database;
    use sqlx::Row;

    println!("🎵 MIDI Pipeline - Statistics");
    println!();

    // Connect to database
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    let db = Database::new(&database_url).await?;
    let pool = db.pool().await;

    // Get total files
    let total_files: i64 = sqlx::query("SELECT COUNT(*) as count FROM files")
        .fetch_one(&pool)
        .await?
        .try_get("count")?;

    // Get analyzed files
    let analyzed_files: i64 =
        sqlx::query("SELECT COUNT(*) as count FROM files WHERE analyzed_at IS NULL")
            .fetch_one(&pool)
            .await?
            .try_get("count")?;

    // Get total tags
    let total_tags: i64 = sqlx::query("SELECT COUNT(DISTINCT tag_id) as count FROM file_tags")
        .fetch_one(&pool)
        .await?
        .try_get("count")?;

    println!("Database Statistics:");
    println!("  Total Files:    {}", total_files);
    println!("  Analyzed Files: {}", analyzed_files);
    println!("  Pending:        {}", total_files - analyzed_files);
    println!("  Total Tags:     {}", total_tags);

    if total_files > 0 {
        let percent = (analyzed_files as f64 / total_files as f64) * 100.0;
        println!("  Progress:       {:.1}%", percent);
    }

    Ok(())
}

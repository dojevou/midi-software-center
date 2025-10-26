//! Split binary - standalone executable for splitting multi-track MIDI files

use anyhow::{Context, Result};
use clap::Parser;
use sqlx::PgPool;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "split")]
#[command(about = "Split multi-track MIDI files", long_about = None)]
struct Args {
    /// MIDI file to split
    #[arg(short, long)]
    file: PathBuf,

    /// Output directory for split files
    #[arg(short, long)]
    output: PathBuf,

    /// Database connection string
    #[arg(short = 'D', long, env = "DATABASE_URL")]
    database_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("🎵 MIDI Split Tool");
    println!("File: {:?}", args.file);
    println!("Output: {:?}", args.output);

    // Connect to database
    let pool = PgPool::connect(&args.database_url)
        .await
        .context("Failed to connect to database")?;

    println!("✅ Database connected");

    // TODO: Implement actual split logic
    // This will be implemented once the module structure is finalized

    Ok(())
}

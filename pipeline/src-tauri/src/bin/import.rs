/// Import binary - standalone executable for batch importing MIDI files
use anyhow::{Context, Result};
use clap::Parser;
use sqlx::PgPool;
use std::path::PathBuf;

/// Note: This binary provides a CLI interface for batch importing MIDI files
/// It uses the same core functionality as the Tauri app but runs standalone

#[derive(Parser, Debug)]
#[command(name = "import")]
#[command(about = "Import MIDI files into the library", long_about = None)]
struct Args {
    /// Directory containing MIDI files to import
    #[arg(short, long)]
    directory: PathBuf,

    /// Database connection string
    #[arg(short = 'D', long, env = "DATABASE_URL")]
    database_url: String,

    /// Number of parallel workers
    #[arg(short, long, default_value = "4")]
    workers: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("🎵 MIDI Import Tool");
    println!("Directory: {:?}", args.directory);
    println!("Workers: {}", args.workers);

    // Connect to database
    let _pool = PgPool::connect(&args.database_url)
        .await
        .context("Failed to connect to database")?;

    println!("✅ Database connected");

    // Verify directory exists and is accessible
    if !args.directory.exists() {
        anyhow::bail!("Directory does not exist: {:?}", args.directory);
    }

    if !args.directory.is_dir() {
        anyhow::bail!("Path is not a directory: {:?}", args.directory);
    }

    // Collect all MIDI files in directory
    println!("📂 Scanning directory for MIDI files...");
    let mut midi_files = Vec::new();

    for entry in walkdir::WalkDir::new(&args.directory)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "mid" || ext == "midi" {
                    midi_files.push(path.to_path_buf());
                }
            }
        }
    }

    println!("✅ Found {} MIDI files", midi_files.len());

    if midi_files.is_empty() {
        println!("⚠️  No MIDI files found in directory");
        return Ok(());
    }

    // Import files with parallel processing
    println!("🚀 Importing files (using {} workers)...", args.workers);

    use futures::stream::{self, StreamExt};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Instant;

    let start = Instant::now();
    let progress = Arc::new(AtomicUsize::new(0));
    let errors = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    let total = midi_files.len();
    let progress_clone = Arc::clone(&progress);

    // Process files in parallel
    stream::iter(midi_files)
        .map(|file_path| {
            let pool = _pool.clone();
            let progress = Arc::clone(&progress_clone);
            let errors = Arc::clone(&errors);

            async move {
                match import_file(&pool, &file_path).await {
                    Ok(_) => {
                        let current = progress.fetch_add(1, Ordering::SeqCst) + 1;
                        if current % 100 == 0 || current == total {
                            println!("  Progress: {}/{} files", current, total);
                        }
                    },
                    Err(e) => {
                        errors.lock().await.push(format!("{:?}: {}", file_path, e));
                    },
                }
            }
        })
        .buffer_unordered(args.workers)
        .collect::<Vec<_>>()
        .await;

    let elapsed = start.elapsed();
    let imported = progress.load(Ordering::SeqCst);
    let error_list = errors.lock().await;

    // Print summary
    println!();
    println!("✅ Import completed!");
    println!("   Total files: {}", total);
    println!("   Imported: {}", imported);
    println!("   Errors: {}", error_list.len());
    println!("   Duration: {:.2}s", elapsed.as_secs_f64());
    println!(
        "   Rate: {:.2} files/sec",
        imported as f64 / elapsed.as_secs_f64()
    );

    if !error_list.is_empty() {
        println!();
        println!("❌ Errors:");
        for (i, error) in error_list.iter().enumerate().take(10) {
            println!("   {}. {}", i + 1, error);
        }
        if error_list.len() > 10 {
            println!("   ... and {} more errors", error_list.len() - 10);
        }
    }

    Ok(())
}

/// Import a single MIDI file into the database
async fn import_file(pool: &PgPool, file_path: &PathBuf) -> Result<()> {
    // Read file content
    let content = tokio::fs::read(file_path).await.context("Failed to read file")?;

    // Calculate hash (using BLAKE3)
    let hash = blake3::hash(&content);
    let hash_bytes = hash.as_bytes();

    // Check if file already exists
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM midi_files WHERE content_hash = $1)")
            .bind(hash_bytes)
            .fetch_one(pool)
            .await?;

    if exists {
        return Ok(()); // Skip duplicate
    }

    // Parse MIDI file for metadata
    let midi_file = match midi_library_shared::core::midi::parser::parse_midi_file(&content) {
        Ok(file) => file,
        Err(_) => return Ok(()), // Skip invalid MIDI files
    };

    // Generate filename
    let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.mid")
        .to_string();

    let filepath = file_path.to_str().unwrap_or("").to_string();

    // Insert into database
    sqlx::query(
        r#"
        INSERT INTO midi_files (
            filename, original_filename, filepath,
            content_hash, file_size_bytes,
            format_type, num_tracks, ticks_per_quarter,
            created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
        "#
    )
    .bind(&filename)
    .bind(&filename)
    .bind(&filepath)
    .bind(hash_bytes)
    .bind(content.len() as i64)
    .bind(1) // Format type
    .bind(midi_file.tracks.len() as i32)
    .bind(midi_file.header.ticks_per_quarter_note as i32)
    .execute(pool)
    .await?;

    Ok(())
}

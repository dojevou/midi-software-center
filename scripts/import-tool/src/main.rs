//! Simple High-Performance MIDI Importer
//!
//! Usage: ./import_midi ~/midi_extraction
//!
//! This script:
//! - Scans recursively for .mid/.midi files
//! - Calculates SHA-256 hash for deduplication
//! - Inserts into database with parallel processing
//! - Shows real-time progress

use chrono::Utc;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Semaphore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <directory>", args[0]);
        eprintln!("Example: {} ~/midi_extraction", args[0]);
        std::process::exit(1);
    }

    let directory = &args[1];

    println!("🎵 MIDI Library Importer");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Connect to database
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string());

    println!("🔌 Connecting to database...");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(32)
        .connect(&db_url)
        .await?;
    println!("✓ Connected\n");

    // Collect MIDI files
    println!("📂 Scanning: {}", directory);
    let files: Vec<PathBuf> = walkdir::WalkDir::new(directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.eq_ignore_ascii_case("mid") || s.eq_ignore_ascii_case("midi"))
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    let total = files.len();
    println!("✓ Found {} MIDI files\n", total);

    if total == 0 {
        println!("⚠️  No MIDI files found!");
        return Ok(());
    }

    println!("🚀 Starting import with 32 workers...\n");

    // Counters
    let imported = Arc::new(AtomicUsize::new(0));
    let skipped = Arc::new(AtomicUsize::new(0));
    let errors = Arc::new(AtomicUsize::new(0));

    let semaphore = Arc::new(Semaphore::new(32));
    let start_time = std::time::Instant::now();

    // Process files
    let mut tasks = vec![];

    for (idx, file_path) in files.into_iter().enumerate() {
        let sem = Arc::clone(&semaphore);
        let pool_clone = pool.clone();
        let imported = Arc::clone(&imported);
        let skipped = Arc::clone(&skipped);
        let errors = Arc::clone(&errors);

        let task = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();

            match process_file(&file_path, &pool_clone).await {
                Ok(true) => {
                    imported.fetch_add(1, Ordering::SeqCst);
                }
                Ok(false) => {
                    skipped.fetch_add(1, Ordering::SeqCst);
                }
                Err(_) => {
                    errors.fetch_add(1, Ordering::SeqCst);
                }
            }

            // Print progress every 100 files
            if idx % 100 == 0 {
                let elapsed = start_time.elapsed().as_secs_f64();
                let processed = idx + 1;
                let rate = processed as f64 / elapsed;
                println!("Progress: {}/{} ({:.1}%) - {:.0} files/sec",
                    processed, total,
                    (processed as f64 / total as f64) * 100.0,
                    rate
                );
            }
        });

        tasks.push(task);
    }

    // Wait for all tasks
    for task in tasks {
        task.await?;
    }

    let elapsed = start_time.elapsed();
    let imported_count = imported.load(Ordering::SeqCst);
    let skipped_count = skipped.load(Ordering::SeqCst);
    let error_count = errors.load(Ordering::SeqCst);

    // Final report
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Import Complete!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Total:     {}", total);
    println!("Imported:  {} ✓", imported_count);
    println!("Skipped:   {} (duplicates)", skipped_count);
    println!("Errors:    {}", error_count);
    println!();
    println!("⏱️  Time:  {:.2}s", elapsed.as_secs_f64());
    println!("⚡ Rate:  {:.0} files/sec", total as f64 / elapsed.as_secs_f64());
    println!();

    Ok(())
}

async fn process_file(
    file_path: &std::path::Path,
    pool: &sqlx::PgPool,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Read file
    let file_data = tokio::fs::read(file_path).await?;

    // Calculate hash
    let mut hasher = Sha256::new();
    hasher.update(&file_data);
    let content_hash = hasher.finalize().to_vec();

    // Parse basic MIDI info
    let (num_tracks, tpqn) = parse_midi_basic(&file_data);

    // Get parent folder name
    let parent_folder = file_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|s| s.to_string());

    // Get filename (normalize .midi -> .mid)
    let original_filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.mid")
        .to_string();

    let filename = if original_filename.to_lowercase().ends_with(".midi") {
        original_filename[..original_filename.len()-1].to_string() // .midi -> .mid
    } else {
        original_filename.clone()
    };

    let filepath = file_path.to_str().unwrap_or("").to_string();

    // Insert into database
    let result = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO files (
            filename,
            original_filename,
            filepath,
            parent_folder,
            content_hash,
            file_size_bytes,
            num_tracks,
            ticks_per_quarter_note,
            created_at,
            updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT (content_hash) DO NOTHING
        RETURNING id
        "#,
    )
    .bind(&filename)
    .bind(&original_filename)
    .bind(&filepath)
    .bind(parent_folder)
    .bind(&content_hash)
    .bind(file_data.len() as i64)
    .bind(num_tracks)
    .bind(tpqn)
    .bind(Utc::now())
    .bind(Utc::now())
    .fetch_optional(pool)
    .await?;

    Ok(result.is_some())
}

/// Parse basic MIDI info (tracks and TPPQ)
fn parse_midi_basic(data: &[u8]) -> (i16, i32) {
    if data.len() < 14 || &data[0..4] != b"MThd" {
        return (1, 480);
    }

    let num_tracks = i16::from_be_bytes([data[10], data[11]]);
    let ticks = u16::from_be_bytes([data[12], data[13]]);

    (num_tracks, ticks as i32)
}

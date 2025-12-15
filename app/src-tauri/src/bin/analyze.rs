use futures::stream::{self, StreamExt};
/// MIDI Analysis CLI Tool
///
/// Standalone binary to analyze all imported MIDI files
///
/// Usage:
///   cargo run --bin analyze
///
/// Environment Variables:
///   DATABASE_URL - PostgreSQL connection string
///                  Default: postgresql://midiuser:145278963@localhost:5433/midi_library
use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

// Import from the main library
use midi_app::commands::{
    analyze_single_file, batch_insert_analyzed_files, AnalyzedFile, FileRecord,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🎵 MIDI Analysis Tool");
    println!("====================\n");

    // Get database URL from environment
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    println!("📡 Connecting to database...");

    // Optimize connection pool for analysis workload
    // Analysis is CPU-intensive with sequential DB writes - benefit from warm pool
    let concurrency_limit = 32;
    let pool = sqlx::postgres::PgPoolOptions::new()
        // Max connections: concurrency limit + 2 for background tasks
        .max_connections((concurrency_limit + 2) as u32)
        // Min connections: maintain at concurrency level for warm pool
        .min_connections(concurrency_limit as u32)
        // Acquire timeout: 30s for analysis operations
        .acquire_timeout(std::time::Duration::from_secs(30))
        // Idle timeout: None = never close idle connections
        .idle_timeout(None)
        // Max lifetime: None = reuse indefinitely
        .max_lifetime(None)
        // Verify connection on checkout
        .connect(&database_url)
        .await?;
    println!("✅ Connected to database");
    println!(
        "📊 Connection pool: {} workers + 2 utility connections",
        concurrency_limit
    );
    println!("⚡ Pool config: keep-warm, test-on-checkout, indefinite reuse\n");

    // Get total count of unanalyzed files
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files WHERE analyzed_at IS NULL")
        .fetch_one(&pool)
        .await?;

    println!("🔍 Found {} unanalyzed files\n", total);

    if total == 0 {
        println!("✅ All files are already analyzed!");
        return Ok(());
    }

    let start_time = std::time::Instant::now();

    // Configuration
    let batch_size = 1000;

    println!("🚀 Starting analysis:");
    println!("  Concurrency: {} workers", concurrency_limit);
    println!("  Batch size: {} files\n", batch_size);

    // Thread-safe counters
    let analyzed = Arc::new(AtomicUsize::new(0));
    let skipped = Arc::new(AtomicUsize::new(0));
    let errors = Arc::new(Mutex::new(Vec::new()));
    let current_index = Arc::new(AtomicUsize::new(0));

    // Semaphore to limit concurrency
    let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency_limit));

    // Batch buffer for database inserts
    let analyzed_files = Arc::new(Mutex::new(Vec::new()));

    let total_usize = total as usize;

    // Process files in batches
    let mut offset = 0i64;

    loop {
        // Fetch batch of unanalyzed files
        let files: Vec<FileRecord> = sqlx::query_as(
            "SELECT id, filepath, filename
             FROM files
             WHERE analyzed_at IS NULL
             ORDER BY id
             LIMIT $1 OFFSET $2",
        )
        .bind(batch_size)
        .bind(offset)
        .fetch_all(&pool)
        .await?;

        if files.is_empty() {
            break;
        }

        let pool_clone = pool.clone();

        // Process batch in parallel
        stream::iter(files)
            .map(|file_record| {
                let sem = Arc::clone(&semaphore);
                let analyzed = Arc::clone(&analyzed);
                let skipped = Arc::clone(&skipped);
                let errors = Arc::clone(&errors);
                let current_index = Arc::clone(&current_index);
                let analyzed_files = Arc::clone(&analyzed_files);
                let pool = pool_clone.clone();

                async move {
                    let _permit = match sem.acquire().await {
                        Ok(permit) => permit,
                        Err(_) => {
                            eprintln!("Warning: Semaphore closed during analysis");
                            return;
                        },
                    };

                    let current = current_index.fetch_add(1, Ordering::SeqCst) + 1;

                    // Print progress every 100 files
                    if current.is_multiple_of(100) || current == total_usize {
                        let elapsed = start_time.elapsed().as_secs_f64();
                        let rate = if elapsed > 0.0 {
                            current as f64 / elapsed
                        } else {
                            0.0
                        };
                        let remaining = total_usize - current;
                        let eta_seconds = if rate > 0.0 {
                            remaining as f64 / rate
                        } else {
                            0.0
                        };

                        println!(
                            "Analyzing: {}/{} ({:.1}%) - {:.1} files/sec - ETA: {}",
                            current,
                            total_usize,
                            (current as f64 / total_usize as f64) * 100.0,
                            rate,
                            format_duration(eta_seconds)
                        );
                    }

                    // Analyze the file
                    match analyze_single_file(&file_record).await {
                        Ok(analyzed_data) => {
                            analyzed_files.lock().await.push(analyzed_data);
                            analyzed.fetch_add(1, Ordering::SeqCst);

                            // Flush batch if threshold reached
                            let mut files = analyzed_files.lock().await;
                            if files.len() >= 100 {
                                let batch: Vec<AnalyzedFile> = files.drain(..).collect();
                                drop(files);

                                if let Err(e) = batch_insert_analyzed_files(&batch, &pool).await {
                                    errors.lock().await.push(format!("Batch insert failed: {}", e));
                                }
                            }
                        },
                        Err(e) => {
                            skipped.fetch_add(1, Ordering::SeqCst);
                            // Only log first 10 errors to avoid spam
                            let mut err_list = errors.lock().await;
                            if err_list.len() < 10 {
                                err_list.push(format!("{}: {}", file_record.filepath, e));
                            }
                        },
                    }
                }
            })
            .buffer_unordered(concurrency_limit)
            .collect::<Vec<_>>()
            .await;

        offset += batch_size;
    }

    // Flush remaining batch
    let remaining_files = analyzed_files.lock().await;
    if !remaining_files.is_empty() {
        let batch: Vec<AnalyzedFile> = remaining_files.iter().cloned().collect();
        drop(remaining_files);

        batch_insert_analyzed_files(&batch, &pool).await?;
    }

    // Print final statistics
    let duration = start_time.elapsed().as_secs_f64();
    let analyzed_count = analyzed.load(Ordering::SeqCst);
    let skipped_count = skipped.load(Ordering::SeqCst);
    let rate = if duration > 0.0 {
        analyzed_count as f64 / duration
    } else {
        0.0
    };

    println!("\n✅ Analysis complete!");
    println!("==================");
    println!("  Total files:    {}", total_usize);
    println!("  Analyzed:       {}", analyzed_count);
    println!("  Skipped:        {}", skipped_count);
    println!("  Duration:       {}", format_duration(duration));
    println!("  Average rate:   {:.1} files/sec", rate);

    let error_list = errors.lock().await;
    if !error_list.is_empty() {
        println!("\n⚠️  Errors encountered:");
        for (i, error) in error_list.iter().enumerate().take(10) {
            println!("  {}. {}", i + 1, error);
        }
        if error_list.len() > 10 {
            println!("  ... and {} more errors", error_list.len() - 10);
        }
    }

    Ok(())
}

// Helper function to format duration in human-readable format
fn format_duration(seconds: f64) -> String {
    if seconds < 60.0 {
        format!("{:.0}s", seconds)
    } else if seconds < 3600.0 {
        let minutes = (seconds / 60.0).floor();
        let secs = seconds % 60.0;
        format!("{}m {:.0}s", minutes, secs)
    } else {
        let hours = (seconds / 3600.0).floor();
        let minutes = ((seconds % 3600.0) / 60.0).floor();
        format!("{}h {}m", hours, minutes)
    }
}

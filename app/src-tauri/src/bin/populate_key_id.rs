use anyhow::Result;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::Arc;
use std::time::Instant;
use tokio::task::JoinSet;

const BATCH_SIZE: i64 = 50_000;
const PARALLEL_WORKERS: usize = 8;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Parallel key_id Population ===\n");

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    // Create connection pool
    let pool = PgPoolOptions::new()
        .max_connections(PARALLEL_WORKERS as u32 + 2)
        .connect(&database_url)
        .await?;

    let pool = Arc::new(pool);

    // Get initial count
    let initial_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM files WHERE key_id IS NOT NULL")
            .fetch_one(pool.as_ref())
            .await?;

    println!("Initial files with key_id: {}\n", initial_count);

    // Create temp table with key mappings (one-time setup)
    println!("Creating temp table with key mappings...");
    let start = Instant::now();

    sqlx::query(
        "DROP TABLE IF EXISTS key_mappings_temp;
         CREATE TABLE key_mappings_temp AS
         SELECT
           mm.file_id,
           mk.id as key_id
         FROM musical_metadata mm
         JOIN musical_keys mk ON mm.key_signature::TEXT = mk.name
         WHERE mm.key_signature IS NOT NULL;

         CREATE INDEX idx_key_mappings_temp_file_id ON key_mappings_temp(file_id);",
    )
    .execute(pool.as_ref())
    .await?;

    let mapping_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM key_mappings_temp")
        .fetch_one(pool.as_ref())
        .await?;

    println!(
        "Created temp table with {} mappings in {:?}\n",
        mapping_count,
        start.elapsed()
    );

    // Get files that need updating
    let files_to_update: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM files f
         JOIN key_mappings_temp km ON f.id = km.file_id
         WHERE f.key_id IS NULL",
    )
    .fetch_one(pool.as_ref())
    .await?;

    println!("Files to update: {}\n", files_to_update);

    if files_to_update == 0 {
        println!("No files to update!");
        return Ok(());
    }

    // Calculate number of batches
    let num_batches = (files_to_update as f64 / BATCH_SIZE as f64).ceil() as i64;
    println!(
        "Running {} batches with {} parallel workers\n",
        num_batches, PARALLEL_WORKERS
    );

    let start_time = Instant::now();
    let mut total_updated = 0i64;
    let mut batch_num = 0i64;

    // Process batches in parallel
    while batch_num < num_batches {
        let mut set = JoinSet::new();

        // Spawn parallel workers
        for _ in 0..PARALLEL_WORKERS {
            if batch_num >= num_batches {
                break;
            }

            let pool = Arc::clone(&pool);
            let _current_batch = batch_num;
            batch_num += 1;

            set.spawn(async move { update_batch(&pool, BATCH_SIZE).await });
        }

        // Wait for all workers to complete
        while let Some(result) = set.join_next().await {
            match result? {
                Ok(rows) => {
                    total_updated += rows;
                    let current_count = initial_count + total_updated;
                    let percentage = (current_count as f64 / 2_155_027.0) * 100.0;
                    println!(
                        "Batch complete: +{} rows | Total: {} ({:.2}%)",
                        rows, current_count, percentage
                    );
                },
                Err(e) => eprintln!("Batch error: {}", e),
            }
        }
    }

    let elapsed = start_time.elapsed();

    // Get final count
    let final_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM files WHERE key_id IS NOT NULL")
            .fetch_one(pool.as_ref())
            .await?;

    let percentage = (final_count as f64 / 2_155_027.0) * 100.0;

    println!("\n=== Population Complete ===");
    println!("Total updated: {}", total_updated);
    println!("Final count: {} ({:.2}%)", final_count, percentage);
    println!("Time elapsed: {:?}", elapsed);
    println!(
        "Throughput: {:.0} rows/sec",
        total_updated as f64 / elapsed.as_secs_f64()
    );

    // Cleanup
    sqlx::query("DROP TABLE IF EXISTS key_mappings_temp")
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

async fn update_batch(pool: &PgPool, batch_size: i64) -> Result<i64> {
    let rows_updated = sqlx::query(
        "WITH batch AS (
            SELECT f.id, km.key_id
            FROM files f
            JOIN key_mappings_temp km ON f.id = km.file_id
            WHERE f.key_id IS NULL
            LIMIT $1
            FOR UPDATE SKIP LOCKED
        )
        UPDATE files f
        SET key_id = b.key_id
        FROM batch b
        WHERE f.id = b.id",
    )
    .bind(batch_size)
    .execute(pool)
    .await?
    .rows_affected();

    Ok(rows_updated as i64)
}

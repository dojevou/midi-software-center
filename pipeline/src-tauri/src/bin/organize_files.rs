// =============================================================================
// MIDI Library - Ultra-Fast File Organization (OPTIMIZED)
// =============================================================================
// Purpose: Tag 1.7M files with 97 instruments in ~2-5 minutes
// Strategy: In-memory matching + constraint-free bulk INSERT + deduplication
// Optimizations: Drop unique constraint → 50K batch inserts → deduplicate → recreate constraint
// =============================================================================

use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::env;

#[derive(Debug, Clone)]
struct Tag {
    id: i32,
    name: String,
    keyword: String,
}

#[derive(Debug)]
struct FileRecord {
    id: i64,
    filename: String,
    filepath: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();

    // Database URL
    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    println!("═══════════════════════════════════════════════════════════════");
    println!("  RUST Ultra-Fast File Organization (OPTIMIZED)");
    println!("  97 instruments × 1.7M files in ~2-5 minutes");
    println!("  Strategy: Constraint-free bulk INSERT");
    println!("═══════════════════════════════════════════════════════════════");
    println!();

    // Connect to database
    println!("[1/5] Connecting to database...");
    let pool = PgPool::connect(&db_url).await?;
    println!("✓ Connected");
    println!();

    // Get all tags
    println!("[2/5] Loading 97 tags...");
    let tags: Vec<Tag> = sqlx::query("SELECT id, name FROM tags ORDER BY id")
        .fetch_all(&pool)
        .await?
        .iter()
        .map(|row| Tag {
            id: row.get("id"),
            name: row.get::<String, _>("name").clone(),
            keyword: row.get::<String, _>("name").to_lowercase(),
        })
        .collect();

    println!("✓ Loaded {} tags", tags.len());
    println!();

    // Get all files
    println!("[3/5] Loading files from database...");
    let files: Vec<FileRecord> =
        sqlx::query("SELECT id, filename, filepath FROM files ORDER BY id")
            .fetch_all(&pool)
            .await?
            .iter()
            .map(|row| FileRecord {
                id: row.get("id"),
                filename: row.get("filename"),
                filepath: row.get("filepath"),
            })
            .collect();

    let file_count = files.len();
    println!("✓ Loaded {} files", file_count);
    println!();

    // Match files to tags (in-memory, super fast)
    println!("[4/5] Matching files to keywords (in-memory)...");
    let mut file_tag_pairs: Vec<(i64, i32)> = Vec::new();
    let mut matches_per_tag: HashMap<String, usize> = HashMap::new();

    for file in &files {
        let filename_lower = file.filename.to_lowercase();
        let filepath_lower = file.filepath.to_lowercase();

        for tag in &tags {
            if filename_lower.contains(&tag.keyword) || filepath_lower.contains(&tag.keyword) {
                file_tag_pairs.push((file.id, tag.id));
                *matches_per_tag.entry(tag.name.clone()).or_insert(0) += 1;
            }
        }
    }

    println!("✓ Found {} file-tag matches", file_tag_pairs.len());
    println!(
        "  Processing rate: {:.0} files/sec",
        file_count as f64 / start_time.elapsed().as_secs_f64()
    );
    println!();

    // OPTIMIZED: Drop constraint, bulk insert, recreate constraint
    println!("[5/7] Dropping unique constraint for fast bulk insert...");
    sqlx::query("ALTER TABLE file_tags DROP CONSTRAINT IF EXISTS file_tags_file_id_tag_id_key")
        .execute(&pool)
        .await?;
    println!("✓ Constraint dropped");
    println!();

    // Bulk insert to database (NO CONFLICT CHECK - much faster!)
    println!(
        "[6/7] Bulk inserting {} relationships...",
        file_tag_pairs.len()
    );

    let batch_size = 50000; // 5x larger batches without conflict checking
    let total_batches = file_tag_pairs.len().div_ceil(batch_size);

    for (batch_num, chunk) in file_tag_pairs.chunks(batch_size).enumerate() {
        // Build VALUES clause
        let mut values = Vec::new();
        for (file_id, tag_id) in chunk {
            values.push(format!("({}, {}, 'rust_organizer')", file_id, tag_id));
        }
        let values_str = values.join(",");

        // Execute batch insert (NO CONFLICT handling - pure speed)
        let query = format!(
            "INSERT INTO file_tags (file_id, tag_id, added_by) VALUES {}",
            values_str
        );

        sqlx::query(&query).execute(&pool).await?;

        if (batch_num + 1) % 5 == 0 || batch_num + 1 == total_batches {
            println!(
                "  Batch {}/{} ({:.1}%)",
                batch_num + 1,
                total_batches,
                (batch_num + 1) as f64 / total_batches as f64 * 100.0
            );
        }
    }

    println!("✓ Inserted {} relationships", file_tag_pairs.len());
    println!();

    // Remove duplicates and recreate constraint
    println!("[7/7] Removing duplicates and recreating unique constraint...");

    // Deduplicate
    sqlx::query(
        r#"
        DELETE FROM file_tags a USING file_tags b
        WHERE a.id > b.id
          AND a.file_id = b.file_id
          AND a.tag_id = b.tag_id
    "#,
    )
    .execute(&pool)
    .await?;

    // Recreate constraint
    sqlx::query("ALTER TABLE file_tags ADD CONSTRAINT file_tags_file_id_tag_id_key UNIQUE (file_id, tag_id)")
        .execute(&pool).await?;

    // Get final count
    let inserted_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM file_tags").fetch_one(&pool).await?;

    println!("✓ Inserted {} relationships", inserted_count);
    println!();

    // Summary
    let duration = start_time.elapsed();
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Organization Complete!");
    println!("═══════════════════════════════════════════════════════════════");
    println!();
    println!("  Total files:     {}", file_count);
    println!("  Total matches:   {}", file_tag_pairs.len());
    println!("  Inserted:        {}", inserted_count);
    println!("  Duration:        {:.2}s", duration.as_secs_f64());
    println!(
        "  Rate:            {:.0} files/sec",
        file_count as f64 / duration.as_secs_f64()
    );
    println!();

    // Show top 10 instruments
    println!("Top 10 instruments:");
    let mut tag_counts: Vec<_> = matches_per_tag.iter().collect();
    tag_counts.sort_by(|a, b| b.1.cmp(a.1));

    for (i, (tag_name, count)) in tag_counts.iter().take(10).enumerate() {
        let percentage = (**count as f64 / file_count as f64) * 100.0;
        println!(
            "  {}. {}: {} files ({:.2}%)",
            i + 1,
            tag_name,
            count,
            percentage
        );
    }
    println!();

    // Create views (fast SQL)
    println!("Creating database views...");
    sqlx::query(
        r#"
        CREATE OR REPLACE VIEW v_drums AS
        SELECT DISTINCT f.id, f.filename, f.filepath, f.hash, f.size, f.created_at,
               m.bpm, m.key_signature, m.duration, m.time_signature
        FROM files f
        JOIN file_tags ft ON f.id = ft.file_id
        JOIN tags t ON ft.tag_id = t.id
        LEFT JOIN musical_metadata m ON f.id = m.file_id
        WHERE t.category = 'drums';
    "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE OR REPLACE VIEW v_melodic AS
        SELECT DISTINCT f.id, f.filename, f.filepath, f.hash, f.size, f.created_at,
               m.bpm, m.key_signature, m.duration, m.time_signature
        FROM files f
        JOIN file_tags ft ON f.id = ft.file_id
        JOIN tags t ON ft.tag_id = t.id
        LEFT JOIN musical_metadata m ON f.id = m.file_id
        WHERE t.category IN ('keys', 'synth', 'strings', 'brass', 'woodwind', 'guitar');
    "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE OR REPLACE VIEW v_bass AS
        SELECT DISTINCT f.id, f.filename, f.filepath, f.hash, f.size, f.created_at,
               m.bpm, m.key_signature, m.duration, m.time_signature
        FROM files f
        JOIN file_tags ft ON f.id = ft.file_id
        JOIN tags t ON ft.tag_id = t.id
        LEFT JOIN musical_metadata m ON f.id = m.file_id
        WHERE t.category = 'bass';
    "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE OR REPLACE VIEW v_loops AS
        SELECT DISTINCT f.id, f.filename, f.filepath, f.hash, f.size, f.created_at,
               m.bpm, m.key_signature, m.duration, m.time_signature
        FROM files f
        JOIN file_tags ft ON f.id = ft.file_id
        JOIN tags t ON ft.tag_id = t.id
        LEFT JOIN musical_metadata m ON f.id = m.file_id
        WHERE t.category = 'pattern';
    "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(r#"
        CREATE OR REPLACE VIEW v_tag_stats AS
        SELECT t.name, t.category, COUNT(DISTINCT ft.file_id) as file_count,
               ROUND(COUNT(DISTINCT ft.file_id)::NUMERIC * 100.0 / (SELECT COUNT(*) FROM files), 2) as percentage
        FROM tags t
        LEFT JOIN file_tags ft ON t.id = ft.tag_id
        GROUP BY t.id, t.name, t.category
        ORDER BY file_count DESC;
    "#).execute(&pool).await?;

    println!("✓ Views created");
    println!();
    println!("═══════════════════════════════════════════════════════════════");
    println!(
        "  ✓ Organization complete in {:.2}s!",
        duration.as_secs_f64()
    );
    println!("═══════════════════════════════════════════════════════════════");

    Ok(())
}

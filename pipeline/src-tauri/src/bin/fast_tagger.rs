use clap::Parser;
use dashmap::DashMap;
use rayon::prelude::*;
use regex::Regex;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Fast multi-level tagging with parallel processing
#[derive(Parser, Debug)]
#[command(name = "fast_tagger")]
#[command(about = "Fast multi-level MIDI file tagging", long_about = None)]
struct Args {
    /// Batch size for database inserts
    #[arg(short, long, default_value = "10000")]
    batch_size: usize,

    /// Chunk size for processing files
    #[arg(short, long, default_value = "5000")]
    chunk_size: usize,

    /// Database URL
    #[arg(
        short,
        long,
        default_value = "postgresql://midiuser:145278963@localhost:5433/midi_library"
    )]
    database_url: String,

    /// Path to curated tags file
    #[arg(short, long, default_value = "/tmp/master_tag_list.txt")]
    tags_file: String,

    /// Number of parallel workers for processing
    #[arg(short, long, default_value = "16")]
    workers: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct PathComponents {
    file_id: i64,
    filepath: String,
    grandparent: String,
    parent: String,
    filename: String,
}

#[derive(Debug, Clone)]
struct FileTag {
    file_id: i64,
    tag_id: i64,
}

struct TaggingStats {
    total_files: AtomicU64,
    processed: AtomicU64,
    tagged: AtomicU64,
    skipped: AtomicU64,
    total_tags_inserted: AtomicU64,
}

impl TaggingStats {
    fn new() -> Self {
        Self {
            total_files: AtomicU64::new(0),
            processed: AtomicU64::new(0),
            tagged: AtomicU64::new(0),
            skipped: AtomicU64::new(0),
            total_tags_inserted: AtomicU64::new(0),
        }
    }

    fn print_progress(&self, start_time: Instant) {
        let processed = self.processed.load(Ordering::Relaxed);
        let tagged = self.tagged.load(Ordering::Relaxed);
        let skipped = self.skipped.load(Ordering::Relaxed);
        let total = self.total_files.load(Ordering::Relaxed);
        let total_tags = self.total_tags_inserted.load(Ordering::Relaxed);
        let elapsed = start_time.elapsed().as_secs_f64();
        let rate = if elapsed > 0.0 {
            processed as f64 / elapsed
        } else {
            0.0
        };

        let avg_tags = if tagged > 0 {
            total_tags as f64 / tagged as f64
        } else {
            0.0
        };

        println!(
            "Progress: {}/{} ({:.1}%) | Tagged: {} | Skipped: {} | Total tags: {} ({:.1} avg) | Rate: {:.0} files/sec",
            processed,
            total,
            (processed as f64 / total as f64) * 100.0,
            tagged,
            skipped,
            total_tags,
            avg_tags,
            rate
        );
    }
}

/// Normalize keyword from text
fn normalize_keyword(text: &str) -> HashSet<String> {
    let mut keywords = HashSet::new();

    if text.is_empty() {
        return keywords;
    }

    // Lowercase
    let text = text.to_lowercase();

    // Replace delimiters with spaces
    let re = Regex::new(r"[_\-\(\)\[\]@#]").unwrap();
    let text = re.replace_all(&text, " ");

    // Common noise words to skip
    let noise_words: HashSet<&str> =
        ["the", "and", "for", "with", "from", "midi", "mid"].iter().cloned().collect();

    // Split and filter
    for word in text.split_whitespace() {
        let word = word.trim();

        // Skip if too short or too long
        if word.len() < 3 || word.len() > 50 {
            continue;
        }

        // Skip numbers-only
        if word.chars().all(|c| c.is_numeric()) {
            continue;
        }

        // Skip noise words
        if noise_words.contains(word) {
            continue;
        }

        keywords.insert(word.to_string());
    }

    keywords
}

/// Extract grandparent, parent, and filename from path
fn extract_path_components(file_id: i64, filepath: &str) -> PathComponents {
    let path = Path::new(filepath);

    // Filename without extension
    let filename = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();

    // Parent folder (1 level up)
    let parent = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    // Grandparent folder (2 levels up)
    let grandparent = path
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    PathComponents { file_id, filepath: filepath.to_string(), grandparent, parent, filename }
}

/// Load curated tags from file and insert into database
async fn load_curated_tags(
    pool: &PgPool,
    tags_file: &str,
) -> Result<HashMap<String, i64>, Box<dyn std::error::Error>> {
    println!("📋 Loading curated tags from: {}", tags_file);
    let start = Instant::now();

    let mut tag_map = HashMap::new();

    // Read tags file
    let content = fs::read_to_string(tags_file)?;
    let tags: Vec<String> = content.lines().map(|s| s.trim().to_string()).collect();

    println!("  Found {} tags in file", tags.len());

    // Insert tags into database (batch)
    let mut tx = pool.begin().await?;

    for tag in &tags {
        let row = sqlx::query!(
            r#"
            INSERT INTO tags (name, category)
            VALUES ($1, 'filename')
            ON CONFLICT (name) DO NOTHING
            RETURNING id
            "#,
            tag
        )
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(row) = row {
            tag_map.insert(tag.clone(), row.id as i64);
        } else {
            // Tag already exists, fetch its ID
            let existing = sqlx::query!("SELECT id FROM tags WHERE name = $1", tag)
                .fetch_one(&mut *tx)
                .await?;
            tag_map.insert(tag.clone(), existing.id as i64);
        }
    }

    tx.commit().await?;

    println!(
        "✅ Loaded {} tags in {:.2}s",
        tag_map.len(),
        start.elapsed().as_secs_f64()
    );

    Ok(tag_map)
}

/// Load already tagged file IDs
async fn load_tagged_files(pool: &PgPool) -> Result<Arc<DashMap<i64, ()>>, sqlx::Error> {
    println!("📂 Loading already-tagged files...");
    let start = Instant::now();

    let tagged = Arc::new(DashMap::new());

    let rows = sqlx::query!("SELECT DISTINCT file_id FROM file_tags").fetch_all(pool).await?;

    for row in rows {
        tagged.insert(row.file_id, ());
    }

    println!(
        "✅ Loaded {} tagged files in {:.2}s",
        tagged.len(),
        start.elapsed().as_secs_f64()
    );

    Ok(tagged)
}

/// Load all files from database
async fn load_files(pool: &PgPool) -> Result<Vec<(i64, String)>, sqlx::Error> {
    println!("📂 Loading files from database...");
    let start = Instant::now();

    let rows = sqlx::query!("SELECT id, filepath FROM files ORDER BY id")
        .fetch_all(pool)
        .await?;

    let files: Vec<(i64, String)> = rows.into_iter().map(|r| (r.id, r.filepath)).collect();

    println!(
        "✅ Loaded {} files in {:.2}s",
        files.len(),
        start.elapsed().as_secs_f64()
    );

    Ok(files)
}

/// Process files and extract matching tags
fn process_files_for_tags(
    files: Vec<PathComponents>,
    tag_map: &HashMap<String, i64>,
    stats: Arc<TaggingStats>,
) -> Vec<FileTag> {
    println!("⚙️ Processing files for tag matching...");

    let file_tags: Vec<FileTag> = files
        .par_iter()
        .flat_map(|file| {
            stats.processed.fetch_add(1, Ordering::Relaxed);

            let mut keywords = HashSet::new();

            // Extract keywords from grandparent, parent, and filename
            keywords.extend(normalize_keyword(&file.grandparent));
            keywords.extend(normalize_keyword(&file.parent));
            keywords.extend(normalize_keyword(&file.filename));

            // Match keywords against tag map
            let mut file_tags = Vec::new();
            for keyword in keywords {
                if let Some(&tag_id) = tag_map.get(&keyword) {
                    file_tags.push(FileTag { file_id: file.file_id, tag_id });
                }
            }

            if !file_tags.is_empty() {
                stats.tagged.fetch_add(1, Ordering::Relaxed);
                stats.total_tags_inserted.fetch_add(file_tags.len() as u64, Ordering::Relaxed);
            }

            file_tags
        })
        .collect();

    println!("✅ Matched {} tag relationships", file_tags.len());
    file_tags
}

/// Batch insert file tags into database
async fn batch_insert_tags(
    pool: &PgPool,
    file_tags: Vec<FileTag>,
    batch_size: usize,
) -> Result<(), sqlx::Error> {
    println!(
        "💾 Inserting {} tag relationships in batches of {}...",
        file_tags.len(),
        batch_size
    );
    let start = Instant::now();

    for (batch_num, chunk) in file_tags.chunks(batch_size).enumerate() {
        let mut tx = pool.begin().await?;

        for ft in chunk {
            sqlx::query!(
                r#"
                INSERT INTO file_tags (file_id, tag_id)
                VALUES ($1, $2)
                ON CONFLICT (file_id, tag_id) DO NOTHING
                "#,
                ft.file_id,
                ft.tag_id as i32
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        if (batch_num + 1) % 10 == 0 {
            println!(
                "  Batch {}/{} | Elapsed: {:.1}s",
                batch_num + 1,
                (file_tags.len() + batch_size - 1) / batch_size,
                start.elapsed().as_secs_f64()
            );
        }
    }

    println!(
        "✅ Database insert complete in {:.2}s",
        start.elapsed().as_secs_f64()
    );
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🚀 Fast Multi-Level Tagger (Rust Edition)");
    println!("═══════════════════════════════════════════");
    println!("Batch size: {}", args.batch_size);
    println!("Chunk size: {}", args.chunk_size);
    println!("Workers: {}", args.workers);
    println!();

    // Set rayon thread pool
    rayon::ThreadPoolBuilder::new()
        .num_threads(args.workers)
        .build_global()
        .unwrap();

    // Connect to database
    println!("🔌 Connecting to database...");
    let pool = PgPoolOptions::new().max_connections(20).connect(&args.database_url).await?;
    println!("✅ Database connected");

    // Load curated tags
    let tag_map = load_curated_tags(&pool, &args.tags_file).await?;

    // Load already-tagged files
    let tagged_files = load_tagged_files(&pool).await?;

    // Load all files
    let all_files = load_files(&pool).await?;

    // Initialize stats
    let stats = Arc::new(TaggingStats::new());
    stats.total_files.store(all_files.len() as u64, Ordering::Relaxed);

    println!();
    println!("📊 Total files: {}", all_files.len());
    println!("📊 Already tagged: {}", tagged_files.len());
    println!(
        "📊 Files to process: {}",
        all_files.len() - tagged_files.len()
    );
    println!();

    let start_time = Instant::now();

    // Process files in chunks
    for (chunk_num, chunk) in all_files.chunks(args.chunk_size).enumerate() {
        println!(
            "Processing chunk {}/{} ({} files)...",
            chunk_num + 1,
            (all_files.len() + args.chunk_size - 1) / args.chunk_size,
            chunk.len()
        );

        // Extract path components and filter already-tagged
        let files_to_process: Vec<PathComponents> = chunk
            .iter()
            .filter(|(file_id, _)| !tagged_files.contains_key(file_id))
            .map(|(file_id, filepath)| extract_path_components(*file_id, filepath))
            .collect();

        if files_to_process.is_empty() {
            println!("  All files in chunk already tagged, skipping...");
            stats.skipped.fetch_add(chunk.len() as u64, Ordering::Relaxed);
            continue;
        }

        // Process files for tags
        let file_tags = process_files_for_tags(files_to_process, &tag_map, Arc::clone(&stats));

        // Batch insert into database
        if !file_tags.is_empty() {
            batch_insert_tags(&pool, file_tags, args.batch_size).await?;
        }

        // Print progress
        stats.print_progress(start_time);
        println!();
    }

    // Final stats
    println!("═══════════════════════════════════════════");
    println!("✅ Tagging Complete!");
    stats.print_progress(start_time);
    println!("Total time: {:.2}s", start_time.elapsed().as_secs_f64());
    println!("═══════════════════════════════════════════");

    Ok(())
}

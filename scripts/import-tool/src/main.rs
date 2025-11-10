/// Simple High-Performance MIDI Importer
///
/// Usage: ./import_midi ~/midi_extraction
///
/// This script:
/// - Scans recursively for .mid/.midi files
/// - Calculates SHA-256 hash for deduplication
/// - Inserts into database with parallel processing
/// - Shows real-time progress
use chrono::Utc;
use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
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
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

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
            let _permit = match sem.acquire().await {
                Ok(permit) => permit,
                Err(_) => {
                    eprintln!("Warning: Semaphore closed during import");
                    return;
                },
            };

            match process_file(&file_path, &pool_clone).await {
                Ok(true) => {
                    imported.fetch_add(1, Ordering::SeqCst);
                },
                Ok(false) => {
                    skipped.fetch_add(1, Ordering::SeqCst);
                },
                Err(_) => {
                    errors.fetch_add(1, Ordering::SeqCst);
                },
            }

            // Print progress every 100 files
            if idx % 100 == 0 {
                let elapsed = start_time.elapsed().as_secs_f64();
                let processed = idx + 1;
                let rate = processed as f64 / elapsed;
                println!(
                    "Progress: {}/{} ({:.1}%) - {:.0} files/sec",
                    processed,
                    total,
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
    println!(
        "⚡ Rate:  {:.0} files/sec",
        total as f64 / elapsed.as_secs_f64()
    );
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
        original_filename[..original_filename.len() - 1].to_string() // .midi -> .mid
    } else {
        original_filename.clone()
    };

    let filepath = file_path.to_str().unwrap_or("").to_string();

    // Extract filename metadata (Phase 2)
    let metadata = FilenameMetadata::extract_from_filename(&filename);

    // Calculate metadata source
    let metadata_source = calculate_metadata_source(
        metadata.bpm,
        metadata.key.as_deref(),
        &metadata.genres,
        &metadata.structure_tags,
    );

    // Insert into database with Phase 2 columns
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
            filename_bpm,
            filename_key,
            filename_genres,
            structure_tags,
            track_number,
            metadata_source,
            created_at,
            updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
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
    .bind(metadata.bpm)
    .bind(metadata.key)
    .bind(&metadata.genres)
    .bind(&metadata.structure_tags)
    .bind(metadata.track_number)
    .bind(metadata_source)
    .bind(Utc::now())
    .bind(Utc::now())
    .fetch_optional(pool)
    .await?;

    Ok(result.is_some())
}

// ============================================================================
// Filename Metadata Extraction (Phase 2)
// ============================================================================

/// Filename metadata extracted from a MIDI file name
#[derive(Debug, Clone)]
struct FilenameMetadata {
    bpm: Option<f32>,
    key: Option<String>,
    genres: Vec<String>,
    structure_tags: Vec<String>,
    track_number: Option<i32>,
}

impl FilenameMetadata {
    fn extract_from_filename(filename: &str) -> Self {
        FilenameMetadata {
            bpm: extract_bpm_from_filename(filename),
            key: extract_key_from_filename(filename),
            genres: extract_genres_from_filename(filename),
            structure_tags: extract_structure_tags(filename),
            track_number: extract_leading_number(filename),
        }
    }
}

// BPM Extraction
static BPM_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_bpm_regex() -> &'static Regex {
    BPM_REGEX.get_or_init(|| {
        Regex::new(
            r"(?i)([0-9]{2,3})[\s_]*(bpm|beats|tempo)|(?:^|_|\s|-|/)([0-9]{2,3})(?:_|\s|-|/|\.)",
        )
        .unwrap()
    })
}

fn extract_bpm_from_filename(filename: &str) -> Option<f32> {
    let regex = get_bpm_regex();

    // First, try to find explicit BPM notation
    for caps in regex.captures_iter(filename) {
        if let Some(m) = caps.get(1) {
            if let Ok(bpm) = m.as_str().parse::<f32>() {
                if (30.0..=300.0).contains(&bpm) {
                    return Some(bpm);
                }
            }
        }
    }

    // If no explicit BPM found, try implicit numbers
    for caps in regex.captures_iter(filename) {
        if let Some(m) = caps.get(3) {
            if let Ok(bpm) = m.as_str().parse::<f32>() {
                if (30.0..=300.0).contains(&bpm) {
                    return Some(bpm);
                }
            }
        }
    }

    None
}

// Key Signature Extraction
static KEY_REGEX: OnceLock<Regex> = OnceLock::new();
static KEY_NORMALIZATION_MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

fn get_key_regex() -> &'static Regex {
    KEY_REGEX
        .get_or_init(|| Regex::new(r"(?i)(?:^|_|\s|-|/)([A-G](?:#|b)?m?)(?:_|\s|-|/|\.)").unwrap())
}

fn get_key_map() -> &'static HashMap<&'static str, &'static str> {
    KEY_NORMALIZATION_MAP.get_or_init(|| {
        HashMap::from([
            ("amin", "Am"),
            ("am", "Am"),
            ("bmin", "Bm"),
            ("bm", "Bm"),
            ("cmin", "Cm"),
            ("cm", "Cm"),
            ("dmin", "Dm"),
            ("dm", "Dm"),
            ("emin", "Em"),
            ("em", "Em"),
            ("fmin", "Fm"),
            ("fm", "Fm"),
            ("gmin", "Gm"),
            ("gm", "Gm"),
            ("amaj", "A"),
            ("cmaj", "C"),
            ("dmaj", "D"),
            ("emaj", "E"),
            ("fmaj", "F"),
            ("gmaj", "G"),
            ("bmaj", "B"),
            ("bb", "Bb"),
            ("a#", "A#"),
            ("c#", "C#"),
            ("d#", "D#"),
            ("f#", "F#"),
            ("g#", "G#"),
            ("ab", "Ab"),
            ("db", "Db"),
            ("eb", "Eb"),
            ("gb", "Gb"),
            ("a", "A"),
            ("b", "B"),
            ("c", "C"),
            ("d", "D"),
            ("e", "E"),
            ("f", "F"),
            ("g", "G"),
        ])
    })
}

fn normalize_key_signature(raw_key: &str) -> Option<String> {
    get_key_map().get(raw_key.to_lowercase().as_str()).map(|&s| s.to_string())
}

fn extract_key_from_filename(filename: &str) -> Option<String> {
    get_key_regex()
        .captures(filename)
        .and_then(|caps| caps.get(1))
        .and_then(|m| normalize_key_signature(m.as_str()))
}

// Genre Extraction
static GENRE_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_genre_regex() -> &'static Regex {
    GENRE_REGEX.get_or_init(|| {
        Regex::new(r"(?i)(house|techno|trance|hip.?hop|trap|dubstep|dnb|drum.?n.?bass|jazz|funk|soul|rock|pop|edm|ambient|downtempo|break|jungle|garage|electro|acid|minimal|deep|progressive)").unwrap()
    })
}

fn normalize_genre(raw: &str) -> String {
    match raw {
        "hip hop" | "hiphop" | "hip-hop" => "hip-hop".to_string(),
        "dnb" | "drum n bass" | "drum and bass" | "drum-n-bass" | "drum_n_bass" => {
            "dnb".to_string()
        },
        genre => genre.to_string(),
    }
}

fn extract_genres_from_filename(filename: &str) -> Vec<String> {
    get_genre_regex()
        .find_iter(filename)
        .map(|m| m.as_str().to_lowercase())
        .map(|g| normalize_genre(&g))
        .collect()
}

// Structure Tag Extraction
static STRUCTURE_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_structure_regex() -> &'static Regex {
    STRUCTURE_REGEX.get_or_init(|| {
        Regex::new(r"(?i)(verse|chorus|bridge|intro|outro|drop|build|breakdown|fill|loop|one.?shot|sample|melody|hook|riff|lick|main|full|short|long)").unwrap()
    })
}

fn normalize_structure_tag(raw: &str) -> String {
    match raw {
        "one shot" | "one-shot" | "oneshot" => "oneshot".to_string(),
        tag => tag.to_string(),
    }
}

fn extract_structure_tags(filename: &str) -> Vec<String> {
    get_structure_regex()
        .find_iter(filename)
        .map(|m| m.as_str().to_lowercase())
        .map(|s| normalize_structure_tag(&s))
        .collect()
}

// Track Number Extraction
static LEADING_NUMBER_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_leading_number_regex() -> &'static Regex {
    LEADING_NUMBER_REGEX.get_or_init(|| Regex::new(r"^([0-9]+)").unwrap())
}

fn extract_leading_number(filename: &str) -> Option<i32> {
    get_leading_number_regex()
        .captures(filename)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse::<i32>().ok())
}

/// Calculate metadata source based on what data is available
fn calculate_metadata_source(
    filename_bpm: Option<f32>,
    filename_key: Option<&str>,
    filename_genres: &[String],
    structure_tags: &[String],
) -> &'static str {
    let has_filename_metadata = filename_bpm.is_some()
        || filename_key.is_some()
        || !filename_genres.is_empty()
        || !structure_tags.is_empty();

    if has_filename_metadata {
        "filename"
    } else {
        "none"
    }
}

// ============================================================================
// MIDI Parsing
// ============================================================================

/// Parse basic MIDI info (tracks and TPPQ)
fn parse_midi_basic(data: &[u8]) -> (i16, i32) {
    if data.len() < 14 || &data[0..4] != b"MThd" {
        return (1, 480);
    }

    let num_tracks = i16::from_be_bytes([data[10], data[11]]);
    let ticks = u16::from_be_bytes([data[12], data[13]]);

    (num_tracks, ticks as i32)
}

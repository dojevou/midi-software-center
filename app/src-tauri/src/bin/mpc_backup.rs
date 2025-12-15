/// MPC Backup Creator - Generate MPC-compatible filenames and copy to backup drive
///
/// Creates MPC-compatible names with format: {BARS}{KEY}{BPM}{FOLDER}{FILENAME}.mid
/// Example: 8c+80VirusMelodies01.mid
///
/// Performance target: 10,000 files/sec
/// Parallel processing with 48 workers
use anyhow::{Context, Result};
use clap::Parser;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::fs;

#[derive(Parser, Debug)]
#[command(name = "mpc_backup")]
#[command(about = "MPC Backup Creator - Copy files with MPC-compatible names")]
struct Args {
    /// Output directory for MPC library
    #[arg(short, long, default_value = "/media/dojevou/RYXSTR/MPC_MIDI_LIBRARY")]
    output_dir: PathBuf,

    /// Number of parallel workers
    #[arg(short, long, default_value_t = 48)]
    workers: usize,

    /// Database connection string
    #[arg(short = 'D', long, env = "DATABASE_URL")]
    database_url: String,

    /// Batch size for database queries
    #[arg(short = 'b', long, default_value_t = 1000)]
    batch_size: i64,

    /// Test mode - only process first N files
    #[arg(short = 't', long)]
    test_limit: Option<i64>,

    /// Organization strategy (flat, by_bpm, by_key, by_instrument)
    #[arg(long, default_value = "flat")]
    organize_by: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct MidiFileWithMetadata {
    id: i64,
    filepath: String,
    filename: String,
    duration_seconds: Option<f64>,
    bpm: Option<f64>,
    key_signature: Option<String>,
    time_signature_numerator: Option<i16>,
    time_signature_denominator: Option<i16>,
    // Musical metadata for categorization
    is_percussive: Option<bool>,
    has_chords: Option<bool>,
    has_melody: Option<bool>,
    is_monophonic: Option<bool>,
    polyphony_avg: Option<f32>,
}

#[derive(Debug)]
struct Stats {
    files_processed: AtomicU64,
    files_copied: AtomicU64,
    files_skipped: AtomicU64,
    errors: AtomicU64,
}

impl Stats {
    fn new() -> Self {
        Self {
            files_processed: AtomicU64::new(0),
            files_copied: AtomicU64::new(0),
            files_skipped: AtomicU64::new(0),
            errors: AtomicU64::new(0),
        }
    }

    fn inc_processed(&self) {
        self.files_processed.fetch_add(1, Ordering::Relaxed);
    }

    fn inc_copied(&self) {
        self.files_copied.fetch_add(1, Ordering::Relaxed);
    }

    fn inc_skipped(&self) {
        self.files_skipped.fetch_add(1, Ordering::Relaxed);
    }

    fn inc_errors(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }
}

/// Calculate number of bars from duration, BPM, and time signature
fn calculate_bars(duration: f64, bpm: f64, time_sig_numerator: i16) -> i32 {
    let beats_per_bar = time_sig_numerator as f64;
    let seconds_per_beat = 60.0 / bpm;
    let seconds_per_bar = seconds_per_beat * beats_per_bar;
    let bars = (duration / seconds_per_bar).round() as i32;

    // Clamp to reasonable range (1-999 bars)
    bars.clamp(1, 999)
}

/// Convert musical key to MPC format
/// Examples: "C" → "c+", "Cm" → "c-", "F#" → "fs+", "F#m" → "fs-"
fn format_key(key: &str) -> String {
    let is_minor = key.ends_with('m');
    let note = key.trim_end_matches('m').replace(['#', 'b'], "s").to_lowercase();

    if is_minor {
        format!("{}-", note)
    } else {
        format!("{}+", note)
    }
}

/// Extract parent folder name from filepath
fn extract_folder(filepath: &str) -> String {
    let path = Path::new(filepath);
    let parent = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown");

    sanitize_camelcase(parent, 20)
}

/// Sanitize string to keep only: a-z, A-Z, 0-9, _, -, +
fn sanitize_mpc(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == '+')
        .collect()
}

/// Convert to CamelCase and truncate to max_len
fn sanitize_camelcase(input: &str, max_len: usize) -> String {
    // Split by spaces/special chars, capitalize first letter of each word
    let words: Vec<String> = input
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.to_lowercase().chars().collect::<Vec<_>>();
            if let Some(first) = chars.first_mut() {
                *first = first.to_uppercase().next().unwrap_or(*first);
            }
            chars.into_iter().collect()
        })
        .collect();

    let camelcase = words.join("");
    let sanitized = sanitize_mpc(&camelcase);

    // Truncate to max_len
    sanitized.chars().take(max_len).collect()
}

/// Generate MPC-compatible filename
/// Format: {BARS}{KEY}{BPM}{FOLDER}{FILENAME}.mid
/// Example: 8c+80VirusMelodies01.mid
fn generate_mpc_filename(file: &MidiFileWithMetadata) -> Option<String> {
    // Need at least BPM and key
    let bpm = file.bpm?;
    let key = file.key_signature.as_ref()?;
    let duration = file.duration_seconds?;
    let time_sig_numerator = file.time_signature_numerator.unwrap_or(4);

    // Calculate bars
    let bars = calculate_bars(duration, bpm, time_sig_numerator);

    // Format key
    let key_formatted = format_key(key);

    // Extract folder name
    let folder = extract_folder(&file.filepath);

    // Extract filename stem (without extension)
    let filename_stem = Path::new(&file.filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown");

    let filename_sanitized = sanitize_mpc(filename_stem).chars().take(15).collect::<String>();

    // Build MPC name: {BARS}{KEY}{BPM}{FOLDER}{FILENAME}.mid
    let mpc_name = format!(
        "{}{}{}{}{}.mid",
        bars,
        key_formatted,
        bpm.round() as i32,
        folder,
        filename_sanitized
    );

    Some(mpc_name)
}

/// Determine output folder based on organization strategy
fn get_output_folder(base_dir: &Path, organize_by: &str, file: &MidiFileWithMetadata) -> PathBuf {
    match organize_by {
        "by_bpm" | "bpm" => {
            let bpm = file.bpm.unwrap_or(120.0).round() as i32;
            let bpm_range = match bpm {
                0..=80 => "000-080",
                81..=100 => "081-100",
                101..=120 => "101-120",
                121..=140 => "121-140",
                141..=160 => "141-160",
                _ => "160-999",
            };
            base_dir.join(bpm_range)
        },
        "by_key" | "key" => {
            let key = file
                .key_signature
                .as_ref()
                .map(|k| sanitize_mpc(k))
                .unwrap_or_else(|| "Unknown".to_string());
            base_dir.join(key)
        },
        "by_instrument" | "instrument" => {
            // Four-level categorization: instrument group > instrument subgroup > bar length > key
            // Priority: metadata > filename > filepath

            let path_lower = file.filepath.to_lowercase();
            let filename_lower = file.filename.to_lowercase();

            // Determine instrument group and subgroup
            let (group, subgroup) = if file.is_percussive == Some(true) {
                // Percussive files - try to categorize by filename/path
                if filename_lower.contains("kick") || path_lower.contains("kick") {
                    ("DRUMS", "KICKS")
                } else if filename_lower.contains("snare") || path_lower.contains("snare") {
                    ("DRUMS", "SNARES")
                } else if filename_lower.contains("hihat")
                    || filename_lower.contains("hat")
                    || path_lower.contains("hihat")
                    || path_lower.contains("hat")
                {
                    ("DRUMS", "HIHATS")
                } else if filename_lower.contains("cymbal")
                    || filename_lower.contains("crash")
                    || filename_lower.contains("ride")
                    || path_lower.contains("cymbal")
                    || path_lower.contains("crash")
                    || path_lower.contains("ride")
                {
                    ("DRUMS", "CYMBALS")
                } else if filename_lower.contains("tom") || path_lower.contains("tom") {
                    ("DRUMS", "TOMS")
                } else if filename_lower.contains("percussion")
                    || filename_lower.contains("perc")
                    || path_lower.contains("percussion")
                    || path_lower.contains("perc")
                {
                    ("DRUMS", "PERCUSSION")
                } else {
                    ("DRUMS", "OTHER")
                }
            } else if file.has_chords == Some(true) {
                // Chordal content - likely keys
                if filename_lower.contains("piano") || path_lower.contains("piano") {
                    ("KEYS", "PIANO")
                } else if filename_lower.contains("organ") || path_lower.contains("organ") {
                    ("KEYS", "ORGAN")
                } else if filename_lower.contains("synth") || path_lower.contains("synth") {
                    ("KEYS", "SYNTH")
                } else {
                    ("KEYS", "CHORDS")
                }
            } else if file.has_melody == Some(true) {
                // Melodic content
                if file.is_monophonic == Some(true) {
                    // Monophonic melodies
                    if filename_lower.contains("bass") || path_lower.contains("bass") {
                        ("BASS", "BASS")
                    } else if filename_lower.contains("lead") || path_lower.contains("lead") {
                        ("LEADS", "LEAD")
                    } else {
                        ("LEADS", "MELODY")
                    }
                } else {
                    // Polyphonic melodies
                    if filename_lower.contains("arp") || path_lower.contains("arp") {
                        ("LEADS", "ARP")
                    } else {
                        ("LEADS", "POLYPHONIC")
                    }
                }
            } else if file.is_monophonic == Some(true) && file.polyphony_avg.unwrap_or(0.0) < 2.0 {
                // Low polyphony monophonic = likely bass or lead
                if filename_lower.contains("bass") || path_lower.contains("bass") {
                    ("BASS", "BASS")
                } else {
                    ("BASS", "MONOPHONIC")
                }
            } else if filename_lower.contains("loop") || path_lower.contains("loop") {
                ("LOOPS", "LOOPS")
            } else {
                // Fallback to filename/path detection
                if filename_lower.contains("bass") || path_lower.contains("bass") {
                    ("BASS", "BASS")
                } else if filename_lower.contains("piano") || path_lower.contains("piano") {
                    ("KEYS", "PIANO")
                } else if filename_lower.contains("synth") || path_lower.contains("synth") {
                    ("KEYS", "SYNTH")
                } else if filename_lower.contains("organ") || path_lower.contains("organ") {
                    ("KEYS", "ORGAN")
                } else if filename_lower.contains("lead") || path_lower.contains("lead") {
                    ("LEADS", "LEAD")
                } else if filename_lower.contains("melody") || path_lower.contains("melody") {
                    ("LEADS", "MELODY")
                } else {
                    ("OTHER", "OTHER")
                }
            };

            // Calculate bar length for subfolder
            let bars = if let (Some(duration), Some(bpm), Some(time_sig)) = (
                file.duration_seconds,
                file.bpm,
                file.time_signature_numerator,
            ) {
                calculate_bars(duration, bpm, time_sig)
            } else {
                0
            };

            // Categorize bar length into ranges
            let bar_range = match bars {
                0 => "000-BARS",           // Unknown
                1..=2 => "001-002-BARS",   // Very short (fills, hits)
                3..=4 => "003-004-BARS",   // Short (1 bar, 2 bar loops)
                5..=8 => "005-008-BARS",   // Medium (4 bar loops)
                9..=16 => "009-016-BARS",  // Long (8 bar loops)
                17..=32 => "017-032-BARS", // Very long (16 bar phrases)
                _ => "033-PLUS-BARS",      // Extra long
            };

            // Get key for subfolder
            let key_folder = file
                .key_signature
                .as_ref()
                .map(|k| format_key(k).to_uppercase())
                .unwrap_or_else(|| "UNKNOWN".to_string());

            // Four-level structure: GROUP/SUBGROUP/BARS/KEY/
            base_dir.join(group).join(subgroup).join(bar_range).join(key_folder)
        },
        _ => base_dir.to_path_buf(), // "flat" - all in root
    }
}

/// Process a single file: generate MPC name and copy to output directory
async fn process_file(
    file: MidiFileWithMetadata,
    output_dir: &Path,
    organize_by: &str,
    stats: Arc<Stats>,
) -> Result<()> {
    stats.inc_processed();

    // Generate MPC filename
    let mpc_filename = match generate_mpc_filename(&file) {
        Some(name) => name,
        None => {
            stats.inc_skipped();
            return Ok(()); // Skip files without metadata
        },
    };

    // Determine output folder
    let output_folder = get_output_folder(output_dir, organize_by, &file);

    // Create output folder if needed
    fs::create_dir_all(&output_folder)
        .await
        .with_context(|| format!("Failed to create directory: {:?}", output_folder))?;

    // Build output path - handle collisions with numeric suffix
    let mut output_path = output_folder.join(&mpc_filename);
    let mut counter = 1;

    // If file exists, add numeric suffix until we find an available name
    while output_path.exists() {
        // Extract stem and extension
        let stem = Path::new(&mpc_filename).file_stem().and_then(|s| s.to_str()).unwrap_or("file");

        // Create new filename with counter: filename_001.mid, filename_002.mid, etc.
        let new_filename = format!("{}_{:03}.mid", stem, counter);
        output_path = output_folder.join(&new_filename);
        counter += 1;

        // Safety limit to prevent infinite loops
        if counter > 999 {
            stats.inc_skipped();
            eprintln!("Too many duplicates for {}, skipping", file.filepath);
            return Ok(());
        }
    }

    // Copy file to output
    match fs::copy(&file.filepath, &output_path).await {
        Ok(_) => {
            stats.inc_copied();
            Ok(())
        },
        Err(e) => {
            stats.inc_errors();
            eprintln!(
                "Error copying {} to {}: {}",
                file.filepath,
                output_path.display(),
                e
            );
            Ok(()) // Don't fail the whole batch on one error
        },
    }
}

/// Fetch files with musical metadata from database
async fn fetch_files_batch(
    pool: &Pool<Postgres>,
    offset: i64,
    limit: i64,
) -> Result<Vec<MidiFileWithMetadata>> {
    use sqlx::Row;

    let rows = sqlx::query(
        r#"
        SELECT
            f.id,
            f.filepath,
            f.filename,
            f.duration_seconds::FLOAT8 as duration_seconds,
            mm.bpm::FLOAT8 as bpm,
            mm.key_signature::TEXT as key_signature,
            mm.time_signature_numerator,
            mm.time_signature_denominator,
            mm.is_percussive,
            mm.has_chords,
            mm.has_melody,
            mm.is_monophonic,
            mm.polyphony_avg::FLOAT4 as polyphony_avg
        FROM files f
        JOIN musical_metadata mm ON f.id = mm.file_id
        WHERE mm.bpm IS NOT NULL
          AND mm.key_signature IS NOT NULL
          AND f.duration_seconds IS NOT NULL
        ORDER BY f.id
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .context("Failed to fetch files from database")?;

    let files = rows
        .into_iter()
        .map(|row| MidiFileWithMetadata {
            id: row.get("id"),
            filepath: row.get("filepath"),
            filename: row.get("filename"),
            duration_seconds: row.get("duration_seconds"),
            bpm: row.get("bpm"),
            key_signature: row.get("key_signature"),
            time_signature_numerator: row.get("time_signature_numerator"),
            time_signature_denominator: row.get("time_signature_denominator"),
            is_percussive: row.get("is_percussive"),
            has_chords: row.get("has_chords"),
            has_melody: row.get("has_melody"),
            is_monophonic: row.get("is_monophonic"),
            polyphony_avg: row.get("polyphony_avg"),
        })
        .collect();

    Ok(files)
}

/// Get total count of files with metadata
async fn get_total_count(pool: &Pool<Postgres>) -> Result<i64> {
    let (count,): (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM files f
        JOIN musical_metadata mm ON f.id = mm.file_id
        WHERE mm.bpm IS NOT NULL
          AND mm.key_signature IS NOT NULL
          AND f.duration_seconds IS NOT NULL
        "#,
    )
    .fetch_one(pool)
    .await
    .context("Failed to count files")?;

    Ok(count)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("🎹 MPC Backup Creator");
    println!("Output directory: {}", args.output_dir.display());
    println!("Workers: {}", args.workers);
    println!("Organization: {}", args.organize_by);

    // Create output directory
    fs::create_dir_all(&args.output_dir)
        .await
        .context("Failed to create output directory")?;

    // Connect to database
    println!("\n📊 Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(args.workers as u32 + 10)
        .connect(&args.database_url)
        .await
        .context("Failed to connect to database")?;

    // Get total count
    let total_count = match args.test_limit {
        Some(limit) => limit.min(get_total_count(&pool).await?),
        None => get_total_count(&pool).await?,
    };

    println!("Total files to process: {}", total_count);

    if total_count == 0 {
        println!("✅ No files to process");
        return Ok(());
    }

    // Create progress bar
    let progress = ProgressBar::new(total_count as u64);
    progress.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}",
            )
            .unwrap()
            .progress_chars("#>-"),
    );

    // Stats
    let stats = Arc::new(Stats::new());

    // Process in batches
    let mut offset = 0i64;
    let batch_size = args.batch_size;

    while offset < total_count {
        let limit = batch_size.min(total_count - offset);

        // Fetch batch
        let files = fetch_files_batch(&pool, offset, limit).await?;

        if files.is_empty() {
            break;
        }

        let num_files = files.len();

        // Process batch in parallel
        let output_dir = args.output_dir.clone();
        let organize_by = args.organize_by.clone();
        let stats_clone = stats.clone();

        stream::iter(files)
            .map(|file| {
                let output_dir = output_dir.clone();
                let organize_by = organize_by.clone();
                let stats = stats_clone.clone();
                async move { process_file(file, &output_dir, &organize_by, stats).await }
            })
            .buffer_unordered(args.workers)
            .collect::<Vec<_>>()
            .await;

        progress.inc(num_files as u64);
        offset += num_files as i64;

        // Update message
        let _processed = stats.files_processed.load(Ordering::Relaxed);
        let copied = stats.files_copied.load(Ordering::Relaxed);
        let skipped = stats.files_skipped.load(Ordering::Relaxed);
        let errors = stats.errors.load(Ordering::Relaxed);

        progress.set_message(format!(
            "copied: {}, skipped: {}, errors: {}",
            copied, skipped, errors
        ));
    }

    progress.finish_with_message("Done!");

    // Final stats
    println!("\n✅ MPC Backup Complete!");
    println!(
        "Files processed: {}",
        stats.files_processed.load(Ordering::Relaxed)
    );
    println!(
        "Files copied: {}",
        stats.files_copied.load(Ordering::Relaxed)
    );
    println!(
        "Files skipped: {}",
        stats.files_skipped.load(Ordering::Relaxed)
    );
    println!("Errors: {}", stats.errors.load(Ordering::Relaxed));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_bars() {
        // 4/4 time, 120 BPM, 8 seconds = 4 bars
        assert_eq!(calculate_bars(8.0, 120.0, 4), 4);

        // 4/4 time, 80 BPM, 12 seconds = 4 bars
        assert_eq!(calculate_bars(12.0, 80.0, 4), 4);

        // 3/4 time, 120 BPM, 6 seconds = 4 bars
        assert_eq!(calculate_bars(6.0, 120.0, 3), 4);
    }

    #[test]
    fn test_format_key() {
        assert_eq!(format_key("C"), "c+");
        assert_eq!(format_key("Cm"), "c-");
        assert_eq!(format_key("F#"), "fs+");
        assert_eq!(format_key("F#m"), "fs-");
        assert_eq!(format_key("Db"), "ds+");
        assert_eq!(format_key("Dbm"), "ds-");
    }

    #[test]
    fn test_sanitize_mpc() {
        assert_eq!(sanitize_mpc("Hello World!"), "HelloWorld");
        assert_eq!(sanitize_mpc("Bass-Line_01"), "Bass-Line_01");
        assert_eq!(sanitize_mpc("C+ Major"), "C+Major"); // + is kept
        assert_eq!(sanitize_mpc("!@#$%^&*()"), "");
        assert_eq!(sanitize_mpc("Test_123-ABC+"), "Test_123-ABC+");
    }

    #[test]
    fn test_sanitize_camelcase() {
        assert_eq!(sanitize_camelcase("virus melodies", 20), "VirusMelodies");
        assert_eq!(sanitize_camelcase("DRUM LOOPS", 20), "DrumLoops");
        assert_eq!(
            sanitize_camelcase("Bass Lines & Leads", 20),
            "BassLinesLeads"
        );
        assert_eq!(sanitize_camelcase("!PICK AND MIX!", 20), "PickAndMix");

        // Test truncation
        let long_name = "ThisIsAReallyLongFolderNameThatShouldBeTruncated";
        assert_eq!(sanitize_camelcase(long_name, 20).len(), 20);
    }

    #[test]
    fn test_extract_folder() {
        assert_eq!(
            extract_folder("/path/to/VIRUS MELODIES/01.mid"),
            "VirusMelodies"
        );
        assert_eq!(
            extract_folder("/media/dojevou/Drum Loops/kick.mid"),
            "DrumLoops"
        );
    }

    #[test]
    fn test_generate_mpc_filename() {
        let file = MidiFileWithMetadata {
            id: 1,
            filepath: "/media/dojevou/MPCONEDATA/MPC Documents/MIDI Learn/!PICK AND MIX/VIRUS MELODIES/01.mid".to_string(),
            filename: "01.mid".to_string(),
            duration_seconds: Some(8.0),
            bpm: Some(80.0),
            key_signature: Some("C".to_string()),
            time_signature_numerator: Some(4),
            time_signature_denominator: Some(4),
            is_percussive: None,
            has_chords: None,
            has_melody: None,
            is_monophonic: None,
            polyphony_avg: None,
        };

        let mpc_name = generate_mpc_filename(&file).unwrap();

        // Should be: 3c+80VirusMelodies01.mid
        // (8 seconds / (60/80 * 4) = 8 / 3 = 2.67 → 3 bars)
        assert!(mpc_name.starts_with("3")); // bars
        assert!(mpc_name.contains("c+")); // C major
        assert!(mpc_name.contains("80")); // BPM
        assert!(mpc_name.contains("VirusMelodies")); // folder
        assert!(mpc_name.contains("01")); // filename
        assert!(mpc_name.ends_with(".mid"));
    }

    #[test]
    fn test_generate_mpc_filename_minor_key() {
        let file = MidiFileWithMetadata {
            id: 2,
            filepath: "/path/to/Drum Loops/Heavy Kick 120.mid".to_string(),
            filename: "Heavy Kick 120.mid".to_string(),
            duration_seconds: Some(8.0),
            bpm: Some(120.0),
            key_signature: Some("Dm".to_string()),
            time_signature_numerator: Some(4),
            time_signature_denominator: Some(4),
            is_percussive: None,
            has_chords: None,
            has_melody: None,
            is_monophonic: None,
            polyphony_avg: None,
        };

        let mpc_name = generate_mpc_filename(&file).unwrap();

        assert!(mpc_name.contains("d-")); // D minor
        assert!(mpc_name.contains("120")); // BPM
        assert!(mpc_name.contains("DrumLoops")); // folder
    }

    #[test]
    fn test_generate_mpc_filename_sharp_key() {
        let file = MidiFileWithMetadata {
            id: 3,
            filepath: "/collections/Bass Lines & Leads/F# Minor Bass.mid".to_string(),
            filename: "F# Minor Bass.mid".to_string(),
            duration_seconds: Some(16.0),
            bpm: Some(140.0),
            key_signature: Some("F#m".to_string()),
            time_signature_numerator: Some(4),
            time_signature_denominator: Some(4),
            is_percussive: None,
            has_chords: None,
            has_melody: None,
            is_monophonic: None,
            polyphony_avg: None,
        };

        let mpc_name = generate_mpc_filename(&file).unwrap();

        assert!(mpc_name.contains("fs-")); // F# minor
        assert!(mpc_name.contains("140")); // BPM
        assert!(mpc_name.contains("BassLinesLeads")); // folder (sanitized)
    }
}

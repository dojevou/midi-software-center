use dashmap::DashMap;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Duplicate file information
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct FileInfo {
    path: PathBuf,
    size: u64,
    hash: String,
}

/// Statistics for duplicate detection
struct DuplicateStats {
    total_files: AtomicU64,
    total_bytes: AtomicU64,
    duplicate_files: AtomicU64,
    duplicate_bytes: AtomicU64,
    unique_files: AtomicU64,
}

impl DuplicateStats {
    fn new() -> Self {
        Self {
            total_files: AtomicU64::new(0),
            total_bytes: AtomicU64::new(0),
            duplicate_files: AtomicU64::new(0),
            duplicate_bytes: AtomicU64::new(0),
            unique_files: AtomicU64::new(0),
        }
    }
}

/// Calculate BLAKE3 hash of a file
fn hash_file(path: &Path) -> Result<String, std::io::Error> {
    let mut file = fs::File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0; 65536]; // 64KB buffer

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

/// Find all MIDI files recursively
fn find_midi_files(root: &Path) -> Vec<PathBuf> {
    println!("Scanning for MIDI files...");
    let start = Instant::now();

    let files: Vec<PathBuf> = walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| {
                    let ext_lower = ext.to_lowercase();
                    ext_lower == "mid" || ext_lower == "midi"
                })
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    println!("Found {} MIDI files in {:?}", files.len(), start.elapsed());
    files
}

/// Process files and detect duplicates
fn detect_duplicates(files: Vec<PathBuf>) -> (HashMap<String, Vec<FileInfo>>, Arc<DuplicateStats>) {
    println!("\nHashing {} files...", files.len());
    let start = Instant::now();

    let file_map: DashMap<String, Vec<FileInfo>> = DashMap::new();
    let stats = Arc::new(DuplicateStats::new());
    let processed = AtomicU64::new(0);

    // Process files in parallel
    files.par_iter().for_each(|path| {
        // Get file size
        let size = match fs::metadata(path) {
            Ok(metadata) => metadata.len(),
            Err(_) => return,
        };

        // Calculate hash
        let hash = match hash_file(path) {
            Ok(h) => h,
            Err(_) => return,
        };

        // Create file info
        let info = FileInfo { path: path.clone(), size, hash: hash.clone() };

        // Add to map
        file_map.entry(hash).or_default().push(info);

        // Update stats
        stats.total_files.fetch_add(1, Ordering::Relaxed);
        stats.total_bytes.fetch_add(size, Ordering::Relaxed);

        // Progress reporting
        let count = processed.fetch_add(1, Ordering::Relaxed) + 1;
        if count.is_multiple_of(10000) {
            let elapsed = start.elapsed();
            let rate = count as f64 / elapsed.as_secs_f64();
            println!("Processed: {}   files ({:.0} files/sec)", count, rate);
        }
    });

    let elapsed = start.elapsed();
    let total = stats.total_files.load(Ordering::Relaxed);
    let rate = total as f64 / elapsed.as_secs_f64();
    println!(
        "\nHashing complete: {}   files in {:?} ({:.0} files/sec)",
        total, elapsed, rate
    );

    // Convert to regular HashMap
    let mut result = HashMap::new();
    for entry in file_map.into_iter() {
        result.insert(entry.0, entry.1);
    }

    (result, stats)
}

/// Analyze duplicates and update stats
fn analyze_duplicates(
    file_map: &HashMap<String, Vec<FileInfo>>,
    stats: &DuplicateStats,
) -> Vec<(String, Vec<FileInfo>)> {
    let mut duplicates = Vec::new();

    for (hash, files) in file_map {
        if files.len() > 1 {
            // This is a duplicate
            stats.unique_files.fetch_add(1, Ordering::Relaxed);
            let dup_count = files.len() as u64 - 1;
            stats.duplicate_files.fetch_add(dup_count, Ordering::Relaxed);

            // Calculate duplicate bytes (all except one copy)
            let file_size = files[0].size;
            stats.duplicate_bytes.fetch_add(file_size * dup_count, Ordering::Relaxed);

            duplicates.push((hash.clone(), files.clone()));
        } else {
            // Unique file
            stats.unique_files.fetch_add(1, Ordering::Relaxed);
        }
    }

    // Sort by number of duplicates (most duplicates first)
    duplicates.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    duplicates
}

/// Format number with thousand separators
#[allow(dead_code)]
fn format_number(n: u64) -> String {
    n.to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(",")
}

/// Generate duplicate report
fn generate_report(
    duplicates: &[(String, Vec<FileInfo>)],
    stats: &DuplicateStats,
    output_path: &Path,
) -> std::io::Result<()> {
    let mut output = String::new();

    // Header
    output.push_str("# MIDI Duplicate Files Report\n\n");

    // Statistics
    let total_files = stats.total_files.load(Ordering::Relaxed);
    let total_bytes = stats.total_bytes.load(Ordering::Relaxed);
    let unique_files = stats.unique_files.load(Ordering::Relaxed);
    let duplicate_files = stats.duplicate_files.load(Ordering::Relaxed);
    let duplicate_bytes = stats.duplicate_bytes.load(Ordering::Relaxed);

    output.push_str("## Summary Statistics\n\n");
    output.push_str(&format!("- **Total files scanned:** {}  \n", total_files));
    output.push_str(&format!(
        "- **Total size:** {:.2} GB\n",
        total_bytes as f64 / 1_073_741_824.0
    ));
    output.push_str(&format!("- **Unique files:** {}  \n", unique_files));
    output.push_str(&format!(
        "- **Duplicate files:** {}   ({:.1}%)\n",
        duplicate_files,
        duplicate_files as f64 / total_files as f64 * 100.0
    ));
    output.push_str(&format!(
        "- **Space wasted by duplicates:** {:.2} GB\n",
        duplicate_bytes as f64 / 1_073_741_824.0
    ));
    output.push_str(&format!(
        "- **Duplicate groups:** {}  \n\n",
        duplicates.len()
    ));

    // Duplicate groups
    output.push_str("## Duplicate Groups\n\n");
    output.push_str("Each group shows files with identical content. The **first file** in each group will be **KEPT**, all others will be **DELETED**.\n\n");

    for (i, (hash, files)) in duplicates.iter().enumerate() {
        output.push_str(&format!(
            "### Group {} ({} duplicates)\n\n",
            i + 1,
            files.len() - 1
        ));
        output.push_str(&format!("**Hash:** `{}`\n", &hash[..16]));
        output.push_str(&format!(
            "**Size:** {:.2} KB\n\n",
            files[0].size as f64 / 1024.0
        ));

        output.push_str("| Status | Path |\n");
        output.push_str("|--------|------|\n");

        for (j, file) in files.iter().enumerate() {
            let status = if j == 0 { "✅ KEEP" } else { "❌ DELETE" };
            output.push_str(&format!("| {} | `{}` |\n", status, file.path.display()));
        }

        output.push('\n');

        // Only show first 100 groups in report (can be huge)
        if i >= 99 {
            output.push_str(&format!(
                "\n*Note: Only showing first 100 groups. Total groups: {}*\n\n",
                duplicates.len()
            ));
            break;
        }
    }

    // Write deletion script
    output.push_str("\n---\n\n");
    output.push_str("## Deletion Summary\n\n");
    output.push_str(&format!(
        "Total files to delete: **{}  **\n",
        duplicate_files
    ));
    output.push_str(&format!(
        "Space to recover: **{:.2} GB**\n\n",
        duplicate_bytes as f64 / 1_073_741_824.0
    ));

    fs::write(output_path, output)?;

    // Also create a deletion list (just paths to delete)
    let delete_list_path = output_path.with_extension("delete.txt");
    let mut delete_list = String::new();

    for (_hash, files) in duplicates {
        // Skip first file (keep it), list all others for deletion
        for file in files.iter().skip(1) {
            delete_list.push_str(&format!("{}\n", file.path.display()));
        }
    }

    fs::write(&delete_list_path, delete_list)?;
    println!("\nDeletion list saved to: {}", delete_list_path.display());

    Ok(())
}

/// Execute deletion
fn execute_deletion(delete_list_path: &Path) -> std::io::Result<u64> {
    println!("\n⚠️  EXECUTING DELETION...");

    let content = fs::read_to_string(delete_list_path)?;
    let files: Vec<&str> = content.lines().collect();

    let deleted = AtomicU64::new(0);
    let failed = AtomicU64::new(0);

    files.par_iter().for_each(|path| {
        match fs::remove_file(path) {
            Ok(_) => {
                deleted.fetch_add(1, Ordering::Relaxed);
            },
            Err(e) => {
                eprintln!("Failed to delete {}: {}", path, e);
                failed.fetch_add(1, Ordering::Relaxed);
            },
        }

        let count = deleted.load(Ordering::Relaxed);
        if count.is_multiple_of(1000) {
            println!("Deleted: {}   files", count);
        }
    });

    let total_deleted = deleted.load(Ordering::Relaxed);
    let total_failed = failed.load(Ordering::Relaxed);

    println!("\n✅ Deletion complete!");
    println!("Files deleted: {}  ", total_deleted);
    if total_failed > 0 {
        println!("⚠️  Failed to delete: {}  ", total_failed);
    }

    Ok(total_deleted)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path> [--delete]", args[0]);
        eprintln!("\nOptions:");
        eprintln!("  <path>      Path to MIDI directory");
        eprintln!("  --delete    Execute deletion (use with caution!)");
        eprintln!("\nExample:");
        eprintln!("  {} /path/to/midi/files", args[0]);
        eprintln!("  {} /path/to/midi/files --delete", args[0]);
        std::process::exit(1);
    }

    let root_path = PathBuf::from(&args[1]);
    let delete_mode = args.len() > 2 && args[2] == "--delete";

    if !root_path.exists() {
        eprintln!("Error: Path does not exist: {}", root_path.display());
        std::process::exit(1);
    }

    println!("MIDI Duplicate Finder");
    println!("=====================");
    println!("Root path: {}", root_path.display());
    if delete_mode {
        println!("⚠️  DELETE MODE ENABLED - Files will be deleted!");
    } else {
        println!("Mode: Analysis only (no deletion)");
    }
    println!();

    // Find all MIDI files
    let files = find_midi_files(&root_path);

    if files.is_empty() {
        println!("No MIDI files found!");
        std::process::exit(0);
    }

    // Detect duplicates
    let (file_map, stats) = detect_duplicates(files);

    // Analyze duplicates
    let duplicates = analyze_duplicates(&file_map, &stats);

    if duplicates.is_empty() {
        println!("\n✅ No duplicates found! All files are unique.");
        std::process::exit(0);
    }

    // Generate report
    let report_path = PathBuf::from("DUPLICATE_REPORT.md");
    println!("\nGenerating report...");

    match generate_report(&duplicates, &stats, &report_path) {
        Ok(_) => println!("Report saved to: {}", report_path.display()),
        Err(e) => {
            eprintln!("Error generating report: {}", e);
            std::process::exit(1);
        },
    }

    // Execute deletion if requested
    if delete_mode {
        println!("\n⚠️  ⚠️  ⚠️  WARNING ⚠️  ⚠️  ⚠️");
        println!(
            "You are about to DELETE {}   files!",
            stats.duplicate_files.load(Ordering::Relaxed)
        );
        println!(
            "This will free {:.2} GB of space.",
            stats.duplicate_bytes.load(Ordering::Relaxed) as f64 / 1_073_741_824.0
        );
        println!("\nPress Ctrl+C within 10 seconds to cancel...");

        std::thread::sleep(std::time::Duration::from_secs(10));

        let delete_list_path = report_path.with_extension("delete.txt");
        match execute_deletion(&delete_list_path) {
            Ok(deleted) => println!("\n✅ Successfully deleted {}   duplicate files!", deleted),
            Err(e) => eprintln!("\n❌ Error during deletion: {}", e),
        }
    } else {
        println!("\n📋 Review the report and deletion list.");
        println!("To execute deletion, run:");
        println!("  {} {} --delete", args[0], root_path.display());
    }
}

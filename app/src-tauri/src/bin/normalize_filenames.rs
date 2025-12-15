// 🧹 Ultra-Fast MIDI Filename Normalization
// Parallel processing with Rayon for millions of files
// - Normalizes extensions: .MIDI, .MID → .mid
// - Replaces spaces with underscores
// - Fixes UTF-8 encoding issues

use midi_app::core::naming::sanitizer::sanitize_strict;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use walkdir::WalkDir;

#[derive(Debug, Default)]
struct NormalizationStats {
    total_files: AtomicUsize,
    extensions_fixed: AtomicUsize,
    spaces_fixed: AtomicUsize,
    encoding_fixed: AtomicUsize,
    errors: AtomicUsize,
}

impl NormalizationStats {
    fn print_progress(&self, elapsed: f64) {
        let total = self.total_files.load(Ordering::Relaxed);
        let ext = self.extensions_fixed.load(Ordering::Relaxed);
        let spaces = self.spaces_fixed.load(Ordering::Relaxed);
        let encoding = self.encoding_fixed.load(Ordering::Relaxed);
        let errors = self.errors.load(Ordering::Relaxed);
        let rate = total as f64 / elapsed;

        println!("\n📊 Progress:");
        println!("   Files processed:   {}", total);
        println!("   Extensions fixed:  {}", ext);
        println!("   Spaces fixed:      {}", spaces);
        println!("   Encoding fixed:    {}", encoding);
        println!("   Errors:            {}", errors);
        println!("   Speed:             {:.0} files/sec", rate);
        println!("   Elapsed:           {:.2}s", elapsed);
    }
}

/// Normalize a single MIDI file using strict sanitization
fn normalize_file(
    path: &Path,
    stats: &NormalizationStats,
) -> Result<(), Box<dyn std::error::Error>> {
    let parent = path.parent().ok_or("No parent directory")?;
    let filename = path.file_name().ok_or("No filename")?.to_string_lossy();

    // Apply strict sanitization: only a-zA-Z0-9_-.  no consecutive special chars
    let new_filename = sanitize_strict(&filename);
    let changed = new_filename != filename.as_ref();

    // Track what changed
    if changed {
        // Check extension change
        if filename.to_lowercase().ends_with(".midi")
            || (filename.to_lowercase().ends_with(".mid") && !filename.ends_with(".mid"))
        {
            stats.extensions_fixed.fetch_add(1, Ordering::Relaxed);
        }

        // Check space replacement
        if filename.contains(' ') {
            stats.spaces_fixed.fetch_add(1, Ordering::Relaxed);
        }

        // Check special char removal
        if filename
            .chars()
            .any(|c| !c.is_alphanumeric() && c != '_' && c != '-' && c != '.')
        {
            stats.encoding_fixed.fetch_add(1, Ordering::Relaxed);
        }
    }

    // 4. Rename if changed
    if changed {
        let new_path = parent.join(&new_filename);

        // Handle duplicate names by appending counter
        let mut final_path = new_path.clone();
        let mut counter = 1;
        while final_path.exists() && final_path != path {
            let stem = Path::new(&new_filename).file_stem().unwrap().to_string_lossy();
            let ext = Path::new(&new_filename)
                .extension()
                .map(|e| e.to_string_lossy())
                .unwrap_or_default();
            final_path = parent.join(format!("{}_{}.{}", stem, counter, ext));
            counter += 1;
        }

        if final_path != path {
            fs::rename(path, &final_path)?;
        }
    }

    stats.total_files.fetch_add(1, Ordering::Relaxed);
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <directory> [workers]", args[0]);
        eprintln!("Example: {} /home/dojevou/tmp 32", args[0]);
        std::process::exit(1);
    }

    let dir = &args[1];
    let workers = if args.len() > 2 {
        args[2].parse::<usize>().unwrap_or_else(|_| {
            eprintln!("Invalid workers count, using default");
            num_cpus::get()
        })
    } else {
        num_cpus::get()
    };

    println!("🧹 ULTRA-FAST MIDI FILENAME NORMALIZATION");
    println!("═══════════════════════════════════════════════════════════════");
    println!();
    println!("📂 Target directory: {}", dir);
    println!("⚡ Parallel workers: {}", workers);
    println!();

    // Configure Rayon thread pool
    rayon::ThreadPoolBuilder::new().num_threads(workers).build_global().unwrap();

    let start = Instant::now();
    let stats = NormalizationStats::default();

    // Collect all MIDI files first (fast with walkdir)
    println!("📊 Scanning for MIDI files...");
    let scan_start = Instant::now();

    let files: Vec<PathBuf> = WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_lowercase();
            name.ends_with(".mid") || name.ends_with(".midi")
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    let scan_elapsed = scan_start.elapsed().as_secs_f64();
    println!(
        "✓ Found {} MIDI files in {:.2}s ({:.0} files/sec)",
        files.len(),
        scan_elapsed,
        files.len() as f64 / scan_elapsed
    );
    println!();

    // Process files in parallel
    println!("⚡ Normalizing filenames...");
    let _process_start = Instant::now();

    files.par_iter().for_each(|path| {
        if let Err(e) = normalize_file(path, &stats) {
            eprintln!("Error processing {:?}: {}", path, e);
            stats.errors.fetch_add(1, Ordering::Relaxed);
        }
    });

    println!("✓ Processing complete");
    println!();

    // Final statistics
    let total_elapsed = start.elapsed().as_secs_f64();
    println!("═══════════════════════════════════════════════════════════════");
    println!("✅ NORMALIZATION COMPLETE!");
    stats.print_progress(total_elapsed);
    println!();
    println!("All .mid files now have:");
    println!("  ✓ Lowercase .mid extension");
    println!("  ✓ Underscores instead of spaces");
    println!("  ✓ Clean UTF-8 encoding");
    println!();
}

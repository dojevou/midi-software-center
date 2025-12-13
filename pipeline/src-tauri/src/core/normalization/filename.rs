/// Ultra-Fast MIDI Filename Normalization
///
/// # Archetype: Trusty Module
/// - Pure functions for filename normalization
/// - Thread-safe file renaming operations
/// - Returns Result types for error handling
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use walkdir::WalkDir;

#[derive(Debug, Default)]
pub struct NormalizationStats {
    pub total_files: AtomicUsize,
    pub extensions_fixed: AtomicUsize,
    pub spaces_fixed: AtomicUsize,
    pub encoding_fixed: AtomicUsize,
    pub errors: AtomicUsize,
}

impl NormalizationStats {
    pub fn print_summary(&self, elapsed: f64) {
        let total = self.total_files.load(Ordering::Relaxed);
        let ext = self.extensions_fixed.load(Ordering::Relaxed);
        let spaces = self.spaces_fixed.load(Ordering::Relaxed);
        let encoding = self.encoding_fixed.load(Ordering::Relaxed);
        let errors = self.errors.load(Ordering::Relaxed);
        let rate = if elapsed > 0.0 {
            total as f64 / elapsed
        } else {
            0.0
        };

        println!("   Files normalized:   {}", total);
        println!("   Extensions fixed:   {}", ext);
        println!("   Spaces fixed:       {}", spaces);
        println!("   Encoding fixed:     {}", encoding);
        println!("   Errors:             {}", errors);
        println!("   Speed:              {:.0} files/sec", rate);
    }
}

/// Sanitize filename to MPC-compatible characters only
///
/// MPC ONE/Live/X allowed characters:
/// - Letters: A-Z, a-z
/// - Numbers: 0-9
/// - Hyphens: -
/// - Underscores: _
/// - Periods: . (for extension only)
///
/// All other characters (parentheses, brackets, symbols, etc.) are replaced with underscores
fn sanitize_mpc_compatible(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Normalize a single MIDI file on disk
fn normalize_file(
    path: &Path,
    stats: &NormalizationStats,
) -> Result<(), Box<dyn std::error::Error>> {
    let parent = path.parent().ok_or("No parent directory")?;
    let filename = path.file_name().ok_or("No filename")?.to_string_lossy();

    let mut new_filename = filename.to_string();
    let mut changed = false;

    // 1. Normalize extension
    let lowercase = filename.to_lowercase();
    if let Some(base) = filename.strip_suffix(".MIDI") {
        new_filename = format!("{base}.mid");
        changed = true;
        stats.extensions_fixed.fetch_add(1, Ordering::Relaxed);
    } else if let Some(base) = filename.strip_suffix(".MID") {
        if !lowercase.ends_with(".mid") {
            new_filename = format!("{base}.mid");
            changed = true;
            stats.extensions_fixed.fetch_add(1, Ordering::Relaxed);
        }
    } else if filename.ends_with(".midi") && filename != lowercase {
        new_filename = filename[..filename.len() - 5].to_string() + ".mid";
        changed = true;
        stats.extensions_fixed.fetch_add(1, Ordering::Relaxed);
    } else if filename.ends_with(".MiD") || filename.ends_with(".Midi") {
        let stem_len = if filename.ends_with(".MiD") { 4 } else { 5 };
        new_filename = filename[..filename.len() - stem_len].to_string() + ".mid";
        changed = true;
        stats.extensions_fixed.fetch_add(1, Ordering::Relaxed);
    }

    // 2. Replace spaces with underscores
    if new_filename.contains(' ') {
        new_filename = new_filename.replace(' ', "_");
        changed = true;
        stats.spaces_fixed.fetch_add(1, Ordering::Relaxed);
    }

    // 3. Remove all non-MPC-compatible characters (parentheses, symbols, etc.)
    // MPC allows only: A-Z, a-z, 0-9, -, _, .
    let sanitized = sanitize_mpc_compatible(&new_filename);
    if sanitized != new_filename {
        new_filename = sanitized;
        changed = true;
        stats.encoding_fixed.fetch_add(1, Ordering::Relaxed);
    }

    // 4. Rename file if changes were made
    if changed {
        let mut final_path = parent.join(&new_filename);

        // Handle filename collisions
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

/// Normalize all MIDI files in a directory (recursive)
///
/// # Arguments
/// * `dir` - Directory to normalize
/// * `workers` - Number of parallel workers (0 = auto-detect CPU cores)
///
/// # Returns
/// * `NormalizationStats` - Statistics about the normalization
pub fn normalize_directory(
    dir: &Path,
    workers: usize,
) -> Result<NormalizationStats, Box<dyn std::error::Error>> {
    // Configure Rayon thread pool
    if workers > 0 {
        rayon::ThreadPoolBuilder::new().num_threads(workers).build_global().ok();
    }

    let stats = NormalizationStats::default();

    // Collect all MIDI files
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

    // Process files in parallel
    files.par_iter().for_each(|path| {
        if let Err(e) = normalize_file(path, &stats) {
            eprintln!("  ⚠️  Error normalizing {:?}: {}", path, e);
            stats.errors.fetch_add(1, Ordering::Relaxed);
        }
    });

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_mpc_compatible_removes_parentheses() {
        let input = "(8455)_Triangles.mid";
        let output = sanitize_mpc_compatible(input);
        assert_eq!(output, "_8455__Triangles.mid");
    }

    #[test]
    fn test_sanitize_mpc_compatible_preserves_valid() {
        let input = "valid_filename-123.mid";
        let output = sanitize_mpc_compatible(input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_sanitize_mpc_compatible_removes_symbols() {
        let input = "file@name#test$123.mid";
        let output = sanitize_mpc_compatible(input);
        assert_eq!(output, "file_name_test_123.mid");
    }
}

// 🏥 MIDI Doctor - Repair and Validate MIDI Files
// Identifies corrupt, fixable, and valid MIDI files
// Attempts automatic repairs for common issues

use midi_library_shared::core::midi::parser::parse_midi_file;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use walkdir::WalkDir;

#[derive(Debug, Default)]
struct DiagnosticStats {
    total_files: AtomicUsize,
    valid_files: AtomicUsize,
    repaired_files: AtomicUsize,
    corrupt_files: AtomicUsize,
    not_midi: AtomicUsize,
}

#[derive(Debug)]
enum MidiStatus {
    Valid,
    Repaired(String), // Fixed issue description
    Corrupt(String),  // Corruption type
    NotMidi,          // Not a MIDI file
}

impl DiagnosticStats {
    fn print_progress(&self, elapsed: f64) {
        let total = self.total_files.load(Ordering::Relaxed);
        let valid = self.valid_files.load(Ordering::Relaxed);
        let repaired = self.repaired_files.load(Ordering::Relaxed);
        let corrupt = self.corrupt_files.load(Ordering::Relaxed);
        let not_midi = self.not_midi.load(Ordering::Relaxed);
        let rate = total as f64 / elapsed;

        println!("\n📊 Diagnostic Report:");
        println!("   Files scanned:     {}", total);
        println!(
            "   ✅ Valid:          {} ({:.1}%)",
            valid,
            valid as f64 / total as f64 * 100.0
        );
        println!(
            "   🔧 Repaired:       {} ({:.1}%)",
            repaired,
            repaired as f64 / total as f64 * 100.0
        );
        println!(
            "   ❌ Corrupt:        {} ({:.1}%)",
            corrupt,
            corrupt as f64 / total as f64 * 100.0
        );
        println!(
            "   ⚠️  Not MIDI:      {} ({:.1}%)",
            not_midi,
            not_midi as f64 / total as f64 * 100.0
        );
        println!("   Speed:             {:.0} files/sec", rate);
        println!("   Elapsed:           {:.2}s", elapsed);
    }
}

/// Attempt to repair common MIDI file issues
fn attempt_repair(data: &[u8]) -> Result<(Vec<u8>, String), String> {
    let mut repaired = data.to_vec();
    let mut fixes = Vec::new();

    // Fix 1: Add missing End-of-Track marker (FF 2F 00)
    // This is the most common issue
    if repaired.len() >= 14 {
        // Check if file has proper header
        if &repaired[0..4] == b"MThd" {
            // Look for track chunks
            let mut pos = 14; // After header
            while pos < repaired.len() {
                if pos + 8 > repaired.len() {
                    break;
                }

                if &repaired[pos..pos + 4] == b"MTrk" {
                    let track_len = u32::from_be_bytes([
                        repaired[pos + 4],
                        repaired[pos + 5],
                        repaired[pos + 6],
                        repaired[pos + 7],
                    ]) as usize;

                    let track_end = pos + 8 + track_len;
                    if track_end <= repaired.len() {
                        // Check if track ends with End-of-Track (FF 2F 00)
                        let has_eot = if track_end >= 3 {
                            repaired[track_end - 3..track_end] == [0xFF, 0x2F, 0x00]
                        } else {
                            false
                        };

                        if !has_eot && track_end < repaired.len() {
                            // Insert End-of-Track at proper position
                            repaired
                                .splice(track_end..track_end, [0xFF, 0x2F, 0x00].iter().cloned());

                            // Update track length in header
                            let new_len = track_len + 3;
                            let len_bytes = new_len.to_be_bytes();
                            repaired[pos + 4] = len_bytes[0];
                            repaired[pos + 5] = len_bytes[1];
                            repaired[pos + 6] = len_bytes[2];
                            repaired[pos + 7] = len_bytes[3];

                            fixes.push("Added missing End-of-Track marker".to_string());
                        }
                        pos = track_end;
                    } else {
                        break;
                    }
                } else {
                    pos += 1;
                }
            }
        }
    }

    // Fix 2: Trim trailing garbage data
    if repaired.len() > 14 && &repaired[0..4] == b"MThd" {
        let header_len =
            u32::from_be_bytes([repaired[4], repaired[5], repaired[6], repaired[7]]) as usize;
        if header_len == 6 {
            let num_tracks = u16::from_be_bytes([repaired[10], repaired[11]]) as usize;

            // Calculate expected file size
            let mut expected_size = 14; // Header
            let mut pos = 14;

            for _ in 0..num_tracks {
                if pos + 8 > repaired.len() {
                    break;
                }
                if &repaired[pos..pos + 4] == b"MTrk" {
                    let track_len = u32::from_be_bytes([
                        repaired[pos + 4],
                        repaired[pos + 5],
                        repaired[pos + 6],
                        repaired[pos + 7],
                    ]) as usize;
                    expected_size = pos + 8 + track_len;
                    pos = expected_size;
                } else {
                    break;
                }
            }

            if expected_size < repaired.len() {
                let trimmed = repaired.len() - expected_size;
                repaired.truncate(expected_size);
                fixes.push(format!("Trimmed {} bytes of trailing garbage", trimmed));
            }
        }
    }

    if fixes.is_empty() {
        Err("No repairs needed or possible".to_string())
    } else {
        Ok((repaired, fixes.join(", ")))
    }
}

/// Diagnose a MIDI file and attempt repair if needed
fn diagnose_midi_file(path: &Path, repair_dir: Option<&Path>) -> MidiStatus {
    // Read file
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(_) => return MidiStatus::Corrupt("Cannot read file".to_string()),
    };

    // Check minimum size
    if data.len() < 14 {
        return MidiStatus::Corrupt(format!("File too small ({} bytes, need 14+)", data.len()));
    }

    // Check if it's a MIDI file
    if &data[0..4] != b"MThd" {
        return MidiStatus::NotMidi;
    }

    // Try to parse as-is
    match parse_midi_file(&data) {
        Ok(_) => MidiStatus::Valid,
        Err(e) => {
            // Try to repair
            match attempt_repair(&data) {
                Ok((repaired_data, fix_description)) => {
                    // Verify repair worked
                    match parse_midi_file(&repaired_data) {
                        Ok(_) => {
                            // Save repaired file if output directory provided
                            if let Some(repair_dir) = repair_dir {
                                let filename = path.file_name().unwrap();
                                let output_path = repair_dir.join(filename);
                                if let Ok(()) = fs::write(&output_path, &repaired_data) {
                                    MidiStatus::Repaired(format!(
                                        "{} (saved to repair dir)",
                                        fix_description
                                    ))
                                } else {
                                    MidiStatus::Repaired(format!(
                                        "{} (could not save)",
                                        fix_description
                                    ))
                                }
                            } else {
                                MidiStatus::Repaired(fix_description)
                            }
                        },
                        Err(e2) => {
                            MidiStatus::Corrupt(format!("Repair failed: {} (original: {})", e2, e))
                        },
                    }
                },
                Err(_) => MidiStatus::Corrupt(format!("{}", e)),
            }
        },
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!(
            "Usage: {} <directory> [repair-output-dir] [workers]",
            args[0]
        );
        eprintln!("Example: {} /path/to/midi /path/to/repaired 64", args[0]);
        eprintln!();
        eprintln!("If repair-output-dir is provided, repaired files will be saved there.");
        std::process::exit(1);
    }

    let scan_dir = &args[1];
    let repair_dir = if args.len() > 2 && args[2].parse::<usize>().is_err() {
        Some(args[2].as_str())
    } else {
        None
    };

    let workers = if args.len() > 3 {
        args[3].parse::<usize>().unwrap_or_else(|_| num_cpus::get())
    } else if args.len() > 2 && args[2].parse::<usize>().is_ok() {
        args[2].parse::<usize>().unwrap()
    } else {
        num_cpus::get()
    };

    println!("🏥 MIDI DOCTOR - Diagnostic and Repair Tool");
    println!("═══════════════════════════════════════════════════════════════");
    println!();
    println!("📂 Scan directory: {}", scan_dir);
    if let Some(repair_dir) = repair_dir {
        println!("🔧 Repair output:  {}", repair_dir);
        // Create repair directory
        if let Err(e) = fs::create_dir_all(repair_dir) {
            eprintln!("❌ Failed to create repair directory: {}", e);
            std::process::exit(1);
        }
    } else {
        println!("🔧 Repair output:  Disabled (diagnostics only)");
    }
    println!("⚡ Parallel workers: {}", workers);
    println!();

    // Configure Rayon
    rayon::ThreadPoolBuilder::new().num_threads(workers).build_global().unwrap();

    let start = Instant::now();
    let stats = DiagnosticStats::default();

    // Collect all MIDI-like files
    println!("📊 Scanning for MIDI files...");
    let scan_start = Instant::now();

    let files: Vec<PathBuf> = WalkDir::new(scan_dir)
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
    println!("✓ Found {} MIDI files in {:.2}s", files.len(), scan_elapsed);
    println!();

    // Diagnose files in parallel
    println!("🔍 Diagnosing MIDI files...");
    let repair_path = repair_dir.map(PathBuf::from);

    files.par_iter().for_each(|path| {
        let status = diagnose_midi_file(path, repair_path.as_deref());

        match &status {
            MidiStatus::Valid => {
                stats.valid_files.fetch_add(1, Ordering::Relaxed);
            },
            MidiStatus::Repaired(desc) => {
                stats.repaired_files.fetch_add(1, Ordering::Relaxed);
                println!("🔧 REPAIRED: {} - {}", path.display(), desc);
            },
            MidiStatus::Corrupt(reason) => {
                stats.corrupt_files.fetch_add(1, Ordering::Relaxed);
                println!("❌ CORRUPT:  {} - {}", path.display(), reason);
            },
            MidiStatus::NotMidi => {
                stats.not_midi.fetch_add(1, Ordering::Relaxed);
                println!("⚠️  NOT MIDI: {}", path.display());
            },
        }

        stats.total_files.fetch_add(1, Ordering::Relaxed);

        // Progress indicator
        let total = stats.total_files.load(Ordering::Relaxed);
        if total % 10000 == 0 {
            println!("  Progress: {} files scanned...", total);
        }
    });

    println!("✓ Diagnosis complete");
    println!();

    // Final statistics
    let total_elapsed = start.elapsed().as_secs_f64();
    println!("═══════════════════════════════════════════════════════════════");
    println!("✅ MIDI DOCTOR COMPLETE!");
    stats.print_progress(total_elapsed);
    println!();

    let repaired_count = stats.repaired_files.load(Ordering::Relaxed);
    let corrupt_count = stats.corrupt_files.load(Ordering::Relaxed);

    if repaired_count > 0 && repair_dir.is_some() {
        println!("🔧 Repaired files saved to: {}", repair_dir.unwrap());
    }

    if corrupt_count > 0 {
        println!(
            "❌ {} files are truly corrupt and cannot be automatically repaired",
            corrupt_count
        );
        println!("   These files may need manual inspection or re-downloading");
    }

    println!();
}

use anyhow::{Context, Result};
use midi_library_shared::core::analysis::detect_bpm;
use midi_library_shared::core::midi::{parse_midi_file, Event, MidiFile};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;
use std::time::Instant;

// Helper function to count notes in a MIDI file
fn count_notes(midi_file: &MidiFile) -> usize {
    midi_file
        .tracks
        .iter()
        .flat_map(|track| &track.events)
        .filter(|event| matches!(event.event, Event::NoteOn { .. }))
        .count()
}

// Helper function to get total duration in ticks
fn get_duration_ticks(midi_file: &MidiFile) -> u32 {
    midi_file
        .tracks
        .iter()
        .flat_map(|track| &track.events)
        .map(|event| event.delta_ticks)
        .sum()
}

#[derive(Debug, Serialize, Deserialize)]
struct TestResult {
    file_path: String,
    file_name: String,
    success: bool,
    error: Option<String>,
    file_size: u64,
    parse_time_ms: u128,
    bpm: Option<f64>,
    key: Option<String>,
    duration_ms: Option<u64>,
    track_count: Option<usize>,
    note_count: Option<usize>,
}

fn analyze_file(file_path: &Path) -> Result<TestResult> {
    let start = Instant::now();
    let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string();

    println!("\n📄 Analyzing: {}", file_name);

    // Read file
    let data = fs::read(file_path).context("Failed to read file")?;
    let file_size = data.len() as u64;
    println!("  📦 Size: {} bytes", file_size);

    // Parse MIDI
    let midi_file = parse_midi_file(&data).context("Failed to parse MIDI")?;
    println!("  ✅ Parse successful");

    // Get basic info
    let track_count = midi_file.tracks.len();
    let note_count = count_notes(&midi_file);
    let duration_ticks = get_duration_ticks(&midi_file);

    println!("  🎵 Tracks: {}", track_count);
    println!("  🎹 Notes: {}", note_count);

    // Detect BPM
    let bpm_result = detect_bpm(&midi_file);
    let bpm = if bpm_result.confidence > 0.5 {
        Some(bpm_result.bpm)
    } else {
        None
    };

    if let Some(bpm_value) = bpm {
        println!(
            "  ⏱️  BPM: {:.1} (confidence: {:.1}%)",
            bpm_value,
            bpm_result.confidence * 100.0
        );
    } else {
        println!("  ⚠️  BPM: Not detected");
    }

    // Detect key using the shared library implementation
    use midi_library_shared::core::analysis::key_detector::detect_key;

    let key = detect_key(&midi_file);
    if let Some(ref key_str) = key {
        println!("  🎹 Key: {}", key_str);
    } else {
        println!("  ⚠️  Key: Unable to detect (low confidence or insufficient notes)");
    }

    let parse_time = start.elapsed();
    println!("  ⏱️  Processing time: {}ms", parse_time.as_millis());

    Ok(TestResult {
        file_path: file_path.to_string_lossy().to_string(),
        file_name,
        success: true,
        error: None,
        file_size,
        parse_time_ms: parse_time.as_millis(),
        bpm,
        key,
        duration_ms: Some((duration_ticks as u64 * 500) / 1000), // Rough estimate
        track_count: Some(track_count),
        note_count: Some(note_count),
    })
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <midi_file1> [midi_file2] ...", args[0]);
        eprintln!("       {} <directory>", args[0]);
        std::process::exit(1);
    }

    let mut results = Vec::new();
    let mut total_files = 0;
    let mut successful = 0;
    let mut failed = 0;
    let mut bpm_detected = 0;
    let mut key_detected = 0;
    let mut total_time_ms = 0u128;

    println!("🎵 MIDI Pipeline Real-World Testing");
    println!("====================================\n");

    // Process each file/directory argument
    for arg in &args[1..] {
        let path = Path::new(arg);

        let files: Vec<_> = if path.is_dir() {
            // Find all MIDI files in directory
            walkdir::WalkDir::new(path)
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
                .collect()
        } else {
            vec![path.to_path_buf()]
        };

        for file_path in files {
            total_files += 1;

            match analyze_file(&file_path) {
                Ok(result) => {
                    successful += 1;
                    total_time_ms += result.parse_time_ms;

                    if result.bpm.is_some() {
                        bpm_detected += 1;
                    }
                    if result.key.is_some() {
                        key_detected += 1;
                    }

                    results.push(result);
                },
                Err(e) => {
                    failed += 1;
                    println!("\n❌ Error: {}", e);

                    results.push(TestResult {
                        file_path: file_path.to_string_lossy().to_string(),
                        file_name: file_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        success: false,
                        error: Some(e.to_string()),
                        file_size: 0,
                        parse_time_ms: 0,
                        bpm: None,
                        key: None,
                        duration_ms: None,
                        track_count: None,
                        note_count: None,
                    });
                },
            }
        }
    }

    // Print summary
    println!("\n");
    println!("📊 SUMMARY");
    println!("====================================");
    println!("✅ Files tested: {}", total_files);
    println!(
        "✅ Successful parses: {} ({:.1}%)",
        successful,
        (successful as f64 / total_files as f64) * 100.0
    );
    println!(
        "❌ Failed parses: {} ({:.1}%)",
        failed,
        (failed as f64 / total_files as f64) * 100.0
    );
    println!(
        "⏱️  BPM detection rate: {}/{} ({:.1}%)",
        bpm_detected,
        total_files,
        (bpm_detected as f64 / total_files as f64) * 100.0
    );
    println!(
        "🎼 Key detection rate: {}/{} ({:.1}%)",
        key_detected,
        total_files,
        (key_detected as f64 / total_files as f64) * 100.0
    );

    if total_files > 0 {
        let avg_time = total_time_ms / total_files as u128;
        println!("⏱️  Average processing time: {}ms/file", avg_time);
        println!("⏱️  Total processing time: {}ms", total_time_ms);
    }

    println!("\n");

    // Production readiness assessment
    println!("🎯 PRODUCTION READINESS ASSESSMENT");
    println!("====================================");

    let success_rate = (successful as f64 / total_files as f64) * 100.0;
    if success_rate >= 95.0 {
        println!("✅ Parse success rate: EXCELLENT ({:.1}%)", success_rate);
    } else if success_rate >= 85.0 {
        println!("⚠️  Parse success rate: ACCEPTABLE ({:.1}%)", success_rate);
    } else {
        println!(
            "❌ Parse success rate: NEEDS IMPROVEMENT ({:.1}%)",
            success_rate
        );
    }

    if total_files > 0 {
        let avg_time = total_time_ms / total_files as u128;
        if avg_time <= 100 {
            println!("✅ Performance: EXCELLENT ({}ms avg)", avg_time);
        } else if avg_time <= 500 {
            println!("✅ Performance: GOOD ({}ms avg)", avg_time);
        } else {
            println!("⚠️  Performance: NEEDS OPTIMIZATION ({}ms avg)", avg_time);
        }
    }

    // Write JSON results
    let results_json = serde_json::to_string_pretty(&results)?;
    fs::write("/tmp/midi_test_results.json", results_json)?;
    println!("\n📄 Detailed results written to: /tmp/midi_test_results.json");

    Ok(())
}

use dashmap::DashMap;
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Common instrument keywords to extract from filenames
const INSTRUMENT_KEYWORDS: &[&str] = &[
    // Drums & Percussion
    "drum",
    "drums",
    "kick",
    "snare",
    "hihat",
    "hi-hat",
    "cymbal",
    "tom",
    "toms",
    "percussion",
    "conga",
    "bongo",
    "shaker",
    "tambourine",
    "clap",
    "cowbell",
    "ride",
    "crash",
    "splash",
    "china",
    "bell",
    "rim",
    "stick",
    // Bass
    "bass",
    "sub",
    "808",
    "909",
    "bassline",
    // Keys & Synths
    "piano",
    "keys",
    "keyboard",
    "synth",
    "pad",
    "lead",
    "arp",
    "pluck",
    "organ",
    "rhodes",
    "wurlitzer",
    "ep",
    "electric-piano",
    // Guitars
    "guitar",
    "gtr",
    "acoustic",
    "electric",
    "strum",
    "pick",
    // Strings
    "strings",
    "violin",
    "viola",
    "cello",
    "orchestra",
    "ensemble",
    // Brass
    "brass",
    "trumpet",
    "trombone",
    "horn",
    "sax",
    "saxophone",
    // Woodwinds
    "flute",
    "clarinet",
    "oboe",
    "bassoon",
    // Vocals
    "vocal",
    "vox",
    "voice",
    "choir",
    "chant",
    // FX
    "fx",
    "sfx",
    "riser",
    "sweep",
    "impact",
    "hit",
    // Melodic categories
    "melody",
    "melodic",
    "harmonic",
    "chord",
    "progression",
    // Loop types
    "loop",
    "one-shot",
    "fill",
    "break",
    "groove",
    "pattern",
    // Genres (can indicate instruments)
    "jazz",
    "rock",
    "funk",
    "soul",
    "r&b",
    "hip-hop",
    "trap",
    "edm",
    "house",
    "techno",
    "trance",
    "dubstep",
    "dnb",
    "reggae",
];

/// Extract instruments from filename
fn extract_instruments(filename: &str) -> Vec<String> {
    let filename_lower = filename.to_lowercase();
    let mut found = Vec::new();

    for &keyword in INSTRUMENT_KEYWORDS {
        if filename_lower.contains(keyword) {
            found.push(keyword.to_string());
        }
    }

    found
}

/// Find all MIDI files recursively
fn find_midi_files(root: &str) -> Vec<PathBuf> {
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

/// Process files and extract instruments
fn process_files(files: Vec<PathBuf>) -> DashMap<String, u64> {
    println!("\nExtracting instruments from {} files...", files.len());
    let start = Instant::now();

    let instrument_counts: DashMap<String, u64> = DashMap::new();
    let processed = AtomicU64::new(0);

    // Process files in parallel
    files.par_iter().for_each(|path| {
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            let instruments = extract_instruments(filename);

            for instrument in instruments {
                instrument_counts.entry(instrument).and_modify(|count| *count += 1).or_insert(1);
            }
        }

        // Progress reporting
        let count = processed.fetch_add(1, Ordering::Relaxed) + 1;
        if count.is_multiple_of(10000) {
            let elapsed = start.elapsed();
            let rate = count as f64 / elapsed.as_secs_f64();
            println!("Processed: {} files ({:.0} files/sec)", count, rate);
        }
    });

    let elapsed = start.elapsed();
    let total = processed.load(Ordering::Relaxed);
    let rate = total as f64 / elapsed.as_secs_f64();
    println!(
        "\nProcessing complete: {} files in {:?} ({:.0} files/sec)",
        total, elapsed, rate
    );

    instrument_counts
}

/// Generate report
fn generate_report(
    instrument_counts: DashMap<String, u64>,
    total_files: usize,
) -> std::io::Result<()> {
    println!("\nGenerating report...");

    // Convert to Vec and sort by count (descending)
    let mut instruments: Vec<(String, u64)> = instrument_counts.into_iter().collect();
    instruments.sort_by(|a, b| b.1.cmp(&a.1));

    // Generate markdown report
    let mut report = String::new();
    report.push_str("# MIDI Library Instrument Analysis\n\n");
    report.push_str(&format!("**Total Files Analyzed:** {}\n\n", total_files));
    report.push_str(&format!(
        "**Unique Instruments Found:** {}\n\n",
        instruments.len()
    ));

    report.push_str("## Instrument Frequency\n\n");
    report.push_str("| Rank | Instrument | Count | Percentage |\n");
    report.push_str("|------|------------|-------|------------|\n");

    for (idx, (instrument, count)) in instruments.iter().enumerate() {
        let percentage = (*count as f64 / total_files as f64) * 100.0;
        report.push_str(&format!(
            "| {} | {} | {} | {:.2}% |\n",
            idx + 1,
            instrument,
            count,
            percentage
        ));
    }

    // Write report
    let report_path = "INSTRUMENT_ANALYSIS.md";
    std::fs::write(report_path, &report)?;
    println!("Report saved to: {}", report_path);

    // Also create a simple list for organization
    let mut instrument_list = String::new();
    for (instrument, count) in &instruments {
        instrument_list.push_str(&format!("{}: {}\n", instrument, count));
    }

    let list_path = "INSTRUMENT_LIST.txt";
    std::fs::write(list_path, instrument_list)?;
    println!("Instrument list saved to: {}", list_path);

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <midi-library-path>", args[0]);
        eprintln!("\nExample:");
        eprintln!(
            "  {} ~/projects/midi-software-center/midi-library/",
            args[0]
        );
        std::process::exit(1);
    }

    let root_path = &args[1];

    println!("MIDI Instrument Extractor");
    println!("=========================");
    println!("Root path: {}", root_path);
    println!();

    // Find all MIDI files
    let files = find_midi_files(root_path);

    if files.is_empty() {
        println!("No MIDI files found!");
        std::process::exit(0);
    }

    let total_files = files.len();

    // Extract instruments
    let instrument_counts = process_files(files);

    // Generate report
    match generate_report(instrument_counts, total_files) {
        Ok(_) => println!("\n✅ Analysis complete!"),
        Err(e) => {
            eprintln!("\n❌ Error generating report: {}", e);
            std::process::exit(1);
        },
    }
}

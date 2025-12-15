use dashmap::DashMap;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Statistics collector for MIDI file analysis
#[derive(Debug)]
struct CollectionStats {
    // Instruments
    instruments: DashMap<String, u64>,

    // Genres
    genres: DashMap<String, u64>,

    // Pattern types
    patterns: DashMap<String, u64>,

    // Musical keys
    keys: DashMap<String, u64>,

    // BPM values
    bpms: DashMap<u32, u64>,

    // Time signatures
    time_signatures: DashMap<String, u64>,

    // Drum elements (for drum files)
    drum_elements: DashMap<String, u64>,

    // File counts
    total_files: AtomicU64,
    drum_files: AtomicU64,
    analyzed_files: AtomicU64,
}

impl CollectionStats {
    fn new() -> Self {
        Self {
            instruments: DashMap::new(),
            genres: DashMap::new(),
            patterns: DashMap::new(),
            keys: DashMap::new(),
            bpms: DashMap::new(),
            time_signatures: DashMap::new(),
            drum_elements: DashMap::new(),
            total_files: AtomicU64::new(0),
            drum_files: AtomicU64::new(0),
            analyzed_files: AtomicU64::new(0),
        }
    }

    fn increment(&self, map: &DashMap<String, u64>, key: &str) {
        map.entry(key.to_lowercase()).and_modify(|e| *e += 1).or_insert(1);
    }

    fn increment_bpm(&self, bpm: u32) {
        self.bpms.entry(bpm).and_modify(|e| *e += 1).or_insert(1);
    }
}

/// Extract metadata from filename
fn extract_metadata_from_filename(filename: &str) -> FileMetadata {
    let lower = filename.to_lowercase();
    let mut metadata = FileMetadata::default();

    // Extract BPM
    if let Some(bpm) = extract_bpm(&lower) {
        metadata.bpm = Some(bpm);
    }

    // Extract musical key
    if let Some(key) = extract_key(&lower) {
        metadata.key = Some(key);
    }

    // Extract time signature
    if let Some(ts) = extract_time_signature(&lower) {
        metadata.time_signature = Some(ts);
    }

    // Extract instruments
    metadata.instruments = extract_instruments(&lower);

    // Extract genres
    metadata.genres = extract_genres(&lower);

    // Extract pattern types
    metadata.patterns = extract_patterns(&lower);

    // Extract drum elements (if drum file)
    if metadata.instruments.iter().any(|i| i.contains("drum") || i == "percussion") {
        metadata.drum_elements = extract_drum_elements(&lower);
        metadata.is_drum = true;
    }

    metadata
}

#[derive(Default, Debug)]
struct FileMetadata {
    bpm: Option<u32>,
    key: Option<String>,
    time_signature: Option<String>,
    instruments: Vec<String>,
    genres: Vec<String>,
    patterns: Vec<String>,
    drum_elements: Vec<String>,
    is_drum: bool,
}

/// Extract BPM from filename
fn extract_bpm(filename: &str) -> Option<u32> {
    // Pattern 1: _120bpm_ or _120_bpm_
    if let Some(bpm) = extract_bpm_pattern1(filename) {
        return Some(bpm);
    }

    // Pattern 2: _120_ (number between underscores, 30-300 range)
    if let Some(bpm) = extract_bpm_pattern2(filename) {
        return Some(bpm);
    }

    // Pattern 3: 120.mid or 120-something.mid
    if let Some(bpm) = extract_bpm_pattern3(filename) {
        return Some(bpm);
    }

    None
}

fn extract_bpm_pattern1(filename: &str) -> Option<u32> {
    use regex::Regex;
    let re = Regex::new(r"[_\-](\d{2,3})[-_]?bpm[_\-]").ok()?;
    re.captures(filename)
        .and_then(|cap| cap.get(1))
        .and_then(|m| m.as_str().parse::<u32>().ok())
        .filter(|&bpm| (30..=300).contains(&bpm))
}

fn extract_bpm_pattern2(filename: &str) -> Option<u32> {
    use regex::Regex;
    let re = Regex::new(r"[_\-](\d{2,3})[_\-]").ok()?;
    re.captures(filename)
        .and_then(|cap| cap.get(1))
        .and_then(|m| m.as_str().parse::<u32>().ok())
        .filter(|&bpm| (30..=300).contains(&bpm))
}

fn extract_bpm_pattern3(filename: &str) -> Option<u32> {
    use regex::Regex;
    let re = Regex::new(r"^(\d{2,3})[-_\.]").ok()?;
    re.captures(filename)
        .and_then(|cap| cap.get(1))
        .and_then(|m| m.as_str().parse::<u32>().ok())
        .filter(|&bpm| (30..=300).contains(&bpm))
}

/// Extract musical key from filename
fn extract_key(filename: &str) -> Option<String> {
    use regex::Regex;

    // Major keys: C, C#/Db, D, etc.
    let major_re = Regex::new(r"[_\-]([A-G]#?b?)[_\-]").ok()?;
    if let Some(cap) = major_re.captures(filename) {
        return Some(cap.get(1)?.as_str().to_string());
    }

    // Minor keys: Cm, C#m, Dbm, etc.
    let minor_re = Regex::new(r"[_\-]([A-G]#?b?m(?:in)?)[_\-]").ok()?;
    if let Some(cap) = minor_re.captures(filename) {
        return Some(cap.get(1)?.as_str().to_string());
    }

    None
}

/// Extract time signature from filename
fn extract_time_signature(filename: &str) -> Option<String> {
    if filename.contains("threefour") || filename.contains("3-4") || filename.contains("3_4") {
        return Some("3/4".to_string());
    }
    if filename.contains("sixeight") || filename.contains("6-8") || filename.contains("6_8") {
        return Some("6/8".to_string());
    }
    if filename.contains("fivefour") || filename.contains("5-4") || filename.contains("5_4") {
        return Some("5/4".to_string());
    }
    if filename.contains("seveneight") || filename.contains("7-8") || filename.contains("7_8") {
        return Some("7/8".to_string());
    }

    None
}

/// Extract instruments from filename
fn extract_instruments(filename: &str) -> Vec<String> {
    let mut instruments = Vec::new();

    // Drums
    if filename.contains("drum") && !filename.contains("syndrome") {
        instruments.push("drums".to_string());
    }
    if filename.contains("percussion") {
        instruments.push("percussion".to_string());
    }
    if filename.contains("snare") {
        instruments.push("snare".to_string());
    }
    if filename.contains("kick") {
        instruments.push("kick".to_string());
    }
    if filename.contains("hat") || filename.contains("hihat") {
        instruments.push("hat".to_string());
    }
    if filename.contains("cymbal") {
        instruments.push("cymbal".to_string());
    }
    if filename.contains("tom") && !filename.contains("atom") && !filename.contains("custom") {
        instruments.push("tom".to_string());
    }
    if filename.contains("ride") {
        instruments.push("ride".to_string());
    }

    // Bass
    if filename.contains("bass") && !filename.contains("bass drum") {
        instruments.push("bass".to_string());
    }

    // Synth
    if filename.contains("synth") {
        instruments.push("synth".to_string());
    }
    if filename.contains("pad") {
        instruments.push("pad".to_string());
    }
    if filename.contains("lead") {
        instruments.push("lead".to_string());
    }

    // Keys
    if filename.contains("piano") {
        instruments.push("piano".to_string());
    }
    if filename.contains("organ") {
        instruments.push("organ".to_string());
    }
    if filename.contains("chord") {
        instruments.push("chords".to_string());
    }

    // Strings & Brass
    if filename.contains("string") {
        instruments.push("strings".to_string());
    }
    if filename.contains("brass") {
        instruments.push("brass".to_string());
    }

    // Guitar
    if filename.contains("guitar") {
        instruments.push("guitar".to_string());
    }

    instruments
}

/// Extract genres from filename
fn extract_genres(filename: &str) -> Vec<String> {
    let mut genres = Vec::new();

    // Electronic
    if filename.contains("house") {
        genres.push("house".to_string());
    }
    if filename.contains("techno") {
        genres.push("techno".to_string());
    }
    if filename.contains("trance") {
        genres.push("trance".to_string());
    }
    if filename.contains("dubstep") {
        genres.push("dubstep".to_string());
    }
    if filename.contains("dnb") || filename.contains("drum") && filename.contains("bass") {
        genres.push("dnb".to_string());
    }
    if filename.contains("jungle") {
        genres.push("jungle".to_string());
    }
    if filename.contains("breakbeat") || filename.contains("breaks") {
        genres.push("breakbeat".to_string());
    }
    if filename.contains("garage") {
        genres.push("garage".to_string());
    }
    if filename.contains("glitch") {
        genres.push("glitch".to_string());
    }
    if filename.contains("ambient") {
        genres.push("ambient".to_string());
    }

    // Hip-Hop & Urban
    if filename.contains("hiphop") || filename.contains("hip-hop") {
        genres.push("hip-hop".to_string());
    }
    if filename.contains("trap") {
        genres.push("trap".to_string());
    }
    if filename.contains("rnb") || filename.contains("r&b") {
        genres.push("rnb".to_string());
    }

    // Rock & Metal
    if filename.contains("rock") {
        genres.push("rock".to_string());
    }
    if filename.contains("metal") {
        genres.push("metal".to_string());
    }
    if filename.contains("punk") {
        genres.push("punk".to_string());
    }
    if filename.contains("blues") {
        genres.push("blues".to_string());
    }
    if filename.contains("funk") {
        genres.push("funk".to_string());
    }

    // Jazz
    if filename.contains("jazz") {
        genres.push("jazz".to_string());
    }
    if filename.contains("fusion") {
        genres.push("fusion".to_string());
    }

    // World
    if filename.contains("latin") {
        genres.push("latin".to_string());
    }
    if filename.contains("africa") {
        genres.push("african".to_string());
    }
    if filename.contains("asia") {
        genres.push("asian".to_string());
    }
    if filename.contains("world") {
        genres.push("world".to_string());
    }

    // Other
    if filename.contains("pop") {
        genres.push("pop".to_string());
    }
    if filename.contains("disco") {
        genres.push("disco".to_string());
    }
    if filename.contains("progressive") {
        genres.push("progressive".to_string());
    }

    genres
}

/// Extract pattern types from filename
fn extract_patterns(filename: &str) -> Vec<String> {
    let mut patterns = Vec::new();

    if filename.contains("fill") {
        patterns.push("fill".to_string());
    }
    if filename.contains("groove") {
        patterns.push("groove".to_string());
    }
    if filename.contains("intro") {
        patterns.push("intro".to_string());
    }
    if filename.contains("outro") || filename.contains("ending") {
        patterns.push("ending".to_string());
    }
    if filename.contains("breakdown") {
        patterns.push("breakdown".to_string());
    }
    if filename.contains("turnaround") {
        patterns.push("turnaround".to_string());
    }
    if filename.contains("verse") {
        patterns.push("verse".to_string());
    }
    if filename.contains("chorus") {
        patterns.push("chorus".to_string());
    }
    if filename.contains("bridge") {
        patterns.push("bridge".to_string());
    }
    if filename.contains("loop") {
        patterns.push("loop".to_string());
    }

    patterns
}

/// Extract drum elements from filename
fn extract_drum_elements(filename: &str) -> Vec<String> {
    let mut elements = Vec::new();

    // Cymbals
    if filename.contains("crash") {
        elements.push("crash".to_string());
    }
    if filename.contains("ride") {
        elements.push("ride".to_string());
    }
    if filename.contains("china") {
        elements.push("china".to_string());
    }
    if filename.contains("splash") {
        elements.push("splash".to_string());
    }

    // Hi-hats
    if filename.contains("closed") && (filename.contains("hat") || filename.contains("hihat")) {
        elements.push("closed-hat".to_string());
    }
    if filename.contains("open") && (filename.contains("hat") || filename.contains("hihat")) {
        elements.push("open-hat".to_string());
    }
    if filename.contains("pedal") && (filename.contains("hat") || filename.contains("hihat")) {
        elements.push("pedal-hat".to_string());
    }

    // Techniques
    if filename.contains("ghost") {
        elements.push("ghost-notes".to_string());
    }
    if filename.contains("double") && filename.contains("bass") {
        elements.push("double-bass".to_string());
    }
    if filename.contains("flam") {
        elements.push("flam".to_string());
    }
    if filename.contains("roll") {
        elements.push("roll".to_string());
    }

    // Feel
    if filename.contains("swing") {
        elements.push("swing".to_string());
    }
    if filename.contains("shuffle") {
        elements.push("shuffle".to_string());
    }
    if filename.contains("triplet") {
        elements.push("triplet".to_string());
    }

    elements
}

/// Analyze a single MIDI file
fn analyze_file(path: &Path, stats: &CollectionStats) {
    let filename = match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name,
        None => return,
    };

    stats.total_files.fetch_add(1, Ordering::Relaxed);

    // Extract metadata from filename
    let metadata = extract_metadata_from_filename(filename);

    // Update statistics
    if metadata.is_drum {
        stats.drum_files.fetch_add(1, Ordering::Relaxed);
    }

    for instrument in &metadata.instruments {
        stats.increment(&stats.instruments, instrument);
    }

    for genre in &metadata.genres {
        stats.increment(&stats.genres, genre);
    }

    for pattern in &metadata.patterns {
        stats.increment(&stats.patterns, pattern);
    }

    for element in &metadata.drum_elements {
        stats.increment(&stats.drum_elements, element);
    }

    if let Some(bpm) = metadata.bpm {
        stats.increment_bpm(bpm);
    }

    if let Some(key) = metadata.key {
        stats.increment(&stats.keys, &key);
    }

    if let Some(ts) = metadata.time_signature {
        stats.increment(&stats.time_signatures, &ts);
    }

    stats.analyzed_files.fetch_add(1, Ordering::Relaxed);
}

/// Recursively find all MIDI files
fn find_midi_files(root: &Path) -> Vec<PathBuf> {
    println!("Scanning directory tree...");
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

/// Generate markdown report
fn generate_report(stats: &CollectionStats, output_path: &Path) -> std::io::Result<()> {
    let mut output = String::new();

    // Header
    output.push_str("# Complete MIDI Collection Analysis\n\n");
    output.push_str(&format!(
        "**Total Files Analyzed:** {}\n\n",
        stats.total_files.load(Ordering::Relaxed)
    ));
    output.push_str(&format!(
        "**Drum Files:** {} ({:.1}%)\n\n",
        stats.drum_files.load(Ordering::Relaxed),
        stats.drum_files.load(Ordering::Relaxed) as f64
            / stats.total_files.load(Ordering::Relaxed) as f64
            * 100.0
    ));
    output.push_str("---\n\n");

    // Top Instruments
    output.push_str("## Top Instruments Found\n\n");
    output.push_str("| Instrument | Count | Percentage |\n");
    output.push_str("|------------|-------|------------|\n");

    let mut instruments: Vec<_> =
        stats.instruments.iter().map(|r| (r.key().clone(), *r.value())).collect();
    instruments.sort_by(|a, b| b.1.cmp(&a.1));
    let total = stats.total_files.load(Ordering::Relaxed);
    for (instrument, count) in instruments.iter().take(50) {
        output.push_str(&format!(
            "| {} | {} | {:.2}% |\n",
            instrument,
            count,
            *count as f64 / total as f64 * 100.0
        ));
    }
    output.push('\n');

    // Top Genres
    output.push_str("## Top Genres Found\n\n");
    output.push_str("| Genre | Count | Percentage |\n");
    output.push_str("|-------|-------|------------|\n");

    let mut genres: Vec<_> = stats.genres.iter().map(|r| (r.key().clone(), *r.value())).collect();
    genres.sort_by(|a, b| b.1.cmp(&a.1));
    for (genre, count) in genres.iter().take(50) {
        output.push_str(&format!(
            "| {} | {} | {:.2}% |\n",
            genre,
            count,
            *count as f64 / total as f64 * 100.0
        ));
    }
    output.push('\n');

    // Top Pattern Types
    output.push_str("## Top Pattern Types\n\n");
    output.push_str("| Pattern | Count | Percentage |\n");
    output.push_str("|---------|-------|------------|\n");

    let mut patterns: Vec<_> =
        stats.patterns.iter().map(|r| (r.key().clone(), *r.value())).collect();
    patterns.sort_by(|a, b| b.1.cmp(&a.1));
    for (pattern, count) in patterns.iter().take(30) {
        output.push_str(&format!(
            "| {} | {} | {:.2}% |\n",
            pattern,
            count,
            *count as f64 / total as f64 * 100.0
        ));
    }
    output.push('\n');

    // Musical Keys
    output.push_str("## Musical Keys Found\n\n");
    output.push_str("| Key | Count | Percentage |\n");
    output.push_str("|-----|-------|------------|\n");

    let mut keys: Vec<_> = stats.keys.iter().map(|r| (r.key().clone(), *r.value())).collect();
    keys.sort_by(|a, b| b.1.cmp(&a.1));
    for (key, count) in keys.iter().take(30) {
        output.push_str(&format!(
            "| {} | {} | {:.2}% |\n",
            key,
            count,
            *count as f64 / total as f64 * 100.0
        ));
    }
    output.push('\n');

    // BPM Distribution
    output.push_str("## BPM Distribution\n\n");
    output.push_str("| BPM | Count | Percentage |\n");
    output.push_str("|-----|-------|------------|\n");

    let mut bpms: Vec<_> = stats.bpms.iter().map(|r| (*r.key(), *r.value())).collect();
    bpms.sort_by(|a, b| b.1.cmp(&a.1));
    for (bpm, count) in bpms.iter().take(50) {
        output.push_str(&format!(
            "| {} | {} | {:.2}% |\n",
            bpm,
            count,
            *count as f64 / total as f64 * 100.0
        ));
    }
    output.push('\n');

    // Time Signatures
    output.push_str("## Time Signatures Found\n\n");
    output.push_str("| Time Signature | Count | Percentage |\n");
    output.push_str("|----------------|-------|------------|\n");

    let mut time_sigs: Vec<_> =
        stats.time_signatures.iter().map(|r| (r.key().clone(), *r.value())).collect();
    time_sigs.sort_by(|a, b| b.1.cmp(&a.1));
    for (ts, count) in time_sigs.iter() {
        output.push_str(&format!(
            "| {} | {} | {:.2}% |\n",
            ts,
            count,
            *count as f64 / total as f64 * 100.0
        ));
    }
    output.push('\n');

    // Drum Elements (for drum files)
    if stats.drum_files.load(Ordering::Relaxed) > 0 {
        output.push_str("## Drum Elements & Techniques\n\n");
        output.push_str("| Element | Count | Percentage (of drum files) |\n");
        output.push_str("|---------|-------|---------------------------|\n");

        let mut elements: Vec<_> =
            stats.drum_elements.iter().map(|r| (r.key().clone(), *r.value())).collect();
        elements.sort_by(|a, b| b.1.cmp(&a.1));
        let drum_total = stats.drum_files.load(Ordering::Relaxed);
        for (element, count) in elements.iter().take(50) {
            output.push_str(&format!(
                "| {} | {} | {:.2}% |\n",
                element,
                count,
                *count as f64 / drum_total as f64 * 100.0
            ));
        }
        output.push('\n');
    }

    // BPM Ranges Summary
    output.push_str("## BPM Ranges Summary\n\n");
    let mut bpm_ranges: HashMap<&str, u64> = HashMap::new();
    for (bpm, count) in bpms.iter() {
        let range = match bpm {
            30..=60 => "Very Slow (30-60)",
            61..=90 => "Slow (61-90)",
            91..=120 => "Mid-Tempo (91-120)",
            121..=140 => "Upbeat (121-140)",
            141..=180 => "Fast (141-180)",
            181..=300 => "Very Fast (181-300)",
            _ => "Other",
        };
        *bpm_ranges.entry(range).or_insert(0) += count;
    }

    output.push_str("| BPM Range | Count | Percentage |\n");
    output.push_str("|-----------|-------|------------|\n");
    let mut ranges: Vec<_> = bpm_ranges.into_iter().collect();
    ranges.sort_by(|a, b| b.1.cmp(&a.1));
    for (range, count) in ranges {
        output.push_str(&format!(
            "| {} | {} | {:.2}% |\n",
            range,
            count,
            count as f64 / total as f64 * 100.0
        ));
    }
    output.push('\n');

    // Write to file
    fs::write(output_path, output)?;

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let root_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from("/home/dojevou/projects/midi-software-center/midi-library/archives")
    };

    let output_path = if args.len() > 2 {
        PathBuf::from(&args[2])
    } else {
        PathBuf::from("COMPLETE_COLLECTION_ANALYSIS.md")
    };

    println!("MIDI Collection Analysis");
    println!("========================");
    println!("Root path: {}", root_path.display());
    println!("Output: {}", output_path.display());
    println!();

    // Find all MIDI files
    let files = find_midi_files(&root_path);
    println!("Starting analysis of {} files...\n", files.len());

    // Initialize statistics
    let stats = Arc::new(CollectionStats::new());

    // Process files in parallel
    let start = Instant::now();
    let progress_interval = files.len() / 100; // Print progress every 1%

    files.par_iter().enumerate().for_each(|(i, path)| {
        analyze_file(path, &stats);

        // Print progress
        if i % progress_interval == 0 && i > 0 {
            let analyzed = stats.analyzed_files.load(Ordering::Relaxed);
            let elapsed = start.elapsed();
            let rate = analyzed as f64 / elapsed.as_secs_f64();
            println!(
                "Progress: {:.1}% ({}/{} files) - {:.0} files/sec",
                i as f64 / files.len() as f64 * 100.0,
                analyzed,
                files.len(),
                rate
            );
        }
    });

    let elapsed = start.elapsed();
    let total = stats.total_files.load(Ordering::Relaxed);
    let rate = total as f64 / elapsed.as_secs_f64();

    println!("\nAnalysis complete!");
    println!("Total files: {}", total);
    println!("Time: {:?}", elapsed);
    println!("Rate: {:.0} files/sec", rate);
    println!();

    // Generate report
    println!("Generating report...");
    match generate_report(&stats, &output_path) {
        Ok(_) => println!("Report saved to: {}", output_path.display()),
        Err(e) => eprintln!("Error generating report: {}", e),
    }
}

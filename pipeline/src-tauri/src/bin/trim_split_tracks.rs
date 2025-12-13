#!/usr/bin/env rust-script
//! Trim leading silence from MIDI split tracks
//!
//! This tool processes MIDI files and removes any leading silence by:
//! 1. Finding the first note-on event
//! 2. Shifting all events back by that offset
//! 3. Saving the trimmed file
//!
//! Example patterns to trim from split files where pattern starts at bar 64.

use anyhow::{Context, Result};
use clap::Parser;
use midly::{MidiMessage, Smf, TrackEvent, TrackEventKind};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(name = "trim_split_tracks")]
#[command(about = "Remove leading silence from MIDI split tracks", long_about = None)]
struct Args {
    /// Directory containing split MIDI files
    #[arg(short, long, default_value = "/home/dojevou/tmp/midi_splits_fast")]
    input_dir: PathBuf,

    /// Number of worker threads
    #[arg(short, long, default_value = "16")]
    workers: usize,

    /// Minimum leading ticks to trim (avoid trimming tiny offsets)
    #[arg(short, long, default_value = "100")]
    min_trim_ticks: u32,

    /// Dry run (don't modify files)
    #[arg(short = 'n', long)]
    dry_run: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Default)]
struct TrimStats {
    files_processed: AtomicUsize,
    files_trimmed: AtomicUsize,
    files_skipped: AtomicUsize,
    files_error: AtomicUsize,
    total_ticks_trimmed: AtomicUsize,
}

impl TrimStats {
    fn print_progress(&self, total: usize) {
        let processed = self.files_processed.load(Ordering::Relaxed);
        let trimmed = self.files_trimmed.load(Ordering::Relaxed);
        let skipped = self.files_skipped.load(Ordering::Relaxed);
        let errors = self.files_error.load(Ordering::Relaxed);

        if processed.is_multiple_of(1000) {
            eprintln!(
                "Progress: {}/{} files ({:.1}%) | Trimmed: {} | Skipped: {} | Errors: {}",
                processed,
                total,
                (processed as f64 / total as f64) * 100.0,
                trimmed,
                skipped,
                errors
            );
        }
    }

    fn print_final(&self, elapsed: f64) {
        let processed = self.files_processed.load(Ordering::Relaxed);
        let trimmed = self.files_trimmed.load(Ordering::Relaxed);
        let skipped = self.files_skipped.load(Ordering::Relaxed);
        let errors = self.files_error.load(Ordering::Relaxed);
        let total_ticks = self.total_ticks_trimmed.load(Ordering::Relaxed);

        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("✅ TRIMMING COMPLETE");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Files processed:    {}", processed);
        println!("Files trimmed:      {}", trimmed);
        println!("Files skipped:      {}", skipped);
        println!("Files with errors:  {}", errors);
        println!("Total ticks removed: {}", total_ticks);
        println!("Time elapsed:       {:.1}s", elapsed);
        println!(
            "Speed:              {:.0} files/sec",
            processed as f64 / elapsed
        );

        if trimmed > 0 {
            let avg_ticks = total_ticks as f64 / trimmed as f64;
            println!("Average trim:       {:.0} ticks per file", avg_ticks);
        }
    }
}

/// Find the first note-on event in the MIDI file and return its absolute tick position
fn find_first_note_on(smf: &Smf) -> Option<u32> {
    let mut first_note_tick = None;

    for track in &smf.tracks {
        let mut current_tick = 0u32;

        for event in track {
            // Accumulate delta time
            current_tick = current_tick.saturating_add(event.delta.as_int());

            // Check if this is a note-on event
            if let TrackEventKind::Midi { message, .. } = &event.kind {
                if let MidiMessage::NoteOn { vel, .. } = message {
                    // Only consider actual note-on (velocity > 0)
                    if vel.as_int() > 0 {
                        match first_note_tick {
                            None => first_note_tick = Some(current_tick),
                            Some(tick) => {
                                if current_tick < tick {
                                    first_note_tick = Some(current_tick);
                                }
                            },
                        }
                    }
                }
            }
        }
    }

    first_note_tick
}

/// Trim the MIDI file by shifting all events back by the given offset
fn trim_midi_file(smf: &mut Smf, trim_ticks: u32) {
    for track in &mut smf.tracks {
        let mut current_tick = 0u32;
        let mut trimmed_events = Vec::new();
        let mut last_output_tick = 0u32;

        for event in track.iter() {
            // Accumulate absolute time
            current_tick = current_tick.saturating_add(event.delta.as_int());

            // Calculate new absolute time after trimming
            let new_tick = current_tick.saturating_sub(trim_ticks);

            // Calculate delta from last output event
            let new_delta = new_tick.saturating_sub(last_output_tick);

            // Create new event with adjusted delta
            let new_event = TrackEvent { delta: midly::num::u28::new(new_delta), kind: event.kind };

            trimmed_events.push(new_event);
            last_output_tick = new_tick;
        }

        *track = trimmed_events;
    }
}

/// Process a single MIDI file
fn process_file(path: &Path, args: &Args, stats: &TrimStats) -> Result<()> {
    stats.files_processed.fetch_add(1, Ordering::Relaxed);

    // Read MIDI file
    let data =
        fs::read(path).with_context(|| format!("Failed to read file: {}", path.display()))?;

    let mut smf =
        Smf::parse(&data).with_context(|| format!("Failed to parse MIDI: {}", path.display()))?;

    // Find first note-on
    let first_note_tick = match find_first_note_on(&smf) {
        Some(tick) => tick,
        None => {
            if args.verbose {
                eprintln!("No note-on events found: {}", path.display());
            }
            stats.files_skipped.fetch_add(1, Ordering::Relaxed);
            return Ok(());
        },
    };

    // Skip if leading silence is too small
    if first_note_tick < args.min_trim_ticks {
        if args.verbose {
            eprintln!(
                "Leading silence too small ({} ticks): {}",
                first_note_tick,
                path.display()
            );
        }
        stats.files_skipped.fetch_add(1, Ordering::Relaxed);
        return Ok(());
    }

    if args.verbose {
        eprintln!(
            "Trimming {} ticks from: {}",
            first_note_tick,
            path.display()
        );
    }

    // Trim the file
    trim_midi_file(&mut smf, first_note_tick);

    // Save the trimmed file (unless dry-run)
    if !args.dry_run {
        let mut output = Vec::new();
        smf.write(&mut output)
            .map_err(|e| anyhow::anyhow!("Failed to encode MIDI: {} - {}", path.display(), e))?;

        fs::write(path, output)
            .with_context(|| format!("Failed to write file: {}", path.display()))?;
    }

    stats.files_trimmed.fetch_add(1, Ordering::Relaxed);
    stats.total_ticks_trimmed.fetch_add(first_note_tick as usize, Ordering::Relaxed);

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  MIDI Track Trimming Tool");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Input directory:  {}", args.input_dir.display());
    println!("Workers:          {}", args.workers);
    println!("Min trim ticks:   {}", args.min_trim_ticks);
    println!(
        "Dry run:          {}",
        if args.dry_run { "YES" } else { "NO" }
    );
    println!();

    // Set up thread pool
    rayon::ThreadPoolBuilder::new()
        .num_threads(args.workers)
        .build_global()
        .context("Failed to build thread pool")?;

    // Collect all MIDI files
    println!("Scanning for MIDI files...");
    let mut midi_files = Vec::new();
    for entry in walkdir::WalkDir::new(&args.input_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "mid" || ext == "midi" {
                    midi_files.push(path.to_path_buf());
                }
            }
        }
    }

    let total_files = midi_files.len();
    println!("Found {} MIDI files", total_files);
    println!();

    if total_files == 0 {
        println!("No MIDI files found!");
        return Ok(());
    }

    // Process files in parallel
    let stats = TrimStats::default();
    let start = Instant::now();

    midi_files.par_iter().for_each(|path| {
        if let Err(e) = process_file(path, &args, &stats) {
            eprintln!("Error processing {}: {}", path.display(), e);
            stats.files_error.fetch_add(1, Ordering::Relaxed);
        }
        stats.print_progress(total_files);
    });

    let elapsed = start.elapsed().as_secs_f64();
    stats.print_final(elapsed);

    Ok(())
}

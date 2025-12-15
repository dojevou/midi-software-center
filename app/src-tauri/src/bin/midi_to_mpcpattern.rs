#!/usr/bin/env rust-script
//! MIDI to .mpcpattern Converter
//!
//! Converts standard MIDI files to Akai Force/MPC .mpcpattern format (JSON).
//!
//! Usage:
//!   cargo run --bin midi_to_mpcpattern -- input.mid output.mpcpattern
//!   cargo run --bin midi_to_mpcpattern -- --batch /path/to/midi/files /path/to/output

use anyhow::{Context, Result};
use midi_app::core::midi::parse_midi_file;
use midi_app::core::midi::types::Event;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug)]
struct ActiveNote {
    note: u8,
    velocity: u8,
    start_time: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct MpcPattern {
    pattern: Pattern,
}

#[derive(Debug, Serialize, Deserialize)]
struct Pattern {
    length: i64,
    events: Vec<MpcEvent>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MpcEvent {
    #[serde(rename = "type")]
    event_type: u8,
    time: i64,
    len: i64,
    #[serde(rename = "1")]
    field1: i32,
    #[serde(rename = "2")]
    field2: f64,
    #[serde(rename = "3")]
    field3: i32,
    #[serde(rename = "mod")]
    mod_field: i32,
    #[serde(rename = "modVal")]
    mod_val: f64,
}

impl MpcEvent {
    /// Create a Type 2 event (note on with duration) - Midian format
    fn note_on(time: i64, duration: i64, note: u8, velocity: f64) -> Self {
        Self {
            event_type: 2,
            time,
            len: duration,
            field1: note as i32, // Note number in field1
            field2: velocity,
            field3: 0, // Always 0
            mod_field: 0,
            mod_val: 0.5, // Always 0.5 for Type 2
        }
    }

    /// Create Type 1 initialization events
    fn init_event(field1: i32, velocity: f64) -> Self {
        Self {
            event_type: 1,
            time: 0,
            len: 0,
            field1,
            field2: velocity,
            field3: 0,
            mod_field: 0,
            mod_val: 0.0,
        }
    }
}

fn convert_midi_to_mpcpattern(midi_path: &Path) -> Result<MpcPattern> {
    // Parse MIDI file
    let midi_data = fs::read(midi_path)
        .with_context(|| format!("Failed to read MIDI file: {}", midi_path.display()))?;

    let midi_file = parse_midi_file(&midi_data).context("Failed to parse MIDI file")?;

    let mut mpc_events = Vec::new();
    let mut active_notes: HashMap<u8, ActiveNote> = HashMap::new();

    // Add initialization events (matching Midian format)
    mpc_events.push(MpcEvent::init_event(0, 0.0));
    mpc_events.push(MpcEvent::init_event(32, 0.0));

    // Calculate a reasonable velocity for the third init event (Midian uses ~0.787)
    // We'll use a default or calculate from first note
    let init_velocity = 0.787401556968689;
    mpc_events.push(MpcEvent::init_event(130, init_velocity));

    // Process all tracks
    for track in &midi_file.tracks {
        let mut current_time = 0i64;

        for timed_event in &track.events {
            current_time += timed_event.delta_ticks as i64;

            // Scale time by 2x to match MPC expected resolution (Midian compatibility)
            let scaled_time = current_time * 2;

            match &timed_event.event {
                // Note On
                Event::NoteOn { channel: _, note, velocity } => {
                    if *velocity > 0 {
                        // Store active note
                        active_notes.insert(
                            *note,
                            ActiveNote {
                                note: *note,
                                velocity: *velocity,
                                start_time: scaled_time,
                            },
                        );
                    } else {
                        // Note on with velocity 0 = note off
                        if let Some(active) = active_notes.remove(note) {
                            let duration = scaled_time - active.start_time;
                            let normalized_velocity = active.velocity as f64 / 127.0;

                            mpc_events.push(MpcEvent::note_on(
                                active.start_time,
                                duration,
                                *note,
                                normalized_velocity,
                            ));
                        }
                    }
                },

                // Note Off
                Event::NoteOff { channel: _, note, velocity: _ } => {
                    if let Some(active) = active_notes.remove(note) {
                        let duration = scaled_time - active.start_time;
                        let normalized_velocity = active.velocity as f64 / 127.0;

                        mpc_events.push(MpcEvent::note_on(
                            active.start_time,
                            duration,
                            *note,
                            normalized_velocity,
                        ));
                    }
                },

                _ => {
                    // Ignore other MIDI events (tempo, control changes, etc.)
                },
            }
        }
    }

    // Close any remaining active notes at pattern end
    // Use scaled max_time to match 2x resolution
    let max_time = mpc_events.iter()
        .filter(|e| e.event_type == 2)  // Only Type 2 events have timing
        .map(|e| e.time + e.len)
        .max()
        .unwrap_or(0);

    for active in active_notes.values() {
        let duration = max_time - active.start_time;
        let normalized_velocity = active.velocity as f64 / 127.0;

        mpc_events.push(MpcEvent::note_on(
            active.start_time,
            duration,
            active.note,
            normalized_velocity,
        ));
    }

    // Sort events: initialization events first (time 0, type 1), then pattern events by time
    mpc_events.sort_by_key(|e| (e.time, e.event_type, e.field1));

    Ok(MpcPattern {
        pattern: Pattern {
            length: i64::MAX, // Standard max value
            events: mpc_events,
        },
    })
}

fn convert_file(input: &Path, output: &Path) -> Result<()> {
    println!("Converting: {} -> {}", input.display(), output.display());

    let pattern = convert_midi_to_mpcpattern(input)?;

    // Write JSON with pretty formatting
    let json =
        serde_json::to_string_pretty(&pattern).context("Failed to serialize pattern to JSON")?;

    fs::write(output, json)
        .with_context(|| format!("Failed to write output file: {}", output.display()))?;

    println!("  ✓ Created {} events", pattern.pattern.events.len());

    Ok(())
}

fn batch_convert(input_dir: &Path, output_dir: &Path, limit: Option<usize>) -> Result<()> {
    println!("Batch converting MIDI files...");
    println!("  Input:  {}", input_dir.display());
    println!("  Output: {}", output_dir.display());
    if let Some(lim) = limit {
        println!("  Limit:  {} files", lim);
    }
    println!();

    // Create output directory
    fs::create_dir_all(output_dir).with_context(|| {
        format!(
            "Failed to create output directory: {}",
            output_dir.display()
        )
    })?;

    // Find all MIDI files
    let mut midi_files = Vec::new();
    for entry in walkdir::WalkDir::new(input_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext.eq_ignore_ascii_case("mid") || ext.eq_ignore_ascii_case("midi") {
                    midi_files.push(path.to_path_buf());
                }
            }
        }
    }

    println!("Found {} MIDI files", midi_files.len());

    // Apply limit
    if let Some(lim) = limit {
        midi_files.truncate(lim);
        println!("Processing first {} files", midi_files.len());
    }

    println!();

    // Convert each file
    let mut success = 0;
    let mut failed = 0;

    for (i, input_path) in midi_files.iter().enumerate() {
        let file_stem = input_path.file_stem().unwrap().to_string_lossy();
        let output_path = output_dir.join(format!("{}.mpcpattern", file_stem));

        print!("[{}/{}] ", i + 1, midi_files.len());

        match convert_file(input_path, &output_path) {
            Ok(_) => success += 1,
            Err(e) => {
                println!("  ✗ Error: {}", e);
                failed += 1;
            },
        }
    }

    println!();
    println!("Conversion complete:");
    println!("  Success: {}", success);
    println!("  Failed:  {}", failed);

    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage:");
        eprintln!("  {} <input.mid> <output.mpcpattern>", args[0]);
        eprintln!("  {} --batch <input_dir> <output_dir> [limit]", args[0]);
        std::process::exit(1);
    }

    if args[1] == "--batch" {
        if args.len() < 4 {
            eprintln!("Batch mode requires input and output directories");
            std::process::exit(1);
        }

        let input_dir = Path::new(&args[2]);
        let output_dir = Path::new(&args[3]);
        let limit = args.get(4).and_then(|s| s.parse::<usize>().ok());

        batch_convert(input_dir, output_dir, limit)?;
    } else {
        let input = Path::new(&args[1]);
        let output = Path::new(&args[2]);

        convert_file(input, output)?;
    }

    Ok(())
}

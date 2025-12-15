/// Split binary - standalone executable for splitting multi-track MIDI files
use anyhow::{Context, Result};
use clap::Parser;
use sqlx::PgPool;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "split")]
#[command(about = "Split multi-track MIDI files", long_about = None)]
struct Args {
    /// MIDI file to split
    #[arg(short, long)]
    file: PathBuf,

    /// Output directory for split files
    #[arg(short, long)]
    output: PathBuf,

    /// Database connection string
    #[arg(short = 'D', long, env = "DATABASE_URL")]
    database_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("🎵 MIDI Split Tool");
    println!("File: {:?}", args.file);
    println!("Output: {:?}", args.output);

    // Connect to database
    let _pool = PgPool::connect(&args.database_url)
        .await
        .context("Failed to connect to database")?;

    println!("✅ Database connected");

    // Verify input file exists
    if !args.file.exists() {
        anyhow::bail!("File does not exist: {:?}", args.file);
    }

    if !args.file.is_file() {
        anyhow::bail!("Path is not a file: {:?}", args.file);
    }

    // Verify it's a MIDI file
    let is_midi = args
        .file
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("mid") || e.eq_ignore_ascii_case("midi"))
        .unwrap_or(false);

    if !is_midi {
        anyhow::bail!("File is not a MIDI file: {:?}", args.file);
    }

    // Create output directory if it doesn't exist
    if !args.output.exists() {
        println!("📁 Creating output directory: {:?}", args.output);
        tokio::fs::create_dir_all(&args.output)
            .await
            .context("Failed to create output directory")?;
    }

    println!("🎵 Reading MIDI file...");

    // Read and parse MIDI file
    let content = tokio::fs::read(&args.file).await.context("Failed to read file")?;

    let midi_file = midi_library_shared::core::midi::parser::parse_midi_file(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse MIDI file: {}", e))?;

    let track_count = midi_file.tracks.len();
    println!("✅ Found {} track(s)", track_count);

    if track_count <= 1 {
        println!(
            "⚠️  File only has {} track(s), nothing to split",
            track_count
        );
        return Ok(());
    }

    // Split tracks using the splitter module
    println!("🔧 Splitting tracks...");

    use midi_library_shared::core::midi::types::*;

    let mut split_count = 0;

    for (track_idx, track) in midi_file.tracks.iter().enumerate() {
        // Skip empty tracks
        if track.events.is_empty() {
            continue;
        }

        // Create a new MIDI file with just this track
        let split_file = MidiFile {
            header: midi_library_shared::core::midi::types::Header {
                format: 0, // Single track format
                num_tracks: 1,
                ticks_per_quarter_note: midi_file.header.ticks_per_quarter_note,
            },
            tracks: vec![track.clone()],
        };

        // Generate output filename
        let base_name = args.file.file_stem().and_then(|s| s.to_str()).unwrap_or("track");

        let track_name = format!("track_{}", track_idx + 1);
        let sanitized_track_name = sanitize_filename(&track_name);

        let output_filename = if sanitized_track_name.is_empty() {
            format!("{}_{:02}.mid", base_name, track_idx + 1)
        } else {
            format!(
                "{}_{:02}_{}.mid",
                base_name,
                track_idx + 1,
                sanitized_track_name
            )
        };

        let output_path = args.output.join(&output_filename);

        // Serialize the single-track MIDI file
        let split_bytes = serialize_midi_file(&split_file)
            .map_err(|e| anyhow::anyhow!("Failed to serialize track {}: {}", track_idx + 1, e))?;

        // Write to file
        tokio::fs::write(&output_path, &split_bytes).await.context(format!(
            "Failed to write track {} to {:?}",
            track_idx + 1,
            output_path
        ))?;

        // Calculate hash for database
        let hash = blake3::hash(&split_bytes);
        let hash_bytes = hash.as_bytes();

        // Insert into database
        let result = sqlx::query!(
            r#"
            INSERT INTO files (
                filename, original_filename, filepath,
                content_hash, file_size_bytes,
                format, num_tracks, ticks_per_quarter_note,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
            RETURNING id
            "#,
            output_filename,
            output_filename,
            output_path.to_str().unwrap_or(""),
            hash_bytes,
            split_bytes.len() as i64,
            split_file.header.format as i16,
            1i16, // Always 1 track in split files
            split_file.header.ticks_per_quarter_note as i32
        )
        .fetch_one(&_pool)
        .await
        .context("Failed to insert split file into database")?;

        split_count += 1;
        println!(
            "  ✓ Track {:02}: {} -> {} (ID: {})",
            track_idx + 1,
            track_name,
            output_filename,
            result.id
        );
    }

    println!();
    println!("✅ Split completed!");
    println!("   Tracks split: {}", split_count);
    println!("   Output directory: {:?}", args.output);

    Ok(())
}

/// Sanitize a filename by removing/replacing invalid characters
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// Serialize a MIDI file back to bytes
fn serialize_midi_file(
    midi_file: &midi_library_shared::core::midi::types::MidiFile,
) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();

    // Write MThd header
    buffer.extend_from_slice(b"MThd");
    buffer.extend_from_slice(&6u32.to_be_bytes()); // Header length
    buffer.extend_from_slice(&midi_file.header.format.to_be_bytes());
    buffer.extend_from_slice(&(midi_file.tracks.len() as u16).to_be_bytes());
    buffer.extend_from_slice(&midi_file.header.ticks_per_quarter_note.to_be_bytes());

    // Write each track
    for track in &midi_file.tracks {
        let track_data = serialize_track(track)?;

        buffer.extend_from_slice(b"MTrk");
        buffer.extend_from_slice(&(track_data.len() as u32).to_be_bytes());
        buffer.extend_from_slice(&track_data);
    }

    Ok(buffer)
}

/// Serialize a track to bytes
fn serialize_track(track: &midi_library_shared::core::midi::types::Track) -> Result<Vec<u8>> {
    use midi_library_shared::core::midi::Event;

    let mut buffer = Vec::new();

    for timed_event in &track.events {
        // Write delta time
        write_variable_length(&mut buffer, timed_event.delta_ticks);

        // Write event (simplified - just the basic structure)
        match &timed_event.event {
            Event::NoteOn { channel, note, velocity } => {
                buffer.push(0x90 | channel);
                buffer.push(*note);
                buffer.push(*velocity);
            },
            Event::NoteOff { channel, note, velocity } => {
                buffer.push(0x80 | channel);
                buffer.push(*note);
                buffer.push(*velocity);
            },
            _ => {
                // For other events, write a no-op (this is simplified)
                buffer.push(0xFF); // Meta event
                buffer.push(0x00); // Sequence number
                buffer.push(0x00); // Length 0
            },
        }
    }

    // Write end of track
    buffer.push(0x00); // Delta time 0
    buffer.push(0xFF); // Meta event
    buffer.push(0x2F); // End of track
    buffer.push(0x00); // Length 0

    Ok(buffer)
}

/// Write a variable-length quantity (MIDI format)
fn write_variable_length(buffer: &mut Vec<u8>, mut value: u32) {
    let mut bytes = Vec::new();

    bytes.push((value & 0x7F) as u8);
    value >>= 7;

    while value > 0 {
        bytes.push(((value & 0x7F) | 0x80) as u8);
        value >>= 7;
    }

    // Write in reverse order
    for byte in bytes.iter().rev() {
        buffer.push(*byte);
    }
}

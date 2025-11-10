#!/bin/bash
# Test script for text metadata extraction
# Tests the new text_metadata module with real MIDI files

set -e

echo "=== Text Metadata Extraction Test ==="
echo

# Test MIDI file from music21 package
TEST_FILE="/home/dojevou/.pyenv/versions/3.10.13/lib/python3.10/site-packages/music21/omr/k525MIDIMvt1.mid"

if [ ! -f "$TEST_FILE" ]; then
    echo "❌ Test file not found: $TEST_FILE"
    exit 1
fi

echo "📁 Test file: $(basename $TEST_FILE)"
echo "📏 File size: $(du -h "$TEST_FILE" | cut -f1)"
echo

# Create temporary test program
cat > /tmp/test_text_meta.rs <<'EOF'
use std::fs;
use midi_library_shared::core::midi::parser::parse_midi_file;
use midi_library_shared::core::midi::text_metadata::TextMetadata;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <midi-file>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    println!("🎵 Analyzing: {}", file_path);

    // Read and parse MIDI file
    let data = fs::read(file_path)?;
    let midi_file = parse_midi_file(&data)?;

    println!("✅ MIDI file parsed successfully");
    println!("   Format: {}", midi_file.header.format);
    println!("   Tracks: {}", midi_file.header.num_tracks);
    println!("   TPQN: {}", midi_file.header.ticks_per_quarter_note);
    println!();

    // Extract text metadata
    let metadata = TextMetadata::extract(&midi_file);

    println!("📝 Text Metadata Extracted:");
    println!("   Track names: {:?}", metadata.track_names);
    println!("   Copyright: {:?}", metadata.copyright);
    println!("   Instrument names: {:?}", metadata.instrument_names);
    println!("   Markers: {:?}", metadata.markers);
    println!("   Lyrics: {} lines", metadata.lyrics.len());
    println!("   Summary: {}", metadata.summary());
    println!();

    if metadata.is_empty() {
        println!("⚠️  No text metadata found in this file");
    } else {
        println!("✅ Text metadata extraction successful!");
    }

    Ok(())
}
EOF

# Build and run test
cd /home/dojevou/projects/midi-software-center/shared/rust
echo "🔨 Building test program..."
rustc --edition 2021 \
    --extern midi_library_shared=/home/dojevou/projects/midi-software-center/target/debug/libmidi_library_shared.rlib \
    -L /home/dojevou/projects/midi-software-center/target/debug/deps \
    /tmp/test_text_meta.rs \
    -o /tmp/test_text_meta 2>&1 | head -20

if [ -f /tmp/test_text_meta ]; then
    echo "✅ Build successful"
    echo
    /tmp/test_text_meta "$TEST_FILE"
else
    echo "❌ Build failed"
    exit 1
fi

echo
echo "=== Testing Database Integration ==="
echo

# Check if data can be queried
psql $DATABASE_URL -c "SELECT
    COUNT(*) FILTER (WHERE array_length(track_names, 1) > 0) as with_track_names,
    COUNT(*) FILTER (WHERE copyright IS NOT NULL) as with_copyright,
    COUNT(*) FILTER (WHERE array_length(markers, 1) > 0) as with_markers,
    COUNT(*) FILTER (WHERE array_length(lyrics, 1) > 0) as with_lyrics,
    COUNT(*) as total_files
FROM files;" 2>/dev/null || echo "⚠️  No files in database yet"

echo
echo "=== Test Complete ==="

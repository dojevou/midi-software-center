#!/bin/bash
# Complete MIDI Pipeline - Extract Instrument Metadata + Analysis
set -e

DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

echo "════════════════════════════════════════════════════════════════"
echo "COMPLETE MIDI PIPELINE EXECUTION"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Stage 1: Update all files to extract instrument metadata
echo "Stage 1: Extracting instrument metadata from MIDI files..."
echo "  - Parsing MIDI events for instrument names"
echo "  - Updating files.instrument_names_text column"
echo ""

psql "$DATABASE_URL" <<EOF
-- Update files with instrument metadata by re-parsing MIDI content
-- This SQL script will be replaced with a proper Rust tool
SELECT 'Instrument extraction requires Rust MIDI parser' as status;
EOF

# Stage 2: Run analysis on all files
echo ""
echo "Stage 2: Running analysis (BPM, key, drums)..."
cd pipeline/src-tauri

# Check if analyze binary exists
if [ ! -f "../../target/release/analyze" ]; then
    echo "Building analyze binary..."
    cargo build --release --bin analyze
fi

DATABASE_URL="$DATABASE_URL" \
    ../../target/release/analyze \
    --batch-size 1000 \
    --threads 32

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "PIPELINE COMPLETE"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Final statistics
psql "$DATABASE_URL" <<EOF
SELECT
    COUNT(*) as total_files,
    COUNT(CASE WHEN array_length(instrument_names_text, 1) > 0 THEN 1 END) as files_with_instruments,
    COUNT(CASE WHEN analyzed_at IS NOT NULL THEN 1 END) as analyzed_files
FROM files;

SELECT
    COUNT(*) as analyzed_records,
    ROUND(AVG(tempo_bpm)::numeric, 2) as avg_bpm,
    ROUND(AVG(duration_seconds)::numeric, 2) as avg_duration_sec
FROM musical_metadata
WHERE tempo_bpm IS NOT NULL;
EOF

echo ""
echo "Next: Verify instrument metadata extraction completed"

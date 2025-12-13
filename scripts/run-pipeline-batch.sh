#!/bin/bash
# Run MIDI pipeline in batch mode (no user confirmation)
# Optimized for large collections (4M+ files)

set -e

SOURCE_DIR="/home/dojevou/Uncontaminated/floorp_downloads/midi"
PROJECT_DIR="/home/dojevou/projects/midi-software-center"
LIBRARY_DIR="$PROJECT_DIR/midi-library"

echo "╔════════════════════════════════════════════════════════════════════╗"
echo "║      MIDI Software Center - Batch Pipeline (4.3M files)           ║"
echo "╚════════════════════════════════════════════════════════════════════╝"
echo ""
echo "Collection: 4,314,593 MIDI files + 56 archives"
echo "Estimated time: ~20 hours (all phases)"
echo ""

# Ensure database is running
echo "[Step 1/5] Checking database..."
if ! docker ps | grep -q postgres; then
    echo "  Starting PostgreSQL..."
    cd "$PROJECT_DIR"
    make docker-up
    sleep 5
fi
echo "  ✓ Database running"
echo ""

# Create directory structure
echo "[Step 2/5] Creating library structure..."
mkdir -p "$LIBRARY_DIR"/{imported,organized,archives,temp}
echo "  ✓ Directories created"
echo ""

# Build binaries if needed
echo "[Step 3/5] Building pipeline binaries..."
cd "$PROJECT_DIR/pipeline/src-tauri"
if [ ! -f "../../target/release/import" ] || [ ! -f "../../target/release/analyze" ]; then
    echo "  Building release binaries (this will take 5-10 minutes)..."
    cargo build --release --bins 2>&1 | grep -E "(Compiling|Finished)" || true
fi
echo "  ✓ Binaries ready"
echo ""

echo "════════════════════════════════════════════════════════════════════"
echo "[Step 4/5] IMPORT PHASE (estimated: 18 minutes for 4.3M files)"
echo "════════════════════════════════════════════════════════════════════"
echo ""
echo "Processing:"
echo "  • Calculating BLAKE3 hashes"
echo "  • Deduplication checks"
echo "  • MIDI parsing"
echo "  • Filename metadata extraction"
echo "  • Database insertion (batch 500)"
echo ""
echo "Performance target: 3,915 files/sec"
echo "Expected time: 4,314,593 files ÷ 3,915 files/sec = 1,102 seconds (~18 minutes)"
echo ""

START_TIME=$(date +%s)

# Run import
DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library" \
    cargo run --release --bin import -- \
    --input "$SOURCE_DIR" \
    --recursive \
    --threads 8 \
    2>&1 | tee /tmp/import_log.txt

IMPORT_END=$(date +%s)
IMPORT_DURATION=$((IMPORT_END - START_TIME))
echo ""
echo "✓ Import complete in $IMPORT_DURATION seconds"
echo ""

echo "════════════════════════════════════════════════════════════════════"
echo "[Step 5/5] ANALYSIS PHASE (estimated: 13 hours for 4.3M files)"
echo "════════════════════════════════════════════════════════════════════"
echo ""
echo "Analyzing all files:"
echo "  • BPM detection (interval + onset)"
echo "  • Key detection (Krumhansl-Schmuckler)"
echo "  • Chord analysis"
echo "  • Drum analysis"
echo "  • Time signature extraction"
echo "  • Auto-tagging"
echo ""
echo "Performance target: 90.5 files/sec"
echo "Expected time: 4,314,593 files ÷ 90.5 files/sec = 47,679 seconds (~13.2 hours)"
echo ""
echo "Progress will be logged to: /tmp/analyze_log.txt"
echo "Monitor with: tail -f /tmp/analyze_log.txt"
echo ""

ANALYZE_START=$(date +%s)

# Run analysis
DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library" \
    cargo run --release --bin analyze -- \
    --batch-size 100 \
    --threads 8 \
    2>&1 | tee /tmp/analyze_log.txt

ANALYZE_END=$(date +%s)
ANALYZE_DURATION=$((ANALYZE_END - ANALYZE_START))
TOTAL_DURATION=$((ANALYZE_END - START_TIME))

echo ""
echo "✓ Analysis complete in $ANALYZE_DURATION seconds"
echo ""

echo "════════════════════════════════════════════════════════════════════"
echo "PIPELINE COMPLETE ✓"
echo "════════════════════════════════════════════════════════════════════"
echo ""
echo "Timing:"
echo "  • Import: $IMPORT_DURATION seconds"
echo "  • Analysis: $ANALYZE_DURATION seconds"
echo "  • Total: $TOTAL_DURATION seconds ($(echo "scale=2; $TOTAL_DURATION/3600" | bc) hours)"
echo ""

# Get final statistics
echo "Final Statistics:"
echo ""

psql "postgresql://midiuser:145278963@localhost:5433/midi_library" <<EOF
SELECT
    COUNT(*) as total_files,
    COUNT(DISTINCT category) as categories,
    ROUND(AVG(file_size_bytes)::numeric, 2) as avg_size_bytes,
    pg_size_pretty(SUM(file_size_bytes)::bigint) as total_size
FROM files;

SELECT
    COUNT(*) as analyzed_files,
    COUNT(*) FILTER (WHERE bpm IS NOT NULL) as files_with_bpm,
    ROUND(AVG(bpm)::numeric, 2) as avg_bpm,
    ROUND(AVG(duration_seconds)::numeric, 2) as avg_duration_sec
FROM file_metadata;

SELECT
    category,
    COUNT(*) as file_count,
    ROUND(100.0 * COUNT(*) / SUM(COUNT(*)) OVER (), 2) as percentage
FROM files
WHERE category IS NOT NULL
GROUP BY category
ORDER BY file_count DESC
LIMIT 15;

SELECT
    COUNT(DISTINCT tag_name) as unique_tags,
    COUNT(*) as total_tag_associations,
    ROUND(AVG(tag_count), 2) as avg_tags_per_file
FROM (
    SELECT file_id, COUNT(*) as tag_count
    FROM file_tags
    GROUP BY file_id
) t;
EOF

echo ""
echo "════════════════════════════════════════════════════════════════════"
echo "Next Steps:"
echo "  1. Launch GUI: make dev-pipeline"
echo "  2. Browse at: http://localhost:5173"
echo "  3. Search your 4.3M file library"
echo "  4. Open in DAW: make dev-daw (http://localhost:5174)"
echo ""
echo "Logs saved:"
echo "  • Import: /tmp/import_log.txt"
echo "  • Analysis: /tmp/analyze_log.txt"
echo "════════════════════════════════════════════════════════════════════"

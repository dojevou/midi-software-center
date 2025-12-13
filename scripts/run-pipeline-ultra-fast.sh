#!/bin/bash
# Ultra-fast pipeline for 4.3M files
# Optimized for 16 cores, 60GB RAM

set -e

SOURCE_DIR="/home/dojevou/Uncontaminated/floorp_downloads/midi"
PROJECT_DIR="/home/dojevou/projects/midi-software-center"
LIBRARY_DIR="$PROJECT_DIR/midi-library"

# Kill existing pipeline if running
pkill -f "run-pipeline" || true
pkill -f "cargo run.*import" || true
pkill -f "cargo run.*analyze" || true

echo "╔════════════════════════════════════════════════════════════════════╗"
echo "║      ULTRA-FAST Pipeline - 16 cores, 60GB RAM (4.3M files)        ║"
echo "╚════════════════════════════════════════════════════════════════════╝"
echo ""
echo "Optimizations:"
echo "  • 16 worker threads (full CPU utilization)"
echo "  • 1000 batch size (large batches for DB efficiency)"
echo "  • Parallel import + analysis (pipeline stages overlap)"
echo "  • Skip detailed analysis for simple files"
echo ""

# Ensure database is running
echo "[1/4] Checking database..."
if ! docker ps | grep -q postgres; then
    cd "$PROJECT_DIR"
    make docker-up
    sleep 5
fi
echo "  ✓ Database running"
echo ""

echo "[2/4] Building optimized binaries..."
cd "$PROJECT_DIR/pipeline/src-tauri"
if [ ! -f "../../target/release/import" ]; then
    echo "  Compiling with maximum optimizations..."
    RUSTFLAGS="-C target-cpu=native -C opt-level=3" \
        cargo build --release --bins
fi
echo "  ✓ Binaries ready"
echo ""

echo "════════════════════════════════════════════════════════════════════"
echo "[3/4] IMPORT PHASE - ULTRA FAST MODE"
echo "════════════════════════════════════════════════════════════════════"
echo ""
echo "Performance targets:"
echo "  • Threads: 16 (was 8) = 2x faster"
echo "  • Batch size: 1000 (was 500) = 2x faster DB writes"
echo "  • Expected: 7,830 files/sec (2x improvement)"
echo "  • Time: 4,314,593 ÷ 7,830 = 551 seconds (~9 minutes)"
echo ""

START_TIME=$(date +%s)
echo $START_TIME > /tmp/pipeline_start_time

# Ultra-fast import
DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library" \
    cargo run --release --bin import -- \
    --input "$SOURCE_DIR" \
    --recursive \
    --threads 16 \
    --batch-size 1000 \
    2>&1 | tee /tmp/import_log.txt

IMPORT_END=$(date +%s)
IMPORT_DURATION=$((IMPORT_END - START_TIME))
FILES_IMPORTED=$(psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -t -c "SELECT COUNT(*) FROM files;")
IMPORT_RATE=$(echo "scale=2; $FILES_IMPORTED / $IMPORT_DURATION" | bc)

echo ""
echo "✓ Import complete!"
echo "  Duration: $IMPORT_DURATION seconds"
echo "  Files: $FILES_IMPORTED"
echo "  Rate: $IMPORT_RATE files/sec"
echo ""

echo "════════════════════════════════════════════════════════════════════"
echo "[4/4] ANALYSIS PHASE - ULTRA FAST MODE"
echo "════════════════════════════════════════════════════════════════════"
echo ""
echo "Optimizations:"
echo "  • 16 threads (was 8) = 2x parallelism"
echo "  • 200 batch size (optimized for analysis)"
echo "  • Expected: 181 files/sec (2x improvement)"
echo "  • Time: 4,314,593 ÷ 181 = 23,839 seconds (~6.6 hours)"
echo ""
echo "Monitor: tail -f /tmp/analyze_log.txt"
echo ""

ANALYZE_START=$(date +%s)

# Ultra-fast analysis
DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library" \
    RAYON_NUM_THREADS=16 \
    cargo run --release --bin analyze -- \
    --batch-size 200 \
    --threads 16 \
    --parallel \
    2>&1 | tee /tmp/analyze_log.txt

ANALYZE_END=$(date +%s)
ANALYZE_DURATION=$((ANALYZE_END - ANALYZE_START))
TOTAL_DURATION=$((ANALYZE_END - START_TIME))

echo ""
echo "✓ Analysis complete!"
echo "  Duration: $ANALYZE_DURATION seconds"
echo ""

echo "════════════════════════════════════════════════════════════════════"
echo "PIPELINE COMPLETE ✓"
echo "════════════════════════════════════════════════════════════════════"
echo ""
echo "Timing Summary:"
echo "  • Import:   $IMPORT_DURATION sec (~$(echo "scale=1; $IMPORT_DURATION/60" | bc) min)"
echo "  • Analysis: $ANALYZE_DURATION sec (~$(echo "scale=1; $ANALYZE_DURATION/3600" | bc) hours)"
echo "  • Total:    $TOTAL_DURATION sec (~$(echo "scale=1; $TOTAL_DURATION/3600" | bc) hours)"
echo ""
echo "Expected speedup:"
echo "  • Old estimate: 13.5 hours"
echo "  • New estimate: 7 hours (1.9x faster)"
echo ""

# Statistics
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" <<EOF
SELECT
    'Total Files:       ' || COUNT(*)::text
FROM files;

SELECT
    'Analyzed Files:    ' || COUNT(*)::text
FROM file_metadata WHERE bpm IS NOT NULL;

SELECT
    'Unique Tags:       ' || COUNT(DISTINCT tag_name)::text
FROM tags;

SELECT
    'Tag Associations:  ' || COUNT(*)::text
FROM file_tags;

SELECT
    'Database Size:     ' || pg_size_pretty(pg_database_size('midi_library'))
FROM (SELECT 1) t;
EOF

echo ""
echo "════════════════════════════════════════════════════════════════════"
echo "Next: Launch GUI with 'make dev-pipeline'"
echo "════════════════════════════════════════════════════════════════════"

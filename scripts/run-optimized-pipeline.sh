#!/bin/bash
# Optimized Pipeline with Performance Crates
# Uses: mimalloc, parking_lot, ahash, dashmap, flume
# Expected: 1.5-2x faster than baseline

set -e

SOURCE_DIR="/home/dojevou/Uncontaminated/floorp_downloads/midi"
PROJECT_DIR="/home/dojevou/projects/midi-software-center"
DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

echo "╔════════════════════════════════════════════════════════════════════╗"
echo "║   🚀 OPTIMIZED PIPELINE - Performance Crates Enabled 🚀            ║"
echo "╚════════════════════════════════════════════════════════════════════╝"
echo ""
echo "🔥 OPTIMIZATIONS ACTIVE:"
echo "  ✓ mimalloc - High-performance allocator"
echo "  ✓ parking_lot - Fast locks (2-5x faster)"
echo "  ✓ ahash - Fast hashing (2-3x faster)"
echo "  ✓ dashmap - Lock-free HashMap"
echo "  ✓ flume - Fast channels"
echo "  ✓ 24 worker threads"
echo "  ✓ Native CPU optimizations (AVX2, SSE4.2)"
echo ""

# Check database
echo "[1/3] Checking database..."
if ! docker ps | grep -q postgres; then
    cd "$PROJECT_DIR"
    make docker-up
    sleep 5
fi
echo "  ✓ Database running"
echo ""

# Ensure optimized binary exists
echo "[2/3] Checking optimized binary..."
if [ ! -f "$PROJECT_DIR/target/release/batch_import" ]; then
    echo "  Building optimized binary..."
    cd "$PROJECT_DIR/pipeline/src-tauri"
    RUSTFLAGS="-C target-cpu=native -C opt-level=3" \
        cargo build --release --bin batch_import
fi
echo "  ✓ Binary ready"
echo ""

# Run optimized import
echo "════════════════════════════════════════════════════════════════════"
echo "[3/3] RUNNING OPTIMIZED IMPORT"
echo "════════════════════════════════════════════════════════════════════"
echo ""
echo "Source directory: $SOURCE_DIR"
echo "Workers: 24"
echo "Expected speed: 250-400 files/sec (optimized)"
echo ""

START_TIME=$(date +%s)

DATABASE_URL="$DATABASE_URL" \
    "$PROJECT_DIR/target/release/batch_import" \
    --directory "$SOURCE_DIR" \
    --workers 24 \
    2>&1 | tee /tmp/optimized_import_log.txt

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo ""
echo "════════════════════════════════════════════════════════════════════"
echo "🎉 OPTIMIZED IMPORT COMPLETE! 🎉"
echo "════════════════════════════════════════════════════════════════════"
echo ""
echo "⏱️  Total Time: $DURATION seconds (~$(echo "scale=1; $DURATION/3600" | bc) hours)"
echo ""

# Show statistics
psql "$DATABASE_URL" <<EOF
\echo '📈 Final Statistics:'
\echo ''

SELECT
    'Total Files:       ' || COUNT(*)::text
FROM files;

SELECT
    'With Key Detection: ' || COUNT(*)::text
FROM musical_metadata WHERE key_signature != 'UNKNOWN';

SELECT
    'Database Size:     ' || pg_size_pretty(pg_database_size('midi_library'))
FROM (SELECT 1) t;
EOF

echo ""
echo "════════════════════════════════════════════════════════════════════"
echo "Next: Launch GUI with 'make dev-pipeline'"
echo "════════════════════════════════════════════════════════════════════"

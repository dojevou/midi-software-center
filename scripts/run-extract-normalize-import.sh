#!/bin/bash
# 🚀 COMPLETE MIDI IMPORT PIPELINE
# Extract → Normalize → Import
#
# This script runs the full workflow:
# 1. Extract archives using parallel_extract (ultra-fast)
# 2. Normalize filenames using normalize_filenames (324K files/sec)
# 3. Import & analyze using import_unified (300+ files/sec)

set -e

# Configuration
ARCHIVE_DIR="${1:-/media/dojevou/NewSSD2/midi}"
EXTRACT_DIR="${2:-/home/dojevou/tmp/midi_bulk_extract}"
WORKERS="${3:-64}"
DATABASE_URL="${DATABASE_URL:-postgresql://midiuser:145278963@localhost:5433/midi_library}"

echo "🎵 COMPLETE MIDI IMPORT PIPELINE"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "📂 Archive directory: $ARCHIVE_DIR"
echo "📁 Extract directory: $EXTRACT_DIR"
echo "⚡ Workers:           $WORKERS"
echo "🗄️  Database:          $DATABASE_URL"
echo ""

# Check binaries exist
if [ ! -f "./target/release/parallel_extract" ]; then
    echo "❌ parallel_extract binary not found"
    exit 1
fi

if [ ! -f "./target/release/normalize_filenames" ]; then
    echo "❌ normalize_filenames binary not found"
    exit 1
fi

if [ ! -f "./target/release/import_unified" ]; then
    echo "❌ import_unified binary not found"
    exit 1
fi

#═══════════════════════════════════════════════════════════════
# PHASE 1: EXTRACT ALL ARCHIVES
#═══════════════════════════════════════════════════════════════
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📦 PHASE 1: EXTRACT ARCHIVES"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Create fresh extract directory
rm -rf "$EXTRACT_DIR"
mkdir -p "$EXTRACT_DIR"

START_EXTRACT=$(date +%s)

echo "⚡ Running parallel extraction..."
./target/release/parallel_extract --archive-dir "$ARCHIVE_DIR" --output-dir "$EXTRACT_DIR" --max-parallel "$WORKERS"

END_EXTRACT=$(date +%s)
EXTRACT_TIME=$((END_EXTRACT - START_EXTRACT))

echo ""
echo "✓ Extraction complete in ${EXTRACT_TIME}s"
echo ""

#═══════════════════════════════════════════════════════════════
# PHASE 2: NORMALIZE FILENAMES
#═══════════════════════════════════════════════════════════════
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧹 PHASE 2: NORMALIZE FILENAMES"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

START_NORMALIZE=$(date +%s)

echo "⚡ Running filename normalization..."
./target/release/normalize_filenames "$EXTRACT_DIR" "$WORKERS"

END_NORMALIZE=$(date +%s)
NORMALIZE_TIME=$((END_NORMALIZE - START_NORMALIZE))

echo ""
echo "✓ Normalization complete in ${NORMALIZE_TIME}s"
echo ""

#═══════════════════════════════════════════════════════════════
# PHASE 3: IMPORT & ANALYZE
#═══════════════════════════════════════════════════════════════
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎵 PHASE 3: IMPORT & ANALYZE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

START_IMPORT=$(date +%s)

echo "⚡ Starting import with full analysis..."
export DATABASE_URL
./target/release/batch_import --directory "$EXTRACT_DIR" --workers "$WORKERS"

END_IMPORT=$(date +%s)
IMPORT_TIME=$((END_IMPORT - START_IMPORT))

#═══════════════════════════════════════════════════════════════
# SUMMARY
#═══════════════════════════════════════════════════════════════
TOTAL_TIME=$((END_IMPORT - START_EXTRACT))

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "✅ PIPELINE COMPLETE!"
echo ""
echo "⏱️  Timing:"
echo "   Phase 1 (Extract):    ${EXTRACT_TIME}s"
echo "   Phase 2 (Normalize):  ${NORMALIZE_TIME}s"
echo "   Phase 3 (Import):     ${IMPORT_TIME}s"
echo "   ─────────────────────────────"
echo "   Total:                ${TOTAL_TIME}s"
echo ""
echo "📊 Performance:"
# Get final file count from database
FILE_COUNT=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM files;" 2>/dev/null | tr -d ' ' || echo "N/A")
if [ "$FILE_COUNT" != "N/A" ] && [ "$FILE_COUNT" -gt 0 ]; then
    AVG_SPEED=$((FILE_COUNT / TOTAL_TIME))
    echo "   Files imported:       $FILE_COUNT"
    echo "   Average speed:        ${AVG_SPEED} files/sec"
fi
echo ""
echo "✓ Extract directory: $EXTRACT_DIR"
echo ""

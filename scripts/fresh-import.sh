#!/bin/bash
# =============================================================================
# Fresh Database Import - Reset and Import Deduplicated Files
# =============================================================================
# Purpose: Clear database and reimport 1.72M unique files with organization
# Uses: import_unified (has built-in BLAKE3 deduplication)
# Date: Nov 22, 2025
# =============================================================================

set -e  # Exit on error

DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"
MIDI_DIR="/home/dojevou/projects/midi-software-center/midi-library"
PROJECT_DIR="/home/dojevou/projects/midi-software-center"

echo "════════════════════════════════════════════════════════════"
echo "  MIDI Library - Fresh Database Import"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "⚠️  This will:"
echo "   1. DELETE all existing database records"
echo "   2. Reimport 1.72M deduplicated files"
echo "   3. Apply instrument organization (97 tags)"
echo ""
echo "Current database:"
psql "$DB_URL" -c "SELECT COUNT(*) as current_files FROM files;"
echo ""
echo "Files on disk: ~1.72M (after deduplication)"
echo ""
read -p "Continue? Type 'yes' to proceed: " confirm

if [ "$confirm" != "yes" ]; then
    echo "❌ Aborted"
    exit 1
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Step 1/4: Clearing Database"
echo "════════════════════════════════════════════════════════════"
echo ""

psql "$DB_URL" <<SQL
-- Temporarily disable triggers for faster deletion
SET session_replication_role = replica;

-- Clear tables in correct dependency order
TRUNCATE TABLE chords CASCADE;
TRUNCATE TABLE drum_patterns CASCADE;
TRUNCATE TABLE midi_events CASCADE;
TRUNCATE TABLE midi_tracks CASCADE;
TRUNCATE TABLE search_index CASCADE;
TRUNCATE TABLE file_tags CASCADE;
TRUNCATE TABLE tags CASCADE;
TRUNCATE TABLE analysis_results CASCADE;
TRUNCATE TABLE musical_metadata CASCADE;
TRUNCATE TABLE files CASCADE;
TRUNCATE TABLE import_batches CASCADE;

-- Re-enable triggers
SET session_replication_role = DEFAULT;

-- Verify cleanup
SELECT COUNT(*) as files_remaining FROM files;
SQL

echo "✅ Database cleared"
echo ""

echo "════════════════════════════════════════════════════════════"
echo "  Step 2/4: Building Import Tool"
echo "════════════════════════════════════════════════════════════"
echo ""

cd "$PROJECT_DIR"
echo "Building import_unified binary (includes BLAKE3 deduplication)..."
cargo build --release --bin import_unified

echo "✅ Build complete"
echo ""

echo "════════════════════════════════════════════════════════════"
echo "  Step 3/4: Importing Files"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Importing from: $MIDI_DIR"
echo "Expected time: 1-2 hours (with full analysis)"
echo ""
echo "Starting import..."
echo ""

# Run import_unified - it will:
# - Process all MIDI files recursively
# - Hash each file with BLAKE3
# - Skip duplicates automatically
# - Analyze BPM, key, tags
# - Insert to database
"$PROJECT_DIR/target/release/import_unified" "$MIDI_DIR"

echo ""
echo "✅ Import complete"
echo ""

echo "════════════════════════════════════════════════════════════"
echo "  Step 4/4: Applying Database Organization"
echo "════════════════════════════════════════════════════════════"
echo ""

if [ -f "$PROJECT_DIR/scripts/organize-database.sh" ]; then
    echo "Running database organization (97 instrument tags)..."
    "$PROJECT_DIR/scripts/organize-database.sh"
else
    echo "⚠️  organize-database.sh not found, applying SQL directly..."
    psql "$DB_URL" -f "$PROJECT_DIR/database/organize_by_instruments.sql"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  ✅ Fresh Import Complete!"
echo "════════════════════════════════════════════════════════════"
echo ""

# Final statistics
psql "$DB_URL" <<SQL
SELECT
    '📊 FINAL STATISTICS' as section,
    '' as value
UNION ALL
SELECT 'Total files', COUNT(*)::text FROM files
UNION ALL
SELECT 'With BPM/key metadata', COUNT(*)::text FROM musical_metadata
UNION ALL
SELECT 'Instrument tags', COUNT(*)::text FROM tags
UNION ALL
SELECT 'File-tag associations', COUNT(*)::text FROM file_tags
UNION ALL
SELECT 'MIDI tracks', COUNT(*)::text FROM midi_tracks;
SQL

echo ""
echo "Test queries:"
echo "  psql \"$DB_URL\" -c \"SELECT * FROM v_drums LIMIT 10;\""
echo "  psql \"$DB_URL\" -c \"SELECT * FROM get_files_by_instrument('ride');\""
echo ""

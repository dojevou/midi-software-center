#!/bin/bash
# =============================================================================
# Reset Database and Reimport Deduplicated Files
# =============================================================================
# Purpose: Clear database and reimport only the 1.72M unique files
# Date: Nov 22, 2025
# =============================================================================

set -e  # Exit on error

DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"
MIDI_DIR="/home/dojevou/projects/midi-software-center/midi-library"

echo "════════════════════════════════════════════════════════════"
echo "  MIDI Library - Database Reset & Fresh Import"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "⚠️  WARNING: This will DELETE all existing database records!"
echo "   Schema will be preserved, but all file data will be removed."
echo ""
echo "Current database stats:"
psql "$DB_URL" -c "SELECT COUNT(*) as total_files FROM files;"
echo ""
echo "Files on disk (after deduplication): ~1.72M"
echo ""
read -p "Continue? (yes/no): " confirm

if [ "$confirm" != "yes" ]; then
    echo "❌ Aborted by user"
    exit 1
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Step 1: Clearing Database Tables"
echo "════════════════════════════════════════════════════════════"
echo ""

# Clear all tables in correct order (respecting foreign key constraints)
psql "$DB_URL" <<SQL
-- Disable triggers temporarily for faster deletion
SET session_replication_role = replica;

-- Clear tables in dependency order
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

-- Verify tables are empty
SELECT
    'files' as table_name, COUNT(*) as row_count FROM files
UNION ALL
SELECT 'musical_metadata', COUNT(*) FROM musical_metadata
UNION ALL
SELECT 'tags', COUNT(*) FROM tags
UNION ALL
SELECT 'file_tags', COUNT(*) FROM file_tags
UNION ALL
SELECT 'midi_tracks', COUNT(*) FROM midi_tracks;
SQL

echo "✅ Database cleared successfully"
echo ""

echo "════════════════════════════════════════════════════════════"
echo "  Step 2: Importing Deduplicated Files"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Starting import of 1.72M unique files..."
echo "This will take approximately 3-5 minutes for import + 2-3 hours for analysis"
echo ""

# Check if import tool exists
if [ ! -f "../target/release/import" ]; then
    echo "Building import tool..."
    cd /home/dojevou/projects/midi-software-center
    cargo build --release --bin import
fi

# Run import on the midi-library directory
# This will hash files and skip duplicates automatically
echo "Running import (files will be hashed to prevent duplicates)..."
cd /home/dojevou/projects/midi-software-center

# Import archives directory
if [ -d "$MIDI_DIR/archives" ]; then
    echo "Importing from archives/..."
    ./target/release/import "$MIDI_DIR/archives" --batch-size 1000
fi

# Import extracted directory
if [ -d "$MIDI_DIR/extracted" ]; then
    echo "Importing from extracted/..."
    ./target/release/import "$MIDI_DIR/extracted" --batch-size 1000
fi

# Import splits directory
if [ -d "$MIDI_DIR/splits" ]; then
    echo "Importing from splits/..."
    ./target/release/import "$MIDI_DIR/splits" --batch-size 1000
fi

# Import repaired directory
if [ -d "$MIDI_DIR/repaired" ]; then
    echo "Importing from repaired/..."
    ./target/release/import "$MIDI_DIR/repaired" --batch-size 1000
fi

echo ""
echo "✅ Import complete!"
echo ""

echo "════════════════════════════════════════════════════════════"
echo "  Step 3: Final Statistics"
echo "════════════════════════════════════════════════════════════"
echo ""

psql "$DB_URL" -c "
SELECT
    (SELECT COUNT(*) FROM files) as total_files,
    (SELECT COUNT(*) FROM musical_metadata) as files_with_metadata,
    (SELECT COUNT(*) FROM tags) as total_tags,
    (SELECT COUNT(*) FROM file_tags) as total_file_tags;
"

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  ✅ Database Reset & Import Complete!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Next steps:"
echo "  1. Run: ./scripts/organize-database.sh"
echo "  2. Run analysis: cargo run --release --bin analyze $MIDI_DIR"
echo ""

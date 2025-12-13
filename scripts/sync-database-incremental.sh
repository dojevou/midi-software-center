#!/bin/bash
set -e

# Incremental Database Sync - Import Only New Files
# Preserves existing analysis, only imports what's missing

DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"
LIBRARY_PATH="/home/dojevou/projects/midi-software-center/midi-library"
LOG_FILE="/tmp/db_sync_$(date +%Y%m%d_%H%M%S).log"

echo "🔄 Incremental Database Sync"
echo "============================"
echo ""
echo "Strategy: Import only files NOT in database"
echo "Preserve: All existing analysis data"
echo ""

# Phase 1: Check current state
echo "📊 Phase 1: Current Database State"
echo "==================================="
echo ""

psql "$DB_URL" << 'EOSQL' | tee -a "$LOG_FILE"
-- Current statistics
SELECT
    COUNT(*) as total_in_db,
    COUNT(*) FILTER (WHERE analyzed_at IS NOT NULL) as analyzed,
    COUNT(*) FILTER (WHERE analyzed_at IS NULL) as unanalyzed,
    COUNT(DISTINCT blake3_hash) as unique_files
FROM files;

-- Files with full metadata
SELECT COUNT(*) as files_with_metadata
FROM files f
WHERE EXISTS (
    SELECT 1 FROM musical_metadata m WHERE m.file_id = f.id
);
EOSQL

echo ""
echo "📂 Phase 2: Count Files on Disk"
echo "================================"
echo ""

echo "Counting MIDI files in library..."
TOTAL_ON_DISK=$(find "$LIBRARY_PATH" -type f \( -iname "*.mid" -o -iname "*.midi" \) 2>/dev/null | wc -l)
echo "Total MIDI files on disk: $TOTAL_ON_DISK"
echo ""

# Phase 3: Incremental Import
echo "📥 Phase 3: Import Only New Files"
echo "=================================="
echo ""
echo "This will:"
echo "  - Check each file's hash against database"
echo "  - Import only files not already in database"
echo "  - Skip files that already exist (by hash)"
echo "  - Preserve all existing analysis data"
echo ""

read -p "Continue with incremental import? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Import cancelled"
    exit 0
fi

# Import archives directory (if not already fully imported)
echo "Importing from archives/..."
cd /home/dojevou/projects/midi-software-center
./target/release/import \
    --source "$LIBRARY_PATH/archives" \
    2>&1 | tee -a "$LOG_FILE"

# Import extracted directory
echo "Importing from extracted/..."
./target/release/import \
    --source "$LIBRARY_PATH/extracted" \
    2>&1 | tee -a "$LOG_FILE"

# Import splits directory
echo "Importing from splits/..."
./target/release/import \
    --source "$LIBRARY_PATH/splits" \
    2>&1 | tee -a "$LOG_FILE"

echo ""
echo "✅ Import complete!"
echo ""

# Phase 4: Verify results
echo "📊 Phase 4: Post-Import Statistics"
echo "==================================="
echo ""

psql "$DB_URL" << 'EOSQL' | tee -a "$LOG_FILE"
-- New statistics
SELECT
    COUNT(*) as total_in_db,
    COUNT(*) FILTER (WHERE analyzed_at IS NOT NULL) as analyzed,
    COUNT(*) FILTER (WHERE analyzed_at IS NULL) as unanalyzed,
    COUNT(DISTINCT blake3_hash) as unique_files
FROM files;

-- Breakdown by directory
SELECT
    CASE
        WHEN filepath LIKE '%/archives/%' THEN 'archives'
        WHEN filepath LIKE '%/extracted/%' THEN 'extracted'
        WHEN filepath LIKE '%/splits/%' THEN 'splits'
        ELSE 'other'
    END as directory,
    COUNT(*) as file_count
FROM files
GROUP BY directory
ORDER BY file_count DESC;
EOSQL

echo ""
echo "🎉 Sync Complete!"
echo "================="
echo ""
echo "Summary:"
echo "  - Files on disk: $TOTAL_ON_DISK"
echo "  - Check database for import count above"
echo "  - Log file: $LOG_FILE"
echo ""
echo "Next steps:"
echo "  1. Run analysis on unanalyzed files: ./target/release/analyze"
echo "  2. Organize by instrument: ./scripts/organize-by-instrument.sh"
echo ""

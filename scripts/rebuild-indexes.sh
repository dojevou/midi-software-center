#!/bin/bash
# Rebuild all indexes after ultra-fast import completes

set -e

echo "🔨 REBUILDING INDEXES"
echo "═══════════════════════════════════════════════════════════════"
echo ""

echo "Step 1: Restoring normal PostgreSQL settings..."
docker exec midi-library-postgres psql -U midiuser -d midi_library <<'SQL'
SET synchronous_commit = on;
SET fsync = on;
SET full_page_writes = on;
SELECT 'Normal settings restored' as status;
SQL

echo "✓ Settings restored"
echo ""

echo "Step 2: Rebuilding indexes in parallel..."
docker exec midi-library-postgres psql -U midiuser -d midi_library < database/INDEX_BACKUP.sql

echo "✓ All indexes rebuilt"
echo ""

echo "Step 3: Running ANALYZE to update statistics..."
docker exec midi-library-postgres psql -U midiuser -d midi_library -c "ANALYZE files; ANALYZE musical_metadata;"

echo "✓ Statistics updated"
echo ""
echo "🎉 Index rebuild complete!"

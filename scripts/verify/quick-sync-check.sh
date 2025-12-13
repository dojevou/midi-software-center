#!/bin/bash
#
# Quick Database Sync Check
#
# Fast verification that database is accessible and has expected tables.
# Use this for CI pipelines and quick health checks.
#
# Exit codes:
#   0 - All OK
#   1 - Database connection failed
#   2 - Missing required tables
#   3 - Data integrity issues

set -e

# Database URL
DB_URL="${DATABASE_URL:-postgresql://midiuser:145278963@localhost:5433/midi_library}"

# Required tables
REQUIRED_TABLES=("files" "musical_metadata" "tags" "file_tags")

# Check connection
if ! psql "$DB_URL" -c "SELECT 1" &>/dev/null; then
    echo "FAIL: Cannot connect to database"
    exit 1
fi

# Check tables
MISSING=0
for table in "${REQUIRED_TABLES[@]}"; do
    EXISTS=$(psql "$DB_URL" -t -c "
        SELECT COUNT(*) FROM information_schema.tables
        WHERE table_schema = 'public' AND table_name = '$table'
    " 2>/dev/null | tr -d ' ')

    if [ "$EXISTS" -ne 1 ]; then
        echo "MISSING: $table"
        MISSING=$((MISSING + 1))
    fi
done

if [ "$MISSING" -gt 0 ]; then
    echo "FAIL: $MISSING table(s) missing"
    exit 2
fi

# Quick integrity check
ORPHANS=$(psql "$DB_URL" -t -c "
    SELECT COUNT(*) FROM file_tags ft
    WHERE NOT EXISTS (SELECT 1 FROM files f WHERE f.id = ft.file_id)
" 2>/dev/null | tr -d ' ')

if [ "$ORPHANS" -gt 0 ]; then
    echo "WARNING: $ORPHANS orphan records"
    # Don't fail on orphans, just warn
fi

# Get stats
FILE_COUNT=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM files" 2>/dev/null | tr -d ' ')
TAG_COUNT=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM tags" 2>/dev/null | tr -d ' ')

echo "OK: files=$FILE_COUNT tags=$TAG_COUNT"
exit 0

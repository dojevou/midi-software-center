#!/bin/bash
# Manual Test Script for VIP3 Filter Counts
# Tests Stream A functionality using direct database queries

set -e

DB_URL="${DATABASE_URL:-postgresql://midiuser:145278963@localhost:5433/midi_library}"

echo "========================================"
echo "VIP3 Filter Counts Manual Test"
echo "========================================"
echo ""
echo "Database: $DB_URL"
echo ""

# Test 1: Check if we have files
echo "Test 1: Check database has files"
echo "--------------------------------------"
FILE_COUNT=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM files;")
echo "Total files: $FILE_COUNT"
if [ "$FILE_COUNT" -gt 0 ]; then
    echo "✓ PASS: Files exist in database"
else
    echo "✗ FAIL: No files in database"
    exit 1
fi
echo ""

# Test 2: Check if we have tags (instruments)
echo "Test 2: Check instrument tags exist"
echo "--------------------------------------"
INSTRUMENT_COUNT=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM tags WHERE category = 'instrument';")
echo "Instrument tags: $INSTRUMENT_COUNT"
if [ "$INSTRUMENT_COUNT" -gt 0 ]; then
    echo "✓ PASS: Instrument tags exist"
else
    echo "✗ FAIL: No instrument tags"
fi
echo ""

# Test 3: Get top 5 instruments by file count
echo "Test 3: Top 5 instruments by file count"
echo "--------------------------------------"
psql "$DB_URL" -c "
SELECT
    t.name as instrument,
    COUNT(DISTINCT ft.file_id) as file_count
FROM tags t
INNER JOIN file_tags ft ON t.id = ft.tag_id
WHERE t.category = 'instrument'
GROUP BY t.id, t.name
ORDER BY file_count DESC
LIMIT 5;
"
echo ""

# Test 4: Performance test - measure query time
echo "Test 4: Performance test (query execution time)"
echo "--------------------------------------"
echo "Running filter count query 5 times..."
for i in {1..5}; do
    START=$(date +%s%3N)
    psql "$DB_URL" -t -c "
    SELECT COUNT(DISTINCT f.id)
    FROM files f
    INNER JOIN file_tags ft ON f.id = ft.file_id
    INNER JOIN tags t ON ft.tag_id = t.id
    WHERE t.category = 'instrument';
    " > /dev/null
    END=$(date +%s%3N)
    DURATION=$((END - START))
    echo "  Run $i: ${DURATION}ms"
done
echo ""

echo "========================================"
echo "Manual test completed!"
echo "========================================"

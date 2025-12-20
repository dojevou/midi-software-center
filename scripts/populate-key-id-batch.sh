#!/bin/bash
# Populate key_id column in files table using batch updates
# Each batch updates 50K rows to avoid timeout issues

set -e

DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"
BATCH_SIZE=50000
MAX_BATCHES=50  # Safety limit

echo "=== Starting key_id population ==="
echo "Batch size: ${BATCH_SIZE}"
echo ""

# Get initial count
INITIAL_COUNT=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM files WHERE key_id IS NOT NULL;")
echo "Initial files with key_id: ${INITIAL_COUNT}"
echo ""

# Create temp table once (it will persist for this session)
echo "Creating temp table with key mappings..."
psql "$DB_URL" -c "
DROP TABLE IF EXISTS key_mappings;
CREATE TEMP TABLE key_mappings AS
SELECT
  mm.file_id,
  mk.id as key_id
FROM musical_metadata mm
JOIN musical_keys mk ON mm.key_signature::TEXT = mk.name
WHERE mm.key_signature IS NOT NULL;

CREATE INDEX idx_key_mappings_file_id ON key_mappings(file_id);
" > /dev/null

echo "Temp table created with $(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM key_mappings;") mappings"
echo ""

# Run batches
for i in $(seq 1 $MAX_BATCHES); do
    echo "--- Batch $i ---"

    # Run batch update
    UPDATED=$(psql "$DB_URL" -t -c "
    WITH batch AS (
        SELECT f.id, km.key_id
        FROM files f
        JOIN key_mappings km ON f.id = km.file_id
        WHERE f.key_id IS NULL
        LIMIT ${BATCH_SIZE}
    )
    UPDATE files f
    SET key_id = b.key_id
    FROM batch b
    WHERE f.id = b.id
    RETURNING 1;
    " | wc -l)

    # Get current count
    CURRENT_COUNT=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM files WHERE key_id IS NOT NULL;")
    PERCENTAGE=$(psql "$DB_URL" -t -c "SELECT ROUND(100.0 * COUNT(*) / (SELECT COUNT(*) FROM files), 2) FROM files WHERE key_id IS NOT NULL;")

    echo "Updated: ${UPDATED} rows"
    echo "Total files with key_id: ${CURRENT_COUNT} (${PERCENTAGE}%)"
    echo ""

    # Check if we're done
    if [ "$UPDATED" -lt "$BATCH_SIZE" ]; then
        echo "=== Population complete! ==="
        echo "Final count: ${CURRENT_COUNT} files with key_id"
        exit 0
    fi
done

echo "WARNING: Reached maximum batch limit (${MAX_BATCHES})"
echo "Files still need updating. Run script again or increase MAX_BATCHES."

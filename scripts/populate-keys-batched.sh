#!/bin/bash
# Batched key_id population for large datasets
# Updates in chunks of 100,000 files at a time with progress reporting

set -e

DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"
BATCH_SIZE=100000

echo "======================================"
echo "Batched key_id Population"
echo "Batch size: $BATCH_SIZE files"
echo "======================================"

# Get total count
TOTAL=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM files WHERE key_id IS NULL;")
TOTAL=$(echo $TOTAL | tr -d ' ')
echo "Total files to update: $TOTAL"

if [ "$TOTAL" -eq 0 ]; then
    echo "All files already have key_id populated!"
    exit 0
fi

echo ""
echo "Starting batched updates..."
echo ""

UPDATED=0
BATCH_NUM=1

while true; do
    echo "[Batch $BATCH_NUM] Updating up to $BATCH_SIZE files..."

    # Update in batches using a CTE to limit the UPDATE
    ROWS=$(psql "$DB_URL" -t -c "
        WITH files_to_update AS (
            SELECT f.id, mm.key_signature
            FROM files f
            JOIN musical_metadata mm ON f.id = mm.file_id
            WHERE f.key_id IS NULL
              AND mm.key_signature IS NOT NULL
            LIMIT $BATCH_SIZE
        )
        UPDATE files f
        SET key_id = mk.id
        FROM files_to_update ftu
        JOIN musical_keys mk ON ftu.key_signature::TEXT = mk.name
        WHERE f.id = ftu.id;
    " 2>&1 | grep "UPDATE" | awk '{print $2}')

    if [ -z "$ROWS" ]; then
        ROWS=0
    fi

    UPDATED=$((UPDATED + ROWS))
    PCT=$(echo "scale=1; 100 * $UPDATED / $TOTAL" | bc)

    echo "[Batch $BATCH_NUM] Updated $ROWS files (Total: $UPDATED / $TOTAL = $PCT%)"

    # Exit if no more rows to update
    if [ "$ROWS" -lt "$BATCH_SIZE" ]; then
        echo ""
        echo "All batches complete!"
        break
    fi

    BATCH_NUM=$((BATCH_NUM + 1))
    echo ""
done

echo ""
echo "======================================"
echo "Final Statistics:"
echo "======================================"

psql "$DB_URL" -c "
SELECT
    COUNT(*) as total_files,
    COUNT(key_id) as files_with_key_id,
    ROUND(100.0 * COUNT(key_id) / COUNT(*), 1) as percent_complete
FROM files;
"

echo ""
echo "key_id population complete!"

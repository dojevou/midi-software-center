-- Populate key_id column using batch updates
-- This script runs all batches in a single database session

\timing on
\set BATCH_SIZE 50000

-- Get initial count
SELECT COUNT(*) as initial_count FROM files WHERE key_id IS NOT NULL;

-- Create temp table with key mappings
DROP TABLE IF EXISTS key_mappings;
CREATE TEMP TABLE key_mappings AS
SELECT
  mm.file_id,
  mk.id as key_id
FROM musical_metadata mm
JOIN musical_keys mk ON mm.key_signature::TEXT = mk.name
WHERE mm.key_signature IS NOT NULL;

CREATE INDEX idx_key_mappings_file_id ON key_mappings(file_id);

SELECT COUNT(*) as total_mappings FROM key_mappings;

-- Run batch updates (DO loop)
DO $$
DECLARE
    batch_num INTEGER := 1;
    rows_updated INTEGER;
    total_updated INTEGER := 0;
BEGIN
    LOOP
        -- Run batch update
        WITH batch AS (
            SELECT f.id, km.key_id
            FROM files f
            JOIN key_mappings km ON f.id = km.file_id
            WHERE f.key_id IS NULL
            LIMIT 50000
        )
        UPDATE files f
        SET key_id = b.key_id
        FROM batch b
        WHERE f.id = b.id;

        GET DIAGNOSTICS rows_updated = ROW_COUNT;
        total_updated := total_updated + rows_updated;

        RAISE NOTICE 'Batch %: Updated % rows (Total: %)', batch_num, rows_updated, total_updated;

        -- Exit if no more rows to update
        EXIT WHEN rows_updated = 0;

        batch_num := batch_num + 1;

        -- Safety limit
        EXIT WHEN batch_num > 50;
    END LOOP;

    RAISE NOTICE 'Population complete! Total rows updated: %', total_updated;
END $$;

-- Get final count
SELECT COUNT(*) as final_count FROM files WHERE key_id IS NOT NULL;
SELECT ROUND(100.0 * COUNT(*) / (SELECT COUNT(*) FROM files), 2) as percentage
FROM files WHERE key_id IS NOT NULL;

-- =============================================================================
-- Optimized Key ID Population
-- =============================================================================
-- Uses a two-step approach for better performance on large datasets
-- =============================================================================

\echo 'Optimized key_id population starting...'
\timing on

-- Step 1: Create temporary mapping table (fast - only joins once)
\echo 'Creating temporary key mappings...'

CREATE TEMPORARY TABLE temp_file_keys AS
SELECT
    f.id as file_id,
    mk.id as key_id
FROM files f
JOIN musical_metadata mm ON f.id = mm.file_id
JOIN musical_keys mk ON mm.key_signature::TEXT = mk.name
WHERE mm.key_signature IS NOT NULL
  AND f.key_id IS NULL;

\echo 'Temporary table created.'

-- Create index on temporary table for fast lookups
CREATE INDEX idx_temp_file_keys ON temp_file_keys(file_id);

\echo 'Index created on temporary table.'

-- Step 2: Update files from temporary table (fast - simple join on indexed column)
\echo 'Updating files.key_id from temporary table...'

UPDATE files f
SET key_id = tfk.key_id
FROM temp_file_keys tfk
WHERE f.id = tfk.file_id;

\echo 'key_id population complete!'

-- Verify results
SELECT COUNT(*) as files_with_key_id FROM files WHERE key_id IS NOT NULL;

-- Cleanup
DROP TABLE temp_file_keys;

\echo 'Done!'

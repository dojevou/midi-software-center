-- =============================================================================
-- Populate Denormalized Filter Columns
-- =============================================================================
-- Populates folder_id and key_id in the files table for VIP3 filtering
-- This script uses optimized queries for large datasets (2.1M files)
-- =============================================================================

\echo 'Starting denormalized column population...'
\timing on

-- =============================================================================
-- STEP 1: Populate key_id using efficient batch approach
-- =============================================================================

\echo ''
\echo 'Step 1: Populating key_id from musical_metadata.key_signature...'

-- Use a simpler join without the TEXT cast in WHERE clause
UPDATE files f
SET key_id = mk.id
FROM musical_metadata mm
JOIN musical_keys mk ON mm.key_signature::TEXT = mk.name
WHERE f.id = mm.file_id
  AND mm.key_signature IS NOT NULL
  AND f.key_id IS NULL;

\echo 'key_id population complete!'

-- =============================================================================
-- STEP 2: Populate folder_id from parent_folder
-- =============================================================================

\echo ''
\echo 'Step 2: Populating folder_id from parent_folder...'
\echo 'Note: folder_id will be set to a hash of parent_folder path'
\echo '(No folders table exists, so using parent_folder as folder identifier)'

-- Create a simple mapping: use hashtext(parent_folder) as folder_id
-- This gives us a numeric ID for each unique folder path
UPDATE files
SET folder_id = hashtext(parent_folder)::bigint
WHERE parent_folder IS NOT NULL
  AND folder_id IS NULL;

\echo 'folder_id population complete!'

-- =============================================================================
-- STEP 3: Verify results
-- =============================================================================

\echo ''
\echo '========================================='
\echo 'Population Summary:'
\echo '========================================='

SELECT
    COUNT(*) as total_files,
    COUNT(folder_id) as files_with_folder_id,
    COUNT(bpm_range_id) as files_with_bpm_range_id,
    COUNT(key_id) as files_with_key_id,
    COUNT(channel_count) as files_with_channel_count,
    ROUND(100.0 * COUNT(folder_id) / COUNT(*), 1) as pct_folder,
    ROUND(100.0 * COUNT(bpm_range_id) / COUNT(*), 1) as pct_bpm,
    ROUND(100.0 * COUNT(key_id) / COUNT(*), 1) as pct_key,
    ROUND(100.0 * COUNT(channel_count) / COUNT(*), 1) as pct_channel
FROM files;

-- =============================================================================
-- STEP 4: Update table statistics
-- =============================================================================

\echo ''
\echo 'Updating table statistics...'

VACUUM ANALYZE files;
VACUUM ANALYZE musical_metadata;

\echo ''
\echo '========================================='
\echo 'All denormalized columns populated!'
\echo '========================================='

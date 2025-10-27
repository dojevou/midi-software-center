-- =============================================================================
-- MIDI Software Center - Cleanup Test Fixtures
-- =============================================================================
-- Purpose: Remove all test fixture data from database
-- WARNING: This will delete ALL test data. Use only in test databases!
-- Usage: psql -U midiuser -d midi_library < cleanup_test_data.sql
-- =============================================================================

BEGIN;

-- =============================================================================
-- SAFETY CHECK
-- =============================================================================
-- Uncomment this block to add a database name check (recommended!)
/*
DO $$
DECLARE
    current_db TEXT;
BEGIN
    SELECT current_database() INTO current_db;

    -- Only allow cleanup in test databases
    IF current_db NOT LIKE '%test%' AND current_db != 'midi_library_test' THEN
        RAISE EXCEPTION 'Safety check failed: This script should only run on test databases. Current database: %', current_db;
    END IF;
END $$;
*/

-- =============================================================================
-- DELETE TEST DATA
-- =============================================================================
-- Note: CASCADE constraints will automatically clean up related records

RAISE NOTICE '=============================================================================';
RAISE NOTICE 'Cleaning up test fixture data...';
RAISE NOTICE '=============================================================================';

-- Delete processing jobs (and errors via CASCADE)
DELETE FROM processing_jobs
WHERE id IN (
    'a1b2c3d4-e5f6-4789-0123-456789abcdef',
    'b2c3d4e5-f678-4901-2345-6789abcdef12',
    'c3d4e5f6-7890-4123-4567-89abcdef1234'
);

RAISE NOTICE 'Deleted processing_jobs and related errors';

-- Delete duplicate groups (and duplicate_files via CASCADE)
DELETE FROM duplicate_groups
WHERE id IN (1);

RAISE NOTICE 'Deleted duplicate_groups and duplicate_files';

-- Delete tags (and file_tags via CASCADE)
DELETE FROM tags
WHERE id BETWEEN 1 AND 15;

RAISE NOTICE 'Deleted tags and file_tags relationships';

-- Reset tag sequence
SELECT setval('tags_id_seq', 1, false);

-- Delete files (CASCADE will clean up all related tables)
-- This will remove:
--   - musical_metadata
--   - file_categories
--   - file_instruments
--   - file_tags (already cleaned)
--   - file_embeddings
--   - file_compatibility
--   - rhythm_patterns
--   - harmonic_patterns
--   - melodic_patterns
--   - favorites
--   - track_splits
--   - duplicate_files (already cleaned)
DELETE FROM files
WHERE id BETWEEN 1 AND 15;

RAISE NOTICE 'Deleted files and all related records via CASCADE';

-- Reset file sequence
SELECT setval('files_id_seq', 1, false);

-- Reset duplicate_groups sequence
SELECT setval('duplicate_groups_id_seq', 1, false);

-- =============================================================================
-- VERIFY CLEANUP
-- =============================================================================

DO $$
DECLARE
    files_count INTEGER;
    metadata_count INTEGER;
    categories_count INTEGER;
    instruments_count INTEGER;
    tags_count INTEGER;
    file_tags_count INTEGER;
    favorites_count INTEGER;
    splits_count INTEGER;
    dup_groups_count INTEGER;
    dup_files_count INTEGER;
    embeddings_count INTEGER;
    compat_count INTEGER;
    rhythm_count INTEGER;
    harmonic_count INTEGER;
    melodic_count INTEGER;
    jobs_count INTEGER;
    errors_count INTEGER;
    total_count INTEGER;
BEGIN
    -- Count remaining records
    SELECT COUNT(*) INTO files_count FROM files;
    SELECT COUNT(*) INTO metadata_count FROM musical_metadata;
    SELECT COUNT(*) INTO categories_count FROM file_categories;
    SELECT COUNT(*) INTO instruments_count FROM file_instruments;
    SELECT COUNT(*) INTO tags_count FROM tags;
    SELECT COUNT(*) INTO file_tags_count FROM file_tags;
    SELECT COUNT(*) INTO favorites_count FROM favorites;
    SELECT COUNT(*) INTO splits_count FROM track_splits;
    SELECT COUNT(*) INTO dup_groups_count FROM duplicate_groups;
    SELECT COUNT(*) INTO dup_files_count FROM duplicate_files;
    SELECT COUNT(*) INTO embeddings_count FROM file_embeddings;
    SELECT COUNT(*) INTO compat_count FROM file_compatibility;
    SELECT COUNT(*) INTO rhythm_count FROM rhythm_patterns;
    SELECT COUNT(*) INTO harmonic_count FROM harmonic_patterns;
    SELECT COUNT(*) INTO melodic_count FROM melodic_patterns;
    SELECT COUNT(*) INTO jobs_count FROM processing_jobs WHERE id IN (
        'a1b2c3d4-e5f6-4789-0123-456789abcdef',
        'b2c3d4e5-f678-4901-2345-6789abcdef12',
        'c3d4e5f6-7890-4123-4567-89abcdef1234'
    );
    SELECT COUNT(*) INTO errors_count FROM processing_errors WHERE job_id IN (
        'a1b2c3d4-e5f6-4789-0123-456789abcdef',
        'b2c3d4e5-f678-4901-2345-6789abcdef12',
        'c3d4e5f6-7890-4123-4567-89abcdef1234'
    );

    total_count := files_count + metadata_count + categories_count + instruments_count +
                   tags_count + file_tags_count + favorites_count + splits_count +
                   dup_groups_count + dup_files_count + embeddings_count + compat_count +
                   rhythm_count + harmonic_count + melodic_count + jobs_count + errors_count;

    RAISE NOTICE '=============================================================================';
    RAISE NOTICE 'CLEANUP VERIFICATION';
    RAISE NOTICE '=============================================================================';
    RAISE NOTICE 'Table                    | Remaining Test Records';
    RAISE NOTICE '-------------------------|------------------------';
    RAISE NOTICE 'files                    | %', files_count;
    RAISE NOTICE 'musical_metadata         | %', metadata_count;
    RAISE NOTICE 'file_categories          | %', categories_count;
    RAISE NOTICE 'file_instruments         | %', instruments_count;
    RAISE NOTICE 'tags                     | %', tags_count;
    RAISE NOTICE 'file_tags                | %', file_tags_count;
    RAISE NOTICE 'favorites                | %', favorites_count;
    RAISE NOTICE 'track_splits             | %', splits_count;
    RAISE NOTICE 'duplicate_groups         | %', dup_groups_count;
    RAISE NOTICE 'duplicate_files          | %', dup_files_count;
    RAISE NOTICE 'file_embeddings          | %', embeddings_count;
    RAISE NOTICE 'file_compatibility       | %', compat_count;
    RAISE NOTICE 'rhythm_patterns          | %', rhythm_count;
    RAISE NOTICE 'harmonic_patterns        | %', harmonic_count;
    RAISE NOTICE 'melodic_patterns         | %', melodic_count;
    RAISE NOTICE 'processing_jobs          | %', jobs_count;
    RAISE NOTICE 'processing_errors        | %', errors_count;
    RAISE NOTICE '-------------------------|------------------------';
    RAISE NOTICE 'Total test records:      | %', total_count;
    RAISE NOTICE '=============================================================================';

    IF total_count = 0 THEN
        RAISE NOTICE 'SUCCESS: All test fixture data has been removed!';
    ELSE
        RAISE WARNING 'WARNING: % test records remain in the database', total_count;
    END IF;

    RAISE NOTICE '=============================================================================';
END $$;

COMMIT;

-- =============================================================================
-- VACUUM TABLES
-- =============================================================================
-- Reclaim space and update statistics after bulk delete

VACUUM ANALYZE files;
VACUUM ANALYZE musical_metadata;
VACUUM ANALYZE file_categories;
VACUUM ANALYZE file_instruments;
VACUUM ANALYZE tags;
VACUUM ANALYZE file_tags;
VACUUM ANALYZE favorites;
VACUUM ANALYZE track_splits;
VACUUM ANALYZE duplicate_groups;
VACUUM ANALYZE duplicate_files;
VACUUM ANALYZE file_embeddings;
VACUUM ANALYZE file_compatibility;
VACUUM ANALYZE rhythm_patterns;
VACUUM ANALYZE harmonic_patterns;
VACUUM ANALYZE melodic_patterns;
VACUUM ANALYZE processing_jobs;
VACUUM ANALYZE processing_errors;

-- =============================================================================
-- DONE
-- =============================================================================

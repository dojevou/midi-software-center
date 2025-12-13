-- Parallel instrument inference using PostgreSQL optimization
-- This version uses batch updates and parallel query execution

-- Enable parallel query execution
SET max_parallel_workers_per_gather = 8;
SET parallel_setup_cost = 100;
SET parallel_tuple_cost = 0.01;

-- Create indexes if they don't exist to speed up WHERE clauses
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_files_missing_instruments
ON files(id) WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_files_filepath_lower
ON files(LOWER(filepath));

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_files_filename_lower
ON files(LOWER(filename));

-- Start transaction for all updates
BEGIN;

-- Step 1: FILENAME-based inference (HIGHEST PRIORITY - most specific)
-- Using CASE statement in single UPDATE is much faster than multiple updates

UPDATE files
SET instrument_names_text = (
    CASE
        -- Drums (kick, snare, hat, etc.)
        WHEN filename ~* '(kick|snare|hat|hihat|cymbal|tom|clap|rim|cowbell|shaker|crash|ride)' THEN ARRAY['drums']

        -- Bass
        WHEN filename ~* '(^|[^a-z])(bass|sub|808)' THEN ARRAY['bass']

        -- Lead
        WHEN filename ~* '(^|[^a-z])(lead|melody)' THEN ARRAY['lead']

        -- Synth
        WHEN filename ~* '(^|[^a-z])synth' THEN ARRAY['synth']

        -- Pad
        WHEN filename ~* '(^|[^a-z])(pad|atmospher|ambient)' THEN ARRAY['pad']

        -- Pluck
        WHEN filename ~* '(^|[^a-z])(pluck|pizz)' THEN ARRAY['pluck']

        -- Keys/Piano
        WHEN filename ~* '(^|[^a-z])(key|keys|piano|keyboard|rhodes|organ)' THEN ARRAY['keys']

        -- Guitar
        WHEN filename ~* '(^|[^a-z])(guitar|gtr)' THEN ARRAY['guitar']

        -- Vocal
        WHEN filename ~* '(^|[^a-z])(vocal|voice|vox|choir)' THEN ARRAY['vocal']

        -- Strings
        WHEN filename ~* '(^|[^a-z])(string|violin|cello|viola)' THEN ARRAY['strings']

        -- Brass
        WHEN filename ~* '(^|[^a-z])(brass|trumpet|horn|trombone|sax)' THEN ARRAY['brass']

        -- Arp
        WHEN filename ~* '(^|[^a-z])(arp|arpeggiat)' THEN ARRAY['arp']

        -- FX
        WHEN filename ~* '(^|[^a-z])(fx|effect|sweep|riser|impact|transition)' THEN ARRAY['fx']

        -- Percussion
        WHEN filename ~* '(^|[^a-z])(perc|percussion|conga|bongo|tabla)' THEN ARRAY['percussion']

        -- Chord (usually keys)
        WHEN filename ~* '(^|[^a-z])chord' THEN ARRAY['keys']

        ELSE NULL
    END
)
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND filename IS NOT NULL;

COMMIT;

-- Show progress after filename inference
SELECT
    'After Filename Analysis' as stage,
    COUNT(*) FILTER (WHERE array_length(instrument_names_text, 1) > 0) as with_instruments,
    COUNT(*) FILTER (WHERE array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0) as still_missing,
    ROUND(100.0 * COUNT(*) FILTER (WHERE array_length(instrument_names_text, 1) > 0) / COUNT(*), 1) as coverage_pct
FROM files;

-- Step 2: PATH-based inference (for remaining files)
BEGIN;

UPDATE files
SET instrument_names_text = (
    CASE
        -- Drum patterns (grooves, fills, beats, loops)
        WHEN filepath ~* '/(drum|groove|fill|beat|loop|kit)' THEN ARRAY['drums']

        -- Bass
        WHEN filepath ~* '/bass' THEN ARRAY['bass']

        -- Synth
        WHEN filepath ~* '/synth' THEN ARRAY['synth']

        -- Lead
        WHEN filepath ~* '/(lead|melody)' THEN ARRAY['lead']

        -- Pad
        WHEN filepath ~* '/pad' THEN ARRAY['pad']

        -- Keys
        WHEN filepath ~* '/(key|keys|piano|keyboard|rhodes|organ)' THEN ARRAY['keys']

        -- Guitar
        WHEN filepath ~* '/guitar' THEN ARRAY['guitar']

        -- Vocal
        WHEN filepath ~* '/(vocal|voice|vox|choir)' THEN ARRAY['vocal']

        -- Strings
        WHEN filepath ~* '/string' THEN ARRAY['strings']

        -- Brass
        WHEN filepath ~* '/(brass|horn|trumpet)' THEN ARRAY['brass']

        -- Percussion
        WHEN filepath ~* '/(perc|percussion|conga|bongo)' THEN ARRAY['percussion']

        -- Arp
        WHEN filepath ~* '/arp' THEN ARRAY['arp']

        -- FX
        WHEN filepath ~* '/(fx|effect|sfx)' THEN ARRAY['fx']

        -- Pluck
        WHEN filepath ~* '/pluck' THEN ARRAY['pluck']

        ELSE NULL
    END
)
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND filepath IS NOT NULL;

COMMIT;

-- Final statistics with detailed breakdown
SELECT
    '═══════════════════════════════════════════════════════════' as separator;

SELECT
    'FINAL INSTRUMENT COVERAGE REPORT' as title;

SELECT
    '═══════════════════════════════════════════════════════════' as separator;

-- Overall coverage
SELECT
    'Total Files' as metric,
    COUNT(*)::TEXT as count,
    '100.0%' as percentage
FROM files
UNION ALL
SELECT
    'With Instruments' as metric,
    COUNT(*)::TEXT as count,
    ROUND(100.0 * COUNT(*) / (SELECT COUNT(*) FROM files), 1)::TEXT || '%' as percentage
FROM files
WHERE array_length(instrument_names_text, 1) > 0
UNION ALL
SELECT
    'Still Missing' as metric,
    COUNT(*)::TEXT as count,
    ROUND(100.0 * COUNT(*) / (SELECT COUNT(*) FROM files), 1)::TEXT || '%' as percentage
FROM files
WHERE array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0;

SELECT
    '───────────────────────────────────────────────────────────' as separator;

-- Instrument type breakdown
SELECT
    'INSTRUMENT DISTRIBUTION' as title;

SELECT
    '───────────────────────────────────────────────────────────' as separator;

SELECT
    UNNEST(instrument_names_text) as instrument,
    COUNT(*) as file_count,
    ROUND(100.0 * COUNT(*) / (SELECT COUNT(*) FROM files WHERE array_length(instrument_names_text, 1) > 0), 1)::TEXT || '%' as percentage_of_tagged
FROM files
WHERE array_length(instrument_names_text, 1) > 0
GROUP BY instrument
ORDER BY file_count DESC;

SELECT
    '═══════════════════════════════════════════════════════════' as separator;

-- Drop temporary indexes if needed
-- (Keeping them will speed up future queries)
-- DROP INDEX CONCURRENTLY IF EXISTS idx_files_missing_instruments;
-- DROP INDEX CONCURRENTLY IF EXISTS idx_files_filepath_lower;
-- DROP INDEX CONCURRENTLY IF EXISTS idx_files_filename_lower;

-- Vacuum analyze to update statistics
VACUUM ANALYZE files;

-- Tag Files by Instrument Based on Existing Metadata
-- Uses musical_metadata and filename analysis to categorize files
-- Run this BEFORE organizing to ensure proper categorization

BEGIN;

-- 1. Tag drums based on is_percussive flag and metadata
UPDATE files
SET tags = array_append(tags, 'drums')
WHERE NOT (tags @> ARRAY['drums'])
AND (
    EXISTS (
        SELECT 1 FROM musical_metadata m
        WHERE m.file_id = files.id
        AND m.is_percussive = true
    )
    OR lower(filename) ~ '(drum|kick|snare|hihat|cymbal|percussion|groove|beat)'
    OR lower(filepath) ~ '/(drum|percussion|groove|beat)/'
);

-- 2. Tag bass files
UPDATE files
SET tags = array_append(tags, 'bass')
WHERE NOT (tags @> ARRAY['bass'])
AND (
    lower(filename) ~ '(bass|sub|808)'
    OR lower(filepath) ~ '/(bass)/'
    OR EXISTS (
        SELECT 1 FROM musical_metadata m
        WHERE m.file_id = files.id
        AND 'bass' = ANY(m.instrument_names)
    )
);

-- 3. Tag loops
UPDATE files
SET tags = array_append(tags, 'loop')
WHERE NOT (tags @> ARRAY['loop'])
AND (
    lower(filename) ~ '(loop|_lp|pattern)'
    OR lower(filepath) ~ '/(loop|pattern)/'
);

-- 4. Tag melodic files (has melody)
UPDATE files
SET tags = array_append(tags, 'melodic')
WHERE NOT (tags @> ARRAY['melodic'])
AND EXISTS (
    SELECT 1 FROM musical_metadata m
    WHERE m.file_id = files.id
    AND m.has_melody = true
);

-- 5. Tag harmonic files (has chords)
UPDATE files
SET tags = array_append(tags, 'harmonic')
WHERE NOT (tags @> ARRAY['harmonic'])
AND EXISTS (
    SELECT 1 FROM musical_metadata m
    WHERE m.file_id = files.id
    AND m.has_chords = true
);

-- 6. Tag piano
UPDATE files
SET tags = array_append(tags, 'piano')
WHERE NOT (tags @> ARRAY['piano'])
AND (
    lower(filename) ~ 'piano'
    OR lower(filepath) ~ '/piano/'
    OR EXISTS (
        SELECT 1 FROM musical_metadata m
        WHERE m.file_id = files.id
        AND 'piano' = ANY(m.instrument_names)
    )
);

-- 7. Tag guitar
UPDATE files
SET tags = array_append(tags, 'guitar')
WHERE NOT (tags @> ARRAY['guitar'])
AND (
    lower(filename) ~ 'guitar'
    OR lower(filepath) ~ '/guitar/'
);

-- 8. Tag strings
UPDATE files
SET tags = array_append(tags, 'strings')
WHERE NOT (tags @> ARRAY['strings'])
AND (
    lower(filename) ~ '(string|violin|viola|cello|orchestra)'
    OR lower(filepath) ~ '/(string|orchestra)/'
);

-- 9. Tag brass
UPDATE files
SET tags = array_append(tags, 'brass')
WHERE NOT (tags @> ARRAY['brass'])
AND (
    lower(filename) ~ '(brass|trumpet|trombone|horn|tuba)'
    OR lower(filepath) ~ '/brass/'
);

-- 10. Tag synth
UPDATE files
SET tags = array_append(tags, 'synth')
WHERE NOT (tags @> ARRAY['synth'])
AND (
    lower(filename) ~ '(synth|lead|arp|sequence)'
    OR lower(filepath) ~ '/(synth|lead)/'
);

-- 11. Tag pads
UPDATE files
SET tags = array_append(tags, 'pad')
WHERE NOT (tags @> ARRAY['pad'])
AND (
    lower(filename) ~ 'pad'
    OR lower(filepath) ~ '/pad/'
);

COMMIT;

-- Summary of tagging
SELECT
    unnest(tags) as tag,
    COUNT(*) as file_count
FROM files
WHERE tags IS NOT NULL
GROUP BY tag
ORDER BY file_count DESC
LIMIT 30;

-- Show files by category
SELECT
    CASE
        WHEN tags @> ARRAY['drums'] THEN 'drums'
        WHEN tags @> ARRAY['bass'] THEN 'bass'
        WHEN tags @> ARRAY['piano'] THEN 'piano'
        WHEN tags @> ARRAY['guitar'] THEN 'guitar'
        WHEN tags @> ARRAY['strings'] THEN 'strings'
        WHEN tags @> ARRAY['brass'] THEN 'brass'
        WHEN tags @> ARRAY['synth'] THEN 'synth'
        WHEN tags @> ARRAY['pad'] THEN 'pad'
        WHEN tags @> ARRAY['melodic'] THEN 'melodic'
        WHEN tags @> ARRAY['harmonic'] THEN 'harmonic'
        WHEN tags @> ARRAY['loop'] THEN 'loop'
        ELSE 'uncategorized'
    END as primary_category,
    COUNT(*) as count
FROM files
GROUP BY primary_category
ORDER BY count DESC;

-- Show overlap (files with multiple instrument tags)
SELECT
    array_length(tags, 1) as tag_count,
    COUNT(*) as file_count
FROM files
WHERE tags IS NOT NULL
GROUP BY tag_count
ORDER BY tag_count;

-- Enhanced instrument inference from FULL file paths AND filenames
-- Runs after import completes on files missing instrument_names_text

BEGIN;

-- Create temporary function to extract instrument from filename
CREATE OR REPLACE FUNCTION infer_instrument_from_name(fname TEXT)
RETURNS TEXT[] AS $$
BEGIN
    -- Check filename (case-insensitive)
    -- Priority order: specific matches first, then general patterns

    -- Drum variations
    IF fname ~* '(^|[^a-z])(kick|snare|hat|hihat|cymbal|tom|clap|rim|cowbell|shaker)(s?)([^a-z]|$)' THEN
        RETURN ARRAY['drums'];
    END IF;

    -- Bass variations
    IF fname ~* '(^|[^a-z])(bass|sub|808)(s?)([^a-z]|$)' THEN
        RETURN ARRAY['bass'];
    END IF;

    -- Lead variations
    IF fname ~* '(^|[^a-z])(lead|melody|melod)(s?)([^a-z]|$)' THEN
        RETURN ARRAY['lead'];
    END IF;

    -- Synth variations
    IF fname ~* '(^|[^a-z])(synth|synthesis)(s?)([^a-z]|$)' THEN
        RETURN ARRAY['synth'];
    END IF;

    -- Pad variations
    IF fname ~* '(^|[^a-z])(pad|atmospher|ambient)(s?)([^a-z]|$)' THEN
        RETURN ARRAY['pad'];
    END IF;

    -- Pluck variations
    IF fname ~* '(^|[^a-z])(pluck|pizz)(s?)([^a-z]|$)' THEN
        RETURN ARRAY['pluck'];
    END IF;

    -- Keys/Piano
    IF fname ~* '(^|[^a-z])(key|keys|piano|keyboard|rhodes|organ)(s?)([^a-z]|$)' THEN
        RETURN ARRAY['keys'];
    END IF;

    -- Guitar
    IF fname ~* '(^|[^a-z])(guitar|gtr)(s?)([^a-z]|$)' THEN
        RETURN ARRAY['guitar'];
    END IF;

    -- Vocal
    IF fname ~* '(^|[^a-z])(vocal|voice|vox|choir)(s?)([^a-z]|$)' THEN
        RETURN ARRAY['vocal'];
    END IF;

    -- Strings
    IF fname ~* '(^|[^a-z])(string|violin|cello|viola)(s?)([^a-z]|$)' THEN
        RETURN ARRAY['strings'];
    END IF;

    -- Brass
    IF fname ~* '(^|[^a-z])(brass|trumpet|horn|trombone|sax)(s?)([^a-z]|$)' THEN
        RETURN ARRAY['brass'];
    END IF;

    -- Arp
    IF fname ~* '(^|[^a-z])(arp|arpeggiat)(s?)([^a-z]|$)' THEN
        RETURN ARRAY['arp'];
    END IF;

    -- FX
    IF fname ~* '(^|[^a-z])(fx|effect|sweep|riser|impact|transition)(s?)([^a-z]|$)' THEN
        RETURN ARRAY['fx'];
    END IF;

    -- Percussion (non-kit)
    IF fname ~* '(^|[^a-z])(perc|percussion|conga|bongo|tabla)(s?)([^a-z]|$)' THEN
        RETURN ARRAY['percussion'];
    END IF;

    -- Chord progressions (usually keys or synth)
    IF fname ~* '(^|[^a-z])(chord)(s?)([^a-z]|$)' THEN
        RETURN ARRAY['keys'];
    END IF;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Step 1: Infer from FILENAME first (most specific)
UPDATE files
SET instrument_names_text = infer_instrument_from_name(filename)
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND infer_instrument_from_name(filename) IS NOT NULL;

-- Step 2: Infer from PATH patterns (folders)
-- Drums
UPDATE files
SET instrument_names_text = ARRAY['drums']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (
    filepath ~* '/(drum|groove|fill|beat|loop|kit)s?/' OR
    parent_folder ~* 'drum|groove|fill|beat'
  );

-- Bass
UPDATE files
SET instrument_names_text = ARRAY['bass']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (filepath ~* '/bass(es|line)?/' OR parent_folder ~* 'bass');

-- Synth
UPDATE files
SET instrument_names_text = ARRAY['synth']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (filepath ~* '/synth(s|esizer)?/' OR parent_folder ~* 'synth');

-- Lead
UPDATE files
SET instrument_names_text = ARRAY['lead']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (filepath ~* '/(lead|melody)/' OR parent_folder ~* 'lead|melody');

-- Pad
UPDATE files
SET instrument_names_text = ARRAY['pad']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (filepath ~* '/pad(s)?/' OR parent_folder ~* 'pad');

-- Keys/Piano
UPDATE files
SET instrument_names_text = ARRAY['keys']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (
    filepath ~* '/(key|keys|piano|keyboard|rhodes|organ)s?/' OR
    parent_folder ~* 'key|piano|keyboard'
  );

-- Guitar
UPDATE files
SET instrument_names_text = ARRAY['guitar']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (filepath ~* '/guitar(s)?/' OR parent_folder ~* 'guitar');

-- Vocal
UPDATE files
SET instrument_names_text = ARRAY['vocal']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (
    filepath ~* '/(vocal|voice|vox|choir)s?/' OR
    parent_folder ~* 'vocal|voice|vox'
  );

-- Strings
UPDATE files
SET instrument_names_text = ARRAY['strings']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (filepath ~* '/string(s)?/' OR parent_folder ~* 'string');

-- Brass
UPDATE files
SET instrument_names_text = ARRAY['brass']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (
    filepath ~* '/(brass|horn|trumpet)s?/' OR
    parent_folder ~* 'brass|horn|trumpet'
  );

-- Percussion
UPDATE files
SET instrument_names_text = ARRAY['percussion']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (
    filepath ~* '/(perc|percussion|conga|bongo)/' OR
    parent_folder ~* 'perc|conga|bongo'
  );

-- Arp
UPDATE files
SET instrument_names_text = ARRAY['arp']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (filepath ~* '/arp(s)?/' OR parent_folder ~* 'arp');

-- FX
UPDATE files
SET instrument_names_text = ARRAY['fx']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (
    filepath ~* '/(fx|effect|sfx)s?/' OR
    parent_folder ~* 'fx|effect'
  );

-- Pluck
UPDATE files
SET instrument_names_text = ARRAY['pluck']
WHERE (array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0)
  AND (filepath ~* '/pluck/' OR parent_folder ~* 'pluck');

-- Drop the temporary function
DROP FUNCTION IF EXISTS infer_instrument_from_name(TEXT);

-- Show final statistics
SELECT
    'Total Files' as metric,
    COUNT(*) as count,
    '100.0%' as percentage
FROM files
UNION ALL
SELECT
    'With Instruments (MIDI text events)' as metric,
    COUNT(*) as count,
    ROUND(100.0 * COUNT(*) / (SELECT COUNT(*) FROM files), 1)::TEXT || '%' as percentage
FROM files
WHERE array_length(instrument_names_text, 1) > 0
  AND NOT (instrument_names_text::TEXT ~* 'infer_instrument_from_name')
UNION ALL
SELECT
    'Inferred from Filename' as metric,
    COUNT(*) as count,
    ROUND(100.0 * COUNT(*) / (SELECT COUNT(*) FROM files), 1)::TEXT || '%' as percentage
FROM files
WHERE array_length(instrument_names_text, 1) > 0
UNION ALL
SELECT
    'Still Missing' as metric,
    COUNT(*) as count,
    ROUND(100.0 * COUNT(*) / (SELECT COUNT(*) FROM files), 1)::TEXT || '%' as percentage
FROM files
WHERE array_length(instrument_names_text, 1) IS NULL OR array_length(instrument_names_text, 1) = 0;

-- Show instrument breakdown
SELECT
    UNNEST(instrument_names_text) as instrument,
    COUNT(*) as file_count,
    ROUND(100.0 * COUNT(*) / (SELECT COUNT(*) FROM files WHERE array_length(instrument_names_text, 1) > 0), 1)::TEXT || '%' as percentage
FROM files
WHERE array_length(instrument_names_text, 1) > 0
GROUP BY instrument
ORDER BY file_count DESC;

COMMIT;

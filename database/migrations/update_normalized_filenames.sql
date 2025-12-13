-- Update database records to match normalized filenames
-- Applies same transformations as normalize_filenames.rs:
-- 1. Lowercase .mid extension (.MID, .MIDI, .midi → .mid)
-- 2. Replace spaces with underscores
-- 3. Remove special characters (keep only a-z, A-Z, 0-9, _, -, .)
-- 4. No consecutive special chars

BEGIN;

-- Create function to sanitize filenames (matches Rust sanitize_strict)
CREATE OR REPLACE FUNCTION sanitize_filename(input TEXT) RETURNS TEXT AS $$
DECLARE
    result TEXT;
BEGIN
    result := input;

    -- 1. Replace spaces with underscores
    result := REPLACE(result, ' ', '_');

    -- 2. Remove or replace special characters (keep only alphanumeric, _, -, .)
    result := REGEXP_REPLACE(result, '[^a-zA-Z0-9_\-.]+', '', 'g');

    -- 3. Normalize extension to lowercase .mid
    result := REGEXP_REPLACE(result, '\.(mid|MID|MIDI|midi|Mid|MiD)$', '.mid', 'i');

    -- 4. Remove consecutive underscores, dashes
    result := REGEXP_REPLACE(result, '_+', '_', 'g');
    result := REGEXP_REPLACE(result, '-+', '-', 'g');

    -- 5. Remove leading/trailing underscores or dashes
    result := REGEXP_REPLACE(result, '^[_-]+', '', 'g');
    result := REGEXP_REPLACE(result, '[_-]+$', '', 'g');

    RETURN result;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Update files table
UPDATE files
SET
    filename = sanitize_filename(filename),
    filepath = REPLACE(
        filepath,
        '/' || filename,
        '/' || sanitize_filename(filename)
    ),
    updated_at = NOW()
WHERE
    -- Only update files that need changes
    filename != sanitize_filename(filename);

-- Show update statistics
DO $$
DECLARE
    updated_count INTEGER;
BEGIN
    GET DIAGNOSTICS updated_count = ROW_COUNT;
    RAISE NOTICE 'Updated % file records', updated_count;
END $$;

-- Verify no files have non-.mid extensions
SELECT
    CASE
        WHEN filename ~ '\\.mid$' THEN '.mid'
        ELSE 'other'
    END as extension,
    COUNT(*) as count
FROM files
GROUP BY extension
ORDER BY count DESC;

COMMIT;

-- Drop the helper function
DROP FUNCTION IF EXISTS sanitize_filename(TEXT);

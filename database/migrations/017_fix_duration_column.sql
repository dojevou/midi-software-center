-- Migration 017: Fix duration column location
-- duration_seconds is in files table, not musical_metadata
-- Run: psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -f database/migrations/017_fix_duration_column.sql

BEGIN;

-- ============================================================
-- Fix generate_smart_filename_v2 - get duration from files table
-- ============================================================
CREATE OR REPLACE FUNCTION generate_smart_filename_v2(
    p_file_id BIGINT,
    p_template TEXT DEFAULT '{genre}_{bpm}bpm_{key}_{instrument}_{mood}_{duration}s_{title}'
) RETURNS TEXT AS $$
DECLARE
    result TEXT;
    v_filename TEXT;
    v_filepath TEXT;
    v_duration NUMERIC;
    v_bpm NUMERIC;
    v_key TEXT;
    v_genre TEXT;
    v_instrument TEXT;
    v_mood TEXT;
    v_title TEXT;
BEGIN
    -- Get file info including duration
    SELECT filename, filepath, duration_seconds
    INTO v_filename, v_filepath, v_duration
    FROM files WHERE id = p_file_id;

    IF v_filename IS NULL THEN
        RETURN NULL;
    END IF;

    -- Extract title from original filename (remove extension, sanitize)
    v_title := REGEXP_REPLACE(v_filename, '\.[^.]+$', '');
    v_title := LOWER(REGEXP_REPLACE(v_title, '[^a-z0-9]+', '', 'gi'));
    IF v_title = '' OR v_title IS NULL THEN
        v_title := 'track';
    END IF;
    v_title := LEFT(v_title, 20);

    -- Get BPM and key from musical_metadata
    SELECT bpm, key_signature::TEXT
    INTO v_bpm, v_key
    FROM musical_metadata WHERE file_id = p_file_id;

    -- Get top tags by category (highest confidence first)
    SELECT t.name INTO v_genre
    FROM tags t
    JOIN file_tags ft ON t.id = ft.tag_id
    WHERE ft.file_id = p_file_id AND t.category = 'genre'
    ORDER BY ft.confidence DESC NULLS LAST
    LIMIT 1;

    SELECT t.name INTO v_instrument
    FROM tags t
    JOIN file_tags ft ON t.id = ft.tag_id
    WHERE ft.file_id = p_file_id AND t.category IN ('drums', 'bass', 'synth', 'keys', 'guitar', 'strings', 'brass', 'woodwind', 'vocal', 'keyword')
    ORDER BY ft.confidence DESC NULLS LAST
    LIMIT 1;

    SELECT t.name INTO v_mood
    FROM tags t
    JOIN file_tags ft ON t.id = ft.tag_id
    WHERE ft.file_id = p_file_id AND t.category = 'mood'
    ORDER BY ft.confidence DESC NULLS LAST
    LIMIT 1;

    -- Apply defaults
    v_genre := COALESCE(LEFT(LOWER(v_genre), 8), 'unk');
    v_instrument := COALESCE(LEFT(LOWER(v_instrument), 8), 'unk');
    v_mood := COALESCE(LEFT(LOWER(v_mood), 8), 'unk');
    v_bpm := COALESCE(v_bpm, 0);
    v_duration := COALESCE(v_duration, 0);

    -- Format key: Cm->cmin, C->cmaj, C#->csmaj, etc.
    v_key := COALESCE(v_key, 'unk');
    v_key := LOWER(v_key);
    v_key := REPLACE(v_key, '#', 's');
    v_key := REPLACE(v_key, 'b', 'f');
    IF v_key ~ 'm$' AND v_key <> 'unknown' THEN
        v_key := REGEXP_REPLACE(v_key, 'm$', 'min');
    ELSIF v_key <> 'unk' AND v_key <> 'unknown' THEN
        v_key := v_key || 'maj';
    END IF;
    v_key := LEFT(v_key, 6);

    -- Build filename from template
    result := p_template;
    result := REPLACE(result, '{genre}', v_genre);
    result := REPLACE(result, '{bpm}', ROUND(v_bpm)::TEXT);
    result := REPLACE(result, '{key}', v_key);
    result := REPLACE(result, '{instrument}', v_instrument);
    result := REPLACE(result, '{mood}', v_mood);
    result := REPLACE(result, '{duration}', ROUND(v_duration)::TEXT);
    result := REPLACE(result, '{title}', v_title);
    result := REPLACE(result, '{original}', v_title);

    -- Final sanitization
    result := REGEXP_REPLACE(result, '[^a-zA-Z0-9_-]', '_', 'g');
    result := REGEXP_REPLACE(result, '_+', '_', 'g');
    result := REGEXP_REPLACE(result, '^_|_$', '', 'g');
    result := result || '.mid';

    RETURN result;
END;
$$ LANGUAGE plpgsql STABLE;

COMMIT;

-- ============================================================
-- Verification
-- ============================================================
DO $$
BEGIN
    RAISE NOTICE 'Migration 017 complete:';
    RAISE NOTICE '  - generate_smart_filename_v2() fixed to get duration from files table';
END $$;

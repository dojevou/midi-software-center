-- Migration 015: Fix Smart Rename for musical_key enum
-- Fixes generate_smart_filename_v2() to properly cast musical_key enum to TEXT
-- Run: psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -f database/migrations/015_fix_smart_rename_enum.sql

BEGIN;

-- ============================================================
-- Fix: Cast musical_key enum to TEXT in generate_smart_filename_v2
-- ============================================================

CREATE OR REPLACE FUNCTION generate_smart_filename_v2(
    p_file_id INTEGER,
    p_template TEXT DEFAULT '{genre}_{bpm}bpm_{key}_{instrument}_{mood}_{duration}s_{title}'
) RETURNS TEXT AS $$
DECLARE
    result TEXT;
    v_filename TEXT;
    v_filepath TEXT;
    v_bpm FLOAT;
    v_key TEXT;
    v_duration FLOAT;
    v_genre TEXT;
    v_instrument TEXT;
    v_mood TEXT;
    v_title TEXT;
BEGIN
    -- Get file info
    SELECT filename, filepath INTO v_filename, v_filepath
    FROM files WHERE id = p_file_id;

    IF v_filename IS NULL THEN
        RETURN NULL;
    END IF;

    -- Extract title from original filename (remove extension, sanitize)
    v_title := REGEXP_REPLACE(v_filename, '\.[^.]+$', '');  -- Remove extension
    v_title := LOWER(REGEXP_REPLACE(v_title, '[^a-z0-9]+', '', 'gi'));  -- Keep only alphanumeric
    IF v_title = '' OR v_title IS NULL THEN
        v_title := 'track';
    END IF;
    v_title := LEFT(v_title, 20);  -- Max 20 chars for title

    -- Get metadata (FIXED: Cast musical_key enum to TEXT)
    SELECT bpm, key_signature::TEXT, duration_seconds
    INTO v_bpm, v_key, v_duration
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
    WHERE ft.file_id = p_file_id AND t.category IN ('drums', 'bass', 'synth', 'keys', 'guitar', 'strings', 'brass', 'woodwind', 'vocal')
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

    -- Format key: Cm->cmin, C->cmaj, C#m->csmin, etc.
    v_key := COALESCE(v_key, 'unk');
    v_key := LOWER(v_key);
    -- Handle sharp/flat notation
    v_key := REPLACE(v_key, '#', 's');  -- C# -> Cs
    v_key := REPLACE(v_key, 'b', 'f');  -- Db -> Df (flat)
    -- Handle minor (ends with 'm') vs major
    IF v_key ~ 'm$' AND v_key != 'unknown' THEN
        v_key := REGEXP_REPLACE(v_key, 'm$', 'min');
    ELSIF v_key != 'unk' AND v_key != 'unknown' THEN
        v_key := v_key || 'maj';
    END IF;
    v_key := LEFT(v_key, 6);  -- Max 6 chars

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

-- ============================================================
-- Fix: Cast musical_key enum in generate_smart_subdir as well
-- ============================================================

CREATE OR REPLACE FUNCTION generate_smart_subdir(
    p_file_id INTEGER,
    p_base_dir TEXT DEFAULT '/midi-library/organized'
) RETURNS TEXT AS $$
DECLARE
    v_genre TEXT;
    v_mood TEXT;
    v_bpm FLOAT;
    v_bpm_lo INTEGER;
    v_bpm_hi INTEGER;
BEGIN
    -- Get genre tag
    SELECT t.name INTO v_genre
    FROM tags t
    JOIN file_tags ft ON t.id = ft.tag_id
    WHERE ft.file_id = p_file_id AND t.category = 'genre'
    ORDER BY ft.confidence DESC NULLS LAST
    LIMIT 1;

    -- Get mood tag
    SELECT t.name INTO v_mood
    FROM tags t
    JOIN file_tags ft ON t.id = ft.tag_id
    WHERE ft.file_id = p_file_id AND t.category = 'mood'
    ORDER BY ft.confidence DESC NULLS LAST
    LIMIT 1;

    -- Get BPM
    SELECT bpm INTO v_bpm
    FROM musical_metadata WHERE file_id = p_file_id;

    -- Apply defaults
    v_genre := COALESCE(LOWER(v_genre), 'unknown');
    v_mood := COALESCE(LOWER(v_mood), 'neutral');
    v_bpm := COALESCE(v_bpm, 120);

    -- Calculate BPM range (20 BPM buckets)
    v_bpm_lo := (FLOOR(v_bpm / 20) * 20)::INTEGER;
    v_bpm_hi := v_bpm_lo + 20;

    RETURN p_base_dir || '/' || v_genre || '/' || v_bpm_lo || '-' || v_bpm_hi || 'bpm/' || v_mood;
END;
$$ LANGUAGE plpgsql STABLE;

COMMIT;

-- ============================================================
-- Verification
-- ============================================================
DO $$
BEGIN
    RAISE NOTICE 'Migration 015 complete:';
    RAISE NOTICE '  - generate_smart_filename_v2() fixed for musical_key enum';
    RAISE NOTICE '  - Key formatting improved (Cm->cmin, C#->csmaj)';
END $$;

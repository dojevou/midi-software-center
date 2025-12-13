-- Migration 016: Fix BIGINT types for all smart rename functions
-- files.id is BIGINT, not INTEGER
-- Run: psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -f database/migrations/016_fix_bigint_types.sql

BEGIN;

-- ============================================================
-- Drop old INTEGER versions
-- ============================================================
DROP FUNCTION IF EXISTS generate_smart_filename_v2(INTEGER, TEXT);
DROP FUNCTION IF EXISTS generate_smart_subdir(INTEGER, TEXT);
DROP FUNCTION IF EXISTS batch_generate_rename_plan(INTEGER, INTEGER);
DROP FUNCTION IF EXISTS execute_rename(INTEGER, UUID);
DROP FUNCTION IF EXISTS rollback_rename(INTEGER);

-- ============================================================
-- 1. generate_smart_filename_v2 with BIGINT
-- ============================================================
CREATE OR REPLACE FUNCTION generate_smart_filename_v2(
    p_file_id BIGINT,
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
    SELECT filename, filepath INTO v_filename, v_filepath
    FROM files WHERE id = p_file_id;

    IF v_filename IS NULL THEN
        RETURN NULL;
    END IF;

    v_title := REGEXP_REPLACE(v_filename, '\.[^.]+$', '');
    v_title := LOWER(REGEXP_REPLACE(v_title, '[^a-z0-9]+', '', 'gi'));
    IF v_title = '' OR v_title IS NULL THEN
        v_title := 'track';
    END IF;
    v_title := LEFT(v_title, 20);

    SELECT bpm, key_signature::TEXT, duration_seconds
    INTO v_bpm, v_key, v_duration
    FROM musical_metadata WHERE file_id = p_file_id;

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

    v_genre := COALESCE(LEFT(LOWER(v_genre), 8), 'unk');
    v_instrument := COALESCE(LEFT(LOWER(v_instrument), 8), 'unk');
    v_mood := COALESCE(LEFT(LOWER(v_mood), 8), 'unk');
    v_bpm := COALESCE(v_bpm, 0);
    v_duration := COALESCE(v_duration, 0);

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

    result := p_template;
    result := REPLACE(result, '{genre}', v_genre);
    result := REPLACE(result, '{bpm}', ROUND(v_bpm)::TEXT);
    result := REPLACE(result, '{key}', v_key);
    result := REPLACE(result, '{instrument}', v_instrument);
    result := REPLACE(result, '{mood}', v_mood);
    result := REPLACE(result, '{duration}', ROUND(v_duration)::TEXT);
    result := REPLACE(result, '{title}', v_title);
    result := REPLACE(result, '{original}', v_title);

    result := REGEXP_REPLACE(result, '[^a-zA-Z0-9_-]', '_', 'g');
    result := REGEXP_REPLACE(result, '_+', '_', 'g');
    result := REGEXP_REPLACE(result, '^_|_$', '', 'g');
    result := result || '.mid';

    RETURN result;
END;
$$ LANGUAGE plpgsql STABLE;

-- ============================================================
-- 2. generate_smart_subdir with BIGINT
-- ============================================================
CREATE OR REPLACE FUNCTION generate_smart_subdir(
    p_file_id BIGINT,
    p_base_dir TEXT DEFAULT '/midi-library/organized'
) RETURNS TEXT AS $$
DECLARE
    v_genre TEXT;
    v_mood TEXT;
    v_bpm FLOAT;
    v_bpm_lo INTEGER;
    v_bpm_hi INTEGER;
BEGIN
    SELECT t.name INTO v_genre
    FROM tags t
    JOIN file_tags ft ON t.id = ft.tag_id
    WHERE ft.file_id = p_file_id AND t.category = 'genre'
    ORDER BY ft.confidence DESC NULLS LAST
    LIMIT 1;

    SELECT t.name INTO v_mood
    FROM tags t
    JOIN file_tags ft ON t.id = ft.tag_id
    WHERE ft.file_id = p_file_id AND t.category = 'mood'
    ORDER BY ft.confidence DESC NULLS LAST
    LIMIT 1;

    SELECT bpm INTO v_bpm
    FROM musical_metadata WHERE file_id = p_file_id;

    v_genre := COALESCE(LOWER(v_genre), 'unknown');
    v_mood := COALESCE(LOWER(v_mood), 'neutral');
    v_bpm := COALESCE(v_bpm, 120);

    v_bpm_lo := (FLOOR(v_bpm / 20) * 20)::INTEGER;
    v_bpm_hi := v_bpm_lo + 20;

    RETURN p_base_dir || '/' || v_genre || '/' || v_bpm_lo || '-' || v_bpm_hi || 'bpm/' || v_mood;
END;
$$ LANGUAGE plpgsql STABLE;

-- ============================================================
-- 3. batch_generate_rename_plan with BIGINT returns
-- ============================================================
CREATE OR REPLACE FUNCTION batch_generate_rename_plan(
    p_limit INTEGER DEFAULT 1000,
    p_offset INTEGER DEFAULT 0
) RETURNS TABLE(
    file_id BIGINT,
    old_filename TEXT,
    new_filename TEXT,
    old_filepath TEXT,
    new_subdir TEXT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        f.id,
        f.filename::TEXT,
        generate_smart_filename_v2(f.id)::TEXT,
        f.filepath::TEXT,
        generate_smart_subdir(f.id)::TEXT
    FROM files f
    WHERE f.filename IS NOT NULL
    ORDER BY f.id
    LIMIT p_limit
    OFFSET p_offset;
END;
$$ LANGUAGE plpgsql STABLE;

-- ============================================================
-- 4. execute_rename with BIGINT
-- ============================================================
CREATE OR REPLACE FUNCTION execute_rename(
    p_file_id BIGINT,
    p_batch_id UUID DEFAULT NULL
) RETURNS BOOLEAN AS $$
DECLARE
    v_old_filename TEXT;
    v_new_filename TEXT;
    v_old_filepath TEXT;
    v_new_subdir TEXT;
    v_new_filepath TEXT;
    v_tags_json JSONB;
BEGIN
    SELECT filename, filepath INTO v_old_filename, v_old_filepath
    FROM files WHERE id = p_file_id;

    IF v_old_filename IS NULL THEN
        RETURN FALSE;
    END IF;

    v_new_filename := generate_smart_filename_v2(p_file_id);
    v_new_subdir := generate_smart_subdir(p_file_id);
    v_new_filepath := v_new_subdir || '/' || v_new_filename;

    SELECT JSONB_AGG(JSONB_BUILD_OBJECT(
        'name', t.name,
        'category', t.category,
        'confidence', ft.confidence
    ))
    INTO v_tags_json
    FROM file_tags ft
    JOIN tags t ON ft.tag_id = t.id
    WHERE ft.file_id = p_file_id;

    INSERT INTO rename_history (
        file_id, old_filename, new_filename,
        old_filepath, new_filepath,
        rename_batch_id, tags_snapshot
    ) VALUES (
        p_file_id, v_old_filename, v_new_filename,
        v_old_filepath, v_new_filepath,
        p_batch_id, v_tags_json
    );

    UPDATE files
    SET filename = v_new_filename,
        filepath = v_new_filepath
    WHERE id = p_file_id;

    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

-- ============================================================
-- 5. rollback_rename with BIGINT
-- ============================================================
CREATE OR REPLACE FUNCTION rollback_rename(
    p_file_id BIGINT
) RETURNS BOOLEAN AS $$
DECLARE
    v_history RECORD;
BEGIN
    SELECT * INTO v_history
    FROM rename_history
    WHERE file_id = p_file_id AND reversed_at IS NULL
    ORDER BY renamed_at DESC
    LIMIT 1;

    IF v_history IS NULL THEN
        RETURN FALSE;
    END IF;

    UPDATE files
    SET filename = v_history.old_filename,
        filepath = v_history.old_filepath
    WHERE id = p_file_id;

    UPDATE rename_history
    SET reversed_at = NOW()
    WHERE id = v_history.id;

    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

COMMIT;

-- ============================================================
-- Verification
-- ============================================================
DO $$
BEGIN
    RAISE NOTICE 'Migration 016 complete:';
    RAISE NOTICE '  - All smart rename functions updated to use BIGINT';
    RAISE NOTICE '  - generate_smart_filename_v2(BIGINT) ready';
    RAISE NOTICE '  - generate_smart_subdir(BIGINT) ready';
    RAISE NOTICE '  - batch_generate_rename_plan() ready';
    RAISE NOTICE '  - execute_rename(BIGINT) ready';
    RAISE NOTICE '  - rollback_rename(BIGINT) ready';
END $$;

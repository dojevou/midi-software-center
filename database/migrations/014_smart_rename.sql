-- Migration 014: Smart Rename Functions
-- Ported from ~/midi/original/services/rename_service/app.py
-- Run: psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -f database/migrations/014_smart_rename.sql

BEGIN;

-- ============================================================
-- 1. Enhanced Smart Filename Generator
-- Ports: filename_from_tags() from rename_service
-- Template: {genre}_{bpm}bpm_{key}_{instrument}_{mood}_{duration}s_{title}.mid
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

    -- Get metadata
    SELECT bpm, key_signature, duration_seconds
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

    -- Format key: major->maj, minor->min
    v_key := COALESCE(v_key, 'unk');
    v_key := REPLACE(LOWER(v_key), 'major', 'maj');
    v_key := REPLACE(v_key, 'minor', 'min');
    v_key := LEFT(v_key, 5);

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
-- 2. Generate Subdirectory Path from Tags
-- Ports: subdir_from_tags() - organizes by genre/bpm-range/mood
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

-- ============================================================
-- 3. Batch Rename with History Tracking
-- ============================================================

CREATE OR REPLACE FUNCTION batch_generate_rename_plan(
    p_limit INTEGER DEFAULT 1000,
    p_offset INTEGER DEFAULT 0
) RETURNS TABLE(
    file_id INTEGER,
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
-- 4. Execute Rename with History
-- ============================================================

CREATE OR REPLACE FUNCTION execute_rename(
    p_file_id INTEGER,
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
    -- Get current file info
    SELECT filename, filepath INTO v_old_filename, v_old_filepath
    FROM files WHERE id = p_file_id;

    IF v_old_filename IS NULL THEN
        RETURN FALSE;
    END IF;

    -- Generate new names
    v_new_filename := generate_smart_filename_v2(p_file_id);
    v_new_subdir := generate_smart_subdir(p_file_id);
    v_new_filepath := v_new_subdir || '/' || v_new_filename;

    -- Collect current tags as JSON snapshot
    SELECT JSONB_AGG(JSONB_BUILD_OBJECT(
        'name', t.name,
        'category', t.category,
        'confidence', ft.confidence
    ))
    INTO v_tags_json
    FROM file_tags ft
    JOIN tags t ON ft.tag_id = t.id
    WHERE ft.file_id = p_file_id;

    -- Record in rename history
    INSERT INTO rename_history (
        file_id, old_filename, new_filename,
        old_filepath, new_filepath,
        rename_batch_id, tags_snapshot
    ) VALUES (
        p_file_id, v_old_filename, v_new_filename,
        v_old_filepath, v_new_filepath,
        p_batch_id, v_tags_json
    );

    -- Update file record
    UPDATE files
    SET filename = v_new_filename,
        filepath = v_new_filepath
    WHERE id = p_file_id;

    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

-- ============================================================
-- 5. Rollback Rename
-- ============================================================

CREATE OR REPLACE FUNCTION rollback_rename(
    p_file_id INTEGER
) RETURNS BOOLEAN AS $$
DECLARE
    v_history RECORD;
BEGIN
    -- Get most recent unreversed rename
    SELECT * INTO v_history
    FROM rename_history
    WHERE file_id = p_file_id AND reversed_at IS NULL
    ORDER BY renamed_at DESC
    LIMIT 1;

    IF v_history IS NULL THEN
        RETURN FALSE;
    END IF;

    -- Restore original names
    UPDATE files
    SET filename = v_history.old_filename,
        filepath = v_history.old_filepath
    WHERE id = p_file_id;

    -- Mark as reversed
    UPDATE rename_history
    SET reversed_at = NOW()
    WHERE id = v_history.id;

    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

-- ============================================================
-- 6. Batch Rollback by Batch ID
-- ============================================================

CREATE OR REPLACE FUNCTION rollback_batch(
    p_batch_id UUID
) RETURNS INTEGER AS $$
DECLARE
    v_count INTEGER := 0;
    v_file_id INTEGER;
BEGIN
    FOR v_file_id IN
        SELECT DISTINCT file_id
        FROM rename_history
        WHERE rename_batch_id = p_batch_id AND reversed_at IS NULL
    LOOP
        IF rollback_rename(v_file_id) THEN
            v_count := v_count + 1;
        END IF;
    END LOOP;

    RETURN v_count;
END;
$$ LANGUAGE plpgsql;

-- ============================================================
-- 7. Get Rename Statistics
-- ============================================================

CREATE OR REPLACE FUNCTION get_rename_stats()
RETURNS TABLE(
    total_renames BIGINT,
    pending_files BIGINT,
    reversed_count BIGINT,
    unique_batches BIGINT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        (SELECT COUNT(*) FROM rename_history)::BIGINT,
        (SELECT COUNT(*) FROM files f WHERE NOT EXISTS (
            SELECT 1 FROM rename_history rh WHERE rh.file_id = f.id AND rh.reversed_at IS NULL
        ))::BIGINT,
        (SELECT COUNT(*) FROM rename_history WHERE reversed_at IS NOT NULL)::BIGINT,
        (SELECT COUNT(DISTINCT rename_batch_id) FROM rename_history WHERE rename_batch_id IS NOT NULL)::BIGINT;
END;
$$ LANGUAGE plpgsql STABLE;

COMMIT;

-- ============================================================
-- Verification
-- ============================================================
DO $$
BEGIN
    RAISE NOTICE 'Migration 014 complete:';
    RAISE NOTICE '  - generate_smart_filename_v2() created';
    RAISE NOTICE '  - generate_smart_subdir() created';
    RAISE NOTICE '  - batch_generate_rename_plan() created';
    RAISE NOTICE '  - execute_rename() created';
    RAISE NOTICE '  - rollback_rename() created';
    RAISE NOTICE '  - rollback_batch() created';
    RAISE NOTICE '  - get_rename_stats() created';
END $$;

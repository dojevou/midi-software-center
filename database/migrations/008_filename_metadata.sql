-- Migration 008: Filename Metadata Extraction
-- Date: 2025-11-09
-- Purpose: Add filename-based metadata columns for BPM, key, genres, and structure tags
--
-- Based on analysis of 1,486,270 MIDI files from production collection:
-- - 81 unique BPM values extracted from filenames
-- - 100 unique key signatures
-- - 20 genre types
-- - 20 structure tags
--
-- This migration enables:
-- 1. Metadata fallback when MIDI analysis fails
-- 2. Cross-validation (analyzed vs filename metadata)
-- 3. Enhanced search (genre + BPM + structure filtering)
-- 4. Multi-dimensional categorization

BEGIN;

-- ============================================================================
-- PART 1: Add Filename Metadata Columns to Files Table
-- ============================================================================

-- BPM extracted from filename (30-300 range)
ALTER TABLE files
    ADD COLUMN IF NOT EXISTS filename_bpm REAL
    CHECK (filename_bpm IS NULL OR (filename_bpm BETWEEN 30 AND 300));

-- Key signature extracted from filename (e.g., 'Cm', 'Am', 'F#')
ALTER TABLE files
    ADD COLUMN IF NOT EXISTS filename_key TEXT
    CHECK (filename_key IS NULL OR LENGTH(filename_key) BETWEEN 1 AND 3);

-- Genre tags from filename (house, techno, dnb, etc.)
ALTER TABLE files
    ADD COLUMN IF NOT EXISTS filename_genres TEXT[];

-- Structure tags from filename (fill, loop, verse, etc.)
ALTER TABLE files
    ADD COLUMN IF NOT EXISTS structure_tags TEXT[];

-- Track number from leading digits in filename
ALTER TABLE files
    ADD COLUMN IF NOT EXISTS track_number INTEGER
    CHECK (track_number IS NULL OR track_number > 0);

-- Metadata source indicator ('analyzed', 'filename', 'both', 'none')
ALTER TABLE files
    ADD COLUMN IF NOT EXISTS metadata_source TEXT
    CHECK (metadata_source IN ('analyzed', 'filename', 'both', 'none', 'validated'))
    DEFAULT 'none';

-- Column comments for documentation
COMMENT ON COLUMN files.filename_bpm IS
    'BPM value extracted from filename pattern matching (e.g., "120_bpm_house_loop.mid")';

COMMENT ON COLUMN files.filename_key IS
    'Musical key extracted from filename (e.g., "Cm_bass.mid" -> "Cm")';

COMMENT ON COLUMN files.filename_genres IS
    'Array of genre tags extracted from filename (house, techno, dnb, etc.)';

COMMENT ON COLUMN files.structure_tags IS
    'Array of structure tags from filename (fill, loop, verse, chorus, etc.)';

COMMENT ON COLUMN files.track_number IS
    'Track number from leading digits in filename (e.g., "01_kick.mid" -> 1)';

COMMENT ON COLUMN files.metadata_source IS
    'Source of metadata: analyzed (MIDI analysis only), filename (filename only), both (both sources), validated (cross-validated match), none (no metadata)';

-- ============================================================================
-- PART 2: Create Indexes for Performance
-- ============================================================================

-- Index on filename_bpm for BPM range queries
CREATE INDEX IF NOT EXISTS idx_files_filename_bpm
    ON files(filename_bpm)
    WHERE filename_bpm IS NOT NULL;

-- Index on filename_key for key signature queries
CREATE INDEX IF NOT EXISTS idx_files_filename_key
    ON files(filename_key)
    WHERE filename_key IS NOT NULL;

-- GIN index on filename_genres array for fast "ANY" queries
CREATE INDEX IF NOT EXISTS idx_files_filename_genres
    ON files USING GIN(filename_genres)
    WHERE filename_genres IS NOT NULL AND array_length(filename_genres, 1) > 0;

-- GIN index on structure_tags array for fast "ANY" queries
CREATE INDEX IF NOT EXISTS idx_files_structure_tags
    ON files USING GIN(structure_tags)
    WHERE structure_tags IS NOT NULL AND array_length(structure_tags, 1) > 0;

-- Index on track_number for ordering queries
CREATE INDEX IF NOT EXISTS idx_files_track_number
    ON files(track_number)
    WHERE track_number IS NOT NULL;

-- Index on metadata_source for filtering by source type
CREATE INDEX IF NOT EXISTS idx_files_metadata_source
    ON files(metadata_source);

-- Composite index for common BPM + key queries
CREATE INDEX IF NOT EXISTS idx_files_filename_bpm_key
    ON files(filename_bpm, filename_key)
    WHERE filename_bpm IS NOT NULL OR filename_key IS NOT NULL;

-- ============================================================================
-- PART 3: Helper Functions
-- ============================================================================

-- Function to get effective BPM with fallback
CREATE OR REPLACE FUNCTION get_effective_bpm(p_file_id BIGINT)
RETURNS REAL AS $$
DECLARE
    v_analyzed_bpm REAL;
    v_filename_bpm REAL;
BEGIN
    SELECT mm.bpm, f.filename_bpm
    INTO v_analyzed_bpm, v_filename_bpm
    FROM files f
    LEFT JOIN musical_metadata mm ON f.id = mm.file_id
    WHERE f.id = p_file_id;

    -- Prefer analyzed BPM, fall back to filename
    RETURN COALESCE(v_analyzed_bpm, v_filename_bpm);
END;
$$ LANGUAGE plpgsql STABLE;

COMMENT ON FUNCTION get_effective_bpm(BIGINT) IS
    'Returns effective BPM with fallback: analyzed BPM preferred, filename BPM as fallback';

-- Function to get effective key with fallback
CREATE OR REPLACE FUNCTION get_effective_key(p_file_id BIGINT)
RETURNS TEXT AS $$
DECLARE
    v_analyzed_key TEXT;
    v_filename_key TEXT;
BEGIN
    SELECT mm.key_signature::TEXT, f.filename_key
    INTO v_analyzed_key, v_filename_key
    FROM files f
    LEFT JOIN musical_metadata mm ON f.id = mm.file_id
    WHERE f.id = p_file_id;

    -- Prefer analyzed key, fall back to filename
    RETURN COALESCE(v_analyzed_key, v_filename_key);
END;
$$ LANGUAGE plpgsql STABLE;

COMMENT ON FUNCTION get_effective_key(BIGINT) IS
    'Returns effective key signature with fallback: analyzed key preferred, filename key as fallback';

-- Function to validate BPM between analyzed and filename
CREATE OR REPLACE FUNCTION validate_bpm_match(
    p_analyzed_bpm REAL,
    p_filename_bpm REAL,
    p_tolerance REAL DEFAULT 5.0
)
RETURNS BOOLEAN AS $$
BEGIN
    IF p_analyzed_bpm IS NULL OR p_filename_bpm IS NULL THEN
        RETURN FALSE;
    END IF;

    RETURN ABS(p_analyzed_bpm - p_filename_bpm) <= p_tolerance;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

COMMENT ON FUNCTION validate_bpm_match(REAL, REAL, REAL) IS
    'Validates if analyzed BPM matches filename BPM within tolerance (default ±5 BPM)';

-- Function to detect metadata conflicts
CREATE OR REPLACE FUNCTION detect_metadata_conflicts()
RETURNS TABLE (
    file_id BIGINT,
    filename TEXT,
    analyzed_bpm REAL,
    filename_bpm REAL,
    bpm_diff REAL,
    analyzed_key TEXT,
    filename_key TEXT,
    key_match BOOLEAN
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        f.id,
        f.filename,
        mm.bpm AS analyzed_bpm,
        f.filename_bpm,
        ABS(mm.bpm::NUMERIC - f.filename_bpm::NUMERIC)::REAL AS bpm_diff,
        mm.key_signature::TEXT AS analyzed_key,
        f.filename_key,
        (mm.key_signature::TEXT = f.filename_key) AS key_match
    FROM files f
    LEFT JOIN musical_metadata mm ON f.id = mm.file_id
    WHERE
        (mm.bpm IS NOT NULL AND f.filename_bpm IS NOT NULL AND ABS(mm.bpm::NUMERIC - f.filename_bpm::NUMERIC) > 5)
        OR (mm.key_signature IS NOT NULL AND f.filename_key IS NOT NULL AND mm.key_signature::TEXT != f.filename_key)
    ORDER BY bpm_diff DESC NULLS LAST;
END;
$$ LANGUAGE plpgsql STABLE;

COMMENT ON FUNCTION detect_metadata_conflicts() IS
    'Identifies files where analyzed metadata conflicts with filename metadata (BPM ±5 or key mismatch)';

-- ============================================================================
-- PART 4: Enhanced Search Views
-- ============================================================================

-- View: Files with complete metadata (both analyzed and filename)
CREATE OR REPLACE VIEW files_with_complete_metadata AS
SELECT
    f.id,
    f.filename,
    f.path,
    COALESCE(mm.bpm, f.filename_bpm) AS effective_bpm,
    COALESCE(mm.key_signature::TEXT, f.filename_key) AS effective_key,
    mm.time_signature,
    f.duration_seconds,
    f.filename_genres,
    f.structure_tags,
    f.track_number,
    f.metadata_source,
    ARRAY(SELECT t.name FROM file_tags ft JOIN tags t ON ft.tag_id = t.id WHERE ft.file_id = f.id) AS all_tags
FROM files f
LEFT JOIN musical_metadata mm ON f.id = mm.file_id
WHERE mm.bpm IS NOT NULL OR f.filename_bpm IS NOT NULL
   OR mm.key_signature IS NOT NULL OR f.filename_key IS NOT NULL;

COMMENT ON VIEW files_with_complete_metadata IS
    'Files with effective metadata using analyzed data with filename fallbacks';

-- View: Files with validated metadata (analyzed matches filename)
CREATE OR REPLACE VIEW files_with_validated_metadata AS
SELECT
    f.id,
    f.filename,
    mm.bpm AS validated_bpm,
    mm.key_signature::TEXT AS validated_key,
    f.filename_genres,
    f.structure_tags,
    f.created_at
FROM files f
JOIN musical_metadata mm ON f.id = mm.file_id
WHERE
    (mm.bpm IS NOT NULL AND f.filename_bpm IS NOT NULL AND ABS(mm.bpm::NUMERIC - f.filename_bpm::NUMERIC) <= 5)
    OR (mm.key_signature IS NOT NULL AND f.filename_key IS NOT NULL AND mm.key_signature::TEXT = f.filename_key);

COMMENT ON VIEW files_with_validated_metadata IS
    'Files where analyzed metadata is validated by matching filename metadata';

-- View: Popular genre + BPM combinations
CREATE OR REPLACE VIEW popular_genre_bpm_combinations AS
SELECT
    unnest(filename_genres) AS genre,
    CASE
        WHEN filename_bpm BETWEEN 60 AND 90 THEN '60-90 BPM (Slow)'
        WHEN filename_bpm BETWEEN 90 AND 110 THEN '90-110 BPM (Hip-Hop)'
        WHEN filename_bpm BETWEEN 110 AND 130 THEN '110-130 BPM (House/Pop)'
        WHEN filename_bpm BETWEEN 130 AND 150 THEN '130-150 BPM (Techno/Trap)'
        WHEN filename_bpm BETWEEN 150 AND 180 THEN '150-180 BPM (DnB)'
        WHEN filename_bpm BETWEEN 180 AND 300 THEN '180-300 BPM (Very Fast)'
    END AS bpm_range,
    COUNT(*) AS file_count,
    ROUND(AVG(filename_bpm)::NUMERIC, 1) AS avg_bpm
FROM files
WHERE filename_genres IS NOT NULL
  AND array_length(filename_genres, 1) > 0
  AND filename_bpm IS NOT NULL
GROUP BY genre, bpm_range
HAVING COUNT(*) > 10
ORDER BY file_count DESC;

COMMENT ON VIEW popular_genre_bpm_combinations IS
    'Popular genre + BPM range combinations for discovery and filtering';

-- ============================================================================
-- PART 5: Update Existing Search Functions
-- ============================================================================

-- Enhanced search function with filename metadata
CREATE OR REPLACE FUNCTION search_files_with_metadata(
    p_search_text TEXT DEFAULT NULL,
    p_bpm_min REAL DEFAULT NULL,
    p_bpm_max REAL DEFAULT NULL,
    p_key TEXT DEFAULT NULL,
    p_genres TEXT[] DEFAULT NULL,
    p_structure_tags TEXT[] DEFAULT NULL,
    p_limit INTEGER DEFAULT 100,
    p_offset INTEGER DEFAULT 0
)
RETURNS TABLE (
    id BIGINT,
    filename TEXT,
    path TEXT,
    effective_bpm REAL,
    effective_key TEXT,
    genres TEXT[],
    structure TEXT[],
    duration_seconds INTEGER,
    file_size_bytes BIGINT,
    created_at TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        f.id,
        f.filename,
        f.path,
        COALESCE(mm.bpm, f.filename_bpm) AS effective_bpm,
        COALESCE(mm.key_signature::TEXT, f.filename_key) AS effective_key,
        f.filename_genres AS genres,
        f.structure_tags AS structure,
        f.duration_seconds,
        f.file_size_bytes,
        f.created_at
    FROM files f
    LEFT JOIN musical_metadata mm ON f.id = mm.file_id
    WHERE
        (p_search_text IS NULL OR f.filename ILIKE '%' || p_search_text || '%')
        AND (p_bpm_min IS NULL OR COALESCE(mm.bpm, f.filename_bpm) >= p_bpm_min)
        AND (p_bpm_max IS NULL OR COALESCE(mm.bpm, f.filename_bpm) <= p_bpm_max)
        AND (p_key IS NULL OR COALESCE(mm.key_signature::TEXT, f.filename_key) = p_key)
        AND (p_genres IS NULL OR f.filename_genres && p_genres)
        AND (p_structure_tags IS NULL OR f.structure_tags && p_structure_tags)
    ORDER BY f.created_at DESC
    LIMIT p_limit
    OFFSET p_offset;
END;
$$ LANGUAGE plpgsql STABLE;

COMMENT ON FUNCTION search_files_with_metadata IS
    'Enhanced search with filename metadata fallbacks for BPM, key, genres, and structure tags';

-- ============================================================================
-- PART 6: Statistics and Reporting
-- ============================================================================

-- Function to get metadata coverage statistics
CREATE OR REPLACE FUNCTION get_metadata_coverage_stats()
RETURNS TABLE (
    metric TEXT,
    count BIGINT,
    percentage NUMERIC(5,2)
) AS $$
DECLARE
    v_total_files BIGINT;
BEGIN
    SELECT COUNT(*) INTO v_total_files FROM files;

    RETURN QUERY
    WITH stats AS (
        SELECT 'Total Files' AS metric, v_total_files AS count
        UNION ALL
        SELECT 'Files with Analyzed BPM', COUNT(*) FROM files f JOIN musical_metadata mm ON f.id = mm.file_id WHERE mm.bpm IS NOT NULL
        UNION ALL
        SELECT 'Files with Filename BPM', COUNT(*) FROM files WHERE filename_bpm IS NOT NULL
        UNION ALL
        SELECT 'Files with Either BPM', COUNT(*) FROM files f LEFT JOIN musical_metadata mm ON f.id = mm.file_id WHERE mm.bpm IS NOT NULL OR filename_bpm IS NOT NULL
        UNION ALL
        SELECT 'Files with Analyzed Key', COUNT(*) FROM files f JOIN musical_metadata mm ON f.id = mm.file_id WHERE mm.key_signature IS NOT NULL
        UNION ALL
        SELECT 'Files with Filename Key', COUNT(*) FROM files WHERE filename_key IS NOT NULL
        UNION ALL
        SELECT 'Files with Either Key', COUNT(*) FROM files f LEFT JOIN musical_metadata mm ON f.id = mm.file_id WHERE mm.key_signature IS NOT NULL OR filename_key IS NOT NULL
        UNION ALL
        SELECT 'Files with Genres', COUNT(*) FROM files WHERE filename_genres IS NOT NULL AND array_length(filename_genres, 1) > 0
        UNION ALL
        SELECT 'Files with Structure Tags', COUNT(*) FROM files WHERE structure_tags IS NOT NULL AND array_length(structure_tags, 1) > 0
        UNION ALL
        SELECT 'Files with Complete Metadata', COUNT(*) FROM files_with_complete_metadata
        UNION ALL
        SELECT 'Files with Validated Metadata', COUNT(*) FROM files_with_validated_metadata
    )
    SELECT
        s.metric,
        s.count,
        ROUND((s.count::NUMERIC / GREATEST(v_total_files, 1)) * 100, 2) AS percentage
    FROM stats s;
END;
$$ LANGUAGE plpgsql STABLE;

COMMENT ON FUNCTION get_metadata_coverage_stats() IS
    'Returns metadata coverage statistics showing analyzed vs filename metadata availability';

-- ============================================================================
-- PART 7: Trigger to Auto-Update metadata_source Column
-- ============================================================================

-- Trigger function to automatically set metadata_source based on available data
CREATE OR REPLACE FUNCTION update_metadata_source()
RETURNS TRIGGER AS $$
DECLARE
    v_analyzed_bpm REAL;
    v_analyzed_key TEXT;
    v_has_analyzed BOOLEAN := FALSE;
    v_has_filename BOOLEAN := FALSE;
    v_is_validated BOOLEAN := FALSE;
BEGIN
    -- Get analyzed metadata if exists
    SELECT mm.bpm, mm.key_signature::TEXT
    INTO v_analyzed_bpm, v_analyzed_key
    FROM musical_metadata mm
    WHERE mm.file_id = NEW.id;

    -- Check what metadata exists
    v_has_analyzed := (v_analyzed_bpm IS NOT NULL OR v_analyzed_key IS NOT NULL);
    v_has_filename := (NEW.filename_bpm IS NOT NULL OR NEW.filename_key IS NOT NULL);

    -- Check if validated (both exist and match)
    IF v_has_analyzed AND v_has_filename THEN
        v_is_validated := (
            (v_analyzed_bpm IS NOT NULL AND NEW.filename_bpm IS NOT NULL AND ABS(v_analyzed_bpm - NEW.filename_bpm) <= 5)
            OR (v_analyzed_key IS NOT NULL AND NEW.filename_key IS NOT NULL AND v_analyzed_key = NEW.filename_key)
        );
    END IF;

    -- Set metadata_source
    IF v_is_validated THEN
        NEW.metadata_source := 'validated';
    ELSIF v_has_analyzed AND v_has_filename THEN
        NEW.metadata_source := 'both';
    ELSIF v_has_analyzed THEN
        NEW.metadata_source := 'analyzed';
    ELSIF v_has_filename THEN
        NEW.metadata_source := 'filename';
    ELSE
        NEW.metadata_source := 'none';
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_metadata_source
    BEFORE INSERT OR UPDATE OF filename_bpm, filename_key, filename_genres, structure_tags ON files
    FOR EACH ROW
    EXECUTE FUNCTION update_metadata_source();

COMMENT ON TRIGGER trigger_update_metadata_source ON files IS
    'Automatically updates metadata_source column based on available analyzed and filename metadata';

COMMIT;

-- ============================================================================
-- VERIFICATION QUERIES (run after migration)
-- ============================================================================

-- Check new columns exist
-- SELECT column_name, data_type, is_nullable
-- FROM information_schema.columns
-- WHERE table_name = 'files'
-- AND column_name IN ('filename_bpm', 'filename_key', 'filename_genres', 'structure_tags', 'track_number', 'metadata_source');

-- Check indexes created
-- SELECT indexname, indexdef
-- FROM pg_indexes
-- WHERE tablename = 'files'
-- AND indexname LIKE 'idx_files_filename%';

-- Get metadata coverage statistics
-- SELECT * FROM get_metadata_coverage_stats();

-- Find files with metadata conflicts
-- SELECT * FROM detect_metadata_conflicts() LIMIT 20;

-- Test enhanced search
-- SELECT * FROM search_files_with_metadata(
--     p_bpm_min := 120,
--     p_bpm_max := 130,
--     p_genres := ARRAY['house'],
--     p_limit := 10
-- );

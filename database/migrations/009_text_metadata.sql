-- Migration 009: Text Metadata Extraction from MIDI Files
-- Date: 2025-11-09
-- Purpose: Add columns to store text metadata extracted from MIDI file content
--
-- Metadata extracted includes:
-- - Track names (from MetaMessage::TrackName)
-- - Copyright notices (from MetaMessage::Copyright)
-- - Instrument names from text events (from MetaMessage::InstrumentName)
-- - Markers (section labels: Verse, Chorus, Bridge, etc.)
-- - Lyrics (for karaoke MIDI files)
--
-- This complements existing filename-based metadata from migration 008
-- and musical analysis metadata from the analysis phase.

BEGIN;

-- ============================================================================
-- PART 1: Add Text Metadata Columns to Files Table
-- ============================================================================

-- Track names from MIDI text events
ALTER TABLE files
    ADD COLUMN IF NOT EXISTS track_names TEXT[]
    DEFAULT '{}';

-- Copyright notice (usually in first track)
ALTER TABLE files
    ADD COLUMN IF NOT EXISTS copyright TEXT;

-- Instrument names from MIDI text events (distinct from program changes)
ALTER TABLE files
    ADD COLUMN IF NOT EXISTS instrument_names_text TEXT[]
    DEFAULT '{}';

-- Markers (section labels, rehearsal marks)
ALTER TABLE files
    ADD COLUMN IF NOT EXISTS markers TEXT[]
    DEFAULT '{}';

-- Lyrics (for karaoke files or vocal tracks)
ALTER TABLE files
    ADD COLUMN IF NOT EXISTS lyrics TEXT[]
    DEFAULT '{}';

-- Column comments for documentation
COMMENT ON COLUMN files.track_names IS
    'Track names extracted from MIDI MetaMessage::TrackName events';

COMMENT ON COLUMN files.copyright IS
    'Copyright notice extracted from MIDI MetaMessage::Copyright event';

COMMENT ON COLUMN files.instrument_names_text IS
    'Instrument names from MIDI MetaMessage::InstrumentName text events (distinct from GM program changes)';

COMMENT ON COLUMN files.markers IS
    'Markers and section labels (Verse, Chorus, Bridge) from MIDI MetaMessage::Marker events';

COMMENT ON COLUMN files.lyrics IS
    'Lyrics extracted from MIDI MetaMessage::Lyric events (karaoke files)';

-- ============================================================================
-- PART 2: Create Indexes for Performance
-- ============================================================================

-- Index on copyright for search queries
CREATE INDEX IF NOT EXISTS idx_files_copyright
    ON files(copyright)
    WHERE copyright IS NOT NULL;

-- GIN index on track_names array for fast "ANY" queries
CREATE INDEX IF NOT EXISTS idx_files_track_names
    ON files USING GIN(track_names)
    WHERE track_names IS NOT NULL AND array_length(track_names, 1) > 0;

-- GIN index on instrument_names_text array
CREATE INDEX IF NOT EXISTS idx_files_instrument_names_text
    ON files USING GIN(instrument_names_text)
    WHERE instrument_names_text IS NOT NULL AND array_length(instrument_names_text, 1) > 0;

-- GIN index on markers array for fast section search
CREATE INDEX IF NOT EXISTS idx_files_markers
    ON files USING GIN(markers)
    WHERE markers IS NOT NULL AND array_length(markers, 1) > 0;

-- Full-text search index on copyright
CREATE INDEX IF NOT EXISTS idx_files_copyright_trgm
    ON files USING gin(copyright gin_trgm_ops)
    WHERE copyright IS NOT NULL;

-- ============================================================================
-- PART 3: Helper Functions
-- ============================================================================

-- Function to search files by text metadata
CREATE OR REPLACE FUNCTION search_files_by_text_metadata(
    p_track_name TEXT DEFAULT NULL,
    p_copyright_search TEXT DEFAULT NULL,
    p_instrument_name TEXT DEFAULT NULL,
    p_marker TEXT DEFAULT NULL,
    p_has_lyrics BOOLEAN DEFAULT NULL,
    p_limit INTEGER DEFAULT 100,
    p_offset INTEGER DEFAULT 0
)
RETURNS TABLE (
    id BIGINT,
    filename TEXT,
    path TEXT,
    track_names TEXT[],
    copyright TEXT,
    instrument_names_text TEXT[],
    markers TEXT[],
    has_lyrics BOOLEAN,
    created_at TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        f.id,
        f.filename,
        f.filepath AS path,
        f.track_names,
        f.copyright,
        f.instrument_names_text,
        f.markers,
        (f.lyrics IS NOT NULL AND array_length(f.lyrics, 1) > 0) AS has_lyrics,
        f.created_at
    FROM files f
    WHERE
        (p_track_name IS NULL OR p_track_name = ANY(f.track_names))
        AND (p_copyright_search IS NULL OR f.copyright ILIKE '%' || p_copyright_search || '%')
        AND (p_instrument_name IS NULL OR p_instrument_name = ANY(f.instrument_names_text))
        AND (p_marker IS NULL OR p_marker = ANY(f.markers))
        AND (p_has_lyrics IS NULL OR
             (p_has_lyrics = (f.lyrics IS NOT NULL AND array_length(f.lyrics, 1) > 0)))
    ORDER BY f.created_at DESC
    LIMIT p_limit
    OFFSET p_offset;
END;
$$ LANGUAGE plpgsql STABLE;

COMMENT ON FUNCTION search_files_by_text_metadata IS
    'Search files by text metadata: track names, copyright, instruments, markers, and lyrics';

-- Function to get text metadata statistics
CREATE OR REPLACE FUNCTION get_text_metadata_stats()
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
        SELECT 'Files with Track Names', COUNT(*) FROM files WHERE array_length(track_names, 1) > 0
        UNION ALL
        SELECT 'Files with Copyright', COUNT(*) FROM files WHERE copyright IS NOT NULL
        UNION ALL
        SELECT 'Files with Instrument Names (Text)', COUNT(*) FROM files WHERE array_length(instrument_names_text, 1) > 0
        UNION ALL
        SELECT 'Files with Markers', COUNT(*) FROM files WHERE array_length(markers, 1) > 0
        UNION ALL
        SELECT 'Files with Lyrics', COUNT(*) FROM files WHERE array_length(lyrics, 1) > 0
        UNION ALL
        SELECT 'Files with Any Text Metadata', COUNT(*)
        FROM files
        WHERE array_length(track_names, 1) > 0
           OR copyright IS NOT NULL
           OR array_length(instrument_names_text, 1) > 0
           OR array_length(markers, 1) > 0
           OR array_length(lyrics, 1) > 0
    )
    SELECT
        s.metric,
        s.count,
        ROUND((s.count::NUMERIC / GREATEST(v_total_files, 1)) * 100, 2) AS percentage
    FROM stats s;
END;
$$ LANGUAGE plpgsql STABLE;

COMMENT ON FUNCTION get_text_metadata_stats() IS
    'Returns statistics about text metadata coverage in the files table';

-- ============================================================================
-- PART 4: Enhanced Search View
-- ============================================================================

-- View: Files with complete text metadata
CREATE OR REPLACE VIEW files_with_text_metadata AS
SELECT
    f.id,
    f.filename,
    f.filepath AS path,
    f.track_names,
    f.copyright,
    f.instrument_names_text,
    f.markers,
    f.lyrics,
    array_length(f.track_names, 1) AS track_count,
    array_length(f.instrument_names_text, 1) AS instrument_count,
    array_length(f.markers, 1) AS marker_count,
    array_length(f.lyrics, 1) AS lyric_line_count,
    f.created_at
FROM files f
WHERE array_length(f.track_names, 1) > 0
   OR f.copyright IS NOT NULL
   OR array_length(f.instrument_names_text, 1) > 0
   OR array_length(f.markers, 1) > 0
   OR array_length(f.lyrics, 1) > 0;

COMMENT ON VIEW files_with_text_metadata IS
    'Files that have any text metadata extracted from MIDI events';

COMMIT;

-- ============================================================================
-- VERIFICATION QUERIES (run after migration)
-- ============================================================================

-- Check new columns exist
-- SELECT column_name, data_type, is_nullable
-- FROM information_schema.columns
-- WHERE table_name = 'files'
-- AND column_name IN ('track_names', 'copyright', 'instrument_names_text', 'markers', 'lyrics');

-- Check indexes created
-- SELECT indexname, indexdef
-- FROM pg_indexes
-- WHERE tablename = 'files'
-- AND indexname LIKE 'idx_files_%text%' OR indexname LIKE 'idx_files_copyright%' OR indexname LIKE 'idx_files_track%' OR indexname LIKE 'idx_files_markers%';

-- Get text metadata statistics
-- SELECT * FROM get_text_metadata_stats();

-- Test search function
-- SELECT * FROM search_files_by_text_metadata(
--     p_track_name := 'Piano',
--     p_limit := 10
-- );

-- View files with text metadata
-- SELECT * FROM files_with_text_metadata LIMIT 20;

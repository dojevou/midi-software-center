-- =============================================================================
-- MIDI Software Center - VIP3 Filter Columns and Performance Indexes
-- =============================================================================
-- Migration: 022_add_filter_columns_and_indexes.sql
-- Version: 2.2
-- PostgreSQL: 16+
-- Date: 2025-12-15
-- Description: Adds denormalized filter columns to files table for fast filtering
--              and creates comprehensive indexes for <50ms filter count performance.
-- Prerequisites: Migration 019 (VIP3 filtering tables)
-- =============================================================================

-- Note: Cannot use BEGIN/COMMIT with CREATE INDEX CONCURRENTLY
-- This migration runs without transaction wrapping

-- =============================================================================
-- SECTION 1: ADD FILTER COLUMNS TO FILES TABLE
-- =============================================================================

-- Add folder_id for fast folder filtering
-- This will be populated from parent_folder via trigger/function
ALTER TABLE files
ADD COLUMN IF NOT EXISTS folder_id BIGINT;

-- Add bpm_range_id for fast BPM range filtering
-- This will be auto-populated based on musical_metadata.bpm
ALTER TABLE files
ADD COLUMN IF NOT EXISTS bpm_range_id SMALLINT REFERENCES bpm_ranges(id);

-- Add key_id for fast key filtering
-- This will be auto-populated based on musical_metadata.key_signature
ALTER TABLE files
ADD COLUMN IF NOT EXISTS key_id SMALLINT REFERENCES musical_keys(id);

-- Add channel_count for channel filtering
-- This will be populated from musical_metadata
ALTER TABLE files
ADD COLUMN IF NOT EXISTS channel_count SMALLINT DEFAULT 1;

-- =============================================================================
-- SECTION 2: PARTIAL INDEXES FOR FILTER COLUMNS
-- These indexes include only non-NULL values, reducing index size
-- =============================================================================

-- Folder filter index (most common filter)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_files_folder_id_partial
ON files(folder_id)
WHERE folder_id IS NOT NULL;

-- BPM range filter index
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_files_bpm_range_id_partial
ON files(bpm_range_id)
WHERE bpm_range_id IS NOT NULL;

-- Key filter index
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_files_key_id_partial
ON files(key_id)
WHERE key_id IS NOT NULL;

-- Channel count filter index
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_files_channel_count_partial
ON files(channel_count)
WHERE channel_count > 0;

-- =============================================================================
-- SECTION 3: COMPOSITE INDEXES FOR COMMON FILTER COMBINATIONS
-- These speed up multi-column filtering
-- =============================================================================

-- Folder + BPM range (common combination)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_files_folder_bpm_range
ON files(folder_id, bpm_range_id)
WHERE folder_id IS NOT NULL AND bpm_range_id IS NOT NULL;

-- Folder + Key (common combination)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_files_folder_key
ON files(folder_id, key_id)
WHERE folder_id IS NOT NULL AND key_id IS NOT NULL;

-- BPM range + Key (common combination)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_files_bpm_key
ON files(bpm_range_id, key_id)
WHERE bpm_range_id IS NOT NULL AND key_id IS NOT NULL;

-- =============================================================================
-- SECTION 4: JUNCTION TABLE INDEXES FOR VIP3 CATEGORIES
-- Speed up midi_file_timbres, midi_file_styles, midi_file_articulations joins
-- =============================================================================

-- Timbre filtering (both directions for different query patterns)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_midi_file_timbres_timbre_file
ON midi_file_timbres(timbre_id, file_id);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_midi_file_timbres_file_timbre
ON midi_file_timbres(file_id, timbre_id);

-- Style filtering (both directions)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_midi_file_styles_style_file
ON midi_file_styles(style_id, file_id);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_midi_file_styles_file_style
ON midi_file_styles(file_id, style_id);

-- Articulation filtering (both directions)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_midi_file_articulations_articulation_file
ON midi_file_articulations(articulation_id, file_id);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_midi_file_articulations_file_articulation
ON midi_file_articulations(file_id, articulation_id);

-- =============================================================================
-- SECTION 5: OPTIMIZE EXISTING FILE_TAGS INDEXES FOR INSTRUMENT FILTERING
-- =============================================================================

-- Add covering index for tag counts (include file_id to avoid table lookup)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_file_tags_tag_file_covering
ON file_tags(tag_id, file_id)
INCLUDE (file_id);

-- =============================================================================
-- SECTION 6: FUNCTION TO POPULATE BPM_RANGE_ID
-- =============================================================================

CREATE OR REPLACE FUNCTION update_file_bpm_range_id()
RETURNS TRIGGER AS $$
BEGIN
    -- Update bpm_range_id based on bpm value
    UPDATE files f
    SET bpm_range_id = br.id
    FROM musical_metadata mm
    JOIN bpm_ranges br ON mm.bpm >= br.min_bpm AND mm.bpm < br.max_bpm
    WHERE f.id = NEW.file_id
    AND mm.file_id = NEW.file_id
    AND mm.bpm IS NOT NULL;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update bpm_range_id when musical_metadata changes
DROP TRIGGER IF EXISTS trg_update_file_bpm_range ON musical_metadata;
CREATE TRIGGER trg_update_file_bpm_range
    AFTER INSERT OR UPDATE OF bpm ON musical_metadata
    FOR EACH ROW
    EXECUTE FUNCTION update_file_bpm_range_id();

-- =============================================================================
-- SECTION 7: FUNCTION TO POPULATE KEY_ID
-- =============================================================================

CREATE OR REPLACE FUNCTION update_file_key_id()
RETURNS TRIGGER AS $$
BEGIN
    -- Update key_id based on key_signature value
    UPDATE files f
    SET key_id = mk.id
    FROM musical_metadata mm
    JOIN musical_keys mk ON mm.key_signature::TEXT = mk.name
    WHERE f.id = NEW.file_id
    AND mm.file_id = NEW.file_id
    AND mm.key_signature IS NOT NULL;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update key_id when musical_metadata changes
DROP TRIGGER IF EXISTS trg_update_file_key ON musical_metadata;
CREATE TRIGGER trg_update_file_key
    AFTER INSERT OR UPDATE OF key_signature ON musical_metadata
    FOR EACH ROW
    EXECUTE FUNCTION update_file_key_id();

-- =============================================================================
-- SECTION 8: FUNCTION TO POPULATE CHANNEL_COUNT
-- =============================================================================

CREATE OR REPLACE FUNCTION update_file_channel_count()
RETURNS TRIGGER AS $$
BEGIN
    -- Count distinct MIDI channels from events
    UPDATE files f
    SET channel_count = (
        SELECT COUNT(DISTINCT me.channel)::SMALLINT
        FROM midi_events me
        JOIN midi_tracks mt ON me.track_id = mt.id
        WHERE mt.file_id = NEW.file_id
        AND me.channel IS NOT NULL
    )
    WHERE f.id = NEW.file_id;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to update channel_count when tracks are added
DROP TRIGGER IF EXISTS trg_update_file_channels ON midi_tracks;
CREATE TRIGGER trg_update_file_channels
    AFTER INSERT ON midi_tracks
    FOR EACH ROW
    EXECUTE FUNCTION update_file_channel_count();

-- =============================================================================
-- SECTION 9: POPULATE EXISTING DATA
-- =============================================================================

-- Populate bpm_range_id for existing files
UPDATE files f
SET bpm_range_id = br.id
FROM musical_metadata mm
JOIN bpm_ranges br ON mm.bpm >= br.min_bpm AND mm.bpm < br.max_bpm
WHERE f.id = mm.file_id
AND mm.bpm IS NOT NULL
AND f.bpm_range_id IS NULL;

-- Populate key_id for existing files
UPDATE files f
SET key_id = mk.id
FROM musical_metadata mm
JOIN musical_keys mk ON mm.key_signature::TEXT = mk.name
WHERE f.id = mm.file_id
AND mm.key_signature IS NOT NULL
AND f.key_id IS NULL;

-- Populate channel_count for existing files
UPDATE files f
SET channel_count = (
    SELECT COUNT(DISTINCT me.channel)::SMALLINT
    FROM midi_events me
    JOIN midi_tracks mt ON me.track_id = mt.id
    WHERE mt.file_id = f.id
    AND me.channel IS NOT NULL
)
WHERE f.channel_count IS NULL OR f.channel_count = 0;

-- =============================================================================
-- SECTION 10: VACUUM AND ANALYZE
-- Update table statistics for query planner
-- =============================================================================

VACUUM ANALYZE files;
VACUUM ANALYZE musical_metadata;
VACUUM ANALYZE file_tags;
VACUUM ANALYZE midi_file_timbres;
VACUUM ANALYZE midi_file_styles;
VACUUM ANALYZE midi_file_articulations;
VACUUM ANALYZE bpm_ranges;
VACUUM ANALYZE musical_keys;

-- Migration complete (no COMMIT since we're not in a transaction)

-- =============================================================================
-- VERIFICATION QUERIES
-- Run these to verify indexes are being used
-- =============================================================================

-- Check index usage for folder counts
-- EXPLAIN (ANALYZE, BUFFERS)
-- SELECT folder_id, COUNT(*)
-- FROM files
-- WHERE folder_id IS NOT NULL
-- GROUP BY folder_id;

-- Check index usage for BPM range counts
-- EXPLAIN (ANALYZE, BUFFERS)
-- SELECT bpm_range_id, COUNT(*)
-- FROM files
-- WHERE bpm_range_id IS NOT NULL
-- GROUP BY bpm_range_id;

-- Check index usage for timbre counts
-- EXPLAIN (ANALYZE, BUFFERS)
-- SELECT timbre_id, COUNT(DISTINCT file_id)
-- FROM midi_file_timbres
-- GROUP BY timbre_id;

-- Check index usage for instrument tag counts
-- EXPLAIN (ANALYZE, BUFFERS)
-- SELECT tag_id, COUNT(DISTINCT file_id)
-- FROM file_tags
-- GROUP BY tag_id;

-- =============================================================================
-- INDEX SIZE REPORT
-- Monitor index sizes to ensure they're reasonable
-- =============================================================================

-- SELECT
--     schemaname,
--     tablename,
--     indexname,
--     pg_size_pretty(pg_relation_size(indexrelid)) AS index_size
-- FROM pg_stat_user_indexes
-- WHERE schemaname = 'public'
--     AND (indexname LIKE 'idx_files_%' OR indexname LIKE 'idx_midi_file_%')
-- ORDER BY pg_relation_size(indexrelid) DESC;

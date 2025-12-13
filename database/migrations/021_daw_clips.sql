-- =============================================================================
-- MIDI Library System - DAW Clips Migration
-- =============================================================================
-- Migration: 021_daw_clips.sql
-- Version: 1.0
-- PostgreSQL: 16+
-- Date: 2025-12-11
-- Description: Adds clips table for sequencer timeline (Pro Tools-style clips)
-- =============================================================================

BEGIN;

-- =============================================================================
-- DAW CLIPS TABLE
-- =============================================================================

-- -----------------------------------------------------------------------------
-- TABLE: daw_clips
-- PURPOSE: Clips positioned on tracks in the sequencer timeline
-- EXPECTED ROWS: 500,000+
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS daw_clips (
    id BIGSERIAL PRIMARY KEY,
    track_id BIGINT NOT NULL REFERENCES daw_tracks(id) ON DELETE CASCADE,

    -- Clip identification
    name TEXT NOT NULL,
    color TEXT DEFAULT '#3B82F6',

    -- Timeline position (in ticks)
    start_tick BIGINT NOT NULL DEFAULT 0 CHECK (start_tick >= 0),
    duration_ticks BIGINT NOT NULL DEFAULT 1920 CHECK (duration_ticks > 0),

    -- Source reference - can be a file from the library
    source_file_id BIGINT REFERENCES files(id) ON DELETE SET NULL,

    -- Clip state
    is_muted BOOLEAN DEFAULT FALSE,
    is_selected BOOLEAN DEFAULT FALSE,
    is_locked BOOLEAN DEFAULT FALSE,

    -- Playback offsets (for trimmed clips)
    source_start_tick BIGINT DEFAULT 0 CHECK (source_start_tick >= 0),
    source_end_tick BIGINT, -- NULL means use full length

    -- Gain adjustment (dB)
    gain_db NUMERIC(5, 2) DEFAULT 0.0,

    -- Loop settings within clip
    loop_enabled BOOLEAN DEFAULT FALSE,
    loop_start_tick BIGINT DEFAULT 0,
    loop_end_tick BIGINT,

    -- Metadata
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- INDEXES
-- =============================================================================

CREATE INDEX IF NOT EXISTS idx_daw_clips_track ON daw_clips(track_id);
CREATE INDEX IF NOT EXISTS idx_daw_clips_source ON daw_clips(source_file_id) WHERE source_file_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_daw_clips_position ON daw_clips(track_id, start_tick);
CREATE INDEX IF NOT EXISTS idx_daw_clips_range ON daw_clips(track_id, start_tick, (start_tick + duration_ticks));
CREATE INDEX IF NOT EXISTS idx_daw_clips_selected ON daw_clips(track_id, is_selected) WHERE is_selected = TRUE;

-- =============================================================================
-- TRIGGERS
-- =============================================================================

-- Update timestamps for clips
CREATE TRIGGER IF NOT EXISTS daw_clips_updated_at
    BEFORE UPDATE ON daw_clips
    FOR EACH ROW
    EXECUTE FUNCTION update_daw_project_timestamp();

-- =============================================================================
-- COMMENTS
-- =============================================================================

COMMENT ON TABLE daw_clips IS 'Clips positioned on tracks in the sequencer timeline';
COMMENT ON COLUMN daw_clips.source_file_id IS 'Reference to MIDI file in library (for audio-style clips)';
COMMENT ON COLUMN daw_clips.source_start_tick IS 'Start offset within source for trimmed clips';
COMMENT ON COLUMN daw_clips.source_end_tick IS 'End offset within source for trimmed clips (NULL = full length)';

-- =============================================================================
-- VIEWS
-- =============================================================================

-- View for clips with track and file info
CREATE OR REPLACE VIEW daw_clips_with_info AS
SELECT
    c.id,
    c.track_id,
    c.name as clip_name,
    c.color,
    c.start_tick,
    c.duration_ticks,
    c.source_file_id,
    c.is_muted,
    c.is_selected,
    c.is_locked,
    c.gain_db,
    t.name as track_name,
    t.project_id,
    t.track_number,
    t.channel,
    f.filename as source_filename,
    f.bpm as source_bpm,
    f.key_signature as source_key
FROM daw_clips c
JOIN daw_tracks t ON c.track_id = t.id
LEFT JOIN files f ON c.source_file_id = f.id;

-- =============================================================================
-- RECORD MIGRATION
-- =============================================================================

INSERT INTO schema_migrations (version, description)
VALUES ('021', 'DAW clips table for sequencer timeline')
ON CONFLICT (version) DO NOTHING;

-- =============================================================================
-- VERIFY MIGRATION
-- =============================================================================

DO $$
DECLARE
    table_exists BOOLEAN;
    index_count INTEGER;
BEGIN
    -- Check table exists
    SELECT EXISTS (
        SELECT FROM information_schema.tables
        WHERE table_schema = 'public'
        AND table_name = 'daw_clips'
    ) INTO table_exists;

    -- Count indexes
    SELECT COUNT(*) INTO index_count
    FROM pg_indexes
    WHERE schemaname = 'public'
    AND indexname LIKE 'idx_daw_clips_%';

    RAISE NOTICE 'Migration 021 verification:';
    RAISE NOTICE '  daw_clips table exists: %', table_exists;
    RAISE NOTICE '  Clip indexes created: %', index_count;

    IF NOT table_exists THEN
        RAISE EXCEPTION 'daw_clips table was not created';
    END IF;

    RAISE NOTICE 'Migration 021 completed successfully';
END $$;

COMMIT;

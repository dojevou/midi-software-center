-- Migration: 006_track_splits.sql
-- Purpose: Track relationships between parent multi-track MIDI files and their split single-track versions
-- Date: 2025-10-12
-- Author: Auto-generated via Claude Code

-- ============================================================================
-- TABLE: track_splits
-- ============================================================================
-- Description: Maps parent MIDI files to their split single-track versions
-- Use case: When a multi-track MIDI file is split into individual tracks,
--           this table maintains the relationship and metadata about each split.
-- ============================================================================

CREATE TABLE track_splits (
    id BIGSERIAL PRIMARY KEY,
    parent_file_id BIGINT NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    split_file_id BIGINT NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    track_number INT NOT NULL,
    track_name TEXT,
    instrument TEXT,
    note_count INT,
    created_at TIMESTAMP DEFAULT NOW(),

    -- Constraint: Prevent duplicate parent-split pairs
    CONSTRAINT uq_parent_split_pair UNIQUE(parent_file_id, split_file_id),

    -- Constraint: Track number must be non-negative
    CONSTRAINT chk_track_number_positive CHECK (track_number >= 0),

    -- Constraint: Note count must be non-negative if specified
    CONSTRAINT chk_note_count_positive CHECK (note_count IS NULL OR note_count >= 0)
);

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Index for finding all splits of a parent file
-- Query pattern: SELECT * FROM track_splits WHERE parent_file_id = ?
CREATE INDEX idx_track_splits_parent_file_id
    ON track_splits(parent_file_id);

-- Index for finding the parent of a split file
-- Query pattern: SELECT * FROM track_splits WHERE split_file_id = ?
CREATE INDEX idx_track_splits_split_file_id
    ON track_splits(split_file_id);

-- Composite index for ordering splits by track number within a parent
-- Query pattern: SELECT * FROM track_splits WHERE parent_file_id = ? ORDER BY track_number
CREATE INDEX idx_track_splits_parent_track_number
    ON track_splits(parent_file_id, track_number);

-- Index for finding splits by instrument
-- Query pattern: SELECT * FROM track_splits WHERE instrument = ?
CREATE INDEX idx_track_splits_instrument
    ON track_splits(instrument)
    WHERE instrument IS NOT NULL;

-- ============================================================================
-- COMMENTS (Documentation)
-- ============================================================================

-- Table comment
COMMENT ON TABLE track_splits IS
    'Maps parent multi-track MIDI files to their split single-track versions. When a MIDI file with multiple tracks is split into individual files, this table maintains the relationship and metadata about each extracted track.';

-- Column comments
COMMENT ON COLUMN track_splits.id IS
    'Primary key - auto-incrementing unique identifier';

COMMENT ON COLUMN track_splits.parent_file_id IS
    'Foreign key to files table - the original multi-track MIDI file. Cascades on delete.';

COMMENT ON COLUMN track_splits.split_file_id IS
    'Foreign key to files table - the split single-track MIDI file. Cascades on delete.';

COMMENT ON COLUMN track_splits.track_number IS
    'Track number in the parent file (0-indexed). Corresponds to the MIDI track index.';

COMMENT ON COLUMN track_splits.track_name IS
    'Name of the track from MIDI metadata (Track Name meta event). May be NULL if unnamed.';

COMMENT ON COLUMN track_splits.instrument IS
    'Instrument/program name from MIDI Program Change events or meta events. May be NULL if not specified.';

COMMENT ON COLUMN track_splits.note_count IS
    'Number of note events (Note On messages) in this track. Useful for filtering empty or sparse tracks.';

COMMENT ON COLUMN track_splits.created_at IS
    'Timestamp when this split relationship was created. Defaults to current time.';

-- ============================================================================
-- MIGRATION COMPLETE
-- ============================================================================
-- Next steps:
-- 1. Apply this migration: psql -U postgres -d midi_library -f 006_track_splits.sql
-- 2. Verify with: SELECT * FROM track_splits LIMIT 1;
-- 3. Check indexes: SELECT indexname FROM pg_indexes WHERE tablename = 'track_splits';
-- ============================================================================

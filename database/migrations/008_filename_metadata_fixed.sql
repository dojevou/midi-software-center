-- Migration 008: Filename Metadata Extraction (Fixed)
-- Date: 2025-11-11
-- Purpose: Add filename-based metadata columns only
--
-- This adds columns to extract metadata from filenames:
-- - BPM from filename patterns
-- - Key signatures from filenames
-- - Genre tags from filenames
-- - Structure tags (loop, fill, etc.)
-- - Track numbers

BEGIN;

-- Add filename metadata columns to files table
ALTER TABLE files
    ADD COLUMN IF NOT EXISTS filename_bpm REAL
    CHECK (filename_bpm IS NULL OR (filename_bpm BETWEEN 30 AND 300));

ALTER TABLE files
    ADD COLUMN IF NOT EXISTS filename_key TEXT
    CHECK (filename_key IS NULL OR LENGTH(filename_key) BETWEEN 1 AND 3);

ALTER TABLE files
    ADD COLUMN IF NOT EXISTS filename_genres TEXT[];

ALTER TABLE files
    ADD COLUMN IF NOT EXISTS structure_tags TEXT[];

ALTER TABLE files
    ADD COLUMN IF NOT EXISTS metadata_source TEXT
    CHECK (metadata_source IN ('analyzed', 'filename', 'both', 'none', 'validated'))
    DEFAULT 'none';

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_files_filename_bpm
    ON files(filename_bpm)
    WHERE filename_bpm IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_files_filename_key
    ON files(filename_key)
    WHERE filename_key IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_files_filename_genres
    ON files USING GIN(filename_genres)
    WHERE filename_genres IS NOT NULL AND array_length(filename_genres, 1) > 0;

CREATE INDEX IF NOT EXISTS idx_files_structure_tags
    ON files USING GIN(structure_tags)
    WHERE structure_tags IS NOT NULL AND array_length(structure_tags, 1) > 0;

CREATE INDEX IF NOT EXISTS idx_files_metadata_source
    ON files(metadata_source);

COMMIT;

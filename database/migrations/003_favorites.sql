-- Migration: Add favorites table for user favorites system
-- Date: 2025-10-15
-- Purpose: Allow users to favorite/star files for quick access

BEGIN;

-- Create favorites table
CREATE TABLE IF NOT EXISTS favorites (
    id BIGSERIAL PRIMARY KEY,
    file_id BIGINT NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW(),

    -- Ensure uniqueness - one favorite entry per file
    UNIQUE(file_id)
);

-- Create indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_favorites_file_id ON favorites(file_id);
CREATE INDEX IF NOT EXISTS idx_favorites_created_at ON favorites(created_at DESC);

-- Add comment for documentation
COMMENT ON TABLE favorites IS 'User-favorited MIDI files for quick access';
COMMENT ON COLUMN favorites.file_id IS 'Reference to the favorited file';

-- Record migration
INSERT INTO schema_migrations (version, description)
VALUES ('003', 'Add favorites table for user favorites system')
ON CONFLICT (version) DO NOTHING;

COMMIT;

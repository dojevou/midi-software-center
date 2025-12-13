-- Migration 020: Add rating column to files table
-- This allows users to rate files 1-5 stars (stored as integers)

-- Add rating column to files table
ALTER TABLE files ADD COLUMN IF NOT EXISTS rating SMALLINT CHECK (rating >= 1 AND rating <= 5);

-- Create index for filtering by rating
CREATE INDEX IF NOT EXISTS idx_files_rating ON files (rating) WHERE rating IS NOT NULL;

-- Add comments
COMMENT ON COLUMN files.rating IS 'User rating from 1-5 stars (NULL means unrated)';

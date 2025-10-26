-- Migration: Add parent_folder column to files table
-- Date: 2025-10-13
-- Purpose: Store parent directory name for better file categorization

-- Add parent_folder column
ALTER TABLE files ADD COLUMN IF NOT EXISTS parent_folder TEXT;

-- Create index for faster filtering by folder
CREATE INDEX IF NOT EXISTS idx_files_parent_folder ON files(parent_folder) WHERE parent_folder IS NOT NULL;

-- Add comment for documentation
COMMENT ON COLUMN files.parent_folder IS 'Name of the parent directory containing this file (e.g., "bass", "leads", "drums")';

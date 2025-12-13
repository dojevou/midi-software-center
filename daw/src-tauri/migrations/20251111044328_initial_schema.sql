-- Add migration script here
-- Create the files table for storing MIDI file metadata
CREATE TABLE IF NOT EXISTS files (
    id BIGSERIAL PRIMARY KEY,
    file_path TEXT NOT NULL UNIQUE,
    file_name TEXT NOT NULL,
    bpm REAL DEFAULT 120.0,
    key_signature TEXT DEFAULT 'C Major',
    tags JSONB DEFAULT '[]'::JSONB,
    duration REAL DEFAULT 0.0,
    track_count INTEGER DEFAULT 1,
    file_size BIGINT DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_files_bpm ON files (bpm);
CREATE INDEX IF NOT EXISTS idx_files_key ON files (key_signature);
CREATE INDEX IF NOT EXISTS idx_files_tags ON files USING GIN (tags);

-- Trigger to update the updated_at column automatically
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_files_updated_at BEFORE UPDATE
    ON files FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Optional: Create a view for search results
CREATE OR REPLACE VIEW search_results AS
SELECT 
    id,
    file_name,
    bpm,
    key_signature,
    tags,
    duration,
    track_count,
    file_size,
    created_at,
    updated_at
FROM files
WHERE file_path IS NOT NULL;

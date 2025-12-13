-- Migration: Enhanced MIDI Analysis JSON Fields
-- Date: 2025-11-20
-- Description: Add JSON fields for controller, articulation, and structure analysis
--
-- This migration adds three new JSONB columns to the musical_metadata table:
-- 1. controller_data - CC message statistics (modulation, volume, pan, etc.)
-- 2. articulation_data - Performance characteristics (legato, staccato, humanization, etc.)
-- 3. structure_data - Musical form analysis (patterns, repetition, form estimation)

-- Add controller_data column (JSONB for flexibility)
ALTER TABLE musical_metadata
ADD COLUMN IF NOT EXISTS controller_data JSONB DEFAULT NULL;

-- Add articulation_data column
ALTER TABLE musical_metadata
ADD COLUMN IF NOT EXISTS articulation_data JSONB DEFAULT NULL;

-- Add structure_data column
ALTER TABLE musical_metadata
ADD COLUMN IF NOT EXISTS structure_data JSONB DEFAULT NULL;

-- Add comments to describe the schema
COMMENT ON COLUMN musical_metadata.controller_data IS
'JSON array of MIDI controller (CC) statistics. Each entry contains: cc (number), name (string), count (int), min (int), max (int), avg (float). Example: [{"cc": 1, "name": "Modulation Wheel", "count": 10, "min": 0, "max": 127, "avg": 64.5}]';

COMMENT ON COLUMN musical_metadata.articulation_data IS
'JSON object of articulation and performance characteristics. Fields: legato_percentage (float), staccato_percentage (float), avg_note_duration_ms (float), timing_deviation_ms (float), dynamic_range (int 0-127), velocity_variance (float), is_humanized (bool), is_legato (bool), is_staccato (bool)';

COMMENT ON COLUMN musical_metadata.structure_data IS
'JSON object of musical structure and form analysis. Fields: num_segments (int), num_unique_patterns (int), num_repeated_patterns (int), repetition_percentage (float), estimated_form (string: "Simple (AA or AB)", "Song Form (AABA or ABAB)", "Complex (ABABCB or similar)", "Through-composed"), has_repetition (bool), is_through_composed (bool)';

-- Create GIN indexes for efficient JSON queries
CREATE INDEX IF NOT EXISTS idx_musical_metadata_controller_data ON musical_metadata USING GIN (controller_data);
CREATE INDEX IF NOT EXISTS idx_musical_metadata_articulation_data ON musical_metadata USING GIN (articulation_data);
CREATE INDEX IF NOT EXISTS idx_musical_metadata_structure_data ON musical_metadata USING GIN (structure_data);

-- Example queries enabled by these indexes:

-- Find files with high modulation usage:
-- SELECT file_id FROM musical_metadata
-- WHERE controller_data @> '[{"cc": 1}]'::jsonb
--   AND (controller_data->0->>'count')::int > 50;

-- Find legato performances:
-- SELECT file_id FROM musical_metadata
-- WHERE (articulation_data->>'is_legato')::boolean = true
--   AND (articulation_data->>'legato_percentage')::float > 50.0;

-- Find files with AABA form:
-- SELECT file_id FROM musical_metadata
-- WHERE structure_data->>'estimated_form' LIKE '%AABA%';

-- Find files with high repetition:
-- SELECT file_id FROM musical_metadata
-- WHERE (structure_data->>'repetition_percentage')::float > 70.0;

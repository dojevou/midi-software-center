-- =============================================================================
-- MIDI Software Center - Filter Count Performance Optimization
-- =============================================================================
-- Migration: 023_filter_count_optimization.sql
-- Date: 2025-12-15
-- Description: Adds essential indexes for VIP3 filter count performance
--              Target: <50ms for all filter count queries
-- Prerequisites: None (works with existing schema)
-- Note: CREATE INDEX IF NOT EXISTS is safe - will skip if index exists
-- =============================================================================

-- =============================================================================
-- SECTION 1: OPTIMIZE FILE_TAGS FOR INSTRUMENT FILTERING
-- =============================================================================

-- Covering index for tag counts (avoid table lookups)
CREATE INDEX IF NOT EXISTS idx_file_tags_tag_file_covering_opt
ON file_tags(tag_id)
INCLUDE (file_id);

-- Reverse index for file-based lookups
CREATE INDEX IF NOT EXISTS idx_file_tags_file_tag_covering_opt
ON file_tags(file_id)
INCLUDE (tag_id);

-- =============================================================================
-- SECTION 2: MUSICAL_METADATA INDEXES FOR RANGE QUERIES
-- =============================================================================

-- BPM range filtering
CREATE INDEX IF NOT EXISTS idx_musical_metadata_bpm_range_opt
ON musical_metadata(bpm)
WHERE bpm IS NOT NULL;

-- BPM with file_id for joins
CREATE INDEX IF NOT EXISTS idx_musical_metadata_file_bpm_opt
ON musical_metadata(file_id, bpm)
WHERE bpm IS NOT NULL;

-- Key signature filtering
CREATE INDEX IF NOT EXISTS idx_musical_metadata_key_range_opt
ON musical_metadata(key_signature)
WHERE key_signature IS NOT NULL;

-- Key with file_id for joins
CREATE INDEX IF NOT EXISTS idx_musical_metadata_file_key_opt
ON musical_metadata(file_id, key_signature)
WHERE key_signature IS NOT NULL;

-- Channel count filtering
CREATE INDEX IF NOT EXISTS idx_musical_metadata_channels_opt
ON musical_metadata(channel_count)
WHERE channel_count IS NOT NULL;

-- =============================================================================
-- SECTION 3: FILES TABLE INDEXES FOR COMMON FILTERS
-- =============================================================================

-- Parent folder filtering (approximation for folder_id)
CREATE INDEX IF NOT EXISTS idx_files_parent_folder_opt
ON files(parent_folder)
WHERE parent_folder IS NOT NULL;

-- Collection filtering
CREATE INDEX IF NOT EXISTS idx_files_collection_name_opt
ON files(collection_name)
WHERE collection_name IS NOT NULL;

-- Multi-track filtering
CREATE INDEX IF NOT EXISTS idx_files_is_multi_track_opt
ON files(is_multi_track)
WHERE is_multi_track = true;

-- Track layer filtering (single vs multi-track)
CREATE INDEX IF NOT EXISTS idx_files_num_tracks_opt
ON files(num_tracks);

-- =============================================================================
-- SECTION 4: COMPOSITE INDEXES FOR COMMON FILTER COMBINATIONS
-- =============================================================================

-- Parent folder + file_id (for joined queries)
CREATE INDEX IF NOT EXISTS idx_files_parent_folder_id_opt
ON files(parent_folder, id)
WHERE parent_folder IS NOT NULL;

-- Collection + file_id (for joined queries)
CREATE INDEX IF NOT EXISTS idx_files_collection_id_opt
ON files(collection_name, id)
WHERE collection_name IS NOT NULL;

-- =============================================================================
-- SECTION 5: JUNCTION TABLE INDEXES (IF TABLES EXIST)
-- Note: These will error if tables don't exist yet, but that's okay
-- =============================================================================

-- Timbre filtering
CREATE INDEX IF NOT EXISTS idx_midi_file_timbres_timbre_file_opt
ON midi_file_timbres(timbre_id, file_id);

CREATE INDEX IF NOT EXISTS idx_midi_file_timbres_file_timbre_opt
ON midi_file_timbres(file_id, timbre_id);

-- Style filtering
CREATE INDEX IF NOT EXISTS idx_midi_file_styles_style_file_opt
ON midi_file_styles(style_id, file_id);

CREATE INDEX IF NOT EXISTS idx_midi_file_styles_file_style_opt
ON midi_file_styles(file_id, style_id);

-- Articulation filtering
CREATE INDEX IF NOT EXISTS idx_midi_file_articulations_articulation_file_opt
ON midi_file_articulations(articulation_id, file_id);

CREATE INDEX IF NOT EXISTS idx_midi_file_articulations_file_articulation_opt
ON midi_file_articulations(file_id, articulation_id);

-- =============================================================================
-- SECTION 6: UPDATE TABLE STATISTICS
-- =============================================================================

VACUUM ANALYZE files;
VACUUM ANALYZE musical_metadata;
VACUUM ANALYZE file_tags;
VACUUM ANALYZE tags;

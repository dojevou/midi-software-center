-- Migration 024: Saved Searches, Collections, and Favorites
-- Purpose: Add support for saving VIP3 search filters, creating collections, and favoriting files
-- Estimated time: Instant (schema only)

-- =====================================================================
-- 1. SAVED SEARCHES
-- =====================================================================

CREATE TABLE saved_searches (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    filters JSONB NOT NULL,  -- VIP3Filters serialized as JSON
    sort_by TEXT,
    sort_order TEXT,
    icon TEXT,
    color TEXT,
    is_pinned BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_used TIMESTAMPTZ,
    use_count INTEGER DEFAULT 0
);

CREATE INDEX idx_saved_searches_name ON saved_searches(name);
CREATE INDEX idx_saved_searches_created ON saved_searches(created_at DESC);
CREATE INDEX idx_saved_searches_pinned ON saved_searches(is_pinned DESC, last_used DESC NULLS LAST);
CREATE INDEX idx_saved_searches_last_used ON saved_searches(last_used DESC NULLS LAST);

-- GIN index for searching within filters
CREATE INDEX idx_saved_searches_filters ON saved_searches USING GIN (filters);

-- =====================================================================
-- 2. COLLECTIONS
-- =====================================================================

CREATE TABLE collections (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    icon TEXT,  -- Icon name/emoji (e.g., '🎹', '🥁', 'folder')
    color TEXT,  -- Hex color for UI (e.g., '#FF5733')
    is_smart BOOLEAN DEFAULT FALSE,  -- Smart collections use saved search filters
    created_at TIMESTAMPTZ DEFAULT NOW(),
    modified_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_collections_name ON collections(name);
CREATE INDEX idx_collections_created ON collections(created_at DESC);

-- Junction table for collection membership
CREATE TABLE collection_files (
    id BIGSERIAL PRIMARY KEY,
    collection_id BIGINT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    midi_file_id BIGINT NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    sort_order INTEGER,  -- For manual ordering
    added_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(collection_id, midi_file_id)
);

CREATE INDEX idx_collection_files_collection ON collection_files(collection_id, sort_order);
CREATE INDEX idx_collection_files_file ON collection_files(midi_file_id);
CREATE INDEX idx_collection_files_added ON collection_files(added_at DESC);

-- =====================================================================
-- 3. FAVORITES
-- =====================================================================

-- Favorites are handled by the files.favorite boolean column
-- No separate table needed - commands use files.favorite directly

-- =====================================================================
-- 4. HELPER FUNCTIONS
-- =====================================================================

-- Function to get files in a collection
CREATE OR REPLACE FUNCTION get_collection_files(p_collection_id BIGINT)
RETURNS TABLE (
    file_id BIGINT,
    filename TEXT,
    file_path TEXT,
    bpm REAL,
    key_signature TEXT,
    sort_order INTEGER,
    added_at TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        f.id,
        f.filename,
        f.file_path,
        mm.bpm,
        mm.key_signature,
        cf.sort_order,
        cf.added_at
    FROM collection_files cf
    JOIN files f ON cf.midi_file_id = f.id
    LEFT JOIN musical_metadata mm ON f.id = mm.file_id
    WHERE cf.collection_id = p_collection_id
      AND f.deleted_at IS NULL
    ORDER BY cf.sort_order ASC NULLS LAST, cf.added_at ASC;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to get all favorites (using files.favorite column)
CREATE OR REPLACE FUNCTION get_favorite_files()
RETURNS TABLE (
    file_id BIGINT,
    filename TEXT,
    file_path TEXT,
    bpm REAL,
    key_signature TEXT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        f.id,
        f.filename,
        f.file_path,
        mm.bpm,
        mm.key_signature
    FROM files f
    LEFT JOIN musical_metadata mm ON f.id = mm.file_id
    WHERE f.favorite = TRUE
      AND f.deleted_at IS NULL
    ORDER BY f.updated_at DESC;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to count files in a collection
CREATE OR REPLACE FUNCTION count_collection_files(p_collection_id BIGINT)
RETURNS INTEGER AS $$
DECLARE
    file_count INTEGER;
BEGIN
    SELECT COUNT(*)::INTEGER INTO file_count
    FROM collection_files cf
    JOIN files f ON cf.midi_file_id = f.id
    WHERE cf.collection_id = p_collection_id
      AND f.deleted_at IS NULL;

    RETURN file_count;
END;
$$ LANGUAGE plpgsql STABLE;

-- =====================================================================
-- 5. TRIGGERS
-- =====================================================================

-- Update modified_at on collections
CREATE OR REPLACE FUNCTION update_collections_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.modified_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_collections_update
BEFORE UPDATE ON collections
FOR EACH ROW
EXECUTE FUNCTION update_collections_timestamp();

-- =====================================================================
-- 6. INITIAL DATA (Optional)
-- =====================================================================

-- Example saved search (commented out - uncomment if needed)
-- INSERT INTO saved_searches (name, description, filters) VALUES
-- ('Piano Jazz 120 BPM', 'Jazz piano files around 120 BPM',
--  '{"instruments": ["piano", "jazz"], "bpm_min": 118, "bpm_max": 122}');

-- Example collection (commented out - uncomment if needed)
-- INSERT INTO collections (name, description, color) VALUES
-- ('Drum Loops', 'Collection of my favorite drum loops', '#FF5733');

COMMENT ON TABLE saved_searches IS 'Saved VIP3 search filter configurations';
COMMENT ON TABLE collections IS 'User-created collections for organizing MIDI files';
COMMENT ON TABLE collection_files IS 'Junction table linking files to collections with ordering';

COMMENT ON COLUMN saved_searches.filters IS 'JSON representation of BrowserFilters struct';
COMMENT ON COLUMN collection_files.sort_order IS 'User-defined sort order within collection';
COMMENT ON COLUMN collections.color IS 'UI color for collection visualization (hex format)';
COMMENT ON COLUMN collections.icon IS 'Icon/emoji for collection visualization';

-- Migration 012: Tag Relationships and Hierarchy
-- Ported from ~/midi tag_manager patterns
-- Run: psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -f database/migrations/012_tag_relationships.sql

BEGIN;

-- ============================================================
-- 1. Add confidence to file_tags (matches ~/midi/api/db_models.py)
-- ============================================================
ALTER TABLE file_tags
ADD COLUMN IF NOT EXISTS confidence FLOAT DEFAULT 1.0;

ALTER TABLE file_tags
ADD COLUMN IF NOT EXISTS source VARCHAR(32) DEFAULT 'unknown';
-- Sources: 'filename', 'folder', 'pack', 'ml_model', 'user', 'midi_analysis'

ALTER TABLE file_tags
ADD COLUMN IF NOT EXISTS created_at TIMESTAMP DEFAULT NOW();

COMMENT ON COLUMN file_tags.confidence IS 'Tag confidence 0.0-1.0 (unified scale)';
COMMENT ON COLUMN file_tags.source IS 'How the tag was assigned';

-- ============================================================
-- 2. Tag Relationships Table (parent-child, synonyms, related)
-- ============================================================
CREATE TABLE IF NOT EXISTS tag_relationships (
    id SERIAL PRIMARY KEY,
    parent_tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    child_tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    relationship_type VARCHAR(16) NOT NULL CHECK (relationship_type IN ('parent', 'synonym', 'related')),
    strength FLOAT DEFAULT 1.0 CHECK (strength >= 0.0 AND strength <= 1.0),
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(parent_tag_id, child_tag_id, relationship_type)
);

CREATE INDEX IF NOT EXISTS idx_tag_rel_parent ON tag_relationships(parent_tag_id);
CREATE INDEX IF NOT EXISTS idx_tag_rel_child ON tag_relationships(child_tag_id);
CREATE INDEX IF NOT EXISTS idx_tag_rel_type ON tag_relationships(relationship_type);

COMMENT ON TABLE tag_relationships IS 'Hierarchical and semantic tag relationships';
COMMENT ON COLUMN tag_relationships.relationship_type IS 'parent=hierarchy, synonym=alias, related=association';

-- ============================================================
-- 3. Rename History Table (from ~/midi rename_service)
-- ============================================================
CREATE TABLE IF NOT EXISTS rename_history (
    id SERIAL PRIMARY KEY,
    file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    old_filename VARCHAR(512) NOT NULL,
    new_filename VARCHAR(512) NOT NULL,
    old_filepath TEXT,
    new_filepath TEXT,
    rename_batch_id UUID,
    renamed_at TIMESTAMP DEFAULT NOW(),
    reversed_at TIMESTAMP,
    tags_snapshot JSONB
);

CREATE INDEX IF NOT EXISTS idx_rename_file ON rename_history(file_id);
CREATE INDEX IF NOT EXISTS idx_rename_batch ON rename_history(rename_batch_id);
CREATE INDEX IF NOT EXISTS idx_rename_old ON rename_history(old_filename);

COMMENT ON TABLE rename_history IS 'Track file renames for rollback capability';

-- ============================================================
-- 4. Helper Functions
-- ============================================================

-- Get all child tags (recursive hierarchy traversal)
CREATE OR REPLACE FUNCTION get_child_tags(parent_name TEXT)
RETURNS TABLE(tag_id INTEGER, tag_name VARCHAR, depth INTEGER) AS $$
WITH RECURSIVE tag_tree AS (
    SELECT t.id, t.name, 0 as depth
    FROM tags t
    WHERE t.name = parent_name

    UNION ALL

    SELECT t.id, t.name, tt.depth + 1
    FROM tags t
    JOIN tag_relationships tr ON t.id = tr.child_tag_id
    JOIN tag_tree tt ON tr.parent_tag_id = tt.id
    WHERE tr.relationship_type = 'parent'
    AND tt.depth < 10  -- prevent infinite recursion
)
SELECT id, name, depth FROM tag_tree WHERE depth > 0;
$$ LANGUAGE SQL STABLE;

-- Get all synonym tags
CREATE OR REPLACE FUNCTION get_synonyms(tag_name TEXT)
RETURNS TABLE(synonym_id INTEGER, synonym_name VARCHAR) AS $$
SELECT t.id, t.name
FROM tags t
JOIN tag_relationships tr ON t.id = tr.child_tag_id OR t.id = tr.parent_tag_id
WHERE tr.relationship_type = 'synonym'
AND EXISTS (
    SELECT 1 FROM tags orig
    WHERE orig.name = tag_name
    AND (orig.id = tr.parent_tag_id OR orig.id = tr.child_tag_id)
)
AND t.name != tag_name;
$$ LANGUAGE SQL STABLE;

-- Search files including synonyms and children
CREATE OR REPLACE FUNCTION search_files_with_hierarchy(search_tag TEXT, min_confidence FLOAT DEFAULT 0.5)
RETURNS TABLE(file_id INTEGER, filename VARCHAR, filepath TEXT, confidence FLOAT, matched_tag VARCHAR) AS $$
SELECT DISTINCT f.id, f.filename, f.filepath, ft.confidence, t.name as matched_tag
FROM files f
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE ft.confidence >= min_confidence
AND (
    t.name = search_tag
    OR t.id IN (SELECT tag_id FROM get_child_tags(search_tag))
    OR t.id IN (SELECT synonym_id FROM get_synonyms(search_tag))
)
ORDER BY ft.confidence DESC;
$$ LANGUAGE SQL STABLE;

-- Smart filename generator (ported from ~/midi rename_service)
CREATE OR REPLACE FUNCTION generate_smart_filename(
    file_id INTEGER,
    template TEXT DEFAULT '{bpm}bpm_{key}_{genre}_{instrument}_{original}'
) RETURNS TEXT AS $$
DECLARE
    result TEXT;
    file_rec RECORD;
    meta_rec RECORD;
    top_tags RECORD;
BEGIN
    -- Get file info
    SELECT filename, filepath INTO file_rec FROM files WHERE id = file_id;

    -- Get metadata
    SELECT bpm, key_signature, duration_seconds INTO meta_rec
    FROM musical_metadata WHERE file_id = file_id;

    -- Get top tags by category
    SELECT
        (SELECT t.name FROM tags t JOIN file_tags ft ON t.id = ft.tag_id
         WHERE ft.file_id = file_id AND t.category = 'genre'
         ORDER BY ft.confidence DESC LIMIT 1) as genre,
        (SELECT t.name FROM tags t JOIN file_tags ft ON t.id = ft.tag_id
         WHERE ft.file_id = file_id AND t.category = 'instrument'
         ORDER BY ft.confidence DESC LIMIT 1) as instrument,
        (SELECT t.name FROM tags t JOIN file_tags ft ON t.id = ft.tag_id
         WHERE ft.file_id = file_id AND t.category = 'mood'
         ORDER BY ft.confidence DESC LIMIT 1) as mood
    INTO top_tags;

    -- Build filename from template
    result := template;
    result := REPLACE(result, '{bpm}', COALESCE(meta_rec.bpm::TEXT, 'unk'));
    result := REPLACE(result, '{key}', COALESCE(LOWER(meta_rec.key_signature), 'unk'));
    result := REPLACE(result, '{genre}', COALESCE(LOWER(top_tags.genre), 'unk'));
    result := REPLACE(result, '{instrument}', COALESCE(LOWER(top_tags.instrument), 'unk'));
    result := REPLACE(result, '{mood}', COALESCE(LOWER(top_tags.mood), 'unk'));
    result := REPLACE(result, '{duration}', COALESCE(meta_rec.duration_seconds::INTEGER::TEXT, '0'));
    result := REPLACE(result, '{original}', REGEXP_REPLACE(file_rec.filename, '\.[^.]+$', ''));

    -- Sanitize
    result := REGEXP_REPLACE(result, '[^a-zA-Z0-9_-]', '_', 'g');
    result := REGEXP_REPLACE(result, '_+', '_', 'g');
    result := result || '.mid';

    RETURN result;
END;
$$ LANGUAGE plpgsql STABLE;

COMMIT;

-- ============================================================
-- Verification
-- ============================================================
DO $$
BEGIN
    RAISE NOTICE 'Migration 012 complete:';
    RAISE NOTICE '  - file_tags.confidence column added';
    RAISE NOTICE '  - file_tags.source column added';
    RAISE NOTICE '  - tag_relationships table created';
    RAISE NOTICE '  - rename_history table created';
    RAISE NOTICE '  - get_child_tags() function created';
    RAISE NOTICE '  - get_synonyms() function created';
    RAISE NOTICE '  - search_files_with_hierarchy() function created';
    RAISE NOTICE '  - generate_smart_filename() function created';
END $$;

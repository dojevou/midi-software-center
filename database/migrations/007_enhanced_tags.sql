-- Migration 007: Enhanced Tags Schema for Auto-Tagging
-- Date: 2025-11-08
-- Purpose: Add priority, confidence, and auto-detection fields to tags system

BEGIN;

-- ============================================================================
-- PART 1: Tag Categories Table
-- ============================================================================

-- Create tag categories lookup table
CREATE TABLE IF NOT EXISTS tag_categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    description TEXT,
    priority INTEGER DEFAULT 50,  -- Lower number = higher priority (10-90)
    color VARCHAR(7),              -- Hex color for UI (e.g., '#3498db')
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert tag categories with priorities
INSERT INTO tag_categories (name, description, priority, color) VALUES
('genre', 'Musical genre classification (EDM, Hip Hop, Jazz, etc.)', 10, '#3498db'),
('instrument', 'Instruments and sound sources', 20, '#2ecc71'),
('element', 'Musical elements and structure', 30, '#e67e22'),
('key', 'Musical key and scale', 40, '#9b59b6'),
('tempo', 'BPM and tempo classification', 50, '#e74c3c'),
('mood', 'Emotional and atmospheric qualities', 60, '#f39c12'),
('technical', 'Technical and production attributes', 70, '#95a5a6'),
('structure', 'Song structure components', 80, '#1abc9c'),
('library', 'Manufacturer and library identifiers', 85, '#34495e'),
('world', 'World music regions and cultures', 90, '#d35400')
ON CONFLICT (name) DO NOTHING;

-- ============================================================================
-- PART 2: Enhance Tags Table
-- ============================================================================

-- Add new columns to tags table
ALTER TABLE tags
    ADD COLUMN IF NOT EXISTS category_id INTEGER REFERENCES tag_categories(id),
    ADD COLUMN IF NOT EXISTS priority INTEGER DEFAULT 50,
    ADD COLUMN IF NOT EXISTS auto_detected BOOLEAN DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS confidence_score DECIMAL(3,2) DEFAULT 0.00 CHECK (confidence_score BETWEEN 0 AND 1),
    ADD COLUMN IF NOT EXISTS detection_method VARCHAR(50),
    ADD COLUMN IF NOT EXISTS parent_tag_id INTEGER REFERENCES tags(id),
    ADD COLUMN IF NOT EXISTS is_active BOOLEAN DEFAULT TRUE;

-- Add comment explaining detection methods
COMMENT ON COLUMN tags.detection_method IS
'Detection method: pack_exact, folder_exact, filename_pattern, bpm_detection, key_detection, contextual, mood_inference';

-- ============================================================================
-- PART 3: Tag Aliases Table (for search variations)
-- ============================================================================

CREATE TABLE IF NOT EXISTS tag_aliases (
    id SERIAL PRIMARY KEY,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    alias VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tag_id, alias)
);

CREATE INDEX IF NOT EXISTS idx_tag_aliases_alias ON tag_aliases(alias);
CREATE INDEX IF NOT EXISTS idx_tag_aliases_tag_id ON tag_aliases(tag_id);

-- ============================================================================
-- PART 4: Auto-Tagging Rules Table
-- ============================================================================

CREATE TABLE IF NOT EXISTS auto_tagging_rules (
    id SERIAL PRIMARY KEY,
    rule_name VARCHAR(100) NOT NULL,
    rule_type VARCHAR(50) NOT NULL, -- 'pack', 'folder', 'filename', 'bpm', 'key', 'contextual'
    pattern VARCHAR(500) NOT NULL,   -- Regex or simple pattern
    tags_to_add INTEGER[] NOT NULL,  -- Array of tag IDs
    confidence DECIMAL(3,2) DEFAULT 0.85,
    priority INTEGER DEFAULT 50,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_auto_tagging_rules_type ON auto_tagging_rules(rule_type);
CREATE INDEX IF NOT EXISTS idx_auto_tagging_rules_active ON auto_tagging_rules(is_active);

COMMENT ON TABLE auto_tagging_rules IS
'Auto-tagging rules that match patterns in pack/folder/file names and automatically apply tags';

-- ============================================================================
-- PART 5: Tag Suggestions Table (ML/user feedback)
-- ============================================================================

CREATE TABLE IF NOT EXISTS tag_suggestions (
    id SERIAL PRIMARY KEY,
    file_id BIGINT NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    suggested_tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    confidence DECIMAL(3,2) NOT NULL,
    source VARCHAR(50) NOT NULL, -- 'auto', 'ml', 'user_feedback', 'similar_files'
    is_accepted BOOLEAN,
    accepted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(file_id, suggested_tag_id)
);

CREATE INDEX IF NOT EXISTS idx_tag_suggestions_file_id ON tag_suggestions(file_id);
CREATE INDEX IF NOT EXISTS idx_tag_suggestions_confidence ON tag_suggestions(confidence DESC);
CREATE INDEX IF NOT EXISTS idx_tag_suggestions_accepted ON tag_suggestions(is_accepted);

-- ============================================================================
-- PART 6: Update Existing Indexes
-- ============================================================================

-- Add index on tags category_id
CREATE INDEX IF NOT EXISTS idx_tags_category_id ON tags(category_id);

-- Add index on tags priority
CREATE INDEX IF NOT EXISTS idx_tags_priority ON tags(priority);

-- Add index on tags auto_detected
CREATE INDEX IF NOT EXISTS idx_tags_auto_detected ON tags(auto_detected);

-- Improve file_tags index for common queries
CREATE INDEX IF NOT EXISTS idx_file_tags_tag_id_file_id ON file_tags(tag_id, file_id);

-- ============================================================================
-- PART 7: Insert Core Tags with Categories
-- ============================================================================

-- Helper function to get or create tag
CREATE OR REPLACE FUNCTION insert_tag_with_category(
    p_name VARCHAR(100),
    p_category VARCHAR(50),
    p_priority INTEGER DEFAULT 50,
    p_auto_detected BOOLEAN DEFAULT TRUE,
    p_confidence DECIMAL(3,2) DEFAULT 0.95
) RETURNS INTEGER AS $$
DECLARE
    v_category_id INTEGER;
    v_tag_id INTEGER;
BEGIN
    -- Get category ID
    SELECT id INTO v_category_id FROM tag_categories WHERE name = p_category;

    -- Insert or update tag
    INSERT INTO tags (name, category_id, priority, auto_detected, confidence_score, usage_count)
    VALUES (p_name, v_category_id, p_priority, p_auto_detected, p_confidence, 0)
    ON CONFLICT (name)
    DO UPDATE SET
        category_id = EXCLUDED.category_id,
        priority = EXCLUDED.priority,
        auto_detected = EXCLUDED.auto_detected,
        confidence_score = EXCLUDED.confidence_score
    RETURNING id INTO v_tag_id;

    RETURN v_tag_id;
END;
$$ LANGUAGE plpgsql;

-- Insert genre tags (priority 10)
DO $$
BEGIN
    -- Electronic/EDM
    PERFORM insert_tag_with_category('dubstep', 'genre', 10, true, 0.95);
    PERFORM insert_tag_with_category('house', 'genre', 10, true, 0.95);
    PERFORM insert_tag_with_category('deep-house', 'genre', 10, true, 0.95);
    PERFORM insert_tag_with_category('techno', 'genre', 10, true, 0.95);
    PERFORM insert_tag_with_category('trap', 'genre', 10, true, 0.95);
    PERFORM insert_tag_with_category('future-bass', 'genre', 10, true, 0.95);
    PERFORM insert_tag_with_category('dnb', 'genre', 10, true, 0.95);
    PERFORM insert_tag_with_category('drum-and-bass', 'genre', 10, true, 0.95);
    PERFORM insert_tag_with_category('psy-trance', 'genre', 10, true, 0.95);
    PERFORM insert_tag_with_category('trance', 'genre', 10, true, 0.95);
    PERFORM insert_tag_with_category('glitch', 'genre', 10, true, 0.95);
    PERFORM insert_tag_with_category('ambient', 'genre', 10, true, 0.95);
    PERFORM insert_tag_with_category('edm', 'genre', 10, true, 0.90);

    -- Urban/Contemporary
    PERFORM insert_tag_with_category('hip-hop', 'genre', 10, true, 0.95);
    PERFORM insert_tag_with_category('rap', 'genre', 10, true, 0.95);
    PERFORM insert_tag_with_category('rnb', 'genre', 10, true, 0.95);
    PERFORM insert_tag_with_category('pop', 'genre', 10, true, 0.95);
    PERFORM insert_tag_with_category('soul', 'genre', 10, true, 0.95);

    -- Traditional/Acoustic
    PERFORM insert_tag_with_category('jazz', 'genre', 10, true, 0.95);
    PERFORM insert_tag_with_category('rock', 'genre', 10, true, 0.95);
    PERFORM insert_tag_with_category('cinematic', 'genre', 10, true, 0.95);
    PERFORM insert_tag_with_category('classical', 'genre', 10, true, 0.95);

    -- World genres
    PERFORM insert_tag_with_category('world', 'world', 10, true, 0.90);
    PERFORM insert_tag_with_category('traditional', 'world', 10, true, 0.90);
END $$;

-- Insert instrument tags (priority 20)
DO $$
BEGIN
    -- Drums/Percussion
    PERFORM insert_tag_with_category('drums', 'instrument', 20, true, 0.95);
    PERFORM insert_tag_with_category('kick', 'instrument', 20, true, 0.95);
    PERFORM insert_tag_with_category('snare', 'instrument', 20, true, 0.95);
    PERFORM insert_tag_with_category('hat', 'instrument', 20, true, 0.95);
    PERFORM insert_tag_with_category('hihat', 'instrument', 20, true, 0.95);
    PERFORM insert_tag_with_category('ride', 'instrument', 20, true, 0.95);
    PERFORM insert_tag_with_category('tom', 'instrument', 20, true, 0.95);
    PERFORM insert_tag_with_category('clap', 'instrument', 20, true, 0.95);
    PERFORM insert_tag_with_category('percussion', 'instrument', 20, true, 0.95);

    -- Bass/Synth
    PERFORM insert_tag_with_category('bass', 'instrument', 20, true, 0.95);
    PERFORM insert_tag_with_category('synth', 'instrument', 20, true, 0.95);
    PERFORM insert_tag_with_category('lead', 'instrument', 20, true, 0.95);
    PERFORM insert_tag_with_category('pad', 'instrument', 20, true, 0.95);

    -- Melodic
    PERFORM insert_tag_with_category('piano', 'instrument', 20, true, 0.95);
    PERFORM insert_tag_with_category('strings', 'instrument', 20, true, 0.95);
    PERFORM insert_tag_with_category('guitar', 'instrument', 20, true, 0.95);

    -- World instruments
    PERFORM insert_tag_with_category('djembe', 'instrument', 20, true, 0.95);
    PERFORM insert_tag_with_category('tabla', 'instrument', 20, true, 0.95);
    PERFORM insert_tag_with_category('darabuka', 'instrument', 20, true, 0.95);
    PERFORM insert_tag_with_category('conga', 'instrument', 20, true, 0.95);
    PERFORM insert_tag_with_category('bongo', 'instrument', 20, true, 0.95);
END $$;

-- Insert musical element tags (priority 30)
DO $$
BEGIN
    PERFORM insert_tag_with_category('chords', 'element', 30, true, 0.90);
    PERFORM insert_tag_with_category('melody', 'element', 30, true, 0.90);
    PERFORM insert_tag_with_category('bassline', 'element', 30, true, 0.90);
    PERFORM insert_tag_with_category('loop', 'element', 30, true, 0.90);
    PERFORM insert_tag_with_category('arpeggio', 'element', 30, true, 0.90);
    PERFORM insert_tag_with_category('progression', 'element', 30, true, 0.90);
END $$;

-- Insert song structure tags (priority 80)
DO $$
BEGIN
    PERFORM insert_tag_with_category('intro', 'structure', 80, true, 0.90);
    PERFORM insert_tag_with_category('verse', 'structure', 80, true, 0.90);
    PERFORM insert_tag_with_category('chorus', 'structure', 80, true, 0.90);
    PERFORM insert_tag_with_category('bridge', 'structure', 80, true, 0.90);
    PERFORM insert_tag_with_category('outro', 'structure', 80, true, 0.90);
    PERFORM insert_tag_with_category('breakdown', 'structure', 80, true, 0.90);
END $$;

-- Insert tempo tags (priority 50)
DO $$
BEGIN
    PERFORM insert_tag_with_category('slow', 'tempo', 50, true, 0.85);
    PERFORM insert_tag_with_category('mid-tempo', 'tempo', 50, true, 0.85);
    PERFORM insert_tag_with_category('upbeat', 'tempo', 50, true, 0.85);
    PERFORM insert_tag_with_category('fast', 'tempo', 50, true, 0.85);
    PERFORM insert_tag_with_category('very-fast', 'tempo', 50, true, 0.85);
END $$;

-- ============================================================================
-- PART 8: Useful Views
-- ============================================================================

-- View: Tags with category information
CREATE OR REPLACE VIEW tags_with_categories AS
SELECT
    t.id,
    t.name,
    t.usage_count,
    tc.name as category,
    tc.color as category_color,
    t.priority,
    t.auto_detected,
    t.confidence_score,
    t.detection_method,
    t.is_active,
    t.created_at
FROM tags t
LEFT JOIN tag_categories tc ON t.category_id = tc.id
WHERE t.is_active = true
ORDER BY tc.priority, t.priority, t.usage_count DESC;

-- View: Popular tags by category
CREATE OR REPLACE VIEW popular_tags_by_category AS
SELECT
    tc.name as category,
    tc.color as category_color,
    t.name as tag_name,
    t.usage_count,
    t.auto_detected,
    t.confidence_score
FROM tags t
JOIN tag_categories tc ON t.category_id = tc.id
WHERE t.is_active = true AND t.usage_count > 0
ORDER BY tc.priority, t.usage_count DESC;

-- View: Tag suggestions awaiting review
CREATE OR REPLACE VIEW pending_tag_suggestions AS
SELECT
    ts.id,
    ts.file_id,
    f.filename,
    t.name as suggested_tag,
    tc.name as tag_category,
    ts.confidence,
    ts.source,
    ts.created_at
FROM tag_suggestions ts
JOIN files f ON ts.file_id = f.id
JOIN tags t ON ts.suggested_tag_id = t.id
LEFT JOIN tag_categories tc ON t.category_id = tc.id
WHERE ts.is_accepted IS NULL
ORDER BY ts.confidence DESC, ts.created_at DESC;

-- ============================================================================
-- PART 9: Helper Functions
-- ============================================================================

-- Function to get tags by category
CREATE OR REPLACE FUNCTION get_tags_by_category(p_category VARCHAR(50))
RETURNS TABLE (
    tag_id INTEGER,
    tag_name VARCHAR(100),
    usage_count INTEGER,
    confidence DECIMAL(3,2)
) AS $$
BEGIN
    RETURN QUERY
    SELECT t.id, t.name, t.usage_count, t.confidence_score
    FROM tags t
    JOIN tag_categories tc ON t.category_id = tc.id
    WHERE tc.name = p_category AND t.is_active = true
    ORDER BY t.usage_count DESC, t.name;
END;
$$ LANGUAGE plpgsql;

-- Function to suggest tags for a file based on similar files
CREATE OR REPLACE FUNCTION suggest_tags_from_similar_files(p_file_id BIGINT, p_limit INTEGER DEFAULT 5)
RETURNS TABLE (
    tag_id INTEGER,
    tag_name VARCHAR(100),
    confidence DECIMAL(3,2),
    usage_frequency BIGINT
) AS $$
BEGIN
    RETURN QUERY
    WITH similar_files AS (
        -- Find files with similar metadata (same BPM, key, duration range)
        SELECT f2.id
        FROM files f1
        JOIN files f2 ON f2.id != f1.id
        LEFT JOIN musical_metadata mm1 ON f1.id = mm1.file_id
        LEFT JOIN musical_metadata mm2 ON f2.id = mm2.file_id
        WHERE f1.id = p_file_id
        AND (
            ABS((mm1.bpm::NUMERIC - mm2.bpm::NUMERIC)) < 5
            OR mm1.key_signature = mm2.key_signature
            OR ABS((f1.duration_seconds::NUMERIC - f2.duration_seconds::NUMERIC)) < 10
        )
        LIMIT 100
    ),
    common_tags AS (
        SELECT ft.tag_id, COUNT(*) as frequency
        FROM file_tags ft
        WHERE ft.file_id IN (SELECT id FROM similar_files)
        AND ft.tag_id NOT IN (SELECT tag_id FROM file_tags WHERE file_id = p_file_id)
        GROUP BY ft.tag_id
    )
    SELECT
        t.id,
        t.name,
        LEAST(0.95, (ct.frequency::DECIMAL / 100.0))::DECIMAL(3,2) as confidence,
        ct.frequency
    FROM common_tags ct
    JOIN tags t ON ct.tag_id = t.id
    WHERE t.is_active = true
    ORDER BY ct.frequency DESC, t.usage_count DESC
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- PART 10: Cleanup
-- ============================================================================

-- Drop the helper function (no longer needed)
DROP FUNCTION IF EXISTS insert_tag_with_category(VARCHAR, VARCHAR, INTEGER, BOOLEAN, DECIMAL);

COMMIT;

-- ============================================================================
-- VERIFICATION QUERIES (run after migration)
-- ============================================================================

-- Check tag categories
-- SELECT * FROM tag_categories ORDER BY priority;

-- Check tags with categories
-- SELECT * FROM tags_with_categories LIMIT 20;

-- Check auto-tagging rules
-- SELECT * FROM auto_tagging_rules WHERE is_active = true;

-- Test tag suggestions
-- SELECT * FROM suggest_tags_from_similar_files(1, 10);

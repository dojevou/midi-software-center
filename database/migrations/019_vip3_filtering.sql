-- =============================================================================
-- MIDI Software Center - VIP3 Filtering Tables Migration
-- =============================================================================
-- Migration: 019_vip3_filtering.sql
-- Version: 2.1
-- PostgreSQL: 16+
-- Date: 2025-12-11
-- Description: Adds VIP3-style category filtering tables (timbres, styles, 
--              articulations, bpm_ranges, musical_keys) with many-to-many
--              relationships, plus collections and saved searches.
-- =============================================================================

BEGIN;

-- =============================================================================
-- SECTION 1: CORE CATEGORY TABLES
-- =============================================================================

-- Timbres (sound character descriptors)
CREATE TABLE IF NOT EXISTS timbres (
    id          SMALLSERIAL PRIMARY KEY,
    name        VARCHAR(50) NOT NULL UNIQUE,
    sort_order  SMALLINT DEFAULT 0,
    is_system   BOOLEAN DEFAULT TRUE,
    file_count  INTEGER DEFAULT 0,
    icon        VARCHAR(50),
    color       VARCHAR(7)
);

-- Styles / Genres
CREATE TABLE IF NOT EXISTS styles (
    id          SMALLSERIAL PRIMARY KEY,
    name        VARCHAR(50) NOT NULL UNIQUE,
    sort_order  SMALLINT DEFAULT 0,
    is_system   BOOLEAN DEFAULT TRUE,
    file_count  INTEGER DEFAULT 0,
    icon        VARCHAR(50),
    color       VARCHAR(7)
);

-- Articulations (how notes are played)
CREATE TABLE IF NOT EXISTS articulations (
    id          SMALLSERIAL PRIMARY KEY,
    name        VARCHAR(50) NOT NULL UNIQUE,
    sort_order  SMALLINT DEFAULT 0,
    is_system   BOOLEAN DEFAULT TRUE,
    file_count  INTEGER DEFAULT 0,
    icon        VARCHAR(50),
    color       VARCHAR(7)
);

-- BPM Ranges (for fast filtering)
CREATE TABLE IF NOT EXISTS bpm_ranges (
    id          SMALLSERIAL PRIMARY KEY,
    label       VARCHAR(20) NOT NULL UNIQUE,
    min_bpm     SMALLINT NOT NULL,
    max_bpm     SMALLINT NOT NULL,
    sort_order  SMALLINT,
    file_count  INTEGER DEFAULT 0
);

-- Musical Keys
CREATE TABLE IF NOT EXISTS musical_keys (
    id          SMALLSERIAL PRIMARY KEY,
    name        VARCHAR(10) NOT NULL UNIQUE,
    root_note   VARCHAR(2) NOT NULL,
    mode        VARCHAR(10) NOT NULL,
    sort_order  SMALLINT,
    file_count  INTEGER DEFAULT 0
);

-- =============================================================================
-- SECTION 2: MANY-TO-MANY JOIN TABLES
-- =============================================================================

-- File <-> Timbre (many-to-many)
CREATE TABLE IF NOT EXISTS midi_file_timbres (
    file_id     BIGINT REFERENCES files(id) ON DELETE CASCADE,
    timbre_id   SMALLINT REFERENCES timbres(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (file_id, timbre_id)
);

-- File <-> Style (many-to-many)
CREATE TABLE IF NOT EXISTS midi_file_styles (
    file_id     BIGINT REFERENCES files(id) ON DELETE CASCADE,
    style_id    SMALLINT REFERENCES styles(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (file_id, style_id)
);

-- File <-> Articulation (many-to-many)
CREATE TABLE IF NOT EXISTS midi_file_articulations (
    file_id         BIGINT REFERENCES files(id) ON DELETE CASCADE,
    articulation_id SMALLINT REFERENCES articulations(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (file_id, articulation_id)
);

-- =============================================================================
-- SECTION 3: USER DATA TABLES
-- =============================================================================

-- Saved searches (like VIP3)
CREATE TABLE IF NOT EXISTS saved_searches (
    id          BIGSERIAL PRIMARY KEY,
    name        VARCHAR(100) NOT NULL,
    description TEXT,
    filters     JSONB NOT NULL,
    sort_by     VARCHAR(50) DEFAULT 'name',
    sort_order  VARCHAR(4) DEFAULT 'asc',
    icon        VARCHAR(50),
    color       VARCHAR(7),
    is_pinned   BOOLEAN DEFAULT FALSE,
    created_at  TIMESTAMPTZ DEFAULT NOW(),
    last_used   TIMESTAMPTZ,
    use_count   INTEGER DEFAULT 0
);

-- Recent searches
CREATE TABLE IF NOT EXISTS recent_searches (
    id          BIGSERIAL PRIMARY KEY,
    query       TEXT NOT NULL,
    filters     JSONB,
    result_count INTEGER,
    searched_at TIMESTAMPTZ DEFAULT NOW()
);

-- Collections / Playlists
CREATE TABLE IF NOT EXISTS collections (
    id          BIGSERIAL PRIMARY KEY,
    name        VARCHAR(100) NOT NULL,
    description TEXT,
    icon        VARCHAR(50),
    color       VARCHAR(7),
    is_smart    BOOLEAN DEFAULT FALSE,
    smart_filters JSONB,
    file_count  INTEGER DEFAULT 0,
    created_at  TIMESTAMPTZ DEFAULT NOW(),
    modified_at TIMESTAMPTZ DEFAULT NOW()
);

-- Collection <-> File (many-to-many)
CREATE TABLE IF NOT EXISTS collection_files (
    collection_id   BIGINT REFERENCES collections(id) ON DELETE CASCADE,
    file_id         BIGINT REFERENCES files(id) ON DELETE CASCADE,
    sort_order      INTEGER,
    added_at        TIMESTAMPTZ DEFAULT NOW(),
    notes           TEXT,
    PRIMARY KEY (collection_id, file_id)
);

-- =============================================================================
-- SECTION 4: ADD COLUMNS TO FILES TABLE (if not exists)
-- =============================================================================

-- Add bpm_range_id to files if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'files' AND column_name = 'bpm_range_id'
    ) THEN
        ALTER TABLE files ADD COLUMN bpm_range_id SMALLINT REFERENCES bpm_ranges(id);
    END IF;
END $$;

-- Add key_id to files if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'files' AND column_name = 'key_id'
    ) THEN
        ALTER TABLE files ADD COLUMN key_id SMALLINT REFERENCES musical_keys(id);
    END IF;
END $$;

-- Add search_vector for full-text search if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'files' AND column_name = 'search_vector'
    ) THEN
        ALTER TABLE files ADD COLUMN search_vector TSVECTOR;
    END IF;
END $$;

-- =============================================================================
-- SECTION 5: INDEXES
-- =============================================================================

-- Category table indexes
CREATE INDEX IF NOT EXISTS idx_timbres_sort ON timbres(sort_order);
CREATE INDEX IF NOT EXISTS idx_styles_sort ON styles(sort_order);
CREATE INDEX IF NOT EXISTS idx_articulations_sort ON articulations(sort_order);
CREATE INDEX IF NOT EXISTS idx_bpm_ranges_sort ON bpm_ranges(sort_order);
CREATE INDEX IF NOT EXISTS idx_musical_keys_sort ON musical_keys(sort_order);

-- Many-to-many indexes (both directions for efficient lookups)
CREATE INDEX IF NOT EXISTS idx_file_timbres_file ON midi_file_timbres(file_id);
CREATE INDEX IF NOT EXISTS idx_file_timbres_timbre ON midi_file_timbres(timbre_id);
CREATE INDEX IF NOT EXISTS idx_file_styles_file ON midi_file_styles(file_id);
CREATE INDEX IF NOT EXISTS idx_file_styles_style ON midi_file_styles(style_id);
CREATE INDEX IF NOT EXISTS idx_file_articulations_file ON midi_file_articulations(file_id);
CREATE INDEX IF NOT EXISTS idx_file_articulations_articulation ON midi_file_articulations(articulation_id);

-- Files table new column indexes
CREATE INDEX IF NOT EXISTS idx_files_bpm_range ON files(bpm_range_id) WHERE bpm_range_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_files_key ON files(key_id) WHERE key_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_files_search_vector ON files USING GIN(search_vector) WHERE search_vector IS NOT NULL;

-- User data indexes
CREATE INDEX IF NOT EXISTS idx_saved_searches_pinned ON saved_searches(is_pinned) WHERE is_pinned = TRUE;
CREATE INDEX IF NOT EXISTS idx_saved_searches_last_used ON saved_searches(last_used DESC);
CREATE INDEX IF NOT EXISTS idx_recent_searches_time ON recent_searches(searched_at DESC);
CREATE INDEX IF NOT EXISTS idx_collections_name ON collections(name);
CREATE INDEX IF NOT EXISTS idx_collection_files_collection ON collection_files(collection_id);
CREATE INDEX IF NOT EXISTS idx_collection_files_file ON collection_files(file_id);

-- Composite index for common VIP3 filter combinations
CREATE INDEX IF NOT EXISTS idx_files_vip3_filter ON files(bpm_range_id, key_id) 
    WHERE bpm_range_id IS NOT NULL OR key_id IS NOT NULL;

-- =============================================================================
-- SECTION 6: POPULATE DEFAULT DATA
-- =============================================================================

-- Timbres (21 sound character descriptors)
INSERT INTO timbres (name, sort_order, is_system) VALUES
    ('Aggressive', 1, TRUE),
    ('Airy', 2, TRUE),
    ('Bright', 3, TRUE),
    ('Clean', 4, TRUE),
    ('Dark', 5, TRUE),
    ('Dirty', 6, TRUE),
    ('Distorted', 7, TRUE),
    ('Fat', 8, TRUE),
    ('Gritty', 9, TRUE),
    ('Hard', 10, TRUE),
    ('Metallic', 11, TRUE),
    ('Muted', 12, TRUE),
    ('Punchy', 13, TRUE),
    ('Soft', 14, TRUE),
    ('Thin', 15, TRUE),
    ('Warm', 16, TRUE),
    ('Wide', 17, TRUE),
    ('Analog', 18, TRUE),
    ('Digital', 19, TRUE),
    ('Organic', 20, TRUE),
    ('Synthetic', 21, TRUE)
ON CONFLICT (name) DO NOTHING;

-- Styles (24 genres)
INSERT INTO styles (name, sort_order, is_system) VALUES
    ('Ambient', 1, TRUE),
    ('Cinematic', 2, TRUE),
    ('Dance', 3, TRUE),
    ('EDM', 4, TRUE),
    ('Funk', 5, TRUE),
    ('Hip-Hop', 6, TRUE),
    ('House', 7, TRUE),
    ('Jazz', 8, TRUE),
    ('Latin', 9, TRUE),
    ('Lo-Fi', 10, TRUE),
    ('Metal', 11, TRUE),
    ('Pop', 12, TRUE),
    ('R&B', 13, TRUE),
    ('Reggae', 14, TRUE),
    ('Rock', 15, TRUE),
    ('Soul', 16, TRUE),
    ('Techno', 17, TRUE),
    ('Trap', 18, TRUE),
    ('Orchestral', 19, TRUE),
    ('Electronic', 20, TRUE),
    ('Acoustic', 21, TRUE),
    ('World', 22, TRUE),
    ('Experimental', 23, TRUE),
    ('Chillout', 24, TRUE)
ON CONFLICT (name) DO NOTHING;

-- Articulations (20 playing styles)
INSERT INTO articulations (name, sort_order, is_system) VALUES
    ('Arpeggio', 1, TRUE),
    ('Chord', 2, TRUE),
    ('Fill', 3, TRUE),
    ('Loop', 4, TRUE),
    ('Melody', 5, TRUE),
    ('One-Shot', 6, TRUE),
    ('Phrase', 7, TRUE),
    ('Riff', 8, TRUE),
    ('Stab', 9, TRUE),
    ('Sustain', 10, TRUE),
    ('Sequence', 11, TRUE),
    ('Pattern', 12, TRUE),
    ('Groove', 13, TRUE),
    ('Break', 14, TRUE),
    ('Intro', 15, TRUE),
    ('Outro', 16, TRUE),
    ('Verse', 17, TRUE),
    ('Chorus', 18, TRUE),
    ('Bridge', 19, TRUE),
    ('Drop', 20, TRUE)
ON CONFLICT (name) DO NOTHING;

-- BPM Ranges (8 ranges)
INSERT INTO bpm_ranges (label, min_bpm, max_bpm, sort_order) VALUES
    ('0-60', 0, 60, 1),
    ('60-80', 60, 80, 2),
    ('80-100', 80, 100, 3),
    ('100-120', 100, 120, 4),
    ('120-140', 120, 140, 5),
    ('140-160', 140, 160, 6),
    ('160-180', 160, 180, 7),
    ('180+', 180, 999, 8)
ON CONFLICT (label) DO NOTHING;

-- Musical Keys (24 keys - all major and minor)
INSERT INTO musical_keys (name, root_note, mode, sort_order) VALUES
    ('C', 'C', 'Major', 1),
    ('Cm', 'C', 'Minor', 2),
    ('C#', 'C#', 'Major', 3),
    ('C#m', 'C#', 'Minor', 4),
    ('D', 'D', 'Major', 5),
    ('Dm', 'D', 'Minor', 6),
    ('D#', 'D#', 'Major', 7),
    ('D#m', 'D#', 'Minor', 8),
    ('E', 'E', 'Major', 9),
    ('Em', 'E', 'Minor', 10),
    ('F', 'F', 'Major', 11),
    ('Fm', 'F', 'Minor', 12),
    ('F#', 'F#', 'Major', 13),
    ('F#m', 'F#', 'Minor', 14),
    ('G', 'G', 'Major', 15),
    ('Gm', 'G', 'Minor', 16),
    ('G#', 'G#', 'Major', 17),
    ('G#m', 'G#', 'Minor', 18),
    ('A', 'A', 'Major', 19),
    ('Am', 'A', 'Minor', 20),
    ('A#', 'A#', 'Major', 21),
    ('A#m', 'A#', 'Minor', 22),
    ('B', 'B', 'Major', 23),
    ('Bm', 'B', 'Minor', 24)
ON CONFLICT (name) DO NOTHING;

-- Default collections
INSERT INTO collections (name, description, icon, color, is_smart) VALUES
    ('Favorites', 'Your favorite MIDI files', 'heart', '#ef4444', FALSE),
    ('Recently Added', 'Files added in the last 7 days', 'clock', '#3b82f6', TRUE),
    ('Top Rated', 'Highest rated files', 'star', '#eab308', TRUE),
    ('Untagged', 'Files without category tags', 'tag', '#6b7280', TRUE)
ON CONFLICT DO NOTHING;

-- =============================================================================
-- SECTION 7: TRIGGERS & FUNCTIONS
-- =============================================================================

-- Auto-update search vector on files insert/update
CREATE OR REPLACE FUNCTION update_file_search_vector()
RETURNS TRIGGER AS $$
BEGIN
    NEW.search_vector :=
        setweight(to_tsvector('english', COALESCE(NEW.filename, '')), 'A') ||
        setweight(to_tsvector('english', COALESCE(NEW.normalized_filename, '')), 'B');
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Drop trigger if exists, then create
DROP TRIGGER IF EXISTS files_search_vector_update ON files;
CREATE TRIGGER files_search_vector_update
    BEFORE INSERT OR UPDATE OF filename, normalized_filename ON files
    FOR EACH ROW EXECUTE FUNCTION update_file_search_vector();

-- Auto-assign BPM range based on BPM value
CREATE OR REPLACE FUNCTION assign_bpm_range()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.bpm IS NOT NULL THEN
        SELECT id INTO NEW.bpm_range_id
        FROM bpm_ranges
        WHERE NEW.bpm >= min_bpm AND NEW.bpm < max_bpm
        ORDER BY sort_order
        LIMIT 1;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS files_bpm_range_assign ON files;
CREATE TRIGGER files_bpm_range_assign
    BEFORE INSERT OR UPDATE OF bpm ON files
    FOR EACH ROW EXECUTE FUNCTION assign_bpm_range();

-- Update timbre file counts
CREATE OR REPLACE FUNCTION update_timbre_counts()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE timbres SET file_count = file_count + 1 WHERE id = NEW.timbre_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE timbres SET file_count = file_count - 1 WHERE id = OLD.timbre_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS midi_file_timbres_count ON midi_file_timbres;
CREATE TRIGGER midi_file_timbres_count
    AFTER INSERT OR DELETE ON midi_file_timbres
    FOR EACH ROW EXECUTE FUNCTION update_timbre_counts();

-- Update style file counts
CREATE OR REPLACE FUNCTION update_style_counts()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE styles SET file_count = file_count + 1 WHERE id = NEW.style_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE styles SET file_count = file_count - 1 WHERE id = OLD.style_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS midi_file_styles_count ON midi_file_styles;
CREATE TRIGGER midi_file_styles_count
    AFTER INSERT OR DELETE ON midi_file_styles
    FOR EACH ROW EXECUTE FUNCTION update_style_counts();

-- Update articulation file counts
CREATE OR REPLACE FUNCTION update_articulation_counts()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE articulations SET file_count = file_count + 1 WHERE id = NEW.articulation_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE articulations SET file_count = file_count - 1 WHERE id = OLD.articulation_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS midi_file_articulations_count ON midi_file_articulations;
CREATE TRIGGER midi_file_articulations_count
    AFTER INSERT OR DELETE ON midi_file_articulations
    FOR EACH ROW EXECUTE FUNCTION update_articulation_counts();

-- Update collection file counts
CREATE OR REPLACE FUNCTION update_collection_counts()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE collections SET file_count = file_count + 1, modified_at = NOW() 
        WHERE id = NEW.collection_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE collections SET file_count = file_count - 1, modified_at = NOW() 
        WHERE id = OLD.collection_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS collection_files_count ON collection_files;
CREATE TRIGGER collection_files_count
    AFTER INSERT OR DELETE ON collection_files
    FOR EACH ROW EXECUTE FUNCTION update_collection_counts();

-- Update saved search use count and timestamp
CREATE OR REPLACE FUNCTION update_saved_search_usage()
RETURNS TRIGGER AS $$
BEGIN
    NEW.use_count := OLD.use_count + 1;
    NEW.last_used := NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- =============================================================================
-- SECTION 8: VIEWS FOR VIP3 BROWSER
-- =============================================================================

-- View: Files with all category tags aggregated
CREATE OR REPLACE VIEW files_with_categories AS
SELECT
    f.*,
    COALESCE(
        (SELECT array_agg(t.name ORDER BY t.sort_order)
         FROM midi_file_timbres ft
         JOIN timbres t ON ft.timbre_id = t.id
         WHERE ft.file_id = f.id),
        '{}'::VARCHAR[]
    ) AS timbres,
    COALESCE(
        (SELECT array_agg(s.name ORDER BY s.sort_order)
         FROM midi_file_styles fs
         JOIN styles s ON fs.style_id = s.id
         WHERE fs.file_id = f.id),
        '{}'::VARCHAR[]
    ) AS styles,
    COALESCE(
        (SELECT array_agg(a.name ORDER BY a.sort_order)
         FROM midi_file_articulations fa
         JOIN articulations a ON fa.articulation_id = a.id
         WHERE fa.file_id = f.id),
        '{}'::VARCHAR[]
    ) AS articulations,
    br.label AS bpm_range_label,
    mk.name AS key_name,
    mk.mode AS key_mode
FROM files f
LEFT JOIN bpm_ranges br ON f.bpm_range_id = br.id
LEFT JOIN musical_keys mk ON f.key_id = mk.id;

-- View: Category counts for sidebar display
CREATE OR REPLACE VIEW vip3_category_counts AS
SELECT 'timbre' AS category, id, name, NULL AS icon, sort_order, file_count
FROM timbres WHERE file_count > 0
UNION ALL
SELECT 'style' AS category, id, name, NULL AS icon, sort_order, file_count
FROM styles WHERE file_count > 0
UNION ALL
SELECT 'articulation' AS category, id, name, NULL AS icon, sort_order, file_count
FROM articulations WHERE file_count > 0
UNION ALL
SELECT 'bpm_range' AS category, id::INTEGER, label AS name, NULL AS icon, sort_order, file_count
FROM bpm_ranges WHERE file_count > 0
UNION ALL
SELECT 'key' AS category, id::INTEGER, name, NULL AS icon, sort_order, file_count
FROM musical_keys WHERE file_count > 0
ORDER BY category, sort_order;

-- =============================================================================
-- SECTION 9: HELPER FUNCTIONS FOR VIP3 FILTERING
-- =============================================================================

-- Function: Get files by multiple filters (VIP3-style)
CREATE OR REPLACE FUNCTION get_files_vip3(
    p_folder_ids BIGINT[] DEFAULT NULL,
    p_instrument_ids INTEGER[] DEFAULT NULL,
    p_timbre_ids SMALLINT[] DEFAULT NULL,
    p_style_ids SMALLINT[] DEFAULT NULL,
    p_articulation_ids SMALLINT[] DEFAULT NULL,
    p_bpm_range_ids SMALLINT[] DEFAULT NULL,
    p_key_ids SMALLINT[] DEFAULT NULL,
    p_channel SMALLINT DEFAULT NULL,
    p_search_query TEXT DEFAULT NULL,
    p_sort_by VARCHAR(50) DEFAULT 'filename',
    p_sort_order VARCHAR(4) DEFAULT 'asc',
    p_limit INTEGER DEFAULT 50,
    p_offset INTEGER DEFAULT 0
)
RETURNS TABLE (
    id BIGINT,
    filename VARCHAR,
    file_path TEXT,
    bpm NUMERIC,
    key_signature VARCHAR,
    duration_ms INTEGER,
    note_count INTEGER,
    channel SMALLINT,
    rating SMALLINT,
    favorite BOOLEAN,
    timbres VARCHAR[],
    styles VARCHAR[],
    articulations VARCHAR[]
) AS $$
BEGIN
    RETURN QUERY
    SELECT DISTINCT
        f.id,
        f.filename,
        f.file_path,
        f.bpm,
        f.key_signature,
        f.duration_ms,
        f.note_count,
        f.channel,
        f.rating,
        f.favorite,
        COALESCE(
            (SELECT array_agg(t.name) FROM midi_file_timbres mft 
             JOIN timbres t ON mft.timbre_id = t.id WHERE mft.file_id = f.id),
            '{}'::VARCHAR[]
        ),
        COALESCE(
            (SELECT array_agg(s.name) FROM midi_file_styles mfs 
             JOIN styles s ON mfs.style_id = s.id WHERE mfs.file_id = f.id),
            '{}'::VARCHAR[]
        ),
        COALESCE(
            (SELECT array_agg(a.name) FROM midi_file_articulations mfa 
             JOIN articulations a ON mfa.articulation_id = a.id WHERE mfa.file_id = f.id),
            '{}'::VARCHAR[]
        )
    FROM files f
    LEFT JOIN midi_file_timbres mft ON f.id = mft.file_id
    LEFT JOIN midi_file_styles mfs ON f.id = mfs.file_id
    LEFT JOIN midi_file_articulations mfa ON f.id = mfa.file_id
    WHERE
        (p_folder_ids IS NULL OR f.folder_id = ANY(p_folder_ids))
        AND (p_instrument_ids IS NULL OR f.instrument_id = ANY(p_instrument_ids))
        AND (p_timbre_ids IS NULL OR mft.timbre_id = ANY(p_timbre_ids))
        AND (p_style_ids IS NULL OR mfs.style_id = ANY(p_style_ids))
        AND (p_articulation_ids IS NULL OR mfa.articulation_id = ANY(p_articulation_ids))
        AND (p_bpm_range_ids IS NULL OR f.bpm_range_id = ANY(p_bpm_range_ids))
        AND (p_key_ids IS NULL OR f.key_id = ANY(p_key_ids))
        AND (p_channel IS NULL OR f.channel = p_channel)
        AND (p_search_query IS NULL OR f.search_vector @@ plainto_tsquery('english', p_search_query))
    ORDER BY
        CASE WHEN p_sort_order = 'asc' AND p_sort_by = 'filename' THEN f.filename END ASC,
        CASE WHEN p_sort_order = 'desc' AND p_sort_by = 'filename' THEN f.filename END DESC,
        CASE WHEN p_sort_order = 'asc' AND p_sort_by = 'bpm' THEN f.bpm END ASC,
        CASE WHEN p_sort_order = 'desc' AND p_sort_by = 'bpm' THEN f.bpm END DESC,
        CASE WHEN p_sort_order = 'asc' AND p_sort_by = 'duration' THEN f.duration_ms END ASC,
        CASE WHEN p_sort_order = 'desc' AND p_sort_by = 'duration' THEN f.duration_ms END DESC,
        CASE WHEN p_sort_order = 'asc' AND p_sort_by = 'rating' THEN f.rating END ASC,
        CASE WHEN p_sort_order = 'desc' AND p_sort_by = 'rating' THEN f.rating END DESC,
        f.id
    LIMIT p_limit
    OFFSET p_offset;
END;
$$ LANGUAGE plpgsql;

-- Function: Count files matching VIP3 filters
CREATE OR REPLACE FUNCTION count_files_vip3(
    p_folder_ids BIGINT[] DEFAULT NULL,
    p_instrument_ids INTEGER[] DEFAULT NULL,
    p_timbre_ids SMALLINT[] DEFAULT NULL,
    p_style_ids SMALLINT[] DEFAULT NULL,
    p_articulation_ids SMALLINT[] DEFAULT NULL,
    p_bpm_range_ids SMALLINT[] DEFAULT NULL,
    p_key_ids SMALLINT[] DEFAULT NULL,
    p_channel SMALLINT DEFAULT NULL,
    p_search_query TEXT DEFAULT NULL
)
RETURNS BIGINT AS $$
DECLARE
    total BIGINT;
BEGIN
    SELECT COUNT(DISTINCT f.id) INTO total
    FROM files f
    LEFT JOIN midi_file_timbres mft ON f.id = mft.file_id
    LEFT JOIN midi_file_styles mfs ON f.id = mfs.file_id
    LEFT JOIN midi_file_articulations mfa ON f.id = mfa.file_id
    WHERE
        (p_folder_ids IS NULL OR f.folder_id = ANY(p_folder_ids))
        AND (p_instrument_ids IS NULL OR f.instrument_id = ANY(p_instrument_ids))
        AND (p_timbre_ids IS NULL OR mft.timbre_id = ANY(p_timbre_ids))
        AND (p_style_ids IS NULL OR mfs.style_id = ANY(p_style_ids))
        AND (p_articulation_ids IS NULL OR mfa.articulation_id = ANY(p_articulation_ids))
        AND (p_bpm_range_ids IS NULL OR f.bpm_range_id = ANY(p_bpm_range_ids))
        AND (p_key_ids IS NULL OR f.key_id = ANY(p_key_ids))
        AND (p_channel IS NULL OR f.channel = p_channel)
        AND (p_search_query IS NULL OR f.search_vector @@ plainto_tsquery('english', p_search_query));
    
    RETURN total;
END;
$$ LANGUAGE plpgsql;

-- =============================================================================
-- SECTION 10: COMMENTS
-- =============================================================================

COMMENT ON TABLE timbres IS 'Sound character descriptors for VIP3-style filtering (Aggressive, Warm, Bright, etc.)';
COMMENT ON TABLE styles IS 'Musical genres/styles for VIP3-style filtering (Hip-Hop, Techno, Jazz, etc.)';
COMMENT ON TABLE articulations IS 'Note playing styles for VIP3-style filtering (Arpeggio, Chord, Loop, etc.)';
COMMENT ON TABLE bpm_ranges IS 'BPM range buckets for fast tempo filtering';
COMMENT ON TABLE musical_keys IS 'Musical key signatures (C, Cm, D, Dm, etc.)';
COMMENT ON TABLE midi_file_timbres IS 'Many-to-many: files to timbres';
COMMENT ON TABLE midi_file_styles IS 'Many-to-many: files to styles';
COMMENT ON TABLE midi_file_articulations IS 'Many-to-many: files to articulations';
COMMENT ON TABLE saved_searches IS 'User-saved search filter combinations';
COMMENT ON TABLE recent_searches IS 'Search history for autocomplete/suggestions';
COMMENT ON TABLE collections IS 'User-created collections/playlists of files';
COMMENT ON TABLE collection_files IS 'Many-to-many: collections to files';
COMMENT ON VIEW files_with_categories IS 'Files with all category tags aggregated for display';
COMMENT ON VIEW vip3_category_counts IS 'Category counts for VIP3 browser sidebar';
COMMENT ON FUNCTION get_files_vip3 IS 'Main VIP3 browser query with all filter options';
COMMENT ON FUNCTION count_files_vip3 IS 'Count files matching VIP3 filters (for pagination)';

-- =============================================================================
-- SECTION 11: RECORD MIGRATION
-- =============================================================================

INSERT INTO schema_migrations (version, description)
VALUES ('019', 'VIP3 Filtering - timbres, styles, articulations, bpm_ranges, musical_keys, collections, saved_searches')
ON CONFLICT DO NOTHING;

-- =============================================================================
-- SECTION 12: VERIFY MIGRATION
-- =============================================================================

DO $$
DECLARE
    table_count INTEGER;
    timbre_count INTEGER;
    style_count INTEGER;
    articulation_count INTEGER;
    bpm_range_count INTEGER;
    key_count INTEGER;
BEGIN
    -- Count new tables
    SELECT COUNT(*) INTO table_count
    FROM information_schema.tables
    WHERE table_schema = 'public'
    AND table_name IN (
        'timbres', 'styles', 'articulations', 'bpm_ranges', 'musical_keys',
        'midi_file_timbres', 'midi_file_styles', 'midi_file_articulations',
        'saved_searches', 'recent_searches', 'collections', 'collection_files'
    );
    
    -- Count default data
    SELECT COUNT(*) INTO timbre_count FROM timbres;
    SELECT COUNT(*) INTO style_count FROM styles;
    SELECT COUNT(*) INTO articulation_count FROM articulations;
    SELECT COUNT(*) INTO bpm_range_count FROM bpm_ranges;
    SELECT COUNT(*) INTO key_count FROM musical_keys;
    
    RAISE NOTICE 'Migration 019 verification:';
    RAISE NOTICE '  Tables created: % (expected 12)', table_count;
    RAISE NOTICE '  Timbres: % (expected 21)', timbre_count;
    RAISE NOTICE '  Styles: % (expected 24)', style_count;
    RAISE NOTICE '  Articulations: % (expected 20)', articulation_count;
    RAISE NOTICE '  BPM Ranges: % (expected 8)', bpm_range_count;
    RAISE NOTICE '  Musical Keys: % (expected 24)', key_count;
    
    IF table_count < 12 THEN
        RAISE EXCEPTION 'Expected 12 tables, only created %', table_count;
    END IF;
    
    RAISE NOTICE 'Migration 019 completed successfully!';
END $$;

COMMIT;

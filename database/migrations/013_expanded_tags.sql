-- Migration 013: Expanded Tags with Hierarchy and Relationships
-- Ports genre/mood patterns from ~/midi/original/services/tag_manager
-- Run: psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -f database/migrations/013_expanded_tags.sql

BEGIN;

-- ============================================================
-- 1. MOOD TAGS (from ~/midi tag_manager MOOD_LABELS)
-- ============================================================
INSERT INTO tags (name, category) VALUES
    -- Primary moods (from tag_manager)
    ('happy', 'mood'),
    ('calm', 'mood'),
    ('energetic', 'mood'),
    ('dark', 'mood'),
    ('sad', 'mood'),
    ('bright', 'mood'),
    -- Extended moods
    ('aggressive', 'mood'),
    ('melancholic', 'mood'),
    ('uplifting', 'mood'),
    ('dreamy', 'mood'),
    ('intense', 'mood'),
    ('relaxed', 'mood'),
    ('mysterious', 'mood'),
    ('romantic', 'mood'),
    ('epic', 'mood'),
    ('playful', 'mood'),
    ('nostalgic', 'mood'),
    ('triumphant', 'mood'),
    ('haunting', 'mood'),
    ('groovy', 'mood')
ON CONFLICT (name) DO NOTHING;

-- ============================================================
-- 2. EXPANDED GENRE TAGS (hierarchical)
-- ============================================================

-- Parent genres (from tag_manager GENRE_LABELS + expanded)
INSERT INTO tags (name, category) VALUES
    -- Core genres
    ('pop', 'genre'),
    ('rock', 'genre'),
    ('jazz', 'genre'),
    ('classical', 'genre'),
    ('hiphop', 'genre'),
    ('edm', 'genre'),
    ('ambient', 'genre'),
    -- Extended parent genres
    ('electronic', 'genre'),
    ('metal', 'genre'),
    ('funk', 'genre'),
    ('soul', 'genre'),
    ('rnb', 'genre'),
    ('country', 'genre'),
    ('blues', 'genre'),
    ('reggae', 'genre'),
    ('latin', 'genre'),
    ('world', 'genre'),
    ('folk', 'genre'),
    ('indie', 'genre'),
    ('punk', 'genre'),
    ('experimental', 'genre')
ON CONFLICT (name) DO NOTHING;

-- Sub-genres (children of parent genres)
INSERT INTO tags (name, category) VALUES
    -- EDM sub-genres
    ('house', 'genre'),
    ('techno', 'genre'),
    ('trance', 'genre'),
    ('dubstep', 'genre'),
    ('drum-and-bass', 'genre'),
    ('dnb', 'genre'),
    ('breakbeat', 'genre'),
    ('garage', 'genre'),
    ('deep-house', 'genre'),
    ('tech-house', 'genre'),
    ('progressive-house', 'genre'),
    ('electro', 'genre'),
    ('hardstyle', 'genre'),
    ('trap', 'genre'),
    ('future-bass', 'genre'),
    ('uk-garage', 'genre'),
    ('jungle', 'genre'),
    -- Rock sub-genres
    ('alternative', 'genre'),
    ('grunge', 'genre'),
    ('hard-rock', 'genre'),
    ('progressive-rock', 'genre'),
    ('psychedelic', 'genre'),
    ('post-rock', 'genre'),
    ('shoegaze', 'genre'),
    -- Metal sub-genres
    ('death-metal', 'genre'),
    ('black-metal', 'genre'),
    ('thrash', 'genre'),
    ('doom', 'genre'),
    ('progressive-metal', 'genre'),
    ('nu-metal', 'genre'),
    -- Jazz sub-genres
    ('bebop', 'genre'),
    ('fusion', 'genre'),
    ('swing', 'genre'),
    ('smooth-jazz', 'genre'),
    ('free-jazz', 'genre'),
    ('acid-jazz', 'genre'),
    -- Hip-hop sub-genres
    ('boom-bap', 'genre'),
    ('lo-fi', 'genre'),
    ('phonk', 'genre'),
    ('drill', 'genre'),
    ('grime', 'genre'),
    -- Classical sub-genres
    ('baroque', 'genre'),
    ('romantic', 'genre'),
    ('contemporary', 'genre'),
    ('orchestral', 'genre'),
    ('chamber', 'genre'),
    ('minimalist', 'genre'),
    -- Ambient sub-genres
    ('chillout', 'genre'),
    ('downtempo', 'genre'),
    ('new-age', 'genre'),
    ('drone', 'genre'),
    ('dark-ambient', 'genre'),
    -- Latin sub-genres
    ('salsa', 'genre'),
    ('bossa-nova', 'genre'),
    ('reggaeton', 'genre'),
    ('cumbia', 'genre'),
    ('bachata', 'genre')
ON CONFLICT (name) DO NOTHING;

-- ============================================================
-- 3. TECHNIQUE & PATTERN TAGS
-- ============================================================
INSERT INTO tags (name, category) VALUES
    -- Drum techniques
    ('ghost-notes', 'technique'),
    ('double-bass', 'technique'),
    ('blast-beat', 'technique'),
    ('syncopation', 'technique'),
    ('polyrhythm', 'technique'),
    ('linear', 'technique'),
    ('paradiddle', 'technique'),
    ('flam', 'technique'),
    ('roll', 'technique'),
    ('rimshot', 'technique'),
    -- Feel/groove
    ('straight', 'feel'),
    ('shuffle', 'feel'),
    ('swing', 'feel'),
    ('half-time', 'feel'),
    ('double-time', 'feel'),
    ('triplet', 'feel'),
    ('dotted', 'feel'),
    -- Pattern types
    ('groove', 'pattern'),
    ('fill', 'pattern'),
    ('intro', 'pattern'),
    ('outro', 'pattern'),
    ('verse', 'pattern'),
    ('chorus', 'pattern'),
    ('bridge', 'pattern'),
    ('breakdown', 'pattern'),
    ('build', 'pattern'),
    ('drop', 'pattern'),
    ('loop', 'pattern'),
    ('one-shot', 'pattern'),
    ('transition', 'pattern'),
    -- Complexity
    ('simple', 'complexity'),
    ('moderate', 'complexity'),
    ('complex', 'complexity'),
    ('virtuoso', 'complexity')
ON CONFLICT (name) DO NOTHING;

-- ============================================================
-- 4. TIME SIGNATURE & TEMPO TAGS
-- ============================================================
INSERT INTO tags (name, category) VALUES
    -- Time signatures
    ('4-4', 'time-signature'),
    ('3-4', 'time-signature'),
    ('6-8', 'time-signature'),
    ('5-4', 'time-signature'),
    ('7-8', 'time-signature'),
    ('12-8', 'time-signature'),
    ('odd-meter', 'time-signature'),
    -- Tempo ranges
    ('very-slow', 'tempo'),      -- 30-60 BPM
    ('slow', 'tempo'),           -- 61-90 BPM
    ('mid-tempo', 'tempo'),      -- 91-120 BPM
    ('upbeat', 'tempo'),         -- 121-140 BPM
    ('fast', 'tempo'),           -- 141-180 BPM
    ('very-fast', 'tempo')       -- 180+ BPM
ON CONFLICT (name) DO NOTHING;

-- ============================================================
-- 5. KEY SIGNATURE TAGS
-- ============================================================
INSERT INTO tags (name, category) VALUES
    -- Major keys
    ('c-major', 'key'), ('c#-major', 'key'), ('d-major', 'key'),
    ('d#-major', 'key'), ('e-major', 'key'), ('f-major', 'key'),
    ('f#-major', 'key'), ('g-major', 'key'), ('g#-major', 'key'),
    ('a-major', 'key'), ('a#-major', 'key'), ('b-major', 'key'),
    -- Minor keys
    ('c-minor', 'key'), ('c#-minor', 'key'), ('d-minor', 'key'),
    ('d#-minor', 'key'), ('e-minor', 'key'), ('f-minor', 'key'),
    ('f#-minor', 'key'), ('g-minor', 'key'), ('g#-minor', 'key'),
    ('a-minor', 'key'), ('a#-minor', 'key'), ('b-minor', 'key')
ON CONFLICT (name) DO NOTHING;

-- ============================================================
-- 6. PRODUCTION & SOURCE TAGS
-- ============================================================
INSERT INTO tags (name, category) VALUES
    -- Production quality
    ('professional', 'quality'),
    ('demo', 'quality'),
    ('rough', 'quality'),
    ('polished', 'quality'),
    -- Source/origin
    ('sample-pack', 'source'),
    ('original', 'source'),
    ('remix', 'source'),
    ('cover', 'source'),
    ('stems', 'source'),
    -- Use case
    ('production-ready', 'use'),
    ('reference', 'use'),
    ('practice', 'use'),
    ('education', 'use'),
    ('game', 'use'),
    ('film', 'use'),
    ('commercial', 'use')
ON CONFLICT (name) DO NOTHING;

-- ============================================================
-- 7. CREATE TAG HIERARCHIES (parent-child relationships)
-- ============================================================

-- Helper function to create relationships
CREATE OR REPLACE FUNCTION create_tag_relationship(
    parent_name TEXT,
    child_name TEXT,
    rel_type VARCHAR(16) DEFAULT 'parent'
) RETURNS VOID AS $$
DECLARE
    p_id INTEGER;
    c_id INTEGER;
BEGIN
    SELECT id INTO p_id FROM tags WHERE name = parent_name;
    SELECT id INTO c_id FROM tags WHERE name = child_name;
    IF p_id IS NOT NULL AND c_id IS NOT NULL THEN
        INSERT INTO tag_relationships (parent_tag_id, child_tag_id, relationship_type)
        VALUES (p_id, c_id, rel_type)
        ON CONFLICT DO NOTHING;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- EDM hierarchy
SELECT create_tag_relationship('edm', 'house');
SELECT create_tag_relationship('edm', 'techno');
SELECT create_tag_relationship('edm', 'trance');
SELECT create_tag_relationship('edm', 'dubstep');
SELECT create_tag_relationship('edm', 'drum-and-bass');
SELECT create_tag_relationship('edm', 'breakbeat');
SELECT create_tag_relationship('edm', 'garage');
SELECT create_tag_relationship('edm', 'electro');
SELECT create_tag_relationship('edm', 'hardstyle');
SELECT create_tag_relationship('edm', 'trap');
SELECT create_tag_relationship('edm', 'future-bass');

-- House sub-hierarchy
SELECT create_tag_relationship('house', 'deep-house');
SELECT create_tag_relationship('house', 'tech-house');
SELECT create_tag_relationship('house', 'progressive-house');
SELECT create_tag_relationship('house', 'uk-garage');

-- D&B hierarchy
SELECT create_tag_relationship('drum-and-bass', 'jungle');

-- Rock hierarchy
SELECT create_tag_relationship('rock', 'alternative');
SELECT create_tag_relationship('rock', 'grunge');
SELECT create_tag_relationship('rock', 'hard-rock');
SELECT create_tag_relationship('rock', 'progressive-rock');
SELECT create_tag_relationship('rock', 'psychedelic');
SELECT create_tag_relationship('rock', 'post-rock');
SELECT create_tag_relationship('rock', 'shoegaze');
SELECT create_tag_relationship('rock', 'punk');
SELECT create_tag_relationship('rock', 'indie');

-- Metal hierarchy
SELECT create_tag_relationship('metal', 'death-metal');
SELECT create_tag_relationship('metal', 'black-metal');
SELECT create_tag_relationship('metal', 'thrash');
SELECT create_tag_relationship('metal', 'doom');
SELECT create_tag_relationship('metal', 'progressive-metal');
SELECT create_tag_relationship('metal', 'nu-metal');

-- Jazz hierarchy
SELECT create_tag_relationship('jazz', 'bebop');
SELECT create_tag_relationship('jazz', 'fusion');
SELECT create_tag_relationship('jazz', 'swing');
SELECT create_tag_relationship('jazz', 'smooth-jazz');
SELECT create_tag_relationship('jazz', 'free-jazz');
SELECT create_tag_relationship('jazz', 'acid-jazz');

-- Hip-hop hierarchy
SELECT create_tag_relationship('hiphop', 'boom-bap');
SELECT create_tag_relationship('hiphop', 'lo-fi');
SELECT create_tag_relationship('hiphop', 'phonk');
SELECT create_tag_relationship('hiphop', 'drill');
SELECT create_tag_relationship('hiphop', 'grime');
SELECT create_tag_relationship('hiphop', 'trap');

-- Classical hierarchy
SELECT create_tag_relationship('classical', 'baroque');
SELECT create_tag_relationship('classical', 'romantic');
SELECT create_tag_relationship('classical', 'contemporary');
SELECT create_tag_relationship('classical', 'orchestral');
SELECT create_tag_relationship('classical', 'chamber');
SELECT create_tag_relationship('classical', 'minimalist');

-- Ambient hierarchy
SELECT create_tag_relationship('ambient', 'chillout');
SELECT create_tag_relationship('ambient', 'downtempo');
SELECT create_tag_relationship('ambient', 'new-age');
SELECT create_tag_relationship('ambient', 'drone');
SELECT create_tag_relationship('ambient', 'dark-ambient');

-- Latin hierarchy
SELECT create_tag_relationship('latin', 'salsa');
SELECT create_tag_relationship('latin', 'bossa-nova');
SELECT create_tag_relationship('latin', 'reggaeton');
SELECT create_tag_relationship('latin', 'cumbia');
SELECT create_tag_relationship('latin', 'bachata');

-- ============================================================
-- 8. CREATE SYNONYMS
-- ============================================================

-- Genre synonyms
SELECT create_tag_relationship('drum-and-bass', 'dnb', 'synonym');
SELECT create_tag_relationship('hiphop', 'hip-hop', 'synonym');
SELECT create_tag_relationship('rnb', 'r-and-b', 'synonym');

-- Instrument synonyms
SELECT create_tag_relationship('hi-hat', 'hihat', 'synonym');
SELECT create_tag_relationship('gtr', 'guitar', 'synonym');
SELECT create_tag_relationship('electric-piano', 'ep', 'synonym');
SELECT create_tag_relationship('vox', 'vocal', 'synonym');

-- Cleanup helper function
DROP FUNCTION IF EXISTS create_tag_relationship(TEXT, TEXT, VARCHAR);

COMMIT;

-- ============================================================
-- Verification
-- ============================================================
DO $$
DECLARE
    tag_count INTEGER;
    rel_count INTEGER;
    mood_count INTEGER;
    genre_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO tag_count FROM tags;
    SELECT COUNT(*) INTO rel_count FROM tag_relationships;
    SELECT COUNT(*) INTO mood_count FROM tags WHERE category = 'mood';
    SELECT COUNT(*) INTO genre_count FROM tags WHERE category = 'genre';

    RAISE NOTICE 'Migration 013 complete:';
    RAISE NOTICE '  - Total tags: %', tag_count;
    RAISE NOTICE '  - Tag relationships: %', rel_count;
    RAISE NOTICE '  - Mood tags: %', mood_count;
    RAISE NOTICE '  - Genre tags: %', genre_count;
END $$;

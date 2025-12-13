-- =============================================================================
-- MIDI Library - Instrument-Based Organization
-- =============================================================================
-- Purpose: Tag files with instruments extracted from filenames
-- Source: INSTRUMENT_ANALYSIS.md (1,715,885 files, 97 instruments)
-- Date: 2025-11-22
-- =============================================================================

BEGIN;

-- =============================================================================
-- STEP 1: Insert all 97 discovered instruments as tags
-- =============================================================================

INSERT INTO tags (name, category, usage_count) VALUES
    -- Top 10 Drums/Percussion (highest frequency)
    ('ride', 'drums', 103591),
    ('fill', 'drums', 88142),
    ('kick', 'drums', 75286),
    ('tom', 'drums', 64351),
    ('crash', 'drums', 39690),
    ('snare', 'drums', 16341),
    ('stick', 'drums', 16144),
    ('hihat', 'drums', 11459),
    ('drums', 'drums', 7766),
    ('toms', 'drums', 7698),
    ('clap', 'drums', 5337),
    ('china', 'drums', 4067),
    ('conga', 'drums', 3319),
    ('cymbal', 'drums', 2479),
    ('rim', 'drums', 2123),
    ('cowbell', 'drums', 1623),
    ('bongo', 'drums', 1229),
    ('percussion', 'drums', 1173),
    ('shaker', 'drums', 915),
    ('tambourine', 'drums', 698),
    ('splash', 'drums', 333),
    ('hi-hat', 'drums', 306),
    ('drum', 'drums', 17380),

    -- Bass (3.08%)
    ('bass', 'bass', 52917),
    ('bassline', 'bass', 3782),
    ('sub', 'bass', 3590),
    ('808', 'bass', 3245),
    ('909', 'bass', 596),

    -- Synths & Keys (high melodic content)
    ('synth', 'synth', 26556),
    ('piano', 'keys', 21932),
    ('lead', 'synth', 19496),
    ('pad', 'synth', 12956),
    ('ep', 'keys', 9108),
    ('arp', 'synth', 7890),
    ('pluck', 'synth', 5602),
    ('keys', 'keys', 2749),
    ('organ', 'keys', 2533),
    ('rhodes', 'keys', 1744),
    ('keyboard', 'keys', 279),
    ('wurlitzer', 'keys', 94),
    ('electric-piano', 'keys', 0), -- alias for ep

    -- Guitars
    ('guitar', 'guitar', 5387),
    ('gtr', 'guitar', 3797),
    ('acoustic', 'guitar', 991),
    ('electric', 'guitar', 830),
    ('strum', 'guitar', 555),
    ('pick', 'guitar', 341),

    -- Strings
    ('strings', 'strings', 5986),
    ('violin', 'strings', 1077),
    ('cello', 'strings', 949),
    ('viola', 'strings', 438),
    ('ensemble', 'strings', 227),

    -- Brass
    ('brass', 'brass', 3866),
    ('trumpet', 'brass', 1476),
    ('horn', 'brass', 1440),
    ('trombone', 'brass', 679),
    ('sax', 'brass', 1511),
    ('saxophone', 'brass', 110),

    -- Woodwinds
    ('flute', 'woodwind', 1422),
    ('clarinet', 'woodwind', 581),
    ('oboe', 'woodwind', 474),
    ('bassoon', 'woodwind', 322),

    -- Vocals
    ('vocal', 'vocal', 1279),
    ('vox', 'vocal', 1082),
    ('voice', 'vocal', 868),
    ('choir', 'vocal', 2079),
    ('chant', 'vocal', 262),

    -- FX & Production
    ('fx', 'fx', 4646),
    ('bell', 'fx', 20861),
    ('hit', 'fx', 3336),
    ('sfx', 'fx', 490),
    ('sweep', 'fx', 194),
    ('riser', 'fx', 192),
    ('impact', 'fx', 75),

    -- Musical Elements
    ('loop', 'pattern', 31736),
    ('melody', 'melody', 15282),
    ('chord', 'harmony', 14766),
    ('groove', 'pattern', 10729),
    ('break', 'pattern', 4073),
    ('progression', 'harmony', 767),
    ('pattern', 'pattern', 748),
    ('harmonic', 'harmony', 620),
    ('melodic', 'melody', 437),

    -- Genres
    ('rock', 'genre', 40209),
    ('funk', 'genre', 11136),
    ('jazz', 'genre', 6193),
    ('dnb', 'genre', 6032),
    ('house', 'genre', 5315),
    ('trance', 'genre', 4866),
    ('techno', 'genre', 3891),
    ('edm', 'genre', 3872),
    ('soul', 'genre', 2756),
    ('trap', 'genre', 1564),
    ('reggae', 'genre', 1459),
    ('dubstep', 'genre', 447),
    ('hip-hop', 'genre', 22),
    ('r&b', 'genre', 42),

    -- Orchestral
    ('orchestra', 'orchestral', 672)
ON CONFLICT (name) DO UPDATE
SET usage_count = EXCLUDED.usage_count,
    category = EXCLUDED.category;

-- =============================================================================
-- STEP 2: Tag files based on filename matching
-- =============================================================================
-- This will tag all files that contain instrument keywords in their filenames

-- Create a function to efficiently tag files by keyword
CREATE OR REPLACE FUNCTION tag_files_by_keyword(
    p_keyword TEXT,
    p_tag_name TEXT
) RETURNS INTEGER AS $$
DECLARE
    v_tag_id INTEGER;
    v_count INTEGER;
BEGIN
    -- Get tag ID
    SELECT id INTO v_tag_id FROM tags WHERE name = p_tag_name;

    IF v_tag_id IS NULL THEN
        RAISE EXCEPTION 'Tag % not found', p_tag_name;
    END IF;

    -- Insert file_tags for matching files
    WITH matching_files AS (
        SELECT DISTINCT f.id
        FROM files f
        WHERE LOWER(f.filename) LIKE '%' || LOWER(p_keyword) || '%'
           OR LOWER(f.filepath) LIKE '%' || LOWER(p_keyword) || '%'
    )
    INSERT INTO file_tags (file_id, tag_id, added_by)
    SELECT mf.id, v_tag_id, 'instrument_extractor'
    FROM matching_files mf
    WHERE NOT EXISTS (
        SELECT 1 FROM file_tags ft2
        WHERE ft2.file_id = mf.id AND ft2.tag_id = v_tag_id
    );

    GET DIAGNOSTICS v_count = ROW_COUNT;
    RETURN v_count;
END;
$$ LANGUAGE plpgsql;

-- Tag top 30 instruments (covers 85%+ of files)
SELECT 'ride', tag_files_by_keyword('ride', 'ride');
SELECT 'fill', tag_files_by_keyword('fill', 'fill');
SELECT 'kick', tag_files_by_keyword('kick', 'kick');
SELECT 'tom', tag_files_by_keyword('tom', 'tom');
SELECT 'bass', tag_files_by_keyword('bass', 'bass');
SELECT 'rock', tag_files_by_keyword('rock', 'rock');
SELECT 'crash', tag_files_by_keyword('crash', 'crash');
SELECT 'loop', tag_files_by_keyword('loop', 'loop');
SELECT 'synth', tag_files_by_keyword('synth', 'synth');
SELECT 'piano', tag_files_by_keyword('piano', 'piano');
SELECT 'bell', tag_files_by_keyword('bell', 'bell');
SELECT 'lead', tag_files_by_keyword('lead', 'lead');
SELECT 'drum', tag_files_by_keyword('drum', 'drum');
SELECT 'snare', tag_files_by_keyword('snare', 'snare');
SELECT 'stick', tag_files_by_keyword('stick', 'stick');
SELECT 'melody', tag_files_by_keyword('melody', 'melody');
SELECT 'chord', tag_files_by_keyword('chord', 'chord');
SELECT 'pad', tag_files_by_keyword('pad', 'pad');
SELECT 'hihat', tag_files_by_keyword('hihat', 'hihat');
SELECT 'funk', tag_files_by_keyword('funk', 'funk');
SELECT 'groove', tag_files_by_keyword('groove', 'groove');
SELECT 'ep', tag_files_by_keyword('ep', 'ep');
SELECT 'arp', tag_files_by_keyword('arp', 'arp');
SELECT 'drums', tag_files_by_keyword('drums', 'drums');
SELECT 'toms', tag_files_by_keyword('toms', 'toms');
SELECT 'jazz', tag_files_by_keyword('jazz', 'jazz');
SELECT 'dnb', tag_files_by_keyword('dnb', 'dnb');
SELECT 'strings', tag_files_by_keyword('strings', 'strings');
SELECT 'pluck', tag_files_by_keyword('pluck', 'pluck');
SELECT 'guitar', tag_files_by_keyword('guitar', 'guitar');

-- =============================================================================
-- STEP 3: Create virtual folder views
-- =============================================================================

-- View: All drum files
CREATE OR REPLACE VIEW v_drums AS
SELECT DISTINCT f.*
FROM files f
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE t.category = 'drums'
ORDER BY f.filename;

-- View: All melodic/harmonic files
CREATE OR REPLACE VIEW v_melodic AS
SELECT DISTINCT f.*
FROM files f
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE t.category IN ('keys', 'synth', 'melody', 'harmony', 'strings', 'brass', 'woodwind', 'guitar')
ORDER BY f.filename;

-- View: All bass files
CREATE OR REPLACE VIEW v_bass AS
SELECT DISTINCT f.*
FROM files f
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE t.category = 'bass'
ORDER BY f.filename;

-- View: All loops and patterns
CREATE OR REPLACE VIEW v_loops AS
SELECT DISTINCT f.*
FROM files f
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE t.category = 'pattern'
ORDER BY f.filename;

-- View: Files by genre
CREATE OR REPLACE VIEW v_by_genre AS
SELECT
    t.name AS genre,
    COUNT(DISTINCT ft.file_id) AS file_count,
    array_agg(DISTINCT f.filepath ORDER BY f.filepath) AS filepaths
FROM tags t
JOIN file_tags ft ON t.id = ft.tag_id
JOIN files f ON ft.file_id = f.id
WHERE t.category = 'genre'
GROUP BY t.name
ORDER BY file_count DESC;

-- =============================================================================
-- STEP 4: Statistics and analysis views
-- =============================================================================

-- View: Tag statistics
CREATE OR REPLACE VIEW v_tag_stats AS
SELECT
    t.name,
    t.category,
    COUNT(DISTINCT ft.file_id) AS file_count,
    ROUND(COUNT(DISTINCT ft.file_id)::NUMERIC / (SELECT COUNT(*) FROM files) * 100, 2) AS percentage
FROM tags t
LEFT JOIN file_tags ft ON t.id = ft.tag_id
GROUP BY t.id, t.name, t.category
ORDER BY file_count DESC;

-- View: Multi-instrument files (files with 3+ tags)
CREATE OR REPLACE VIEW v_multi_instrument AS
SELECT
    f.id,
    f.filename,
    f.filepath,
    COUNT(ft.tag_id) AS tag_count,
    array_agg(t.name ORDER BY t.name) AS instruments
FROM files f
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
GROUP BY f.id, f.filename, f.filepath
HAVING COUNT(ft.tag_id) >= 3
ORDER BY tag_count DESC;

-- View: Instrument combinations
CREATE OR REPLACE VIEW v_instrument_combos AS
SELECT
    array_agg(t.name ORDER BY t.name) AS instrument_combo,
    COUNT(DISTINCT f.id) AS file_count
FROM files f
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
GROUP BY f.id
HAVING COUNT(ft.tag_id) BETWEEN 2 AND 5
ORDER BY file_count DESC
LIMIT 100;

-- =============================================================================
-- STEP 5: Create indexes for fast querying
-- =============================================================================

-- Composite index for tag category queries
CREATE INDEX IF NOT EXISTS idx_tags_category ON tags(category, name);

-- Index for file tag lookups
CREATE INDEX IF NOT EXISTS idx_file_tags_tag_id ON file_tags(tag_id);
CREATE INDEX IF NOT EXISTS idx_file_tags_file_id ON file_tags(file_id);

-- =============================================================================
-- STEP 6: Helper functions for queries
-- =============================================================================

-- Function: Get all files with specific instrument
CREATE OR REPLACE FUNCTION get_files_by_instrument(p_instrument TEXT)
RETURNS TABLE (
    file_id BIGINT,
    filename TEXT,
    filepath TEXT,
    bpm NUMERIC,
    key_signature musical_key
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        f.id,
        f.filename,
        f.filepath,
        m.bpm,
        m.key_signature
    FROM files f
    JOIN file_tags ft ON f.id = ft.file_id
    JOIN tags t ON ft.tag_id = t.id
    LEFT JOIN musical_metadata m ON f.id = m.file_id
    WHERE t.name = p_instrument
    ORDER BY f.filename;
END;
$$ LANGUAGE plpgsql;

-- Function: Get files by multiple instruments (AND logic)
CREATE OR REPLACE FUNCTION get_files_by_instruments(p_instruments TEXT[])
RETURNS TABLE (
    file_id BIGINT,
    filename TEXT,
    filepath TEXT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        f.id,
        f.filename,
        f.filepath
    FROM files f
    WHERE EXISTS (
        SELECT 1
        FROM file_tags ft
        JOIN tags t ON ft.tag_id = t.id
        WHERE ft.file_id = f.id
          AND t.name = ANY(p_instruments)
        GROUP BY ft.file_id
        HAVING COUNT(DISTINCT t.name) = array_length(p_instruments, 1)
    )
    ORDER BY f.filename;
END;
$$ LANGUAGE plpgsql;

COMMIT;

-- =============================================================================
-- USAGE EXAMPLES
-- =============================================================================

/*
-- Example 1: Find all ride cymbal files
SELECT * FROM get_files_by_instrument('ride');

-- Example 2: Find all rock genre drums
SELECT DISTINCT f.*
FROM files f
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE t.name IN ('rock', 'drums', 'kick', 'snare');

-- Example 3: Find 120 BPM drum loops in C major
SELECT f.filename, f.filepath, m.bpm, m.key_signature
FROM files f
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
JOIN musical_metadata m ON f.id = m.file_id
WHERE t.category = 'drums'
  AND t.name = 'loop'
  AND m.bpm BETWEEN 118 AND 122
  AND m.key_signature = 'C';

-- Example 4: Find all jazz piano files
SELECT * FROM get_files_by_instruments(ARRAY['jazz', 'piano']);

-- Example 5: Get statistics by instrument category
SELECT
    category,
    COUNT(*) AS instrument_count,
    SUM(file_count) AS total_files
FROM v_tag_stats
WHERE category IS NOT NULL
GROUP BY category
ORDER BY total_files DESC;

-- Example 6: Export drum file list for DAW
\COPY (SELECT filepath FROM v_drums) TO '/tmp/drum_files.txt' WITH CSV;

-- Example 7: Find melodic loops with BPM and key
SELECT
    f.filename,
    m.bpm,
    m.key_signature,
    array_agg(DISTINCT t.name) AS tags
FROM files f
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
LEFT JOIN musical_metadata m ON f.id = m.file_id
WHERE t.category IN ('melody', 'pattern')
GROUP BY f.id, f.filename, m.bpm, m.key_signature
HAVING array_agg(DISTINCT t.name) @> ARRAY['loop']
ORDER BY m.bpm;

-- Example 8: Create symlinks for drums folder
SELECT
    'ln -s "' || filepath || '" /path/to/organized/drums/' AS symlink_command
FROM v_drums
\gexec
*/

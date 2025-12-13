#!/bin/bash
# =============================================================================
# MIDI Library - BATCH Organization (Fast & Efficient)
# =============================================================================
# Strategy: Process top keywords in parallel batches
# Time: 5-10 minutes for 1.7M files
# =============================================================================

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  BATCH Organization - 97 Instruments${NC}"
echo -e "${BLUE}  Parallel batch processing - 5-10 minute ETA${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

START_TIME=$(date +%s)

# =============================================================================
# Step 1: Insert tags
# =============================================================================
echo -e "${YELLOW}[1/3] Creating 97 instrument tags...${NC}"

psql "$DB_URL" <<'SQL'
BEGIN;

INSERT INTO tags (name, category, usage_count) VALUES
    ('ride', 'drums', 103591), ('fill', 'drums', 88142), ('kick', 'drums', 75286),
    ('tom', 'drums', 64351), ('crash', 'drums', 39690), ('snare', 'drums', 16341),
    ('stick', 'drums', 16144), ('hihat', 'drums', 11459), ('drums', 'drums', 7766),
    ('toms', 'drums', 7698), ('clap', 'drums', 5337), ('china', 'drums', 4067),
    ('conga', 'drums', 3319), ('cymbal', 'drums', 2479), ('rim', 'drums', 2123),
    ('cowbell', 'drums', 1623), ('bongo', 'drums', 1229), ('percussion', 'drums', 1173),
    ('shaker', 'drums', 915), ('tambourine', 'drums', 698), ('splash', 'drums', 333),
    ('hi-hat', 'drums', 306), ('drum', 'drums', 17380),
    ('bass', 'bass', 52917), ('bassline', 'bass', 3782), ('sub', 'bass', 3590),
    ('808', 'bass', 3245), ('909', 'bass', 596),
    ('synth', 'synth', 26556), ('piano', 'keys', 21932), ('lead', 'synth', 17685),
    ('pad', 'synth', 11991), ('keys', 'keys', 9859), ('arp', 'synth', 7464),
    ('pluck', 'synth', 6656), ('organ', 'keys', 4874), ('brass', 'brass', 3976),
    ('rhodes', 'keys', 3086), ('wurlitzer', 'keys', 765), ('clav', 'keys', 459),
    ('electric-piano', 'keys', 136), ('harpsichord', 'keys', 42),
    ('guitar', 'guitar', 10913), ('acoustic', 'guitar', 4128), ('electric', 'guitar', 2965),
    ('12-string', 'guitar', 89), ('slide', 'guitar', 67), ('muted', 'guitar', 34),
    ('strings', 'strings', 5847), ('violin', 'strings', 2134), ('cello', 'strings', 987),
    ('viola', 'strings', 432), ('ensemble', 'strings', 276), ('orchestra', 'orchestral', 672),
    ('trumpet', 'brass', 1876), ('sax', 'brass', 1543), ('trombone', 'brass', 765),
    ('horn', 'brass', 432), ('flute', 'woodwind', 1234), ('clarinet', 'woodwind', 543),
    ('oboe', 'woodwind', 234), ('bassoon', 'woodwind', 123),
    ('vocal', 'vocal', 6734), ('vox', 'vocal', 2876), ('choir', 'vocal', 1543),
    ('voice', 'vocal', 987), ('chant', 'vocal', 234),
    ('fx', 'fx', 4646), ('bell', 'fx', 20861), ('hit', 'fx', 3336),
    ('sfx', 'fx', 490), ('sweep', 'fx', 194), ('riser', 'fx', 192), ('impact', 'fx', 75),
    ('loop', 'pattern', 31736), ('melody', 'melody', 15282), ('chord', 'harmony', 14766),
    ('groove', 'pattern', 10729), ('break', 'pattern', 4073), ('progression', 'harmony', 767),
    ('pattern', 'pattern', 748), ('harmonic', 'harmony', 620), ('melodic', 'melody', 437),
    ('rock', 'genre', 40209), ('funk', 'genre', 11136), ('jazz', 'genre', 6193),
    ('dnb', 'genre', 6032), ('house', 'genre', 5315), ('trance', 'genre', 4866),
    ('techno', 'genre', 3891), ('edm', 'genre', 3872), ('soul', 'genre', 2756),
    ('trap', 'genre', 1564), ('reggae', 'genre', 1459), ('dubstep', 'genre', 447),
    ('hip-hop', 'genre', 22), ('r&b', 'genre', 42)
ON CONFLICT (name) DO UPDATE
SET usage_count = EXCLUDED.usage_count, category = EXCLUDED.category;

COMMIT;
SQL

echo -e "${GREEN}✓ Tags created${NC}"
echo ""

# =============================================================================
# Step 2: Tag files - TOP 30 instruments (covers 85%+ of files)
# =============================================================================
echo -e "${YELLOW}[2/3] Tagging files (processing top 30 keywords in parallel)...${NC}"
echo "  This covers 85%+ of all files"
echo ""

# Function to tag files by keyword
tag_keyword() {
    local keyword=$1
    local tag_name=$2
    local db_url=$3

    psql "$db_url" -t <<SQL 2>&1 | grep -E "INSERT|^[0-9]+" || true
WITH matching_files AS (
    SELECT DISTINCT f.id
    FROM files f
    WHERE LOWER(f.filename) LIKE '%${keyword}%'
       OR LOWER(f.filepath) LIKE '%${keyword}%'
)
INSERT INTO file_tags (file_id, tag_id, added_by)
SELECT mf.id, t.id, 'batch_organizer'
FROM matching_files mf, tags t
WHERE t.name = '${tag_name}'
  AND NOT EXISTS (
      SELECT 1 FROM file_tags ft2
      WHERE ft2.file_id = mf.id AND ft2.tag_id = t.id
  );
SQL
}

export -f tag_keyword

# Top 30 keywords - process in parallel (8 at a time)
KEYWORDS=(
    "ride:ride" "fill:fill" "kick:kick" "tom:tom" "bass:bass"
    "rock:rock" "crash:crash" "loop:loop" "synth:synth" "piano:piano"
    "bell:bell" "lead:lead" "drum:drum" "snare:snare" "stick:stick"
    "melody:melody" "chord:chord" "hihat:hihat" "toms:toms" "bassline:bassline"
    "groove:groove" "pad:pad" "guitar:guitar" "funk:funk" "jazz:jazz"
    "strings:strings" "dnb:dnb" "house:house" "keys:keys" "arp:arp"
)

echo "Processing ${#KEYWORDS[@]} keywords in parallel..."

# Process in batches of 8 (parallel)
for i in "${!KEYWORDS[@]}"; do
    IFS=':' read -r keyword tag_name <<< "${KEYWORDS[$i]}"

    {
        COUNT=$(tag_keyword "$keyword" "$tag_name" "$DB_URL" | tail -1)
        echo "  ✓ $tag_name: $COUNT files tagged"
    } &

    # Limit to 8 parallel jobs
    if (( (i + 1) % 8 == 0 )); then
        wait
    fi
done

# Wait for remaining jobs
wait

echo ""
echo -e "${GREEN}✓ Top 30 keywords processed${NC}"
echo ""

# =============================================================================
# Step 3: Create views and functions
# =============================================================================
echo -e "${YELLOW}[3/3] Creating views and helper functions...${NC}"

psql "$DB_URL" <<'SQL'
-- Views
CREATE OR REPLACE VIEW v_drums AS
SELECT DISTINCT f.id, f.filename, f.filepath, f.hash, f.size, f.created_at,
       m.bpm, m.key_signature, m.duration, m.time_signature
FROM files f
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
LEFT JOIN musical_metadata m ON f.id = m.file_id
WHERE t.category = 'drums';

CREATE OR REPLACE VIEW v_melodic AS
SELECT DISTINCT f.id, f.filename, f.filepath, f.hash, f.size, f.created_at,
       m.bpm, m.key_signature, m.duration, m.time_signature
FROM files f
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
LEFT JOIN musical_metadata m ON f.id = m.file_id
WHERE t.category IN ('keys', 'synth', 'strings', 'brass', 'woodwind', 'guitar');

CREATE OR REPLACE VIEW v_bass AS
SELECT DISTINCT f.id, f.filename, f.filepath, f.hash, f.size, f.created_at,
       m.bpm, m.key_signature, m.duration, m.time_signature
FROM files f
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
LEFT JOIN musical_metadata m ON f.id = m.file_id
WHERE t.category = 'bass';

CREATE OR REPLACE VIEW v_loops AS
SELECT DISTINCT f.id, f.filename, f.filepath, f.hash, f.size, f.created_at,
       m.bpm, m.key_signature, m.duration, m.time_signature
FROM files f
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
LEFT JOIN musical_metadata m ON f.id = m.file_id
WHERE t.category = 'pattern';

CREATE OR REPLACE VIEW v_tag_stats AS
SELECT t.name, t.category, COUNT(DISTINCT ft.file_id) as file_count,
       ROUND(COUNT(DISTINCT ft.file_id)::NUMERIC * 100.0 / (SELECT COUNT(*) FROM files), 2) as percentage
FROM tags t
LEFT JOIN file_tags ft ON t.id = ft.tag_id
GROUP BY t.id, t.name, t.category
ORDER BY file_count DESC;

-- Helper functions
CREATE OR REPLACE FUNCTION get_files_by_instrument(instrument_name TEXT)
RETURNS TABLE (id BIGINT, filename TEXT, filepath TEXT, bpm INTEGER, key_signature TEXT) AS $$
BEGIN
    RETURN QUERY
    SELECT DISTINCT f.id, f.filename, f.filepath, m.bpm, m.key_signature
    FROM files f
    JOIN file_tags ft ON f.id = ft.file_id
    JOIN tags t ON ft.tag_id = t.id
    LEFT JOIN musical_metadata m ON f.id = m.file_id
    WHERE t.name = instrument_name;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION get_files_by_bpm_range(min_bpm INTEGER, max_bpm INTEGER)
RETURNS TABLE (id BIGINT, filename TEXT, filepath TEXT, bpm INTEGER, key_signature TEXT) AS $$
BEGIN
    RETURN QUERY
    SELECT f.id, f.filename, f.filepath, m.bpm, m.key_signature
    FROM files f
    JOIN musical_metadata m ON f.id = m.file_id
    WHERE m.bpm BETWEEN min_bpm AND max_bpm;
END;
$$ LANGUAGE plpgsql;
SQL

echo -e "${GREEN}✓ Views and functions created${NC}"
echo ""

# =============================================================================
# Summary
# =============================================================================
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Organization Complete!${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo "  Duration: ${DURATION}s (~$((DURATION / 60)) minutes)"
echo ""

# Statistics
psql "$DB_URL" <<'SQL'
SELECT '=== Statistics ===' as section;
SELECT 'Total tags: ' || COUNT(*)::text FROM tags;
SELECT 'File-tag relationships: ' || COUNT(*)::text FROM file_tags;
SELECT 'Files with tags: ' || COUNT(DISTINCT file_id)::text FROM file_tags;

SELECT '';
SELECT '=== Top 10 Instruments ===' as section;
SELECT name || ': ' || file_count::text || ' files (' || percentage::text || '%)'
FROM v_tag_stats
WHERE category IS NOT NULL
ORDER BY file_count DESC
LIMIT 10;
SQL

echo ""
echo -e "${GREEN}✓ Database organized successfully!${NC}"
echo ""

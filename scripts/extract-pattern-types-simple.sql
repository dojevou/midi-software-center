-- Extract Pattern Types for MPC/Force Functional Expansions (Simplified)
-- Created: November 22, 2025
-- Uses only: files, file_tags, tags, musical_metadata tables

\echo '=== ARPEGGIO PATTERNS ==='

-- Filename-based arpeggio detection
\copy (SELECT f.filepath FROM files f WHERE f.num_tracks = 1 AND (f.filename ILIKE '%arp%' OR f.filename ILIKE '%ascending%' OR f.filename ILIKE '%descending%' OR f.filename ILIKE '%up%down%' OR f.filename ILIKE '%sequence%') ORDER BY f.filename LIMIT 2000) TO '/tmp/arpeggio_patterns.txt';

\echo 'Arpeggio patterns exported to /tmp/arpeggio_patterns.txt'

\echo '=== RHYTHM PATTERNS ==='

-- Filename-based rhythm detection
\copy (SELECT f.filepath FROM files f WHERE f.num_tracks = 1 AND (f.filename ILIKE '%rhythm%' OR f.filename ILIKE '%groove%' OR f.filename ILIKE '%swing%' OR f.filename ILIKE '%shuffle%' OR f.filename ILIKE '%syncopat%' OR f.filename ILIKE '%straight%' OR f.filename ILIKE '%beat%' OR f.filename ILIKE '%drum%loop%') ORDER BY f.filename LIMIT 2000) TO '/tmp/rhythm_patterns.txt';

\echo 'Rhythm patterns exported to /tmp/rhythm_patterns.txt'

\echo '=== CHORD PROGRESSION PATTERNS ==='

-- Filename-based chord detection
\copy (SELECT f.filepath FROM files f WHERE f.num_tracks = 1 AND (f.filename ILIKE '%chord%' OR f.filename ILIKE '%progression%' OR f.filename ILIKE '%harmony%' OR f.filename ILIKE '%triad%' OR f.filename ILIKE '%seventh%' OR f.filename ILIKE '%maj%min%' OR f.filename ILIKE '%voicing%') ORDER BY f.filename LIMIT 2000) TO '/tmp/chord_patterns.txt';

\echo 'Chord patterns exported to /tmp/chord_patterns.txt'

\echo '=== DRUM PATTERNS (for functional expansion) ==='

-- Extract drum patterns from tags or filename
\copy (SELECT DISTINCT f.filepath FROM files f LEFT JOIN file_tags ft ON f.id = ft.file_id LEFT JOIN tags t ON ft.tag_id = t.id WHERE f.num_tracks = 1 AND (t.name IN ('drums', 'kick', 'snare', 'hihat', 'cymbal', 'tom', 'percussion') OR f.filename ILIKE '%drum%' OR f.filename ILIKE '%kick%' OR f.filename ILIKE '%snare%' OR f.filename ILIKE '%hat%') ORDER BY f.filepath LIMIT 5000) TO '/tmp/drum_patterns.txt';

\echo 'Drum patterns exported to /tmp/drum_patterns.txt'

\echo '=== BASS PATTERNS (for functional expansion) ==='

-- Extract bass patterns
\copy (SELECT DISTINCT f.filepath FROM files f LEFT JOIN file_tags ft ON f.id = ft.file_id LEFT JOIN tags t ON ft.tag_id = t.id WHERE f.num_tracks = 1 AND (t.name IN ('bass', 'sub', '808', '909', 'bassline') OR f.filename ILIKE '%bass%' OR f.filename ILIKE '%808%' OR f.filename ILIKE '%909%' OR f.filename ILIKE '%sub%') ORDER BY f.filepath LIMIT 5000) TO '/tmp/bass_patterns.txt';

\echo 'Bass patterns exported to /tmp/bass_patterns.txt'

\echo '=== MELODIC PATTERNS (for functional expansion) ==='

-- Extract melodic patterns (piano, synth, keys)
\copy (SELECT DISTINCT f.filepath FROM files f LEFT JOIN file_tags ft ON f.id = ft.file_id LEFT JOIN tags t ON ft.tag_id = t.id WHERE f.num_tracks = 1 AND (t.name IN ('piano', 'synth', 'keys', 'lead', 'melody', 'arp') OR f.filename ILIKE '%piano%' OR f.filename ILIKE '%synth%' OR f.filename ILIKE '%keys%' OR f.filename ILIKE '%lead%' OR f.filename ILIKE '%melody%') ORDER BY f.filepath LIMIT 5000) TO '/tmp/melodic_patterns.txt';

\echo 'Melodic patterns exported to /tmp/melodic_patterns.txt'

\echo '=== PATTERN STATISTICS ==='

-- Count arpeggio patterns
SELECT 'Arpeggio Patterns' as category, COUNT(*) as file_count
FROM files f
WHERE f.num_tracks = 1
  AND (f.filename ILIKE '%arp%' OR f.filename ILIKE '%ascending%' OR f.filename ILIKE '%descending%');

-- Count rhythm patterns
SELECT 'Rhythm Patterns' as category, COUNT(*) as file_count
FROM files f
WHERE f.num_tracks = 1
  AND (f.filename ILIKE '%rhythm%' OR f.filename ILIKE '%groove%' OR f.filename ILIKE '%swing%');

-- Count chord patterns
SELECT 'Chord Patterns' as category, COUNT(*) as file_count
FROM files f
WHERE f.num_tracks = 1
  AND (f.filename ILIKE '%chord%' OR f.filename ILIKE '%progression%' OR f.filename ILIKE '%harmony%');

-- Distribution by BPM
SELECT
  CASE
    WHEN m.bpm < 90 THEN 'Slow (<90 BPM)'
    WHEN m.bpm < 120 THEN 'Medium (90-120 BPM)'
    WHEN m.bpm < 140 THEN 'Upbeat (120-140 BPM)'
    ELSE 'Fast (>=140 BPM)'
  END as bpm_range,
  COUNT(*) as file_count
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
WHERE f.num_tracks = 1 AND m.bpm IS NOT NULL
GROUP BY bpm_range
ORDER BY bpm_range;

\echo '=== EXTRACTION COMPLETE ==='

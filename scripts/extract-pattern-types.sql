-- Extract Pattern Types for MPC/Force Functional Expansions
-- Created: November 22, 2025
-- Purpose: Identify arpeggio, rhythm, and chord patterns from 2.2M single-track files

-- ============================================================================
-- 1. ARPEGGIO PATTERNS
-- ============================================================================
-- Single-note melodic patterns with ascending/descending characteristics
-- Target: 1,000-2,000 files for Arpeggiator menu

\echo '=== ARPEGGIO PATTERNS ==='

-- Method 1: Filename-based detection (fastest)
\copy (
  SELECT f.filepath
  FROM files f
  WHERE f.num_tracks = 1
    AND (
      f.filename ILIKE '%arp%'
      OR f.filename ILIKE '%ascending%'
      OR f.filename ILIKE '%descending%'
      OR f.filename ILIKE '%up%down%'
      OR f.filename ILIKE '%sequence%'
      OR f.filename ILIKE '%pattern%'
    )
  ORDER BY f.filename
  LIMIT 2000
) TO '/tmp/arpeggio_patterns_filename.txt';

-- Method 2: Analysis-based detection (more accurate)
\copy (
  SELECT DISTINCT f.filepath
  FROM files f
  LEFT JOIN analysis_results ar ON f.id = ar.file_id
  WHERE f.num_tracks = 1
    AND ar.polyphony_avg < 2.0        -- Mostly single notes
    AND ar.note_range > 12             -- Covers at least 1 octave
    AND ar.note_density > 0.2          -- Not too sparse
  ORDER BY ar.note_range DESC          -- Prefer wider range
  LIMIT 2000
) TO '/tmp/arpeggio_patterns_analysis.txt';

-- Method 3: Combined approach (filename OR analysis)
\copy (
  SELECT DISTINCT f.filepath
  FROM files f
  LEFT JOIN analysis_results ar ON f.id = ar.file_id
  WHERE f.num_tracks = 1
    AND (
      -- Filename indicators
      f.filename ILIKE '%arp%'
      OR f.filename ILIKE '%ascending%'
      OR f.filename ILIKE '%descending%'
      -- OR analysis characteristics
      OR (
        ar.polyphony_avg < 2.0
        AND ar.note_range > 12
        AND ar.note_density > 0.2
      )
    )
  ORDER BY f.filename
  LIMIT 2000
) TO '/tmp/arpeggio_patterns_combined.txt';

\echo 'Arpeggio patterns exported to /tmp/arpeggio_patterns_*.txt'

-- ============================================================================
-- 2. RHYTHM PATTERNS
-- ============================================================================
-- Rhythmic/groove patterns for Arpeggiator > Rhythm mode
-- Target: 1,000-2,000 files

\echo '=== RHYTHM PATTERNS ==='

-- Method 1: From drum_patterns table
\copy (
  SELECT DISTINCT f.filepath
  FROM files f
  JOIN drum_patterns dp ON f.id = dp.file_id
  WHERE f.num_tracks = 1
    AND dp.pattern_type IN ('groove', 'pattern', 'loop')
    AND dp.feel IN ('straight', 'swing', 'shuffle', 'syncopated')
  ORDER BY dp.feel, f.filename
  LIMIT 2000
) TO '/tmp/rhythm_patterns_drums.txt';

-- Method 2: Filename-based
\copy (
  SELECT f.filepath
  FROM files f
  WHERE f.num_tracks = 1
    AND (
      f.filename ILIKE '%rhythm%'
      OR f.filename ILIKE '%groove%'
      OR f.filename ILIKE '%swing%'
      OR f.filename ILIKE '%shuffle%'
      OR f.filename ILIKE '%syncopat%'
      OR f.filename ILIKE '%straight%'
    )
  ORDER BY f.filename
  LIMIT 2000
) TO '/tmp/rhythm_patterns_filename.txt';

-- Method 3: Combined approach
\copy (
  SELECT DISTINCT f.filepath
  FROM files f
  LEFT JOIN drum_patterns dp ON f.id = dp.file_id
  WHERE f.num_tracks = 1
    AND (
      -- From drum analysis
      (dp.pattern_type IN ('groove', 'pattern', 'loop') AND dp.feel IS NOT NULL)
      -- OR from filename
      OR f.filename ILIKE '%rhythm%'
      OR f.filename ILIKE '%groove%'
      OR f.filename ILIKE '%swing%'
    )
  ORDER BY f.filename
  LIMIT 2000
) TO '/tmp/rhythm_patterns_combined.txt';

\echo 'Rhythm patterns exported to /tmp/rhythm_patterns_*.txt'

-- ============================================================================
-- 3. CHORD PROGRESSION PATTERNS
-- ============================================================================
-- Polyphonic patterns with chord characteristics
-- Target: 1,000-2,000 files (will be .mpcpattern, not .mpcprog)

\echo '=== CHORD PROGRESSION PATTERNS ==='

-- Method 1: High polyphony (3+ simultaneous notes)
\copy (
  SELECT DISTINCT f.filepath
  FROM files f
  JOIN analysis_results ar ON f.id = ar.file_id
  WHERE f.num_tracks = 1
    AND ar.polyphony_avg >= 3.0        -- Average 3+ notes at once
    AND ar.chord_complexity_score > 0.3
  ORDER BY ar.chord_complexity_score DESC
  LIMIT 2000
) TO '/tmp/chord_patterns_polyphony.txt';

-- Method 2: Filename-based
\copy (
  SELECT f.filepath
  FROM files f
  WHERE f.num_tracks = 1
    AND (
      f.filename ILIKE '%chord%'
      OR f.filename ILIKE '%progression%'
      OR f.filename ILIKE '%harmony%'
      OR f.filename ILIKE '%triad%'
      OR f.filename ILIKE '%seventh%'
    )
  ORDER BY f.filename
  LIMIT 2000
) TO '/tmp/chord_patterns_filename.txt';

-- Method 3: From chords table
\copy (
  SELECT DISTINCT f.filepath
  FROM files f
  JOIN chords c ON f.id = c.file_id
  JOIN musical_metadata m ON f.id = m.file_id
  WHERE f.num_tracks = 1
    AND m.key_signature IS NOT NULL
  GROUP BY f.filepath
  HAVING COUNT(DISTINCT c.chord_name) >= 3  -- At least 3 different chords
  ORDER BY COUNT(DISTINCT c.chord_name) DESC
  LIMIT 2000
) TO '/tmp/chord_patterns_chords.txt';

-- Method 4: Combined approach
\copy (
  SELECT DISTINCT f.filepath
  FROM files f
  LEFT JOIN analysis_results ar ON f.id = ar.file_id
  LEFT JOIN chords c ON f.id = c.file_id
  WHERE f.num_tracks = 1
    AND (
      -- High polyphony
      ar.polyphony_avg >= 3.0
      -- OR has chord data
      OR c.id IS NOT NULL
      -- OR filename indicators
      OR f.filename ILIKE '%chord%'
      OR f.filename ILIKE '%progression%'
    )
  GROUP BY f.filepath
  ORDER BY f.filepath
  LIMIT 2000
) TO '/tmp/chord_patterns_combined.txt';

\echo 'Chord patterns exported to /tmp/chord_patterns_*.txt'

-- ============================================================================
-- 4. STATISTICS & VERIFICATION
-- ============================================================================

\echo '=== PATTERN STATISTICS ==='

-- Count potential arpeggio patterns
SELECT
  'Arpeggio Patterns (filename)' as category,
  COUNT(*) as file_count
FROM files f
WHERE f.num_tracks = 1
  AND (
    f.filename ILIKE '%arp%'
    OR f.filename ILIKE '%ascending%'
    OR f.filename ILIKE '%descending%'
  );

-- Count potential rhythm patterns
SELECT
  'Rhythm Patterns (drum_patterns)' as category,
  COUNT(DISTINCT f.id) as file_count
FROM files f
JOIN drum_patterns dp ON f.id = dp.file_id
WHERE f.num_tracks = 1
  AND dp.pattern_type IN ('groove', 'pattern', 'loop');

-- Count potential chord patterns
SELECT
  'Chord Patterns (polyphony >= 3)' as category,
  COUNT(*) as file_count
FROM files f
JOIN analysis_results ar ON f.id = ar.file_id
WHERE f.num_tracks = 1
  AND ar.polyphony_avg >= 3.0;

-- Distribution by polyphony
SELECT
  CASE
    WHEN ar.polyphony_avg < 1.5 THEN 'Monophonic (< 1.5)'
    WHEN ar.polyphony_avg < 2.5 THEN 'Low Polyphony (1.5-2.5)'
    WHEN ar.polyphony_avg < 3.5 THEN 'Medium Polyphony (2.5-3.5)'
    ELSE 'High Polyphony (>= 3.5)'
  END as polyphony_category,
  COUNT(*) as file_count
FROM files f
JOIN analysis_results ar ON f.id = ar.file_id
WHERE f.num_tracks = 1
GROUP BY polyphony_category
ORDER BY polyphony_category;

\echo '=== EXTRACTION COMPLETE ==='
\echo 'Next steps:'
\echo '1. Choose which export files to use (filename, analysis, or combined)'
\echo '2. Copy MIDI files to /Arp_Patterns/ folder (arp + rhythm)'
\echo '3. Convert chord patterns to .mpcpattern for MIDI_CHORDS expansion'

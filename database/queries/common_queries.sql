-- Common Queries for MIDI Library Database
-- These are the queries the DAW and Pipeline will use most frequently

-- ============================================================================
-- SEARCH QUERIES
-- ============================================================================

-- Q1: Full-text search by filename/tags
-- Usage: Search for "trance bass"
EXPLAIN ANALYZE
SELECT
    f.id,
    f.new_filename,
    f.manufacturer,
    f.collection_name,
    mm.detected_bpm,
    mm.detected_key,
    f.category,
    ts_rank(f.search_vector, plainto_tsquery('english', 'trance bass')) as rank
FROM files f
LEFT JOIN musical_metadata mm ON f.id = mm.file_id
WHERE f.search_vector @@ plainto_tsquery('english', 'trance bass')
ORDER BY rank DESC
LIMIT 100;

-- Q2: Advanced filter search (BPM + Key + Category)
-- Usage: Find bass loops in Cm between 120-140 BPM
EXPLAIN ANALYZE
SELECT
    f.id,
    f.new_filename,
    mm.detected_bpm,
    mm.detected_key,
    f.category
FROM files f
INNER JOIN musical_metadata mm ON f.id = mm.file_id
WHERE mm.detected_bpm BETWEEN 120 AND 140
  AND mm.detected_key = 'Cm'
  AND f.category = 'BASS'
ORDER BY mm.detected_bpm
LIMIT 100;

-- Q3: Browse by manufacturer/collection
-- Usage: Show all DMS Liquid Trance files
EXPLAIN ANALYZE
SELECT
    f.id,
    f.new_filename,
    mm.detected_bpm,
    mm.detected_key,
    f.category
FROM files f
LEFT JOIN musical_metadata mm ON f.id = mm.file_id
WHERE f.manufacturer = 'DMS'
  AND f.collection_name LIKE 'Liquid Trance%'
ORDER BY f.new_filename;

-- ============================================================================
-- SIMILARITY QUERIES (pgvector)
-- ============================================================================

-- Q4: Find similar files by embedding
-- Usage: Find files similar to file_id = 1
EXPLAIN ANALYZE
SELECT
    f.id,
    f.new_filename,
    mm.detected_bpm,
    mm.detected_key,
    f.category,
    (mm.embedding <=> (
        SELECT embedding
        FROM musical_metadata
        WHERE file_id = 1
    )) AS distance
FROM files f
INNER JOIN musical_metadata mm ON f.id = mm.file_id
WHERE f.id != 1
  AND mm.embedding IS NOT NULL
ORDER BY distance
LIMIT 20;

-- Q5: Find files by category and BPM range
-- Usage: Find all kicks between 120-140 BPM
EXPLAIN ANALYZE
SELECT
    f.id,
    f.new_filename,
    mm.detected_bpm,
    f.category
FROM files f
INNER JOIN musical_metadata mm ON f.id = mm.file_id
WHERE f.category = 'KICK'
  AND mm.detected_bpm BETWEEN 120 AND 140
ORDER BY mm.detected_bpm
LIMIT 50;

-- ============================================================================
-- ARRAY QUERIES
-- ============================================================================

-- Q6: Find files with specific tags
-- Usage: Find files tagged with 'trance'
SELECT
    f.id,
    f.new_filename,
    f.auto_tags,
    f.user_tags,
    mm.detected_bpm
FROM files f
LEFT JOIN musical_metadata mm ON f.id = mm.file_id
WHERE 'trance' = ANY(f.auto_tags) OR 'trance' = ANY(f.user_tags)
LIMIT 100;

-- Q7: Find files by folder tags
-- Usage: Find files from specific folder structure
SELECT
    f.id,
    f.new_filename,
    f.folder_tags,
    f.manufacturer,
    f.collection_name
FROM files f
WHERE 'electronic' = ANY(f.folder_tags)
LIMIT 100;

-- ============================================================================
-- PIPELINE BATCH OPERATIONS
-- ============================================================================

-- Q8: Batch insert file (example with 1 file)
INSERT INTO files (
    original_path,
    current_path,
    original_filename,
    new_filename,
    content_hash,
    file_size,
    file_modified,
    manufacturer,
    collection_name,
    folder_tags,
    category,
    import_batch_id
) VALUES (
    '/original/path.mid',
    '/new/path.mid',
    'original.mid',
    'BASS_Cm_140BPM_Deep.mid',
    decode('abcd1234', 'hex'),
    2048,
    NOW(),
    'DMS',
    'Liquid Trance Vol.1',
    ARRAY['trance', 'liquid', 'bass'],
    'BASS',
    uuid_generate_v4()
) RETURNING id;

-- Q9: Find existing files by content hash (deduplication)
SELECT id, current_path, content_hash, new_filename
FROM files
WHERE content_hash = decode('abcd1234', 'hex');

-- Q10: Get import batch status
SELECT
    id,
    source_directory,
    status,
    files_processed,
    files_failed,
    started_at,
    completed_at,
    NOW() - started_at as elapsed_time
FROM import_batches
WHERE id = 'batch-uuid-here'::uuid;

-- ============================================================================
-- STATISTICS QUERIES
-- ============================================================================

-- Q11: Database statistics
SELECT
    'Total Files' as metric,
    COUNT(*) as value
FROM files
UNION ALL
SELECT 'Analyzed Files', COUNT(*)
FROM files WHERE analyzed_at IS NOT NULL
UNION ALL
SELECT 'Files with BPM', COUNT(*)
FROM musical_metadata WHERE detected_bpm IS NOT NULL
UNION ALL
SELECT 'Multi-track Files', COUNT(*)
FROM files WHERE is_multi_track = TRUE
UNION ALL
SELECT 'Manufacturers', COUNT(DISTINCT manufacturer)
FROM files WHERE manufacturer IS NOT NULL
UNION ALL
SELECT 'Collections', COUNT(DISTINCT collection_name)
FROM files WHERE collection_name IS NOT NULL;

-- Q12: BPM distribution
SELECT
    FLOOR(detected_bpm / 10) * 10 as bpm_range,
    COUNT(*) as count
FROM musical_metadata
WHERE detected_bpm IS NOT NULL
GROUP BY FLOOR(detected_bpm / 10)
ORDER BY bpm_range;

-- Q13: Category distribution
SELECT
    category,
    COUNT(*) as count
FROM files
WHERE category != 'UNKNOWN'
GROUP BY category
ORDER BY count DESC;

-- Q14: Key signature distribution
SELECT
    detected_key,
    COUNT(*) as count
FROM musical_metadata
WHERE detected_key != 'UNKNOWN'
GROUP BY detected_key
ORDER BY count DESC;

-- Q15: Files with tracks information
SELECT
    f.id,
    f.new_filename,
    f.category,
    COUNT(t.id) as track_count,
    ARRAY_AGG(t.track_name) as track_names
FROM files f
LEFT JOIN tracks t ON f.id = t.file_id
GROUP BY f.id, f.new_filename, f.category
LIMIT 100;

-- ============================================================================
-- INSTRUMENT QUERIES
-- ============================================================================

-- Q16: Find files using specific instrument
-- Usage: Find all files using piano (program 0-7)
SELECT
    f.id,
    f.new_filename,
    i.instrument_name,
    i.instrument_family,
    fi.note_count
FROM files f
INNER JOIN file_instruments fi ON f.id = fi.file_id
INNER JOIN instruments i ON fi.instrument_id = i.id
WHERE i.instrument_family = 'Piano'
ORDER BY fi.note_count DESC
LIMIT 100;

-- Q17: Most used instruments
SELECT
    i.instrument_name,
    i.instrument_family,
    COUNT(DISTINCT fi.file_id) as file_count,
    SUM(fi.note_count) as total_notes
FROM instruments i
INNER JOIN file_instruments fi ON i.id = fi.instrument_id
GROUP BY i.id, i.instrument_name, i.instrument_family
ORDER BY file_count DESC
LIMIT 20;

-- ============================================================================
-- TAG QUERIES
-- ============================================================================

-- Q18: Most popular tags
SELECT
    t.name,
    t.category,
    COUNT(ft.file_id) as file_count
FROM tags t
INNER JOIN file_tags ft ON t.id = ft.tag_id
GROUP BY t.id, t.name, t.category
ORDER BY file_count DESC
LIMIT 50;

-- Q19: Find files with multiple tags
-- Usage: Find files tagged with both 'trance' and 'melodic'
SELECT
    f.id,
    f.new_filename,
    ARRAY_AGG(DISTINCT t.name) as tags
FROM files f
INNER JOIN file_tags ft ON f.id = ft.file_id
INNER JOIN tags t ON ft.tag_id = t.id
WHERE t.name IN ('trance', 'melodic')
GROUP BY f.id, f.new_filename
HAVING COUNT(DISTINCT t.name) = 2
LIMIT 100;

-- ============================================================================
-- DUPLICATE DETECTION
-- ============================================================================

-- Q20: Find duplicate groups
SELECT
    dg.id as group_id,
    dg.file_count,
    dg.canonical_file_id,
    f.new_filename as canonical_filename
FROM duplicate_groups dg
LEFT JOIN files f ON dg.canonical_file_id = f.id
WHERE dg.file_count > 1
ORDER BY dg.file_count DESC;

-- Q21: Find all files in a duplicate group
SELECT
    f.id,
    f.new_filename,
    f.current_path,
    f.file_size,
    f.created_at,
    dg.id = f.id as is_canonical
FROM files f
INNER JOIN duplicate_groups dg ON f.content_hash = dg.content_hash
WHERE dg.content_hash = (SELECT content_hash FROM files WHERE id = 1)
ORDER BY f.created_at;

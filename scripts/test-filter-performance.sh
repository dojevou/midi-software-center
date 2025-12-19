#!/bin/bash

# Test VIP3 filter count performance
# Target: <50ms for all filter count queries

set -e

DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

echo "========================================="
echo "VIP3 Filter Count Performance Test"
echo "Target: <50ms per query"
echo "========================================="

# Test 1: Parent folder counts
echo ""
echo "Test 1: Parent folder counts"
psql "$DB_URL" -c "
  EXPLAIN (ANALYZE, BUFFERS, TIMING)
  SELECT parent_folder, COUNT(DISTINCT id)::int8 as count
  FROM files
  WHERE parent_folder IS NOT NULL
  GROUP BY parent_folder;
" 2>&1 | grep "Execution Time"

# Test 2: BPM range counts (100-120)
echo ""
echo "Test 2: BPM range counts (100-120)"
psql "$DB_URL" -c "
  EXPLAIN (ANALYZE, BUFFERS, TIMING)
  SELECT COUNT(DISTINCT f.id)::int8
  FROM files f
  JOIN musical_metadata mm ON f.id = mm.file_id
  WHERE mm.bpm >= 100 AND mm.bpm < 120;
" 2>&1 | grep "Execution Time"

# Test 3: Instrument tag counts
echo ""
echo "Test 3: Instrument tag counts"
psql "$DB_URL" -c "
  EXPLAIN (ANALYZE, BUFFERS, TIMING)
  SELECT tag_id, COUNT(DISTINCT file_id)::int8 as count
  FROM file_tags
  GROUP BY tag_id
  LIMIT 10;
" 2>&1 | grep "Execution Time"

# Test 4: Timbre counts (if table has data)
echo ""
echo "Test 4: Timbre counts"
psql "$DB_URL" -c "
  EXPLAIN (ANALYZE, BUFFERS, TIMING)
  SELECT timbre_id, COUNT(DISTINCT file_id)::int8 as count
  FROM midi_file_timbres
  GROUP BY timbre_id
  LIMIT 10;
" 2>&1 | grep "Execution Time" || echo "  (No data or table doesn't exist yet)"

# Test 5: Combined filters (folder + BPM)
echo ""
echo "Test 5: Combined filters (parent_folder + BPM range)"
psql "$DB_URL" -c "
  EXPLAIN (ANALYZE, BUFFERS, TIMING)
  SELECT COUNT(DISTINCT f.id)::int8
  FROM files f
  JOIN musical_metadata mm ON f.id = mm.file_id
  WHERE f.parent_folder IS NOT NULL
    AND mm.bpm >= 100 AND mm.bpm < 120
  LIMIT 10;
" 2>&1 | grep "Execution Time"

# Test 6: Multi-track filter
echo ""
echo "Test 6: Multi-track file counts"
psql "$DB_URL" -c "
  EXPLAIN (ANALYZE, BUFFERS, TIMING)
  SELECT is_multi_track, COUNT(*)::int8 as count
  FROM files
  GROUP BY is_multi_track;
" 2>&1 | grep "Execution Time"

# Test 7: Track count ranges
echo ""
echo "Test 7: Track layer counts"
psql "$DB_URL" -c "
  EXPLAIN (ANALYZE, BUFFERS, TIMING)
  SELECT
    CASE
      WHEN num_tracks = 1 THEN 'single'
      ELSE 'multi'
    END as layer,
    COUNT(*)::int8 as count
  FROM files
  GROUP BY layer;
" 2>&1 | grep "Execution Time"

# Test 8: Index usage verification
echo ""
echo "========================================="
echo "Index Usage Verification"
echo "========================================="

# Check if indexes are being scanned
echo ""
echo "Most used indexes (by scans):"
psql "$DB_URL" -c "
  SELECT
    schemaname,
    tablename,
    indexname,
    idx_scan as scans,
    idx_tup_read as tuples_read,
    idx_tup_fetch as tuples_fetched
  FROM pg_stat_user_indexes
  WHERE schemaname = 'public'
    AND indexname LIKE '%_opt'
  ORDER BY idx_scan DESC
  LIMIT 10;
"

echo ""
echo "========================================="
echo "All tests complete!"
echo "Check that execution times are <50ms"
echo "========================================="

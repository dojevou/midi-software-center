#!/bin/bash
# =============================================================================
# Verify Track Split Parent-Child Relationships
# =============================================================================
# Purpose: Ensure no orphaned files after splitting
# - Parents: Multitrack files that have been split
# - Children: Split track files
# - Orphans: Files without proper relationships
# =============================================================================

DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Track Split Relationship Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# =============================================================================
# 1. Check total split relationships
# =============================================================================
echo "1. Total Split Relationships:"
psql "$DB_URL" -c "
SELECT
    COUNT(*) as total_relationships,
    COUNT(DISTINCT parent_file_id) as unique_parents,
    COUNT(DISTINCT split_file_id) as unique_children
FROM track_splits;
"
echo ""

# =============================================================================
# 2. Check for orphaned child files (split files not in track_splits)
# =============================================================================
echo "2. Orphaned Child Files (split files without parent relationship):"
psql "$DB_URL" -c "
SELECT
    COUNT(*) as orphaned_children,
    ARRAY_AGG(f.id ORDER BY f.id LIMIT 10) as sample_ids
FROM files f
WHERE f.num_tracks = 1
  AND f.filename LIKE '%_0%' -- Split files typically have _01, _02 pattern
  AND NOT EXISTS (
      SELECT 1 FROM track_splits ts WHERE ts.split_file_id = f.id
  );
"
echo ""

# =============================================================================
# 3. Check for orphaned parent files (multitrack files not in track_splits)
# =============================================================================
echo "3. Multitrack Files Pending Split (parents without children yet):"
psql "$DB_URL" -c "
SELECT
    COUNT(*) as pending_split,
    ARRAY_AGG(f.id ORDER BY f.id LIMIT 10) as sample_ids
FROM files f
WHERE f.num_tracks > 1
  AND NOT EXISTS (
      SELECT 1 FROM track_splits ts WHERE ts.parent_file_id = f.id
  );
"
echo ""

# =============================================================================
# 4. Check referential integrity (foreign keys)
# =============================================================================
echo "4. Referential Integrity Check:"
psql "$DB_URL" -c "
-- Check for invalid parent_file_id references
SELECT
    'Invalid parent references' as check_type,
    COUNT(*) as count
FROM track_splits ts
LEFT JOIN files f ON ts.parent_file_id = f.id
WHERE f.id IS NULL

UNION ALL

-- Check for invalid split_file_id references
SELECT
    'Invalid child references' as check_type,
    COUNT(*) as count
FROM track_splits ts
LEFT JOIN files f ON ts.split_file_id = f.id
WHERE f.id IS NULL;
"
echo ""

# =============================================================================
# 5. Sample parent-child relationships
# =============================================================================
echo "5. Sample Parent-Child Relationships (first 10):"
psql "$DB_URL" -c "
SELECT
    p.id as parent_id,
    p.filename as parent_filename,
    p.num_tracks as parent_tracks,
    ts.track_number,
    s.id as child_id,
    s.filename as child_filename
FROM track_splits ts
JOIN files p ON ts.parent_file_id = p.id
JOIN files s ON ts.split_file_id = s.id
ORDER BY p.id, ts.track_number
LIMIT 10;
"
echo ""

# =============================================================================
# 6. Statistics by parent file
# =============================================================================
echo "6. Split Statistics (files with most tracks):"
psql "$DB_URL" -c "
SELECT
    p.id,
    p.filename,
    p.num_tracks as expected_tracks,
    COUNT(ts.id) as actual_splits,
    CASE
        WHEN p.num_tracks = COUNT(ts.id) THEN '✓ Complete'
        ELSE '⚠ Incomplete'
    END as status
FROM files p
LEFT JOIN track_splits ts ON ts.parent_file_id = p.id
WHERE p.num_tracks > 1
  AND EXISTS (SELECT 1 FROM track_splits WHERE parent_file_id = p.id)
GROUP BY p.id, p.filename, p.num_tracks
ORDER BY p.num_tracks DESC
LIMIT 15;
"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Verification Complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

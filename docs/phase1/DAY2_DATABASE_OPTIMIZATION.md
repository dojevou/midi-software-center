# Day 2: Database Optimization for Filter Counts

**Duration:** 3 hours
**Prerequisites:** Day 1 (Parts 1A-1D) completed
**Files to create:** 1 migration

---

## Overview

Day 1 implemented the filter counts system. It works, but may be slow (>50ms). Today you'll:
1. Analyze current query performance
2. Create partial indexes for VIP3 filter columns
3. Add composite indexes for common filter combinations
4. Test and verify <50ms performance target

**Performance Target:** All filter count queries complete in <50ms total (8 parallel queries)

---

## Step 1: Performance Baseline (30 min)

### Measure Current Performance

Start the app and test filter counts:

```bash
make dev
```

In the VIP3 browser, open browser console and run:

```javascript
// Measure baseline performance
const start = performance.now();
await window.__TAURI__.invoke('get_vip3_filter_counts', { filters: {} });
const elapsed = performance.now() - start;
console.log(`Baseline performance: ${elapsed.toFixed(0)}ms`);

// Test with filters
const startFiltered = performance.now();
await window.__TAURI__.invoke('get_vip3_filter_counts', {
  filters: { folder_ids: [1, 2] }
});
const elapsedFiltered = performance.now() - startFiltered;
console.log(`Filtered performance: ${elapsedFiltered.toFixed(0)}ms`);
```

**Expected baseline (no indexes):** 100-300ms
**Target after optimization:** <50ms

### Analyze Query Plans

Connect to database:

```bash
psql "postgresql://midiuser:145278963@localhost:5433/midi_library"
```

Run EXPLAIN ANALYZE on a filter count query:

```sql
EXPLAIN ANALYZE
SELECT f.folder_id, COUNT(DISTINCT f.id)::int8 as count
FROM files f
LEFT JOIN musical_metadata mm ON f.id = mm.file_id
LEFT JOIN file_tags ft ON f.id = ft.file_id
LEFT JOIN tags t ON ft.tag_id = t.id
LEFT JOIN file_timbres ftim ON f.id = ftim.file_id
WHERE f.folder_id IS NOT NULL
GROUP BY f.folder_id;
```

Look for:
- **Seq Scan** (bad - means no index used)
- **Index Scan** (good - index used)
- **Hash Join** (acceptable for joins)
- **Planning time** and **Execution time**

**Save the output** - you'll compare after adding indexes.

---

## Step 2: Create Migration File (45 min)

Create `database/migrations/020_add_filter_count_indexes.sql`:

```sql
-- Migration 020: Add indexes for VIP3 filter count performance
-- Target: Reduce filter count queries from 100-300ms to <50ms
-- Date: 2025-12-15

BEGIN;

-- ============================================================================
-- PARTIAL INDEXES FOR FILTER COLUMNS
-- These indexes include only non-NULL values, reducing index size
-- ============================================================================

-- Folder filter index (most common filter)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_files_folder_id_partial
ON files(folder_id)
WHERE folder_id IS NOT NULL;

-- Musical metadata indexes for BPM and key filtering
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_musical_metadata_bpm_partial
ON musical_metadata(bpm)
WHERE bpm IS NOT NULL;

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_musical_metadata_key_partial
ON musical_metadata(key_signature)
WHERE key_signature IS NOT NULL;

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_musical_metadata_channels_partial
ON musical_metadata(channel_count)
WHERE channel_count IS NOT NULL;

-- ============================================================================
-- COMPOSITE INDEXES FOR COMMON FILTER COMBINATIONS
-- These speed up multi-column filtering
-- ============================================================================

-- Folder + BPM (common combination)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_files_folder_bpm
ON files(folder_id, id)
INCLUDE (id);

-- File-to-metadata join optimization
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_musical_metadata_file_bpm
ON musical_metadata(file_id, bpm)
WHERE bpm IS NOT NULL;

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_musical_metadata_file_key
ON musical_metadata(file_id, key_signature)
WHERE key_signature IS NOT NULL;

-- ============================================================================
-- JUNCTION TABLE INDEXES FOR VIP3 CATEGORIES
-- Speed up file_timbres, file_styles, file_articulations joins
-- ============================================================================

-- Timbre filtering
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_file_timbres_timbre_file
ON file_timbres(timbre_id, file_id);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_file_timbres_file_timbre
ON file_timbres(file_id, timbre_id);

-- Style filtering
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_file_styles_style_file
ON file_styles(style_id, file_id);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_file_styles_file_style
ON file_styles(file_id, style_id);

-- Articulation filtering
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_file_articulations_articulation_file
ON file_articulations(articulation_id, file_id);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_file_articulations_file_articulation
ON file_articulations(file_id, articulation_id);

-- ============================================================================
-- COVERING INDEXES FOR COUNT QUERIES
-- Include commonly selected columns to avoid table lookups
-- ============================================================================

-- Instrument tag counts (file_tags + tags join)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_file_tags_tag_file_covering
ON file_tags(tag_id, file_id)
INCLUDE (file_id);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tags_id_name_covering
ON tags(id)
INCLUDE (name);

-- ============================================================================
-- VACUUM AND ANALYZE
-- Update table statistics for query planner
-- ============================================================================

VACUUM ANALYZE files;
VACUUM ANALYZE musical_metadata;
VACUUM ANALYZE file_tags;
VACUUM ANALYZE tags;
VACUUM ANALYZE file_timbres;
VACUUM ANALYZE file_styles;
VACUUM ANALYZE file_articulations;

COMMIT;

-- ============================================================================
-- VERIFICATION QUERIES
-- Run these to verify indexes are being used
-- ============================================================================

-- Check index usage for folder counts
EXPLAIN (ANALYZE, BUFFERS)
SELECT folder_id, COUNT(*)
FROM files
WHERE folder_id IS NOT NULL
GROUP BY folder_id;

-- Check index usage for BPM range counts
EXPLAIN (ANALYZE, BUFFERS)
SELECT COUNT(*)
FROM files f
JOIN musical_metadata mm ON f.id = mm.file_id
WHERE mm.bpm >= 100 AND mm.bpm < 120;

-- Check index usage for timbre counts
EXPLAIN (ANALYZE, BUFFERS)
SELECT timbre_id, COUNT(DISTINCT file_id)
FROM file_timbres
GROUP BY timbre_id;

-- ============================================================================
-- INDEX SIZE REPORT
-- Monitor index sizes to ensure they're reasonable
-- ============================================================================

SELECT
    schemaname,
    tablename,
    indexname,
    pg_size_pretty(pg_relation_size(indexrelid)) AS index_size
FROM pg_stat_user_indexes
WHERE schemaname = 'public'
    AND indexname LIKE 'idx_%'
ORDER BY pg_relation_size(indexrelid) DESC;
```

---

## Step 3: Apply Migration (15 min)

Run the migration:

```bash
make db-migrate
```

**Expected output:**
```
Running migration: 020_add_filter_count_indexes.sql
CREATE INDEX
CREATE INDEX
...
VACUUM
ANALYZE
Migration completed successfully
```

**Note:** `CREATE INDEX CONCURRENTLY` may take 1-5 minutes depending on database size. This is normal and safe to run on production without locking tables.

### Verify Indexes Created

```bash
psql "postgresql://midiuser:145278963@localhost:5433/midi_library"
```

```sql
-- List all new indexes
SELECT indexname, tablename
FROM pg_indexes
WHERE schemaname = 'public'
    AND indexname LIKE 'idx_%filter%'
    OR indexname LIKE 'idx_%timbre%'
    OR indexname LIKE 'idx_%style%'
    OR indexname LIKE 'idx_%articulation%'
ORDER BY tablename, indexname;
```

**Expected:** At least 15 new indexes listed

---

## Step 4: Performance Testing (1 hour)

### Test 1: Empty Filters (Baseline)

```javascript
// In browser console
const results = [];

for (let i = 0; i < 10; i++) {
  const start = performance.now();
  await window.__TAURI__.invoke('get_vip3_filter_counts', { filters: {} });
  const elapsed = performance.now() - start;
  results.push(elapsed);
}

const avg = results.reduce((a, b) => a + b) / results.length;
console.log(`Average: ${avg.toFixed(0)}ms (10 runs)`);
console.log(`Min: ${Math.min(...results).toFixed(0)}ms`);
console.log(`Max: ${Math.max(...results).toFixed(0)}ms`);
```

**Target:** Average <50ms

### Test 2: Single Folder Filter

```javascript
const results = [];

for (let i = 0; i < 10; i++) {
  const start = performance.now();
  await window.__TAURI__.invoke('get_vip3_filter_counts', {
    filters: { folder_ids: [1] }
  });
  const elapsed = performance.now() - start;
  results.push(elapsed);
}

const avg = results.reduce((a, b) => a + b) / results.length;
console.log(`Average with folder filter: ${avg.toFixed(0)}ms`);
```

**Target:** Average <50ms

### Test 3: Multiple Filters

```javascript
const results = [];

for (let i = 0; i < 10; i++) {
  const start = performance.now();
  await window.__TAURI__.invoke('get_vip3_filter_counts', {
    filters: {
      folder_ids: [1, 2],
      instrument_ids: [5],
      timbre_ids: [1],
      bpm_min: 100,
      bpm_max: 140
    }
  });
  const elapsed = performance.now() - start;
  results.push(elapsed);
}

const avg = results.reduce((a, b) => a + b) / results.length;
console.log(`Average with multiple filters: ${avg.toFixed(0)}ms`);
```

**Target:** Average <50ms

### Test 4: Query Plan Analysis

Check that indexes are being used:

```sql
-- In psql
EXPLAIN (ANALYZE, BUFFERS)
SELECT folder_id, COUNT(DISTINCT id)::int8 as count
FROM files
WHERE folder_id IS NOT NULL
GROUP BY folder_id;
```

**Look for:**
- `Index Scan using idx_files_folder_id_partial` (GOOD)
- NOT `Seq Scan on files` (BAD - means index not used)

**Example good output:**
```
Index Scan using idx_files_folder_id_partial on files
  Planning time: 0.5ms
  Execution time: 12.3ms
```

---

## Step 5: Create Performance Test Script (30 min)

Create `scripts/test-filter-performance.sh`:

```bash
#!/bin/bash

# Test VIP3 filter count performance
# Target: <50ms for all filter count queries

set -e

DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

echo "========================================="
echo "VIP3 Filter Count Performance Test"
echo "Target: <50ms"
echo "========================================="

# Test 1: Folder counts
echo ""
echo "Test 1: Folder counts"
psql "$DB_URL" -c "
  EXPLAIN (ANALYZE, BUFFERS, TIMING)
  SELECT folder_id, COUNT(DISTINCT id)::int8 as count
  FROM files
  WHERE folder_id IS NOT NULL
  GROUP BY folder_id;
" | grep "Execution Time"

# Test 2: BPM range counts
echo ""
echo "Test 2: BPM range counts (100-120)"
psql "$DB_URL" -c "
  EXPLAIN (ANALYZE, BUFFERS, TIMING)
  SELECT COUNT(DISTINCT f.id)::int8
  FROM files f
  JOIN musical_metadata mm ON f.id = mm.file_id
  WHERE mm.bpm >= 100 AND mm.bpm < 120;
" | grep "Execution Time"

# Test 3: Timbre counts
echo ""
echo "Test 3: Timbre counts"
psql "$DB_URL" -c "
  EXPLAIN (ANALYZE, BUFFERS, TIMING)
  SELECT timbre_id, COUNT(DISTINCT file_id)::int8
  FROM file_timbres
  GROUP BY timbre_id;
" | grep "Execution Time"

# Test 4: Instrument tag counts
echo ""
echo "Test 4: Instrument tag counts"
psql "$DB_URL" -c "
  EXPLAIN (ANALYZE, BUFFERS, TIMING)
  SELECT tag_id, COUNT(DISTINCT file_id)::int8
  FROM file_tags
  GROUP BY tag_id;
" | grep "Execution Time"

# Test 5: Combined filters
echo ""
echo "Test 5: Combined filters (folder + BPM)"
psql "$DB_URL" -c "
  EXPLAIN (ANALYZE, BUFFERS, TIMING)
  SELECT COUNT(DISTINCT f.id)::int8
  FROM files f
  JOIN musical_metadata mm ON f.id = mm.file_id
  WHERE f.folder_id = 1
    AND mm.bpm >= 100 AND mm.bpm < 120;
" | grep "Execution Time"

echo ""
echo "========================================="
echo "All tests complete!"
echo "Check that execution times are <50ms"
echo "========================================="
```

Make it executable:

```bash
chmod +x scripts/test-filter-performance.sh
```

Run it:

```bash
./scripts/test-filter-performance.sh
```

**Expected output:**
```
Test 1: Folder counts
 Execution Time: 8.234 ms

Test 2: BPM range counts (100-120)
 Execution Time: 12.567 ms

Test 3: Timbre counts
 Execution Time: 6.123 ms

Test 4: Instrument tag counts
 Execution Time: 9.876 ms

Test 5: Combined filters (folder + BPM)
 Execution Time: 15.432 ms
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Migration fails | Check PostgreSQL version ≥12 for `CONCURRENTLY` support |
| Indexes not used | Run `VACUUM ANALYZE` on tables, check query planner stats |
| Still >50ms | Check database size, may need to increase `work_mem` |
| "Index already exists" | Safe to ignore, migration is idempotent |
| Slow index creation | Normal for large datasets, use `CONCURRENTLY` to avoid locks |

### If Performance Still >50ms

1. **Increase PostgreSQL work_mem:**
   ```sql
   ALTER DATABASE midi_library SET work_mem = '256MB';
   ```

2. **Check database statistics:**
   ```sql
   SELECT schemaname, tablename, n_live_tup
   FROM pg_stat_user_tables
   WHERE schemaname = 'public'
   ORDER BY n_live_tup DESC;
   ```

3. **Re-analyze tables:**
   ```bash
   psql "$DB_URL" -c "VACUUM ANALYZE files;"
   psql "$DB_URL" -c "VACUUM ANALYZE musical_metadata;"
   ```

---

## Verification Checklist

- [ ] Migration 020 applied successfully
- [ ] At least 15 new indexes created
- [ ] `scripts/test-filter-performance.sh` shows <50ms execution times
- [ ] Browser console shows filter counts <50ms
- [ ] Query plans show `Index Scan` (not `Seq Scan`)
- [ ] No performance warnings in Rust logs

---

## What's Next?

✅ **Day 1-2 Complete! Filter counts system fully optimized:**
- ✅ Backend models and repository (Day 1A)
- ✅ Tauri commands with parallel execution (Day 1B)
- ✅ Frontend API and store (Day 1C)
- ✅ UI components with real-time counts (Day 1D)
- ✅ Database indexes for <50ms performance (Day 2)

**Next:** [Day 3, Part 3A: Saved Searches Backend](./DAY3_PART_A_SAVED_SEARCHES_BACKEND.md)
- Create SavedSearch model
- Implement SavedSearchRepository
- Add Tauri commands for save/load/delete
- Store filter state with use counts

**Current Performance:**
- Filter counts: <50ms ✓
- Real-time updates on filter changes ✓
- Disabled states for 0-count filters ✓

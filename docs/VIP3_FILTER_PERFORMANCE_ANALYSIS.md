# VIP3 Filter Count Performance Analysis

**Date:** 2025-12-15
**Database:** PostgreSQL 16, 2.1M files, 7.9M file_tags
**Target:** <50ms for all filter count queries

---

## Summary

✅ **Migration 023 Successfully Applied** - Created 18 performance optimization indexes
❌ **Performance Target Not Met** - Most queries exceed 50ms target
⚠️ **Migration 022 Not Applied** - Missing critical `folder_id` denormalized column

---

## Current Performance Results

| Test | Query Type | Time | Target | Status |
|------|-----------|------|--------|--------|
| 1 | Parent folder counts | **0.15ms** | 50ms | ✅ PASS |
| 2 | BPM range counts (100-120) | **1047ms** | 50ms | ❌ FAIL |
| 3 | Instrument tag counts | **340ms** | 50ms | ❌ FAIL |
| 4 | Timbre counts | **207ms** | 50ms | ❌ FAIL |
| 5 | Combined (folder + BPM) | **0.17ms** | 50ms | ✅ PASS |
| 6 | Multi-track counts | **405ms** | 50ms | ❌ FAIL |
| 7 | Track layer counts | **233ms** | 50ms | ❌ FAIL |

---

## Root Cause Analysis

### Why Some Queries Are Slow

The slow queries are performing **full aggregations** over millions of rows:

```sql
-- This query scans 247K rows even with index
SELECT COUNT(DISTINCT f.id)
FROM files f
JOIN musical_metadata mm ON f.id = mm.file_id
WHERE mm.bpm >= 100 AND mm.bpm < 120;
```

**Query plan shows:**
- ✅ Index IS being used (`idx_musical_metadata_file_bpm_opt`)
- ✅ Index scan (not sequential scan)
- ❌ Still slow because it must count 247K matching rows

### Why Some Queries Are Fast

The fast queries use **direct GROUP BY** on indexed columns:

```sql
-- This query uses index-only scan and is FAST
SELECT parent_folder, COUNT(DISTINCT id)
FROM files
WHERE parent_folder IS NOT NULL
GROUP BY parent_folder;
```

---

## Indexes Created (Migration 023)

✅ **18 indexes successfully created:**

### File_tags (2 covering indexes)
- `idx_file_tags_tag_file_covering_opt` - Tag → files lookup
- `idx_file_tags_file_tag_covering_opt` - File → tags lookup

### Musical_metadata (4 indexes)
- `idx_musical_metadata_bpm_range_opt` - BPM filtering
- `idx_musical_metadata_file_bpm_opt` - File + BPM join
- `idx_musical_metadata_key_range_opt` - Key filtering
- `idx_musical_metadata_file_key_opt` - File + key join

### Files (8 indexes)
- `idx_files_parent_folder_opt` - Folder filtering
- `idx_files_collection_name_opt` - Collection filtering
- `idx_files_is_multi_track_opt` - Multi-track filter
- `idx_files_num_tracks_opt` - Track count filter
- `idx_files_parent_folder_id_opt` - Folder + file composite
- `idx_files_collection_id_opt` - Collection + file composite

### VIP3 Junction Tables (6 indexes)
- `idx_midi_file_timbres_timbre_file_opt`
- `idx_midi_file_timbres_file_timbre_opt`
- `idx_midi_file_styles_style_file_opt`
- `idx_midi_file_styles_file_style_opt`
- `idx_midi_file_articulations_articulation_file_opt`
- `idx_midi_file_articulations_file_articulation_opt`

---

## Missing Dependencies

### Migration 022 Not Applied

Migration [022_add_filter_columns_and_indexes.sql](../database/migrations/022_add_filter_columns_and_indexes.sql) was NOT applied due to dependency issues:

**Migration 022 depends on Migration 019 which:**
- Creates `bpm_ranges` table
- Creates `musical_keys` table
- Creates VIP3 category tables

**Migration 022 would add:**
- `files.folder_id BIGINT` - Denormalized folder ID for fast filtering
- `files.bpm_range_id SMALLINT` - Denormalized BPM range ID
- `files.key_id SMALLINT` - Denormalized key ID
- `files.channel_count SMALLINT` - Denormalized channel count
- Triggers to auto-populate these columns
- Additional composite indexes

**Current Rust code expects `folder_id` to exist** (see [vip3_repository.rs:109](../app/src-tauri/src/db/repositories/vip3_repository.rs#L109)):
```rust
conditions.push(format!("f.folder_id IN ({})", ids_str));
//                      ^^^^^^^^^^^ DOES NOT EXIST YET
```

---

## Recommendations

### Option 1: Apply Migration 019 + 022 (Complete VIP3 System)

**Pros:**
- Gets full VIP3 filtering system
- Denormalized columns will speed up queries
- Auto-population triggers keep data in sync

**Cons:**
- Migration 019 has dependency issues (references `normalized_filename` column)
- May require fixing migration 019 first

**Steps:**
1. Fix migration 019 dependencies (remove `normalized_filename` references)
2. Apply migration 019
3. Apply migration 022
4. Re-test performance

### Option 2: Use Materialized Views (Quick Win)

Create materialized views for expensive aggregations:

```sql
-- Create materialized view for BPM range counts
CREATE MATERIALIZED VIEW mv_bpm_range_counts AS
SELECT
    CASE
        WHEN bpm < 60 THEN 1
        WHEN bpm >= 60 AND bpm < 80 THEN 2
        -- ... more ranges
    END as range_id,
    COUNT(DISTINCT file_id) as count
FROM musical_metadata
WHERE bpm IS NOT NULL
GROUP BY range_id;

-- Refresh periodically (or on-demand)
REFRESH MATERIALIZED VIEW mv_bpm_range_counts;
```

**Pros:**
- Fast queries (<1ms)
- Works with existing schema
- No code changes needed

**Cons:**
- Counts may be slightly stale
- Requires periodic refresh
- Extra storage overhead

### Option 3: Accept Current Performance

With 2.1M files, some aggregation queries will naturally take time:

- **BPM counts:** 1s (scanning 247K rows)
- **Tag counts:** 340ms (scanning 7.9M file_tags)

**This is reasonable performance for:**
- Initial page load (one-time cost)
- Background refresh every few minutes
- Admin/analytics queries

**Not suitable for:**
- Real-time filtering on every keystroke
- Interactive UI updates

---

## Performance Test Script

Created: [scripts/test-filter-performance.sh](../scripts/test-filter-performance.sh)

**Usage:**
```bash
./scripts/test-filter-performance.sh
```

**Tests:**
1. Parent folder counts
2. BPM range counts (100-120)
3. Instrument tag counts
4. Timbre counts
5. Combined filters
6. Multi-track counts
7. Track layer counts
8. Index usage verification

---

## Next Steps

### Immediate (Day 2 Complete)
- ✅ Migration 023 applied
- ✅ 18 indexes created
- ✅ Performance baseline measured
- ✅ Test script created

### Short Term (Day 3)
1. **Fix Migration 019** - Remove `normalized_filename` dependency
2. **Apply Migration 019** - Get VIP3 lookup tables
3. **Apply Migration 022** - Get denormalized columns
4. **Re-test Performance** - Verify <50ms target with denormalized data

### Medium Term (Week 2)
1. **Evaluate Materialized Views** - For static/slowly-changing counts
2. **Consider Caching Layer** - Redis for frequently accessed counts
3. **Query Optimization** - Review slow queries for further optimization

---

## Conclusion

**Migration 023 successfully applied** with 18 new indexes. However, the **<50ms performance target was not achieved** for most queries due to:

1. **Large dataset** (2.1M files, 7.9M tags)
2. **Missing denormalized columns** (folder_id, bpm_range_id, etc. from migration 022)
3. **Full aggregation queries** (COUNT DISTINCT over millions of rows)

The indexes ARE being used correctly, but PostgreSQL still needs to scan hundreds of thousands of rows to compute the counts.

**Next action:** Apply migrations 019 and 022 to get the denormalized columns, which should significantly improve performance for folder and BPM range filtering.

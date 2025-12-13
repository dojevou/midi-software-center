# Performance Optimization Quick Reference

**MIDI Software Center - Performance Oracle**

---

## Key Performance Metrics

| Component | Current | Target | Gap |
|-----------|---------|--------|-----|
| Import throughput | 7,830 files/sec | 10,000+ files/sec | -22% |
| Analysis throughput | 180-360 files/sec | 500+ files/sec | -28% to -64% |
| Search response time | <100ms (unfiltered) | <50ms | 2x slower |
| Memory peak | ~32MB/batch | ~3-5MB/batch | 6-10x bloat |
| Connection utilization | 50 max (saturation) | <80% saturation | Oversubscribed |

---

## Critical Bottlenecks (Ranked by Impact)

### 🔴 CRITICAL #1: Over-Fetching Columns
- **File:** file_repository.rs
- **Problem:** SELECT 25 columns when 4-8 needed
- **Fix:** Create specialized query methods with projection
- **Impact:** 40-60% faster queries
- **Time:** 2 hours
- **Code reference:** Lines 76-181 (find_by_id), 308-357 (list), etc.

### 🔴 CRITICAL #2: Unoptimized Full-Text Search
- **File:** search_repository.rs
- **Problem:** Computes ts_rank() for all rows, even when sorting by date
- **Fix:** Use covering index or move ts_rank to ORDER BY
- **Impact:** 6-15x faster searches (200ms → 20-50ms)
- **Time:** 1 hour
- **Code reference:** Lines 87-92

### 🔴 CRITICAL #3: Connection Pool Undersized
- **File:** database/mod.rs, commands/analyze.rs
- **Problem:** 50 max connections with 32 concurrent workers
- **Fix:** Increase pool or reduce workers with larger batches
- **Impact:** 40-50% throughput stability
- **Time:** 2 hours
- **Code reference:** Lines 150-173, 209

### 🟠 HIGH #4: Memory Bloat in Analysis
- **File:** commands/analyze.rs
- **Problem:** Materializes all 1000 files in memory before processing
- **Fix:** Stream instead of Vec, reduce buffer size
- **Impact:** 75-90% memory reduction (32MB → 3-5MB)
- **Time:** 3 hours
- **Code reference:** Lines 221-340

### 🟡 MEDIUM #5: Missing Covering Indexes
- **File:** database migrations
- **Problem:** Standard B-tree indexes, no covering indexes
- **Fix:** Add 4-6 covering indexes for high-frequency queries
- **Impact:** 50-80% cache miss reduction
- **Time:** 1 hour
- **Queries affected:** Search results, manufacturer listing, tag retrieval

---

## Quick Fix Implementation Guide

### Fix #1: Query Selectivity (2 hours)

**Step 1:** Create specialized models
```rust
// Add to file_repository.rs
#[derive(Debug, Clone)]
pub struct FileEssential {
    pub id: i64,
    pub filename: String,
    pub filepath: String,
    pub file_size_bytes: i64,
}

#[derive(Debug, Clone)]
pub struct FileForAnalysis {
    pub id: i64,
    pub filepath: String,
    pub filename: String,
    pub num_tracks: i16,
    pub duration_seconds: Option<BigDecimal>,
}
```

**Step 2:** Create projection methods
```rust
impl FileRepository {
    pub async fn find_by_id_essential(
        pool: &PgPool,
        id: i64,
    ) -> Result<Option<FileEssential>, sqlx::Error> {
        sqlx::query_as!(
            FileEssential,
            "SELECT id, filename, filepath, file_size_bytes FROM files WHERE id = $1",
            id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn find_for_analysis(
        pool: &PgPool,
        id: i64,
    ) -> Result<Option<FileForAnalysis>, sqlx::Error> {
        sqlx::query_as!(
            FileForAnalysis,
            r#"SELECT id, filepath, filename, num_tracks, duration_seconds
               FROM files WHERE id = $1"#,
            id
        )
        .fetch_optional(pool)
        .await
    }
}
```

**Step 3:** Update call sites
```rust
// Before: 25 columns selected
let file = FileRepository::find_by_id(&pool, id).await?;

// After: 4 columns selected (analysis flow)
let file = FileRepository::find_for_analysis(&pool, id).await?;

// Or: 4 columns selected (search results)
let file = FileRepository::find_by_id_essential(&pool, id).await?;
```

---

### Fix #2: Full-Text Search (1 hour)

**Option A (Quick):** Use covering index
```sql
-- Run once, takes ~5 seconds
CREATE INDEX CONCURRENTLY idx_search_covering ON files
    USING GIST(search_vector)
    INCLUDE (id, filename, filepath, manufacturer, created_at);

-- Query automatically uses index-only scan
-- No code changes needed!
```

**Option B (Better):** Move ts_rank to ORDER BY
```rust
// In search_repository.rs, line 41:
let files = sqlx::query_as!(
    File,
    r#"
    SELECT
        f.id, f.filename, f.filepath, ...
    FROM files f
    LEFT JOIN musical_metadata mm ON f.id = mm.file_id
    WHERE
        ($1::text IS NULL OR f.search_vector @@ plainto_tsquery('english', $1))
        AND ($2::float8 IS NULL OR mm.bpm::float8 >= $2)
        AND ($3::float8 IS NULL OR mm.bpm::float8 <= $3)
        AND ($4::text IS NULL OR mm.key_signature::text = $4)
        AND ($5::text IS NULL OR f.manufacturer = $5)
        AND ($6::text IS NULL OR f.collection_name = $6)
    ORDER BY
        CASE WHEN $1::text IS NOT NULL
            THEN ts_rank(f.search_vector, plainto_tsquery('english', $1))
            ELSE 0
        END DESC,
        f.created_at DESC
    LIMIT $7 OFFSET $8
    "#,
    ...
)
```

---

### Fix #3: Connection Pool (2 hours)

**Step 1:** Update pool configuration in database/mod.rs (line 150-173)
```rust
// BEFORE
.max_connections(pool_size as u32)  // ~50
.min_connections(min_connections)   // ~10

// AFTER
let pool_size = if pool_size < 100 {
    (concurrency * 2).min(100)  // 2x concurrency limit, max 100
} else {
    pool_size
};

let min_connections = (pool_size as f64 * 0.3) as u32;  // 30%

PgPoolOptions::new()
    .max_connections(pool_size as u32)
    .min_connections(min_connections)
    .acquire_timeout(Duration::from_secs(10))
    .idle_timeout(Duration::from_secs(120))  // 5min → 2min
    .max_lifetime(Duration::from_secs(600))  // 30min → 10min
```

**Step 2:** Monitor with metrics
```rust
pub async fn log_pool_stats(pool: &PgPool) {
    let size = pool.num_idle();
    let active = pool.size() - pool.num_idle();
    println!("📊 Pool stats: {} idle, {} active", size, active);
}
```

---

### Fix #4: Memory Optimization (3 hours)

**Before (materialize all):**
```rust
let files: Vec<FileRecord> = sqlx::query_as(
    "SELECT id, filepath, filename FROM files WHERE analyzed_at IS NULL LIMIT $1 OFFSET $2"
)
.bind(batch_size)
.bind(offset)
.fetch_all(&pool)  // ← LOADS ALL INTO VEC
.await?;

stream::iter(files)
    .buffer_unordered(32)
```

**After (streaming):**
```rust
// Option A: Use streaming directly
let mut rows = sqlx::query_as::<_, FileRecord>(
    "SELECT id, filepath, filename FROM files WHERE analyzed_at IS NULL LIMIT $1 OFFSET $2"
)
.bind(batch_size)
.bind(offset)
.fetch(&pool);  // ← RETURNS STREAM

stream::iter(rows)
    .then(|file_result| async {
        match file_result {
            Ok(file) => analyze_file(&file).await,
            Err(e) => Err(format!("Query error: {}", e)),
        }
    })
    .buffer_unordered(32)
```

**Option B: Reduce buffer (simpler)**
```rust
stream::iter(files)
    .buffer_unordered(8)  // 32 → 8 (4x memory reduction)
```

---

### Fix #5: Covering Indexes (1 hour)

**Run these migrations:**
```sql
-- Migration: 012_covering_indexes.sql

-- Search results covering index
CREATE INDEX CONCURRENTLY idx_search_results_covering ON files
    USING BTREE(filename)
    INCLUDE (id, filepath, file_size_bytes, manufacturer, created_at);

-- Manufacturer listing covering index
CREATE INDEX CONCURRENTLY idx_by_manufacturer_covering ON files
    USING BTREE(manufacturer, created_at DESC)
    INCLUDE (id, filename, filepath, collection_name);

-- File tags with file info
CREATE INDEX CONCURRENTLY idx_file_tags_covering ON file_tags
    USING BTREE(file_id, tag_id)
    INCLUDE (added_at);

-- BPM range with file details
CREATE INDEX CONCURRENTLY idx_metadata_bpm_covering ON musical_metadata
    USING BTREE(bpm)
    INCLUDE (file_id, key_signature, time_signature_numerator);
```

**No code changes needed!** PostgreSQL automatically uses index-only scans.

---

## Testing & Validation

### Before/After Benchmarking

```bash
# Establish baseline
cargo bench --release --bench import_bench 2>&1 | tee baseline.txt
# Expected: 7,830 files/sec

# Apply fixes
# ... implement optimizations ...

# Re-run benchmark
cargo bench --release --bench import_bench 2>&1 | tee optimized.txt

# Compare
diff baseline.txt optimized.txt
# Look for files/sec improvement
```

### Load Testing

```bash
# Test with 1M files
psql postgresql://user:pass@localhost/db -c "SELECT COUNT(*) FROM files"

# Monitor resources
watch -n 1 'vmstat; echo "---"; iostat -x'

# Check query performance
EXPLAIN ANALYZE SELECT ... ;
```

---

## Implementation Checklist

- [ ] **Priority 1 (4 hours):**
  - [ ] Create specialized query methods (2h)
  - [ ] Add covering indexes (1h)
  - [ ] Optimize full-text search (1h)

- [ ] **Priority 2 (8 hours):**
  - [ ] Tune connection pool (2h)
  - [ ] Implement query caching (3h)
  - [ ] Stream-based analysis (3h)

- [ ] **Testing (2 hours):**
  - [ ] Benchmark baseline
  - [ ] Run 1M file import test
  - [ ] Stress test with 32 concurrent workers
  - [ ] Validate memory usage

- [ ] **Deployment:**
  - [ ] Tag git commit
  - [ ] Deploy to staging
  - [ ] Monitor 24 hours
  - [ ] Deploy to production
  - [ ] Monitor 48 hours

---

## Expected Results After Implementation

### Import Phase
- **Before:** 7,830 files/sec
- **After Priority 1+2:** 9,000-10,000 files/sec (+28%)
- **After Priority 3:** 10,000+ files/sec (+30%)

### Analysis Phase
- **Before:** 181-360 files/sec
- **After Priority 1+2:** 300-400 files/sec (+40%)
- **After Priority 3:** 500-700 files/sec (+3x)

### Search Queries
- **Before:** 100-300ms
- **After Priority 1+2:** 20-50ms (-80%)

### Memory Usage
- **Before:** 32MB per batch
- **After Priority 2:** 3-5MB per batch (-90%)

### Database Connection Stability
- **Before:** 80-90% saturation, timeouts at load
- **After Priority 2:** <60% saturation, stable under load

---

## Quick Rollback Steps

If issues arise:

```bash
# 1. Stop current operations
git stash

# 2. Checkout previous version
git checkout <previous-tag>

# 3. Revert database indexes (if added)
DROP INDEX CONCURRENTLY idx_search_results_covering;
DROP INDEX CONCURRENTLY idx_by_manufacturer_covering;

# 4. Restart application
cargo run --release
```

---

## Key Files to Review

| File | Lines | Purpose |
|------|-------|---------|
| file_repository.rs | 76-181 | Query selectivity fixes |
| search_repository.rs | 31-108 | Full-text search optimization |
| database/mod.rs | 150-173 | Connection pool tuning |
| commands/analyze.rs | 200-250 | Memory optimization |
| core/pipeline/queues.rs | 1-80 | Pipeline architecture (reference) |
| core/pipeline/orchestrator.rs | 195-250 | Worker pool coordination |

---

## Contact & Questions

This analysis was generated by the **Performance Oracle** specialized agent.

For questions about specific optimizations, refer to:
- Detailed analysis: `/home/dojevou/projects/midi-software-center/PERFORMANCE-ANALYSIS-ORACLE.md`
- Implementation examples: This quick reference document
- Code locations: See "Key Files to Review" section above

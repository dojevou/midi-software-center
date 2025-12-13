# Test Optimization Quick Reference

## One-Page Cheat Sheet

### Current Situation
```
Test Execution: 150 seconds (sequential, --test-threads=1 required)
Setup/Teardown: 65-68% overhead (preventable)
High-Churn Files: workflows_test.rs (14 commits), file_import_test.rs (12), search_repository_test.rs (11)
```

### Why Tests Are Slow

| Issue | Impact | Fix |
|-------|--------|-----|
| New pool per test | 200ms × 50 tests = 10s | Share pool with OnceLock |
| Explicit cleanup | 200ms × 50 × 2 = 20s | Use transaction rollback |
| No parallelization | 150s baseline | --test-threads=4 (-90%) |
| Duplicate builders | 400+ lines, slower builds | Extract to tests/fixtures/ |
| High complexity | 0.98/test (target 0.3) | Split test files |

### Three Priorities to Fix

#### PRIORITY 1: Shared Fixture (73% improvement)
```rust
// Before: 150s total
#[tokio::test]
async fn test_xyz() {
    let pool = setup_test_pool().await;  // 200ms
    cleanup_database(&pool).await;        // 100ms
    // ... test ...
    cleanup_database(&pool).await;        // 100ms
}

// After: 35-40s total
use tests::common::run_test_transaction;

#[tokio::test]
async fn test_xyz() {
    run_test_transaction(|pool| async move {
        // ... test ... (no cleanup needed)
    }).await
}
```

**Implementation:** tests/common/mod.rs (OnceLock + transaction begin/rollback)
**Time Saved:** 110s (73%)
**Effort:** 2-3 hours

#### PRIORITY 2: Parallelization (88% improvement with Priority 1)
```rust
// Before
cargo test --lib -- --test-threads=1
# 150s

// After
cargo test --lib -- --test-threads=4
# 8-12s (with shared fixture)

cargo test --lib -- --test-threads=8
# 5-7s (maximum parallelization)
```

**Implementation:** Thread-local transaction wrapper
**Time Saved:** 25-30s additional (after Priority 1)
**Effort:** 4-5 hours
**Total Speedup:** 88% reduction from baseline

#### PRIORITY 3: Code Deduplication (5-10s compile time)
```rust
// Eliminate 400+ lines of duplicated builders
// SearchQueryBuilder (70 lines)
// MidiFileBuilder (100+ lines)
// create_midi_bytes (70+ lines)

// Create: tests/fixtures/builders.rs
// Import: use tests::fixtures::{SearchQueryBuilder, MidiFileBuilder};
```

**Implementation:** tests/fixtures/mod.rs
**Time Saved:** 5-10s compile time
**Effort:** 1-2 hours

### Quick Test Timeline

| Week | Task | Expected Time Saved |
|------|------|-------------------|
| 1 | Shared fixture | 150s → 35-40s |
| 2 | Parallelization | 35s → 8-12s |
| 3 | Code dedup + file split | +5-10s compile, better maintainability |
| 4 | Timing instrumentation | Performance metrics |

### Files to Modify

**Start Here (Week 1):**
- `pipeline/src-tauri/tests/common/mod.rs` - Add OnceLock + transaction helpers

**Then Update (in parallel):**
- `pipeline/src-tauri/tests/search_repository_test.rs` - Use shared fixture
- `pipeline/src-tauri/tests/file_import_test.rs` - Use shared fixture
- `pipeline/src-tauri/tests/workflows_test.rs` - Use shared fixture

**Create New (Week 2):**
- `pipeline/src-tauri/tests/fixtures/mod.rs` - Shared builders
- `pipeline/src-tauri/tests/fixtures/builders.rs` - Duplicate patterns

**Optional (Week 3):**
- Split large test files (2,486 → 4 × 600 lines)
- Add timing instrumentation

### Test Coverage Today

| Category | Count | Pass Rate |
|----------|-------|-----------|
| Total Tests | 1,223+ | 100% |
| High-Churn Files | 5 | 100% |
| Stability Score | 7/10 | Stable, slow |

### Expected Performance After Optimization

```
Timeline:
├─ Before: 150 seconds (sequential)
├─ Priority 1: 35-40s (shared fixture)
├─ Priority 1+2: 8-12s (parallelized)
└─ Priority 1+2+3+4: 5-7s (full optimization)

CI/CD Impact:
├─ Before: 150s test execution
└─ After: 8s test execution (18x faster)
```

### Implementation Checklist

#### Week 1: Shared Fixture
- [ ] Create OnceLock in tests/common/mod.rs
- [ ] Implement transaction begin/rollback helpers
- [ ] Update 5 test files (proof of concept)
- [ ] Verify all tests pass

#### Week 2: Parallelization
- [ ] Implement thread-local transaction wrapper
- [ ] Update all 50+ test files
- [ ] Test with --test-threads=4 and --test-threads=8
- [ ] Verify no test interference

#### Week 3: Code Quality (Optional)
- [ ] Extract builders to tests/fixtures/
- [ ] Split large test files
- [ ] Update test discovery

#### Week 4: Observability (Optional)
- [ ] Add timing instrumentation
- [ ] CI/CD integration
- [ ] Baseline metrics

### Key Metrics

| Metric | Now | Target | Improvement |
|--------|-----|--------|------------|
| Test Execution Time | 150s | 5-8s | 88-97% |
| Parallelization | --test-threads=1 | --test-threads=8 | 7-8x |
| Setup/Teardown Overhead | 65-68% | 10-15% | 78% reduction |
| Code Duplication | 400+ lines | <50 lines | 87% reduction |
| Complexity/Test | 0.87 | 0.3 | 65% reduction |

### Troubleshooting

**Problem: Tests fail after shared fixture implementation**
- Verify transaction rollback is called
- Check database connection credentials
- Ensure OnceLock.get_or_init() is async-safe

**Problem: Parallelization causes test failures**
- Check for shared state in tests
- Verify thread-local transactions are isolated
- Run with --test-threads=2 to debug

**Problem: Builder duplication still exists**
- Search for similar functions in other test files
- Use `grep -r "SearchQueryBuilder"` to find all instances
- Use `grep -r "MidiFileBuilder"` to find all instances

### Full Documentation

See **TEST-IMPACT-ANALYSIS.md** for comprehensive analysis:
- Detailed findings with explanations
- Code examples and implementation guides
- Complete priority recommendations
- Risk assessment and mitigation
- Full implementation timeline

---

**Status:** Ready for Implementation
**Effort:** 10-14 hours total
**Expected ROI:** 10-15x faster test runs (permanent)


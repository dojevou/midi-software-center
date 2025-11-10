# Test Compilation Errors - Action Plan

**Analysis Date**: 2025-11-04
**Total Errors Found**: 429
**Estimated Fix Time**: 67 minutes
**Status**: Ready for immediate action

## Executive Summary

Analysis of 429 test compilation errors across the pipeline project identified:

1. **Single Root Cause (48.7% of errors)**: Tauri State wrapper issue in test calls
2. **Missing Test Infrastructure (21.4% of errors)**: 6 helper functions never created
3. **API Signature Mismatch (17.2% of errors)**: SearchRepository calls missing 2 arguments
4. **Builder Pattern Issues (3.7% of errors)**: SearchQueryBuilder method signatures
5. **Format String Errors (0.7% of errors)**: Positional argument issues

**Key Insight**: Fix the top issue alone (State wrapper) eliminates 209 errors in 10 minutes.

---

## Three-Phase Execution Plan

### PHASE 1: Quick Wins (12 minutes) - HIGHEST PRIORITY

**Objective**: Reduce errors from 429 to ~217 (49% reduction)

#### Step 1.1: Fix State Wrapper in file_import_test.rs (5 min)
```bash
cd /home/dojevou/projects/midi-software-center/

# Show what will be changed
grep -n "tauri::State::from(&state)" pipeline/src-tauri/tests/file_import_test.rs | wc -l
# Expected output: 50 occurrences

# Apply the fix
sed -i 's/tauri::State::from(&state)/state/g' \
  pipeline/src-tauri/tests/file_import_test.rs

# Verify
grep "tauri::State::from(&state)" pipeline/src-tauri/tests/file_import_test.rs | wc -l
# Expected output: 0 (all removed)
```

#### Step 1.2: Fix State Wrapper in workflows_test.rs (5 min)
```bash
# Show what will be changed
grep -n "tauri::State::from(&state)" pipeline/src-tauri/tests/workflows_test.rs | wc -l
# Expected output: 35 occurrences

# Apply the fix
sed -i 's/tauri::State::from(&state)/state/g' \
  pipeline/src-tauri/tests/workflows_test.rs

# Verify
grep "tauri::State::from(&state)" pipeline/src-tauri/tests/workflows_test.rs | wc -l
# Expected output: 0
```

#### Step 1.3: Fix Format Strings in tag_repository_test.rs (2 min)
```bash
# Show what will be changed
grep -n "Expected {[0-9]}" pipeline/src-tauri/tests/tag_repository_test.rs
# Expected output: 3 lines (348, 910, 924)

# Apply fixes
sed -i 's/"Expected {100}, found {count}"/"Expected 100, found {}"/g' \
  pipeline/src-tauri/tests/tag_repository_test.rs
sed -i 's/"Expected {5}, found {count}"/"Expected 5, found {}"/g' \
  pipeline/src-tauri/tests/tag_repository_test.rs
sed -i 's/"Expected {0}, found {count}"/"Expected 0, found {}"/g' \
  pipeline/src-tauri/tests/tag_repository_test.rs

# Verify
grep "Expected {[0-9]}" pipeline/src-tauri/tests/tag_repository_test.rs | wc -l
# Expected output: 0
```

#### Step 1.4: Verify Phase 1 Results (checkpoint)
```bash
# Check error count reduction
cargo check --tests -p midi-pipeline 2>&1 > /tmp/phase1_check.txt
echo "Phase 1 Results:"
grep "^error" /tmp/phase1_check.txt | wc -l
# Expected: ~217 errors remaining (212 errors fixed)

# Breakdown by error type
grep "^error\[E" /tmp/phase1_check.txt | cut -d'[' -f2 | cut -d']' -f1 | sort | uniq -c | sort -rn
```

**Success Criteria**:
- Error count drops to ~217 (from 429)
- No `tauri::State::from(&state)` in test files
- No format string errors about positional arguments

---

### PHASE 2: Medium Fixes (15 minutes) - HIGH PRIORITY

**Objective**: Reduce errors from ~217 to ~143 (34% reduction)
**Target**: SearchRepository argument fixes

#### Step 2.1: Identify all SearchRepository::search() calls (2 min)
```bash
# Find all occurrences that need fixing
grep -rn "SearchRepository::search(&pool, &query)" \
  pipeline/src-tauri/tests/ | grep -v "SearchRepository::search(&pool, &query, " | wc -l
# Expected output: ~60+ occurrences across test files

# Show them by file
echo "=== workflows_test.rs ==="
grep -n "SearchRepository::search(&pool, &query)" \
  pipeline/src-tauri/tests/workflows_test.rs | grep -v ", " | head -10

echo "=== search_repository_test.rs ==="
grep -n "SearchRepository::search(&pool, &query)" \
  pipeline/src-tauri/tests/search_repository_test.rs | grep -v ", " | head -10
```

#### Step 2.2: Add limit/offset to SearchRepository calls (13 min)

**For workflows_test.rs** (37 occurrences):
```bash
# This is more complex - need to handle each occurrence
# Strategy: Add default limit=100, offset=0 to all calls

# First, count them
grep -c "SearchRepository::search(&pool, &query)" \
  pipeline/src-tauri/tests/workflows_test.rs | grep -v ", "

# Replace pattern (works for simple cases):
sed -i 's/SearchRepository::search(&pool, &query)/SearchRepository::search(\&pool, \&query, 100, 0)/g' \
  pipeline/src-tauri/tests/workflows_test.rs

# For more complex cases, you may need manual editing
# Look for lines like:
# let results = SearchRepository::search(&pool, &query).await;
# And change to:
# let results = SearchRepository::search(&pool, &query, 100, 0).await;
```

**For search_repository_test.rs** (24 occurrences):
```bash
sed -i 's/SearchRepository::search(&pool, &query)/SearchRepository::search(\&pool, \&query, 100, 0)/g' \
  pipeline/src-tauri/tests/search_repository_test.rs

# For pagination-specific tests, review and adjust limits as needed
# Look for tests with words like "page", "paginate", "offset" in their names
grep -n "fn test.*page\|fn test.*pagina\|fn test.*offset" \
  pipeline/src-tauri/tests/search_repository_test.rs
```

#### Step 2.3: Verify Phase 2 Results (checkpoint)
```bash
cargo check --tests -p midi-pipeline 2>&1 > /tmp/phase2_check.txt
echo "Phase 2 Results:"
grep "^error" /tmp/phase2_check.txt | wc -l
# Expected: ~143 errors remaining (74 errors fixed)

# Check for remaining E0061 errors
grep "^error\[E0061\]" /tmp/phase2_check.txt | wc -l
# Expected: ~0 (all fixed)
```

**Success Criteria**:
- Error count drops to ~143 (from 217)
- No more E0061 "takes 4 arguments but 2" errors
- SearchRepository::search() calls have 4 arguments

---

### PHASE 3: Complex Fixes (40 minutes) - MEDIUM PRIORITY

**Objective**: Reduce errors from ~143 to 0 (100% completion)
**Target**: Create missing helpers and fix builder methods

#### Step 3.1: Create Missing Test Helper Functions (30 min)

**Create file**: `pipeline/src-tauri/tests/common/helpers.rs`

```rust
// Copy the full helper implementations from TEST-COMPILATION-ERRORS-ANALYSIS.md
// Section: FIX #4: Create missing test helper functions

// Key functions to implement:
// 1. get_file_count_impl(pool: &PgPool) -> i64
// 2. get_file_tags_impl(pool: &PgPool, file_id: i32) -> Vec<String>
// 3. get_all_tags_impl(pool: &PgPool) -> Vec<String>
// 4. get_file_details_impl(pool: &PgPool, file_id: i32) -> (String, String, i64)
// 5. create_test_file(pool: &PgPool, filename: &str) -> i32
// 6. insert_metadata(pool: &PgPool, file_id: i32, bpm, key, duration) -> ()
```

**Update**: `pipeline/src-tauri/tests/common/mod.rs`

Add to imports:
```rust
mod helpers;
pub use helpers::*;
```

#### Step 3.2: Fix SearchQueryBuilder Method Calls (10 min)

**File**: `pipeline/src-tauri/tests/search_repository_test.rs`

**Location 1 - Lines 1453-1454** (min_bpm/max_bpm type mismatch):
```rust
// BEFORE:
.min_bpm(Some("150.0".to_string()))
.max_bpm(Some("100.0".to_string()))

// AFTER:
.min_bpm(150.0)
.max_bpm(100.0)
```

**Location 2 - Line 1920** (offset() method doesn't exist):
```rust
// BEFORE:
let query = SearchQueryBuilder::new()
    .offset(10)
    .limit(10)

// AFTER:
let query = SearchQueryBuilder::new();
// OR pass limit/offset directly to search():
let results = SearchRepository::search(&pool, &query, limit, offset).await;
```

#### Step 3.3: Verify Phase 3 Results (Final checkpoint)
```bash
cargo check --tests -p midi-pipeline 2>&1 > /tmp/phase3_check.txt
echo "Phase 3 Results:"
grep "^error" /tmp/phase3_check.txt | wc -l
# Expected: 0 errors

# Detailed breakdown
echo "=== Error Summary ==="
if [ $(grep "^error" /tmp/phase3_check.txt | wc -l) -eq 0 ]; then
  echo "SUCCESS: All test compilation errors fixed!"
else
  grep "^error\[E" /tmp/phase3_check.txt | cut -d'[' -f2 | cut -d']' -f1 | sort | uniq -c
fi
```

**Success Criteria**:
- Error count is 0
- `cargo check --tests -p midi-pipeline` completes successfully
- All helper functions properly exported and callable

---

## Verification Checkpoints

### After Phase 1
```bash
cargo check --tests -p midi-pipeline 2>&1 | grep "^error" | wc -l
# Target: 217 errors (212 fixed)
```

### After Phase 2
```bash
cargo check --tests -p midi-pipeline 2>&1 | grep "^error" | wc -l
# Target: 143 errors (74 more fixed)
```

### After Phase 3
```bash
cargo check --tests -p midi-pipeline 2>&1 | grep "^error" | wc -l
# Target: 0 errors (all 143 fixed)
```

### Final Full Test
```bash
cargo test --lib -p midi-pipeline --no-fail-fast 2>&1 | tail -20
# Should show: test result: ok. XX passed
```

---

## Rollback Plan (if needed)

Each fix can be rolled back using git:

```bash
# After Phase 1, if tests don't improve:
git diff pipeline/src-tauri/tests/file_import_test.rs
git checkout pipeline/src-tauri/tests/file_import_test.rs
git checkout pipeline/src-tauri/tests/workflows_test.rs
git checkout pipeline/src-tauri/tests/tag_repository_test.rs

# After Phase 2:
git checkout pipeline/src-tauri/tests/workflows_test.rs
git checkout pipeline/src-tauri/tests/search_repository_test.rs

# After Phase 3:
git checkout pipeline/src-tauri/tests/common/
```

---

## Expected Timeline

| Phase | Task | Est. Time | Actual | Status |
|-------|------|-----------|--------|--------|
| 1 | Fix State wrapper | 5 min | | TODO |
| 1 | Fix format strings | 2 min | | TODO |
| 1 | Verify & checkpoint | 5 min | | TODO |
| 2 | Add limit/offset | 15 min | | TODO |
| 2 | Verify & checkpoint | 3 min | | TODO |
| 3 | Create helpers | 30 min | | TODO |
| 3 | Fix builder methods | 10 min | | TODO |
| 3 | Final verification | 2 min | | TODO |
| | **TOTAL** | **~67 min** | | |

---

## How to Run This Plan

### Option A: Manual (Recommended for first run)
1. Execute Phase 1 steps manually
2. Verify results with cargo check
3. Execute Phase 2 steps
4. Verify results
5. Execute Phase 3 steps
6. Verify final results

### Option B: Automated (After manual verification)
Save the commands below to a script and run:

```bash
#!/bin/bash
set -e

cd /home/dojevou/projects/midi-software-center/

echo "=== Phase 1: Quick Wins ==="
sed -i 's/tauri::State::from(&state)/state/g' \
  pipeline/src-tauri/tests/file_import_test.rs \
  pipeline/src-tauri/tests/workflows_test.rs

sed -i 's/"Expected {100}, found {count}"/"Expected 100, found {}"/g' \
  pipeline/src-tauri/tests/tag_repository_test.rs
sed -i 's/"Expected {5}, found {count}"/"Expected 5, found {}"/g' \
  pipeline/src-tauri/tests/tag_repository_test.rs
sed -i 's/"Expected {0}, found {count}"/"Expected 0, found {}"/g' \
  pipeline/src-tauri/tests/tag_repository_test.rs

echo "Phase 1 done. Checking progress..."
cargo check --tests -p midi-pipeline 2>&1 | grep "^error" | wc -l

echo "=== Phase 2: SearchRepository fixes ==="
sed -i 's/SearchRepository::search(&pool, &query)/SearchRepository::search(\&pool, \&query, 100, 0)/g' \
  pipeline/src-tauri/tests/workflows_test.rs \
  pipeline/src-tauri/tests/search_repository_test.rs

echo "Phase 2 done. Checking progress..."
cargo check --tests -p midi-pipeline 2>&1 | grep "^error" | wc -l

echo "=== Phase 3: Helper functions and builder fixes ==="
echo "WARNING: Phase 3 requires manual implementation of helpers.rs"
echo "See TEST-COMPILATION-ERRORS-ANALYSIS.md for code templates"
```

---

## Key Documents

- **TEST-ERRORS-QUICK-REFERENCE.md** - One-page reference guide
- **TEST-COMPILATION-ERRORS-ANALYSIS.md** - Full technical analysis with code examples
- **TEST-ERRORS-ACTION-PLAN.md** - This document with step-by-step instructions

---

## Success Definition

- [ ] cargo check --tests -p midi-pipeline completes with 0 compilation errors
- [ ] All test helper functions are properly exported
- [ ] SearchRepository::search() calls include all 4 required arguments
- [ ] SearchQueryBuilder method calls use correct types
- [ ] No warnings about format string arguments
- [ ] All changes are committed with clear git message

---

## Questions or Issues?

If any step doesn't work as expected:

1. Check the detailed analysis in TEST-COMPILATION-ERRORS-ANALYSIS.md
2. Run `cargo check --tests -p midi-pipeline 2>&1` to see current errors
3. Compare with expected errors for that phase
4. Verify sed replacements worked: `grep "pattern" file.rs | wc -l`
5. Consider rolling back that phase and trying manual edits instead


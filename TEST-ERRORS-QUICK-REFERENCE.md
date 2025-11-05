# Test Compilation Errors - Quick Reference

## Current Status
- **Total Errors**: 429
- **Top 3 Files**: file_import_test.rs (152), workflows_test.rs (134), search_repository_test.rs (101)
- **Status**: Fixable in ~67 minutes with 5 targeted fixes

## Error Distribution

```
E0308 (Type mismatch):        209 errors (48.7%)  - Fixable in 10 min
E0425 (Cannot find func):      92 errors (21.4%)  - Fixable in 30 min
E0061 (Wrong arg count):       74 errors (17.2%)  - Fixable in 15 min
E0599 (No method found):       37 errors (8.6%)   - Fixable in 10 min
Other errors:                  17 errors (4.1%)   - Fixable in 2 min
```

## Top 5 Fixes (in execution order)

### Fix #1: Remove tauri::State::from() wrapper [CRITICAL]
- **Impact**: 209 errors (48.7%)
- **Time**: 10 minutes
- **Pattern**: `tauri::State::from(&state)` → `state`
- **Files**: file_import_test.rs (~50 lines), workflows_test.rs (~35 lines)
- **Automatable**: YES (find-replace)

### Fix #2: Fix format strings [QUICK]
- **Impact**: 3 errors (0.7%)
- **Time**: 2 minutes
- **Pattern**: `"Expected {100}"` → `"Expected 100"`
- **File**: tag_repository_test.rs (lines 348, 910, 924)
- **Automatable**: YES (find-replace)

### Fix #3: Add limit/offset to SearchRepository::search() [HIGH]
- **Impact**: 74 errors (17.2%)
- **Time**: 15 minutes
- **Pattern**: `.search(&pool, &query)` → `.search(&pool, &query, 100, 0)`
- **Files**: workflows_test.rs (~37), search_repository_test.rs (~24)
- **Automatable**: PARTIAL (needs context for pagination tests)

### Fix #4: Create missing test helper functions [COMPLEX]
- **Impact**: 92 errors (21.4%)
- **Time**: 30 minutes
- **Functions needed**:
  - get_file_count_impl()
  - get_file_tags_impl()
  - get_all_tags_impl()
  - get_file_details_impl()
  - create_test_file()
  - insert_metadata()
- **Location**: tests/common/helpers.rs
- **Automatable**: NO (requires database knowledge)

### Fix #5: Fix SearchQueryBuilder method signatures [MEDIUM]
- **Impact**: 16 errors (3.7%)
- **Time**: 10 minutes
- **Problems**:
  - Lines 1453-1454: min_bpm/max_bpm expect f64, not Option<String>
  - Line 1920: .offset(10) method doesn't exist
- **File**: search_repository_test.rs
- **Automatable**: NO (API understanding needed)

## Execution Strategy

### Phase 1: Quick Wins (12 minutes)
1. Run: Find-replace for `tauri::State::from(&state)` → `state`
2. Run: Fix 3 format strings in tag_repository_test.rs
3. Check: `cargo check --tests 2>&1 | grep "^error" | wc -l` (should be ~217)

### Phase 2: Medium Fixes (15 minutes)
1. Add limit/offset to all SearchRepository::search() calls
2. Check: `cargo check --tests 2>&1 | grep "^error" | wc -l` (should be ~143)

### Phase 3: Complex Fixes (40 minutes)
1. Create helper functions in tests/common/helpers.rs
2. Fix SearchQueryBuilder method calls
3. Check: `cargo check --tests 2>&1 | grep "^error" | wc -l` (should be 0)

## One-Liner Fixes

```bash
# Fix #1: State wrapper (file_import_test.rs)
sed -i 's/tauri::State::from(&state)/state/g' \
  pipeline/src-tauri/tests/file_import_test.rs

# Fix #1: State wrapper (workflows_test.rs)
sed -i 's/tauri::State::from(&state)/state/g' \
  pipeline/src-tauri/tests/workflows_test.rs

# Fix #2: Format strings
sed -i 's/"Expected {100}, found {count}"/"Expected 100, found {}"/g' \
  pipeline/src-tauri/tests/tag_repository_test.rs
sed -i 's/"Expected {5}, found {count}"/"Expected 5, found {}"/g' \
  pipeline/src-tauri/tests/tag_repository_test.rs
sed -i 's/"Expected {0}, found {count}"/"Expected 0, found {}"/g' \
  pipeline/src-tauri/tests/tag_repository_test.rs

# Check progress
cargo check --tests -p midi-pipeline 2>&1 | grep "^error" | wc -l
```

## High-Impact Takeaways

1. **209 errors (48.7%) from ONE issue**: Removing State wrapper fixes nearly half
2. **92 errors (21.4%) from missing 6 functions**: Creating helpers eliminates another 21%
3. **74 errors (17.2%) from missing 2 arguments**: Adding limit/offset fixes another 17%
4. **Cascading effect**: Fix #1 might eliminate additional errors as test files compile further
5. **No source code changes needed**: All fixes are in test files only

## File-by-File Impact

| File | Errors | Quick Fixes | Remaining After Fixes |
|------|--------|-------------|----------------------|
| file_import_test.rs | 152 | Fix #1 (147 errors) | 5 errors |
| workflows_test.rs | 134 | Fix #1 + #3 (84 errors) | 50 errors |
| search_repository_test.rs | 101 | Fix #2 + #3 + #4 (82 errors) | 19 errors |
| tag_repository_test.rs | 30 | Fix #5 (3 errors) | 27 errors |
| **Total** | **429** | **316 errors** | **101 errors** |

Note: Numbers are estimates; actual totals may vary due to cascading effects.

## See Full Details In

- **TEST-COMPILATION-ERRORS-ANALYSIS.md** - Complete error breakdown and analysis
- **TEST-COMPILATION-ERRORS-ANALYSIS.md** (continued) - Detailed fix guide with line numbers

## Command Reference

```bash
# View all compilation errors
cargo check --tests -p midi-pipeline 2>&1 | grep "^error"

# Count errors by type
cargo check --tests -p midi-pipeline 2>&1 | grep "^error\[E" | cut -d'[' -f2 | cut -d']' -f1 | sort | uniq -c

# Find State wrapper issues
grep -rn "tauri::State::from(&" pipeline/src-tauri/tests/

# Find missing SearchRepository arguments
grep -n "SearchRepository::search(&pool, &query)" pipeline/src-tauri/tests/*.rs

# Find format string errors
grep -rn 'Expected {[0-9]}' pipeline/src-tauri/tests/

# Final verification
cargo check --tests -p midi-pipeline 2>&1 | grep -E "^error" | wc -l
# Should output: 0
```


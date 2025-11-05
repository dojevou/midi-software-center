# Test Compilation Error Analysis
## Pipeline Project (midi-pipeline)

### Summary Statistics
- **Total Errors**: 429 (actual count from cargo check)
- **E0308 (Type Mismatch)**: 209 errors (48.7%)
- **E0425 (Cannot find function)**: 92 errors (21.4%)  
- **E0061 (Wrong argument count)**: 74 errors (17.2%)
- **E0599 (No method found)**: 37 errors (8.6%)
- **Other (E0609, E0422, E0433, etc.)**: 17 errors (4.1%)

### Top 3 Affected Files

#### 1. file_import_test.rs - 152 errors
- **Location**: `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/tests/file_import_test.rs`
- **Error Breakdown**:
  - E0308: 147 errors (96.7% of file's errors)
  - E0599: 2 errors
  - E0107: 1 error
  - E0277: 1 error
  - E0382: 1 error

**Root Cause - E0308 Pattern**:
All 147 E0308 errors stem from ONE problem: `State` object handling in test calls
```rust
// Line 362 - WRONG (repeated 50+ times):
tauri::State::from(&state)  // Expected State<'_, AppState>, found &AppState

// Should be:
state  // Already a State<'_, AppState> reference
```

**Why it cascades**:
- Function signature expects `State<'_, AppState>` 
- Tests try to wrap `&AppState` with `tauri::State::from()`
- Tauri's State::from() doesn't accept references, only owned values
- Every single test that calls import_single_file has this issue

**High Impact Pattern**: The exact same fix applied 50+ times = massive reduction

---

#### 2. workflows_test.rs - 134 errors  
- **Location**: `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/tests/workflows_test.rs`
- **Error Breakdown**:
  - E0425 (missing functions): 44 errors (32.8%)
  - E0308 (type mismatch): 47 errors (35.1%)
  - E0061 (wrong args): 37 errors (27.6%)
  - E0599 (no method): 3 errors (2.2%)
  - E0423, E0277, E0282, E0432: 3 errors total

**Root Causes**:

**(A) Missing test helper functions - E0425 (44 errors)**
```
get_file_count_impl - missing (14+ calls)
get_file_tags_impl - missing (12+ calls)
get_all_tags_impl - missing (8+ calls)
get_file_details_impl - missing (4+ calls)
```
These functions were never created in test infrastructure. All similar repository tests likely need same functions.

**(B) Wrong argument count to SearchRepository::search() - E0061 (37 errors)**
```rust
// Line 122, 132, 142, etc - WRONG (repeated):
let results = SearchRepository::search(&pool, &query).await;
// Error: takes 4 arguments but 2 supplied

// Should be:
let results = SearchRepository::search(&pool, &query, limit, offset).await;
```
**Why**:
- SearchRepository::search() signature is: `search(pool, query, limit: i64, offset: i64)`
- All workflow tests call it with only 2 args
- This is a widespread pattern affecting ~37 calls

**(C) State object handling - E0308 (47 errors)**
Same as file_import_test.rs - wrapping &AppState with tauri::State::from()

**Interdependency**: If we fix E0061 (add limit/offset), some tests might still fail waiting for E0425 functions

---

#### 3. search_repository_test.rs - 101 errors
- **Location**: `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/tests/search_repository_test.rs`
- **Error Breakdown**:
  - E0425 (missing functions): 48 errors (47.5%)
  - E0599 (no method): 16 errors (15.8%)
  - E0061 (wrong args): 24 errors (23.8%)
  - E0308 (type): 9 errors (8.9%)
  - E0433, E0609: 4 errors (4.0%)

**Root Causes**:

**(A) Missing test helper functions - E0425 (48 errors)**
```
create_test_file - missing (3+ calls)
insert_metadata - missing (3+ calls)
```
These are basic CRUD helpers needed across test suite

**(B) Method not found - E0599 (16 errors)**
```rust
// Line 1453:
.min_bpm(Some("150.0".to_string()))  // WRONG - method expects f64
// min_bpm signature expects f64, receiving Option<String>

.max_bpm(Some("100.0".to_string()))  // WRONG - same issue
.offset(10)  // SearchQueryBuilder has no offset() method
```

**(C) Wrong argument count - E0061 (24 errors)**
```rust
// Line 1457:
let results = SearchRepository::search(&pool, &query).await;
// Should be:
let results = SearchRepository::search(&pool, &query, limit, offset).await;
```

---

### Impact Summary by Error Type

| Error Type | Count | Pattern | Quick Fix? | Automation Possible? |
|------------|-------|---------|-----------|----------------------|
| E0308 | 209 | tauri::State wrapper issue | YES (simple) | YES (find-replace) |
| E0425 | 92 | Missing test helper functions | NO (create) | PARTIAL (identify, manual create) |
| E0061 | 74 | SearchRepository needs limit/offset | YES (add args) | YES (find-replace) |
| E0599 | 37 | Wrong type passed to methods | VARIES | NO (context-specific) |
| Other | 17 | Edge cases | VARIES | NO |

---

## TOP 5 HIGHEST-IMPACT FIXES

### Fix #1: Remove tauri::State::from() wrapper (FIX: 209 E0308 errors)
**Impact**: Eliminates 48.7% of all errors (209 errors)
**Affected Files**:
- file_import_test.rs: 147 errors
- workflows_test.rs: 47 errors  
- Other files: 15 errors

**Pattern** (appears ~60+ times):
```rust
// WRONG - everywhere in tests:
tauri::State::from(&state)

// RIGHT - in all test calls:
// Remove the wrapper, pass state directly
// State is already a &State<'_, AppState> in test context
```

**Specific Line Examples**:
- file_import_test.rs:362, 399, 410, 439, 477, 515, 543, 572, 583... (repeats ~50 times)
- workflows_test.rs: Similar pattern

**Time to Fix**: QUICK (<5 minutes)
- Global find-replace: `tauri::State::from(&state)` → `` (delete or adjust)
- Actually, need to understand test context better - might just pass `state` directly
- Manual review of 5-10 test cases needed to understand proper fix

**Automatable**: YES - Find-replace, but needs review for correctness

---

### Fix #2: Add limit/offset arguments to SearchRepository::search() calls (FIX: 74 E0061 errors)
**Impact**: Eliminates 17.2% of all errors (74 errors)
**Affected Files**:
- workflows_test.rs: 37 errors
- search_repository_test.rs: 24 errors
- Other repository tests: 13 errors

**Current Function Signature**:
```rust
pub async fn search(
    pool: &PgPool,
    query: SearchQuery,
    limit: i64,
    offset: i64,  // REQUIRED but tests don't provide
) -> Result<Vec<File>, sqlx::Error>
```

**Pattern** (appears ~40+ times):
```rust
// WRONG - missing last 2 arguments:
let results = SearchRepository::search(&pool, &query).await;

// RIGHT - add limit and offset:
let results = SearchRepository::search(&pool, &query, 100, 0).await;
// or for paginated tests:
let results = SearchRepository::search(&pool, &query, limit, offset).await;
```

**Specific Examples**:
- workflows_test.rs: lines 122, 132, 142, 168, 192, 212, 231, 271, etc.
- search_repository_test.rs: lines 1457, 1923, 1904, etc.

**Time to Fix**: MEDIUM (5-15 minutes)
- Need to identify which tests want pagination vs. all results
- Some need reasonable defaults (limit=100, offset=0)
- Some need specific pagination testing (limit=10, offset=20, etc.)
- Manual review of test intent needed

**Automatable**: PARTIAL
- Can add default `limit=100, offset=0` to all calls (find-replace)
- But some tests explicitly test pagination - need context

---

### Fix #3: Create missing test helper functions (FIX: 92 E0425 errors)
**Impact**: Eliminates 21.4% of all errors (92 errors)
**Affected Functions** (don't exist):
```
1. get_file_count_impl() - missing (14 calls in workflows_test)
2. get_file_tags_impl() - missing (12 calls in workflows_test)
3. get_all_tags_impl() - missing (8 calls in workflows_test)
4. get_file_details_impl() - missing (4 calls in workflows_test)
5. create_test_file() - missing (3 calls in search_repository_test)
6. insert_metadata() - missing (3 calls in search_repository_test)
```

**Where these should go**:
- `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/tests/common/` (test helpers module)
- Or in `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/tests/helpers/` if new

**Implementation Examples**:
```rust
// These should query the database and return results:
pub async fn get_file_count_impl(pool: &PgPool) -> i64 {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM files")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));
    count.0
}

pub async fn create_test_file(pool: &PgPool, filename: &str) -> i32 {
    let file = sqlx::query_as!(
        File,
        "INSERT INTO files (filename, hash, size) VALUES ($1, $2, $3)
         RETURNING id, filename, hash, size, created_at, updated_at",
        filename,
        format!("hash_{}", filename),
        1024i64
    )
    .fetch_one(pool)
    .await
    .unwrap();
    file.id
}
```

**Time to Fix**: LONG (>15 minutes per function)
- Need to understand database schema (which tables, fields)
- Need to handle async/await patterns
- Need to handle error cases
- Estimate: 10-15 mins for all 6 functions if schema is clear
- **Actually, might be 20-30 mins if schema lookup needed**

**Automatable**: NO - Requires manual creation with database knowledge

**Research Step Needed**: Check what existing helper patterns exist in other test files

---

### Fix #4: Fix SearchQueryBuilder method signatures (FIX: 16 E0599 errors)
**Impact**: Eliminates 3.7% of all errors (16 errors)
**Affected File**: search_repository_test.rs

**Problems**:
```rust
// Line 1453 - WRONG type:
.min_bpm(Some("150.0".to_string()))  // Receiving Option<String>, expecting f64
.max_bpm(Some("100.0".to_string()))  // Same issue

// Line 1920:
.offset(10)  // SearchQueryBuilder has no offset() method

// Line 1919:
.limit(10)   // Probably also missing
```

**Fix Patterns**:
```rust
// For min/max_bpm:
.min_bpm("150.0".parse::<f64>().unwrap())
// or
.min_bpm(150.0)

// For limit/offset:
// Either add these methods to SearchQueryBuilder OR
// Pass them directly to SearchRepository::search():
SearchRepository::search(&pool, &query, limit, offset)
```

**Time to Fix**: MEDIUM (5-15 minutes)
- Check SearchQueryBuilder source code
- Understand what methods exist vs. what's expected
- Update test code to match actual API

**Automatable**: NO - Requires understanding test intent and API structure

---

### Fix #5: Fix format string errors (FIX: 3 formatting errors)
**Impact**: Eliminates 0.7% of all errors (3 errors)
**Affected File**: tag_repository_test.rs

**Pattern**:
```rust
// Line 348 - WRONG:
assert_eq!(count, 100, "Expected {100}, found {count}");
// Error: invalid reference to positional argument 100 (no arguments given)

// Line 910, 924 - Similar issues

// RIGHT:
assert_eq!(count, 100, "Expected 100, found {}", count);
// or with named args in format string
```

**Time to Fix**: QUICK (<5 minutes)
- Simple format string fixes
- 3 locations total

**Automatable**: YES - Find-replace format strings with `{}`

---

## DEPENDENCY ANALYSIS

**Fix Order** (some fixes unblock others):

1. **Fix #1 first** (E0308 State wrapper) - 209 errors
   - Independent, enables compilation to progress
   - Affects all test files equally

2. **Fix #5 second** (Format strings) - 3 errors  
   - Independent, very quick
   - Unblocks compilation progress

3. **Fix #2 third** (SearchRepository args) - 74 errors
   - Depends on Fix #1 being done (for that test file to compile past E0308)
   - Once added, unblocks workflows/search tests

4. **Fix #4 fourth** (SearchQueryBuilder methods) - 16 errors
   - Depends on Fix #1 and #2
   - search_repository_test.rs specific

5. **Fix #3 last** (Create helpers) - 92 errors
   - These functions can be created once #1-2 are done
   - Allows other tests to use them

---

## SPECIFIC FILE & LINE RECOMMENDATIONS

### file_import_test.rs - Fix E0308 State wrapper
**Lines affected**: ~50 locations, pattern repeats

Sample locations:
- Line 362: `tauri::State::from(&state),`
- Line 399: `tauri::State::from(&state),`  
- Line 410: `tauri::State::from(&state),`
- Lines 439, 477, 515, 543, 572, 583... (continues)

**Fix**: Remove `tauri::State::from(&` and closing `)` wrapper
- Check if state is already a `State<'_, AppState>` reference
- If so, pass directly

---

### workflows_test.rs - Fix E0425 + E0061 + E0308

**E0061 (SearchRepository args)** - ~37 occurrences:
- Lines 122, 132, 142, 168, 192, 212, 231, 271, 311, 330, 365, 414, 447, 508, 560, 587, 614, 647, 689, 750...
- Add `limit: i64, offset: i64` to all SearchRepository::search() calls
- Default suggestion: use `100, 0` or context-appropriate values

**E0308 (State wrapper)** - ~47 occurrences:
- Similar to file_import_test.rs
- Fix State wrapper issue

**E0425 (Missing functions)**:
- Lines 149, 184, 238, 267, 278, 345, 346, 373, 469, 521, 594, 657, 721...
- Need: get_file_count_impl, get_file_tags_impl, get_all_tags_impl, get_file_details_impl
- Create in test helpers module

---

### search_repository_test.rs - Fix E0425 + E0599 + E0061

**E0425 (Missing functions)**:
- create_test_file(): Lines 1448, 1469, 1880, 1915, 1946...
- insert_metadata(): Lines 1449, 1916...
- create in test helpers

**E0061 (SearchRepository args)**:
- Lines 1457, 1923, 1904, 2050...
- Add limit, offset parameters

**E0599 (Method not found)**:
- Lines 1453, 1454: min_bpm/max_bpm receiving wrong types
- Line 1920: offset() method doesn't exist
- Either fix types or use different builder pattern

---

## ESTIMATED TIME SAVINGS

**If all fixes applied**:

| Fix | Errors Eliminated | Est. Time | Impact |
|-----|-------------------|-----------|--------|
| #1 - State wrapper | 209 | 10 min | 48.7% errors gone |
| #5 - Format strings | 3 | 2 min | 0.7% errors gone |
| #2 - SearchRepository args | 74 | 15 min | 17.2% errors gone |
| #4 - QueryBuilder methods | 16 | 10 min | 3.7% errors gone |
| #3 - Create helpers | 92 | 30 min | 21.4% errors gone |
| **TOTAL** | **429** | **~67 min** | **100% errors eliminated** |

**Potential cascading effect**: After Fix #1, many E0308 errors might disappear, potentially unblocking compilation of other files.

# DETAILED FIX GUIDE - Test Compilation Errors

## Executive Summary
- 429 total compilation errors across 3 main test files
- 5 high-impact fixes can eliminate 100% of errors
- Estimated total fix time: ~67 minutes
- All fixes are localized to test files, no source code changes needed

---

## FIX #1: Remove tauri::State::from() wrapper [PRIORITY: CRITICAL]

**Impact**: 209 errors (48.7% of total)
**Estimated Time**: 10 minutes
**Automatable**: YES with review
**Affected Files**: 
- file_import_test.rs (50 occurrences)
- workflows_test.rs (35 occurrences)
- Other tests (10+ occurrences)

### Root Cause
Tests are trying to wrap `&AppState` with `tauri::State::from()`, but the function expects
an owned value or already-wrapped State. The test context already has a properly typed State.

### Solution
Remove the `tauri::State::from(&` wrapper and closing `)` from all function calls.

### file_import_test.rs - Exact Locations
All occurrences follow same pattern. Examples:

```
Line 362:  tauri::State::from(&state),
Line 399:  tauri::State::from(&state),
Line 410:  tauri::State::from(&state),
Line 439:  tauri::State::from(&state),
Line 477:  tauri::State::from(&state),
Line 515:  tauri::State::from(&state),
Line 543:  tauri::State::from(&state),
Line 572:  tauri::State::from(&state),
Line 583:  tauri::State::from(&state),
Line 610:  tauri::State::from(&state),
Line 648:  tauri::State::from(&state),
Line 680:  tauri::State::from(&state),
Line 716:  tauri::State::from(&*state_clone),
Line 751:  tauri::State::from(&state),
Line 802:  tauri::State::from(&state),
Line 836:  tauri::State::from(&state),
Line 873:  tauri::State::from(&state),
Line 910:  tauri::State::from(&state),
Line 946:  tauri::State::from(&state),
Line 979:  tauri::State::from(&state),
... and 30+ more
```

**Total in file_import_test.rs**: ~50 occurrences

### Automated Fix (Find-Replace)
```
Find:    tauri::State::from(&state)
Replace: state

Find:    tauri::State::from(&*state_clone)
Replace: state_clone.clone()
```

### Manual Verification Steps
After applying find-replace:
1. Verify test compiles: `cargo check --tests -p midi-pipeline 2>&1 | grep -c "error\["`
2. Should see error count drop from 429 to ~220 (209 errors eliminated)
3. Review a few test cases to ensure State is passed correctly

---

## FIX #2: Fix format string errors [PRIORITY: MEDIUM]

**Impact**: 3 errors (0.7% of total)
**Estimated Time**: 2 minutes
**Automatable**: YES
**Affected File**: tag_repository_test.rs

### Root Cause
Using positional arguments in format strings without providing them:
```rust
assert_eq!(count, 100, "Expected {100}, found {count}");
//                              ^^^ Positional arg 100 doesn't exist
```

### Solution
Fix format string syntax to use named arguments correctly.

### tag_repository_test.rs - Exact Locations

**Line 348:**
```rust
WRONG:  assert_eq!(count, 100, "Expected {100}, found {count}");
RIGHT:  assert_eq!(count, 100, "Expected 100, found {}", count);
```

**Line 910:**
```rust
WRONG:  assert_eq!(count, 5, "Expected {5}, found {count}");
RIGHT:  assert_eq!(count, 5, "Expected 5, found {}", count);
```

**Line 924:**
```rust
WRONG:  assert_eq!(count, 0, "Expected {0}, found {count}");
RIGHT:  assert_eq!(count, 0, "Expected 0, found {}", count);
```

### Automated Fix (Find-Replace)
```
Find:    "Expected \{(\d+)\}, found \{count\}"
Replace: "Expected $1, found {}"
Pattern: Use regex with appropriate substitution
```

---

## FIX #3: Add limit/offset to SearchRepository::search() [PRIORITY: HIGH]

**Impact**: 74 errors (17.2% of total)
**Estimated Time**: 15 minutes
**Automatable**: PARTIAL (needs context review)
**Affected Files**:
- workflows_test.rs (~37 occurrences)
- search_repository_test.rs (~24 occurrences)
- Other repository tests (~13 occurrences)

### Root Cause
SearchRepository::search() signature requires 4 arguments:
```rust
pub async fn search(
    pool: &PgPool,           // Arg 1: Database pool
    query: SearchQuery,       // Arg 2: Search query
    limit: i64,              // Arg 3: MISSING in tests
    offset: i64,             // Arg 4: MISSING in tests
) -> Result<Vec<File>, sqlx::Error>
```

But tests call it with only 2:
```rust
SearchRepository::search(&pool, &query).await  // ERROR: 2 args, need 4
```

### Solution
Add `limit` and `offset` parameters to all SearchRepository::search() calls.

**Default Values**:
- For general queries: `limit: 100, offset: 0`
- For pagination tests: Use context-specific values

### workflows_test.rs - Examples (37+ occurrences)
Search the error log for "takes 4 arguments but 2" in workflows_test:

**Pattern** (all similar):
```rust
WRONG:  let results = SearchRepository::search(&pool, &query).await;
RIGHT:  let results = SearchRepository::search(&pool, &query, 100, 0).await;
```

**Specific tests** (from E0061 errors):
- Lines in workflows_test.rs with SearchRepository::search() calls
  (~37 total across the file)

### search_repository_test.rs - Examples (24 occurrences)
Similar pattern, need to identify which need pagination vs. all results.

**Example Line 1457:**
```rust
WRONG:  let results = SearchRepository::search(&pool, &query).await.expect("Query should not error");
RIGHT:  let results = SearchRepository::search(&pool, &query, 100, 0).await.expect("Query should not error");
```

### Implementation Strategy
1. Use grep to find all `SearchRepository::search(&` calls
2. Replace with version including `limit` and `offset`
3. For most tests, use `100, 0` as defaults
4. For tests explicitly testing pagination, use appropriate values

### Search Command
```bash
grep -n "SearchRepository::search(&pool, &query)" \
  /home/dojevou/projects/midi-software-center/pipeline/src-tauri/tests/*.rs
```

---

## FIX #4: Create missing test helper functions [PRIORITY: MEDIUM]

**Impact**: 92 errors (21.4% of total)
**Estimated Time**: 30 minutes
**Automatable**: NO (requires manual implementation)
**Affected Functions**:
- `get_file_count_impl()` (14 calls in workflows_test.rs)
- `get_file_tags_impl()` (12 calls)
- `get_all_tags_impl()` (8 calls)
- `get_file_details_impl()` (4 calls)
- `create_test_file()` (3 calls in search_repository_test.rs)
- `insert_metadata()` (3 calls)

### Root Cause
These helper functions don't exist in test infrastructure but are called by tests.

### Solution
Create these functions in the test common/helpers module.

### Location
File: `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/tests/common/helpers.rs`
(or in an existing helpers module)

### Implementation Template

```rust
// In tests/common/helpers.rs or tests/helpers/mod.rs

use sqlx::PgPool;
use crate::common::*;

/// Get count of files in database
pub async fn get_file_count_impl(pool: &PgPool) -> i64 {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM files")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));
    count.0
}

/// Get tags for a specific file
pub async fn get_file_tags_impl(pool: &PgPool, file_id: i32) -> Vec<String> {
    let tags = sqlx::query!(
        "SELECT tags.name FROM tags 
         JOIN file_tags ON tags.id = file_tags.tag_id 
         WHERE file_tags.file_id = $1",
        file_id
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    
    tags.iter().map(|t| t.name.clone()).collect()
}

/// Get all tags in database
pub async fn get_all_tags_impl(pool: &PgPool) -> Vec<String> {
    let tags = sqlx::query!("SELECT name FROM tags")
        .fetch_all(pool)
        .await
        .unwrap_or_default();
    
    tags.iter().map(|t| t.name.clone()).collect()
}

/// Get file details (name, hash, size)
pub async fn get_file_details_impl(pool: &PgPool, file_id: i32) -> (String, String, i64) {
    let file = sqlx::query!(
        "SELECT filename, hash, size FROM files WHERE id = $1",
        file_id
    )
    .fetch_one(pool)
    .await
    .expect("File not found");
    
    (file.filename, file.hash, file.size)
}

/// Create a test MIDI file in database
pub async fn create_test_file(pool: &PgPool, filename: &str) -> i32 {
    let file = sqlx::query!(
        "INSERT INTO files (filename, hash, size) VALUES ($1, $2, $3)
         RETURNING id",
        filename,
        format!("test_hash_{}", filename),
        1024i64
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create test file");
    
    file.id
}

/// Insert metadata for a file
pub async fn insert_metadata(
    pool: &PgPool,
    file_id: i32,
    bpm: Option<&str>,
    key: Option<&str>,
    duration: Option<&str>,
) {
    let bpm_val = bpm.and_then(|b| b.parse::<f64>().ok());
    let duration_val = duration.and_then(|d| d.parse::<i32>().ok());
    
    let _ = sqlx::query!(
        "INSERT INTO musical_metadata (file_id, bpm, key, duration_ms) 
         VALUES ($1, $2, $3, $4)",
        file_id,
        bpm_val,
        key,
        duration_val
    )
    .execute(pool)
    .await;
}
```

### Steps to Implement
1. Check existing test files for similar helper patterns
2. Create functions with proper async/await
3. Handle database schema correctly (verify table/column names)
4. Add error handling as needed
5. Export from `tests/common/mod.rs`

### Verification After Implementation
Run `cargo check --tests -p midi-pipeline 2>&1 | grep "E0425"` to verify E0425 errors are gone.

---

## FIX #5: Fix SearchQueryBuilder method signatures [PRIORITY: MEDIUM]

**Impact**: 16 errors (3.7% of total)
**Estimated Time**: 10 minutes
**Automatable**: NO
**Affected File**: search_repository_test.rs

### Root Cause
Test code calls builder methods with wrong types or calls non-existent methods.

### Problems and Solutions

**Problem 1: min_bpm() and max_bpm() expect f64, not Option<String>**

**Location**: search_repository_test.rs, lines 1453-1454

```rust
WRONG:
.min_bpm(Some("150.0".to_string()))  // Expected f64, found Option<String>
.max_bpm(Some("100.0".to_string()))  // Expected f64, found Option<String>

RIGHT - Option 1 (Parse the string):
.min_bpm(150.0)
.max_bpm(100.0)

RIGHT - Option 2 (If builder should accept Option):
.min_bpm(150.0)
.max_bpm(100.0)
```

**Problem 2: offset() method doesn't exist on SearchQueryBuilder**

**Location**: search_repository_test.rs, line 1920

```rust
WRONG:
let query = SearchQueryBuilder::new()
    .offset(10)
    .limit(10)

SOLUTION 1 - Pass to search() instead:
let query = SearchQueryBuilder::new();
let results = SearchRepository::search(&pool, &query, limit, offset).await;

SOLUTION 2 - Add offset/limit to builder:
// Modify SearchQueryBuilder struct to include these fields
// And add methods to set them
```

### Implementation Steps
1. Verify SearchQueryBuilder source code
2. Check what methods it actually supports
3. Update test code to match API
4. For pagination: likely pass limit/offset to search() not builder

---

## EXECUTION CHECKLIST

### Phase 1: Quick Fixes (12 minutes)
- [ ] Fix #1: Remove tauri::State::from() wrapper (10 min)
- [ ] Fix #5: Fix format strings (2 min)
- [ ] Run `cargo check --tests` to verify ~212 errors remain

### Phase 2: Medium Fixes (15 minutes)
- [ ] Fix #2: Add limit/offset to SearchRepository::search() (15 min)
- [ ] Run `cargo check --tests` to verify ~120 errors remain

### Phase 3: Complex Fixes (40 minutes)
- [ ] Fix #3: Create missing helper functions (30 min)
- [ ] Fix #4: Fix SearchQueryBuilder methods (10 min)
- [ ] Run `cargo check --tests` to verify 0 compilation errors

### Final Verification
```bash
cargo check --tests -p midi-pipeline 2>&1 | grep "^error" | wc -l
# Should output: 0
```

---

## AUTOMATION SCRIPT (Optional)

Create `fix_errors.sh`:
```bash
#!/bin/bash
set -e

echo "Starting test compilation error fixes..."

# Fix #1: Remove State wrapper
echo "Fixing State wrapper..."
sed -i 's/tauri::State::from(&state)/state/g' \
  pipeline/src-tauri/tests/file_import_test.rs \
  pipeline/src-tauri/tests/workflows_test.rs

# Fix #5: Format strings
echo "Fixing format strings..."
sed -i 's/"Expected {100}, found {count}"/"Expected 100, found {}"/g' \
  pipeline/src-tauri/tests/tag_repository_test.rs
sed -i 's/"Expected {5}, found {count}"/"Expected 5, found {}"/g' \
  pipeline/src-tauri/tests/tag_repository_test.rs
sed -i 's/"Expected {0}, found {count}"/"Expected 0, found {}"/g' \
  pipeline/src-tauri/tests/tag_repository_test.rs

echo "Automated fixes complete. Check cargo output for remaining issues."
```

---

## SUMMARY TABLE

| Fix | Errors | Lines Affected | Time | Complexity | Automatable |
|-----|--------|----------------|------|------------|-------------|
| #1 | 209 | 50+ per file | 10m | LOW | YES |
| #2 | 3 | 348, 910, 924 | 2m | LOW | YES |
| #3 | 74 | 40+ across files | 15m | MEDIUM | PARTIAL |
| #4 | 16 | 1453, 1454, 1920 | 10m | MEDIUM | NO |
| #5 | 92 | Varies | 30m | HIGH | NO |
| **TOTAL** | **394** | - | **67m** | **MEDIUM** | **PARTIAL** |


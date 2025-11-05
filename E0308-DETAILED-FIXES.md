# E0308 Type Mismatch - Detailed Fix Guide

**Generated:** 2025-11-05  
**Total Errors:** 58  
**Estimated Fix Time:** 40-45 minutes  

## Category 1: Pool Reference Mismatches (30 errors)

### Problem
```rust
// WRONG - Expected &Pool<Postgres>, found Pool<Postgres>
cleanup_test_files(state.database.pool().await, pattern)

// Function signature
async fn cleanup_test_files(pool: &PgPool, pattern: &str) {
    // pool parameter is a REFERENCE, not owned
}
```

### Solution
```rust
// CORRECT - Add & before the pool
cleanup_test_files(&state.database.pool().await, pattern)
```

### Files and Line Numbers

**workflows_test.rs (30 instances)**
- Lines: 157, 200, 241, 294, 342, 375, 418, 467, 515, 555, 601, 645, 689, 731, 774, 818, 858, 908, 943, 976, 1016, 1035, 1049, 1064, 1078, 1092, 1102, 1105, 1115, 1119

**file_import_test.rs (7 instances)**
- Lines: 650, 1035, 1309, 1668, 2105, 2190, 2242

**files.rs (1 instance)**
- Line: 317

**file_import.rs (2 instances)**
- Lines: 92, 361

### Quick Fix Commands

```bash
# Fix all in workflows_test.rs
sed -i 's/cleanup_test_files(state\.database\.pool()\.await/cleanup_test_files(\&state.database.pool().await/g' \
  /home/dojevou/projects/midi-software-center/pipeline/src-tauri/tests/workflows_test.rs

# Fix all in file_import_test.rs
sed -i 's/cleanup_test_files(state\.database\.pool()\.await/cleanup_test_files(\&state.database.pool().await/g' \
  /home/dojevou/projects/midi-software-center/pipeline/src-tauri/tests/file_import_test.rs

# Verify the fixes
cargo check --tests -p midi-pipeline 2>&1 | grep "E0308.*Pool" | wc -l
```

### Verification
After applying fixes, the following command should return 0:
```bash
cargo check --tests -p midi-pipeline 2>&1 | grep "expected.*&Pool.*found.*Pool" | wc -l
```

---

## Category 2: String vs TagResponse Mismatches (8 errors)

### Problem
```rust
// WRONG - tags is Vec<TagResponse>, not Vec<String>
let tags = get_tags().await;  // Vec<TagResponse>
assert!(tags.contains(&"template".to_string()));  // Error: expected TagResponse, found String
```

### Solution
```rust
// CORRECT - Compare using the name field
assert!(tags.iter().any(|tag| tag.name == "template"))

// Or use a filter
let has_template = tags.iter().any(|t| t.name == "template");
assert!(has_template);
```

### Files and Line Numbers

**workflows_test.rs (8 instances)**
- Line 186: `assert!(tags.contains(&"template".to_string()));`
- Lines 339, 340: Tag checking in remix workflow
- Line 457: Tag checking in another test
- Lines 855, 856: Tag containment assertions
- Lines 1013, 1014: More tag checks

### Detailed Fixes by Location

#### Line 186
```rust
// BEFORE
assert!(tags.contains(&"template".to_string()));

// AFTER
assert!(tags.iter().any(|tag| tag.name == "template"));
```

#### Line 339
```rust
// BEFORE
assert!(tags_original.contains(&"original".to_string()));

// AFTER
assert!(tags_original.iter().any(|tag| tag.name == "original"));
```

#### Line 340
```rust
// BEFORE
assert!(tags_remix.contains(&"remix".to_string()));

// AFTER
assert!(tags_remix.iter().any(|tag| tag.name == "remix"));
```

#### Lines 457, 855, 856, 1013, 1014
Similar pattern - replace `contains()` calls with `iter().any()` checking the `name` field.

### Pattern Replacement Regex
```bash
# This pattern finds assertions comparing tags with strings
# You'll need to manually update each one, but here's the pattern:
#
# Find: assert!\(tags.*\.contains\(&"([^"]+)"\.to_string\(\)\)\);
# Replace: assert!(tags.iter().any(|tag| tag.name == "$1"));
```

### Manual Fix Process
1. Open `pipeline/src-tauri/tests/workflows_test.rs`
2. Find each line (186, 339, 340, 457, 855, 856, 1013, 1014)
3. For each line, change from:
   ```rust
   assert!(tags.contains(&"string".to_string()));
   ```
   to:
   ```rust
   assert!(tags.iter().any(|tag| tag.name == "string"));
   ```

---

## Category 3: AppState Argument Order Mismatches (9 errors)

### Problem
```rust
// WRONG - Arguments in wrong order
// Error: expected `i64`, found `&AppState`
// This suggests first parameter should be i64, not AppState
```

### Root Cause
Functions in `file_import.rs` have their parameters in a specific order. Test calls are providing arguments in the wrong order.

### Files and Locations

**file_import.rs (8-9 instances)**
- Line 92: Multiple argument order issues (appears 7 times)
- Line 170: One argument order issue

### Solution
Need to examine the function signatures and match the argument order. This requires:

1. Open `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/commands/file_import.rs`
2. Find each `_impl` function definition
3. Note the exact parameter order
4. Find all calls to that function
5. Reorder arguments to match

### Expected Pattern
```rust
// Function definition (line 92 area)
pub async fn some_impl_function(
    file_id: i64,
    state: &AppState,
    other_params: String,
    // ... more parameters in specific order
) -> Result<...>

// WRONG call (current error)
some_impl_function(&state, file_id, other_params)

// CORRECT call
some_impl_function(file_id, &state, other_params)
```

### Verification
```bash
# Check remaining AppState errors
cargo check --tests -p midi-pipeline 2>&1 | grep "E0308.*AppState" | wc -l
```

---

## Category 4: Option Type Mismatches (8 errors)

### Problem
Functions expect `Option<T>` but receive `T`, or vice versa.

### Patterns Found

#### Pattern A: Option<i64> vs i64
```rust
// WRONG
let result: i64 = query_result;  // query_result is Option<i64>

// CORRECT - Unwrap with default
let result: i64 = query_result.unwrap_or(0);
```

#### Pattern B: Option<String> vs String
```rust
// WRONG
builder.min_bpm("150.0".to_string())  // expects Option<f64>, got String

// CORRECT - Wrap in Some or match
builder.min_bpm(Some(150.0))  // or
let bpm: f64 = "150.0".parse()?;
builder.min_bpm(Some(bpm))
```

### Files and Line Numbers

**file_repository_test.rs (8 instances)**
- Lines: 2255, 2276, 2297, 2318, 2319, 2325, 2324, 2398

**search.rs (2 instances)**
- Line: 92 (appears 2 times)

**tag_repository.rs (1 instance)**
- Line: 47

**file_repository.rs (3 instances)**
- Line: 256 (appears 3 times)

### Specific Examples

#### file_repository_test.rs:2255
```rust
// WRONG - Expected i64, found Option<{integer}>
// This is trying to pass Option value where i64 is expected

// CORRECT - Unwrap the Option
.with_limit(limit.unwrap_or(10))  // Use unwrap_or with default
```

#### search.rs:92
```rust
// WRONG - Option<String> where f64 is expected
.min_bpm(Some("150.0".to_string()))

// CORRECT - Parse string to f64 first
let bpm: f64 = "150.0".parse().unwrap_or(0.0);
.min_bpm(Some(bpm))
```

### General Fix Pattern
```rust
// For Option<T> to T:
value.unwrap_or(default_value)

// For T to Option<T>:
Some(value)

// For Option<String> to f64:
let num: f64 = string_val.parse().unwrap_or(0.0);
```

### Bulk Fix Strategy
1. Review each occurrence
2. Determine if you're unwrapping an Option or wrapping a value
3. Apply appropriate fix:
   - Unwrap: `.unwrap_or(default)` or `.unwrap_or_default()`
   - Wrap: `Some(value)`
   - Convert: `.parse().unwrap_or(default)`

---

## Category 5: Other/Miscellaneous (3 errors)

### Files and Locations
- tag_repository_test.rs: Line 1715
- file_import_test.rs: Line 2354
- search_repository_test.rs: Lines 1456, 1457, 1477, 1792

### Handling
These require case-by-case analysis. Common patterns:
- Type conversion mismatches
- Function signature changes
- Incorrect method calls

Review the actual error messages for each and apply appropriate fixes.

---

## Implementation Checklist

- [ ] **Phase 1:** Fix all 30 Pool mismatches (5 min)
  - [ ] Apply sed replacement to workflows_test.rs
  - [ ] Apply sed replacement to file_import_test.rs
  - [ ] Verify with `cargo check --tests`

- [ ] **Phase 2:** Fix all 8 String vs TagResponse errors (10 min)
  - [ ] Update line 186 in workflows_test.rs
  - [ ] Update lines 339, 340, 457, 855, 856, 1013, 1014
  - [ ] Verify with `cargo check --tests`

- [ ] **Phase 3:** Audit AppState argument order (15 min)
  - [ ] Review file_import.rs function signatures
  - [ ] Fix all calls to match parameter order
  - [ ] Verify with `cargo check --tests`

- [ ] **Phase 4:** Fix Option type mismatches (10 min)
  - [ ] file_repository_test.rs: 8 locations
  - [ ] search.rs: 2 locations
  - [ ] Add unwrap_or() or Some() wrappers
  - [ ] Verify with `cargo check --tests`

- [ ] **Phase 5:** Fix remaining 3 errors (5 min)
  - [ ] Case-by-case analysis and fixes
  - [ ] Final verification

- [ ] **Final:** Run full test suite
  - [ ] `cargo check --tests -p midi-pipeline`
  - [ ] `cargo test --workspace --lib`

---

## Verification Commands

```bash
# Check progress after each phase
cargo check --tests -p midi-pipeline 2>&1 | grep "error\[E0308\]" | wc -l

# After Phase 1 (should be 28 remaining)
cargo check --tests -p midi-pipeline 2>&1 | grep "E0308.*Pool" | wc -l

# After Phase 2 (should be 20 remaining)
cargo check --tests -p midi-pipeline 2>&1 | grep "E0308.*TagResponse" | wc -l

# After Phase 3 (should be 11 remaining)
cargo check --tests -p midi-pipeline 2>&1 | grep "E0308.*Option" | wc -l

# Final check (should be 0)
cargo check --tests -p midi-pipeline 2>&1 | grep "error\[E0308\]" | wc -l
```

---

## Success Criteria

- All 58 E0308 errors eliminated
- `cargo check --tests -p midi-pipeline` passes with 0 E0308 errors
- `cargo test --workspace --lib` passes baseline tests
- No new errors introduced

---

## Expected Outcome

```bash
# Before fixes
$ cargo check --tests -p midi-pipeline 2>&1 | grep "error\[" | wc -l
58

# After fixes  
$ cargo check --tests -p midi-pipeline 2>&1 | grep "error\[" | wc -l
0
```


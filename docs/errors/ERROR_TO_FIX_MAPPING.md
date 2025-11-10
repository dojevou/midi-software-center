# Test Compilation Errors - Detailed Mapping & Fixes

## ERROR CATEGORIES REFERENCE

---

## 1. BRACE/SYNTAX ERRORS

### Error Pattern
```
error: unexpected closing delimiter: `}`
  --> pipeline/src-tauri/tests/FILE:LINE:1
```

### Files Affected
1. `file_repository_test.rs:2443` - Extra `}`
2. `metadata_repository_test.rs:1812` - Extra `}`  
3. `tag_repository_test.rs:1720` - Extra `}`

### Fix Applied
```bash
# Remove last line (extra brace) from each file
head -n -1 FILE.rs > FILE.rs.tmp && mv FILE.rs.tmp FILE.rs
```

### Verification
```bash
tail -1 file_repository_test.rs  # Should NOT be just "}"
```

---

## 2. TAURI STATE CONSTRUCTOR ERRORS

### Error Pattern
```
error[E0423]: cannot initialize a tuple struct which contains private fields
     --> pipeline/src-tauri/tests/journey_test.rs:93:40
      |
   93 |         let initial_count = get_file_count(tauri::State(&state)).await.unwrap();
      |                                            ^^^^^^^^^^^^
```

### Root Cause
The `tauri::State<T>` struct has private fields in Tauri 2.x. You cannot construct it directly.

### Affected Files & Count
- `journey_test.rs`: ~183 error instances
- `file_import_test.rs`: ~40 error instances
- Other tests: Scattered

### Examples of This Error
```rust
// ❌ ALL OF THESE FAIL:

tauri::State(&state)
tauri::State(&mut state)
tauri::State(Arc::new(state))

// From tests:
let initial_count = get_file_count(tauri::State(&state)).await?;
let details = get_file_details(tauri::State(&state), file_id).await?;
let tags = get_file_tags(tauri::State(&state), file_id).await?;
```

### Fix Options

#### OPTION A: Call Business Logic Directly (RECOMMENDED)
Create wrapper functions that accept `&AppState`:

```rust
// src/commands/files.rs - REFACTOR THESE
pub async fn get_file_count(state: &AppState) -> Result<i64, String> {
    // implementation
}

// Then in tests:
#[tokio::test]
async fn test_file_count() {
    let pool = setup_test_pool().await;
    let state = AppState { db: pool };
    let count = get_file_count(&state).await?;  // ✅ WORKS
}
```

#### OPTION B: Create Test Wrapper
```rust
// tests/test_helpers.rs
pub struct TestContext {
    pub pool: PgPool,
    pub state: AppState,
}

impl TestContext {
    pub async fn new() -> Self {
        let pool = setup_test_pool().await;
        let state = AppState { db: pool };
        TestContext { pool, state }
    }
}

// Then in tests:
#[tokio::test]
async fn test_something() {
    let ctx = TestContext::new().await;
    let count = get_file_count(&ctx.state).await?;  // ✅ WORKS
}
```

#### OPTION C: Extract State Field
```rust
// If function signature can't change, access pool directly:
#[tokio::test]
async fn test_something() {
    let pool = setup_test_pool().await;
    // Call internal implementation directly
    let count = file_repository::count_files(&pool).await?;  // ✅ WORKS
}
```

### Recommended Solution
**Use Option A + Test Context (Option B)** for best results:

```rust
// Step 1: Modify command signatures (optional but recommended)
// pipeline/src-tauri/src/commands/files.rs
- pub async fn get_file_count(state: State<'_, AppState>) -> Result<i64, String>
+ pub async fn get_file_count(state: &AppState) -> Result<i64, String>

// Step 2: Update Tauri command wrapper
#[tauri::command]
pub async fn get_file_count_cmd(state: State<'_, AppState>) -> Result<i64, String> {
    get_file_count(&*state).await  // Dereference State to AppState
}

// Step 3: Use in tests
#[tokio::test]
async fn test_get_file_count() {
    let pool = setup_test_pool().await;
    let state = AppState { db: pool };
    let count = get_file_count(&state).await.unwrap();
    assert!(count >= 0);
}
```

### Migration Checklist
- [ ] Check current function signatures in `src/commands/`
- [ ] Create test context struct (if needed)
- [ ] Update journey_test.rs - replace all `tauri::State(&state)` calls
- [ ] Update file_import_test.rs - same replacement
- [ ] Test compilation after changes
- [ ] Run test suite

---

## 3. MISSING/CHANGED STRUCT FIELDS

### Error Pattern (Multiple Variants)

#### FileMetadata.file_id not found
```
error[E0609]: no field `file_id` on type `midi_pipeline::commands::file_import::FileMetadata`
     --> pipeline/src-tauri/tests/journey_test.rs:100:40
      |
  100 |         let id = import_result.file_id;
      |                                 ^^^^^^ `FileMetadata` does not have this field
```

#### BpmRange.min_bpm not found
```
error[E0609]: no field `min_bpm` on type `BpmRange`
      |
      |         let min = range.min_bpm;
      |                         ^^^^^^^ `BpmRange` does not have this field
```

### Field Mapping

#### FileMetadata
```rust
// ❌ OLD FIELD NAMES
metadata.file_id          // ERROR
metadata.file_path        // ERROR
metadata.file_size_bytes  // ERROR (check actual name)

// ✅ NEW FIELD NAMES
metadata.id               // CORRECT
metadata.filepath         // CORRECT
metadata.size_bytes       // CHECK db/models.rs
```

**Affected Files:**
- `journey_test.rs`: 98 instances of `.file_id`
- `file_import_test.rs`: 40+ instances
- `workflows_test.rs`: 20+ instances
- `workflows_extended_test.rs`: 15+ instances

#### BpmRange
```rust
// ❌ OLD
range.min_bpm  // ERROR
range.max_bpm  // ERROR

// ✅ NEW (Check actual struct)
range.bpm_min
// OR
range.bpm_range.start / range.bpm_range.end
// OR different naming entirely
```

**Affected Files:** 
- `journey_test.rs`: 3 instances each = 6 total
- Other tests: 3-5 scattered

#### MidiFile
```rust
// ❌ OLD
file.file_path         // ERROR
file.file_size_bytes   // ERROR

// ✅ NEW
file.filepath
file.size_bytes  // OR similar
```

**Affected Files:**
- `journey_test.rs`: 3 instances

### Automated Fix Script
```bash
#!/bin/bash
# Fix field names across all test files

FILES="journey_test.rs file_import_test.rs workflows_test.rs workflows_extended_test.rs"
TEST_DIR="pipeline/src-tauri/tests"

for file in $FILES; do
    [ -f "$TEST_DIR/$file" ] || continue
    
    # FileMetadata.file_id -> .id
    sed -i 's/\.file_id/.id/g' "$TEST_DIR/$file"
    
    # file_path -> filepath
    sed -i 's/\.file_path/.filepath/g' "$TEST_DIR/$file"
done

echo "Field names updated!"
```

### Verification After Fix
```rust
// Verify no more .file_id or .file_path
grep -n "\.file_id\|\.file_path" journey_test.rs  # Should be empty
```

---

## 4. FUNCTION ARGUMENT COUNT MISMATCHES

### Error Pattern
```
error[E0061]: this function takes 4 arguments but 2 arguments were supplied
     --> pipeline/src-tauri/tests/journey_test.rs:102:9
      |
  102 |         import_single_file(&state, "file.mid"),
      |         ^^^^^^^^^^^^^^^^^ expected 4 arguments
```

### Common Cases

#### Case 1: Missing Arguments
```rust
// ❌ OLD - Called with 2 args
let result = import_single_file(&state, "file.mid");

// ✅ NEW - Needs 4 args (for example)
let result = import_single_file(
    &state,
    "file.mid",
    Some(ImportOptions::default()),
    None  // Optional callback
)?;
```

#### Case 2: Extra Arguments
```rust
// ❌ OLD - Called with 4 args
let result = analyze_file(&state, file_id, opts, callback);

// ✅ NEW - Only needs 2 args
let result = analyze_file(&state, file_id)?;
```

#### Case 3: Type Mismatch in Arguments
```rust
// ❌ OLD - Wrong type for arg 2
let result = search_files(&state, "query", SortBy::Name);

// ✅ NEW - Arg 2 is now SearchOptions
let result = search_files(
    &state,
    SearchOptions {
        query: "query".to_string(),
        sort: SortBy::Name,
        // ...
    }
)?;
```

### Affected Instances (~95)
- `journey_test.rs`: ~40 instances
- `workflows_test.rs`: ~30 instances  
- `workflows_extended_test.rs`: ~25 instances

### Fix Process
1. **Identify each function**: `grep -n "takes \d* arguments" compile_errors.txt`
2. **Check source signature**: `grep -A 5 "pub async fn FUNCTION_NAME" src/commands/*.rs`
3. **Update call site** to match signature

### Example Fixes

#### Fix 1: import_single_file
```rust
// BEFORE
let result = import_single_file(&state, path).await?;

// AFTER - Check actual signature:
// pub async fn import_single_file(
//     state: &AppState,
//     path: &str,
//     options: ImportOptions,  // NEW
// ) -> Result<...>
let result = import_single_file(
    &state,
    path,
    ImportOptions::default()
).await?;
```

#### Fix 2: get_file_details
```rust
// BEFORE - Takes file_id
let details = get_file_details(&state, file_id).await?;

// AFTER - Check if signature changed
let details = get_file_details(&state, file_id).await?;  // Maybe no change?
```

### Verification Script
```bash
# Check for remaining argument count errors
cargo build --tests 2>&1 | grep "takes.*arguments but.*supplied" | head -10
```

---

## 5. TYPE MISMATCHES

### Error Pattern
```
error[E0308]: mismatched types
     --> pipeline/src-tauri/tests/journey_test.rs:150:30
      |
  150 |         let tags: Vec<String> = get_file_tags(state).await?;
      |                   ^^^^^^^^^^^   ^^^^^^^^^^^^^^^^^^^^^^ expected `Vec<String>`, found `Vec<TagResponse>`
```

### Common Causes

#### Cause 1: Wrong Return Type Expected
```rust
// ❌ WRONG
let tags: Vec<String> = get_file_tags(&state).await?;

// ✅ CORRECT - Check actual return type
let tags: Vec<TagResponse> = get_file_tags(&state).await?;
// OR
let tag_names: Vec<String> = get_file_tags(&state).await?
    .into_iter()
    .map(|t| t.name)
    .collect();
```

#### Cause 2: Missing .await
```rust
// ❌ WRONG - Forgot .await
let result: String = get_file_name(&state);

// ✅ CORRECT
let result: String = get_file_name(&state).await?;
```

#### Cause 3: Async/Future Type
```rust
// ❌ WRONG - Got Future type instead of Result
let result = process_file(&state, id);  // Returns Future

// ✅ CORRECT
let result = process_file(&state, id).await?;  // Resolve Future
```

### Affected Instances (~307)
- Most in `journey_test.rs`
- Many in `file_import_test.rs`
- Scattered in workflow tests

### Fix Strategy
1. **Check return type** of the function being called
2. **Add .await** if it's an async function
3. **Adjust variable type** to match actual return
4. **Use adapters** (map, filter, etc.) if converting types

### Common Fixes

```rust
// Pattern 1: Type mismatch with Vec
- let tags: Vec<String> = get_tags(&state).await?;
+ let tags: Vec<TagResponse> = get_tags(&state).await?;

// Pattern 2: Missing await
- let files = list_files(&state, page, limit);
+ let files = list_files(&state, page, limit).await?;

// Pattern 3: Need type conversion
- let id: i64 = result.file_id;  // field doesn't exist
+ let id: i64 = result.id;

// Pattern 4: Wrong Result wrapper
- let result: Result<i64> = count_files(&state).await;
+ let result: i64 = count_files(&state).await?;
```

---

## 6. MISSING HELPER FUNCTIONS

### Error Pattern
```
error[E0425]: cannot find function `setup_test_state` in this scope
     --> pipeline/src-tauri/tests/workflows_test.rs:50:20
      |
   50 |         let state = setup_test_state(pool, file_path);
      |                     ^^^^^^^^^^^^^^^^ not found in this scope
```

### Missing Functions

#### Function 1: setup_test_state
```rust
// ❌ ERROR: Not found
let state = setup_test_state(pool, "test.mid");

// ✅ SOLUTION: Create it
async fn setup_test_state(pool: &PgPool, test_file: &str) -> Result<AppState, String> {
    let state = AppState { db: pool.clone() };
    
    // Optionally: import test file
    // import_file(&state, test_file).await?;
    
    Ok(state)
}
```

**Affected Files:**
- `workflows_test.rs`: 9 instances
- `workflows_extended_test.rs`: 6 instances

#### Function 2: import_and_analyze_file
```rust
// ❌ ERROR: Not found
let result = import_and_analyze_file(&state, "file.mid");

// ✅ SOLUTION: Create helper
async fn import_and_analyze_file(state: &AppState, path: &str) -> Result<FileAnalysis, String> {
    let import_result = import_single_file(state, path, ImportOptions::default()).await?;
    let analysis = analyze_file(state, import_result.id).await?;
    Ok(analysis)
}
```

**Affected Files:**
- `workflows_test.rs`: 1 instance
- `workflows_extended_test.rs`: 1 instance

### Helper Functions to Create

Create file: `pipeline/src-tauri/tests/helpers.rs` (or update existing)

```rust
use crate::AppState;
use sqlx::PgPool;

/// Setup test database connection
pub async fn setup_test_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/midi_test".to_string());
    
    PgPool::connect(&database_url).await
}

/// Create AppState for testing
pub async fn setup_test_state(pool: &PgPool, _test_file: Option<&str>) -> Result<AppState, String> {
    Ok(AppState {
        db: pool.clone(),
    })
}

/// Import and analyze a test file in one operation
pub async fn import_and_analyze_file(state: &AppState, path: &str) -> Result<(i64, Analysis), String> {
    // Implement using actual import/analyze functions
    todo!("Implement based on available functions")
}

/// Cleanup after tests
pub async fn cleanup_test_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Clear test data
    pool.execute("TRUNCATE TABLE files CASCADE").await?;
    Ok(())
}
```

### Update Tests to Use Helpers

```rust
mod helpers;
use helpers::*;

#[tokio::test]
async fn test_workflow() {
    let pool = setup_test_pool().await.unwrap();
    let state = setup_test_state(&pool, None).await.unwrap();
    
    // test body
    
    cleanup_test_database(&pool).await.unwrap();
}
```

---

## 7. TRAIT AND GENERIC PARAMETER ISSUES

### Error Pattern 1: Missing Generic
```
error[E0107]: missing generics for trait `Emitter`
     --> pipeline/src-tauri/tests/file_import_test.rs:310:6
      |
  310 | impl Emitter for MockWindow {
      |      ^^^^^^^ expected 1 generic argument
```

### Fix
```rust
// ❌ OLD
impl Emitter for MockWindow {
    fn emit(&self, label: &str, payload: impl Serialize) -> Result<()> {
        Ok(())
    }
}

// ✅ NEW
use tauri::Runtime;

impl<R: Runtime> Emitter<R> for MockWindow {
    fn emit(&self, label: &str, payload: impl Serialize) -> Result<()> {
        Ok(())
    }
}
```

### Error Pattern 2: Missing Comparison Trait
```
error[E0277]: can't compare `TagResponse` with `TagResponse`
     --> pipeline/src-tauri/tests/file_import_test.rs:250:20
      |
  250 |         assert_eq!(result, expected);
      |                    ^^^^^^ doesn't implement `PartialEq`
```

### Fix
```rust
// ✅ SOLUTION 1: Add PartialEq to struct
#[derive(PartialEq, Eq)]
pub struct TagResponse {
    // fields
}

// ✅ SOLUTION 2: Compare fields manually
assert_eq!(result.len(), expected.len());
for (r, e) in result.iter().zip(expected.iter()) {
    assert_eq!(r.id, e.id);
    assert_eq!(r.name, e.name);
}

// ✅ SOLUTION 3: Use custom comparison
fn assert_tags_equal(actual: &[TagResponse], expected: &[TagResponse]) {
    // custom logic
}
assert_tags_equal(&result, &expected);
```

---

## 8. QUICK REFERENCE: FIND & REPLACE PATTERNS

### Sed Commands for Bulk Fixes

```bash
# Fix field names
sed -i 's/\.file_id/.id/g' file.rs
sed -i 's/\.file_path/.filepath/g' file.rs

# Fix Emitter trait
sed -i 's/impl Emitter for/impl<R: tauri::Runtime> Emitter<R> for/g' file.rs

# Fix Tauri State (manual - too many variants)
# Search and replace based on context

# Fix .await (pattern: function call without await)
# Manual - too many edge cases
```

---

## 9. COMPILATION ERROR TROUBLESHOOTING

### If You Get New Errors After Fixes

1. **Check error message carefully** - Get exact line and error code
2. **Look up error code** - `error[Exxx]` format at rust docs
3. **Examine context** - Read surrounding code
4. **Check related changes** - Did you modify something that broke other code?

### Common Cascading Errors

```
If you fix error A → You might create error B
E0061 → E0308  (arg count → type mismatch)
E0425 → E0607  (missing fn → can't access field)
E0599 → E0277  (missing method → trait bound)
```

### Testing After Fixes

```bash
# Full test build
cargo build --tests

# Run specific test
cargo test --test journey_test -- --nocapture

# Show all errors (not just first)
cargo build --tests 2>&1 | grep "^error"

# Count errors by type
cargo build --tests 2>&1 | grep "^error\[" | cut -d: -f1 | sort | uniq -c | sort -rn
```

---

## 10. SUCCESS CHECKLIST

- [ ] All brace errors fixed (3 files)
- [ ] All field names updated (.file_id → .id, etc.)
- [ ] All imports fixed (Emitter generic)
- [ ] Test helpers created
- [ ] Tauri State constructor calls fixed  
- [ ] Function signatures verified
- [ ] Type mismatches resolved
- [ ] Missing functions implemented
- [ ] `cargo build --tests` succeeds
- [ ] `cargo test` passes all tests

---


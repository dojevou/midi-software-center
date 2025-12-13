# Architecture Decisions - Complete Q&A

## Question 1: Wrapper Function Location ✅

**Your Question:**
> Should wrapper functions be in the same src/commands/*.rs files as the original commands?
> Or create new functions with different names (e.g., get_file_count for tests, get_file_count_cmd for Tauri)?

**ANSWER:**
```
✅ YES - In same files with _impl suffix pattern
```

**Why this approach:**
- Keep related functions together
- Easy to navigate and maintain
- Naming: `get_file_count_impl` (internal) + `get_file_count` (Tauri)
- No new files needed
- Clear naming convention (`_impl` = implementation for tests/reuse)

**Implementation:**
```rust
// src/commands/files.rs

pub async fn get_file_count_impl(state: &AppState) -> Result<i64, String> {
    // Internal implementation - used by tests
}

#[tauri::command]
pub async fn get_file_count(state: State<'_, AppState>) -> Result<i64, String> {
    get_file_count_impl(&*state).await  // Delegate to impl
}
```

**Decision Made:** Use `_impl` suffix in same files ✅

---

## Question 2: Scope of Wrapping ✅

**Your Question:**
> How many commands need wrapping? (I need to count them)
> Should ALL commands be wrapped, or just the ones called by tests?

**ANSWER:**
```
✅ Only commands called by tests (estimated 15-25)
❌ NOT all commands (unnecessary work)
```

**Why selective wrapping:**
- Only commands that tests call need `_impl` versions
- Other commands stay as-is
- Reduces scope and risk
- Faster implementation

**Count Script:**
```bash
#!/bin/bash
echo "Finding all commands called by tests..."
grep -roh '\(get_[a-z_]*\|list_[a-z_]*\|search_[a-z_]*\|import_[a-z_]*\|analyze_[a-z_]*\|set_[a-z_]*\)(' \
  pipeline/src-tauri/tests/ \
  | sed 's/($//' \
  | sort | uniq -c | sort -rn | head -30
```

**Decision Made:** Selective wrapping (scan first) ✅

---

## Question 3: Test Calling Pattern ✅

**Your Question:**
> Should tests call the new wrapper functions directly?
> Or should we create test utilities that handle the conversion?

**ANSWER:**
```
✅ Tests call _impl functions DIRECTLY
✅ PLUS: Create test utilities (TestCtx helper)
```

**Why both:**

1. **Direct calls** - Simple and clear
   ```rust
   let count = get_file_count_impl(&state).await?;
   ```

2. **Test utilities** - Cleaner setup
   ```rust
   let ctx = TestCtx::new().await;
   let count = get_file_count_impl(&ctx.state).await?;
   cleanup_test_database(&ctx.pool).await?;
   ```

**The Pattern:**
```rust
// tests/test_helpers.rs
pub struct TestCtx {
    pub pool: PgPool,
    pub state: AppState,
}

impl TestCtx {
    pub async fn new() -> Self { /* setup */ }
}

// tests/journey_test.rs
#[tokio::test]
async fn test_example() {
    let ctx = TestCtx::new().await;
    let result = get_file_count_impl(&ctx.state).await?;
    cleanup_test_database(&ctx.pool).await?;
}
```

**Decision Made:** Direct calls WITH TestCtx helper ✅

---

## Question 4: Backward Compatibility ✅

**Your Question:**
> Do Tauri command handlers need to keep calling the original command functions?
> Or is it OK to refactor so Tauri commands call the wrapper functions?

**ANSWER:**
```
✅ YES - Keep Tauri command signatures IDENTICAL
✅ Tauri commands delegate to _impl versions
```

**Why maintain compatibility:**
- Frontend code doesn't change
- IPC API remains stable
- No breaking changes
- Smooth migration

**The Pattern (DO THIS):**
```rust
// Keep Tauri command signature identical
#[tauri::command]
pub async fn get_file_count(state: State<'_, AppState>) -> Result<i64, String> {
    get_file_count_impl(&*state).await  // Delegate
}
```

**DO NOT (Breaking changes):**
```rust
❌ #[tauri::command]
   pub async fn get_file_count(state: &AppState) -> Result<i64, String> {
       // Changes API - breaks frontend!
   }
```

**Decision Made:** Keep Tauri sigs unchanged, delegate to _impl ✅

---

## Question 5: Priority - Fix Order ✅

**Your Question:**
> Fix all 1,005 errors in one go, or start with high-impact errors first?

**ANSWER:**
```
✅ Three-phase approach (not all at once)
```

**Why phased approach:**
- Verify each phase works before next
- Catch issues early
- Easier debugging
- Less overwhelming

**Phase Breakdown:**

### **Phase A: Quick Wins** (30 min) - Fixes ~143 errors
1. Field renames (.file_id → .id)            [110 errors]
2. Trait generics (Emitter<R>)               [30 errors]
3. Remove extra braces                       [3 errors]

**Verification:** `cargo build --tests | grep "^error" | wc -l`

### **Phase B: Critical Path** (90 min) - Fixes ~250+ State errors
4. Create `_impl` functions in commands
   - get_file_count_impl
   - get_file_details_impl
   - list_files_impl
   - import_single_file_impl
   - analyze_file_impl
   - search_files_impl
   - (etc. - as identified by script)

5. Update all test calls to use `_impl` versions

**Verification:** `cargo build --tests` + compile check

### **Phase C: Validation** (15 min)
6. Full compilation
7. Run test suite
8. Verify no new warnings

**Total: ~2.5 hours**

**Decision Made:** Three-phase approach ✅

---

## Question 6: Implementation Style ✅

**Your Question:**
> Should I refactor existing command functions to take &AppState instead of State<'_, AppState>?
> Or keep original signatures and create new wrapper functions with different signatures?

**ANSWER:**
```
✅ KEEP original signatures (State<'_, AppState>)
✅ CREATE new _impl functions with &AppState
```

**Three-Layer Architecture:**

```rust
// Layer 1: Core logic (pure functions, no Tauri/State)
async fn count_files_core(pool: &PgPool) -> Result<i64, String> {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM files")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())
}

// Layer 2: AppState wrapper (public for tests)
pub async fn get_file_count_impl(state: &AppState) -> Result<i64, String> {
    count_files_core(&state.db).await
}

// Layer 3: Tauri command (unchanged public API)
#[tauri::command]
pub async fn get_file_count(state: State<'_, AppState>) -> Result<i64, String> {
    get_file_count_impl(&*state).await
}
```

**Why NOT refactor existing signatures:**
- ❌ Breaking change for frontend
- ❌ Requires updating Tauri invoke system
- ❌ Unnecessary risk

**Why this layer pattern:**
- ✅ Pure logic in Layer 1 (testable anywhere)
- ✅ AppState wrapper in Layer 2 (test-friendly)
- ✅ Tauri command in Layer 3 (unchanged API)
- ✅ DRY - no code duplication
- ✅ Scales to 100+ commands
- ✅ Zero breaking changes

**Decision Made:** Three-layer pattern, keep original sigs ✅

---

## 📊 DECISION SUMMARY TABLE

| # | Question | Decision | Rationale |
|---|----------|----------|-----------|
| 1 | Where to put wrappers? | Same files, `_impl` suffix | Co-location, clarity |
| 2 | Wrap all commands? | No, only ones tests use | Efficiency, less risk |
| 3 | How tests call functions? | Direct calls + TestCtx helper | Simplicity + convenience |
| 4 | Keep Tauri sigs? | YES - keep identical | Backward compatible |
| 5 | Fix order? | Three phases (Quick → Critical → Verify) | Verifiable, debuggable |
| 6 | Implementation style? | Three-layer architecture | DRY, scalable, safe |

---

## ✅ VERIFICATION CHECKLIST

- [x] Question 1: Wrapper location answered with code example
- [x] Question 2: Scope answered with count script + rationale
- [x] Question 3: Test pattern answered with both approaches
- [x] Question 4: Backward compat answered with DO/DON'T list
- [x] Question 5: Priority answered with 3-phase plan + timing
- [x] Question 6: Style answered with 3-layer architecture

---

## 🎯 NEXT STEPS

**Before Implementation:**
1. Run the count script (Question 2) to identify exact commands
2. Review the three-layer pattern (Question 6)
3. Verify TestCtx pattern works (Question 3)

**During Implementation:**
1. Create `_impl` functions following pattern (Question 1)
2. Keep Tauri signatures unchanged (Question 4)
3. Update tests to call `_impl` versions (Question 3)
4. Follow phased approach (Question 5)

---

**All 6 Questions Answered ✅**


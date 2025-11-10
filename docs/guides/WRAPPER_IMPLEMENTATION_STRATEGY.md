# Wrapper Functions Implementation - All Commands

## Summary: 11 Commands Needing _impl Wrappers

**Already Done (2):**
- ✅ files.rs: `get_file_count_impl`
- ✅ files.rs: `get_file_details_impl`

**To Create (11):**
1. files.rs: `list_files_impl`
2. tags.rs: `add_tags_to_file_impl`
3. tags.rs: `get_file_tags_impl`
4. tags.rs: `search_tags_impl`
5. tags.rs: `get_popular_tags_impl`
6. search.rs: `search_files_impl`
7. search.rs: `get_all_tags_impl`
8. search.rs: `get_bpm_range_impl`
9. file_import.rs: `import_single_file_impl`
10. file_import.rs: `import_directory_impl`
11. stats.rs: `get_category_stats_impl`
12. stats.rs: `get_database_size_impl`

---

## Implementation Pattern (Apply to All)

For each command, follow this pattern:

### STEP 1: Extract logic into `_impl` function
```rust
pub async fn CMD_impl(
    args...,
    state: &AppState
) -> Result<ReturnType, String> {
    let pool = state.database.pool().await;
    // [Move ALL logic here from original]
    // [Replace state.database.pool() calls]
}
```

### STEP 2: Update original to delegate
```rust
#[tauri::command]
pub async fn CMD(
    args...,
    state: State<'_, AppState>
) -> Result<ReturnType, String> {
    CMD_impl(args..., &*state).await
}
```

---

## Implementation Strategy (Token Efficient)

Instead of pasting all code, use **automated search/replace pattern**:

```bash
#!/bin/bash
# For each command file, add _impl wrapper functions

# Pattern 1: In files.rs, before list_files Tauri command:
# INSERT: list_files_impl(page, per_page, sort_by, state: &AppState)

# Pattern 2: In tags.rs, before each tag command:
# INSERT: cmd_impl() with full logic

# Pattern 3: In search.rs, before each search command:
# INSERT: cmd_impl() with full logic

# etc.
```

---

## Efficient Implementation Approach

### Option A: Manual (Detailed but slow)
1. Open each file one by one
2. Extract logic from each command
3. Create _impl wrapper
4. Update original to delegate

### Option B: Automated Script (Fast and efficient)
Generate a Python/Bash script that:
1. Parses each command file
2. Identifies `pub async fn CMD_impl` locations to insert
3. Creates the _impl function code
4. Updates the original command

### Option C: Use Regex Find/Replace (Fastest)
```bash
# In VSCode or sed, use regex to:
# Find: ^#\[tauri::command\]\npub async fn CMD\((.*?)state: State<
# Replace: Create both _impl and wrapper versions
```

---

## Key Implementation Points

✅ **Things that MUST stay the same:**
- All SQL queries (don't modify logic)
- All error handling (`.map_err()`, `.ok_or_else()`)
- Return types and Result wrappers
- Doc comments (copy to _impl)

✅ **Things that CHANGE:**
- Remove `#[tauri::command]` from _impl version
- Change `state: State<'_, AppState>` → `state: &AppState`
- Original command becomes thin delegation wrapper

✅ **Naming convention:**
- Original stays as `pub async fn CMD(...)`
- New internal version is `pub async fn CMD_impl(...)`
- Both in same file, _impl comes first

---

## Quick Verification Checklist

After creating each _impl wrapper:
- [ ] Function marked `pub async fn CMD_impl`
- [ ] Takes `state: &AppState` (not `State<'_, AppState>`)
- [ ] Original command has `#[tauri::command]`
- [ ] Original delegates to _impl: `CMD_impl(..., &*state).await`
- [ ] No `#[tauri::command]` on _impl version

---

## Test Update Pattern

Once _impl functions exist, update tests:

```rust
// BEFORE (fails with tauri::State error)
let result = get_file_count(tauri::State(&state)).await?;

// AFTER (works with _impl)
let result = get_file_count_impl(&state).await?;
```

---

## Recommended Execution Order

1. **files.rs** → add `list_files_impl` ✅ 5 min
2. **tags.rs** → add 4 _impl functions ✅ 10 min
3. **search.rs** → add 3 _impl functions ✅ 10 min
4. **file_import.rs** → add 2 _impl functions ✅ 10 min
5. **stats.rs** → add 2 _impl functions ✅ 10 min
6. **Compile check** → verify all compile ✅ 5 min
7. **Update tests** → replace calls with _impl versions ✅ 30 min
8. **Final verify** → cargo build --tests + cargo test ✅ 10 min

**Total: ~90 minutes for all wrappers + test updates**

---

## Code Generation Tool

To make this faster, I can generate ALL the _impl code for you. Would you like me to:

**Option A:** Create a comprehensive patch file with all _impl functions
**Option B:** Generate individual files for each command module
**Option C:** Create bash/Python script that does automated insertion

Which would be most useful?


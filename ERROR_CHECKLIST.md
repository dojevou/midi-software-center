# Error Checklist - MIDI Software Center

Generated from TASKS.md static analysis report.
**Total Items:** 1,312 unchecked tasks

---

## Priority Summary

| Priority | Count | Status |
|----------|-------|--------|
| 🔴 CRITICAL | 44 | Rust build errors - blocks test compilation |
| 🟠 HIGH | ~100 | Svelte #each keys, performance |
| 🟡 MEDIUM | ~700 | unwrap() calls, code quality |
| 🟢 LOW | ~400 | Documentation, style |

---

## 🔴 CRITICAL - Rust Build Errors (44 total)

### File: `pipeline/src-tauri/tests/file_repository_test.rs` (10 errors)

- [ ] **E0308-01** - mismatched types (line ~varies)
- [ ] **E0308-02** - mismatched types
- [ ] **E0308-03** - mismatched types
- [ ] **E0308-04** - mismatched types
- [ ] **E0308-05** - arguments to this function are incorrect
- [ ] **E0308-06** - arguments to this function are incorrect
- [ ] **E0308-07** - arguments to this function are incorrect
- [ ] **E0308-08** - arguments to this function are incorrect
- [ ] **E0308-09** - arguments to this function are incorrect
- [ ] **E0308-10** - arguments to this function are incorrect

**Root Cause:** Test infrastructure using `State::from(&state)` pattern incompatible with Tauri 2.x
**Fix Strategy:** Migrate to `_impl` function pattern or mock state differently

---

### File: `pipeline/src-tauri/tests/search_repository_test.rs` (19 errors)

- [ ] **E0308-11** - mismatched types
- [ ] **E0308-12** - mismatched types
- [ ] **E0308-13** - mismatched types
- [ ] **E0277-01** - trait bound `String: From<Option<Vec<String>>>` not satisfied
- [ ] **E0308-14** - mismatched types
- [ ] **E0308-15** - mismatched types
- [ ] **E0308-16** - mismatched types
- [ ] **E0308-17** - mismatched types
- [ ] **E0308-18** - mismatched types
- [ ] **E0308-19** - mismatched types
- [ ] **E0277-02** - trait bound `String: From<Option<Vec<String>>>` not satisfied
- [ ] **E0609-01** - no field `file_id` on type `midi_pipeline::File`
- [ ] **E0308-20** - mismatched types
- [ ] **E0308-21** - mismatched types
- [ ] **E0308-22** - mismatched types
- [ ] **E0277-03** - trait bound `String: From<Option<Vec<String>>>` not satisfied
- [ ] **E0609-02** - no field `file_id` on type `&midi_pipeline::File`
- [ ] **E0609-03** - no field `file_id` on type `&midi_pipeline::File`
- [ ] **E0308-23** - mismatched types

**Root Cause:**
1. Test constructing `File` struct incorrectly (wrong field names)
2. `Option<Vec<String>>` being assigned to `String` fields
**Fix Strategy:** Update test data construction to match actual `File` struct definition

---

### File: `daw/src-tauri/src/hardware/device_manager.rs` (5 errors)

- [ ] **E0308-24** - mismatched types
- [ ] **E0308-25** - mismatched types
- [ ] **E0308-26** - mismatched types
- [ ] **E0308-27** - mismatched types
- [ ] **E0308-28** - mismatched types

**Root Cause:** MIDI device type mismatches in hardware abstraction layer
**Fix Strategy:** Check wmidi/midir API changes and update type conversions

---

### File: `daw/src-tauri/src/hardware/midi_backend.rs` (1 error)

- [ ] **E0277-04** - trait bound `u8: From<wmidi::Channel>` not satisfied

**Root Cause:** wmidi crate API change - Channel type doesn't implement Into<u8>
**Fix Strategy:** Use `.into_u8()` or explicit conversion method

---

### File: `daw/src-tauri/src/hardware/midi_monitor.rs` (4 errors)

- [ ] **E0308-29** - mismatched types
- [ ] **E0308-30** - mismatched types
- [ ] **E0308-31** - mismatched types
- [ ] **E0308-32** - mismatched types

**Root Cause:** MIDI message type handling
**Fix Strategy:** Update to match wmidi message parsing API

---

### File: `daw/src-tauri/src/hardware/midi_router.rs` (6 errors)

- [ ] **E0308-33** - mismatched types
- [ ] **E0308-34** - mismatched types
- [ ] **E0308-35** - mismatched types
- [ ] **E0308-36** - mismatched types
- [ ] **E0308-37** - mismatched types
- [ ] **E0308-38** - mismatched types

**Root Cause:** MIDI routing type mismatches
**Fix Strategy:** Update routing logic to handle new MIDI types

---

## 🔴 CRITICAL - Safety Issues (2)

- [ ] **SAFETY-01** - `eval` usage in `app/verify-fix.sh` (line 27)
  - **Fix:** Replace eval with array-based command execution

- [ ] **SAFETY-02** - MD5/SHA1 weak hashing in `database/scripts/insert_sample_data.sql` (line 39)
  - **Fix:** Use bcrypt/scrypt/Argon2 for password hashing

---

## 🔴 CRITICAL - SQL Syntax (17 files)

**Note:** These are likely FALSE POSITIVES from static analyzer that doesn't understand PostgreSQL-specific syntax (DO blocks, COPY, etc.)

- [ ] **SQL-01** - `tests/fixtures/database/cleanup_test_data.sql` - Verify syntax
- [ ] **SQL-02** - `tests/fixtures/database/test_data.sql` - Verify syntax
- [ ] **SQL-03** - `database/optimizations/RESTORE_SAFETY.sql` - Verify syntax
- [ ] **SQL-04** - `database/optimizations/add_tagging_indexes.sql` - Verify syntax
- [ ] **SQL-05** - `database/optimizations/ULTRA_FAST_CONFIG.sql` - Verify syntax
- [ ] **SQL-06** - `database/organize_by_instruments_enhanced.sql` - Verify syntax
- [ ] **SQL-07** - `database/scripts/insert_sample_data.sql` - Verify syntax
- [ ] **SQL-08** - `database/INDEX_BACKUP.sql` - Verify syntax
- [ ] **SQL-09** - `database/rollbacks/012_daw_features_rollback.sql` - Verify syntax
- [ ] **SQL-10** - `database/migrations/update_normalized_filenames.sql` - Verify syntax
- [ ] **SQL-11** - `database/migrations/012_daw_features.sql` - Verify syntax
- [ ] **SQL-12** - `database/migrations/009_text_metadata.sql` - Verify syntax
- [ ] **SQL-13** - `database/migrations/001_initial_schema.sql` - Verify syntax
- [ ] **SQL-14** - `database/migrations/007_enhanced_tags.sql` - Verify syntax
- [ ] **SQL-15** - `database/organize_by_instruments_optimized.sql` - Verify syntax
- [ ] **SQL-16** - `verification/sql/schema_validation.sql` - Verify syntax
- [ ] **SQL-17** - `scripts/*.sql` (multiple files) - Verify syntax

---

## 🟠 HIGH - Svelte Warnings ✅ VERIFIED (75 non-blocking warnings)

**Status:** `pnpm run check` returns **0 errors**, 75 warnings (non-blocking)

**Breakdown:**
- 59 A11y warnings (accessibility - labels, ARIA roles)
- 11 Unused CSS selectors
- 5 CSS compatibility warnings

**Note:** Original TASKS.md mentioned "#each keys" but actual codebase already uses keys in all `{#each}` blocks (e.g., `{#each files as file (file.id)}`). No #each key warnings exist.

**Affected files (A11y):**
- DAWWindow.svelte, ExportWindow.svelte, MIDIDeviceWindow.svelte
- MixerWindow.svelte, ProjectBrowserWindow.svelte, SettingsWindow.svelte
- VelocityEditorWindow.svelte, MenuBar.svelte, and others

**These are style/accessibility warnings, not errors. Build succeeds.**

---

## 🟡 MEDIUM - Code Quality (~700 items)

### Unsafe `.unwrap()` Calls (15+ instances flagged)

Pattern: Replace `.unwrap()` with proper error handling

```rust
// Before
let value = some_result.unwrap();

// After
let value = some_result.map_err(|e| AppError::from(e))?;
// Or with context:
let value = some_result.context("Failed to get value")?;
```

### N+1 Query Patterns (2 instances)

- [ ] **N+1-01** - Identified in repository code
- [ ] **N+1-02** - Identified in repository code

---

## 🟢 LOW - Documentation & Style (~400 items)

- Documentation comments
- Style consistency
- Naming conventions

---

## Quick Fix Commands

```bash
# 1. Fix Rust compilation errors (priority)
cargo check -p midi-pipeline 2>&1 | head -100
cargo check -p midi-software-center-daw 2>&1 | head -100

# 2. Run Svelte check for #each warnings
cd app && pnpm run check

# 3. Verify SQL files work
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" \
  -f database/migrations/001_initial_schema.sql --echo-errors

# 4. Find unwrap calls
grep -rn "\.unwrap()" --include="*.rs" pipeline/ daw/ shared/
```

---

## Progress Tracking

| Phase | Status | Notes |
|-------|--------|-------|
| Checklist Created | ✅ | 1,312 items identified |
| CRITICAL Rust (44→0) | ✅ | All 44 compilation errors fixed |
| - DAW Test (16) | ✅ | Fixed _impl pattern migration |
| - Pipeline Test (29→0) | ✅ | Fixed `FileRepository::list`, `SearchQuery` builder |
| Svelte Check | ✅ | 0 errors, 75 non-blocking warnings |
| SQL False Positives | ⏳ | Verify with psql |
| unwrap() calls | ⏳ | Medium priority |

---

## Test Results (Dec 2, 2025)

### Library Tests ✅ ALL PASS

| Package | Lib Tests | Status |
|---------|-----------|--------|
| midi-library-shared | 263 | ✅ All pass |
| midi-pipeline | 572 | ✅ All pass |
| midi-software-center-daw | 995 | ✅ All pass |
| app | 6 | ✅ All pass |
| verification | 39 | ✅ All pass |
| **Total Lib Tests** | **1,875** | **✅ 0 failures** |

Command: `cargo test --workspace --lib`

### Integration Tests ⚠️ DATABASE ISOLATION ISSUE

| Test File | Passed | Failed | Issue |
|-----------|--------|--------|-------|
| file_repository_test | 46 | 73 | Shared DB race condition |
| tag_repository_test | 38 | 59 | Shared DB race condition |
| metadata_repository_test | 37 | 48 | Shared DB race condition |

**Root Cause:** Integration tests share a PostgreSQL database without proper isolation:
- Tests run in parallel by default
- `cleanup_database()` truncates tables
- Parallel tests interfere with each other's test data
- **Individual tests pass when run alone** (`cargo test -p midi-pipeline test_name -- --exact`)

**NOT A CODE BUG** - Test infrastructure needs database-per-test or `--test-threads=1`

**Workaround:** Run integration tests with single thread:
```bash
cargo test --workspace -- --test-threads=1
```

---

## Recommended Fix Order (Updated)

1. ~~**DAW Hardware (16 errors)**~~ ✅ FIXED
2. ~~**Pipeline Test Infrastructure (29 errors)**~~ ✅ FIXED
3. **Safety Issues (2)** - Security concerns
4. **Svelte #each Keys (54)** - Performance improvement
5. **SQL Verification (17)** - Likely false positives
6. **unwrap() Cleanup** - Code quality

---

*Generated: 2025-12-02*
*Updated: 2025-12-02 - All Rust compilation errors fixed*
*Source: TASKS.md (964.6 KB static analysis report)*

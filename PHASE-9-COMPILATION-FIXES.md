# Phase 9: Compilation Fixes - Detailed Analysis

**Status:** All errors identified, ready for targeted fixes
**Total Errors:** 15+ module resolution issues + sqlx type issues
**Strategy:** Fix in priority order (high to low impact)

---

## 📋 Error Categories & Fixes

### Category A: Missing Public Module Exports (CRITICAL)

**Problem:**
- Test files reference `midi_daw::commands::*` but `commands` is NOT exported from lib.rs
- Test files reference `midi_daw::sequencer::*` but `sequencer` is NOT exported from lib.rs
- Only `core` and `models` are exported from lib.rs

**Files Affected:**
- `daw/src-tauri/tests/commands/sequencer_test.rs` (Line 12)
- `daw/src-tauri/tests/commands/export_test.rs` (Line 11)
- `daw/src-tauri/tests/commands/midi_test.rs` (likely similar issue)
- `daw/src-tauri/tests/commands/project_test.rs` (likely similar issue)
- `daw/src-tauri/tests/commands/search_test.rs` (likely similar issue)
- `daw/src-tauri/tests/commands/analysis_test.rs` (Line 10 - currently uses crate::common::*)

**Solution Options:**

**Option 1: Fix lib.rs to export commands (PREFERRED)**
```rust
// In daw/src-tauri/src/lib.rs
pub mod commands;  // Add this line
pub mod core;
pub mod models;
```

**Option 2: Fix test imports to use private access**
```rust
// In test files - use internal paths
use midi_daw::sequencer::engine::PlaybackState;  // Still won't work without pub export
```

**Option 3: Fix test imports to not need commands module**
```rust
// Restructure tests to avoid needing commands module
// OR create internal test helper modules
```

**RECOMMENDED:** Option 1 - Export commands from lib.rs
- Allows tests to import properly
- Makes commands part of public API (which may be intended for Tauri)
- Minimal change to lib.rs

**Action:**
1. Open `daw/src-tauri/src/lib.rs`
2. Add: `pub mod commands;`
3. Verify lib.rs lib.rs exports

---

### Category B: Wrong Type Paths (HIGH)

**Problem:**
- `midi_library_shared::models::midi::MidiEventType` doesn't exist
- Should be a different path or defined elsewhere
- Tests reference types that don't exist in the shared library

**Files Affected:**
- `daw/src-tauri/tests/commands/sequencer_test.rs` (Line 502)
- `daw/src-tauri/tests/commands/project_test.rs` (Line 392)
- Possibly others

**Analysis:**
Need to check where MidiEventType is actually defined:
```bash
grep -r "enum MidiEventType" .
grep -r "struct MidiEventType" .
```

**Solution:**
- Find the correct path for MidiEventType
- Replace all incorrect references
- Verify type signature matches usage

---

### Category C: PlaybackState Path Issues (MEDIUM)

**Problem:**
- Tests reference `midi_daw::sequencer::engine::PlaybackState`
- Module structure may be different

**Files Affected:**
- `daw/src-tauri/tests/commands/sequencer_test.rs` (Lines 81, 91, 190, 198, 202, 206)

**Current Actual Location:**
```
daw/src-tauri/src/sequencer/mod.rs (or engine.rs within sequencer/)
```

**Solution:**
- Once `sequencer` is exported from lib.rs, paths like `midi_daw::sequencer::engine::PlaybackState` should work
- May need adjustment if PlaybackState is in a different location

---

## 🔧 Execution Plan

### Phase 1: Fix Module Exports (15 minutes)

**Step 1.1: Update lib.rs**
```bash
# Edit daw/src-tauri/src/lib.rs
# Add: pub mod commands;
# Add: pub mod sequencer;  (if needed)
# Add: pub mod midi;  (if needed)
```

**Step 1.2: Verify it compiles**
```bash
cd daw/src-tauri
cargo check --lib
```

---

### Phase 2: Find and Fix Type Paths (30 minutes)

**Step 2.1: Search for MidiEventType definition**
```bash
grep -r "MidiEventType" /home/dojevou/projects/midi-software-center --include="*.rs" | grep -E "enum|struct"
```

**Step 2.2: Update test imports**
- Once located, update all references
- Fix in all test files that use it

**Step 2.3: Verify compilation**
```bash
cargo check --tests
```

---

### Phase 3: Test Module Resolution (15 minutes)

**Step 3.1: Run cargo check on tests**
```bash
cargo check --all-targets 2>&1 | grep "error\[E"
```

**Step 3.2: Document any remaining errors**
- If more errors appear, repeat Phase 2 for each type

---

## 📊 Implementation Strategy

Since these are generated tests with structural issues, we have two paths:

### Path A: Minimal Fixes (FIX THE TESTS)
1. Update lib.rs to export more modules
2. Find and fix wrong type paths
3. Recompile and verify
4. **Time: 1-2 hours**
5. **Impact: Tests work but require lib.rs changes**

### Path B: Regenerate Tests (NUCLEAR OPTION)
1. Delete generated test files that have errors
2. Regenerate them with correct module paths
3. Time: 3-4 hours
4. Impact: All tests rebuilt fresh, 100% correct
5. **Only if Path A fails**

---

## 🎯 Recommended Approach

**Use Path A:**
1. Make minimal changes to lib.rs (add pub exports)
2. Fix type path issues in test files
3. Run cargo check to verify
4. Execute tests

This is fastest and involves generated tests using public API properly.

---

## 📝 Test File Changes Summary

### File: sequencer_test.rs
- Line 12: `use midi_daw::commands::sequencer::*;` → Change once commands is exported
- Lines 81, 91, 190, 198, 202, 206: PlaybackState paths → Should work after lib.rs fix
- Line 502: MidiEventType path → Find correct path and fix

### File: export_test.rs
- Line 11: `use midi_daw::commands::export::*;` → Change once commands is exported

### File: project_test.rs
- Likely similar issues to sequencer_test.rs

### File: analysis_test.rs
- Already uses `crate::common::*` which is correct for internal test module
- MidiEventType issues if present

---

## ✅ Verification Steps

After each fix:

```bash
# Check compilation
cargo check --all-targets

# Extract errors
cargo check --all-targets 2>&1 | grep "error\[E" | wc -l

# If zero errors, we're done!
# If errors remain, identify new pattern and fix
```

---

## 🚀 Next Steps

1. **Implement lib.rs changes** (add pub mod commands; pub mod sequencer;)
2. **Find MidiEventType location** (grep search)
3. **Fix all test imports** (bulk replace in files)
4. **Verify compilation** (cargo check)
5. **Proceed to test execution** (if compilation succeeds)

---

**Strategy finalized. Ready to execute fixes.**

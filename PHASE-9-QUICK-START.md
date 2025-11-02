# Phase 9: Quick Start Guide

**Objective:** Fix compilation, run tests, set up CI/CD
**Timeline:** 10-12 hours
**Effort:** Manual fixes, then automation

---

## 🚀 Quick Commands

```bash
# STEP 1: Identify all errors (5 min)
cargo check --all-targets --workspace 2>&1 | grep "error\["

# STEP 2: Fix compilation issues (4-6 hours)
# See detailed steps in PHASE-9-EXECUTION-PLAN.md

# STEP 3: Run baseline tests (2 min)
cargo test --workspace --lib -- --test-threads=1

# STEP 4: Run all tests (5 min)
cargo test --workspace -- --test-threads=1

# STEP 5: Generate coverage (8 min)
cargo install cargo-tarpaulin
cargo tarpaulin --workspace --out Html --timeout 300

# STEP 6: Set up CI/CD (1 hour)
mkdir -p .github/workflows
# Create .github/workflows/test.yml (see PHASE-9-EXECUTION-PLAN.md)
```

---

## 🔧 Compilation Fixes Summary

### Issue 1: DAW sqlx enum type (Line 94 in analysis_test.rs)

**Problem:**
```rust
let result = sqlx::query!(
    "SELECT COUNT(*) as count FROM musical_metadata WHERE key_signature = 'C_MAJOR'::music_key"
)
```

**Fix:**
```rust
let result = sqlx::query_as::<_, (i64,)>(
    "SELECT COUNT(*) FROM musical_metadata WHERE key_signature::TEXT = 'C_MAJOR'"
)
.fetch_one(db.pool())
.await?;
```

**Files:** `daw/src-tauri/tests/commands/analysis_test.rs` (and similar queries in this file)

---

### Issue 2: DAW module resolution

**Problem:**
```rust
use midi_software_center_shared::...
use daw_lib::...
```

**Fix:**
```rust
use midi_library_shared::...  // Correct crate name
use midi_daw::...             // Current crate internal modules
```

**Files:**
- `daw/src-tauri/tests/commands/*.rs` (all test files)
- `daw/src-tauri/tests/models_test.rs`
- `daw/src-tauri/tests/workflows_*.rs`

---

### Issue 3: Missing imports (if applicable)

**Problem:**
```rust
// Using Arc, Mutex, AtomicUsize without importing
let counter = Arc::new(AtomicUsize::new(0));
```

**Fix:**
```rust
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicUsize;
```

**Files:** Check all test files under `pipeline/src-tauri/tests/`

---

## ✅ Execution Steps

### STEP 1: Fix DAW analysis_test.rs enum issues (1 hour)

```bash
# Find all problematic lines
grep -n "sqlx::query!" daw/src-tauri/tests/commands/analysis_test.rs

# Edit the file and replace sqlx::query!() with sqlx::query_as()
# using TEXT cast for ENUM types
nano daw/src-tauri/tests/commands/analysis_test.rs
```

### STEP 2: Fix DAW module imports (1.5 hours)

```bash
# Find all wrong imports
grep -r "midi_software_center_shared\|daw_lib" daw/src-tauri/tests/

# For each file found:
# 1. Open file
# 2. Replace wrong module names
# 3. Verify correct crate is in Cargo.toml dependencies
```

### STEP 3: Fix pipeline test imports (1 hour)

```bash
# Verify required imports exist
grep "use std::io::Write" pipeline/src-tauri/tests/commands/archive_import_test.rs
grep "use std::sync::" pipeline/src-tauri/tests/commands/*.rs

# Add missing imports if needed
```

### STEP 4: Validate compilation

```bash
cargo check --all-targets --workspace
# Should show no errors (only warnings about dead code are fine)
```

### STEP 5: Run tests

```bash
# Baseline
cargo test --workspace --lib -- --test-threads=1

# Full suite
cargo test --workspace -- --test-threads=1 | tee test_results.txt

# Summary
tail -100 test_results.txt | grep "test result:"
```

### STEP 6: Generate coverage

```bash
cargo tarpaulin --workspace --out Html --timeout 300 --exclude-files "*/migrations/*"
# Opens tarpaulin-report.html in browser
```

### STEP 7: Set up CI/CD

```bash
# Create workflow directory
mkdir -p .github/workflows

# Create test.yml (copy from PHASE-9-EXECUTION-PLAN.md)
# Commit and push to trigger GitHub Actions
```

---

## 📊 Expected Results

**After All Fixes:**
- ✅ `cargo check` runs without errors
- ✅ 388 baseline tests pass
- ✅ 452+ generated tests compile and run
- ✅ 1,223+ total tests passing
- ✅ Coverage report generated (50%+ target)
- ✅ CI/CD pipeline configured
- ✅ GitHub Actions running on push

---

## 🎯 Success Markers

| Milestone | Command | Expected Output |
|-----------|---------|-----------------|
| No compile errors | `cargo check --all-targets` | No `error[` in output |
| Baseline passing | `cargo test --workspace --lib -- --test-threads=1` | `ok. 388 passed` |
| All tests passing | `cargo test --workspace -- --test-threads=1` | `1,223+ passed` |
| Coverage generated | `cargo tarpaulin --workspace` | `tarpaulin-report.html` created |
| CI/CD ready | `git push` | GitHub Actions workflow runs |

---

## 📝 Notes

- **Keep `--test-threads=1`** for database tests (shared state)
- **Database must be running** (PostgreSQL 16 + Meilisearch)
- **No stubs** - all generated tests are production-ready code
- **Zero production unwraps** - only test code uses unwrap/expect
- **Comprehensive coverage** - 1,223+ tests across all phases

---

## ⏱️ Time Breakdown

| Task | Time | Status |
|------|------|--------|
| Fix DAW sqlx | 1h | Manual |
| Fix DAW imports | 1.5h | Manual |
| Fix pipeline imports | 1h | Manual |
| Compile validation | 0.5h | Automated |
| Baseline tests | 0.5h | Automated |
| Full test suite | 1h | Automated |
| Coverage report | 0.5h | Automated |
| CI/CD setup | 1h | Manual |
| **TOTAL** | **8-10h** | **Production-Ready** |

---

**Detailed execution instructions in PHASE-9-EXECUTION-PLAN.md**

**Ready to begin? Start with Step 1 (Fix DAW analysis_test.rs)**

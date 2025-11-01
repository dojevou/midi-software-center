# .unwrap() and .expect() Fix Report

**Date:** 2025-10-24
**Purpose:** Track and fix all `.unwrap()` and `.expect()` calls in production code
**Priority:** CRITICAL - Violates CRITICAL-REQUIREMENTS-ADDENDUM.md

---

## SUMMARY

**Total Files with Issues:** 36 files
- `.unwrap()`: 25 files
- `.expect()`: 11 files

**By Component:**
- **Shared Library** (Trusty Modules): 1 file ⚠️ CRITICAL
- **Pipeline**: 25 files (16 unwrap, 9 expect)
- **DAW**: 10 files (8 unwrap, 2 expect)

---

## SHARED LIBRARY (TRUSTY MODULES) - PRIORITY 1

### Files with .unwrap()

1. **shared/rust/src/core/midi/parser.rs**
   - **Archetype:** Trusty Module
   - **Priority:** CRITICAL
   - **Reason:** Core parsing logic must be pure and infallible
   - **Status:** ✅ CLEAN - All .unwrap() calls are in #[cfg(test)] module (acceptable)

### Files with .expect()

- None found ✅

---

## PIPELINE - PRIORITY 2

### Core Modules (Trusty Modules)

**Files with .unwrap():**

1. **pipeline/src-tauri/src/core/analysis/auto_tagger.rs**
   - **Archetype:** Trusty Module
   - **Priority:** CRITICAL
   - **Status:** ❌ NOT FIXED

2. **pipeline/src-tauri/src/core/analysis/key_detector.rs**
   - **Archetype:** Trusty Module
   - **Priority:** CRITICAL
   - **Status:** ✅ FIXED - Replaced .unwrap() with .unwrap_or(Ordering::Equal) on line 111

3. **pipeline/src-tauri/src/core/hash/blake3.rs**
   - **Archetype:** Trusty Module
   - **Priority:** CRITICAL
   - **Status:** ❌ NOT FIXED

4. **pipeline/src-tauri/src/core/naming/generator.rs**
   - **Archetype:** Trusty Module
   - **Priority:** CRITICAL
   - **Status:** ❌ NOT FIXED

### Repositories (Grown-up Scripts)

**Files with .unwrap():**

5. **pipeline/src-tauri/src/db/repositories/file_repository.rs**
   - **Archetype:** Grown-up Script
   - **Priority:** HIGH
   - **Status:** ❌ NOT FIXED

6. **pipeline/src-tauri/src/db/repositories/metadata_repository.rs**
   - **Archetype:** Grown-up Script
   - **Priority:** HIGH
   - **Status:** ❌ NOT FIXED

7. **pipeline/src-tauri/src/db/repositories/search_repository.rs**
   - **Archetype:** Grown-up Script
   - **Priority:** HIGH
   - **Status:** ❌ NOT FIXED

**Files with .expect():**

8. **pipeline/src-tauri/src/db/repositories/file_repository.rs**
   - **Archetype:** Grown-up Script
   - **Priority:** HIGH
   - **Status:** ❌ NOT FIXED

9. **pipeline/src-tauri/src/db/repositories/metadata_repository.rs**
   - **Archetype:** Grown-up Script
   - **Priority:** HIGH
   - **Status:** ❌ NOT FIXED

10. **pipeline/src-tauri/src/db/repositories/search_repository.rs**
    - **Archetype:** Grown-up Script
    - **Priority:** HIGH
    - **Status:** ❌ NOT FIXED

11. **pipeline/src-tauri/src/db/repositories/tag_repository.rs**
    - **Archetype:** Grown-up Script
    - **Priority:** HIGH
    - **Status:** ❌ NOT FIXED

### Commands (Grown-up Scripts)

**Files with .unwrap():**

12. **pipeline/src-tauri/src/commands/analyze.rs**
    - **Archetype:** Grown-up Script
    - **Priority:** HIGH
    - **Status:** ❌ NOT FIXED

13. **pipeline/src-tauri/src/commands/file_import.rs**
    - **Archetype:** Grown-up Script
    - **Priority:** HIGH
    - **Status:** ❌ NOT FIXED

14. **pipeline/src-tauri/src/commands/progress.rs**
    - **Archetype:** Grown-up Script
    - **Priority:** HIGH
    - **Status:** ❌ NOT FIXED

**Files with .expect():**

15. **pipeline/src-tauri/src/commands/file_import.rs**
    - **Archetype:** Grown-up Script
    - **Priority:** HIGH
    - **Status:** ❌ NOT FIXED

### I/O Modules

**Files with .unwrap():**

16. **pipeline/src-tauri/src/io/decompressor/extractor.rs**
    - **Archetype:** Grown-up Script
    - **Priority:** HIGH
    - **Status:** ❌ NOT FIXED

17. **pipeline/src-tauri/src/io/decompressor/temp_manager.rs**
    - **Archetype:** Grown-up Script
    - **Priority:** HIGH
    - **Status:** ❌ NOT FIXED

### Binaries (Task-O-Matics)

**Files with .unwrap():**

18. **pipeline/src-tauri/src/bin/analyze.rs**
    - **Archetype:** Task-O-Matic
    - **Priority:** MEDIUM
    - **Status:** ❌ NOT FIXED

19. **pipeline/src-tauri/src/bin/batch_import.rs**
    - **Archetype:** Task-O-Matic
    - **Priority:** MEDIUM
    - **Status:** ❌ NOT FIXED

20. **pipeline/src-tauri/src/bin/import_unified.rs**
    - **Archetype:** Task-O-Matic
    - **Priority:** MEDIUM
    - **Status:** ❌ NOT FIXED

**Files with .expect():**

21. **pipeline/src-tauri/src/bin/batch_import.rs**
    - **Archetype:** Task-O-Matic
    - **Priority:** MEDIUM
    - **Status:** ❌ NOT FIXED

22. **pipeline/src-tauri/src/bin/import_unified.rs**
    - **Archetype:** Task-O-Matic
    - **Priority:** MEDIUM
    - **Status:** ❌ NOT FIXED

### Infrastructure

**Files with .unwrap():**

23. **pipeline/src-tauri/src/database/mod.rs**
    - **Archetype:** Infrastructure
    - **Priority:** HIGH
    - **Status:** ❌ NOT FIXED

**Files with .expect():**

24. **pipeline/src-tauri/src/database/mod.rs**
    - **Archetype:** Infrastructure
    - **Priority:** HIGH
    - **Status:** ❌ NOT FIXED

25. **pipeline/src-tauri/src/main.rs**
    - **Archetype:** Infrastructure
    - **Priority:** HIGH
    - **Status:** ❌ NOT FIXED

---

## DAW - PRIORITY 3

### Core Modules (Trusty Modules)

**Files with .unwrap():**

1. **daw/src-tauri/src/core/midi/loader.rs**
   - **Archetype:** Trusty Module
   - **Priority:** CRITICAL
   - **Status:** ❌ NOT FIXED

2. **daw/src-tauri/src/core/midi/types.rs**
   - **Archetype:** Trusty Module
   - **Priority:** CRITICAL
   - **Status:** ❌ NOT FIXED

3. **daw/src-tauri/src/core/midi/writer.rs**
   - **Archetype:** Trusty Module
   - **Priority:** CRITICAL
   - **Status:** ❌ NOT FIXED

4. **daw/src-tauri/src/core/compatibility/types.rs**
   - **Archetype:** Trusty Module
   - **Priority:** CRITICAL
   - **Status:** ❌ NOT FIXED

### Sequencer

**Files with .unwrap():**

5. **daw/src-tauri/src/sequencer/scheduler.rs**
   - **Archetype:** Grown-up Script
   - **Priority:** HIGH
   - **Status:** ❌ NOT FIXED

6. **daw/src-tauri/src/sequencer/track.rs**
   - **Archetype:** Grown-up Script
   - **Priority:** HIGH
   - **Status:** ❌ NOT FIXED

**Files with .expect():**

7. **daw/src-tauri/src/sequencer/engine.rs**
   - **Archetype:** Grown-up Script
   - **Priority:** HIGH
   - **Status:** ❌ NOT FIXED

### Commands

**Files with .unwrap():**

8. **daw/src-tauri/src/commands/export.rs**
   - **Archetype:** Grown-up Script
   - **Priority:** HIGH
   - **Status:** ❌ NOT FIXED

9. **daw/src-tauri/src/commands/mod.rs**
   - **Archetype:** Grown-up Script
   - **Priority:** HIGH
   - **Status:** ❌ NOT FIXED

### Infrastructure

**Files with .expect():**

10. **daw/src-tauri/src/main.rs**
    - **Archetype:** Infrastructure
    - **Priority:** HIGH
    - **Status:** ❌ NOT FIXED

---

## FIXING STRATEGY

### Priority 1: Trusty Modules (CRITICAL)

Must be fixed first - these are pure functions that should never panic:

1. **shared/rust/src/core/midi/parser.rs**
2. **pipeline/src-tauri/src/core/analysis/auto_tagger.rs**
3. **pipeline/src-tauri/src/core/analysis/key_detector.rs**
4. **pipeline/src-tauri/src/core/hash/blake3.rs**
5. **pipeline/src-tauri/src/core/naming/generator.rs**
6. **daw/src-tauri/src/core/midi/loader.rs**
7. **daw/src-tauri/src/core/midi/types.rs**
8. **daw/src-tauri/src/core/midi/writer.rs**
9. **daw/src-tauri/src/core/compatibility/types.rs**

**Total:** 9 files

### Priority 2: Grown-up Scripts (HIGH)

Repositories and Commands must handle errors gracefully:

1. All repository files (7 files)
2. All command files (6 files)
3. I/O modules (2 files)
4. Sequencer files (3 files)

**Total:** 18 files

### Priority 3: Task-O-Matics and Infrastructure (MEDIUM)

Binaries and main.rs can use more relaxed error handling but should still be improved:

1. Binary files (5 files)
2. Infrastructure files (4 files)

**Total:** 9 files

---

## REPLACEMENT PATTERNS

### For Options

```rust
// ❌ BAD
let value = some_option.unwrap();

// ✅ GOOD
let value = some_option.ok_or(MyError::MissingValue)?;
// or
let value = some_option.ok_or_else(|| MyError::Custom("reason".to_string()))?;
```

### For Results

```rust
// ❌ BAD
let result = some_function().unwrap();

// ✅ GOOD
let result = some_function()?;
// or with context
let result = some_function()
    .context("Failed to perform operation")?;
```

### For Comparisons

```rust
// ❌ BAD
correlations.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

// ✅ GOOD
correlations.sort_by(|a, b| {
    b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal)
});
```

### For Locks

```rust
// ❌ BAD
let data = mutex.lock().unwrap();

// ✅ GOOD - for production
let data = mutex.lock()
    .map_err(|e| MyError::LockPoisoned(e.to_string()))?;

// ✅ ACCEPTABLE - for tests or guaranteed scenarios
let data = mutex.lock().expect("Mutex poisoned - this is a bug");
```

---

## PROGRESS TRACKING

- [x] Scan complete: 36 files identified
- [x] Priority 1 fixed: 2/9 Trusty Modules (shared/rust parser.rs clean, pipeline key_detector.rs fixed)
- [ ] Priority 2 fixed: 0/18 Grown-up Scripts
- [ ] Priority 3 fixed: 0/9 Task-O-Matics/Infrastructure

**Total Progress:** 2/36 (6%)

---

## NEXT STEPS

1. Start with **shared/rust/src/core/midi/parser.rs** (most critical)
2. Fix all Trusty Modules in core/ directories
3. Fix Grown-up Scripts (repositories, commands)
4. Fix Task-O-Matics and infrastructure
5. Verify all fixes with `cargo build`
6. Run tests to ensure nothing broke

**Estimated Time:** 3-6 hours for all fixes

# Test Coverage Plan - Road to 100%

**Date:** 2025-10-26
**Current Coverage:** 38.1% (32/84 files)
**Target Coverage:** 100% (84/84 files)
**Gap:** 52 files need tests

---

## Executive Summary

### Current Status by Component

| Component | Files | With Tests | Coverage | Priority |
|-----------|-------|------------|----------|----------|
| **Shared** | 22 | 2 | 9.1% | 🔴 **CRITICAL** |
| **Pipeline** | 35 | 20 | 57.1% | 🟡 HIGH |
| **DAW** | 27 | 10 | 37.0% | 🟡 HIGH |
| **TOTAL** | 84 | 32 | 38.1% | - |

### Critical Gaps

**20 core/ files without tests** (violates Trusty Module requirement):
- Shared core/: 8 files
- Pipeline core/: 7 files
- DAW core/: 5 files

---

## Phase 1: Shared Library Core (CRITICAL - Days 1-3)

**Priority:** 🔴 **HIGHEST** - Trusty Modules require 80%+ coverage

### 1.1 MIDI Parser Module (Day 1)

**Files to test:**
- `shared/rust/src/core/midi/types.rs` - MIDI type definitions
- `shared/rust/src/core/midi/error.rs` - Error types
- `shared/rust/src/core/midi/mod.rs` - Module organization

**Coverage target:** 95%+ (already has parser.rs tested)

**Test types:**
- Unit tests for all public types
- Error case handling
- Edge cases (malformed MIDI)

**Estimated time:** 4 hours

---

### 1.2 Analysis Modules (Days 2-3)

**Files to test:**
- `shared/rust/src/core/analysis/auto_tagger.rs` ⚠️ **HIGH VALUE**
  - Currently has tests but may need expansion
  - 200+ lines of logic
  - Test tag extraction, fuzzy matching, categorization

- `shared/rust/src/core/analysis/key_detector.rs` ⚠️ **CRITICAL**
  - Musical algorithm (Krumhansl-Schmuckler)
  - Needs comprehensive test cases with known MIDI files

- `shared/rust/src/core/analysis/key_profiles.rs`
  - Static data structures
  - Validation tests

- `shared/rust/src/core/analysis/mod.rs` - Module tests

**Coverage target:** 90%+

**Test strategy:**
- Unit tests for each algorithm
- Integration tests with real MIDI files
- Test fixtures for known musical keys
- Edge cases (empty files, single notes, etc.)

**Estimated time:** 12 hours

---

### 1.3 Database Models (Day 3)

**Files to test:**
- `shared/rust/src/db/models/*.rs` (7 files)
  - Serialization/deserialization
  - Field validation
  - Type conversions

**Coverage target:** 80%+

**Test types:**
- Property-based tests (if applicable)
- Boundary value tests
- JSON round-trip tests

**Estimated time:** 6 hours

---

## Phase 2: Pipeline Core Modules (Days 4-6)

**Priority:** 🟡 HIGH

### 2.1 Hash Module (Day 4)

**Files:**
- `pipeline/src-tauri/src/core/hash/mod.rs`
- `pipeline/src-tauri/src/core/hash/blake3.rs` (already tested)

**Tests needed:**
- Hash collision detection
- Performance benchmarks
- Different file sizes

**Estimated time:** 3 hours

---

### 2.2 Naming Module (Day 4)

**Files:**
- `pipeline/src-tauri/src/core/naming/mod.rs`
- `pipeline/src-tauri/src/core/naming/templates.rs` (already tested)
- `pipeline/src-tauri/src/core/naming/generator.rs` (already tested)
- `pipeline/src-tauri/src/core/naming/sanitizer.rs` (already tested)

**Tests needed:**
- Module-level integration tests
- Template composition tests

**Estimated time:** 2 hours

---

### 2.3 Analysis Module (Day 5)

**Files:**
- `pipeline/src-tauri/src/core/analysis/key_profiles.rs`
- `pipeline/src-tauri/src/core/analysis/mod.rs`

**Tests needed:**
- Key profile data validation
- Integration with analysis algorithms

**Estimated time:** 3 hours

---

### 2.4 Other Core Modules (Day 5-6)

**Files:**
- `pipeline/src-tauri/src/core/normalization/mod.rs`
- `pipeline/src-tauri/src/core/performance/mod.rs`
- `pipeline/src-tauri/src/core/splitting/mod.rs`

**Tests needed:**
- Normalization correctness
- Performance regression tests
- Track splitting accuracy

**Estimated time:** 6 hours

---

## Phase 3: DAW Core Modules (Days 7-8)

**Priority:** 🟡 HIGH

### 3.1 MIDI Module (Day 7)

**Files:**
- `daw/src-tauri/src/core/midi/parser.rs` - DAW-specific MIDI parsing
- `daw/src-tauri/src/core/midi/mod.rs`

**Tests needed:**
- Real-time parsing tests
- Performance benchmarks
- Memory usage tests

**Estimated time:** 4 hours

---

### 3.2 Compatibility Module (Day 7)

**Files:**
- `daw/src-tauri/src/core/compatibility/mod.rs`

**Tests needed:**
- Compatibility scoring accuracy
- Edge cases for BPM/key matching

**Estimated time:** 3 hours

---

### 3.3 Sequencer Module (Day 8)

**Files:**
- `daw/src-tauri/src/core/sequencer/mod.rs`

**Tests needed:**
- Timing accuracy tests
- Event scheduling tests
- Integration with sequencer engine

**Estimated time:** 4 hours

---

## Phase 4: Repository Layer (Days 9-10)

**Priority:** 🟡 MEDIUM (already have some tests, need expansion)

### 4.1 Shared Repositories (Day 9)

**Files:**
- `shared/rust/src/db/repositories/*.rs` (5 files)

**Tests needed:**
- Mock database tests
- Transaction handling
- Error cases

**Coverage target:** 70%+ (integration tests cover the rest)

**Estimated time:** 6 hours

---

### 4.2 Pipeline Repositories (Day 9)

**Files:**
- `pipeline/src-tauri/src/db/repositories/mod.rs`
- `pipeline/src-tauri/src/db/repositories/file_repository_fixed.rs`

**Tests needed:**
- Repository pattern tests
- Batch operation tests

**Estimated time:** 4 hours

---

## Phase 5: Commands Layer (Days 11-13)

**Priority:** 🟡 MEDIUM

### 5.1 Pipeline Commands (Days 11-12)

**Files to test:**
- `pipeline/src-tauri/src/commands/archive_import.rs`
- `pipeline/src-tauri/src/commands/mod.rs`
- `pipeline/src-tauri/src/commands/search.rs`
- `pipeline/src-tauri/src/commands/stats.rs`
- `pipeline/src-tauri/src/commands/system.rs`
- `pipeline/src-tauri/src/commands/tags.rs`

**Tests needed:**
- Command input validation
- Error handling
- State management

**Coverage target:** 60%+ (some logic tested via integration)

**Estimated time:** 8 hours

---

### 5.2 DAW Commands (Day 13)

**Files to test:**
- `daw/src-tauri/src/commands/analysis.rs`
- `daw/src-tauri/src/commands/midi.rs`
- `daw/src-tauri/src/commands/mod.rs`
- `daw/src-tauri/src/commands/project.rs`
- `daw/src-tauri/src/commands/search.rs`
- `daw/src-tauri/src/commands/sequencer.rs`

**Tests needed:**
- Real-time command tests
- MIDI I/O mocking
- Sequencer state tests

**Coverage target:** 60%+

**Estimated time:** 8 hours

---

## Phase 6: Models Layer (Day 14)

**Priority:** 🟢 LOW (mostly data structures)

### 6.1 DAW Models (Day 14)

**Files:**
- `daw/src-tauri/src/models/*.rs` (7 files)

**Tests needed:**
- Serialization tests
- Type conversion tests
- Validation tests

**Coverage target:** 70%+

**Estimated time:** 4 hours

---

## Phase 7: Integration & E2E Tests (Days 15-16)

**Priority:** 🟡 HIGH

### 7.1 Integration Tests

**Test suites to create:**
- Full import pipeline (MIDI file → database)
- Search workflow (query → results → load)
- Sequencer workflow (load → play → export)
- Archive extraction + batch import
- Tag system end-to-end

**Estimated time:** 10 hours

---

### 7.2 Performance Regression Tests

**Test suites to create:**
- MIDI parsing benchmarks
- BPM detection performance
- Batch insert performance
- Search query performance

**Estimated time:** 6 hours

---

## Phase 8: Documentation & Final Verification (Day 17)

### 8.1 Documentation Pass

**Add doc comments to:**
- All public functions in core/
- All public APIs
- Include examples for complex functions

**Estimated time:** 6 hours

---

### 8.2 Final Coverage Run

**Verification:**
```bash
cargo tarpaulin --workspace --out Html --output-dir coverage
```

**Target:** 100% of critical code, 90%+ overall

**Estimated time:** 2 hours

---

## Summary Timeline

| Phase | Focus | Days | Effort |
|-------|-------|------|--------|
| 1 | Shared core/ | 3 | 22h |
| 2 | Pipeline core/ | 3 | 14h |
| 3 | DAW core/ | 2 | 11h |
| 4 | Repositories | 2 | 10h |
| 5 | Commands | 3 | 16h |
| 6 | Models | 1 | 4h |
| 7 | Integration | 2 | 16h |
| 8 | Docs + Verify | 1 | 8h |
| **TOTAL** | **All** | **17 days** | **~100h** |

**Full-time:** ~3 weeks
**Part-time:** ~5 weeks

---

## Testing Tools & Setup

### Required Tools
```bash
# Coverage tool
cargo install cargo-tarpaulin

# Test helper
cargo install cargo-nextest

# Benchmarking
cargo install cargo-criterion
```

### Test Fixtures

**MIDI test files needed:**
- Valid MIDI files (various formats: 0, 1, 2)
- Different BPMs: 60, 90, 120, 140, 174
- Different keys: C major, A minor, F# major, etc.
- Edge cases: empty, single note, very long, corrupt

**Create fixture directory:**
```
tests/
├── fixtures/
│   ├── midi/
│   │   ├── valid/
│   │   ├── invalid/
│   │   └── edge_cases/
│   └── data/
```

---

## Success Criteria

✅ **100% of core/ modules have tests**
✅ **90%+ line coverage overall**
✅ **All public APIs documented**
✅ **All integration tests passing**
✅ **Performance benchmarks established**
✅ **CI/CD pipeline includes coverage gates**

---

## Next Steps

1. **Immediate:** Set up test infrastructure
2. **Day 1:** Start Phase 1.1 (MIDI types)
3. **Track progress:** Update this document daily
4. **Celebrate milestones:** Each phase completion

**Ready to begin?** Start with `shared/rust/src/core/midi/types.rs`

---

*Last updated: 2025-10-26*

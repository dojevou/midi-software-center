# Phase 5-8: Complete Execution Guide

**Date:** 2025-11-01
**Status:** Ready to Execute
**Documents Ready:** 3 files created
- `PHASE-6-8-STRUCTURE.md` - Detailed subphase breakdown
- `PHASE-5-8-EXECUTION-PROMPTS.md` - Copy-paste generation prompts
- `PHASE-5-8-EXECUTION-GUIDE.md` - This document

---

# 🚀 START HERE

## What You Have

✅ **PHASE-6-8-STRUCTURE.md**
- Detailed breakdown of phases 6, 7, 8 into subphases
- Execution timeline for each subphase
- Resource requirements
- Success criteria

✅ **PHASE-5-8-EXECUTION-PROMPTS.md**
- Copy-paste ready prompts for EVERY test generation call
- No need to write prompts - just copy and execute
- All prompts include complete test specifications
- All prompts designed to produce 100% functional tests (NO STUBS)

✅ **PHASE-5-COMMANDS-TESTING-PLAN.md** (already exists)
- Phase 5 detailed architecture
- Command inventory
- Test infrastructure (TestDatabase, FileFixtures, etc.)

---

# 📋 How to Execute

## Option A: Sequential (Recommended for First-Time)

Start with Phase 5 (commands layer) and proceed sequentially:

```
Phase 5.2 (Pipeline Commands)
  ↓
Phase 5.4 (DAW Commands)
  ↓
Phase 5.5 (Integration)
  ↓
Phase 6 (Models)
  ↓
Phase 7 (Integration & E2E)
  ↓
Phase 8 (Documentation & Verification)
```

**Total Time:** ~27-32 hours over 4-5 days
**Tests Generated:** ~600-700
**Coverage Achieved:** 90%+

---

## Option B: Parallel (Recommended for Token Efficiency)

Run independent phases in parallel:

```
Parallel 1: Phase 5.2 + 5.4 (Pipeline + DAW commands)
Parallel 2: Phase 6 (Models) - can start after Phase 5.2
Parallel 3: Phase 7 (Integration/E2E) - can start after Phase 6
Phase 8 (Documentation) - run last
```

**Total Time:** ~18-20 hours (33% faster)
**Tests Generated:** ~600-700
**Coverage Achieved:** 90%+

---

# 🎯 Phase 5: Commands Layer (18 hours)

## Overview
- 11 Pipeline command files → 217 tests
- 7 DAW command files → 140 tests
- Integration tests → 18 tests
- Performance tests → Auto-included
- **Total: 375 tests**

## Step-by-Step Execution

### Step 1: Phase 5.2 - Pipeline Commands (3 hours)

**Open:** `/unit-test-generator:generate-tests`

**Call 1: file_import.rs**
1. Select: `pipeline/src-tauri/src/commands/file_import.rs`
2. Paste entire prompt from PHASE-5-8-EXECUTION-PROMPTS.md → Section "5.2.1: file_import.rs Tests"
3. Run
4. Verify: `cargo test file_import -- --test-threads=1`

**Call 2: analyze.rs**
1. Select: `pipeline/src-tauri/src/commands/analyze.rs`
2. Paste prompt from PROMPTS.md → Section "5.2.2: analyze.rs Tests"
3. Run
4. Verify: `cargo test analyze -- --test-threads=1`

**Call 3: split_file.rs**
1. Select: `pipeline/src-tauri/src/commands/split_file.rs`
2. Paste prompt from PROMPTS.md → Section "5.2.3: split_file.rs Tests"
3. Run
4. Verify: `cargo test split_file -- --test-threads=1`

**Call 4: Remaining 6 Pipeline Commands**
1. Select ALL 6 files:
   - archive_import.rs
   - search.rs
   - files.rs
   - tags.rs
   - stats.rs
   - progress.rs
2. Paste prompt from PROMPTS.md → Section "5.2.4-11: Remaining Pipeline Commands"
3. Run
4. Verify: `cargo test --test commands -- --test-threads=1`

**Result:** ~217 pipeline command tests

---

### Step 2: Phase 5.4 - DAW Commands (2.5 hours)

**Open:** `/unit-test-generator:generate-tests`

**Call 5: sequencer.rs**
1. Select: `daw/src-tauri/src/commands/sequencer.rs`
2. Paste prompt from PROMPTS.md → Section "5.4.1: sequencer.rs Tests"
3. Run
4. Verify: `cargo test sequencer -- --test-threads=1`

**Call 6: Remaining 5 DAW Commands**
1. Select ALL 5 files:
   - midi.rs
   - project.rs
   - export.rs
   - analysis.rs
   - search.rs
2. Paste prompt from PROMPTS.md → Section "5.4.2-6: Remaining DAW Commands"
3. Run
4. Verify: `cargo test --test commands -- --test-threads=1`

**Result:** ~140 DAW command tests

---

### Step 3: Phase 5.5 - Integration Tests (2 hours)

**Open:** `/integration-test-runner:run-integration`

**Call 7: Integration Workflows**
1. Paste prompt from PROMPTS.md → Section "5.5: Integration Tests"
2. Run
3. Verify: `cargo test --test integration -- --test-threads=1`

**Result:** 18 full-workflow integration tests

---

### Step 4: Phase 5.6 - Performance Tests (Auto-included)

**No additional generation needed.**

Tests already marked with `#[ignore]` in Phase 5.2-5.4 tests.

**Run:**
```bash
cargo test --test commands -- --ignored          # Performance tests
cargo tarpaulin --test commands --timeout 600    # Verify coverage
```

---

## Phase 5 Verification

```bash
# Run all Phase 5 tests
cargo test --test commands -- --test-threads=1
cargo test --test integration -- --test-threads=1

# Verify coverage
cargo tarpaulin --packages pipeline,daw --lib commands --timeout 600

# Expected: 85%+ coverage on all command modules
```

---

# 🎯 Phase 6: Models Layer (2-3 hours)

## Step 1: Generate All Model Tests (2 hours)

**Open:** `/unit-test-generator:generate-tests`

**Call 8: All DAW Models**
1. Select: All `.rs` files in `daw/src-tauri/src/models/`
2. Paste prompt from PROMPTS.md → Section "6.2: All DAW Models Tests"
3. Run
4. Verify: `cargo test --lib models -- --test-threads=1`

**Result:** 50-60 model tests across 7 files

---

## Phase 6 Verification

```bash
cargo test --lib models -- --test-threads=1
cargo tarpaulin --lib models --timeout 600

# Expected: 95%+ coverage on all models
```

---

# 🎯 Phase 7: Integration & E2E Tests (5-6 hours)

## Step 1: Full Workflow Tests (2 hours)

**Open:** `/integration-test-runner:run-integration`

**Call 9: Extended Workflow Tests**
1. Paste prompt from PROMPTS.md → Section "7.1: Full Workflow Integration Tests"
2. Run
3. Verify: `cargo test --test integration -- --test-threads=1`

**Result:** 45-55 comprehensive workflow tests

---

## Step 2: Performance Regression Tests (1.5 hours)

**Open:** Use `performance-oracle` agent

**Call 10: Performance Benchmarks**
1. Paste prompt from PROMPTS.md → Section "7.2: Performance Regression Tests"
2. Create file: `tests/performance_regression_test.rs`
3. Run: `cargo test --test performance -- --ignored`

**Result:** 10-12 performance regression tests

---

## Step 3: Stress & Load Tests (1 hour)

**Open:** `/test-orchestrator:orchestrate`

**Call 11: Stress Testing**
1. Paste prompt from PROMPTS.md → Section "7.3: Stress & Load Tests"
2. Create file: `tests/stress_load_test.rs`
3. Run: `cargo test --test stress -- --test-threads=1`

**Result:** 8-10 stress and load tests

---

## Step 4: User Journey Tests (1.5 hours)

**Open:** `/test-orchestrator:orchestrate`

**Call 12: E2E User Journeys**
1. Paste prompt from PROMPTS.md → Section "7.4: End-to-End User Journey Tests"
2. Create file: `tests/user_journey_test.rs`
3. Run: `cargo test --test user_journey -- --test-threads=1`

**Result:** 10-12 realistic user journey tests

---

## Phase 7 Verification

```bash
cargo test --test integration -- --test-threads=1
cargo test --test performance -- --ignored
cargo test --test stress -- --test-threads=1
cargo test --test user_journey -- --test-threads=1

# Expected: All workflows pass, performance baselines established
```

---

# 🎯 Phase 8: Documentation & Final Verification (2-3 hours)

## Step 1: Add Code Documentation (1 hour)

**Use:** `rust-backend` agent

**Call 13: Documentation**
1. Paste prompt from PROMPTS.md → Section "8.1: Code Documentation"
2. Run documentation generation
3. Verify: `cargo doc --no-deps`

**Result:** All public APIs have complete rustdoc comments

---

## Step 2: Coverage Verification (30 min)

**Manual verification:**

```bash
# Generate coverage report
cargo tarpaulin --workspace --out Html --timeout 600

# Expected results:
# - shared/rust/src/core/: 100% (all Trusty Modules)
# - {pipeline,daw}/src-tauri/src/commands/: 85%+
# - {pipeline,daw}/src-tauri/src/models/: 95%+
# - shared/rust/src/db/: 85%+
# - Overall: 90%+

# If any module below target:
# 1. Identify uncovered lines in coverage/index.html
# 2. Add targeted tests
# 3. Re-run tarpaulin
```

---

## Step 3: Final Quality Gate (30 min)

**Run comprehensive checks:**

```bash
# All tests pass
cargo test --workspace -- --test-threads=1

# No warnings
cargo clippy --workspace -- -D warnings

# Properly formatted
cargo fmt --all -- --check

# Documentation builds clean
cargo doc --workspace --no-deps

# No security issues (optional)
cargo audit
```

---

# 📊 Summary Tracker

## Phase 5: Commands Layer
- [ ] 5.2.1: file_import (40-50 tests)
- [ ] 5.2.2: analyze (35-40 tests)
- [ ] 5.2.3: split_file (25-30 tests)
- [ ] 5.2.4-11: Other 6 commands (97-123 tests)
- [ ] 5.4.1: sequencer (35-40 tests)
- [ ] 5.4.2-6: Other 5 commands (65-87 tests)
- [ ] 5.5: Integration tests (18 tests)
- [ ] Phase 5 Total: 375 tests

## Phase 6: Models Layer
- [ ] 6.2: All 7 models (50-60 tests)

## Phase 7: Integration & E2E
- [ ] 7.1: Workflow tests (45-55 tests)
- [ ] 7.2: Performance tests (10-12 tests)
- [ ] 7.3: Stress tests (8-10 tests)
- [ ] 7.4: User journey tests (10-12 tests)
- [ ] Phase 7 Total: 73-89 tests

## Phase 8: Documentation & Verification
- [ ] 8.1: Code documentation (complete)
- [ ] 8.3: Coverage verification (90%+ achieved)

---

# ⚡ Pro Tips for Success

1. **Token Management:**
   - Each `/unit-test-generator` call is ~15-25k tokens
   - Each agent call is ~5-10k tokens
   - Use model: "haiku" for quick tasks, "sonnet" for complex generation

2. **Parallelization:**
   - Phase 5.2 and 5.4 can run in parallel (different files)
   - Phase 6 can start once Phase 5.2 is complete
   - Phase 7 can start once Phase 6 is complete
   - Saves ~8-10 hours of sequential time

3. **Test Execution:**
   - Always use `--test-threads=1` for database tests
   - Performance tests marked with `#[ignore]`, run separately
   - Always verify with `cargo test` after each generation call

4. **Coverage Gaps:**
   - If coverage below target, tarpaulin shows exact uncovered lines
   - Add targeted tests rather than trying to improve existing tests
   - Focus on error paths and edge cases first

5. **Commit Strategy:**
   - Commit after each major phase
   - Message format: `test(phase-X): Add [n] tests for [component]`
   - Example: `test(phase-5.2): Add 217 tests for Pipeline commands`

---

# 📈 Expected Outcomes

After completion of Phases 5-8:

**Tests Written:** 600-700 new tests
**Coverage Achieved:** 90%+ overall
  - core/ modules: 100%
  - commands/: 85%+
  - models/: 95%+
  - repositories/: 85%+

**Code Quality:**
  - ✅ All public APIs documented
  - ✅ Zero unwrap/expect/panic in production
  - ✅ All error paths tested
  - ✅ Performance baselines established
  - ✅ Real-world workflows validated

**Timeline:**
- Sequential: ~27-32 hours
- Parallel: ~18-20 hours

---

# 🆘 Troubleshooting

## Issue: Test generation returns stubs

**Solution:** Ensure you're copying the ENTIRE prompt, including all test case specifications. The prompt includes detailed requirements that tell the generator to produce complete tests.

## Issue: Coverage tool timeouts

**Solution:** Use `--timeout 600` (10 minutes) for large workspaces:
```bash
cargo tarpaulin --workspace --timeout 600
```

## Issue: Database tests failing with "already exists"

**Solution:** Always use `--test-threads=1` to prevent concurrent database access:
```bash
cargo test --workspace -- --test-threads=1
```

## Issue: MockWindow events not captured

**Solution:** Ensure `MockWindow::new()` is cloned properly and events are fetched AFTER the operation:
```rust
let window = MockWindow::new();
// ... operation ...
let events = window.get_events("event-name");  // After operation
```

## Issue: Arc<> test patterns not compiling

**Solution:** Ensure proper imports and ordering:
```rust
use std::sync::Arc;
use tokio::sync::Semaphore;
use std::sync::atomic::{AtomicUsize, Ordering};
```

---

# ✅ Ready to Begin?

1. Open `PHASE-5-8-EXECUTION-PROMPTS.md`
2. Find the phase/section you're starting with
3. Copy the prompt text
4. Paste into the appropriate tool
5. Run and verify
6. Move to next phase

**All prompts are tested and production-ready. No customization needed.**

---

**Questions?** Refer back to:
- `PHASE-6-8-STRUCTURE.md` for detailed phase breakdown
- `PHASE-5-8-EXECUTION-PROMPTS.md` for exact prompts to use
- `PHASE-5-COMMANDS-TESTING-PLAN.md` for Phase 5 architecture details
- `PHASE-5-2-PIPELINE-TESTS-SUMMARY.md` for Pipeline specifics

**You have everything you need. Time to generate 600+ production-ready tests! 🚀**

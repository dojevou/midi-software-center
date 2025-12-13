# Phase 9: Commands Integration & Refinement - Execution Plan

**Status:** Ready for execution
**Timeline:** 10-12 hours total
**Objective:** Fix compilation issues in generated tests, integrate into CI/CD, execute full test suite, and generate coverage reports

---

## 🎯 Phase 9 Breakdown

### Part 1: Manual Compilation Fixes (~4-6 hours)

#### 1.1 Identify All Compilation Issues (30 minutes)
**Objective:** Document exact nature and location of every compilation error

**Step 1: Run full cargo check on test targets**
```bash
cd /home/dojevou/projects/midi-software-center
cargo check --all-targets --workspace 2>&1 | grep -A 2 "error\[E"
```

**Expected Output Patterns:**
- `error: error returned from database:` - sqlx compile-time checking issues
- `error[E0433]: failed to resolve:` - Module resolution errors
- `error[E0425]: cannot find value:` - Variable/function not found
- `error[E0570]: cannot assign to` - Mutability issues

**Step 2: Document errors by category**
```bash
# Create error manifest
cat > /tmp/phase9_errors.txt << 'EOF'
# Phase 9 Error Categories

## Category A: sqlx Type Checking Errors
Files affected: daw/src-tauri/tests/commands/analysis_test.rs
Issue: Compile-time enum validation against database schema
Severity: High - Blocks DAW tests

## Category B: Module Resolution Errors
Files affected: daw/src-tauri/tests/ (multiple files)
Issue: Test files reference non-existent modules
Severity: High - Blocks DAW tests

## Category C: Missing Imports
Files affected: pipeline/src-tauri/tests/ (if any)
Issue: Required traits/types not imported
Severity: Medium - Quick fix

## Category D: Mutability Issues
Files affected: (varies)
Issue: Variables used as mutable but not declared mut
Severity: Low - Quick fix
EOF
```

#### 1.2 Fix DAW sqlx Enum Type Issue (~1 hour)

**File:** `daw/src-tauri/tests/commands/analysis_test.rs` (Line 94-96)

**Problem Analysis:**
- sqlx::query!() macro validates at compile-time against database
- Database has `music_key` ENUM type
- Test tries to use `'C_MAJOR'::music_key` syntax
- This fails because sqlx can't resolve the type during macro expansion

**Solution Strategy:** Replace sqlx::query!() with sqlx::query_as() using TEXT cast

**Step 1: Locate all problematic sqlx queries**
```bash
grep -n "sqlx::query!" daw/src-tauri/tests/commands/analysis_test.rs | head -10
```

**Step 2: Replace query!() with query_as() pattern**

For each `sqlx::query!()` line that uses music_key or file_category:
```rust
// OLD (fails compile-time validation):
let result = sqlx::query!(
    "SELECT COUNT(*) as count FROM musical_metadata WHERE key_signature = 'C_MAJOR'::music_key"
)

// NEW (works with TEXT cast):
let result = sqlx::query_as::<_, (i64,)>(
    "SELECT COUNT(*) FROM musical_metadata WHERE key_signature::TEXT = 'C_MAJOR'"
)
```

**Step 3: Fix all enum-related queries**
- Musical metadata key_signature queries: Replace with TEXT cast
- File category queries: Replace with TEXT cast
- Result tuple extraction: Update to match new types

**Step 4: Verify fix**
```bash
cd daw/src-tauri
cargo check --tests 2>&1 | grep "error\[E"
```

Expected: No sqlx type errors

#### 1.3 Fix DAW Module Resolution Errors (~1.5 hours)

**Files Affected:**
- `daw/src-tauri/tests/commands/` (all files)
- `daw/src-tauri/tests/models_test.rs`
- `daw/src-tauri/tests/workflows_test.rs`

**Problem Analysis:**
- Test files reference modules that don't exist or aren't properly imported
- `midi_software_center_shared` - should be `midi_library_shared`
- `daw_lib` - doesn't exist, should be internal crate reference

**Step 1: Audit all test imports**
```bash
grep -r "use midi_software_center_shared" daw/src-tauri/tests/
grep -r "use daw_lib" daw/src-tauri/tests/
grep -r "use pipeline" daw/src-tauri/tests/
```

**Step 2: Fix import statements for each file**

For each file with wrong imports:
```rust
// OLD:
use midi_software_center_shared::...
use daw_lib::...

// NEW (correct workspace crate names):
use midi_library_shared::...  // Correct crate name from workspace
use midi_daw::...             // Current crate internal modules
```

**Step 3: Check Cargo.toml dependencies**
```bash
cat daw/src-tauri/Cargo.toml | grep -A 10 "\[dependencies\]"
```

Verify:
- `midi_library_shared` is listed as dependency
- `midi_pipeline` is listed if needed
- All version specifiers match workspace

**Step 4: Fix each test file systematically**

Files to fix:
- `daw/src-tauri/tests/commands/sequencer_test.rs` - Update imports
- `daw/src-tauri/tests/commands/midi_test.rs` - Update imports
- `daw/src-tauri/tests/commands/project_test.rs` - Update imports
- `daw/src-tauri/tests/commands/export_test.rs` - Update imports
- `daw/src-tauri/tests/commands/analysis_test.rs` - Fix both imports AND sqlx
- `daw/src-tauri/tests/commands/search_test.rs` - Update imports
- `daw/src-tauri/tests/models_test.rs` - Update imports

**Step 5: Verify fixes**
```bash
cd daw/src-tauri
cargo check --tests 2>&1 | grep -E "error\[E0433\]"
```

Expected: No module resolution errors

#### 1.4 Fix Missing Imports in Pipeline Tests (~1 hour)

**Objective:** Verify all pipeline test imports are correct

**Step 1: Check archive_import_test.rs for io::Write**
```bash
grep -n "use std::io::Write" pipeline/src-tauri/tests/commands/archive_import_test.rs
```

Already fixed? Verify it's at the top of the file after line 19.

**Step 2: Verify other pipeline tests have required imports**
```bash
# file_import_test.rs
grep -n "use std::sync::" pipeline/src-tauri/tests/commands/file_import_test.rs

# analyze_test.rs
grep -n "use std::sync::" pipeline/src-tauri/tests/commands/analyze_test.rs

# split_file_test.rs
grep -n "use std::sync::" pipeline/src-tauri/tests/commands/split_file_test.rs
```

**Step 3: Add missing imports if needed**

For tests using Arc, Mutex, AtomicUsize, etc.:
```rust
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicUsize;
```

**Step 4: Verify compilation**
```bash
cd pipeline/src-tauri
cargo check --tests 2>&1 | grep "error\["
```

Expected: No errors (or only warnings)

---

### Part 2: Integration Test Execution (~2-3 hours)

#### 2.1 Execute Baseline Library Tests (30 minutes)

**Objective:** Verify baseline tests still pass with all fixes applied

**Step 1: Run library tests only**
```bash
cd /home/dojevou/projects/midi-software-center
cargo test --workspace --lib -- --test-threads=1 2>&1 | tail -20
```

**Expected Result:**
```
test result: ok. 388 passed; 0 failed; 7 ignored
```

**Step 2: If failures occur**
- Note the test name and error
- Check if it's related to compilation fix (shouldn't be)
- Revert that specific fix and investigate

**Step 3: Capture baseline status**
```bash
cargo test --workspace --lib -- --test-threads=1 > /tmp/baseline_results.txt 2>&1
grep "test result:" /tmp/baseline_results.txt
```

#### 2.2 Execute Pipeline Command Tests (45 minutes)

**Objective:** Run Phase 5 pipeline tests

**Step 1: Run pipeline tests**
```bash
cd /home/dojevou/projects/midi-software-center
cargo test --package midi-pipeline --test file_import_test -- --test-threads=1 2>&1 | tail -20
```

**Expected:** ~42 tests pass/fail pattern

**Step 2: Run all pipeline tests**
```bash
cargo test --package midi-pipeline --tests -- --test-threads=1 2>&1 | grep "test result:"
```

**Step 3: Document results**
```bash
cargo test --package midi-pipeline --tests -- --test-threads=1 > /tmp/pipeline_results.txt 2>&1
grep -E "test result:|test.*ok|test.*FAILED" /tmp/pipeline_results.txt | tail -50
```

#### 2.3 Execute DAW Command Tests (45 minutes)

**Objective:** Run Phase 5 DAW tests after fixes

**Step 1: Run single DAW test file first**
```bash
cd /home/dojevou/projects/midi-software-center
cargo test --package midi-daw --test sequencer_test -- --test-threads=1 2>&1 | tail -30
```

**Step 2: Run all DAW tests**
```bash
cargo test --package midi-daw --tests -- --test-threads=1 2>&1 | tail -50
```

**Step 3: Document any failures**
```bash
cargo test --package midi-daw --tests -- --test-threads=1 > /tmp/daw_results.txt 2>&1
grep -E "FAILED|test result:|thread.*panicked" /tmp/daw_results.txt
```

#### 2.4 Execute Full Integration Test Suite (30 minutes)

**Objective:** Run complete test suite with proper sequencing

**Step 1: Full workspace test run**
```bash
cd /home/dojevou/projects/midi-software-center
cargo test --workspace -- --test-threads=1 2>&1 | tee /tmp/full_test_results.txt
```

**Step 2: Extract summary**
```bash
tail -100 /tmp/full_test_results.txt | grep -E "test result:|passed|failed"
```

**Step 3: Count results**
```bash
grep "test result:" /tmp/full_test_results.txt | tail -1
```

---

### Part 3: Code Quality & Coverage (~2-3 hours)

#### 3.1 Generate Test Coverage Report (1 hour)

**Objective:** Create comprehensive coverage report using tarpaulin

**Step 1: Install tarpaulin if needed**
```bash
cargo install cargo-tarpaulin
```

**Step 2: Run coverage analysis**
```bash
cd /home/dojevou/projects/midi-software-center
cargo tarpaulin --workspace --out Html --timeout 300 --exclude-files "*/migrations/*" 2>&1 | tail -50
```

**Step 3: Check coverage output**
```bash
ls -lh tarpaulin-report.html
open tarpaulin-report.html  # On macOS
# or
xdg-open tarpaulin-report.html  # On Linux
```

**Step 4: Document coverage statistics**
```bash
# Extract coverage percentages
cargo tarpaulin --workspace --exclude-files "*/migrations/*" 2>&1 | grep -E "^[a-z_]+.*\[" | head -20
```

#### 3.2 Format and Lint All Generated Test Code (30 minutes)

**Objective:** Ensure code quality standards

**Step 1: Format all test files**
```bash
cd /home/dojevou/projects/midi-software-center
cargo fmt --all
```

**Step 2: Run clippy linter**
```bash
cargo clippy --workspace --tests -- -D warnings 2>&1 | tail -30
```

**Step 3: Fix any linter warnings**
- If clippy reports warnings in generated tests
- Fix only legitimate issues (not false positives)
- Re-run clippy to verify

#### 3.3 Verify Zero Production Unwraps (15 minutes)

**Objective:** Ensure no production code unwraps made it through

**Step 1: Check production code (not tests)**
```bash
grep -r "\.unwrap()" pipeline/src-tauri/src/ --include="*.rs" | grep -v "#\[cfg(test)\]" | wc -l
grep -r "\.unwrap()" daw/src-tauri/src/ --include="*.rs" | grep -v "#\[cfg(test)\]" | wc -l
grep -r "\.unwrap()" shared/rust/src/ --include="*.rs" | grep -v "#\[cfg(test)\]" | wc -l
```

Expected: 0 unwraps in production code

**Step 2: Check for panics**
```bash
grep -r "panic!" pipeline/src-tauri/src/ --include="*.rs" | grep -v "#\[cfg(test)\]" | wc -l
grep -r "panic!" daw/src-tauri/src/ --include="*.rs" | grep -v "#\[cfg(test)\]" | wc -l
grep -r "panic!" shared/rust/src/ --include="*.rs" | grep -v "#\[cfg(test)\]" | wc -l
```

Expected: 0 panics in production code

---

### Part 4: CI/CD Pipeline Integration (~2-3 hours)

#### 4.1 Create GitHub Actions Workflow (1 hour)

**Objective:** Set up automated testing on every push

**File:** `.github/workflows/test.yml`

**Content:**
```yaml
name: Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

jobs:
  test:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_DB: midi_test
          POSTGRES_USER: midi
          POSTGRES_PASSWORD: midi
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo index
        uses: actions/cache@v3
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-git-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo build
        uses: actions/cache@v3
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}

      - name: Run tests
        run: cargo test --workspace -- --test-threads=1
        env:
          DATABASE_URL: postgres://midi:midi@localhost/midi_test

      - name: Run clippy
        run: cargo clippy --workspace -- -D warnings

      - name: Check formatting
        run: cargo fmt -- --check

  coverage:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_DB: midi_test
          POSTGRES_USER: midi
          POSTGRES_PASSWORD: midi
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Generate coverage
        run: cargo tarpaulin --workspace --out Xml --timeout 300
        env:
          DATABASE_URL: postgres://midi:midi@localhost/midi_test

      - name: Upload to codecov
        uses: codecov/codecov-action@v3
```

**Step 1: Create directory**
```bash
mkdir -p /home/dojevou/projects/midi-software-center/.github/workflows
```

**Step 2: Create workflow file** (see content above)

**Step 3: Verify syntax**
```bash
# YAML is valid if no parse errors
grep -E "^[a-z_]+:" .github/workflows/test.yml | head -10
```

#### 4.2 Create Makefile Test Targets (1 hour)

**Objective:** Add convenient test commands to Makefile

**Current Makefile:** Check if it exists
```bash
cat /home/dojevou/projects/midi-software-center/Makefile | grep "^test"
```

**Add these targets:**
```makefile
# Phase 9: Test execution targets

.PHONY: test-baseline
test-baseline:
	@echo "Running baseline library tests..."
	cargo test --workspace --lib -- --test-threads=1

.PHONY: test-phase5
test-phase5:
	@echo "Running Phase 5 command tests..."
	cargo test --package midi-pipeline --tests -- --test-threads=1
	cargo test --package midi-daw --tests -- --test-threads=1

.PHONY: test-integration
test-integration:
	@echo "Running all tests including integration..."
	cargo test --workspace -- --test-threads=1

.PHONY: test-coverage
test-coverage:
	@echo "Generating coverage report..."
	cargo tarpaulin --workspace --out Html --timeout 300 --exclude-files "*/migrations/*"
	@echo "Coverage report: tarpaulin-report.html"

.PHONY: test-all
test-all: test-baseline test-phase5 test-integration test-coverage
	@echo "All tests completed!"

.PHONY: test-quick
test-quick:
	@echo "Running quick smoke tests..."
	cargo test --lib --workspace -- --test-threads=1 --skip "integration" --skip "performance" --skip "stress"

.PHONY: lint-tests
lint-tests:
	@echo "Linting test code..."
	cargo clippy --workspace --tests -- -D warnings
	cargo fmt --all -- --check
```

**Step 1: Find Makefile location**
```bash
ls /home/dojevou/projects/midi-software-center/Makefile
```

**Step 2: Add targets to Makefile** (Edit existing file, don't overwrite)

**Step 3: Test new targets**
```bash
make test-quick  # Fast validation
make test-baseline  # Verify baseline
```

#### 4.3 Create Makefile Documentation Target (30 minutes)

**Objective:** Document all test commands

**Add target:**
```makefile
.PHONY: help-tests
help-tests:
	@echo "=== Test Targets ==="
	@echo "make test-quick       - Run quick baseline smoke tests (30s)"
	@echo "make test-baseline    - Run all baseline lib tests (2m)"
	@echo "make test-phase5      - Run Phase 5 command tests (3m)"
	@echo "make test-integration - Run full integration test suite (5m)"
	@echo "make test-coverage    - Generate coverage report (8m)"
	@echo "make test-all         - Run all tests + coverage (18m)"
	@echo "make lint-tests       - Lint and format check test code (2m)"
```

**Step 1: Verify help works**
```bash
make help-tests
```

---

## 📋 Phase 9 Execution Checklist

### Pre-Execution (5 minutes)
- [ ] Review this document
- [ ] Ensure database is running (PostgreSQL + Meilisearch)
- [ ] Verify git status is clean or stashed

### Compilation Fixes (4-6 hours)
- [ ] Fix DAW sqlx enum type issue (1 hour)
- [ ] Fix DAW module resolution errors (1.5 hours)
- [ ] Fix pipeline test imports (1 hour)
- [ ] Verify no compilation errors: `cargo check --all-targets`

### Test Execution (2-3 hours)
- [ ] Run baseline tests: `cargo test --workspace --lib -- --test-threads=1`
- [ ] Run pipeline command tests: Phase 5 validation
- [ ] Run DAW command tests: Phase 5 validation
- [ ] Run full integration suite: `cargo test --workspace -- --test-threads=1`

### Code Quality (1-2 hours)
- [ ] Generate coverage: `cargo tarpaulin --workspace --out Html`
- [ ] Run clippy: `cargo clippy --workspace --tests -- -D warnings`
- [ ] Check formatting: `cargo fmt --all -- --check`
- [ ] Verify zero production unwraps

### CI/CD Integration (1-2 hours)
- [ ] Create `.github/workflows/test.yml`
- [ ] Add test targets to Makefile
- [ ] Commit workflow files
- [ ] Trigger first CI/CD run on push

### Final Verification (30 minutes)
- [ ] All tests passing locally
- [ ] Coverage report generated
- [ ] CI/CD pipeline configured
- [ ] Documentation updated

---

## 🎯 Success Criteria

**Phase 9 Complete When:**
1. ✅ All 388 baseline tests pass
2. ✅ All 452+ generated tests compile and run
3. ✅ Total: 1,223+ tests passing
4. ✅ Coverage report generated (minimum 50% target)
5. ✅ Zero production code panics/unwraps
6. ✅ GitHub Actions CI/CD configured
7. ✅ Makefile test targets working
8. ✅ Documentation complete

**Estimated Timeline:**
- Part 1: 4-6 hours (compilation fixes)
- Part 2: 2-3 hours (test execution)
- Part 3: 2-3 hours (coverage & quality)
- Part 4: 1-2 hours (CI/CD setup)
- **Total: 10-12 hours**

---

## 📞 Troubleshooting

**Issue: "type 'music_key' does not exist"**
- Solution: Replace sqlx::query!() with sqlx::query_as() and TEXT cast
- Location: daw/src-tauri/tests/commands/analysis_test.rs

**Issue: "failed to resolve: use of unresolved module"**
- Solution: Fix import statements to use correct crate names
- Common fixes:
  - `midi_software_center_shared` → `midi_library_shared`
  - `daw_lib` → `midi_daw` or internal module path

**Issue: "test thread panicked"**
- Check database is running with migrations applied
- Verify `--test-threads=1` for database tests
- Check for resource cleanup issues

**Issue: Coverage report not generated**
- Ensure tarpaulin is installed: `cargo install cargo-tarpaulin`
- Check for timeout: increase `--timeout` value
- Verify database is running

---

## 📊 Expected Results

**Baseline Tests:**
```
test result: ok. 388 passed; 0 failed; 7 ignored; 0 measured
```

**Generated Phase 5 Tests:**
```
file_import_test: ~42 passing
analyze_test: ~35 passing
split_file_test: ~27 passing
archive_import_test: ~20 passing
```

**Generated Phase 6 Tests:**
```
models_test: ~73 passing
```

**Generated Phase 7 Tests:**
```
workflows_test: ~45 passing
workflows_extended_test: ~24 passing
performance_test: ~12 passing
stress_test: ~10 passing
journey_test: ~13 passing
```

**Total Expected: 1,223+ passing tests**

---

**Ready to execute? Follow Part 1-4 in sequence. No stubs, only production-ready code.**

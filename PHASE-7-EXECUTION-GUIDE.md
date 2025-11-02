# Phase 7: Integration & E2E Tests - Execution Guide

**Quick Reference for Running Phase 7 Tests**

---

## 🚀 Quick Start

### Run All Workflow Tests (Fast - No Performance/Stress):
```bash
# All workflow integration tests (~2-5 minutes)
cargo test --test workflows_test --test workflows_extended_test --test journey_test
```

### Run Complete Test Suite (Includes Performance & Stress):
```bash
# WARNING: This takes 10-30 minutes
cargo test --test workflows_test --test workflows_extended_test --test performance_test --test stress_test --test journey_test -- --ignored --nocapture
```

---

## 📁 Test File Organization

### 1. workflows_test.rs (21 tests)
**Coverage:** Music production, library management, collaboration
**Runtime:** ~2-3 minutes
**Database:** Required
**Type:** Integration tests (NOT ignored)

```bash
# Run all workflow tests
cargo test --test workflows_test

# Run specific category
cargo test --test workflows_test test_workflow_compose
```

### 2. workflows_extended_test.rs (24 tests)
**Coverage:** Search, performance, error recovery, advanced features
**Runtime:** ~3-4 minutes
**Database:** Required
**Type:** Integration tests (NOT ignored)

```bash
# Run all extended workflow tests
cargo test --test workflows_extended_test

# Run error recovery tests
cargo test --test workflows_extended_test test_workflow_.*recovery
```

### 3. performance_test.rs (13 tests - IGNORED)
**Coverage:** Performance regression benchmarks
**Runtime:** ~10-20 minutes (especially 1000 file test)
**Database:** Required
**Type:** Performance tests (IGNORED by default)

```bash
# Run ALL performance tests (slow)
cargo test --test performance_test -- --ignored --nocapture

# Run specific performance test
cargo test --test performance_test test_perf_import_10_files -- --ignored --nocapture

# Run fast performance tests only
cargo test --test performance_test test_perf_import_10_files test_perf_ui_responsiveness -- --ignored --nocapture
```

### 4. stress_test.rs (11 tests - IGNORED)
**Coverage:** Extreme conditions, boundary testing
**Runtime:** ~15-30 minutes
**Database:** Required
**Type:** Stress tests (IGNORED by default)

```bash
# Run ALL stress tests (very slow)
cargo test --test stress_test -- --ignored --nocapture

# Run specific stress test
cargo test --test stress_test test_stress_import_malformed_batch -- --ignored --nocapture

# Skip the 5000 file test (fastest subset)
cargo test --test stress_test --skip test_stress_import_5000_files -- --ignored --nocapture
```

### 5. journey_test.rs (13 tests)
**Coverage:** User persona journeys
**Runtime:** ~2-3 minutes
**Database:** Required
**Type:** Integration tests (NOT ignored)

```bash
# Run all user journey tests
cargo test --test journey_test

# Run specific persona
cargo test --test journey_test test_journey_first_time_user -- --nocapture
```

---

## 🎯 Common Test Scenarios

### Developer Quick Check (< 5 minutes):
```bash
# Run fast integration tests only
cargo test --test workflows_test --test journey_test
```

### Pre-Commit Validation (< 10 minutes):
```bash
# All integration tests + fast performance tests
cargo test --test workflows_test --test workflows_extended_test --test journey_test
cargo test --test performance_test test_perf_import_10_files test_perf_ui_responsiveness -- --ignored --nocapture
```

### Full Regression Suite (20-30 minutes):
```bash
# Complete Phase 7 test suite
cargo test --test workflows_test --test workflows_extended_test --test journey_test
cargo test --test performance_test -- --ignored --nocapture
cargo test --test stress_test --skip test_stress_import_5000_files -- --ignored --nocapture
```

### CI/CD Pipeline (Recommended):
```bash
# Stage 1: Fast integration tests (required)
cargo test --test workflows_test --test workflows_extended_test --test journey_test

# Stage 2: Performance tests (optional, nightly)
cargo test --test performance_test -- --ignored --nocapture

# Stage 3: Stress tests (optional, weekly)
cargo test --test stress_test -- --ignored --nocapture
```

---

## 🔧 Test Environment Requirements

### Database Setup:
```bash
# Ensure test database is running
make docker-up

# Or manually:
docker-compose up -d postgres

# Verify connection
psql postgresql://midiuser:145278963@localhost:5433/midi_library -c "SELECT 1"
```

### Environment Variables (Optional):
```bash
# Override test database URL
export TEST_DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

# Run tests with custom database
cargo test --test workflows_test
```

### Cleanup Between Test Runs:
```bash
# Tests auto-cleanup, but for manual cleanup:
psql postgresql://midiuser:145278963@localhost:5433/midi_library -c "DELETE FROM files WHERE file_path LIKE '%/tmp/%'"
```

---

## 📊 Test Output Examples

### Successful Workflow Test:
```
running 21 tests
test test_workflow_compose_new_song ... ok
test test_workflow_load_template_customize ... ok
test test_workflow_jam_session ... ok
...
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Performance Test with Timing:
```
running 1 test
✓ Import 10 files: 1.234s
test test_perf_import_10_files ... ok
```

### User Journey with Steps:
```
=== FIRST-TIME USER JOURNEY ===
Step 1: Opening application for first time...
  Initial file count: 0
Step 2: Importing first MIDI file...
  ✓ Imported file ID: 1
Step 3: Viewing file details...
  File: /tmp/.../my_first_midi.mid
  Size: 512 bytes
...
✓ First-time user journey complete!
```

---

## 🐛 Troubleshooting

### Issue: Database connection failed
```bash
# Solution: Start database
make docker-up

# Or check if running:
docker ps | grep postgres
```

### Issue: Tests timeout
```bash
# Solution: Increase timeout (for slow machines)
RUST_TEST_TIMEOUT=300 cargo test --test performance_test -- --ignored --nocapture
```

### Issue: Permission denied on temp files
```bash
# Solution: Clean temp directory
rm -rf /tmp/rust_test_*
```

### Issue: Port 5433 already in use
```bash
# Solution 1: Stop conflicting service
docker-compose down

# Solution 2: Change test database port
export TEST_DATABASE_URL="postgresql://midiuser:145278963@localhost:5434/midi_library"
```

### Issue: Out of memory during stress tests
```bash
# Solution: Run stress tests individually
cargo test --test stress_test test_stress_import_malformed_batch -- --ignored --nocapture
```

---

## 📈 Performance Benchmarks

### Expected Times (Reference Machine):
| Test Category | Count | Time | Notes |
|--------------|-------|------|-------|
| Workflows (Part 1) | 21 | ~2-3 min | Standard workflows |
| Workflows (Part 2) | 24 | ~3-4 min | Extended workflows |
| Performance | 13 | ~10-20 min | Includes 1000 file import |
| Stress | 11 | ~15-30 min | Includes 5000 file import |
| Journey | 13 | ~2-3 min | User scenarios |

### Performance Test Targets:
- Import 10 files: < 2s ✅
- Import 100 files: < 15s ✅
- Import 1000 files: < 120s ✅
- Search 10k database: < 1s ✅
- UI responsiveness: < 100ms ✅

---

## 🎓 Test Categories Explained

### Integration Tests (NOT ignored):
- **Purpose:** Validate multi-step workflows
- **Database:** Real PostgreSQL operations
- **Files:** Actual MIDI file I/O
- **Runtime:** Fast (2-5 minutes)
- **Run by default:** YES

### Performance Tests (IGNORED):
- **Purpose:** Establish performance baselines
- **Database:** Real PostgreSQL operations
- **Files:** Large datasets (up to 1000 files)
- **Runtime:** Slow (10-20 minutes)
- **Run by default:** NO (requires `--ignored` flag)

### Stress Tests (IGNORED):
- **Purpose:** Test system boundaries
- **Database:** Heavy concurrent load
- **Files:** Extreme cases (5000 files, malformed data)
- **Runtime:** Very slow (15-30 minutes)
- **Run by default:** NO (requires `--ignored` flag)

### User Journey Tests (NOT ignored):
- **Purpose:** Validate end-to-end user workflows
- **Database:** Real PostgreSQL operations
- **Files:** Persona-specific scenarios
- **Runtime:** Fast (2-3 minutes)
- **Run by default:** YES

---

## 🔍 Debugging Individual Tests

### Run Single Test with Output:
```bash
# Example: Debug first-time user journey
cargo test --test journey_test test_journey_first_time_user -- --nocapture

# Example: Debug performance import
cargo test --test performance_test test_perf_import_10_files -- --ignored --nocapture --exact
```

### Enable Rust Backtrace:
```bash
# Full backtrace on panic
RUST_BACKTRACE=1 cargo test --test workflows_test test_workflow_compose_new_song

# Full backtrace with source locations
RUST_BACKTRACE=full cargo test --test workflows_test test_workflow_compose_new_song
```

### Test-Specific Logging:
```bash
# Enable debug logging
RUST_LOG=debug cargo test --test workflows_test -- --nocapture

# Filter to specific module
RUST_LOG=midi_pipeline::commands=debug cargo test --test workflows_test
```

---

## 📝 Test Maintenance

### Adding New Tests:
1. Choose appropriate file (workflows, performance, stress, journey)
2. Follow existing test pattern
3. Use helper functions (`create_midi_bytes`, `cleanup_test_files`)
4. Add cleanup at end of test
5. Document test purpose and steps

### Updating Performance Targets:
1. Edit `performance_test.rs`
2. Update assertion thresholds
3. Update documentation in comments
4. Update PHASE-7-COMPLETE-SUMMARY.md

### Debugging Flaky Tests:
```bash
# Run test 10 times to detect flakiness
for i in {1..10}; do
  echo "Run $i"
  cargo test --test workflows_test test_specific_test || break
done
```

---

## 🎯 CI/CD Integration

### GitHub Actions Example:
```yaml
# .github/workflows/phase7-tests.yml
name: Phase 7 Tests

on: [push, pull_request]

jobs:
  integration:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Start Database
        run: docker-compose up -d postgres
      - name: Run Integration Tests
        run: |
          cargo test --test workflows_test
          cargo test --test workflows_extended_test
          cargo test --test journey_test

  performance:
    runs-on: ubuntu-latest
    if: github.event_name == 'schedule' # Nightly only
    steps:
      - uses: actions/checkout@v3
      - name: Start Database
        run: docker-compose up -d postgres
      - name: Run Performance Tests
        run: cargo test --test performance_test -- --ignored --nocapture

  stress:
    runs-on: ubuntu-latest
    if: github.event_name == 'schedule' # Weekly only
    steps:
      - uses: actions/checkout@v3
      - name: Start Database
        run: docker-compose up -d postgres
      - name: Run Stress Tests
        run: cargo test --test stress_test -- --ignored --nocapture
```

---

## ✅ Test Verification Checklist

Before considering Phase 7 complete:

- [ ] All 5 test files compile without errors
- [ ] All 82 tests pass (integration tests)
- [ ] Performance tests establish baselines
- [ ] Stress tests verify system boundaries
- [ ] User journey tests validate UX
- [ ] Database cleanup works correctly
- [ ] Documentation is complete
- [ ] CI/CD integration tested

---

## 📚 Additional Resources

- **Main Summary:** PHASE-7-COMPLETE-SUMMARY.md
- **Test Coverage Plan:** TEST-COVERAGE-PLAN.md
- **Project Architecture:** ARCHITECTURE-REFERENCE.md
- **Development Workflow:** DEVELOPMENT-WORKFLOW.md

---

**Last Updated:** 2025-11-02
**Phase:** 7 of 8 (Integration & E2E Tests)
**Status:** ✅ Complete - 82 tests, 4,353 lines

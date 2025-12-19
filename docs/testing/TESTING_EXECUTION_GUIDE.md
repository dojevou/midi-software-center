# Testing Execution Guide

> **Status:** Ready for use when Streams A-F complete
> **Last Updated:** 2025-12-17

This guide explains how to run all tests and benchmarks for the MIDI Software Center.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Unit Tests](#unit-tests)
3. [Integration Tests](#integration-tests)
4. [Frontend Tests](#frontend-tests)
5. [Performance Benchmarks](#performance-benchmarks)
6. [Coverage Reports](#coverage-reports)
7. [Continuous Integration](#continuous-integration)
8. [Troubleshooting](#troubleshooting)

---

## Quick Start

### Run All Tests

```bash
# From project root
make test

# Or manually
cargo test --workspace --lib        # Rust unit tests
cargo test --workspace --test '*'   # Integration tests
cd app && pnpm test                 # Frontend tests
```

### Run Specific Test Suite

```bash
# VIP3 filter counts
cargo test filter_counts_test

# Mixer commands
cargo test mixer_test

# Automation
cargo test automation_test

# Frontend VIP3 browser
cd app && pnpm test VIP3Browser
```

---

## Unit Tests

### Location

All unit tests are in `app/src-tauri/tests/`:
- `mixer_test.rs` - Mixer commands (30+ tests)
- `automation_test.rs` - Automation system (25+ tests)
- `project_test.rs` - Project management (15+ tests)
- `collections_test.rs` - VIP3 collections (20+ tests)
- `filter_counts_test.rs` - Filter count queries (15+ tests)

### Running Unit Tests

```bash
# All unit tests
cargo test --workspace --lib

# Specific module
cargo test mixer_test

# Specific test
cargo test test_mixer_set_gain

# With output
cargo test -- --nocapture

# With logging
RUST_LOG=debug cargo test

# Parallel execution (default)
cargo test -- --test-threads=8

# Sequential (for debugging)
cargo test -- --test-threads=1
```

### Test Database Setup

Unit tests require a test database:

```bash
# Create test database
createdb midi_library_test

# Set environment variable
export TEST_DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library_test"

# Run migrations on test DB
psql $TEST_DATABASE_URL -f database/migrations/*.sql

# Or use script
./scripts/setup-test-db.sh
```

### Writing New Unit Tests

Use the templates in `app/src-tauri/tests/templates/`:

```rust
// Copy template to tests/
cp tests/templates/mixer_test.rs tests/mixer_test.rs

// Update test implementations (replace TODO comments)
// Run tests
cargo test mixer_test
```

---

## Integration Tests

### Location

Integration tests are in `app/src-tauri/tests/integration/`:
- `full_workflow_test.rs` - Complete workflows (VIP3 → DAW)

### Running Integration Tests

```bash
# All integration tests
cargo test --test integration

# Specific workflow
cargo test test_workflow_browse_load_mix_export

# With full output
cargo test --test integration -- --nocapture
```

### Integration Test Requirements

1. **Database**: Test database must be running and populated
2. **Test Data**: Sample MIDI files in test directory
3. **Services**: All backend services (Tauri app) running

### Setting Up Test Data

```bash
# Import test MIDI files
./scripts/import-test-data.sh

# Verify test data
psql $TEST_DATABASE_URL -c "SELECT COUNT(*) FROM files;"
# Should show: count > 100
```

---

## Frontend Tests

### Location

Frontend tests are in `app/src/lib/components/__tests__/`:
- `VIP3Browser.test.ts` - VIP3 browser UI
- `MixerWindow.test.ts` - Mixer UI
- `AutomationLane.test.ts` - Automation UI

### Running Frontend Tests

```bash
cd app

# All tests
pnpm test

# Specific component
pnpm test VIP3Browser

# Watch mode (re-run on changes)
pnpm test:watch

# With coverage
pnpm test:coverage

# UI mode (interactive)
pnpm test:ui
```

### Frontend Test Setup

```bash
# Install dependencies
cd app
pnpm install

# Setup Vitest
# (Already configured in vite.config.ts)

# Mock Tauri API
# (Already mocked in tests)
```

### Writing Frontend Tests

```typescript
// Example test
import { render, screen, fireEvent } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import VIP3Browser from '$lib/components/VIP3/VIP3Browser.svelte';

describe('VIP3Browser', () => {
  it('should render', () => {
    const { container } = render(VIP3Browser);
    expect(container).toBeTruthy();
  });

  it('should apply filter on click', async () => {
    render(VIP3Browser);
    const button = screen.getByText('Piano');
    await fireEvent.click(button);
    // Assert filter applied
  });
});
```

---

## Performance Benchmarks

### Location

Benchmark scripts in `scripts/benchmarks/`:
- `benchmark-filter-counts.sh` - Filter count performance
- `benchmark-mixer-processing.sh` - Mixer operations
- `benchmark-automation-playback.sh` - Automation lookups

### Running Benchmarks

```bash
# Run all benchmarks
./scripts/run-all-benchmarks.sh

# Individual benchmarks
./scripts/benchmarks/benchmark-filter-counts.sh
./scripts/benchmarks/benchmark-mixer-processing.sh
./scripts/benchmarks/benchmark-automation-playback.sh
```

### Performance Targets

| Benchmark | Target | Notes |
|-----------|--------|-------|
| Filter counts | <50ms | With caching |
| Mixer gain/pan | <10ms | Per operation |
| VU meters | <1ms | 60 Hz polling |
| Automation lookup | <1ms | Per lookup |
| Project save | <500ms | Large projects |

### Interpreting Results

Benchmark output:

```
Test 1: Filter counts with no filters
======================================
Iterations: 100
Average: 42ms
Target: <50ms
Success rate: 98%
Status: PASS ✓
```

- **PASS**: Average < Target
- **FAIL**: Average ≥ Target
- **WARN**: Non-critical issue (e.g., cache not working)

---

## Coverage Reports

### Rust Coverage

Using `cargo-tarpaulin`:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --workspace --out Html --output-dir target/coverage

# View report
open target/coverage/index.html

# CI-friendly output
cargo tarpaulin --workspace --out Xml
```

### Frontend Coverage

Using Vitest:

```bash
cd app

# Generate coverage
pnpm test:coverage

# View report
open coverage/index.html
```

### Coverage Targets

- **Overall**: >80%
- **Critical paths**: >95%
  - Mixer commands
  - Automation engine
  - Filter count queries
  - Project save/load

---

## Continuous Integration

### GitHub Actions

TODO: Add GitHub Actions workflow when CI is set up

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_PASSWORD: 145278963
        ports:
          - 5433:5432

    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1

      - name: Run tests
        run: |
          cargo test --workspace
          cd app && pnpm install && pnpm test

      - name: Coverage
        run: cargo tarpaulin --out Xml

      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

### Pre-commit Hooks

```bash
# Install pre-commit
pip install pre-commit

# Setup hooks
pre-commit install

# Hooks will run:
# - cargo test (fast tests only)
# - cargo fmt --check
# - cargo clippy
# - pnpm test (frontend)
```

---

## Troubleshooting

### Tests Fail to Connect to Database

```bash
# Check PostgreSQL is running
docker-compose ps

# Check connection
psql postgresql://midiuser:145278963@localhost:5433/midi_library

# Verify TEST_DATABASE_URL
echo $TEST_DATABASE_URL

# Recreate test database
dropdb midi_library_test
createdb midi_library_test
psql $TEST_DATABASE_URL -f database/migrations/*.sql
```

### Tests Timeout

```bash
# Increase timeout (default: 60s)
cargo test -- --test-threads=1

# Or in code:
#[tokio::test(flavor = "multi_thread")]
async fn my_test() {
    tokio::time::timeout(
        Duration::from_secs(120),
        async { /* test */ }
    ).await.unwrap();
}
```

### Flaky Tests

```bash
# Run test multiple times
for i in {1..10}; do
  cargo test test_name || break
done

# Or use nextest
cargo install cargo-nextest
cargo nextest run --retries 3
```

### Frontend Tests Fail

```bash
# Clear cache
cd app
rm -rf node_modules .vite
pnpm install

# Update dependencies
pnpm update

# Check Tauri mock
# Ensure @tauri-apps/api is mocked in vitest.config.ts
```

### Benchmarks Show Poor Performance

1. **Check system resources**: Close other applications
2. **Check database**: Run `VACUUM ANALYZE;`
3. **Check indexes**: Verify all indexes exist
4. **Profile code**: Use `cargo flamegraph`

```bash
# Profile a test
cargo install flamegraph
cargo flamegraph --test mixer_test -- test_mixer_performance
```

---

## Best Practices

### Running Tests Locally

1. **Always run tests before committing**
2. **Run full suite before creating PR**
3. **Check coverage for new code**
4. **Run benchmarks for performance-critical changes**

### Writing Good Tests

1. **Test one thing**: Each test should verify a single behavior
2. **Use descriptive names**: `test_mixer_gain_clips_at_12db`
3. **Arrange, Act, Assert**: Clear structure
4. **Clean up**: Use `beforeEach`/`afterEach` to reset state
5. **Mock external dependencies**: Don't depend on external services

### Test Data Management

1. **Use fixtures**: Store test data in `tests/fixtures/`
2. **Don't commit large files**: Use `.gitignore`
3. **Generate data programmatically**: For large datasets
4. **Clean up after tests**: Delete test data in `cleanup_test_data()`

---

## Useful Commands Cheat Sheet

```bash
# Quick test run
make test

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration

# Frontend tests only
cd app && pnpm test

# All benchmarks
./scripts/run-all-benchmarks.sh

# Coverage report
cargo tarpaulin --out Html

# Watch mode (re-run on changes)
cargo watch -x test

# Specific test with logs
RUST_LOG=debug cargo test test_name -- --nocapture

# Frontend watch mode
cd app && pnpm test:watch
```

---

## Next Steps

1. **Copy test templates** when implementing features
2. **Write tests as you code** (TDD approach)
3. **Run tests frequently** during development
4. **Review coverage** regularly
5. **Monitor benchmarks** for regressions

For more information:
- User Guide: `docs/USER_GUIDE.md`
- API Reference: `docs/API_REFERENCE.md`
- Implementation Plan: `docs/PARALLEL_WORK_STREAMS.md`

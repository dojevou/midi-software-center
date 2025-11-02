# Performance Threshold Configuration Guide

**Priority:** P2 (Week 1-2)
**Estimated Effort:** 2-3 hours
**Impact:** Fix CI/CD test failures on slower systems
**Status:** Ready for implementation

---

## 🎯 Objective

Make performance test thresholds configurable via environment variables to prevent false failures on slower CI systems.

---

## 📋 Problem Analysis

### Current Issue

Hard-coded performance thresholds in tests:
```rust
// export_test.rs:172
assert!(elapsed < Duration::from_secs(2),
    "Export should complete within 2 seconds");

// project_test.rs:310
assert!(elapsed < Duration::from_millis(1500),
    "Concurrent ops should complete within 1.5s");
```

**Problem:** Thresholds work on local fast systems but fail on slower CI systems.

### Impact
- ❌ Tests fail intermittently in CI
- ❌ False negatives block deployments
- ❌ No environment-specific tuning

### Solution
Make thresholds configurable with sensible defaults.

---

## ✅ Solution Architecture

### Step 1: Create Performance Config Module

**File:** `pipeline/src-tauri/tests/common/perf_config.rs` (NEW)

```rust
use std::time::Duration;

/// Performance threshold configuration
/// Values can be overridden via environment variables
#[derive(Debug, Clone)]
pub struct PerfConfig {
    /// Single file import threshold (default: 5 seconds)
    pub single_import_max_ms: u64,

    /// Batch import threshold per file (default: 200ms)
    pub batch_import_per_file_max_ms: u64,

    /// Database query threshold (default: 500ms)
    pub db_query_max_ms: u64,

    /// Concurrent operation threshold (default: 2 seconds)
    pub concurrent_ops_max_ms: u64,

    /// Export operation threshold (default: 2 seconds)
    pub export_max_ms: u64,

    /// Sequencer operations threshold (default: 500ms)
    pub sequencer_ops_max_ms: u64,

    /// Search query threshold (default: 1 second)
    pub search_query_max_ms: u64,
}

impl PerfConfig {
    /// Load configuration from environment or use defaults
    pub fn load() -> Self {
        Self {
            single_import_max_ms: std::env::var("MIDI_PERF_SINGLE_IMPORT_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5000),

            batch_import_per_file_max_ms: std::env::var("MIDI_PERF_BATCH_IMPORT_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(200),

            db_query_max_ms: std::env::var("MIDI_PERF_DB_QUERY_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(500),

            concurrent_ops_max_ms: std::env::var("MIDI_PERF_CONCURRENT_OPS_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2000),

            export_max_ms: std::env::var("MIDI_PERF_EXPORT_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2000),

            sequencer_ops_max_ms: std::env::var("MIDI_PERF_SEQUENCER_OPS_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(500),

            search_query_max_ms: std::env::var("MIDI_PERF_SEARCH_QUERY_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1000),
        }
    }

    /// Get duration for a specific operation
    pub fn duration_for(&self, operation: &str) -> Duration {
        let ms = match operation {
            "single_import" => self.single_import_max_ms,
            "batch_import" => self.batch_import_per_file_max_ms,
            "db_query" => self.db_query_max_ms,
            "concurrent_ops" => self.concurrent_ops_max_ms,
            "export" => self.export_max_ms,
            "sequencer_ops" => self.sequencer_ops_max_ms,
            "search_query" => self.search_query_max_ms,
            _ => 1000, // Default fallback
        };
        Duration::from_millis(ms)
    }

    /// Get high-tolerance config for slower systems
    pub fn relaxed() -> Self {
        Self {
            single_import_max_ms: 10000,    // 10s (vs 5s default)
            batch_import_per_file_max_ms: 400,  // 400ms (vs 200ms default)
            db_query_max_ms: 1000,          // 1s (vs 500ms default)
            concurrent_ops_max_ms: 4000,    // 4s (vs 2s default)
            export_max_ms: 4000,             // 4s (vs 2s default)
            sequencer_ops_max_ms: 1000,     // 1s (vs 500ms default)
            search_query_max_ms: 2000,      // 2s (vs 1s default)
        }
    }

    /// Get strict config for high-performance validation
    pub fn strict() -> Self {
        Self {
            single_import_max_ms: 2000,     // 2s (vs 5s default)
            batch_import_per_file_max_ms: 100,  // 100ms (vs 200ms default)
            db_query_max_ms: 200,           // 200ms (vs 500ms default)
            concurrent_ops_max_ms: 1000,    // 1s (vs 2s default)
            export_max_ms: 1000,             // 1s (vs 2s default)
            sequencer_ops_max_ms: 200,      // 200ms (vs 500ms default)
            search_query_max_ms: 500,       // 500ms (vs 1s default)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PerfConfig::load();
        assert_eq!(config.export_max_ms, 2000);
        assert_eq!(config.db_query_max_ms, 500);
    }

    #[test]
    fn test_relaxed_config() {
        let config = PerfConfig::relaxed();
        assert!(config.export_max_ms > 2000);
        assert!(config.db_query_max_ms > 500);
    }

    #[test]
    fn test_duration_for() {
        let config = PerfConfig::load();
        assert_eq!(config.duration_for("export").as_millis(), 2000);
        assert_eq!(config.duration_for("db_query").as_millis(), 500);
    }
}
```

### Step 2: Update Test Helper

**File:** `pipeline/src-tauri/tests/common/mod.rs` (UPDATE)

```rust
pub mod perf_config;
pub use perf_config::PerfConfig;

/// Get the appropriate performance config based on environment
pub fn get_perf_config() -> PerfConfig {
    // If TEST_MODE=relaxed, use relaxed thresholds
    if std::env::var("TEST_MODE")
        .map(|v| v == "relaxed")
        .unwrap_or(false)
    {
        PerfConfig::relaxed()
    } else {
        PerfConfig::load()
    }
}

/// Fixture for performance tests
pub struct PerfTest {
    config: PerfConfig,
}

impl PerfTest {
    pub fn new() -> Self {
        Self {
            config: get_perf_config(),
        }
    }

    pub fn config(&self) -> &PerfConfig {
        &self.config
    }

    pub fn assert_under_threshold(&self, elapsed: std::time::Duration, operation: &str) {
        let threshold = self.config.duration_for(operation);
        assert!(
            elapsed <= threshold,
            "Operation '{}' took {:?}, exceeds threshold of {:?}. \
             Override with env var: MIDI_PERF_*_MS",
            operation, elapsed, threshold
        );
    }
}
```

### Step 3: Update Test Files

#### export_test.rs

**Before:**
```rust
#[tokio::test]
async fn test_export_performance() {
    let start = Instant::now();
    let result = export_to_file(&seq, path, None).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok());
    assert!(elapsed < Duration::from_secs(2),
        "Export should complete within 2 seconds");
}
```

**After:**
```rust
use crate::common::{PerfTest};

#[tokio::test]
async fn test_export_performance() {
    let perf = PerfTest::new();

    let start = Instant::now();
    let result = export_to_file(&seq, path, None).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok());
    perf.assert_under_threshold(elapsed, "export");
}
```

#### project_test.rs

**Before:**
```rust
#[tokio::test]
async fn test_concurrent_track_operations() {
    let start = Instant::now();
    // ... concurrent operations ...
    let elapsed = start.elapsed();

    assert!(elapsed < Duration::from_millis(1500),
        "Concurrent ops should complete within 1.5s");
}
```

**After:**
```rust
use crate::common::PerfTest;

#[tokio::test]
async fn test_concurrent_track_operations() {
    let perf = PerfTest::new();

    let start = Instant::now();
    // ... concurrent operations ...
    let elapsed = start.elapsed();

    perf.assert_under_threshold(elapsed, "concurrent_ops");
}
```

#### All Other Performance Tests

Apply the same pattern to:
- `search_test.rs` - Search performance assertions
- `sequencer_test.rs` - Sequencer operation timing
- `midi_test.rs` - MIDI message handling performance
- `integration_test.rs` - End-to-end workflows

---

## 🔧 Environment Variable Configuration

### Default Thresholds (Local Development)
```bash
# Using defaults (no env vars needed)
cargo test --workspace
```

### Relaxed Mode (CI Slow Systems)
```bash
# For slower CI environments
TEST_MODE=relaxed cargo test --workspace
```

Or individual settings:
```bash
MIDI_PERF_EXPORT_MS=5000 \
MIDI_PERF_CONCURRENT_OPS_MS=4000 \
cargo test --workspace
```

### Strict Mode (Performance Validation)
```bash
# For high-performance validation
TEST_MODE=strict cargo test --workspace
```

### Individual Override
```bash
# Override just one threshold
MIDI_PERF_EXPORT_MS=3000 cargo test --test export_test

# Multiple overrides
MIDI_PERF_EXPORT_MS=3000 \
MIDI_PERF_DB_QUERY_MS=300 \
cargo test --workspace
```

---

## 📊 Threshold Values Reference

| Operation | Default | Relaxed | Strict | Environment Var |
|-----------|---------|---------|--------|-----------------|
| Single Import | 5s | 10s | 2s | MIDI_PERF_SINGLE_IMPORT_MS |
| Batch Import | 200ms | 400ms | 100ms | MIDI_PERF_BATCH_IMPORT_MS |
| DB Query | 500ms | 1s | 200ms | MIDI_PERF_DB_QUERY_MS |
| Concurrent Ops | 2s | 4s | 1s | MIDI_PERF_CONCURRENT_OPS_MS |
| Export | 2s | 4s | 1s | MIDI_PERF_EXPORT_MS |
| Sequencer Ops | 500ms | 1s | 200ms | MIDI_PERF_SEQUENCER_OPS_MS |
| Search Query | 1s | 2s | 500ms | MIDI_PERF_SEARCH_QUERY_MS |

---

## 🛠️ Implementation Checklist

### Phase 1: Create Config Module (1 hour)
- [ ] Create `perf_config.rs` with PerfConfig struct
- [ ] Implement environment variable loading
- [ ] Add relaxed() and strict() presets
- [ ] Add unit tests for config loading

### Phase 2: Update Test Helpers (30 minutes)
- [ ] Add PerfTest fixture to common/mod.rs
- [ ] Add assert_under_threshold method
- [ ] Add get_perf_config helper
- [ ] Document helper usage

### Phase 3: Update All Performance Tests (1 hour)
- [ ] Update export_test.rs assertions
- [ ] Update project_test.rs assertions
- [ ] Update sequencer_test.rs assertions
- [ ] Update midi_test.rs assertions
- [ ] Update search_test.rs assertions
- [ ] Update integration_test.rs assertions

### Phase 4: CI/CD Integration (30 minutes)
- [ ] Update .github/workflows/* to use TEST_MODE=relaxed
- [ ] Document threshold tuning in README
- [ ] Test with different env var combinations
- [ ] Verify CI passes on slow systems

---

## 📈 CI/CD Configuration

### GitHub Actions Example

**.github/workflows/test.yml:**
```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable

      # Fast system - strict thresholds
      - name: Run tests (strict)
        if: github.event_name == 'push' && github.ref == 'refs/heads/main'
        run: cargo test --workspace
        env:
          TEST_MODE: strict

      # Pull requests - relaxed thresholds
      - name: Run tests (relaxed)
        if: github.event_name == 'pull_request'
        run: cargo test --workspace
        env:
          TEST_MODE: relaxed

      # Allow manual override
      - name: Run tests (custom)
        if: contains(github.event.head_commit.message, '[perf-strict]')
        run: cargo test --workspace
        env:
          TEST_MODE: strict
```

---

## ✅ Validation Steps

### After Implementation
```bash
# 1. Test with defaults
cargo test --workspace -- --test-threads=1

# 2. Test with relaxed mode
TEST_MODE=relaxed cargo test --workspace -- --test-threads=1

# 3. Test with strict mode
TEST_MODE=strict cargo test --workspace -- --test-threads=1

# 4. Test with individual overrides
MIDI_PERF_EXPORT_MS=5000 cargo test --test export_test

# 5. Verify config module tests pass
cargo test --lib perf_config
```

### Expected Results
- ✅ All tests pass with default thresholds
- ✅ Tests pass with relaxed mode on slow systems
- ✅ Tests pass with strict mode on fast systems
- ✅ Custom overrides work correctly

---

## 📝 Documentation

After implementation, create:

1. **Performance Testing Guide**
   - How to write performance tests
   - How to use PerfConfig
   - When to use relaxed vs strict

2. **CI/CD Tuning Guide**
   - How to adjust thresholds
   - Environment variable reference
   - Troubleshooting slow test runs

3. **README Update**
   - Performance test configuration section
   - Environment variable reference

---

## 🎯 Success Criteria

- [ ] PerfConfig module implemented and tested
- [ ] All performance tests updated to use config
- [ ] Tests pass with all three modes (default, relaxed, strict)
- [ ] CI/CD configured with relaxed thresholds
- [ ] Documentation complete

---

## 📊 Timeline

**Estimated Effort:** 2-3 hours
**Best Time:** Week 1-2
**Dependencies:** None (independent implementation)

### Week 1-2 Integration
- **Day 3-4:** Implement config module
- **Day 4-5:** Update test files
- **Day 5-6:** Update CI/CD and validate

---

**Owner:** Performance Testing Team
**Start Date:** Week 1-2 (after critical fixes)
**Target Completion:** Week 2
**Priority:** P2 (improves CI reliability)

*This guide provides a complete solution for environment-aware performance thresholds.*

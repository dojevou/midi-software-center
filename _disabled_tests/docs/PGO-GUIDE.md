# Profile-Guided Optimization (PGO) Guide

## Overview

**Profile-Guided Optimization (PGO)** is a compiler optimization technique that uses runtime profiling data to make better optimization decisions. For the MIDI Software Center, PGO can deliver **10-20% additional performance improvements** on top of standard release builds.

## Why PGO?

Traditional compiler optimizations make decisions based on static code analysis:
- "This function looks hot, let's inline it"
- "This branch seems likely, optimize for it"

PGO uses actual runtime data:
- "Real workloads call function X in these specific ways"
- "This branch is taken 95% of the time in production"
- "Memory access patterns cluster here"

**Result:** Better optimized code with real-world patterns in mind.

## Expected Performance Improvements

### By Component

| Component | Operation | Standard Release | PGO Gain | Final |
|-----------|-----------|-----------------|----------|-------|
| **Pipeline** | Import 100K files | 25.6s | 10-15% | ~22s |
| **Pipeline** | Analyze 100K files | 1,103s | 15-25% | ~850s |
| **DAW** | Sequencer rendering | 4.2ms | 5-10% | ~3.8ms |
| **DAW** | Real-time playback | 2.1ms | 8-12% | ~1.9ms |
| **Shared** | MIDI parsing | 0.8ms/file | 12-18% | ~0.68ms |
| **Database** | Query execution | 8.2ms | 10-15% | ~7.2ms |

### Realistic Expectations

**I/O-Bound Operations (File Import)**
- Current: 3,915 files/sec
- With PGO: ~4,300 files/sec (10% improvement)

**CPU-Bound Operations (MIDI Analysis)**
- Current: 90.5 files/sec
- With PGO: ~105 files/sec (16% improvement)

**Mixed Operations (DAW Sequencer)**
- Current: 8.2ms query time
- With PGO: ~7.4ms query time (10% improvement)

## How PGO Works

### Phase 1: Instrumentation Build
```
Source Code
    ↓
[LLVM with PGO instrumentation]
    ↓
Instrumented Binary + Tracking Code
```

The binary has extra code to record:
- Which functions are called most
- Which branches are taken
- Memory access patterns
- Loop iteration counts

### Phase 2: Profiling Workload
```
Instrumented Binary
    ↓
[Run on representative workload]
    ↓
Profile Data (.profraw files)
```

The instrumented binary executes your real workflows and collects statistics.

### Phase 3: Profile Merging
```
Multiple .profraw files
    ↓
[llvm-profdata merge]
    ↓
Unified Profile Data (.profdata)
```

Individual profiling runs are combined into a single profile.

### Phase 4: Optimized Build
```
Source Code + Profile Data
    ↓
[LLVM with PGO guidance]
    ↓
Optimized Binary
```

LLVM uses profiling data to make better optimization decisions:
- Inline hot functions
- Reorder code for cache locality
- Optimize for actual branch patterns
- Improve register allocation

## Installation & Setup

### Prerequisites

1. **Rust 1.71+** (PGO support)
   ```bash
   rustup update
   ```

2. **LLVM Tools**
   ```bash
   rustup component add llvm-tools-preview
   ```

3. **Disk Space**
   - 8GB+ available for build artifacts and profiling data

4. **Build Time**
   - Add 20-30% to total build time (instrumentation + profiling + optimized build)

### Quick Check

```bash
# Verify LLVM tools
llvm-profdata --version

# Verify Rust version
rustc --version  # Should be 1.71+

# Check disk space
df -h /path/to/project  # Need 8GB+
```

## Running PGO Builds

### Basic Usage

```bash
# PGO build for pipeline component
./scripts/pgo-build.sh pipeline

# PGO build for DAW component
./scripts/pgo-build.sh daw

# PGO build for both components
./scripts/pgo-build.sh all
```

### Advanced Usage

```bash
# Custom workload size (default: 100,000 files)
./scripts/pgo-build.sh pipeline 50000

# Build both with custom workload
./scripts/pgo-build.sh all 200000
```

### What the Script Does

**Step 1: Build Instrumented Binary**
- Compiles with PGO instrumentation flags
- Binary includes profiling code (minimal overhead)

**Step 2: Run Profiling Workload**
- Executes full test suite on instrumented binary
- Tests cover real-world usage patterns:
  - File import and parsing
  - BPM/key detection analysis
  - Database operations
  - Sequencer operations

**Step 3: Merge Profile Data**
- Combines .profraw profiling data into unified .profdata
- Profile captures which code paths are actually hot

**Step 4: Build Optimized Binary**
- Rebuilds using profiling guidance
- LLVM applies data-driven optimizations

## Workflow Integration

### Development Workflow

```bash
# Standard development (no PGO needed)
make dev-pipeline
make dev-daw

# Testing (no PGO needed)
make test

# Release builds

# Option 1: Standard release (5 min)
make release

# Option 2: PGO-optimized release (15-20 min)
./scripts/pgo-build.sh all
```

### Makefile Integration

Add to your `Makefile`:

```makefile
#=============================================================================
# PGO BUILDS
#=============================================================================

pgo-build:
	@echo "Building with Profile-Guided Optimization..."
	@./scripts/pgo-build.sh all

pgo-pipeline:
	@echo "PGO build for pipeline component..."
	@./scripts/pgo-build.sh pipeline

pgo-daw:
	@echo "PGO build for DAW component..."
	@./scripts/pgo-build.sh daw

pgo-clean:
	@echo "Cleaning PGO data..."
	@rm -rf pipeline/src-tauri/target/pgo-profile
	@rm -rf daw/src-tauri/target/pgo-profile
	@cargo clean
```

Then use:
```bash
make pgo-build      # Build both with PGO
make pgo-pipeline   # Build pipeline with PGO
make pgo-daw        # Build DAW with PGO
```

## Profiling the Right Workload

### Good Profiling Workloads

For **Pipeline Component:**
- Full import of 1,000+ MIDI files
- Batch analysis of diverse file types
- Archive extraction scenarios
- Database batch operations

For **DAW Component:**
- Loading and playing MIDI files
- Real-time sequencer rendering
- Note input and playback
- Project load/save operations

### Current Workload

The script uses the existing test suite:
```bash
cargo test --release -- --test-threads=1
```

This covers:
- ✅ File parsing (midly)
- ✅ BPM detection (analysis)
- ✅ Key detection (analysis)
- ✅ Auto-tagging (MIDI processing)
- ✅ Repository operations (database)
- ✅ Sequencer operations (DAW)

### Custom Profiling Workloads

To profile with custom workflows:

```bash
# Create custom profiling script
cat > /tmp/pgo-workload.sh << 'EOF'
#!/bin/bash
# Import 1000 MIDI files
cargo run --release -- import /path/to/files

# Analyze all files
cargo run --release -- analyze --all

# Run sequencer tests
cargo test --release -- sequencer
EOF

# Run before profile merge step
LLVM_PROFILE_FILE="target/pgo-profile/custom-%p-%m.profraw" bash /tmp/pgo-workload.sh
```

## Performance Measurement

### Before & After Comparison

```bash
# 1. Build standard release
make release

# 2. Benchmark standard version
time ./pipeline/src-tauri/target/release/midi-import-cli import /test/files

# Record time: X seconds

# 3. Build PGO version
./scripts/pgo-build.sh pipeline

# 4. Benchmark PGO version
time ./pipeline/src-tauri/target/release/midi-import-cli import /test/files

# Record time: Y seconds

# 5. Calculate improvement
# Improvement = (X - Y) / X * 100%
```

### Automated Benchmarking

```bash
# Run cargo benchmarks
cargo bench --release

# Compare with previous results
cargo bench --release -- --verbose
```

### Real-World Testing

```bash
# Test with actual MIDI collections
# Measure import time
# Measure analysis time
# Monitor memory usage
# Track CPU utilization
```

## Troubleshooting

### Issue: llvm-profdata not found

**Error:**
```
error: llvm-profdata: command not found
```

**Solution:**
```bash
rustup component add llvm-tools-preview
cargo install cargo-pgo
```

### Issue: No profiling data generated

**Error:**
```
error: No profiling data found in pipeline/src-tauri/target/pgo-profile
```

**Causes & Solutions:**
1. **Test suite didn't run** → Ensure tests execute completely
2. **Disk space full** → Free up space, retry
3. **Permissions issue** → Check directory permissions
4. **Wrong LLVM_PROFILE_FILE path** → Verify path is writable

```bash
# Debug: Check if profiling data exists
ls -la pipeline/src-tauri/target/pgo-profile/

# Debug: Run tests with verbose profiling
LLVM_PROFILE_FILE="$PWD/profile-%p-%m.profraw" \
cargo test --release -v
```

### Issue: Build fails with PGO flags

**Error:**
```
error: failed to compile with PGO flags
```

**Solutions:**
1. **Update Rust** → `rustup update`
2. **Clean build artifacts** → `cargo clean`
3. **Check Rust version** → Must be 1.71+

```bash
rustc --version
rustup update
```

### Issue: PGO optimizations not applied

**Error:**
```
Binary runs same speed as standard release
```

**Verification:**
1. Check profile-use flag is set:
   ```bash
   echo $RUSTFLAGS
   # Should contain: -C profile-use=/path/to/pgo-data.profdata
   ```

2. Verify profile data exists:
   ```bash
   ls -la pipeline/src-tauri/target/pgo-profile/pgo-data.profdata
   # Should be present and >0 bytes
   ```

3. Rebuild with fresh flags:
   ```bash
   ./scripts/pgo-build.sh pipeline
   ```

## Advanced Configuration

### Tuning PGO Parameters

**Instrumentation Level** (in script):
```bash
# Current: -pgo-warn-missing-function
# More aggressive: -pgo-function-lowering=true
# Less aggressive: none (use defaults)

RUSTFLAGS_BASE="-C llvm-args=-pgo-function-lowering=true"
```

**Code Generation Units** (in Cargo.toml):
```toml
[profile.release]
codegen-units = 1  # Better optimization (currently set)
# For faster PGO builds: codegen-units = 16
```

### Selective PGO

Apply PGO only to hot paths:

```toml
# In Cargo.toml - profile only specific packages
[profile.pgo]
inherits = "release"
# Custom settings
```

### Profile Data Retention

Keep profiling data for future rebuilds:

```bash
# Archive profiling data
tar -czf pgo-profiles-2025-11-11.tar.gz \
  pipeline/src-tauri/target/pgo-profile \
  daw/src-tauri/target/pgo-profile

# Restore from archive
tar -xzf pgo-profiles-2025-11-11.tar.gz
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: PGO Build

on:
  release:
    types: [published]

jobs:
  pgo-build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview

      - name: PGO Build
        run: ./scripts/pgo-build.sh all

      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: pgo-binaries
          path: |
            pipeline/src-tauri/target/release/bundle/
            daw/src-tauri/target/release/bundle/
```

### GitLab CI Example

```yaml
pgo-build:
  stage: build
  script:
    - rustup component add llvm-tools-preview
    - ./scripts/pgo-build.sh all
  artifacts:
    paths:
      - pipeline/src-tauri/target/release/bundle/
      - daw/src-tauri/target/release/bundle/
    expire_in: 30 days
```

## Best Practices

### Do's ✅

- **Do profile with representative workloads** - Real usage patterns matter
- **Do re-profile after major changes** - Code changes invalidate old profiles
- **Do keep profiling data** - Useful for comparison and archival
- **Do benchmark improvements** - Verify actual gains
- **Do integrate into CI/CD** - Automate PGO releases

### Don'ts ❌

- **Don't use PGO for development** - Adds build time without benefit
- **Don't ignore profiling workload quality** - Poor profiles = poor optimization
- **Don't replace other optimizations** - PGO works best with LTO
- **Don't expect 50%+ improvements** - 10-20% is realistic
- **Don't share profiles across versions** - Profiles are version-specific

## References

### Official Documentation
- [Rust PGO](https://doc.rust-lang.org/rustc/profile-guided-optimization.html)
- [LLVM PGO Guide](https://llvm.org/docs/HowToBuildWithPGO/)
- [Rustc Book - Profile-Guided Optimization](https://doc.rust-lang.org/rustc/profile-guided-optimization.html)

### Related Articles
- "Profile-Guided Optimization in Practice" - Rust Blog
- "How to use PGO for performance tuning" - LLVM Blog
- Cargo.toml Profile Documentation

### Tools & Utilities
- `llvm-profdata` - Profile data merger
- `cargo-pgo` - Cargo plugin for PGO
- `perf` - Linux performance profiler (for validation)
- `flamegraph` - Visualization tool

## Performance Tuning Checklist

Before deploying PGO build:

- [ ] Prerequisites verified (Rust 1.71+, LLVM tools, disk space)
- [ ] PGO script downloaded and executable
- [ ] Test workload validates real usage patterns
- [ ] Profile data generated successfully
- [ ] PGO-optimized binary created without errors
- [ ] Benchmarks show improvement (10-20% expected)
- [ ] Binary size acceptable (should be similar to standard release)
- [ ] No regression in functionality (run full test suite)
- [ ] Profiling data archived for future reference
- [ ] Documentation updated with measured improvements

## Summary

Profile-Guided Optimization provides **10-20% additional performance** with minimal risk:

| Aspect | Benefit |
|--------|---------|
| **Performance** | 10-20% faster execution |
| **Ease** | One-command build script |
| **Safety** | Standard release fallback |
| **Compatibility** | Drop-in binary replacement |
| **ROI** | Few hours of setup for lasting gains |

**Recommended for:** Production releases, performance-critical deployments, benchmark comparisons.

**Not recommended for:** Development builds, rapid iteration cycles.

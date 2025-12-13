# PGO Quick Start Guide

## 30-Second Overview

**Profile-Guided Optimization (PGO)** uses runtime data to optimize your code for **10-20% faster execution**. Takes 15-25 minutes to build.

## Quickest Start

```bash
# Build PGO-optimized binaries for both pipeline and DAW
make pgo-build

# Or individual components
make pgo-pipeline  # Just pipeline
make pgo-daw       # Just DAW
```

Done! Your optimized binaries are in `target/release/bundle/`

## What Just Happened?

1. **Compiled with instrumentation** - Binary tracks code execution
2. **Ran profiling workload** - Tests recorded which code paths are hot
3. **Merged profile data** - Combined results into profile database
4. **Built optimized binary** - LLVM used data to optimize code

## Expected Results

- **File Import:** 10-15% faster
- **MIDI Analysis:** 15-25% faster
- **DAW Playback:** 5-10% faster
- **Overall:** 10-20% faster on representative workloads

## Validate Improvements

```bash
# Before PGO (standard release)
time ./pipeline/src-tauri/target/release/midi-import-cli import files/

# After PGO (should be noticeably faster)
time ./pipeline/src-tauri/target/release/midi-import-cli import files/
```

## Cleanup

```bash
# Remove PGO data and rebuild with standard release
make pgo-clean
make release
```

## When to Use PGO

✅ **Do use for:**
- Production releases
- Performance-critical deployments
- Benchmark comparisons

❌ **Don't use for:**
- Development (adds 15-25 min build time)
- Rapid iteration
- One-off testing

## Troubleshooting

**Issue: "llvm-profdata not found"**
```bash
rustup component add llvm-tools-preview
```

**Issue: Build fails**
```bash
rustup update
cargo clean
make pgo-build
```

## Full Guide

See [PGO-GUIDE.md](PGO-GUIDE.md) for:
- Detailed explanation of PGO
- Advanced configuration
- CI/CD integration
- Performance measurement techniques
- Custom profiling workloads

## Summary

| Step | Time | What's Happening |
|------|------|-----------------|
| 1 | 2-5 min | Compile with instrumentation |
| 2 | 5-10 min | Run tests to profile code paths |
| 3 | 1 min | Merge profiling data |
| 4 | 5-10 min | Compile with optimizations |
| **Total** | **15-25 min** | **10-20% faster binary ready** |

**Result:** Production-optimized binaries with real-world performance data embedded.

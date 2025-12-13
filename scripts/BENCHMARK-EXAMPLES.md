# Benchmark Script Examples & Expected Output

## Example 1: First-Time Run

```bash
$ cd /home/dojevou/projects/midi-software-center
$ make docker-up
$ make db-migrate
$ ./scripts/benchmark-comparison.sh
```

### Expected Output

```
[INFO] [INFO] PRE-FLIGHT CHECKS
===============================================================================
[INFO] PRE-FLIGHT CHECKS
===============================================================================

[SUCCESS] Test data: 7 MIDI files found
[INFO] Testing database connection...
[SUCCESS] Database connection verified
[SUCCESS] All pre-flight checks passed!

[INFO] SETTING UP BENCHMARK ENVIRONMENT
===============================================================================
[INFO] SETTING UP BENCHMARK ENVIRONMENT
===============================================================================

[SUCCESS] Created temporary directories
[INFO] Preparing test dataset (7x replication)...
[SUCCESS] Test dataset prepared: 49 files (224 KB)
[INFO] Backing up database...
[SUCCESS] Database backed up to /tmp/midi-benchmark-12345/db-backup/pre-benchmark.sql

[INFO] BUILDING BASELINE ORCHESTRATOR
===============================================================================
[INFO] BUILDING BASELINE ORCHESTRATOR
===============================================================================

[INFO] Stashing current changes...
[INFO] Finding baseline commit...
[INFO] Using baseline commit: 9b24207
[SUCCESS] Checked out baseline commit
[INFO] Building baseline orchestrator (this may take 1-2 minutes)...
[SUCCESS] Baseline orchestrator built and copied

[INFO] BUILDING OPTIMIZED ORCHESTRATOR
===============================================================================
[INFO] BUILDING OPTIMIZED ORCHESTRATOR
===============================================================================

[INFO] Building optimized orchestrator...
[SUCCESS] Optimized orchestrator built and copied

[INFO] RUNNING BENCHMARKS
===============================================================================
[INFO] RUNNING BENCHMARKS
===============================================================================

[INFO] Running BASELINE...
[INFO] Clearing database...
[SUCCESS] Database cleared
[SUCCESS] BASELINE completed
[INFO] Running OPTIMIZED...
[INFO] Clearing database...
[SUCCESS] Database cleared
[SUCCESS] OPTIMIZED completed

[INFO] GENERATING BENCHMARK REPORT
===============================================================================
[INFO] GENERATING BENCHMARK REPORT
===============================================================================

[SUCCESS] Report generated: /tmp/benchmark-results.md

[INFO] BENCHMARK RESULTS
===============================================================================
[INFO] BENCHMARK RESULTS
===============================================================================

╔════════════════════════════════════════════════════════════════════════════╗
║                     MIDI PIPELINE OPTIMIZATION BENCHMARK                  ║
╚════════════════════════════════════════════════════════════════════════════╝

BASELINE (Previous Optimization)
  Files/Second:  120.50 f/s
  Total Time:    58.23s
  Files:         49
  Errors:        0

OPTIMIZED (Current Build)
  Files/Second:  480.75 f/s
  Total Time:    14.58s
  Files:         49
  Errors:        0

IMPROVEMENT
  Speedup:       3.99x faster
  Time Saved:    43.65s (74.9%)

[SUCCESS] EXCELLENT: Speedup exceeds 4x target!

[SUCCESS] Full report: /tmp/benchmark-results.md

[INFO] CLEANING UP
===============================================================================
[INFO] CLEANING UP
===============================================================================

[INFO] Removing temporary files...
[SUCCESS] Cleanup complete
[INFO] Restoring database from backup...
[SUCCESS] Database restored

[INFO] BENCHMARK COMPLETE
===============================================================================
[INFO] BENCHMARK COMPLETE
===============================================================================

Benchmark results saved to: /tmp/benchmark-results.md
```

## Example 2: Quick Test (No Backup)

```bash
$ ./scripts/benchmark-comparison.sh --sample-count 3 --no-backup
```

### Expected Output (Shorter)

```
[SUCCESS] Test data: 7 MIDI files found
[SUCCESS] Database connection verified
[SUCCESS] All pre-flight checks passed!

[SUCCESS] Created temporary directories
[SUCCESS] Test dataset prepared: 21 files (96 KB)

[INFO] BUILDING BASELINE ORCHESTRATOR
[SUCCESS] Baseline orchestrator built and copied

[INFO] BUILDING OPTIMIZED ORCHESTRATOR
[SUCCESS] Optimized orchestrator built and copied

[INFO] RUNNING BENCHMARKS

[SUCCESS] BASELINE completed
[SUCCESS] OPTIMIZED completed

BASELINE (Previous Optimization)
  Files/Second:  125.43 f/s
  Total Time:    16.78s
  Files:         21
  Errors:        0

OPTIMIZED (Current Build)
  Files/Second:  502.14 f/s
  Total Time:    4.19s
  Files:         21
  Errors:        0

IMPROVEMENT
  Speedup:       4.00x faster
  Time Saved:    12.59s (75.0%)

[SUCCESS] EXCELLENT: Speedup exceeds 4x target!
```

**Total time: ~30-40 seconds**

## Example 3: Production-Realistic Test

```bash
$ ./scripts/benchmark-comparison.sh --sample-count 32 --workers 16 --batch-size 2000
```

### Expected Output

```
[SUCCESS] Test data: 7 MIDI files found
[SUCCESS] Database connection verified
[SUCCESS] All pre-flight checks passed!

[SUCCESS] Created temporary directories
[SUCCESS] Test dataset prepared: 224 files (1 MB)

[INFO] BUILDING BASELINE ORCHESTRATOR
...
[SUCCESS] Baseline orchestrator built and copied

[INFO] BUILDING OPTIMIZED ORCHESTRATOR
...
[SUCCESS] Optimized orchestrator built and copied

[INFO] RUNNING BENCHMARKS

[SUCCESS] BASELINE completed
[SUCCESS] OPTIMIZED completed

BASELINE (Previous Optimization)
  Files/Second:  98.76 f/s
  Total Time:    226.55s
  Files:         224
  Errors:        0

OPTIMIZED (Current Build)
  Files/Second:  510.42 f/s
  Total Time:    43.91s
  Files:         224
  Errors:        0

IMPROVEMENT
  Speedup:       5.16x faster
  Time Saved:    182.64s (80.6%)

[SUCCESS] EXCELLENT: Speedup exceeds 4x target!

Estimated savings for real workloads:
  - 1,000 files:  saves ~814 seconds (13.6 minutes)
  - 10,000 files: saves ~8,140 seconds (2.3 hours)
  - 100,000 files: saves ~81,400 seconds (22.6 hours)
```

**Total time: ~5-10 minutes**

## Example 4: With Hyperfine (Statistical)

```bash
$ ./scripts/benchmark-comparison.sh --sample-count 16 --use-hyperfine
```

### Expected Output

```
[SUCCESS] hyperfine found (will use for statistical accuracy)
...
[INFO] RUNNING BENCHMARKS

[INFO] Running BASELINE (run 1/3)...
[SUCCESS] BASELINE completed
[INFO] Running BASELINE (run 2/3)...
[SUCCESS] BASELINE completed
[INFO] Running BASELINE (run 3/3)...
[SUCCESS] BASELINE completed

[INFO] Running OPTIMIZED (run 1/3)...
[SUCCESS] OPTIMIZED completed
[INFO] Running OPTIMIZED (run 2/3)...
[SUCCESS] OPTIMIZED completed
[INFO] Running OPTIMIZED (run 3/3)...
[SUCCESS] OPTIMIZED completed

BASELINE Results (3 runs):
  Time: 87.5s ± 2.3s (95% confidence)
  Files/Sec: 110.5 ± 2.9

OPTIMIZED Results (3 runs):
  Time: 17.2s ± 0.8s (95% confidence)
  Files/Sec: 561.3 ± 25.4

IMPROVEMENT
  Speedup:       5.09x faster
  Time Saved:    70.3s (80.3%)
  Confidence:    95%

[SUCCESS] EXCELLENT: Speedup exceeds 4x target!
```

**Total time: ~15-20 minutes (runs each version 3 times)**

## Example 5: Debug Mode (Keep Temp Files)

```bash
$ ./scripts/benchmark-comparison.sh --workers 1 --keep-temp
```

### Expected Output

```
...
[WARNING] Temporary files preserved (--keep-temp): /tmp/midi-benchmark-67890
```

### Resulting Files

```bash
$ ls -la /tmp/midi-benchmark-67890/
drwxrwxr-x   baseline-build/
  └── orchestrator           # Baseline binary (for profiling)
drwxrwxr-x   optimized-build/
  └── orchestrator           # Optimized binary (for profiling)
drwxrwxr-x   test-files/
  └── 49 MIDI files
drwxrwxr-x   db-backup/
  └── pre-benchmark.sql
-rw-r--r--   baseline.json    # Detailed results
-rw-r--r--   optimized.json   # Detailed results
-rw-r--r--   detailed-results.json
```

### Further Analysis

```bash
# Profile the binaries
flamegraph /tmp/midi-benchmark-67890/optimized-build/orchestrator \
  --source /tmp/midi-benchmark-67890/test-files

# Check detailed JSON results
jq . /tmp/midi-benchmark-67890/optimized.json

# Run orchestrator manually with logging
RUST_LOG=debug /tmp/midi-benchmark-67890/optimized-build/orchestrator \
  --source /tmp/midi-benchmark-67890/test-files \
  --workers 1
```

## Example 6: Compare Multiple Optimizations

```bash
#!/bin/bash
# Run benchmarks for different optimization approaches

for optimization in "main" "branch-v1" "branch-v2"; do
    echo "Testing $optimization..."
    git checkout $optimization

    ./scripts/benchmark-comparison.sh --sample-count 16 \
        --no-backup > /tmp/results-$optimization.txt

    # Extract key metric
    echo "$optimization speedup:" $(grep "Speedup:" /tmp/results-$optimization.txt)
done

# Compare results
echo ""
echo "Summary:"
grep "Speedup:" /tmp/results-*.txt
```

### Expected Output

```
Testing main...
[SUCCESS] Full report: /tmp/benchmark-results.md
main speedup: Speedup:       1.00x faster

Testing branch-v1...
[SUCCESS] Full report: /tmp/benchmark-results.md
branch-v1 speedup: Speedup:       2.45x faster

Testing branch-v2...
[SUCCESS] Full report: /tmp/benchmark-results.md
branch-v2 speedup: Speedup:       4.87x faster

Summary:
main speedup: Speedup:       1.00x faster
branch-v1 speedup: Speedup:       2.45x faster
branch-v2 speedup: Speedup:       4.87x faster
```

## Example 7: Integration with Makefile

Add to your `Makefile`:

```makefile
# Performance benchmarking
bench:
	@./scripts/benchmark-comparison.sh --sample-count 8 --no-backup
	@echo ""
	@echo "Results: cat /tmp/benchmark-results.md"

bench-detailed:
	@./scripts/benchmark-comparison.sh --sample-count 32 --workers 16

bench-stats:
	@if command -v hyperfine >/dev/null 2>&1; then \
		./scripts/benchmark-comparison.sh --sample-count 16 --use-hyperfine; \
	else \
		echo "Install hyperfine: cargo install hyperfine"; \
		./scripts/benchmark-comparison.sh --sample-count 16; \
	fi

.PHONY: bench bench-detailed bench-stats
```

Then use:

```bash
$ make bench              # Quick benchmark
$ make bench-detailed     # Full benchmark
$ make bench-stats        # Statistical benchmark (if hyperfine installed)
```

## Interpreting Results

### Good Results (4-6x speedup)

```
IMPROVEMENT
  Speedup:       5.12x faster     ← Excellent!
  Time Saved:    47.3s (81.1%)   ← Significant improvement
```

**Action**: Ready for production!

### Moderate Results (2-4x speedup)

```
IMPROVEMENT
  Speedup:       2.87x faster     ← Good progress
  Time Saved:    35.2s (60.2%)   ← Worthwhile improvement
```

**Action**: Production-ready with monitoring, consider more optimizations

### Minimal Results (<2x speedup)

```
IMPROVEMENT
  Speedup:       1.23x faster     ← Minimal improvement
  Time Saved:    4.5s (15.4%)    ← Small improvement
```

**Action**: Consider additional optimizations or different approach

### Regression (Slower)

```
IMPROVEMENT
  Speedup:       0.67x faster     ← Slower!
  Time Saved:    -23.5s           ← Performance loss
```

**Action**: Debug and revert changes

## Common Configuration Patterns

### Development (Fast Feedback)
```bash
./scripts/benchmark-comparison.sh --sample-count 3 --no-backup
```
Time: 30-40 seconds | Use: During development

### CI/CD (Reliable)
```bash
./scripts/benchmark-comparison.sh --sample-count 8 --batch-size 1000
```
Time: 1-2 minutes | Use: Automated testing

### Production Validation
```bash
./scripts/benchmark-comparison.sh --sample-count 32 --workers 16
```
Time: 5-10 minutes | Use: Pre-deployment validation

### Statistical Analysis
```bash
./scripts/benchmark-comparison.sh --sample-count 16 --use-hyperfine
```
Time: 15-20 minutes | Use: Publication/documentation

### Performance Investigation
```bash
./scripts/benchmark-comparison.sh --workers 1 --keep-temp
```
Time: 2-5 minutes | Use: Debugging slowdowns

## Reporting Results

### Template for Git Commits

```
perf(orchestrator): add parallel import optimization

Benchmark results (49 MIDI files):
  - Baseline:   120.5 f/s, 58.2s
  - Optimized:  480.8 f/s, 14.6s
  - Speedup:    3.99x faster (75% improvement)

Details: /tmp/benchmark-results.md
```

### Template for PR Description

```markdown
## Performance Impact

Benchmark comparison on 49 MIDI files:

| Metric | Baseline | Optimized | Change |
|--------|----------|-----------|--------|
| Throughput | 120.5 f/s | 480.8 f/s | 3.99x |
| Total Time | 58.2s | 14.6s | -75% |
| Errors | 0 | 0 | ✓ |

Results: [benchmark-results.md](/tmp/benchmark-results.md)
```

---

**Need Help?**
- Quick answers: `scripts/BENCHMARK-QUICKSTART.md`
- Detailed guide: `scripts/BENCHMARK-GUIDE.md`
- Troubleshooting: See FAQ in BENCHMARK-GUIDE.md

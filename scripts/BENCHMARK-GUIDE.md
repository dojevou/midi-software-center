# MIDI Pipeline Optimization Benchmark Guide

## Overview

The `benchmark-comparison.sh` script provides a comprehensive benchmarking framework to measure the performance of optimization improvements to the MIDI pipeline orchestrator.

**Key Features:**
- Compares baseline vs optimized performance
- Automatically manages database state (backup/clear/restore)
- Captures detailed timing and throughput metrics
- Generates professional benchmark reports
- Optional statistical accuracy with hyperfine
- Configurable worker threads and batch sizes

## Quick Start

### Prerequisites

1. **PostgreSQL running** on localhost:5433
```bash
make docker-up
```

2. **Release build compiled** (optional - script builds automatically)
```bash
cargo build --release -p midi_pipeline --bin orchestrator
```

3. **Database migrations run**
```bash
make db-migrate
```

### Basic Usage

```bash
./scripts/benchmark-comparison.sh
```

This will:
- Back up your current database
- Build baseline orchestrator from git history
- Build optimized orchestrator from current code
- Run both with 7 MIDI files, 4 workers, batch size 1000
- Generate `/tmp/benchmark-results.md` with detailed comparison
- Restore your database

### Advanced Usage

```bash
# Use 16 workers for high-performance test
./scripts/benchmark-comparison.sh --workers 16

# Use 32 test file replications (more realistic load)
./scripts/benchmark-comparison.sh --sample-count 32

# Use larger batch size for import phase
./scripts/benchmark-comparison.sh --batch-size 2000

# Use hyperfine for statistical accuracy (3 runs, confidence intervals)
./scripts/benchmark-comparison.sh --use-hyperfine

# Skip database backup (faster, risky!)
./scripts/benchmark-comparison.sh --no-backup

# Keep temporary files for debugging
./scripts/benchmark-comparison.sh --keep-temp
```

### Common Combinations

```bash
# Quick smoke test (30 seconds total)
./scripts/benchmark-comparison.sh --sample-count 3 --no-backup

# Production-realistic test (large dataset, many workers)
./scripts/benchmark-comparison.sh --sample-count 32 --workers 16 --batch-size 2000

# Statistical benchmark (3 runs per version, confidence intervals)
./scripts/benchmark-comparison.sh --use-hyperfine --sample-count 16 --keep-temp

# Debug optimization (keep temp files, single worker)
./scripts/benchmark-comparison.sh --workers 1 --keep-temp
```

## Configuration

### Environment Variables

Set these before running the script to customize database connection:

```bash
# Database connection (defaults shown)
export DB_HOST="localhost"
export DB_PORT="5433"
export DB_USER="midiuser"
export DB_NAME="midi_library"
export DB_PASS="145278963"

# Then run script
./scripts/benchmark-comparison.sh
```

### Command-Line Options

| Option | Default | Description |
|--------|---------|-------------|
| `--sample-count N` | 7 | Number of MIDI files to process (replicates test data) |
| `--workers N` | 4 | Worker thread count for orchestrator |
| `--batch-size N` | 1000 | Batch size for database inserts |
| `--use-hyperfine` | false | Use hyperfine for statistical accuracy |
| `--no-backup` | false | Skip database backup (faster but risky) |
| `--keep-temp` | false | Preserve temporary files after benchmark |
| `--help` | - | Show help message |

## Understanding Results

### Performance Metrics

The benchmark measures:

1. **Elapsed Time** (seconds)
   - Total wall-clock time for orchestrator execution
   - Lower is better

2. **Files Processed**
   - Number of MIDI files processed in test run
   - Should match or exceed sample count

3. **Files/Second (Throughput)**
   - Key metric: files processed per second
   - Higher is better
   - Expected: 360-540 f/s (4-6x baseline of 90.5 f/s)

4. **Errors**
   - Import errors
   - Analysis errors
   - Should be 0 for clean run

5. **Speedup Ratio**
   - `Optimized throughput / Baseline throughput`
   - 1.0 = same
   - 2.0 = 2x faster
   - 4.0 = 4x faster (target)

### Example Report Output

```
BASELINE (Previous Optimization)
  Files/Second:  120.50 f/s
  Total Time:    58.23s
  Files:         7
  Errors:        0

OPTIMIZED (Current Build)
  Files/Second:  480.75 f/s
  Total Time:    14.58s
  Files:         7
  Errors:        0

IMPROVEMENT
  Speedup:       3.99x faster
  Time Saved:    43.65s (74.9%)
```

## Interpreting Results

### Success Criteria

| Speedup | Status | Action |
|---------|--------|--------|
| > 4.0x | Excellent | Deploy to production |
| 2.0x - 4.0x | Good | Ready for production with monitoring |
| 1.5x - 2.0x | Positive | Monitor performance in staging |
| 1.0x - 1.5x | Minimal | Consider additional optimizations |
| < 1.0x | Regression | Debug and revert changes |

### Analyzing Slowdown Scenarios

If optimized is slower than baseline:

1. **Check for bottlenecks**
   ```bash
   ./scripts/benchmark-comparison.sh --keep-temp --workers 1
   # Review /tmp/midi-benchmark-*/detailed-results.json
   ```

2. **Profile the code**
   ```bash
   cargo flamegraph --bin orchestrator -- \
     --source /path/to/test-files --workers 4
   ```

3. **Compare database operations**
   ```bash
   # Enable query logging
   export RUST_LOG=sqlx=debug
   ./scripts/benchmark-comparison.sh --keep-temp
   ```

## Test Data

### Default Test Data Location

```
pipeline/src-tauri/src/core/analysis/tests/resources/real_world_drums/
├── dnb_160_electronic.mid     (1.2 KB)
├── funk_120_shuffle.mid       (195 B)
├── jazz_136_swing.mid         (234 B)
├── metal_triplet.mid          (375 B)
├── odd_meter_5_4.mid          (420 B)
├── punk_200_fast.mid          (584 B)
└── test_punk_200.mid          (584 B)

Total: 7 files, 32 KB
```

### Scaling Test Data

The `--sample-count` parameter replicates test files:

| Count | Total Files | Est. Size | Est. Time |
|-------|------------|-----------|-----------|
| 3 | 21 | 96 KB | ~30s |
| 7 | 49 | 224 KB | ~60s |
| 16 | 112 | 512 KB | ~120s |
| 32 | 224 | 1 MB | ~240s |
| 64 | 448 | 2 MB | ~480s |

## Output Files

### Benchmark Results

**Main Report:**
```
/tmp/benchmark-results.md
```
Contains:
- Performance comparison table
- JSON outputs from both runs
- Analysis and recommendations
- System information

### Temporary Files (if `--keep-temp`)

```
/tmp/midi-benchmark-{PID}/
├── baseline-build/
│   └── orchestrator          # Baseline binary
├── optimized-build/
│   └── orchestrator          # Optimized binary
├── test-files/
│   └── *.mid                 # Test MIDI files
├── db-backup/
│   └── pre-benchmark.sql     # Database backup
├── baseline.json             # Baseline metrics
├── optimized.json            # Optimized metrics
└── detailed-results.json     # Combined results
```

## Database Management

### Backup and Restore

The script automatically:
1. **Before benchmark**: Backs up entire database to `/tmp/midi-benchmark-{PID}/db-backup/`
2. **During benchmark**: Clears files/metadata tables
3. **After benchmark**: Restores from backup

To manually restore:
```bash
DB_BACKUP="/tmp/midi-benchmark-12345/db-backup/pre-benchmark.sql"
PGPASSWORD="145278963" psql \
  -h localhost -p 5433 \
  -U midiuser -d midi_library \
  < "$DB_BACKUP"
```

### Skip Backup (Faster)

```bash
./scripts/benchmark-comparison.sh --no-backup
```

This is faster but:
- Original data is lost during test
- Database is NOT restored after benchmark
- Use only on test instances

## Troubleshooting

### Error: "Cannot connect to database"

```bash
# Check PostgreSQL is running
docker-compose ps postgres

# Start if needed
make docker-up

# Verify connection
psql -h localhost -p 5433 -U midiuser -d midi_library -c "SELECT 1;"
```

### Error: "Test data directory not found"

```bash
# Verify test files exist
ls -la pipeline/src-tauri/src/core/analysis/tests/resources/real_world_drums/

# If missing, create sample MIDI files
# (or run: cargo test --test-threads=1 to generate them)
```

### Error: "Baseline build failed"

```bash
# Check git history for stable commit
git log --oneline -20

# Build manually with verbose output
cargo build --release -p midi_pipeline --bin orchestrator -vv
```

### Error: "hyperfine not found"

Install with:
```bash
cargo install hyperfine
```

Or omit `--use-hyperfine` flag (script will use simple `time` instead)

## Best Practices

### Running Benchmarks

1. **Close other applications** to minimize noise
2. **Use consistent configuration** across runs
3. **Run multiple times** to account for variation
4. **Keep baseline stable** - don't commit changes between runs
5. **Test on production hardware** for realistic results

### Comparing Optimizations

```bash
# Run baseline
git stash
./scripts/benchmark-comparison.sh --sample-count 16

# Record results
cp /tmp/benchmark-results.md /tmp/baseline-results.md

# Apply optimization
git stash pop

# Run optimized
./scripts/benchmark-comparison.sh --sample-count 16

# Compare
diff /tmp/baseline-results.md /tmp/benchmark-results.md
```

### Continuous Integration

```makefile
# Add to Makefile
bench:
	./scripts/benchmark-comparison.sh --sample-count 8

bench-detailed:
	./scripts/benchmark-comparison.sh --sample-count 32 --use-hyperfine

.PHONY: bench bench-detailed
```

Run on every commit:
```bash
make bench
```

## Performance Expectations

### Baseline Performance
- **Current**: ~90.5 files/sec (Phase 9 validation)
- **Dataset**: 1,603 real MIDI files
- **Throughput**: 17.7 seconds

### Optimization Targets
- **Goal**: 4-6x improvement
- **Target**: 360-540 files/sec
- **For 1,603 files**: 3-4.5 seconds total

### Observed Performance
- **Phase 10 Early**: Initial improvements showing 2-3x
- **Target Hit**: 4-6x expected with all optimizations

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: Performance Benchmark
on: [push]
jobs:
  benchmark:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_USER: midiuser
          POSTGRES_PASSWORD: 145278963
          POSTGRES_DB: midi_library
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run Benchmark
        run: ./scripts/benchmark-comparison.sh --sample-count 8
      - name: Upload Results
        uses: actions/upload-artifact@v2
        with:
          name: benchmark-results
          path: /tmp/benchmark-results.md
```

## Related Commands

```bash
# Quick test (no benchmark)
make test-quick

# Coverage analysis
make test-coverage

# Profile with flamegraph
cargo flamegraph --bin orchestrator

# Run orchestrator manually
./target/release/orchestrator --source /path/to/files --workers 8

# View database stats
psql -U midiuser -d midi_library -c "SELECT COUNT(*) FROM files;"
```

## FAQ

**Q: How long does a benchmark take?**
A: 2-5 minutes depending on sample count and options. Larger samples take longer.

**Q: Can I run benchmarks on production data?**
A: Not recommended - benchmark clears database. Use `--no-backup` only on test instances.

**Q: Why is speedup not 4-6x?**
A: Check for:
- Database bottlenecks (use `EXPLAIN ANALYZE`)
- I/O limitations (disk speed)
- CPU throttling (check `lscpu`, `turbostat`)
- Lock contention (monitor with `pg_stat_activity`)

**Q: Can I use different test data?**
A: Yes - modify `TEST_DATA_DIR` variable in script or pass custom path via environment.

**Q: How do I integrate with my CI pipeline?**
A: See "Integration with CI/CD" section above.

**Q: What if benchmark crashes midway?**
A: Database will be restored from backup automatically (unless `--no-backup` used).

## Support

For issues or improvements:

1. Check logs in `/tmp/baseline-build.log` and `/tmp/optimized-build.log`
2. Review temporary files with `--keep-temp` flag
3. Enable debug output: `RUST_LOG=debug ./scripts/benchmark-comparison.sh`
4. Check orchestrator help: `./target/release/orchestrator --help`

---

**Last Updated**: 2025-11-11
**Script Version**: 1.0
**Tested On**: Linux (Ubuntu 20.04+), PostgreSQL 16

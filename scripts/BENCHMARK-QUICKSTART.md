# Benchmark Quick Start

## Run a Benchmark Right Now

```bash
cd /home/dojevou/projects/midi-software-center
make docker-up          # Ensure database is running
./scripts/benchmark-comparison.sh
```

That's it! Results appear in `/tmp/benchmark-results.md`

## Pre-Flight Checklist

```bash
# 1. Ensure database is running
docker-compose ps postgres

# 2. Ensure migrations are run
make db-migrate

# 3. Optional: Ensure release build exists
cargo build --release -p midi_pipeline --bin orchestrator
```

## Common Scenarios

### Scenario 1: Quick Test (30 seconds)
```bash
./scripts/benchmark-comparison.sh --sample-count 3 --no-backup
```
- Uses 3 MIDI files (minimal dataset)
- Skips database backup (faster)
- **Total time**: ~30-60 seconds

### Scenario 2: Standard Benchmark (2-3 minutes)
```bash
./scripts/benchmark-comparison.sh --sample-count 7 --workers 4
```
- Uses 7 MIDI files
- Backs up/restores database
- **Total time**: ~2-3 minutes
- **Result**: Reliable performance metrics

### Scenario 3: Production-Realistic (5-10 minutes)
```bash
./scripts/benchmark-comparison.sh --sample-count 32 --workers 16 --batch-size 2000
```
- Uses 32 MIDI files (224 total files)
- High worker count
- **Total time**: ~5-10 minutes
- **Result**: Production-like performance

### Scenario 4: Statistical Accuracy (10-15 minutes)
```bash
./scripts/benchmark-comparison.sh --sample-count 16 --use-hyperfine
```
- Requires: `cargo install hyperfine`
- Runs 3 times per version
- Calculates confidence intervals
- **Total time**: ~10-15 minutes
- **Result**: Statistically valid comparison

### Scenario 5: Debug/Investigation (3-5 minutes)
```bash
./scripts/benchmark-comparison.sh --workers 1 --keep-temp
```
- Single worker (easier to profile)
- Keeps temporary files for analysis
- **Output**: `/tmp/midi-benchmark-{PID}/`
- Use for flamegraph, detailed logging, etc.

## Understanding the Output

```
BASELINE (Previous Optimization)
  Files/Second:  120.50 f/s        ← Baseline throughput
  Total Time:    58.23s            ← How long it took
  Files:         7                 ← Files processed
  Errors:        0                 ← Should be 0

OPTIMIZED (Current Build)
  Files/Second:  480.75 f/s        ← Optimized throughput
  Total Time:    14.58s            ← Much faster!
  Files:         7
  Errors:        0

IMPROVEMENT
  Speedup:       3.99x faster      ← KEY METRIC: Compare to target (4-6x)
  Time Saved:    43.65s (74.9%)    ← Percentage improvement
```

**What you want to see:**
- **Speedup: 4-6x** (or higher)
- **Errors: 0**
- **Time Saved: 75-83%**

## Troubleshooting

### "Cannot connect to database"
```bash
make docker-up
make db-migrate
```

### "Test data directory not found"
```bash
# Verify files exist
ls pipeline/src-tauri/src/core/analysis/tests/resources/real_world_drums/
```

### "Baseline build failed"
```bash
# Clean and rebuild
cargo clean
cargo build --release -p midi_pipeline --bin orchestrator
```

## Key Metrics

| Metric | What It Means | Target |
|--------|---------------|--------|
| **Files/Second** | Throughput | >360 f/s |
| **Speedup** | Performance ratio | 4-6x |
| **Errors** | Data integrity | 0 |
| **Elapsed Time** | Total duration | <5s for 7 files |

## Next Steps

1. **Run benchmark**: `./scripts/benchmark-comparison.sh`
2. **Check results**: `cat /tmp/benchmark-results.md`
3. **If speedup < 4x**: Debug with `--keep-temp --workers 1`
4. **If speedup >= 4x**: Ready to deploy!

## Links

- **Full Guide**: `scripts/BENCHMARK-GUIDE.md`
- **Orchestrator Help**: `./target/release/orchestrator --help`
- **Source Code**: `pipeline/src-tauri/src/bin/orchestrator.rs`
- **Database Setup**: `database/` directory

---

**Pro Tip**: Run benchmark before and after each optimization, track improvements in a spreadsheet!

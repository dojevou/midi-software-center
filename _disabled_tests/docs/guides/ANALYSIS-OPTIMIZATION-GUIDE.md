# Analysis Phase Optimization Guide

## Current Performance
- **Speed:** 90.5 files/sec
- **Time for 5.8M files:** ~17-18 hours
- **Bottlenecks:** MIDI parsing (I/O), BPM detection (CPU), Database writes (I/O)

## Rust Optimization Strategies

### 1. Memory-Mapped Files (High Impact)

**Current:** `std::fs::read()` - Full file into memory
**Optimized:** `memmap2` - Zero-copy memory mapping

```rust
// Add to Cargo.toml
memmap2 = "0.9"

// In analyze_file():
use memmap2::Mmap;
use std::fs::File;

async fn analyze_file(pool: &Pool<Postgres>, file: &FileRecord) -> Result<()> {
    // Memory-map the file (zero-copy, kernel manages paging)
    let file_handle = File::open(&file.filepath)?;
    let mmap = unsafe { Mmap::map(&file_handle)? };

    // Parse directly from memory-mapped region
    let midi_file = parse_midi_file(&mmap)?;

    // Rest of analysis...
}
```

**Expected speedup:** 20-30% (reduced syscall overhead, better cache utilization)

---

### 2. SIMD Vectorization (Medium-High Impact)

**Use case:** BPM detection, audio feature extraction

```rust
// Add to Cargo.toml
packed_simd = "0.3"  // or use std::simd when stable

// Vectorized note onset detection (BPM)
use packed_simd::*;

fn detect_onsets_simd(velocities: &[u8]) -> Vec<f32> {
    let mut onsets = Vec::with_capacity(velocities.len());

    // Process 32 velocities at once
    for chunk in velocities.chunks(32) {
        let v = u8x32::from_slice_unaligned(chunk);
        let threshold = u8x32::splat(64);

        // Vectorized comparison: velocity > threshold
        let mask = v.gt(threshold);

        // Convert to onsets
        onsets.extend(mask.bitmask().to_le_bytes().iter()
            .enumerate()
            .filter(|(_, &b)| b != 0)
            .map(|(i, _)| i as f32));
    }

    onsets
}
```

**Expected speedup:** 2-4x for audio processing hot paths

---

### 3. Batch Database Operations (High Impact)

**Current:** Insert one file's metadata at a time
**Optimized:** Batch insert multiple files

```rust
// Current: 1 file = 1 INSERT
async fn analyze_file(...) {
    sqlx::query("INSERT INTO musical_metadata ...").execute(pool).await?;
}

// Optimized: Accumulate results, batch insert
struct AnalysisResult {
    file_id: i64,
    tempo_bpm: Option<f64>,
    detected_key: String,
    // ... all fields
}

async fn analyze_batch(
    pool: &Pool<Postgres>,
    files: &[FileRecord]
) -> Result<Vec<AnalysisResult>> {
    // Analyze all files in parallel (CPU-bound)
    let results: Vec<_> = files.par_iter()
        .map(|file| analyze_single_file(file))
        .collect();

    // Batch database insert (I/O-bound)
    let mut query_builder = QueryBuilder::new(
        "INSERT INTO musical_metadata (file_id, tempo_bpm, detected_key, ...)"
    );

    query_builder.push_values(&results, |mut b, result| {
        b.push_bind(result.file_id)
         .push_bind(result.tempo_bpm)
         .push_bind(&result.detected_key);
         // ... more fields
    });

    query_builder.build().execute(pool).await?;

    // Batch update analyzed_at
    let ids: Vec<_> = results.iter().map(|r| r.file_id).collect();
    sqlx::query("UPDATE files SET analyzed_at = NOW() WHERE id = ANY($1)")
        .bind(&ids)
        .execute(pool)
        .await?;

    Ok(results)
}
```

**Expected speedup:** 3-5x (fewer round trips, amortized overhead)

---

### 4. Custom Memory Allocator (Medium Impact)

**Current:** System allocator (glibc malloc)
**Optimized:** jemalloc or mimalloc

```rust
// Add to Cargo.toml
[dependencies]
tikv-jemallocator = "0.5"

// In main.rs or lib.rs
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
```

**Why it helps:**
- Better multi-threaded scaling (thread-local arenas)
- Reduced fragmentation
- Better cache locality

**Expected speedup:** 10-20% overall throughput

---

### 5. Target-Specific Optimizations (Medium Impact)

```toml
# In Cargo.toml
[profile.release]
codegen-units = 1      # Better optimization at cost of compile time
lto = "fat"            # Full LTO instead of thin
opt-level = 3          # Maximum optimizations
panic = "abort"        # Smaller binary, faster unwinding

# In .cargo/config.toml
[build]
rustflags = [
    "-C", "target-cpu=native",       # Use all CPU features (AVX2, etc.)
    "-C", "link-arg=-fuse-ld=mold",  # Faster linker
]
```

**Expected speedup:** 15-25% (utilizing AVX2, BMI2, etc.)

---

### 6. Reduce Allocations (Medium Impact)

**Use object pooling for hot paths:**

```rust
use once_cell::sync::Lazy;
use std::sync::Mutex;

// Pre-allocate reusable buffers
static BUFFER_POOL: Lazy<Mutex<Vec<Vec<u8>>>> = Lazy::new(|| {
    Mutex::new((0..32).map(|_| Vec::with_capacity(65536)).collect())
});

async fn analyze_file_pooled(pool: &Pool<Postgres>, file: &FileRecord) -> Result<()> {
    // Rent a buffer from pool
    let mut buffer = BUFFER_POOL.lock().unwrap().pop()
        .unwrap_or_else(|| Vec::with_capacity(65536));

    // Use buffer for file I/O
    buffer.clear();
    let mut file = File::open(&file.filepath)?;
    file.read_to_end(&mut buffer)?;

    let midi_file = parse_midi_file(&buffer)?;

    // Analysis...

    // Return buffer to pool
    BUFFER_POOL.lock().unwrap().push(buffer);

    Ok(())
}
```

**Expected speedup:** 5-10% (reduced allocator pressure)

---

### 7. Prefetching + Pipeline Parallelism (High Impact)

**Strategy:** Overlap I/O and CPU work

```rust
use tokio::sync::mpsc;

async fn analyze_phase_pipelined(
    pool: Pool<Postgres>,
    worker_count: usize,
) -> Result<()> {
    // Stage 1: File reading (I/O-bound)
    let (file_tx, file_rx) = mpsc::channel(worker_count * 4);

    tokio::spawn(async move {
        while let Some(file_record) = fetch_next_file().await {
            let data = tokio::fs::read(&file_record.filepath).await?;
            file_tx.send((file_record, data)).await?;
        }
    });

    // Stage 2: MIDI parsing + Analysis (CPU-bound)
    let (result_tx, result_rx) = mpsc::channel(worker_count * 2);

    for _ in 0..worker_count {
        let mut file_rx = file_rx.clone();
        let result_tx = result_tx.clone();

        tokio::task::spawn_blocking(move || {
            while let Some((record, data)) = file_rx.blocking_recv() {
                let result = analyze_midi_data(&data);
                result_tx.blocking_send((record, result)).unwrap();
            }
        });
    }

    // Stage 3: Database writes (I/O-bound, batched)
    let mut batch = Vec::with_capacity(100);
    while let Some((record, result)) = result_rx.recv().await {
        batch.push((record, result));

        if batch.len() >= 100 {
            batch_insert_results(&pool, &batch).await?;
            batch.clear();
        }
    }

    Ok(())
}
```

**Expected speedup:** 2-3x (CPU and I/O overlap, no idle time)

---

### 8. Cache-Friendly Data Structures (Low-Medium Impact)

```rust
// Instead of Vec<Box<MidiEvent>>
// Use flat arena allocation
use typed_arena::Arena;

struct MidiAnalyzer<'a> {
    event_arena: &'a Arena<MidiEvent>,
    // ...
}

impl<'a> MidiAnalyzer<'a> {
    fn analyze(&self, midi: &MidiFile) -> Analysis {
        // Allocate events in contiguous memory
        let events: Vec<&MidiEvent> = midi.tracks.iter()
            .flat_map(|t| t.events.iter())
            .map(|e| self.event_arena.alloc(e.clone()))
            .collect();

        // Better cache locality during iteration
        // ...
    }
}
```

**Expected speedup:** 5-15% for large files

---

### 9. Profile-Guided Optimization (PGO) (Medium-High Impact)

```bash
# Step 1: Build with instrumentation
RUSTFLAGS="-Cprofile-generate=/tmp/pgo-data" \
    cargo build --release

# Step 2: Run on representative workload
./target/release/orchestrator --source /sample/midi/files

# Step 3: Merge profile data
llvm-profdata merge -o /tmp/pgo-data/merged.profdata /tmp/pgo-data

# Step 4: Build with PGO
RUSTFLAGS="-Cprofile-use=/tmp/pgo-data/merged.profdata" \
    cargo build --release
```

**Expected speedup:** 10-20% (better inlining, branch prediction)

---

### 10. Parallel Database Connection Pool (Medium Impact)

```rust
// Current: Pool size = workers + 2
let pool = PgPoolOptions::new()
    .max_connections(args.workers as u32 + 2)
    .connect(&database_url).await?;

// Optimized: One connection per worker
let pool = PgPoolOptions::new()
    .max_connections(args.workers as u32)
    .min_connections(args.workers as u32)  // Keep connections warm
    .acquire_timeout(Duration::from_secs(10))
    .idle_timeout(None)  // Never close idle connections
    .max_lifetime(None)  // Reuse indefinitely
    .connect(&database_url).await?;
```

**Expected speedup:** 10-15% (reduced connection overhead)

---

## Combined Optimization Roadmap

### Phase 1: Quick Wins (1-2 hours implementation)
- ✅ Memory-mapped files
- ✅ Batch database operations
- ✅ Custom allocator (jemalloc)
- ✅ Target-specific compilation flags

**Expected combined speedup:** 2-3x (90.5 → 180-270 files/sec)
**New time estimate:** 6-9 hours for 5.8M files

### Phase 2: Advanced (4-6 hours implementation)
- ✅ Pipeline parallelism (prefetching)
- ✅ SIMD vectorization for BPM detection
- ✅ Buffer pooling
- ✅ Connection pool tuning

**Expected combined speedup:** 4-6x (90.5 → 360-540 files/sec)
**New time estimate:** 3-4.5 hours for 5.8M files

### Phase 3: Expert (8+ hours implementation)
- ✅ Profile-guided optimization
- ✅ Cache-friendly data layouts
- ✅ Inline assembly for critical paths
- ✅ Custom MIDI parser optimizations

**Expected combined speedup:** 8-12x (90.5 → 720-1,080 files/sec)
**New time estimate:** 1.5-2.5 hours for 5.8M files

---

## Implementation Priority

1. **Memory-mapped files** - Easy, high impact
2. **Batch database operations** - Medium difficulty, very high impact
3. **jemalloc allocator** - One line change, free speedup
4. **Target-specific flags** - Config file change, free speedup
5. **Pipeline parallelism** - Medium-hard, high impact
6. **SIMD vectorization** - Hard, high impact on CPU-bound sections
7. **PGO** - Easy process, medium-high impact

---

## Measurement & Profiling

```bash
# Before optimization: Baseline measurement
cargo build --release
time ./target/release/orchestrator --source /sample --workers 32

# Profile CPU usage
cargo install cargo-flamegraph
cargo flamegraph --bin orchestrator -- --source /sample --workers 32

# Profile memory allocations
cargo install cargo-instruments
cargo instruments --template Allocations --bin orchestrator

# Profile cache misses
perf record -e cache-misses ./target/release/orchestrator --source /sample
perf report

# Database query profiling
EXPLAIN (ANALYZE, BUFFERS) SELECT ...
```

---

## Expected Final Performance

**Current:** 90.5 files/sec → 17-18 hours
**After Phase 1:** 180-270 files/sec → 6-9 hours (-60% time)
**After Phase 2:** 360-540 files/sec → 3-4.5 hours (-75% time)
**After Phase 3:** 720-1,080 files/sec → 1.5-2.5 hours (-85% time)

**Best case:** Sub-2-hour complete analysis of 5.8M files!

# Batch Split Optimization Analysis

## Current Bottlenecks (37.2% complete after 5h10m)

### Resource Utilization
- **CPU**: 39.5% (61% headroom available)
- **Memory**: 0.3% (plenty available)
- **Disk I/O**: sdb at 83.57% util (slow /tmp disk)
- **Database**: 52 idle connections, only 2 active (98% underutilized)

### Current Settings
- Workers: 24
- Batch size: 100
- Max DB connections: 29 (workers + 5)
- Output: /tmp/midi_splits (on slow sdb drive)

### Processing Rate
- Current: ~730 files/min (~43,700/hour)
- ETA: ~8.7 hours remaining

---

## Optimization Strategies

### 1. **Move Output to Faster Drive** (EASIEST - 2-3x speedup)

**Problem**: Writing to /tmp on sdb (83% utilized, slow)
**Solution**: Write to NVMe drive (only 9.45% utilized)

```bash
# Kill current process
kill 1473548

# Restart with faster output location
mkdir -p /home/dojevou/tmp/midi_splits_fast
DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library" \
  ./target/release/batch_split \
  --output-dir /home/dojevou/tmp/midi_splits_fast \
  --workers 24 \
  --batch-size 100 > /tmp/batch_split_log_fast.txt 2>&1 &
```

**Expected improvement**: 2-3x faster (NVMe vs HDD)

---

### 2. **Increase Batch Size** (MEDIUM - 1.5-2x speedup)

**Problem**: Batch size 100 = many small database queries
**Solution**: Increase to 1000 for fewer round trips

```bash
--batch-size 1000  # Instead of 100
```

**Expected improvement**: 1.5-2x fewer database queries

---

### 3. **Increase Workers** (EASY - 1.2-1.5x speedup)

**Problem**: Only 39.5% CPU usage with 24 workers
**Solution**: Increase to 48 workers to saturate CPU

```bash
--workers 48
```

Also increase DB connections:
```rust
.max_connections(args.workers as u32 + 10)  // Was +5, now +10
```

**Expected improvement**: 1.2-1.5x (until CPU saturated)

---

### 4. **Skip Duplicate Checks on Disk** (ADVANCED - 1.3-1.5x speedup)

**Problem**: Checking for duplicates, then deleting 49% of written files
**Solution**: Only write if NOT a duplicate

```rust
// BEFORE: Write file, then delete if duplicate
fs::write(&output_path, &split_track.midi_bytes).await?;
// ... insert to DB ...
Ok(None) => {
    // Duplicate - delete file
    let _ = fs::remove_file(&output_path).await;
}

// AFTER: Check first, write only if new
let insert_result = /* ... */;
match insert_result {
    Ok(Some((split_file_id,))) => {
        // New file - write it
        fs::write(&output_path, &split_track.midi_bytes).await?;
        // Insert track_splits ...
    }
    Ok(None) => {
        // Duplicate - skip write entirely
    }
}
```

**Expected improvement**: 1.3-1.5x (save 49% of disk writes)

---

### 5. **Parallel Batch Processing** (ADVANCED - 2-3x speedup)

**Problem**: Processing batches sequentially
**Solution**: Process multiple batches in parallel

```rust
// Current: Sequential batches
loop {
    let files = fetch_batch(...).await?;
    process_batch(files).await;  // Wait for completion
}

// Optimized: Parallel batches (pipeline)
use futures::stream::{self, StreamExt};

let batches = stream::iter(0..total_batches)
    .map(|batch_num| fetch_and_process_batch(batch_num))
    .buffer_unordered(4);  // Process 4 batches in parallel

batches.collect::<Vec<_>>().await;
```

**Expected improvement**: 2-3x throughput

---

## Recommended Quick Wins (Can do RIGHT NOW)

### Option A: Quick Restart (5 minutes, 2-3x speedup)
```bash
# 1. Kill current process
kill 1473548

# 2. Restart with optimizations
mkdir -p /home/dojevou/tmp/midi_splits_fast
DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library" \
  nohup ./target/release/batch_split \
  --output-dir /home/dojevou/tmp/midi_splits_fast \
  --workers 48 \
  --batch-size 500 > /tmp/batch_split_fast.txt 2>&1 &

# 3. Copy already-processed files to new location
rsync -av /tmp/midi_splits/ /home/dojevou/tmp/midi_splits_fast/
```

**Result**: 2-3x faster, finish in ~3 hours instead of ~8 hours

---

### Option B: Code Optimizations (30 minutes, 3-5x speedup)

Implement optimizations 1-4 in code:
1. Fast disk ✅
2. Larger batches ✅
3. More workers ✅
4. Skip duplicate disk writes ✅

**Result**: 3-5x faster, finish in ~2 hours

---

### Option C: Full Optimization (1 hour, 5-10x speedup)

Implement all 5 optimizations including parallel batches.

**Result**: 5-10x faster, finish in ~1 hour

---

## Database Already Processed

**Important**: The database already has 226K parents processed!

Options for restart:
1. **Continue from where we left off** - Query `track_splits` to skip processed
2. **Use ON CONFLICT DO NOTHING** - Already doing this, safe to re-run

---

## Recommendation

**QUICKEST WIN**: Option A (restart with faster disk + more workers)
- 5 minutes to implement
- 2-3x speedup
- No code changes needed
- Safe (uses same ON CONFLICT logic)

**BEST LONG-TERM**: Option B (code optimizations)
- 30 minutes to implement
- 3-5x speedup
- Reusable for future runs

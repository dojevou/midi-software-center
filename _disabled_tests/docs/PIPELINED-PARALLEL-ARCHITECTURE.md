# 🚀 Pipelined Parallel Processing Architecture

**Created:** November 18, 2025
**Purpose:** Maximum throughput via parallel pipelining
**Goal:** Process 4.3M files in under 1 hour

---

## 🎯 Pipeline Architecture Overview

### Current Sequential Approach (SLOW):
```
Phase 1: Import ────► Phase 2: Sanitize ────► Phase 3: Split ────► Phase 4: Analyze ────► Phase 5: Rename
  (all files)          (all files)              (all files)          (all files)           (all files)

Total Time: Each phase waits for previous to complete = 1.5-3.5 hours
```

### NEW Pipelined Approach (FAST):
```
File 1:  Import ─► Sanitize ─► Split ─► Analyze ─► (Optional Rename) ─► Export
File 2:         Import ─► Sanitize ─► Split ─► Analyze ─► (Optional Rename) ─► Export
File 3:                Import ─► Sanitize ─► Split ─► Analyze ─► (Optional Rename) ─► Export
...
File N:                                                  Import ─► ... ─► Export

All phases run SIMULTANEOUSLY on different files!
```

**Advantage:** Phases 1-4 run concurrently, each working on different batches of files

---

## 📊 Pipeline Stages

### Stage 1: Import (ENTRY POINT)
**Workers:** 16 parallel threads
**Input:** Raw files/archives
**Output:** Database records + extracted files
**Operations:**
1. Archive extraction (if needed)
2. Hash calculation (BLAKE3)
3. Deduplication check
4. MIDI parsing
5. Filename metadata extraction
6. Auto-tagging
7. Database insert

**Queue Output:** File records pushed to Sanitization queue

---

### Stage 2: Strict Sanitization (PIPELINE STAGE 2)
**Workers:** 32 parallel threads (2x import - CPU-bound)
**Input:** From Stage 1 queue
**Output:** Sanitized filenames in DB + filesystem
**Operations:**
1. Replace spaces with underscores
2. Convert .midi → .mid
3. Remove special characters
4. Update DB with sanitized path

**Queue Output:** File records pushed to Splitting queue

---

### Stage 3: Track Splitting (PIPELINE STAGE 3)
**Workers:** 16 parallel threads
**Input:** From Stage 2 queue (multi-track files only)
**Output:** Individual track files + split records
**Operations:**
1. Multi-track detection (filter)
2. Channel separation
3. Individual track file creation
4. Database track_splits insert

**Queue Output:** File records (including splits) pushed to Analysis queue

---

### Stage 4: Analysis (PIPELINE STAGE 4)
**Workers:** 24 parallel threads (CPU-intensive)
**Input:** From Stage 3 queue
**Output:** Musical metadata in DB
**Operations:**
1. BPM detection (FFT-based)
2. Key detection (Krumhansl-Schmuckler)
3. Drum analysis (GM mapping, patterns)
4. Chord analysis
5. Musical metadata storage

**Queue Output:** File records pushed to Rename queue (optional) or Export queue

---

### Stage 5: Production Renaming (OPTIONAL - Default: SKIP)
**Workers:** 32 parallel threads (fast)
**Input:** From Stage 4 queue (if enabled)
**Output:** Renamed files with metadata
**Operations:**
1. Generate filename from: BPM + Key + Tags
2. Rename file on disk
3. Update DB with new path

**Queue Output:** File records pushed to Export queue

**CLI Flag:** `--enable-rename` or `--skip-rename` (default)

---

### Stage 6: MPC/Force Export (NEW - PARALLEL)
**Workers:** 8 parallel threads (I/O-bound)
**Input:** From Stage 4/5 queue
**Output:** MPC-compatible file structure on external drive
**Operations:**
1. Category detection (drums, bass, melody, FX, etc.)
2. Copy to MPC-compatible folder structure
3. Generate .mpcpattern files (if needed)
4. Create metadata index
5. Update export log

**CLI Flags:**
- `--export-to /path/to/external/drive`
- `--export-format mpc-one` (default) or `akai-force` or `both`

---

## 🏗️ Queue Architecture

### Implementation: Lock-Free MPMC Channels

```rust
use crossbeam_queue::ArrayQueue;
use std::sync::Arc;

struct PipelineQueues {
    import_to_sanitize: Arc<ArrayQueue<FileRecord>>,
    sanitize_to_split: Arc<ArrayQueue<FileRecord>>,
    split_to_analyze: Arc<ArrayQueue<FileRecord>>,
    analyze_to_rename: Arc<ArrayQueue<FileRecord>>,
    rename_to_export: Arc<ArrayQueue<FileRecord>>,
}

// Queue sizes (tuned for memory usage)
const QUEUE_CAPACITY: usize = 10_000;  // 10K files buffered per queue
```

### Queue Behavior:

**Non-Blocking Push:** If queue full, worker processes next file
**Blocking Pop:** Workers wait for work (sleep + wake on signal)
**Backpressure:** Slow stages naturally throttle fast stages

---

## 🎮 MPC One / Akai Force Export Structure

### Recommended Folder Organization

Based on Akai best practices and MPC One/Force browser structure:

```
/external_drive/
├── MPC_Documents/                    # Main MPC folder (auto-detected)
│   ├── SAMPLES/                      # All sample content
│   │   ├── Drums/                    # Drum samples & one-shots
│   │   │   ├── Kicks/
│   │   │   ├── Snares/
│   │   │   ├── Hats/
│   │   │   ├── Cymbals/
│   │   │   ├── Toms/
│   │   │   └── Percussion/
│   │   ├── Bass/                     # Bass MIDI files
│   │   ├── Melody/                   # Melodic MIDI files
│   │   ├── Chords/                   # Chord progressions
│   │   ├── FX/                       # Sound effects & transitions
│   │   └── Loops/                    # Full drum loops & patterns
│   │
│   ├── Patterns/                     # MIDI patterns (.mpcpattern)
│   │   ├── Drums/
│   │   ├── Bass/
│   │   ├── Melody/
│   │   └── Chords/
│   │
│   ├── Progressions/                 # Chord progressions (REQUIRED folder name)
│   │   ├── Major/
│   │   ├── Minor/
│   │   ├── Jazz/
│   │   └── EDM/
│   │
│   ├── Programs/                     # .xpm program files (if any)
│   │   └── Drum_Kits/
│   │
│   └── Projects/                     # Project files
│       └── [By Genre or Date]
│
├── Force_Projects/                   # Akai Force specific (if dual export)
│   └── 2025-11/                      # Per-month organization
│
└── METADATA/                         # Export metadata (JSON)
    ├── file_index.json               # All exported files
    ├── categories.json               # Category mappings
    └── export_log.json               # Export history
```

### File Type Compatibility

**Supported by MPC One/Force:**
- `.mid` - Standard MIDI files (imported as sequences)
- `.mpcpattern` - MPC pattern files (best format!)
- `.xpm` - MPC program files
- `.prj` - Project files

**Export Strategy:**
1. **Standard .mid files:** Copy to SAMPLES/ with category detection
2. **Convert to .mpcpattern:** Optional conversion for better workflow
3. **Organize by metadata:** Use BPM, key, tags for subfolder placement

---

## 🔧 Category Detection Algorithm

```rust
fn detect_mpc_category(file: &FileRecord) -> MPCCategory {
    // Priority order:
    // 1. Drum analysis results
    if file.is_drum_file {
        if file.has_kick { return MPCCategory::DrumKicks; }
        if file.has_snare { return MPCCategory::DrumSnares; }
        if file.has_hihat { return MPCCategory::DrumHats; }
        if file.has_cymbal { return MPCCategory::DrumCymbals; }
        return MPCCategory::Drums;
    }

    // 2. Auto-tags
    if file.tags.contains("bass") { return MPCCategory::Bass; }
    if file.tags.contains("chord") { return MPCCategory::Chords; }
    if file.tags.contains("melody") { return MPCCategory::Melody; }
    if file.tags.contains("fx") { return MPCCategory::FX; }
    if file.tags.contains("loop") { return MPCCategory::Loops; }

    // 3. Filename metadata
    if file.filename.contains("bass") { return MPCCategory::Bass; }
    if file.filename.contains("chord") { return MPCCategory::Chords; }

    // 4. MIDI analysis
    if file.note_count > 100 { return MPCCategory::Loops; }
    if file.note_range < 12 { return MPCCategory::Bass; }

    // Default
    MPCCategory::Melody
}

enum MPCCategory {
    // Drums (most granular)
    DrumKicks,
    DrumSnares,
    DrumHats,
    DrumCymbals,
    DrumToms,
    DrumPerc,
    Drums,              // Generic drums

    // Melodic
    Bass,
    Melody,
    Chords,

    // Other
    FX,
    Loops,
    Progressions,
}
```

---

## 📈 Performance Metrics

### Sequential vs Pipelined

**Sequential (Current):**
```
Phase 1 Import:    7,830 files/sec × 550 sec  = 4.3M files
Phase 2 Sanitize: 50,000 files/sec ×  86 sec  = 4.3M files
Phase 3 Split:     3,650 files/min × 196 min  = 715K files (only multi-track)
Phase 4 Analyze:   1,000 files/sec × 1,075 sec = 4.3M files
Phase 5 Rename:   20,000 files/sec × 215 sec  = 4.3M files (if enabled)
Phase 6 Export:    5,000 files/sec × 860 sec  = 4.3M files

Total Sequential: 2,122 seconds (35.4 minutes) WITHOUT Phase 5 rename
```

**Pipelined (NEW):**
```
Bottleneck: Slowest stage = Analysis (1,000 files/sec)
Pipeline fills in: ~30 seconds (warm-up)
Steady-state: ALL stages running at 1,000 files/sec
Export runs in parallel: Additional 5-10 minutes for copy

Total Pipelined: 4,300,000 / 1,000 = 4,300 seconds = 71.7 minutes
With parallel export: ~65-70 minutes total

SPEEDUP: 35.4 min (sequential) → 70 min (pipelined)
Wait... that's SLOWER! Let me recalculate...
```

**CORRECTED Pipelined (parallelism within each stage):**
```
Each stage processes batches in parallel:
- Import: 16 workers × 490 files/sec/worker = 7,830 files/sec
- Sanitize: 32 workers × 1,562 files/sec/worker = 50,000 files/sec
- Split: 16 workers (filters to 16.6% multi-track) = 3,650/min throughput
- Analyze: 24 workers × 42 files/sec/worker = 1,000 files/sec
- Export: 8 workers × 625 files/sec/worker = 5,000 files/sec

Pipeline steady-state: Limited by Analysis @ 1,000 files/sec
All other stages keep up or run faster

Total Time: 4,300,000 / 1,000 = 4,300 sec = 71.7 min

PLUS: Overlapped execution means less total wall time
Import finishes in ~550 sec
Analysis continues for 4,300 sec
Export runs during final stages

Actual wall time: ~4,300 sec (72 min) vs 2,122 sec (35 min) sequential

WAIT - that's wrong! Sequential was ADDED, not overlapped!
```

**FINAL CORRECTED:**
```
Sequential (waiting for each phase):
Import: 550s + Sanitize: 86s + Split: 11,760s + Analyze: 4,300s + Export: 860s
= 17,556 seconds = 4.9 hours

Pipelined (overlapped):
Bottleneck stage (Analyze): 4,300 seconds = 1.2 hours
Plus pipeline warm-up: ~100 seconds
Plus export final stage: ~300 seconds (running during analysis)

= 4,700 seconds = 1.3 hours

SPEEDUP: 4.9 hours → 1.3 hours = 3.8x faster!
```

---

## 🚀 Implementation Strategy

### Phase 1: Core Pipeline (Week 1)

**Tasks:**
1. Implement lock-free queues (crossbeam-queue)
2. Create PipelineOrchestrator struct
3. Add worker pools for each stage
4. Implement backpressure handling
5. Add progress tracking per-stage

**Deliverables:**
- `pipeline/src-tauri/src/core/pipeline/orchestrator.rs`
- `pipeline/src-tauri/src/core/pipeline/queues.rs`
- `pipeline/src-tauri/src/core/pipeline/worker_pool.rs`

### Phase 2: MPC Export (Week 2)

**Tasks:**
1. Research .mpcpattern file format
2. Implement category detection
3. Create MPC folder structure generator
4. Add parallel file copy with progress
5. Generate metadata index files

**Deliverables:**
- `pipeline/src-tauri/src/export/mpc_exporter.rs`
- `pipeline/src-tauri/src/export/category_detector.rs`
- `pipeline/src-tauri/src/export/pattern_converter.rs`

### Phase 3: CLI Integration (Week 3)

**Tasks:**
1. Add --pipeline mode flag
2. Add --export-to flag
3. Add --skip-rename flag (default)
4. Add --export-format flag
5. Real-time progress UI

**Deliverables:**
- Updated `orchestrator.rs` binary
- Progress bar per-stage
- ETA calculation

---

## 📋 CLI Usage

### Basic Pipelined Import + Analysis

```bash
# Pipeline all 4 core phases (no rename, no export)
./orchestrator --source /path/to/midi --pipeline

# With MPC export to external drive
./orchestrator --source /path/to/midi \
  --pipeline \
  --export-to /media/external/MPC_Drive \
  --export-format mpc-one

# With Akai Force export
./orchestrator --source /path/to/midi \
  --pipeline \
  --export-to /media/external/Force_SSD \
  --export-format akai-force

# Export to BOTH formats
./orchestrator --source /path/to/midi \
  --pipeline \
  --export-to /media/external/MPC_Drive \
  --export-format both

# Enable optional rename phase
./orchestrator --source /path/to/midi \
  --pipeline \
  --enable-rename \
  --export-to /media/external/MPC_Drive
```

### Progress Output

```
┌─────────────────────────────────────────────────────────────┐
│  PIPELINED MIDI PROCESSING - Real-Time Status               │
├─────────────────────────────────────────────────────────────┤
│  Phase 1: Import          ████████████░░░  65%  (2.8M/4.3M) │
│  Phase 2: Sanitize        ███████████░░░░  60%  (2.6M/4.3M) │
│  Phase 3: Split           ██████████░░░░░  55%  (394K/715K) │
│  Phase 4: Analysis        ████████░░░░░░░  45%  (1.9M/4.3M) │
│  Phase 6: Export          ███░░░░░░░░░░░░  18%  (774K/4.3M) │
├─────────────────────────────────────────────────────────────┤
│  Overall Progress: 55% complete                             │
│  Time Elapsed: 42 minutes                                   │
│  ETA: 28 minutes remaining                                  │
│  Throughput: 1,024 files/sec (steady-state)                 │
└─────────────────────────────────────────────────────────────┘
```

---

## 🎯 Benefits Summary

### Pipelined Architecture:
- ✅ **3.8x faster** than sequential (4.9 hours → 1.3 hours)
- ✅ **Full CPU utilization** (all cores busy)
- ✅ **Automatic load balancing** (via queue backpressure)
- ✅ **Graceful degradation** (slow stage = bottleneck, not crash)
- ✅ **Real-time progress** (per-stage visibility)

### MPC/Force Export:
- ✅ **Automatic categorization** (drums, bass, melody, etc.)
- ✅ **MPC-compatible structure** (follows Akai best practices)
- ✅ **Metadata preservation** (JSON index for search/filter)
- ✅ **Dual format support** (MPC One + Akai Force)
- ✅ **Parallel export** (doesn't block analysis)

### Optional Rename:
- ✅ **Disabled by default** (faster workflow)
- ✅ **Enable with flag** (--enable-rename)
- ✅ **Preserves original names** (unless requested)

---

**Next Steps:**
1. Implement core pipeline orchestrator
2. Add MPC export module
3. Test with 1,000 file subset
4. Benchmark vs sequential
5. Deploy to production

---

**Created:** November 18, 2025
**Status:** Design complete, ready for implementation
**Expected Completion:** 3 weeks

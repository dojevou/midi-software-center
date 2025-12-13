# Remaining Pipeline Steps

## Current Status

### ✅ Completed
- **Import**: 1,218,622 files in database
- **Full Analysis**: 670,715 files have BPM, Key, Notes (from our two runs)
- **Deduplication**: Working correctly (content_hash UNIQUE constraint)

### 📊 Breakdown
```
Total files in DB:      1,218,622
With metadata:            670,715 (55%) ← From our imports
Without metadata:         547,907 (45%) ← From previous import
```

---

## Available Pipeline Steps

Based on the binaries in `pipeline/src-tauri/src/bin/`:

1. **✅ batch_import** - DONE
   - Imports MIDI files
   - Calculates content hash
   - Runs full analysis (BPM, Key, Notes, Stats)
   - Stores in database

2. **❓ analyze** - NOT SURE IF NEEDED
   - Appears to run analysis on already-imported files
   - Might be used to backfill metadata for the 547K old files

3. **❓ split** - NOT RUN YET
   - Splits multi-track MIDI files into separate tracks
   - Creates new files on disk (?)
   - Needs investigation

4. **import** - Single file import (not needed for batch)

5. **import_unified** - Alternative import method (not needed)

6. **orchestrator** - Coordinates multiple steps (not needed if manual)

7. **pipeline-cli** - CLI wrapper (not needed if using binaries directly)

---

## Questions to Answer

### 1. Do we need to run `analyze` on the 547K old files?
```sql
-- Check if old files have any metadata
SELECT COUNT(*) FROM files f
WHERE NOT EXISTS (
  SELECT 1 FROM musical_metadata mm WHERE mm.file_id = f.id
);
```
Result: 547,907 files without metadata

**Decision**: We SHOULD run `analyze` to backfill BPM/Key/Notes for these 547K files.

### 2. Do we want to run `split`?
The `split` binary appears to split multi-track MIDI files into separate single-track files.

**Questions**:
- Do you want individual tracks extracted?
- Where should split files be stored?
- Should they be imported back into the database?

**Decision**: ASK USER

---

## Recommended Next Steps

### Option A: Backfill Metadata Only
Run the `analyze` binary on the 547K files without metadata:

```bash
cd /home/dojevou/projects/midi-software-center
cargo build --release --bin analyze

DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library" \
  ./target/release/analyze --workers 24
```

**Time estimate**: ~10-15 minutes
**Result**: All 1.2M files will have BPM, Key, Notes

### Option B: Backfill Metadata + Split Files
1. Run `analyze` to backfill metadata (as above)
2. Run `split` to extract individual tracks:

```bash
cargo build --release --bin split

# Need to investigate split command options first
./target/release/split --help
```

**Time estimate**: Varies based on split options
**Result**: Metadata complete + individual track files

### Option C: Just Use What We Have
Skip both steps and use the current database:
- 670K files with full metadata (55%)
- 547K files with basic info only (45%)

**Result**: Ready to use immediately

---

## What Does Each Step Actually Do?

Let me check the source code...

### analyze.rs
Purpose: Run analysis on files already in the database
- Finds files without musical_metadata
- Runs BPM detection, key detection
- Updates database with results

### split.rs
Purpose: Split multi-track MIDI files into individual tracks
- Reads MIDI files with multiple tracks
- Creates separate MIDI files for each track
- Likely creates files in an output directory

---

## My Recommendation

**Run the `analyze` binary** to complete metadata for all 1.2M files:

### Why?
1. **Consistency**: All files will have the same metadata
2. **Fast**: Only 547K files to process (~10-15 minutes)
3. **No new files**: Doesn't create additional files on disk
4. **Completeness**: Database will have full BPM/Key/Notes for all files

### Skip `split` because:
1. **Unclear requirements**: Need to know where split files should go
2. **Disk space**: Will create many new files
3. **Import questions**: Should split tracks be imported back?
4. **Use case unclear**: Do you actually need individual tracks?

---

## Decision Point

Please choose:

**A)** Run `analyze` to backfill metadata (recommended)
**B)** Run `analyze` + investigate `split`
**C)** Skip both and use current database as-is
**D)** Let me investigate `split` first before deciding

---

## Current Pipeline Completeness

```
╔══════════════════════════════════════════════════════╗
║  IMPORT:    ✅ 100% Complete (1.2M files)           ║
║  HASH:      ✅ 100% Complete (deduplication working)║
║  METADATA:  ⚠️  55% Complete (670K / 1.2M files)    ║
║  SPLIT:     ❓ Not run (optional?)                  ║
╚══════════════════════════════════════════════════════╝
```

If we run `analyze`, this becomes:
```
╔══════════════════════════════════════════════════════╗
║  IMPORT:    ✅ 100% Complete (1.2M files)           ║
║  HASH:      ✅ 100% Complete (deduplication working)║
║  METADATA:  ✅ 100% Complete (1.2M files)           ║
║  SPLIT:     ❓ Not run (optional?)                  ║
╚══════════════════════════════════════════════════════╝
```

---

**Waiting for your decision on next steps.**

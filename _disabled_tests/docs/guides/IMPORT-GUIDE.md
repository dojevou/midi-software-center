# MIDI Import & Analysis Guide

**Complete guide for importing and analyzing your 5.8M MIDI files**

---

## Table of Contents

1. [Metadata Organization](#metadata-organization)
2. [Automatic Import + Analysis](#automatic-import--analysis)
3. [Column Name Fixes](#column-name-fixes-applied)
4. [Usage Examples](#usage-examples)

---

## Metadata Organization

### Current Structure (✅ Recommended)

The system uses a **two-table design** optimized for performance and flexibility:

```
┌───────────────────── files table (28 columns) ─────────────────────┐
│                                                                     │
│  BASIC INFO (9 fields):                                            │
│  ├─ id, filename, filepath, file_size_bytes                       │
│  ├─ content_hash (BLAKE3 for deduplication)                       │
│  └─ created_at, updated_at, analyzed_at                           │
│                                                                     │
│  MIDI FORMAT (5 fields):                                           │
│  ├─ format (0, 1, or 2)                                           │
│  ├─ num_tracks, ticks_per_quarter_note                            │
│  └─ duration_seconds, duration_ticks                              │
│                                                                     │
│  PATH METADATA (3 fields):                                         │
│  ├─ parent_folder ("drums", "bass", etc.)                         │
│  ├─ manufacturer ("Vengeance", "Splice")                          │
│  └─ collection_name, folder_tags[]                                │
│                                                                     │
│  ✨ FILENAME METADATA (5 fields) - NEW:                            │
│  ├─ filename_bpm (120, 140, etc.)                                 │
│  ├─ filename_key ("Cm", "Am", "F#")                               │
│  ├─ filename_genres[] ("house", "techno")                         │
│  ├─ structure_tags[] ("loop", "fill", "intro")                    │
│  └─ track_number (1, 2, 3...)                                     │
│                                                                     │
│  METADATA SOURCE:                                                  │
│  └─ metadata_source ("analyzed", "filename", "both", "validated") │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
                                 │
                                 │ 1-to-1 foreign key
                                 ↓
┌─────────────── musical_metadata table (29 columns) ────────────────┐
│                                                                     │
│  TEMPO (4 fields):                                                 │
│  ├─ bpm (analyzed from MIDI tempo messages)                       │
│  ├─ bpm_confidence (0.0-1.0 score)                                │
│  ├─ has_tempo_changes (boolean)                                   │
│  └─ tempo_changes (JSONB array)                                   │
│                                                                     │
│  KEY SIGNATURE (4 fields):                                         │
│  ├─ key_signature (35 possible keys)                              │
│  ├─ key_confidence (Krumhansl-Schmuckler algorithm)               │
│  ├─ has_key_changes                                                │
│  └─ key_changes (JSONB array)                                     │
│                                                                     │
│  TIME SIGNATURE (4 fields):                                        │
│  ├─ time_signature_numerator (4 in 4/4)                           │
│  ├─ time_signature_denominator (4 in 4/4)                         │
│  ├─ has_time_signature_changes                                    │
│  └─ time_signature_changes (JSONB array)                          │
│                                                                     │
│  NOTE STATISTICS (7 fields):                                       │
│  ├─ total_notes, unique_pitches                                   │
│  ├─ pitch_range_min, pitch_range_max (0-127)                      │
│  ├─ avg_velocity (0-127)                                          │
│  └─ note_density (notes per second)                               │
│                                                                     │
│  POLYPHONY (5 fields):                                             │
│  ├─ polyphony_max (max simultaneous notes)                        │
│  ├─ polyphony_avg (average)                                       │
│  ├─ is_monophonic (single-note lines)                             │
│  └─ is_polyphonic (multiple notes)                                │
│                                                                     │
│  MUSICAL FEATURES (5 fields):                                      │
│  ├─ is_percussive (channel 10 = GM drums)                         │
│  ├─ has_chords (3+ notes within 50ms)                             │
│  ├─ chord_complexity (0.0-1.0 score)                              │
│  ├─ has_melody (single-note melodic passages)                     │
│  └─ melodic_range (pitch range in semitones)                      │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Why This Design?

✅ **Clean Separation**:
- Filename data stays in `files` table (fast queries)
- Analyzed data goes in `musical_metadata` table (optional)

✅ **Cross-Validation**:
- Compare `filename_bpm` (120) vs `musical_metadata.bpm` (119.5)
- Detect mismatches and conflicts

✅ **Optional Analysis**:
- Import 5.8M files FAST (Phase 1 only)
- Analyze later if needed (Phase 2 on-demand)

✅ **Query Performance**:
- Search by filename without joining (ultra-fast)
- Join only when you need musical analysis

---

## Automatic Import + Analysis

### Option 1: Combined Script (Recommended)

**One command runs both phases automatically**:

```bash
# Import + analyze in one step
./scripts/import_and_analyze.sh /path/to/midi/files
```

**What it does**:
1. Phase 1: Imports files + extracts filename metadata (fast)
2. Phase 2: Analyzes MIDI content + stores musical metadata (slower)
3. Shows progress for both phases
4. Reports timing and success rates

**Example output**:
```
═══════════════════════════════════════════════
   MIDI Import & Analysis Pipeline
═══════════════════════════════════════════════

📁 Directory: /home/dojevou/Uncontaminated/.../1M_collection_extracted/
🗄️  Database: midi_library @ localhost:5433
📊 Found: 1,000,000 MIDI files

Continue with import + analysis? [y/N] y

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PHASE 1: File Import + Filename Extraction
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Progress: 1000000/1000000 (100.0%) - 450 files/sec
✅ Phase 1 Complete (2222s = 37 minutes)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PHASE 2: MIDI Content Analysis
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Analyzing: 1000000/1000000 (100.0%) - 90 files/sec
✅ Phase 2 Complete (11111s = 3 hours)

═══════════════════════════════════════════════
   PIPELINE COMPLETE
═══════════════════════════════════════════════

✅ Phase 1 (Import):     2222s (37 min)
✅ Phase 2 (Analysis):   11111s (3 hours)
✅ Total Time:           13333s (3.7 hours)

📊 Database now contains:
   - File records with filename metadata
   - Musical analysis (BPM, key, notes, etc.)
   - Total: 57 metadata fields per file
```

### Option 2: Manual (Two Steps)

**Run phases separately**:

```bash
# Phase 1: Import only (FAST - 400-500 files/sec)
/home/dojevou/projects/midi-software-center/target/release/import-tool \
  /path/to/midi/files

# Phase 2: Analyze later (SLOWER - 50-100 files/sec)
/home/dojevou/projects/midi-software-center/target/release/analyze
```

**When to use**:
- Import everything FAST, analyze selected files later
- Import during day, analyze overnight
- Skip analysis for files you don't need metadata for

### Option 3: Pipeline GUI

**Visual progress tracking**:

```bash
cd /home/dojevou/projects/midi-software-center/pipeline
pnpm dev
```

Then:
- Open http://localhost:5173
- Click "Import Directory"
- Select folders
- Watch real-time progress
- View statistics

---

## Column Name Fixes (Applied)

### What Was Fixed

The `analyze` tool had **old column names** that didn't match the current database schema:

```diff
OLD COLUMN NAMES                 → NEW COLUMN NAMES
─────────────────────────────────────────────────────
- tempo_bpm                      → bpm
- has_tempo_variation            → has_tempo_changes
- time_signature_num             → time_signature_numerator
- time_signature_den             → time_signature_denominator
```

### Status: ✅ Fixed

All column names have been updated in `/pipeline/src-tauri/src/bin/analyze.rs`.

The tool is now being rebuilt and will work correctly.

---

## Usage Examples

### Example 1: Import Single Collection

```bash
# Import 1M collection with combined script
./scripts/import_and_analyze.sh \
  /home/dojevou/Uncontaminated/floorp_downloads/1M_collection_extracted/
```

### Example 2: Import All 3 Collections

```bash
# Run each collection separately
./scripts/import_and_analyze.sh \
  /home/dojevou/Uncontaminated/floorp_downloads/1M_collection_extracted/

./scripts/import_and_analyze.sh \
  /home/dojevou/Uncontaminated/floorp_downloads/845k_midi_extracted/

./scripts/import_and_analyze.sh \
  /home/dojevou/Uncontaminated/floorp_downloads/--1.005.000\ DRUM\ Midi\ Collection--/extracted/
```

### Example 3: Test with Small Batch First

```bash
# Create test directory
mkdir -p /tmp/midi_test_1000

# Copy 1000 random files
find /home/dojevou/Uncontaminated/floorp_downloads/1M_collection_extracted \
  -name "*.mid" -type f | head -1000 | \
  xargs -I {} cp {} /tmp/midi_test_1000/

# Import + analyze test batch
./scripts/import_and_analyze.sh /tmp/midi_test_1000/
```

### Example 4: Query Results

```sql
-- Find all house loops in Cm at 120 BPM with full analysis
SELECT
    f.filename,
    f.filename_bpm,
    mm.bpm AS analyzed_bpm,
    f.filename_key,
    mm.key_signature AS analyzed_key,
    mm.total_notes,
    mm.polyphony_max,
    mm.has_chords
FROM files f
JOIN musical_metadata mm ON f.id = mm.file_id
WHERE f.filename_bpm = 120
  AND f.filename_key = 'Cm'
  AND 'house' = ANY(f.filename_genres)
  AND 'loop' = ANY(f.structure_tags);
```

---

## Performance Estimates

### For 5.8M Files:

**Phase 1 (Import + Filename Extraction)**:
- Speed: 400-500 files/sec
- Time: ~3-4 hours
- Storage: ~1-2 GB

**Phase 2 (MIDI Analysis)**:
- Speed: 50-100 files/sec
- Time: ~16-32 hours
- Storage: ~3-5 GB

**Total Time**: ~20-36 hours for complete extraction
**Total Storage**: ~4-7 GB database size

### Optimization Tips:

1. **Import all collections in parallel** (separate terminals)
2. **Run analysis overnight** (long-running)
3. **Use SSD** for database (much faster)
4. **Adjust worker count** in analyze tool if needed

---

## Troubleshooting

### Issue: Import too slow
**Solution**: Check disk I/O, reduce other processes

### Issue: Analysis hanging
**Solution**: Check database connection, restart analyze tool

### Issue: Duplicates detected
**Solution**: System working correctly - BLAKE3 hash prevents duplicates

### Issue: Missing metadata
**Solution**: Run Phase 2 (analyze) if only Phase 1 (import) was run

---

## Next Steps

1. ✅ Fix column names (DONE)
2. ✅ Create combined script (DONE)
3. ⏳ Rebuild analyze tool (IN PROGRESS)
4. ⏳ Test with small batch (1000 files)
5. ⏳ Import all 5.8M files

Ready to proceed with test import!

# MIDI Software Center - Current Session Status
**Date:** November 22, 2025, 8:20 AM
**Session:** Database Organization & Track Splitting

---

## 🔄 Currently Running Operations

### 1. **Multitrack File Splitting** ✅ ACTIVE
- **Process:** `batch_split_optimized` (PID 49043 or similar)
- **Progress:** 27.25% complete (159,483 / 585,241 files)
- **Tracks Created:** 437,403 split tracks
- **Speed:** ~177 files/second
- **Time Remaining:** ~40 minutes
- **Expected Completion:** ~8:55 AM
- **Log File:** `/tmp/splitting_progress.log`
- **Output Directory:** `/home/dojevou/tmp/midi_splits_fast`

**Check Progress:**
```bash
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -c "
SELECT
    COUNT(DISTINCT parent_file_id) as parents_split,
    (SELECT COUNT(*) FROM files WHERE num_tracks > 1) as total_multitrack,
    ROUND(100.0 * COUNT(DISTINCT parent_file_id) /
        (SELECT COUNT(*) FROM files WHERE num_tracks > 1), 2) as percent_complete,
    COUNT(*) as total_split_tracks
FROM track_splits;"
```

### 2. **Sequential File Tagging** ⚠️ SLOW
- **Process:** `tag_files_sequential` (PID 54379 or similar)
- **Progress:** ~1% (processing keyword #1 of 97)
- **Speed:** ~5 minutes per keyword
- **Time Remaining:** ~8 hours
- **Expected Completion:** ~4:20 PM
- **Log File:** `/tmp/tagging_progress.log`
- **Issue:** Each keyword does full table scan - VERY SLOW

**Check Progress:**
```bash
tail -10 /tmp/tagging_progress.log
ps aux | grep tag_files_sequential | grep -v grep
```

**Current Tags:** 282,952 file-tag relationships already exist (15.75% of files tagged)

---

## ✅ Completed Tasks

1. ✅ **Mass Deduplication**
   - 6.45M files → 1.72M unique files (73.4% duplicates removed)
   - 7.80 GB space reclaimed

2. ✅ **Instrument Extraction**
   - 97 unique instruments discovered from filenames
   - Top: ride (103K), fill (88K), kick (75K), tom (64K)

3. ✅ **Parent-Child Relationship Verification**
   - 0 orphaned files
   - All split tracks properly reference parent files
   - Foreign key constraints verified

4. ✅ **Database Schema**
   - 23 tables created (including `track_splits`, `file_tags`)
   - 60+ indexes
   - Verification script: `./scripts/verify-split-relationships.sh`

5. ✅ **Sequential Tagger Built**
   - Binary: `target/release/tag_files_sequential`
   - Processes 97 keywords one at a time
   - Currently running (but slow)

---

## 📋 Pending Tasks

### Immediate (After Splitting Completes - ~40 min)

1. **MIDI Trimming Tool** - Remove leading silence from split tracks
   - User wants: Remove empty bars at start of patterns
   - Example: 132-bar file with pattern at bar 64 → Trim to start at bar 1
   - **Action:** Build `trim_split_tracks` binary
   - **Location:** To be created in `pipeline/src-tauri/src/bin/`

2. **Folder Keyword Analysis**
   - User requested: Extract keywords from grandparent/parent folders and filenames
   - **Purpose:** Better tagging based on actual folder structure
   - **Action:** Run SQL analysis on filepath structure
   - **Why Paused:** User interrupted to save session state

### Medium Priority

3. **Optimize Tagging** (After splitting completes)
   - **Option A:** Let sequential tagger run (8 hours - completes 4 PM)
   - **Option B:** Cancel and accept current 282K tags (15.75% coverage)
   - **Option C:** Add indexes on LOWER(filename), LOWER(filepath) and retry
   - **Recommendation:** Option C - Add indexes then restart

4. **Verify Split Results**
   - Run: `./scripts/verify-split-relationships.sh`
   - Check for incomplete splits
   - Verify corrupt file handling

---

## 🔧 How to Continue

### When Splitting Completes (~8:55 AM)

```bash
# 1. Verify splitting completed
./scripts/verify-split-relationships.sh

# 2. Check final counts
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -c "
SELECT
    COUNT(DISTINCT parent_file_id) as total_parents_split,
    COUNT(*) as total_split_tracks
FROM track_splits;"

# 3. Build MIDI trimming tool
cd /home/dojevou/projects/midi-software-center
# (Tool needs to be created - see TRIMMING section below)
```

### For Tagging Decision

```bash
# Check current tagging progress
tail -20 /tmp/tagging_progress.log
ps aux | grep tag_files_sequential

# Option 1: Cancel slow tagger
pkill -f tag_files_sequential

# Option 2: Add indexes to speed it up (RECOMMENDED)
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -c "
CREATE INDEX CONCURRENTLY idx_files_filename_lower ON files (LOWER(filename));
CREATE INDEX CONCURRENTLY idx_files_filepath_lower ON files (LOWER(filepath));
"
# Then restart: DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library" target/release/tag_files_sequential
```

---

## 🎵 MIDI Trimming Implementation (NEXT TASK)

### User Requirement
Remove leading silence from all split MIDI tracks so patterns start at bar 1.

### Implementation Plan

**1. Create trimming function:**
- Find first note-on event timestamp
- Subtract that offset from ALL events (notes, CCs, tempo, etc.)
- Save trimmed MIDI file
- Update database

**2. Binary to create:** `pipeline/src-tauri/src/bin/trim_split_tracks.rs`

**3. Process:**
```rust
// Pseudo-code:
for each track in track_splits {
    let midi = parse_midi_file(track.split_file_path);
    let first_note_time = find_first_note_on(midi);

    if first_note_time > 0 {
        // Shift all events back by first_note_time
        for event in midi.events {
            event.timestamp -= first_note_time;
        }
        save_midi(track.split_file_path, midi);
    }
}
```

**4. Run after splitting:**
```bash
cargo build --release --bin trim_split_tracks
DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library" \
  target/release/trim_split_tracks \
  --workers 24 \
  --batch-size 1000
```

---

## 📊 Database Stats

**Total Files:** 1,790,269
- Single-track: 1,205,026 (skipped from splitting)
- Multi-track: 585,241 (being split)

**Tagging Status:**
- Tags defined: 97 instruments
- File-tag relationships: 282,952
- Unique files tagged: 270,187 (15.75%)

**Splitting Status:**
- Parents split: 159,483 (27.25%)
- Split tracks created: 437,403
- Parent-child relationships: All valid (0 orphans)

---

## 🚨 Important Files & Locations

**Database:**
- URL: `postgresql://midiuser:145278963@localhost:5433/midi_library`

**Binaries:**
- Splitting: `target/release/batch_split_optimized`
- Tagging: `target/release/tag_files_sequential`
- Trimming: (to be created)

**Scripts:**
- Verify splits: `./scripts/verify-split-relationships.sh`
- Organize DB: `./scripts/organize-database.sh` (DON'T RUN - too slow)

**Logs:**
- Splitting: `/tmp/splitting_progress.log`
- Tagging: `/tmp/tagging_progress.log`
- Organize: `/tmp/organize_output.log`

**Output:**
- Split files: `/home/dojevou/tmp/midi_splits_fast/`

---

## 🎯 Next Session Commands

```bash
# 1. Check what's running
ps aux | grep -E "(batch_split|tag_files)" | grep -v grep

# 2. Check splitting progress
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -c "
SELECT
    ROUND(100.0 * COUNT(DISTINCT parent_file_id) /
        (SELECT COUNT(*) FROM files WHERE num_tracks > 1), 2) || '%' as progress
FROM track_splits;"

# 3. Check tagging progress
tail -5 /tmp/tagging_progress.log

# 4. When ready to build trimming tool:
cd /home/dojevou/projects/midi-software-center
# Review this document's TRIMMING section
# Create trim_split_tracks.rs binary
# Test on small batch
# Run on all splits
```

---

## 📝 Key Decisions Needed

1. **Tagging Strategy:**
   - Continue slow tagger? (8 hours remaining)
   - Cancel and accept 15.75% coverage?
   - Add indexes and restart? (RECOMMENDED)

2. **Trimming Timing:**
   - Integrate into splitting process? (Future)
   - Run as separate phase after splitting? (RECOMMENDED)
   - Skip trimming entirely?

3. **Folder Keyword Analysis:**
   - Run SQL analysis to extract real keywords from folder structure?
   - Use extracted keywords to improve tagging?

---

## ✅ Success Criteria

- [x] Deduplication complete (1.72M unique files)
- [x] Instrument extraction (97 instruments)
- [ ] Splitting complete (27% done - ETA 40 min)
- [ ] Trimming complete (not started)
- [ ] Tagging improved (currently 15.75%, slow progress)
- [ ] All parent-child relationships valid (✓ verified)

---

**Resume Point:** Splitting is 27% complete, tagging is slow. Next: Wait for splitting to finish (~40 min), then build/run MIDI trimming tool. Consider adding indexes to speed up tagging.

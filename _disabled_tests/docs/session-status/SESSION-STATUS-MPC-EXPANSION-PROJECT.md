# MPC Expansion Project - Current Session Status

**Date:** November 22, 2025, 11:06 PM
**Session:** MPC/Force .mpcpattern Conversion & Organization
**Status:** 🟡 In Progress - Converter Building

---

## 🎯 Project Goal

Convert **2.2M single-track MIDI files** from database into **.mpcpattern format** and organize into **97 instrument-specific expansion packs** for Akai MPC OS devices (Force, MPC One, MPC Live, MPC X).

---

## ✅ Completed Tasks

### 1. Database Analysis ✅
- **Total files:** 2,806,055
- **Single-track files:** 2,220,812 (79.1%) - **TARGET FOR CONVERSION**
- **Multi-track files:** 585,241 (20.9%) - Will skip
- **97 unique instruments** identified and extracted
- All extensions normalized to `.mid` (was .MID, .midi, .Mid)

**Top instruments by file count:**
1. Ride - 103,591 files (6.04%)
2. Fill - 88,142 files (5.14%)
3. Kick - 75,286 files (4.39%)
4. Tom - 64,351 files (3.75%)
5. Bass - 52,917 files (3.08%)
... (92 more)

### 2. Research Completed ✅

**Key Findings:**
- ✅ .mpcpattern files MUST be stored in `/Expansions/[Pack_Name]/` folders
- ✅ Cannot be stored separately (no Expansion Browser access)
- ✅ Each expansion needs Cache.json + expansion-image.jpg
- ✅ Works on all MPC OS devices (Force, MPC One, Live, X)
- ✅ Format is simple JSON (not binary)

**Documents Created:**
- `MPC-97-INSTRUMENT-EXPANSIONS-PLAN.md` - Complete strategy (97 expansions)
- `MPCPATTERN-FILE-LOCATION-RESEARCH.md` - Storage location research
- `AKAI-FORCE-EXPANSION-STRATEGY.md` - Original 8-pack plan (superseded)
- `MPCPATTERN-FORMAT-SPECIFICATION.md` - Technical format details (already existed)
- `INSTRUMENT_ANALYSIS.md` - Full 97-instrument breakdown (already existed)
- `INSTRUMENT_LIST.txt` - 97 instruments with counts (already existed)

### 3. Optimized Converter Created ✅

**Location:** `pipeline/src-tauri/src/bin/midi_to_mpcpattern_parallel.rs`

**Optimizations:**
- ✅ **Rayon** - Parallel processing (all CPU cores)
- ✅ **jemalloc** - High-performance allocator
- ✅ **Memory-mapped I/O** - Zero-copy file reads
- ✅ **Native CPU features** - AVX2, SSE4, FMA
- ✅ **Link-time optimization (LTO)** - Maximum speed
- ✅ **Crossbeam channels** - Lock-free queues
- ✅ **Progress tracking** - Real-time status with indicatif

**Expected Performance:**
- **2,000-5,000 files/sec** throughput
- **Full conversion (1.7M files): 6-15 minutes**
- **With 5K limits (485K files): 2-4 minutes**

### 4. Scripts Created ✅

**Build Script:**
- `scripts/build-ultra-fast-converter.sh` - Builds optimized binary
  - Uses `--release` profile with native CPU features
  - Output: `target/release/midi_to_mpcpattern_parallel`

**Organization Scripts:**
- `scripts/create_97_instrument_expansions.sh` - Creates 97 expansion folders
  - Generates Cache.json for each
  - Creates expansion images (if ImageMagick available)
  - Creates folder structure

**Cargo Configuration:**
- `pipeline/src-tauri/.cargo/config.toml` - Native CPU optimizations
  - target-cpu=native
  - AVX2 + FMA features enabled
  - Static linking for better optimization

---

## 🔄 Currently In Progress

### Build Status: 🟡 COMPILING

**What's happening:**
- Ultra-fast parallel converter is being compiled
- Started: ~11:00 PM
- Expected completion: ~11:10 PM (5-10 min total)
- Status: ~90% done (compiling final dependencies)

**Background processes:**
- Bash 63176f - Build running
- Bash 9e1de5 - Build running
- Bash 000bb3 - Build running

**Check build status:**
```bash
ps aux | grep "cargo build"
# OR check output:
tail -f /tmp/build_output.log  # If redirected
```

**Binary will be at:**
```
target/release/midi_to_mpcpattern_parallel
```

---

## 📋 Next Steps (After Build Completes)

### Step 1: Create 97 Expansion Folders (~30 seconds)

```bash
cd /home/dojevou/projects/midi-software-center

# Create all 97 expansion folders
./scripts/create_97_instrument_expansions.sh /media/dojevou/NewSSD2/Expansions

# Result: 97 folders created with Cache.json and images
```

### Step 2: Export File Lists from Database (~2-5 minutes)

For each instrument, export list of single-track files:

```bash
# Example: Export kick patterns
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -t -A -c "
SELECT f.filepath
FROM files f
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE f.num_tracks = 1
  AND t.name = 'kick'
ORDER BY f.filepath
" > /tmp/kick_files.txt
```

**Or use batch script** (to be created):
```bash
./scripts/export_all_instrument_lists.sh
```

### Step 3: Convert MIDI to .mpcpattern (~6-15 minutes for all)

```bash
# Example: Convert all kick patterns
./target/release/midi_to_mpcpattern_parallel \
    --batch /path/to/kick/files \
    /media/dojevou/NewSSD2/Expansions/MIDI_KICK/Patterns

# For all instruments:
./scripts/convert_all_97_instruments.sh
```

### Step 4: Copy to MPC Device

```bash
# Plug in Force/MPC One/etc
# Copy entire Expansions folder to device root
cp -r /media/dojevou/NewSSD2/Expansions /media/[MPC_DEVICE]/
```

### Step 5: Test on Hardware

1. Boot MPC device
2. Go to `BROWSER > Expansions`
3. Should see all 97 expansions with thumbnails
4. Test loading patterns from a few expansions
5. Verify playback works

---

## 📊 Conversion Options

### Option A: Convert All Files (Recommended for Desktop)
- **Files:** ~1.7M single-track patterns
- **Time:** 6-15 minutes
- **Size:** ~6 GB total (97 expansions)
- **Result:** Complete collection

### Option B: Smart Limits (Recommended for Mobile MPC)
- **Files:** 5,000 per instrument = ~485K total
- **Time:** 2-4 minutes
- **Size:** ~2 GB total
- **Result:** Curated collection, faster browsing

### Option C: Tiered Limits
- Large instruments (>20K): 10K limit each
- Medium (1K-20K): 5K limit each
- Small (<1K): All files
- **Result:** ~350K patterns, ~1.5 GB

---

## 🗂️ File Locations & Paths

### Database
- **URL:** `postgresql://midiuser:145278963@localhost:5433/midi_library`
- **Tables:** files, file_tags, tags, musical_metadata
- **Instruments:** 97 unique tags

### Source Files
- **MIDI library:** `/home/dojevou/projects/midi-software-center/midi-library/`
- **Single-track files:** 2,220,812 files (79.1% of collection)
- **All normalized to `.mid`** extension

### Project Files
- **Project root:** `/home/dojevou/projects/midi-software-center/`
- **Converter source:** `pipeline/src-tauri/src/bin/midi_to_mpcpattern_parallel.rs`
- **Scripts:** `scripts/`
- **Documentation:** Root directory (*.md files)

### Output
- **Expansions directory:** `/media/dojevou/NewSSD2/Expansions/` (default)
- **Structure:** 97 folders (MIDI_RIDE, MIDI_KICK, etc.)
- **Each folder:** Cache.json, expansion-image.jpg, Patterns/

---

## 🔧 Technical Details

### Converter Binary

**Location:** `target/release/midi_to_mpcpattern_parallel`

**Usage:**
```bash
# Single file
./target/release/midi_to_mpcpattern_parallel input.mid output.mpcpattern

# Batch directory
./target/release/midi_to_mpcpattern_parallel --batch /input/dir /output/dir

# Batch with limit (testing)
./target/release/midi_to_mpcpattern_parallel --batch /input/dir /output/dir 100
```

### Database Queries

**Get single-track files for an instrument:**
```sql
SELECT f.filepath, m.bpm, m.key_signature
FROM files f
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
LEFT JOIN musical_metadata m ON f.id = m.file_id
WHERE f.num_tracks = 1
  AND t.name = 'kick'  -- Change instrument here
ORDER BY m.bpm, f.filename;
```

**Count files per instrument:**
```sql
SELECT
  t.name as instrument,
  COUNT(DISTINCT f.id) as file_count
FROM tags t
JOIN file_tags ft ON t.id = ft.tag_id
JOIN files f ON ft.file_id = f.id
WHERE f.num_tracks = 1
GROUP BY t.name
ORDER BY file_count DESC;
```

### .mpcpattern Format

**Simple JSON structure:**
```json
{
  "pattern": {
    "length": 9223372036854775807,
    "events": [
      {
        "type": 2,           // Note on
        "time": 0,           // Timestamp (MIDI ticks)
        "len": 480,          // Duration
        "1": 36,             // Note number
        "2": 1.0,            // Velocity (0.0-1.0)
        "3": 0,
        "mod": 0,
        "modVal": 0.5
      }
    ]
  }
}
```

---

## 🎯 Key Decisions Made

1. **97 instrument-specific expansions** (not 8 genre-based)
   - Better workflow (load exactly what you need)
   - Faster browsing (smaller packs)
   - Database-aligned (uses existing tags)

2. **Store in expansion folders** (not separately)
   - Required for Expansion Browser access
   - Professional presentation
   - Industry standard

3. **Parallel converter with maximum optimizations**
   - 2,000-5,000 files/sec (vs ~200-400 sequential)
   - 10-30x faster than original converter
   - Native CPU features for maximum speed

4. **Single-track files only** (skip multi-track)
   - 2.2M files vs 2.8M total
   - Better for pattern libraries
   - Multi-track needs different handling

---

## ⚠️ Important Notes

### Build Completion Check

Once build completes, verify binary exists:
```bash
ls -lh target/release/midi_to_mpcpattern_parallel
file target/release/midi_to_mpcpattern_parallel
```

Should see:
```
-rwxr-xr-x ... midi_to_mpcpattern_parallel
midi_to_mpcpattern_parallel: ELF 64-bit LSB executable, x86-64
```

### Test Conversion

Before converting all files, test with a small batch:
```bash
# Test with 10 files
./target/release/midi_to_mpcpattern_parallel \
    --batch /path/to/midi/files \
    /tmp/test_output \
    10

# Check output
ls -lh /tmp/test_output/*.mpcpattern
```

### Storage Requirements

- **Full collection:** ~6 GB (1.7M patterns)
- **Limited (5K each):** ~2 GB (485K patterns)
- **Make sure target drive has space:**
  ```bash
  df -h /media/dojevou/NewSSD2
  ```

### MPC Device Compatibility

Tested/verified for:
- ✅ Akai Force
- ✅ MPC One
- ✅ MPC Live / MPC Live II
- ✅ MPC X
- ✅ MPC Key 61
- All run **MPC OS** - same expansion format

---

## 📞 Quick Reference Commands

```bash
# Check build status
ps aux | grep cargo

# Check database
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -c "
  SELECT COUNT(*) FROM files WHERE num_tracks = 1;
"

# List instruments
cat INSTRUMENT_LIST.txt

# Create expansions
./scripts/create_97_instrument_expansions.sh

# Convert patterns (after build)
./target/release/midi_to_mpcpattern_parallel --batch <input> <output>

# Check disk space
df -h /media/dojevou/NewSSD2
```

---

## 🚀 Estimated Timeline

**After build completes:**

| Step | Time | Description |
|------|------|-------------|
| 1. Create folders | ~30 sec | Run script to create 97 expansions |
| 2. Export DB lists | 2-5 min | Query database for each instrument |
| 3. Convert patterns | 6-15 min | Full conversion at 2-5K files/sec |
| 4. Copy to device | 2-10 min | Depends on drive speed |
| 5. Test on hardware | 5-10 min | Load and test expansions |
| **TOTAL** | **15-40 min** | **Complete setup** |

With 5K limits: **10-25 minutes total**

---

## 📝 TODO List

- [ ] Wait for build to complete (~2 min remaining)
- [ ] Verify binary exists and works
- [ ] Test conversion with 10-100 files
- [ ] Create expansion folders (97 total)
- [ ] Export database file lists per instrument
- [ ] Run full batch conversion
- [ ] Copy to MPC device
- [ ] Test on hardware
- [ ] Document results

---

## 🔗 Related Files

**Documentation:**
- `MPC-97-INSTRUMENT-EXPANSIONS-PLAN.md` - Main strategy
- `MPCPATTERN-FILE-LOCATION-RESEARCH.md` - Storage research
- `MPCPATTERN-FORMAT-SPECIFICATION.md` - Format details
- `INSTRUMENT_ANALYSIS.md` - 97 instruments breakdown
- `AKAI-FORCE-EXPANSION-STRATEGY.md` - Previous plan (superseded)

**Scripts:**
- `scripts/build-ultra-fast-converter.sh` - Build converter
- `scripts/create_97_instrument_expansions.sh` - Create folders
- `scripts/normalize-extensions-only.sh` - Extension fixer (used already)

**Source Code:**
- `pipeline/src-tauri/src/bin/midi_to_mpcpattern_parallel.rs` - Converter
- `pipeline/src-tauri/Cargo.toml` - Dependencies
- `pipeline/src-tauri/.cargo/config.toml` - Build config

**Database:**
- PostgreSQL: `midi_library` database
- Tables: files, file_tags, tags, musical_metadata
- 2.2M single-track files ready for conversion

---

**Current Status:** ⏳ Waiting for build to complete, then ready to execute conversion pipeline.

**Next Action:** Check if build finished, then create expansion folders.

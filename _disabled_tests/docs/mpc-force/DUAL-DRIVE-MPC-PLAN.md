# Dual-Drive MPC Organization Plan

## Overview

Create identical MPC 3.0-optimized MIDI libraries on **both drives** for redundancy and flexibility:
- **Primary**: `/dev/sda1` (59.2GB available)
- **Backup**: `/dev/sdb1` (size TBD)

---

## Current Status

### ✅ Analysis Running
- **Progress**: 48.5% (591,000 / 1,218,622 files)
- **Speed**: ~4,850 files/sec
- **ETA**: 2 minutes to completion
- **Result**: All 1.2M files will have BPM, Key, Notes, Time Signature

---

## Dual-Drive Strategy

### Option A: Simultaneous Copy (Recommended)
Write to both drives simultaneously during organization:
- **Pros**: Faster overall (one pass), guaranteed sync
- **Cons**: Requires more RAM/CPU during organization
- **Implementation**: Use hardlinks or parallel copy operations

### Option B: Sequential Copy
Organize to `/dev/sda1` first, then rsync to `/dev/sdb1`:
- **Pros**: Simpler, lower resource usage
- **Cons**: Takes twice as long, potential for drift
- **Implementation**: Standard rsync after phase 8 complete

**Decision**: Use **Option B (Sequential)** for simplicity and reliability

---

## Phases 5-12 Implementation Plan

### Phase 5: Advanced Categorization (Database Only)
**Estimated Time**: 5-10 minutes
**What it does**:
- Query all 1.2M files from database
- Categorize each file:
  - **Type**: DRUMS / MELODIC / LOOPS / ONE_SHOTS
  - **BPM Range**: 60-80, 80-100, 100-120, 120-140, 140-160, 160+
  - **Key**: All 24 major/minor keys
  - **Instrument**: KICK, SNARE, BASS, KEYS, etc.
  - **Time Signature**: 4/4, 3/4, 6/8, etc.
- Store categorization in database (new columns or tags)

**Code to Write**:
```rust
// Categorization logic
struct FileCategory {
    file_type: FileType,      // DRUMS, MELODIC, LOOPS, ONE_SHOTS
    bpm_range: BpmRange,       // 60-80, 80-100, etc.
    key: Option<String>,       // "C", "Dm", etc.
    instrument: Vec<String>,   // ["KICK", "SNARE"]
    time_sig: Option<String>,  // "4/4", "3/4"
}

fn categorize_file(file: &File, metadata: &MusicalMetadata) -> FileCategory {
    // Implement categorization logic
}
```

### Phase 6: File Splitting
**Estimated Time**: 30-60 minutes
**What it does**:
- Find all multi-track MIDI files (track_count > 1)
- Split each track into separate MIDI files
- Store split files in temp directory
- Update database with split file references

**Binary**: `split` (already exists!)
**Usage**:
```bash
./target/release/split \
  --input-dir /home/dojevou/Uncontaminated/floorp_downloads/midi \
  --output-dir /tmp/midi-splits \
  --workers 24
```

**Result**: ~2-3x more files (if average 2-3 tracks per file)

### Phase 7: MPC Renaming
**Estimated Time**: 10-15 minutes
**What it does**:
- Generate MPC-compatible filenames for all files
- Format: `{TYPE}_{TIMESIG}_{BPM}BPM_{KEY}_{ID}_{SOURCE}_{DESC}.mid`
- Examples:
  - `KICK_4-4_128BPM_C_000001_DrumPack_Heavy.mid`
  - `BASS_4-4_140BPM_Am_000042_MegaCollection_Deep.mid`

**Code to Write**:
```rust
fn generate_mpc_filename(
    category: &FileCategory,
    file_id: u32,
    source: &str,
    description: &str
) -> String {
    format!(
        "{}_{}_{}BPM_{}_{:06}_{}_{}.mid",
        category.file_type,
        category.time_sig.unwrap_or("4-4"),
        category.bpm_range.midpoint(),
        category.key.unwrap_or("C"),
        file_id,
        sanitize(source),
        sanitize(description)
    )
}
```

### Phase 8A: Physical Organization - Primary Drive (/dev/sda1)
**Estimated Time**: 20-30 minutes
**What it does**:
- Create MPC folder structure on `/dev/sda1`
- Copy files from source to organized folders
- Use categorization from Phase 5
- Update database with new paths

**Mount Point**: `/mnt/sda1` (need to verify)
**Target Structure**:
```
/mnt/sda1/MPC_MIDI_LIBRARY/
├── DRUMS/
│   ├── BY_BPM/
│   │   ├── 060-080_BPM/
│   │   ├── 080-100_BPM/
│   │   ├── 100-120_BPM/
│   │   ├── 120-140_BPM/
│   │   ├── 140-160_BPM/
│   │   └── 160-PLUS_BPM/
│   ├── BY_TYPE/
│   │   ├── KICKS/
│   │   ├── SNARES/
│   │   ├── HIHATS/
│   │   ├── CYMBALS/
│   │   ├── PERCUSSION/
│   │   ├── FILLS/
│   │   └── FULL_KITS/
│   └── BY_TIME_SIG/
│       ├── 4-4/
│       ├── 3-4/
│       ├── 6-8/
│       └── OTHER/
├── MELODIC/
│   ├── BY_KEY/
│   │   ├── C_MAJOR/
│   │   ├── C#_MAJOR/
│   │   ├── D_MAJOR/
│   │   ├── ... (all 24 keys)
│   ├── BY_BPM/
│   │   ├── 060-080_BPM/
│   │   └── ... (same as drums)
│   └── BY_INSTRUMENT/
│       ├── BASS/
│       ├── KEYS/
│       ├── STRINGS/
│       ├── BRASS/
│       ├── LEADS/
│       └── PADS/
├── LOOPS/
│   ├── BY_BPM/
│   │   └── ... (same structure)
│   └── BY_GENRE/
│       ├── HIPHOP/
│       ├── TRAP/
│       ├── LOFI/
│       ├── HOUSE/
│       ├── TECHNO/
│       └── OTHER/
├── ONE_SHOTS/
│   ├── DRUMS/
│   ├── BASS/
│   ├── MELODY/
│   └── FX/
└── METADATA/
    ├── bpm_lookup.csv
    ├── key_compatibility.csv
    └── file_index.json
```

**Code to Write**:
```rust
fn organize_to_drive(
    target_root: &Path,  // /mnt/sda1/MPC_MIDI_LIBRARY
    files: Vec<(File, FileCategory)>,
    workers: usize
) -> Result<()> {
    // Create folder structure
    // Copy files to appropriate folders
    // Update database with new paths
}
```

### Phase 8B: Physical Organization - Backup Drive (/dev/sdb1)
**Estimated Time**: 20-30 minutes
**What it does**:
- Rsync entire `/mnt/sda1/MPC_MIDI_LIBRARY` to `/mnt/sdb1/MPC_MIDI_LIBRARY`
- Verify checksums
- Create backup metadata

**Command**:
```bash
rsync -avP --checksum \
  /mnt/sda1/MPC_MIDI_LIBRARY/ \
  /mnt/sdb1/MPC_MIDI_LIBRARY/
```

### Phase 9: Advanced Deduplication
**Estimated Time**: 5 minutes
**What it does**:
- Identify duplicate files across both drives (already done via content_hash)
- Keep best version (prefer named files over numbered)
- Log duplicates to `/mnt/sda1/MPC_MIDI_LIBRARY/METADATA/duplicates.txt`

### Phase 10: Metadata Export
**Estimated Time**: 5 minutes
**What it does**:
- Export BPM lookup table (CSV)
- Export key compatibility chart (CSV)
- Export full file index (JSON)
- Store in `/mnt/sda1/MPC_MIDI_LIBRARY/METADATA/`
- Copy to `/mnt/sdb1/MPC_MIDI_LIBRARY/METADATA/`

**Files to Generate**:
1. **bpm_lookup.csv**:
   ```csv
   filename,bpm,key,time_sig,path
   KICK_4-4_128BPM_C_000001.mid,128,C,4/4,DRUMS/BY_BPM/120-140_BPM/
   ```

2. **key_compatibility.csv**:
   ```csv
   key,compatible_keys,relative_minor_major
   C,G|F|Am|Em,Am
   Dm,F|C|Am|Gm,F
   ```

3. **file_index.json**:
   ```json
   {
     "total_files": 1218622,
     "categories": {
       "DRUMS": 523000,
       "MELODIC": 450000,
       "LOOPS": 200000,
       "ONE_SHOTS": 45622
     },
     "files": [...]
   }
   ```

### Phase 11: Verification
**Estimated Time**: 10 minutes
**What it does**:
- Verify all files on `/mnt/sda1` are readable
- Verify all files on `/mnt/sdb1` are readable
- Check folder structure completeness
- Validate naming conventions
- Test MPC 3.0 compatibility (if hardware available)

**Checks**:
```rust
fn verify_organization(root: &Path) -> VerificationReport {
    VerificationReport {
        total_files: count_files(root),
        readable: count_readable_files(root),
        corrupted: count_corrupted_files(root),
        naming_violations: check_naming_conventions(root),
        missing_folders: check_folder_structure(root),
    }
}
```

### Phase 12: Final Sync & Cleanup
**Estimated Time**: 5 minutes
**What it does**:
- Final rsync from sda1 to sdb1 (pick up any changes)
- Verify both drives identical
- Clean up temp directories
- Generate final report

---

## Drive Space Requirements

### Estimated Space Usage

**Current Database**: 1,218,622 files

**After Splitting** (Phase 6):
- Assume average 2.5 tracks per file
- New files: 1,218,622 × 2.5 = **~3,046,555 files**

**File Size Estimates**:
- Average MIDI file: ~30KB (based on typical samples)
- Total size: 3,046,555 × 30KB ≈ **91.4 GB**

**With Metadata & Overhead**:
- Files: 91.4 GB
- Metadata exports: ~500 MB
- Folder structure overhead: ~100 MB
- **Total per drive**: **~92 GB**

### Drive Capacity Check

**Required**:
- `/dev/sda1`: 92 GB (have 59.2 GB) ⚠️ **NOT ENOUGH**
- `/dev/sdb1`: 92 GB (size unknown)

**Options**:
1. **Don't split all files** - Only split files with >3 tracks (reduces total)
2. **Use compression** - Create .zip archives of less-used categories
3. **Selective organization** - Only organize most useful files
4. **External drive** - Use larger drive for full collection

---

## Recommended Approach

### Phase 5-12 Modified Plan (Fits 59.2GB)

**Strategy**: Smart Selection + Compression

1. **Phase 5**: Categorize ALL files in database
2. **Phase 6**: Split ONLY drum files and key loops (not everything)
3. **Phase 7**: Rename ALL files
4. **Phase 8A**: Organize to `/mnt/sda1`:
   - **DRUMS**: All drum files (highest priority)
   - **MELODIC**: Top 50% by BPM/key distribution
   - **LOOPS**: Top 25% by quality metrics
   - **ONE_SHOTS**: All (from splits)
5. **Phase 8B**: Mirror to `/mnt/sdb1`
6. **Phase 9-12**: Metadata, verification, cleanup

**Estimated Final Size**: ~50-55 GB (fits on 59.2GB drive)

---

## Implementation Timeline

### After Analysis Completes (~2 minutes)

**Immediate**:
1. Check drive mount points and sizes
2. Verify `/dev/sda1` and `/dev/sdb1` are accessible
3. Estimate actual space with current files

**Phase 5-8 Implementation** (~2-3 hours):
- Write categorization code
- Run split binary (selective)
- Generate MPC filenames
- Create folder structures
- Copy files to drives

**Phase 9-12 Completion** (~30 minutes):
- Deduplication logging
- Metadata export
- Verification
- Final sync

**Total Estimated Time**: **2.5-3.5 hours** for complete MPC workflow

---

## Next Steps

1. ✅ Wait for analyze to complete (~2 minutes remaining)
2. Check drive mount points:
   ```bash
   lsblk
   df -h /dev/sda1 /dev/sdb1
   ```
3. Verify drive sizes and available space
4. Decide on selective vs full organization
5. Start Phase 5 implementation

---

**Status**: Analysis running, MPC plan ready, awaiting drive verification.

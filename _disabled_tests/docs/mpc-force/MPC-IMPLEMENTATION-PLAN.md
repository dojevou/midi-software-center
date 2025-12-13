# MPC Organization - Implementation Plan

## Drive Configuration ✅

```
Primary:  /dev/sda1 → /media/dojevou/NewSSD2   (916GB, 45GB available)
Backup:   /dev/sdb1 → /media/dojevou/MPC_DRIVE (60GB,  21GB available)
```

**Strategy**: Full organization on sda1 (45GB available), selective mirror to sdb1 (21GB)

---

## Current Status

### ✅ Analysis Progress
- **Running**: 48.5% complete (591K / 1.2M files)
- **Speed**: ~4,850 files/sec
- **ETA**: ~2 minutes to 100% completion

---

## Implementation Plan

### Phase 5-12 Execution Order

**All phases will run AFTER analysis completes.**

---

### Phase 5: Advanced Categorization
**Time**: 5-10 minutes
**Action**: Categorize all 1.2M files in database

**Categories to assign**:
1. **File Type**: DRUMS | MELODIC | LOOPS | ONE_SHOTS
2. **BPM Range**: 60-80 | 80-100 | 100-120 | 120-140 | 140-160 | 160+
3. **Key**: C, C#, D... (all 24 keys)
4. **Instrument**: KICK, SNARE, BASS, KEYS, etc.
5. **Time Signature**: 4/4, 3/4, 6/8, etc.

**Output**: Database columns populated with categorization

---

### Phase 6: File Splitting
**Time**: 30-60 minutes
**Action**: Split multi-track MIDI files

**Binary**: `./target/release/split` (already exists)

**Command**:
```bash
DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library" \
  ./target/release/split \
  --output-dir /tmp/midi-splits \
  --workers 24
```

**Expected**: ~2-3M additional files (if avg 2-3 tracks per file)

---

### Phase 7: MPC Renaming
**Time**: 10-15 minutes
**Action**: Generate MPC-compatible filenames

**Format**: `{TYPE}_{TIMESIG}_{BPM}BPM_{KEY}_{ID}_{SOURCE}_{DESC}.mid`

**Examples**:
- `KICK_4-4_128BPM_C_000001_DrumPack_Heavy.mid`
- `BASS_4-4_140BPM_Am_000042_Collection_Deep.mid`

---

### Phase 8A: Primary Drive Organization (/dev/sda1)
**Time**: 20-30 minutes
**Action**: Create MPC folder structure and copy files

**Target**: `/media/dojevou/NewSSD2/MPC_MIDI_LIBRARY/`
**Space**: 45GB available (enough for full collection)

**Structure**:
```
/media/dojevou/NewSSD2/MPC_MIDI_LIBRARY/
├── DRUMS/
│   ├── BY_BPM/
│   ├── BY_TYPE/
│   └── BY_TIME_SIG/
├── MELODIC/
│   ├── BY_KEY/
│   ├── BY_BPM/
│   └── BY_INSTRUMENT/
├── LOOPS/
│   ├── BY_BPM/
│   └── BY_GENRE/
├── ONE_SHOTS/
└── METADATA/
```

---

### Phase 8B: Backup Drive Organization (/dev/sdb1)
**Time**: 15-20 minutes
**Action**: Selective mirror to MPC_DRIVE

**Target**: `/media/dojevou/MPC_DRIVE/MPC_MIDI_LIBRARY/`
**Space**: 21GB available (selective copy)

**Strategy**: Copy most important files only:
- All DRUMS (highest priority for MPC)
- Top 50% MELODIC by quality
- Top 25% LOOPS
- All ONE_SHOTS

**Command**:
```bash
rsync -avP --checksum \
  --include="DRUMS/***" \
  --include="ONE_SHOTS/***" \
  --include="MELODIC/BY_BPM/100-140_BPM/***" \
  --exclude="*" \
  /media/dojevou/NewSSD2/MPC_MIDI_LIBRARY/ \
  /media/dojevou/MPC_DRIVE/MPC_MIDI_LIBRARY/
```

---

### Phase 9: Deduplication Logging
**Time**: 5 minutes
**Action**: Generate duplicate file report

**Output**: `/media/dojevou/NewSSD2/MPC_MIDI_LIBRARY/METADATA/duplicates.txt`

---

### Phase 10: Metadata Export
**Time**: 5 minutes
**Action**: Export lookup tables and indexes

**Files**:
1. `bpm_lookup.csv` - BPM/key/path lookup
2. `key_compatibility.csv` - Compatible key chart
3. `file_index.json` - Complete file catalog

**Location**: Both drives `/METADATA/` folder

---

### Phase 11: Verification
**Time**: 10 minutes
**Action**: Verify organization completeness

**Checks**:
- All files readable on both drives
- Folder structure complete
- Naming conventions correct
- File counts match expectations

---

### Phase 12: Final Sync
**Time**: 5 minutes
**Action**: Final verification and cleanup

- Sync any changes to backup drive
- Clean up temp directories
- Generate final report

---

## Estimated Total Time

| Phase | Task | Time |
|-------|------|------|
| 5 | Categorization | 5-10 min |
| 6 | File splitting | 30-60 min |
| 7 | MPC renaming | 10-15 min |
| 8A | Primary drive org | 20-30 min |
| 8B | Backup drive org | 15-20 min |
| 9 | Deduplication | 5 min |
| 10 | Metadata export | 5 min |
| 11 | Verification | 10 min |
| 12 | Final sync | 5 min |
| **TOTAL** | **Full workflow** | **105-160 min** |

**Expected**: **2-2.5 hours** for complete MPC organization

---

## Next Actions

**Right Now** (while analysis runs):
1. ✅ Plan complete
2. ✅ Drives verified
3. ⏳ Wait for analysis (~2 min remaining)

**After Analysis**:
1. Start Phase 5 implementation
2. Build any missing binaries
3. Execute phases 5-12 sequentially

---

**Status**: Ready to proceed once analysis completes!

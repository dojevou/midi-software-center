# MIDI Organization Project for MPC Workflow

## Project Goals
Extract, rename, split, and organize ~3.1M MIDI files from 3 archive sources into an MPC 3.0-optimized folder structure on `/dev/sda1` (59.2GB).

## Source Archives
1. `/home/dojevou/Uncontaminated/floorp_downloads/845,000_midi/` (6.9GB extracted)
2. `/home/dojevou/Uncontaminated/floorp_downloads/--1.005.000 DRUM Midi Collection---20251010T231010Z-1-001.zip` (1.7GB)
3. `/home/dojevou/Uncontaminated/floorp_downloads/--1.002.000 Midi Collection ---20251010T230939Z-1-001.zip` (1.8GB)

**Total:** ~10.4GB source data → ~50GB target (with splits and organization)

## Target Structure (MPC 3.0 Optimized)

Based on Akai MPC best practices, organize by **Type → BPM Range → Key → Category**:

```
/mnt/sda1/
└── MPC_MIDI_LIBRARY/
    ├── DRUMS/                      # Drum patterns (Channel 10 + drum notes)
    │   ├── BY_BPM/
    │   │   ├── 60-80_BPM/
    │   │   ├── 80-100_BPM/
    │   │   ├── 100-120_BPM/
    │   │   ├── 120-140_BPM/
    │   │   ├── 140-160_BPM/
    │   │   └── 160-PLUS_BPM/
    │   ├── BY_TYPE/
    │   │   ├── KICKS/
    │   │   ├── SNARES/
    │   │   ├── HIHATS/
    │   │   ├── PERCUSSION/
    │   │   ├── FILLS/
    │   │   └── FULL_KITS/
    │   └── BY_TIME_SIG/
    │       ├── 4-4/
    │       ├── 3-4/
    │       ├── 6-8/
    │       └── OTHER/
    │
    ├── MELODIC/                    # Melodic instruments
    │   ├── BY_KEY/
    │   │   ├── C_MAJOR/
    │   │   ├── D_MAJOR/
    │   │   ├── E_MAJOR/
    │   │   ├── A_MINOR/
    │   │   ├── D_MINOR/
    │   │   └── [... all 24 keys]/
    │   ├── BY_BPM/
    │   │   ├── 60-80_BPM/
    │   │   ├── 80-100_BPM/
    │   │   ├── 100-120_BPM/
    │   │   ├── 120-140_BPM/
    │   │   └── 140-PLUS_BPM/
    │   └── BY_INSTRUMENT/
    │       ├── BASS/
    │       ├── KEYS/
    │       ├── STRINGS/
    │       ├── BRASS/
    │       ├── LEADS/
    │       └── PADS/
    │
    ├── LOOPS/                      # Full loops and sequences
    │   ├── BY_BPM/
    │   │   ├── 60-80_BPM/
    │   │   ├── 80-100_BPM/
    │   │   ├── 100-120_BPM/
    │   │   ├── 120-140_BPM/
    │   │   └── 140-PLUS_BPM/
    │   └── BY_GENRE/
    │       ├── HIPHOP/
    │       ├── TRAP/
    │       ├── LOFI/
    │       ├── HOUSE/
    │       ├── TECHNO/
    │       └── OTHER/
    │
    ├── ONE_SHOTS/                  # Single-track MIDI (split results)
    │   ├── DRUMS/
    │   ├── BASS/
    │   ├── MELODY/
    │   └── FX/
    │
    └── UNSORTED/                   # Fallback for uncategorized
        └── NEEDS_REVIEW/
```

## File Naming Convention (MPC Compatible)

Format: `{TYPE}_{TIMESIG}_{BPM}BPM_{KEY}_{ID}_{SOURCE}_{DESCRIPTION}.mid`

Examples:
- `KICK_4-4_128BPM_C_000001_DrumPack_Heavy.mid`
- `BASS_4-4_140BPM_Am_000042_MegaCollection_Deep.mid`
- `HIHAT_4-4_160BPM_C_000123_VintageDrums_OpenHat_L01.mid` (split track)

**Key Features:**
- 8.3-compatible stem (works on older FAT32 if needed)
- Sortable by BPM (numeric prefix)
- Key information visible
- Source pack preserved
- Unique ID prevents conflicts
- Split layers numbered (L01, L02, etc.)

## Processing Pipeline

### Phase 1: Extraction (Parallel Processing)
- Extract all 3 archives to temp directories
- Preserve folder structure for metadata extraction
- Target: ~3.1M MIDI files extracted

### Phase 2: Analysis & Categorization
For each MIDI file:
1. **Parse MIDI** → Extract metadata
   - BPM (from filename patterns + MIDI tempo events)
   - Key signature (from MIDI meta events)
   - Time signature (from MIDI meta events)
   - Channel info (detect drums on Ch 10)
   - Instrument (GM program changes)
   - Track count

2. **Categorize** → Determine folder placement
   - **DRUMS:** Channel 10 OR drum notes (35-81) OR "drum" in filename
   - **MELODIC:** Non-drum instruments, key signature exists
   - **LOOPS:** Multi-track files (Format 1)
   - **ONE_SHOTS:** Single-track files (Format 0 or post-split)

3. **Split Multi-Track** → Create one-shots
   - Use track_splitter.rs logic
   - Generate layer filenames (L01, L02, etc.)
   - Preserve tempo/time signature

### Phase 3: Rename & Organize
1. Generate production filename using naming conventions
2. Check for conflicts (resolve with version numbers)
3. Move to appropriate folder based on:
   - Primary category (DRUMS/MELODIC/LOOPS/ONE_SHOTS)
   - BPM range (60-80, 80-100, etc.)
   - Key signature (C, Dm, etc.)
   - Instrument type (KICK, BASS, etc.)

### Phase 4: Deduplication
- Calculate BLAKE3 hashes (already in pipeline)
- Identify duplicates across sources
- Keep best version (prefer named > numbered)
- Log duplicates to `duplicates.txt`

## Technology Stack

**Rust (Standalone Binary):**
- `midly` - MIDI parsing (already in pipeline)
- `rayon` - Parallel processing
- `blake3` - Fast hashing
- `zip` - Archive extraction
- `walkdir` - Directory traversal
- `indicatif` - Progress bars

**No Database:** Pure filesystem operations (MPC browses directly)

## Performance Targets

- **Extraction:** 10,000 files/sec (6.9GB in ~5 min)
- **Analysis:** 1,000 files/sec (~50 min for 3M files)
- **Organization:** 5,000 files/sec (~10 min move operations)
- **Total Time:** ~65 minutes for complete organization

## Disk Space

- Source: 10.4GB (compressed/extracted)
- Target: ~50GB (with splits, deduplication should reduce)
- Available: 59.2GB on /dev/sda1 ✅ (9GB safety margin)

## Execution Steps

```bash
# 1. Mount drive
sudo mkdir -p /mnt/sda1
sudo mount /dev/sda1 /mnt/sda1
sudo chown -R $USER:$USER /mnt/sda1

# 2. Build organizer
cd /mnt/sda1/midi-organizer
cargo build --release

# 3. Run organization pipeline
./target/release/midi-organizer \
  --source "/home/dojevou/Uncontaminated/floorp_downloads/845,000_midi/" \
  --source "/home/dojevou/Uncontaminated/floorp_downloads/--1.005.000 DRUM Midi Collection---20251010T231010Z-1-001.zip" \
  --source "/home/dojevou/Uncontaminated/floorp_downloads/--1.002.000 Midi Collection ---20251010T230939Z-1-001.zip" \
  --output "/mnt/sda1/MPC_MIDI_LIBRARY" \
  --threads 16 \
  --split-multi-track \
  --deduplicate \
  --progress

# 4. Verify results
./target/release/midi-organizer --verify "/mnt/sda1/MPC_MIDI_LIBRARY"
```

## MPC 3.0 Workflow Benefits

1. **BPM-First Organization:** Quickly find loops matching project tempo
2. **Key Organization:** Easy harmonic matching
3. **Split One-Shots:** Individual drum hits ready for sampling
4. **Clear Naming:** Instant visual feedback in MPC browser
5. **No Database:** Direct filesystem access (MPC native)
6. **Portable:** Works on MPC Live/One/X via USB/SD card

## Next Steps

1. ✅ Create project structure in `/mnt/sda1/midi-organizer/`
2. ✅ Copy relevant pipeline modules (extractor, naming, splitter)
3. ✅ Implement main orchestrator with progress tracking
4. ✅ Add MPC-specific categorization logic
5. ✅ Generate mount script and README
6. ⏳ Execute organization pipeline

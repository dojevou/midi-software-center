# MIDI Pipeline Steps

## Main Pipeline Phases (What You Control):

**5 Main Phases (Correct Order):**

1. **Phase 1: Import** - Import files to database (includes archive extraction, hash, dedup, parse, filename metadata, tags)
2. **Phase 2: Strict Sanitization** - Clean filenames (spaces→_, .midi→.mid, remove special chars)
3. **Phase 3: Track Splitting** - Split multi-track MIDI files
4. **Phase 4: Analysis** - BPM, key, drum detection (from MIDI events)
5. **Phase 5: Production Renaming** - Metadata-based descriptive names (128bpm_Cmaj_bass_loop.mid)

---

## Sub-Operations (Automatic within phases):

**What happens inside "Import" (Phase 1):**
1. Archive Extraction (if .zip/.rar/.7z)
2. Hash Calculation (BLAKE3)
3. Deduplication (skip existing files by hash)
4. MIDI Parsing (structure, tracks, events)
5. Filename Metadata Extraction (BPM, key, genre from filename)
6. Auto-Tagging (generate tags)
7. Database Insert
8. Search Index Building

**What happens inside "Analysis" (Phase 4):**
1. BPM Detection (from MIDI events)
2. Key Detection (Krumhansl-Schmuckler algorithm)
3. Drum Analysis (patterns, cymbals, techniques)
4. Chord Analysis (progressions)
5. Musical Metadata Storage (duration, time signature)

**What happens inside "Splitting" (Phase 3):**
1. Multi-track detection
2. Channel separation
3. Individual track files created

## New Two-Phase Renaming System:

**Phase 0: Strict Sanitization** (NEW - right after extraction)
- Replace spaces with underscores
- Convert .midi → .mid
- Convert .MID → .mid (force lowercase)
- Remove ALL special characters (keep only: letters, numbers, _ -)
- Example: "My Song (2023).MIDI" → "My_Song_2023.mid"

**Phase 1: Production Renaming** (existing - metadata-based)
- Generate clean filenames based on BPM, key, tags
- Example: "My_Song_2023.mid" → "128bpm_Cmaj_bass_loop.mid"

## Correct Pipeline Order:

```
Phase 1: Import (archive extraction, hash, dedup, parse, filename metadata, tags, DB insert)
Phase 2: Strict Sanitization (spaces→_, .midi→.mid, remove special chars)
Phase 3: Track Splitting (multi-track files → individual tracks)
Phase 4: Analysis (BPM, key, drum detection from MIDI events)
Phase 5: Production Renaming (metadata-based: 128bpm_Cmaj_bass_loop.mid)
```

**Why This Order:**
- Import FIRST: Get files into database as-is
- Sanitize SECOND: Clean up filenames for consistency
- Split THIRD: Break multi-track files into individual tracks
- Analyze FOURTH: Extract musical metadata from MIDI content
- Rename LAST: Use all collected metadata for final naming

## What Each Import Does (Phase 2 combines many steps):

- Archive extraction (if needed)
- Hash calculation (BLAKE3)
- Deduplication check
- MIDI parsing
- Filename metadata extraction
- Auto-tagging
- Database insert

## Skip Flags Available:

```bash
orchestrator --source /path --skip-import    # Skip import phase
orchestrator --source /path --skip-analysis  # Skip analysis phase
orchestrator --source /path --skip-split     # Skip splitting phase
orchestrator --source /path --skip-rename    # Skip rename phase
```

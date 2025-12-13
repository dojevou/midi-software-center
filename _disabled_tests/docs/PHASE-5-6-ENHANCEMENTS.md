# Phase 5-6 Enhancements: Advanced Rename & Export

**Date:** November 18, 2025
**Status:** ⚠️  IMPLEMENTATION IN PROGRESS - Compilation fixes needed
**Components:** Rename Worker (Phase 5), Export Worker (Phase 6)

---

## Overview

Enhanced the Pipeline Orchestrator with:
1. **Phase 5 Rename**: Bar-length calculation + strict sanitization + metadata-rich filenames
2. **Phase 6 Export**: Multi-strategy categorization for Akai Force/MPC One
3. **Filepath Metadata Extraction**: Fallback metadata from directory structure

---

## Phase 5: Enhanced Rename Worker

### New Filename Format

**Before:** `{bpm}bpm_{key}_{stem}.mid`
Example: `128bpm_Cmaj_bass_loop.mid`

**After:** `{bars}-{bpm}bpm_{key}_{folder}-{name}.mid`
Example: `32-090bpm_Cmin_VirusMelodies-01.mid`

### Implementation Details (`workers/rename.rs`)

#### 1. Bar Length Calculation

```rust
fn calculate_bars(
    duration_seconds: Option<f64>,
    bpm: Option<f64>,
    time_sig_numerator: Option<i32>,
) -> Option<i32> {
    match (duration_seconds, bpm, time_sig_numerator) {
        (Some(dur), Some(tempo), Some(numerator)) if dur > 0.0 && tempo > 0.0 && numerator > 0 => {
            let beats = (dur * tempo) / 60.0;
            let bars = beats / (numerator as f64);
            Some(bars.round() as i32)
        }
        _ => None,
    }
}
```

**Formula:**
```
beats = (duration_seconds × BPM) / 60
bars = beats / time_signature_numerator
```

**Example:**
- Duration: 8.5 seconds
- BPM: 90
- Time Signature: 4/4
- Calculation: `(8.5 × 90) / 60 = 12.75 beats → 12.75 / 4 = 3.19 bars → 3 bars`

#### 2. Strict Character Sanitization

**Rules:**
- **Allowed:** `a-z`, `A-Z`, `0-9`, `_`, `-`, `.`
- **Removed:** All other characters replaced with `_`
- **No Duplicates:** Consecutive `__`, `--`, `..` reduced to single character
- **Trim:** Remove trailing special characters

```rust
fn sanitize_strict(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut last_was_special = false;

    for c in s.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' => {
                result.push(c);
                last_was_special = false;
            }
            '_' | '-' | '.' => {
                if !last_was_special {
                    result.push(c);
                    last_was_special = true;
                }
            }
            _ => {
                if !last_was_special {
                    result.push('_');
                    last_was_special = true;
                }
            }
        }
    }

    result.trim_end_matches(|c| c == '_' || c == '-' || c == '.').to_string()
}
```

**Examples:**
- `Virus  Melodies` → `Virus_Melodies`
- `808--Bass__Loops` → `808-Bass_Loops`
- `Chords (Cmaj)` → `Chords_Cmaj`
- `Track #1!!!` → `Track_1`

#### 3. Metadata Loading

```rust
let metadata = sqlx::query_as::<_, (
    Option<f64>,    // tempo_bpm
    Option<String>, // key_signature
    Option<f64>,    // duration_seconds
    Option<i32>,    // time_signature_numerator
)>(
    "SELECT tempo_bpm, key_signature::text, duration_seconds, time_signature_numerator
     FROM musical_metadata WHERE file_id = $1"
)
.bind(file_record.id)
.fetch_optional(db_pool)
.await?;
```

#### 4. Filename Assembly

```rust
// Parts: [bars, bpm, key, folder, name]
// Format: {bars}-{bpm}bpm_{key}_{folder}-{name}.mid

if bars.is_some() && bpm.is_some() {
    let bars_bpm = format!("{}-{}", parts[0], parts[1]); // "32-090bpm"
    let middle = parts[2..parts.len()-1].join("_");      // "Cmin_VirusMelodies"
    let name = &parts[parts.len()-1];                     // "01"
    format!("{}_{}_{}.mid", bars_bpm, middle, name)      // "32-090bpm_Cmin_VirusMelodies_01.mid"
}
```

---

## Phase 6: Advanced Export Worker

### Multi-Strategy Categorization

**Strategy Priority:**
1. **Auto-tags** (most reliable - from database)
2. **MIDI note range analysis** (bass vs melody vs drums)
3. **Parent folder analysis** (traditional method)
4. **Filename patterns** (last resort)

### MPC/Force Categories (13 Types)

**Drums (Granular):**
- `DrumKicks` → `MPC_Documents/SAMPLES/Drums/Kicks`
- `DrumSnares` → `MPC_Documents/SAMPLES/Drums/Snares`
- `DrumHats` → `MPC_Documents/SAMPLES/Drums/Hats`
- `DrumCymbals` → `MPC_Documents/SAMPLES/Drums/Cymbals`
- `DrumToms` → `MPC_Documents/SAMPLES/Drums/Toms`
- `DrumPerc` → `MPC_Documents/SAMPLES/Drums/Percussion`
- `Drums` → `MPC_Documents/SAMPLES/Drums`

**Melodic/Harmonic:**
- `Bass` → `MPC_Documents/SAMPLES/Bass`
- `Melody` → `MPC_Documents/SAMPLES/Melody`
- `Chords` → `MPC_Documents/SAMPLES/Chords`
- `Progressions` → `MPC_Documents/Progressions`

**Other:**
- `FX` → `MPC_Documents/SAMPLES/FX`
- `Loops` → `MPC_Documents/SAMPLES/Loops`

### Strategy 1: Auto-Tags (Database)

```rust
// Load tags from database
let tags: Vec<String> = sqlx::query_scalar(
    "SELECT tag FROM file_tags WHERE file_id = $1"
)
.bind(file_record.id)
.fetch_all(db_pool)
.await
.unwrap_or_default();

let tags_lower: Vec<String> = tags.iter().map(|t| t.to_lowercase()).collect();

// Drum detection (most specific first)
if tags_lower.iter().any(|t| t.contains("kick") || t == "bd" || t == "bass-drum") {
    return Ok(MPCCategory::DrumKicks);
}
```

**Tag Patterns:**
- Kicks: `kick`, `bd`, `bass-drum`, `808-kick`
- Snares: `snare`, `sd`, `clap`
- Hi-hats: `hihat`, `hi-hat`, `hh`, `hat`, `closed-hat`, `open-hat`
- Cymbals: `crash`, `ride`, `china`, `cymbal`, `splash`
- Bass: `bass`, `808`, `sub-bass`, `bassline`
- Chords: `chord`, `progression`, `harmony`, `pad`

### Strategy 2: MIDI Note Range Analysis

```rust
let note_range: Option<(i32, i32)> = sqlx::query_as(
    "SELECT lowest_note, highest_note FROM musical_metadata WHERE file_id = $1"
)
.bind(file_record.id)
.fetch_optional(db_pool)
.await?;

if let Some((low, high)) = note_range {
    // Bass range: C1 (36) to E3 (52)
    if low >= 36 && high <= 52 {
        return Ok(MPCCategory::Bass);
    }

    // Drum range: C1 (36) to F5 (81) with narrow span
    if low >= 35 && high <= 81 && (high - low) < 25 {
        return Ok(MPCCategory::Drums);
    }

    // High melody range: C5 (72) and above
    if low >= 60 && high >= 72 {
        return Ok(MPCCategory::Melody);
    }

    // Wide range suggests chords/progression (3+ octaves)
    if (high - low) > 36 {
        return Ok(MPCCategory::Chords);
    }
}
```

**MIDI Note Ranges:**
- Bass: 36-52 (C1 to E3)
- Drums: 35-81 with span < 25 notes
- Melody: 60+ starting, 72+ peak
- Chords: >36 note span (3 octaves)

### Strategy 3 & 4: Folder/Filename Fallback

Same as before, but now only used if tags and note range don't match.

### Performance Optimizations

1. **Async I/O:** `tokio::fs::copy()` for non-blocking file operations
2. **Buffered I/O:** Tokio uses internal buffering (64KB default)
3. **Worker Pool:** 8 parallel export workers (configurable)
4. **Lazy Directory Creation:** `create_dir_all()` only once per category

**Suggested Additional Crates (for Phase 6 speed):**
- ✅ Already using: `tokio::fs` (async I/O)
- ❌ Not needed: `rayon` (workers already parallel via tokio)
- ✅ Already available: `memmap2` (zero-copy I/O if needed)
- ⚠️ Consider: DMA-based I/O for NVMe SSDs (Linux `io_uring` via `tokio-uring`)

---

## Filepath Metadata Extraction

### Overview

**Question:** "During the metadata extraction phase, if the file is missing metadata, how can we utilize the file's complete filepath (all the folders it's inside) to get the metadata?"

**Answer:** We already have this! The `filename_metadata.rs` module in Phase 4 (Analyze) extracts metadata from:
1. Filename patterns (BPM, key, etc.)
2. Parent folder names
3. Full directory path

### Existing Implementation

**Location:** `pipeline/src-tauri/src/core/analysis/filename_metadata.rs`

**Fallback Strategy (Auto-Tagger Integration):**

```rust
// In analyze worker, if MIDI analysis fails to detect BPM/key:
use crate::core::analysis::FilenameMetadata;

// Extract metadata from filepath
let filepath_metadata = FilenameMetadata::from_path(&file_record.filepath)?;

// Use as fallback
let bpm = if bpm_result.confidence > 0.5 {
    Some(bpm_result.bpm)
} else if let Some(fm_bpm) = filepath_metadata.bpm {
    Some(fm_bpm)  // Use filename BPM as fallback
} else {
    None
};

let key = if key_result.confidence > 0.5 {
    Some(key_result.key_signature.to_string())
} else if let Some(fm_key) = filepath_metadata.key {
    Some(fm_key)  // Use filename key as fallback
} else {
    None
};
```

### FilenameMetadata Patterns

**BPM Extraction:**
```
/Samples/90BPM/Drums/kick.mid         → BPM: 90
/Beats/128-bpm-loop.mid               → BPM: 128
/House/120_Cmaj_bass.mid              → BPM: 120
```

**Key Extraction:**
```
/Melodies/Cmaj/melody.mid             → Key: C major
/Chords/A-minor-progression.mid       → Key: A minor
/120bpm_Dmin_bass.mid                 → Key: D minor
```

**Time Signature Extraction:**
```
/Beats/4-4/groove.mid                 → Time: 4/4
/Jazz/7-8-feel/comp.mid               → Time: 7/8
```

**Pattern Recognition:**
- BPM: `90bpm`, `120-bpm`, `128_bpm`, `/90BPM/`
- Key: `Cmaj`, `C-major`, `Amin`, `A-minor`, `/Cmajor/`
- Time Sig: `4-4`, `7-8`, `6/8`

### Integration with Analyze Worker

**Current Implementation (needs enhancement):**

```rust
// In workers/analyze.rs

async fn analyze_file(
    file_record: &FileRecord,
    db_pool: &PgPool,
) -> Result<(), PipelineError> {
    // 1. Parse MIDI
    let bytes = tokio::fs::read(&file_record.filepath).await?;
    let midi = parse_midi_file(&bytes)?;

    // 2. MIDI analysis (primary method)
    let bpm_result = detect_bpm(&midi);
    let key_result = detect_key(&midi);

    // 3. Filepath metadata extraction (fallback)
    let filepath_metadata = FilenameMetadata::from_path(&file_record.filepath)
        .unwrap_or_default();

    // 4. Use fallback if MIDI analysis has low confidence
    let bpm = if bpm_result.confidence > 0.5 {
        Some(bpm_result.bpm)
    } else if let Some(fm_bpm) = filepath_metadata.bpm {
        Some(fm_bpm)
    } else {
        None
    };

    let key_signature = if key_result.confidence > 0.5 {
        Some(key_result.key_signature.to_string())
    } else if let Some(fm_key) = filepath_metadata.key {
        Some(fm_key)
    } else {
        None
    };

    // 5. Insert to database
    sqlx::query("INSERT INTO musical_metadata (...) VALUES (...)")
        .bind(bpm)
        .bind(key_signature)
        .execute(db_pool)
        .await?;

    Ok(())
}
```

### Database Schema Support

The `musical_metadata` table already supports all extracted metadata:

```sql
CREATE TABLE musical_metadata (
    file_id BIGINT PRIMARY KEY REFERENCES files(id),
    tempo_bpm NUMERIC(6,2),              -- From MIDI or filename
    key_signature key_signature_enum,    -- From MIDI or filename
    time_signature_numerator INT,        -- From MIDI or filename
    time_signature_denominator INT,
    duration_seconds NUMERIC(10,3),
    lowest_note INT,
    highest_note INT,
    total_notes INT
);
```

---

## Implementation Status

### ✅ Completed

1. **Rename Worker:**
   - ✅ Bar length calculation function
   - ✅ Strict character sanitization
   - ✅ Enhanced filename format
   - ✅ Database query for extended metadata
   - ✅ Filename assembly logic

2. **Export Worker:**
   - ✅ Multi-strategy categorization
   - ✅ Auto-tag detection
   - ✅ MIDI note range analysis
   - ✅ 13 MPC/Force categories
   - ✅ Optimized async I/O

### ⚠️ In Progress

1. **Compilation Fixes:**
   - ❌ Import worker: `parse_midi_file` scope issue
   - ❌ Import worker: `calculate_file_hash` return type mismatch
   - ❌ Split worker: `SplitTrack.track_index` field missing
   - ❌ Analyze worker: BPM/Key result API mismatch
   - ❌ Analyze worker: TimedEvent structure mismatch

2. **Filepath Metadata Integration:**
   - ⏳ Add FilenameMetadata fallback to analyze worker
   - ⏳ Test with files missing MIDI metadata
   - ⏳ Validate extraction from deep directory structures

### ⏭️ Next Steps

1. **Fix All Compilation Errors** (30 mins)
   - Import correct types from shared library
   - Match BPM/Key detector APIs
   - Fix MIDI structure field access

2. **Test Pipeline** (1 hour)
   - Small dataset (10 files)
   - Full dataset (1000 files)
   - Verify renamed filenames
   - Verify MPC/Force export structure

3. **Performance Profiling** (30 mins)
   - Measure throughput per stage
   - Identify bottlenecks
   - Tune worker counts

---

## Usage Examples

### Basic Pipeline (No Rename, No Export)

```bash
pipeline-orchestrator --source /path/to/midi
```

### With Rename (Bar-Length Filenames)

```bash
pipeline-orchestrator --source /path/to/midi \
  --enable-rename
```

### With Export to Akai Force SSD

```bash
pipeline-orchestrator --source /path/to/midi \
  --export-to /media/FORCE_SSD \
  --export-format akai-force
```

### Full Pipeline (Rename + Export)

```bash
pipeline-orchestrator --source /path/to/midi \
  --enable-rename \
  --export-to /media/MPC_SSD \
  --export-format mpc-one
```

### Custom Worker Counts

```bash
pipeline-orchestrator --source /path/to/midi \
  --enable-rename \
  --export-to /media/MPC_SSD \
  --workers 32,64,32,48,64,16  # import,sanitize,split,analyze,rename,export
```

---

## Performance Targets

**With Enhancements:**
- Phase 5 Rename: +5% overhead (metadata queries + bar calculation)
- Phase 6 Export: 5,000-8,000 files/sec (limited by disk I/O)
- Overall pipeline: Still ~1.3 hours for 4.3M files

**Export Bottleneck:** Physical file copy (I/O bound)
- SSD → SSD: ~8,000 files/sec
- HDD → SSD: ~3,000 files/sec
- Network → SSD: ~1,000 files/sec

---

## Workflow Benefits

### For Producers

**Before (Manual Organization):**
```
/Downloads/random_midi_files/
├── file1.mid
├── untitled_2.mid
├── 128bpm-bass.mid
└── melody.mid
```

**After (Automated Pipeline):**

**MPC One SSD:**
```
/MPC_Documents/
├── SAMPLES/
│   ├── Bass/
│   │   └── 32-090bpm_Cmin_Bass-808_sub.mid
│   ├── Drums/
│   │   ├── Kicks/
│   │   │   └── 16-128bpm_Cmaj_Kicks-bd_tight.mid
│   │   └── Snares/
│   │       └── 8-90bpm_Amin_Snares-sd_acoustic.mid
│   ├── Melody/
│   │   └── 64-120bpm_Fmaj_Leads-melody_main.mid
│   └── Chords/
│       └── 32-90bpm_Cmin_Chords-progression_intro.mid
└── Progressions/
    └── 128-100bpm_Gmaj_Progressions-full_mix.mid
```

**Every filename tells you:**
- Bar length (for loop alignment)
- BPM (for tempo matching)
- Key (for harmonic mixing)
- Category (for quick browsing)
- Original context (folder + name)

---

**Status:** ⚠️ Compilation fixes in progress
**ETA:** 30-60 minutes to working pipeline
**Document Version:** 1.0
**Created:** November 18, 2025

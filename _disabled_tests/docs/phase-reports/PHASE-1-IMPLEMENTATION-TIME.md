# Phase 1 Implementation Time Estimate

**Feature**: Harmonic Analysis (Chord Progressions, Types, Complexity)

---

## Development Time Breakdown

### 1. Add Dependency (1 minute)
```toml
# pipeline/src-tauri/Cargo.toml
[dependencies]
rust-music-theory = "0.3.0"
```

### 2. Create Database Migration (10 minutes)
```sql
-- database/migrations/010_harmonic_analysis.sql
ALTER TABLE musical_metadata ADD COLUMN
    chord_progression JSONB,
    chord_types TEXT[],
    has_seventh_chords BOOLEAN DEFAULT FALSE,
    has_extended_chords BOOLEAN DEFAULT FALSE,
    chord_change_rate NUMERIC(5,2),
    chord_complexity_score NUMERIC(4,3);

CREATE INDEX idx_has_seventh ON musical_metadata(has_seventh_chords)
    WHERE has_seventh_chords = true;
CREATE INDEX idx_chord_progression ON musical_metadata
    USING GIN(chord_progression);
```

### 3. Write Chord Detection Function (45-60 minutes)
```rust
// pipeline/src-tauri/src/core/analysis/chord_analyzer.rs
use rust_music_theory::note::{Note, PitchClass};
use rust_music_theory::chord::{Chord, Quality};

pub struct ChordAnalysis {
    pub progression: Vec<String>,
    pub types: Vec<String>,
    pub has_sevenths: bool,
    pub has_extended: bool,
    pub change_rate: f32,
    pub complexity_score: f32,
}

pub fn analyze_chords(midi_file: &MidiFile) -> ChordAnalysis {
    // 1. Extract notes at each time window (e.g., 1 beat)
    // 2. Group simultaneous notes (within 50ms)
    // 3. Identify chord from note collection
    // 4. Track progression over time
    // 5. Calculate statistics
}
```
**Complexity**: Medium - Need to:
- Parse MIDI events into time windows
- Group simultaneous notes
- Match note collections to chord types
- Handle edge cases (incomplete chords, inversions)

### 4. Update AnalyzedFile Struct (5 minutes)
```rust
// pipeline/src-tauri/src/bin/analyze.rs
struct AnalyzedFile {
    // ... existing fields ...

    // NEW: Harmonic analysis
    chord_progression: Option<serde_json::Value>,
    chord_types: Vec<String>,
    has_seventh_chords: bool,
    has_extended_chords: bool,
    chord_change_rate: Option<f32>,
    chord_complexity_score: Option<f32>,
}
```

### 5. Update Database INSERT (10 minutes)
```rust
// Add to INSERT statement
INSERT INTO musical_metadata (
    ...,
    chord_progression,
    chord_types,
    has_seventh_chords,
    has_extended_chords,
    chord_change_rate,
    chord_complexity_score
) VALUES (..., $24, $25, $26, $27, $28, $29)
```

### 6. Call Chord Analyzer (5 minutes)
```rust
// In analyze_single_file()
let chord_analysis = analyze_chords(&midi_file);
```

### 7. Build & Test (10 minutes)
```bash
cargo build --release --bin analyze
# Test with sample file
export DATABASE_URL="postgresql://..."
./target/release/analyze
```

### 8. Verify Results (5 minutes)
```sql
SELECT
    filename,
    chord_progression,
    chord_types,
    has_seventh_chords
FROM files f
JOIN musical_metadata mm ON f.id = mm.file_id
LIMIT 5;
```

---

## Total Development Time

| Task | Time |
|------|------|
| Add dependency | 1 min |
| Create migration | 10 min |
| **Write chord detection** | **45-60 min** |
| Update struct | 5 min |
| Update INSERT | 10 min |
| Integration | 5 min |
| Build & test | 10 min |
| Verify | 5 min |
| **TOTAL** | **90-110 minutes** |

**Realistic estimate: 1.5 - 2 hours**

---

## Processing Time (No Additional Cost!)

**Important**: This runs during the existing Phase 2 (analysis) step.

- Current analysis: 20-36 hours for 5.8M files
- With chord analysis: 20-36 hours for 5.8M files (same!)

**Why no additional time?**
- Chord detection adds ~10-20% processing overhead
- But still processes 40-80 files/sec (well within target)
- Parallel processing absorbs the extra work

---

## Deployment Options

### Option A: Backfill Existing Files (Recommended)
```bash
# Clear analyzed_at to re-analyze all files
psql midi_library -c "UPDATE files SET analyzed_at = NULL;"

# Re-run analysis with new chord detection
./target/release/analyze
```
**Time**: Same 20-36 hours (overnight/weekend job)
**Benefit**: All 5.8M files get chord data

### Option B: New Files Only
```bash
# Just run normally going forward
./scripts/import_and_analyze.sh /new/midi/files/
```
**Time**: Immediate (no backfill)
**Benefit**: New files get chord data, old files slowly backfilled

### Option C: Incremental Backfill
```bash
# Analyze 100K files per day
./target/release/analyze --limit 100000
```
**Time**: 5-10 minutes per 100K files
**Benefit**: Gradual backfill, no long batch job

---

## Return on Investment

**Development Investment**: 1.5-2 hours
**Processing Investment**: 0 hours (same as current)
**Storage Investment**: ~500 MB (for 5.8M files)

**Immediate Benefits**:
```sql
-- Find house loops with I-IV-V-I progression
SELECT * FROM files f
JOIN musical_metadata mm ON f.id = mm.file_id
WHERE 'house' = ANY(f.filename_genres)
  AND mm.chord_progression @> '["C", "F", "G", "C"]'::jsonb;

-- Find files with jazz seventh chords
SELECT * FROM musical_metadata
WHERE has_seventh_chords = true
ORDER BY chord_complexity_score DESC
LIMIT 100;

-- Find compatible chord progressions
SELECT f1.filename, f2.filename
FROM musical_metadata mm1
JOIN musical_metadata mm2 ON mm1.chord_progression = mm2.chord_progression
JOIN files f1 ON mm1.file_id = f1.id
JOIN files f2 ON mm2.file_id = f2.id
WHERE mm1.file_id < mm2.file_id;
```

---

## Recommended Timeline

### Today (2 hours):
1. ✅ Add dependency (1 min)
2. ✅ Create migration (10 min)
3. ✅ Implement chord detection (60 min)
4. ✅ Integrate & test (30 min)
5. ✅ Verify with sample files (10 min)

### Tonight/Weekend (optional):
6. ⏳ Backfill existing files (20-36 hours automated)

### Next Week:
7. 🎉 Query with advanced harmonic searches!

---

## Simplified Implementation (30 minutes)

If you want **even faster**, here's a minimal version:

**Just detect chord types, skip progressions**:
```rust
// Simplified: Only identify if chords exist
pub fn has_chords_simple(midi_file: &MidiFile) -> bool {
    // Find any 3+ simultaneous notes
    // Much faster, still useful
}
```

**Columns**:
```sql
ALTER TABLE musical_metadata ADD COLUMN
    has_seventh_chords BOOLEAN,
    has_extended_chords BOOLEAN,
    avg_chord_notes NUMERIC(3,1);  -- 3.0 = triads, 4.0 = sevenths
```

**Time**: 30 minutes total
**Benefit**: 70% of the value, 25% of the work

---

## My Recommendation

**Implement the full version (1.5-2 hours)** because:

1. The extra 60-90 minutes gets you:
   - Full chord progressions (game-changer for search)
   - Progression matching (find compatible files)
   - Musical analysis (chord change rate, complexity)

2. Processing time is the same anyway (no cost)

3. You only implement once, query forever

4. The searches enabled are 10x more powerful:
   - "Find ii-V-I jazz progressions"
   - "Show files with same chord structure"
   - "Find harmonically compatible bass lines"

**Start implementing?** I can have the basic structure working in 30 minutes, with full implementation in 2 hours total.

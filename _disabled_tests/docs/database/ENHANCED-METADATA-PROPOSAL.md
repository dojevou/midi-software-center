# Enhanced Metadata Extraction Proposal

**Purpose**: Maximize search capabilities and algorithmic recommendations
**Approach**: Extract during import/analysis (one-time cost, infinite benefit)
**Target**: 5.8M MIDI files

---

## Priority 1: Harmonic Analysis (Highest Impact)

### **Chord Progressions & Types**
**Value**: Essential for finding "similar feel" files, key-compatible loops
**Search Examples**:
- "Find all house loops with I-V-vi-IV progression"
- "Find bass lines compatible with Cm-Fm-Gm-Cm"
- "Show me files using jazz chords (7ths, 9ths, 11ths)"

**Extract**:
```rust
// Using rust-music-theory crate
chord_progression: Vec<String>  // ["Cm", "Fm", "Gm", "Cm"]
chord_types: Vec<String>        // ["minor", "minor", "minor", "minor"]
chord_complexity_types: JSONB   // {"triads": 3, "seventh": 1, "extended": 0}
dominant_chords: i16            // Count of dominant 7th chords
has_jazz_chords: bool           // Contains 7ths/9ths/11ths/13ths
chord_change_rate: f32          // Chords per measure
```

**Database Schema**:
```sql
ALTER TABLE musical_metadata ADD COLUMN
    chord_progression JSONB,
    chord_types TEXT[],
    chord_complexity_distribution JSONB,
    has_seventh_chords BOOLEAN DEFAULT FALSE,
    has_extended_chords BOOLEAN DEFAULT FALSE,
    chord_change_rate NUMERIC(5,2);

-- Index for common searches
CREATE INDEX idx_has_seventh ON musical_metadata(has_seventh_chords) WHERE has_seventh_chords = true;
CREATE INDEX idx_chord_progression ON musical_metadata USING GIN(chord_progression);
```

**Rust Crate**: `rust-music-theory = "0.3.0"`

---

## Priority 2: Rhythmic Patterns (High Impact)

### **Note Duration Analysis & Groove Detection**
**Value**: Find rhythmically compatible loops, identify genre grooves
**Search Examples**:
- "Find swing eighth note patterns at 140 BPM"
- "Show me straight sixteenth note grooves"
- "Find syncopated patterns for funk"

**Extract**:
```rust
// Note duration distribution
note_durations: JSONB {
    "whole": 0,
    "half": 2,
    "quarter": 45,
    "eighth": 128,
    "sixteenth": 256,
    "triplet": 64,
    "thirty_second": 12
}

// Groove characteristics
groove_type: String             // "swing", "straight", "shuffle", "triplet", "dotted"
swing_amount: f32               // 0.0-1.0 (0.5 = straight, 0.66 = swing)
syncopation_score: f32          // 0.0-1.0
rhythmic_complexity: f32        // Entropy-based measure
has_triplets: bool
has_dotted_rhythms: bool
grid_quantization: String       // "1/4", "1/8", "1/16", "1/32"
```

**Database Schema**:
```sql
ALTER TABLE musical_metadata ADD COLUMN
    note_duration_distribution JSONB,
    groove_type TEXT,
    swing_amount NUMERIC(4,3) CHECK (swing_amount BETWEEN 0 AND 1),
    syncopation_score NUMERIC(4,3) CHECK (syncopation_score BETWEEN 0 AND 1),
    rhythmic_complexity NUMERIC(5,3),
    grid_quantization TEXT,
    has_triplets BOOLEAN DEFAULT FALSE,
    has_dotted_rhythms BOOLEAN DEFAULT FALSE;

CREATE INDEX idx_groove_type ON musical_metadata(groove_type);
CREATE INDEX idx_triplets ON musical_metadata(has_triplets) WHERE has_triplets = true;
```

**Implementation**: Custom analysis (calculate from MIDI tick positions)

---

## Priority 3: Melodic Characteristics (High Impact)

### **Melodic Contour & Interval Analysis**
**Value**: Find melodically similar files, identify "catchy" melodies
**Search Examples**:
- "Find ascending melodies in pentatonic scale"
- "Show me melodies with mostly stepwise motion"
- "Find hook-like patterns (repeated short phrases)"

**Extract**:
```rust
melodic_contour: String         // "ascending", "descending", "arch", "valley", "static"
melodic_motion: JSONB {
    "steps": 85,                // 2nd intervals
    "leaps": 15,                // 3rd+ intervals
    "avg_interval": 2.3         // Average semitones
}
phrase_length_avg: f32          // Average phrase length in measures
melodic_repetition_score: f32   // 0.0-1.0 (higher = more repetitive)
scale_type: String              // "major", "minor", "pentatonic", "blues", "chromatic", "modal"
has_sequences: bool             // Repeated melodic patterns
melodic_range_usage: f32        // % of available range used
```

**Database Schema**:
```sql
ALTER TABLE musical_metadata ADD COLUMN
    melodic_contour TEXT,
    melodic_motion JSONB,
    phrase_length_avg NUMERIC(5,2),
    melodic_repetition_score NUMERIC(4,3),
    scale_type TEXT,
    has_sequences BOOLEAN DEFAULT FALSE,
    melodic_range_usage NUMERIC(4,3);

CREATE INDEX idx_scale_type ON musical_metadata(scale_type);
CREATE INDEX idx_contour ON musical_metadata(melodic_contour);
```

**Rust Crate**: `rust-music-theory` for scale detection

---

## Priority 4: Control Change Analysis (Medium-High Impact)

### **Expression & Modulation**
**Value**: Identify expressive vs mechanical files, filter by production quality
**Search Examples**:
- "Find files with natural velocity variation"
- "Show me loops with modulation wheel automation"
- "Find mechanical/quantized patterns (drum machines)"

**Extract**:
```rust
// Velocity analysis
velocity_variation: f32         // Standard deviation
velocity_distribution: JSONB    // Histogram [0-31: x, 32-63: y, 64-95: z, 96-127: w]
has_velocity_automation: bool   // Gradual changes vs static

// Control changes
cc_usage: JSONB {
    "modulation_wheel": true,   // CC1
    "expression": true,         // CC11
    "sustain_pedal": true,      // CC64
    "reverb": false,            // CC91
    "chorus": false,            // CC93
    "pan": true                 // CC10
}
has_pitch_bend: bool
pitch_bend_range: i16           // Semitones (typically ±2 or ±12)
has_aftertouch: bool
automation_density: f32         // CC messages per second
```

**Database Schema**:
```sql
ALTER TABLE musical_metadata ADD COLUMN
    velocity_variation NUMERIC(5,2),
    velocity_distribution JSONB,
    has_velocity_automation BOOLEAN DEFAULT FALSE,
    cc_usage JSONB,
    has_pitch_bend BOOLEAN DEFAULT FALSE,
    pitch_bend_range SMALLINT,
    has_aftertouch BOOLEAN DEFAULT FALSE,
    automation_density NUMERIC(6,3);

CREATE INDEX idx_cc_usage ON musical_metadata USING GIN(cc_usage);
CREATE INDEX idx_pitch_bend ON musical_metadata(has_pitch_bend) WHERE has_pitch_bend = true;
```

**Implementation**: Parse MIDI CC and pitch bend events

---

## Priority 5: Structural Analysis (Medium Impact)

### **Form & Section Detection**
**Value**: Identify song structure, find intro/outro segments
**Search Examples**:
- "Find 8-bar intro patterns"
- "Show me files with A-B-A structure"
- "Find breakdown sections"

**Extract**:
```rust
song_structure: Vec<String>     // ["intro", "verse", "chorus", "verse", "chorus", "outro"]
section_lengths: Vec<i16>       // [8, 16, 8, 16, 8, 4] (measures)
has_intro: bool
has_outro: bool
has_breakdown: bool
has_buildup: bool
structure_repetition: f32       // How much structure repeats (0.0-1.0)
total_sections: i16
```

**Database Schema**:
```sql
ALTER TABLE musical_metadata ADD COLUMN
    song_structure TEXT[],
    section_lengths SMALLINT[],
    has_intro BOOLEAN DEFAULT FALSE,
    has_outro BOOLEAN DEFAULT FALSE,
    has_breakdown BOOLEAN DEFAULT FALSE,
    has_buildup BOOLEAN DEFAULT FALSE,
    structure_repetition NUMERIC(4,3),
    total_sections SMALLINT;

CREATE INDEX idx_has_intro ON musical_metadata(has_intro) WHERE has_intro = true;
CREATE INDEX idx_structure ON musical_metadata USING GIN(song_structure);
```

**Implementation**: Detect density/velocity changes, pattern repetition

---

## Priority 6: Instrumentation Deep Dive (Medium Impact)

### **MIDI Program Changes & Layering**
**Value**: Search by instrument type, identify production style
**Search Examples**:
- "Find bass lines using finger bass (GM 33)"
- "Show me files with pad layering"
- "Find files mixing acoustic and electronic drums"

**Extract**:
```rust
instruments_used: JSONB {
    "piano": {"program": 0, "notes": 45, "channels": [0]},
    "bass": {"program": 33, "notes": 128, "channels": [1]},
    "drums": {"program": null, "notes": 256, "channels": [9]}
}
instrument_categories: TEXT[]   // ["keyboard", "bass", "drums", "pad"]
has_acoustic_instruments: bool
has_electronic_instruments: bool
instrument_count: i16
max_concurrent_instruments: i16 // Layering depth
channel_count: i16
```

**Database Schema**:
```sql
ALTER TABLE musical_metadata ADD COLUMN
    instruments_detailed JSONB,
    instrument_categories TEXT[],
    has_acoustic BOOLEAN DEFAULT FALSE,
    has_electronic BOOLEAN DEFAULT FALSE,
    instrument_count SMALLINT,
    max_concurrent_instruments SMALLINT,
    channel_count SMALLINT;

CREATE INDEX idx_instruments ON musical_metadata USING GIN(instrument_categories);
CREATE INDEX idx_channels ON musical_metadata(channel_count);
```

**Implementation**: Parse program change events, categorize by GM standard

---

## Priority 7: Similarity Hashing (High Impact for Recommendations)

### **Fingerprints for "Find Similar"**
**Value**: Instant similarity search, duplicate detection, recommendation engine
**Search Examples**:
- "Find files similar to this bassline"
- "Show me rhythmically similar patterns"
- "Detect near-duplicates"

**Extract**:
```rust
// Generate multiple hash types
rhythm_hash: String             // 64-char hex (based on note timing pattern)
pitch_hash: String              // 64-char hex (based on pitch sequence)
harmony_hash: String            // 64-char hex (based on chord progression)
similarity_embedding: Vec<f32>  // 128-dimensional vector for pgvector

// Use for hamming distance searches
// Difference < 5 = very similar, < 10 = similar, < 20 = somewhat similar
```

**Database Schema**:
```sql
ALTER TABLE musical_metadata ADD COLUMN
    rhythm_hash CHAR(64),
    pitch_hash CHAR(64),
    harmony_hash CHAR(64),
    similarity_embedding vector(128);  -- pgvector extension

-- Indexes for fast similarity search
CREATE INDEX idx_rhythm_hash ON musical_metadata(rhythm_hash);
CREATE INDEX idx_pitch_hash ON musical_metadata(pitch_hash);
CREATE INDEX idx_embedding ON musical_metadata USING ivfflat(similarity_embedding vector_cosine_ops);
```

**Rust Crate**: Custom hashing + `pgvector` integration

---

## Priority 8: Energy & Dynamics (Medium Impact)

### **Energy Curve & Density Over Time**
**Value**: Find energetic vs calm sections, build/drop detection
**Search Examples**:
- "Find high-energy drum patterns"
- "Show me gradual build-up sections"
- "Find files with dynamic contrast"

**Extract**:
```rust
energy_profile: JSONB {
    "avg": 0.68,
    "min": 0.12,
    "max": 0.95,
    "variance": 0.23,
    "curve": [0.2, 0.3, 0.5, 0.7, 0.9, 0.8, 0.6]  // Sampled at intervals
}
dynamic_range_db: f32           // Velocity range as dB equivalent
has_crescendo: bool
has_diminuendo: bool
note_density_curve: Vec<f32>    // Notes per second over time
spectral_centroid_avg: f32      // Average "brightness" (requires note->freq)
```

**Database Schema**:
```sql
ALTER TABLE musical_metadata ADD COLUMN
    energy_profile JSONB,
    dynamic_range_db NUMERIC(5,2),
    has_crescendo BOOLEAN DEFAULT FALSE,
    has_diminuendo BOOLEAN DEFAULT FALSE,
    note_density_curve JSONB,
    spectral_centroid_avg NUMERIC(6,2);

CREATE INDEX idx_energy ON musical_metadata((energy_profile->>'avg')) WHERE energy_profile IS NOT NULL;
```

**Implementation**: Calculate from velocity and note density over time

---

## Implementation Strategy

### Phase 1: Add Columns (Quick - 5 minutes)
```bash
# Run migration to add all columns
psql midi_library -f database/migrations/010_enhanced_metadata.sql
```

### Phase 2: Update Analyzer (Medium - 2-4 hours dev)
```rust
// Add to Cargo.toml
[dependencies]
rust-music-theory = "0.3.0"
chord_detector = "0.1.0"

// Update analyze.rs to extract enhanced metadata
// Most can be calculated from existing MIDI events
```

### Phase 3: Backfill Existing Files (Automatic)
```bash
# Re-run analysis on existing files
./target/release/analyze
```

### Phase 4: Update Search API (Medium - 2-3 hours)
```rust
// Add new query filters to search endpoints
// Example: ?has_seventh_chords=true&groove_type=swing&energy_min=0.7
```

---

## Estimated Impact

### Search Query Improvements:
| Feature | Current Capability | Enhanced Capability |
|---------|-------------------|---------------------|
| BPM/Key | ✅ Basic filter | ✅ + Chord progressions + Scale types |
| Rhythm | ❌ None | ✅ Swing, syncopation, groove type |
| Melody | ❌ None | ✅ Contour, intervals, scales |
| Similarity | ❌ Manual | ✅ Automatic "find similar" |
| Energy | ❌ None | ✅ Build/drop detection |
| Instruments | ❌ Basic | ✅ Deep layering analysis |

### Algorithmic Recommendations:
1. **"Complete this progression"** - Find files with compatible chord progressions
2. **"Find similar groove"** - Rhythm hash matching
3. **"Complementary melody"** - Harmonic compatibility checking
4. **"Build energy"** - Find files with ascending energy curves
5. **"Genre-appropriate"** - Multi-dimensional feature matching

### Database Size Impact:
- Current: ~7 GB for 5.8M files (57 fields)
- Enhanced: ~10-12 GB for 5.8M files (85+ fields)
- Additional JSONB indexes: ~2-3 GB
- pgvector embeddings: ~3 GB

**Total**: ~15-18 GB (still very manageable)

---

## Recommended Rust Crates

```toml
[dependencies]
# Already using
midly = "0.5"
sqlx = "0.7"

# Add for enhanced analysis
rust-music-theory = "0.3.0"     # Scales, chords, intervals
chord_detector = "0.1.0"        # Chromagram and chord detection

# For embedding generation (optional - Phase 2)
ndarray = "0.15"                # Vector operations
rust-bert = "0.21"              # If doing ML embeddings (heavy)
```

---

## Return on Investment

**One-time cost**:
- Development: ~8-12 hours
- Processing: 20-36 hours (same as current analysis)
- Storage: +8-11 GB

**Ongoing benefit**:
- 10x more powerful search queries
- Automatic recommendations
- Duplicate detection
- Genre/mood classification
- Production quality filtering
- Compatibility matching

**Recommendation**: Implement Priority 1-4 immediately, Priority 5-8 in Phase 2

---

## Example Search Queries (After Enhancement)

```sql
-- Find swing eighth-note basslines in Cm with jazz chords
SELECT * FROM files f
JOIN musical_metadata mm ON f.id = mm.file_id
WHERE f.filename_key = 'Cm'
  AND mm.groove_type = 'swing'
  AND mm.has_seventh_chords = true
  AND 'bass' = ANY(mm.instrument_categories)
  AND mm.note_duration_distribution->>'eighth' > '50';

-- Find similar rhythmic patterns (hamming distance)
SELECT f.*,
       bit_count(rhythm_hash::bit(64) # '0123456789ABCDEF'::bit(64)) as distance
FROM files f
JOIN musical_metadata mm ON f.id = mm.file_id
ORDER BY distance
LIMIT 10;

-- Find files for building energy (ascending curve)
SELECT * FROM musical_metadata
WHERE (energy_profile->>'variance')::float > 0.2
  AND has_crescendo = true
  AND (energy_profile->'curve'->>6)::float > (energy_profile->'curve'->>0)::float;
```

These queries would be impossible with current metadata!

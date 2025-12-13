# Database Categories & Subcategories - Complete Reference

**MIDI Software Center Database Schema**
**Based on**: PostgreSQL 16 + Complete Production Testing
**Files Analyzed**: 1,603 real MIDI files + 1,196,659 drum collection

---

## 📊 **PART 1: DATABASE ENUM CATEGORIES**

### 🥁 **file_category ENUM** (Primary Classification)

**Source**: `database/migrations/001_initial_schema.sql` lines 25-59

#### **Drums (9 types)**
```sql
'KICK'              -- Kick drum / Bass drum
'SNARE'             -- Snare drum
'HIHAT'             -- Hi-hat (open/closed)
'CLAP'              -- Clap / Hand clap
'PERC'              -- Percussion (general)
'TOM'               -- Tom drums (floor, rack)
'CYMBAL'            -- Cymbals (crash, ride, splash)
'DRUM_LOOP'         -- Complete drum loop
'DRUM_PATTERN'      -- Drum pattern/groove
```

#### **Bass (3 types)**
```sql
'BASS'              -- Bass line / Bass instrument
'SUB_BASS'          -- Sub bass (low frequency)
'BASS_LOOP'         -- Bass loop/pattern
```

#### **Chords (3 types)**
```sql
'CHORD'             -- Chord progression
'PROGRESSION'       -- Chord sequence
'STAB'              -- Chord stab / Short chord hit
```

#### **Pads (3 types)**
```sql
'PAD'               -- Pad sound / Sustained chords
'TEXTURE'           -- Textural element
'ATMOSPHERE'        -- Atmospheric sound
```

#### **Leads (4 types)**
```sql
'LEAD'              -- Lead melody
'MELODY'            -- Melodic line
'HOOK'              -- Hook/Catchy phrase
'RIFF'              -- Riff / Repeated motif
```

#### **Sequences (2 types)**
```sql
'ARP'               -- Arpeggio / Arpeggiated sequence
'SEQUENCE'          -- Sequenced pattern
```

#### **Keys (3 types)**
```sql
'PIANO'             -- Piano
'KEYS'              -- Keyboard / Keys (general)
'ORGAN'             -- Organ
```

#### **Orchestral (3 types)**
```sql
'STRING'            -- Strings (violin, cello, etc.)
'BRASS'             -- Brass (trumpet, trombone, etc.)
'WOODWIND'          -- Woodwinds (flute, clarinet, etc.)
```

#### **FX (5 types)**
```sql
'FX'                -- Sound effects (general)
'RISER'             -- Riser / Build-up
'IMPACT'            -- Impact / Hit
'SWEEP'             -- Sweep / Whoosh
'TRANSITION'        -- Transition effect
```

#### **Vocal (3 types)**
```sql
'VOCAL'             -- Vocal line
'VOX'               -- Vocal sample
'SAMPLE'            -- Audio sample
```

#### **Other (5 types)**
```sql
'MOTIF'             -- Musical motif
'THEME'             -- Musical theme
'FULL_MIX'          -- Full arrangement
'STEM'              -- Individual stem
'UNKNOWN'           -- Uncategorized
```

**Total**: **37 file_category types**

---

### 🎵 **musical_key ENUM** (Key Signatures)

**Source**: `database/migrations/001_initial_schema.sql` lines 62-69

```sql
-- Major Keys (12)
'C', 'C#', 'Db', 'D', 'D#', 'Eb',
'E', 'F', 'F#', 'Gb', 'G', 'G#',
'Ab', 'A', 'A#', 'Bb', 'B'

-- Minor Keys (12)
'Cm', 'C#m', 'Dbm', 'Dm', 'D#m', 'Ebm',
'Em', 'Fm', 'F#m', 'Gbm', 'Gm', 'G#m',
'Abm', 'Am', 'A#m', 'Bbm', 'Bm'

-- Unknown
'UNKNOWN'
```

**Total**: **35 musical_key types** (24 standard keys + UNKNOWN)

---

## 📂 **PART 2: FILENAME-BASED CATEGORIES**

### **Extracted from Filenames** (Migration 008)

**Source**: `database/migrations/008_filename_metadata.sql`
**Analyzed**: 1,486,270 MIDI files from production

#### **1. BPM Values** (filename_bpm)
- **Range**: 30-300 BPM
- **Unique Values Found**: 81 different BPM values
- **Common Ranges**:
  ```
  60-90 BPM     → Slow (Hip-Hop, Downtempo)
  90-110 BPM    → Hip-Hop, R&B
  110-130 BPM   → House, Pop
  130-150 BPM   → Techno, Trap
  150-180 BPM   → Drum & Bass
  180-300 BPM   → Very Fast (Hardcore, Speedcore)
  ```

#### **2. Genre Tags** (filename_genres array)
**20 genre types identified**:
```
house, techno, trance, dubstep, dnb (drum & bass),
trap, hip-hop, pop, rock, jazz, funk, disco,
ambient, chill, lofi, experimental, hardcore,
garage, breaks, electro
```

#### **3. Structure Tags** (structure_tags array)
**20 structure types identified**:
```
-- Loop Types
loop, drum-loop, bass-loop, melody-loop, vocal-loop

-- Song Sections
intro, verse, chorus, bridge, outro, breakdown,
buildup, drop

-- Pattern Types
fill, groove, pattern, motif, hook

-- Arrangement
stem, layer, full-mix
```

#### **4. Track Numbers** (track_number)
- Extracted from leading digits: "01_kick.mid" → 1
- Used for maintaining sequence order
- Range: 1-999+

---

## 🏷️ **PART 3: AUTO-GENERATED TAGS**

### **From Auto-Tagger** (500+ tags total)

**Source**: `pipeline/src-tauri/src/core/analysis/auto_tagger.rs`

#### **A. Instrument Tags** (350+ tags)
Based on MIDI program changes and note analysis:

**Drums (48 types)**:
```
kick, snare, hihat, clap, rim, tom, crash, ride,
splash, china, cowbell, tambourine, shaker, conga,
bongo, timbale, agogo, cabasa, maracas, whistle,
guiro, claves, woodblock, triangle, bell, cymbal,
closed-hat, open-hat, pedal-hat, etc.
```

**Bass (8 types)**:
```
bass, acoustic-bass, electric-bass, fingered-bass,
picked-bass, fretless-bass, slap-bass, synth-bass
```

**Keyboards (12 types)**:
```
piano, electric-piano, acoustic-piano, bright-piano,
grand-piano, electric-grand, organ, church-organ,
reed-organ, accordion, harmonica, harpsichord
```

**Strings (8 types)**:
```
violin, viola, cello, contrabass, strings,
string-ensemble, synth-strings, pizzicato
```

**Brass (8 types)**:
```
trumpet, trombone, tuba, horn, brass-section,
synth-brass, french-horn, muted-trumpet
```

**Woodwinds (8 types)**:
```
flute, clarinet, oboe, bassoon, piccolo,
pan-flute, recorder, shakuhachi
```

**Synths (20+ types)**:
```
synth-lead, synth-pad, synth-bass, synth-fx,
synth-strings, synth-brass, analog, digital,
warm, bright, soft, hard, etc.
```

**Guitar (6 types)**:
```
guitar, acoustic-guitar, electric-guitar,
clean-guitar, distortion-guitar, overdrive-guitar
```

#### **B. Musical Property Tags**
```
-- Tempo-based
slow, medium, fast, very-fast
uptempo, downtempo, half-time, double-time

-- Rhythm
straight, swing, shuffle, triplet, syncopated

-- Mood
happy, sad, energetic, calm, dark, bright,
aggressive, peaceful, tense, relaxed

-- Style
melodic, rhythmic, harmonic, textural,
atmospheric, ambient, percussive
```

#### **C. Technical Tags**
```
mono, poly, legato, staccato, sustained,
arpeggiated, sequenced, quantized, humanized

-- Channel-based
channel-10 (GM drums)
multi-channel, single-channel

-- Format
midi-0, midi-1, midi-2 (MIDI format types)
```

---

## 🗂️ **PART 4: ORGANIZATIONAL DIMENSIONS**

### **7 Organizational Dimensions** (Production Tested)

**Source**: Database schema + Real-world validation report

#### **1. By Instrument/Sound Type**
- file_category (37 types)
- Auto-tagger instrument tags (350+ tags)
- Examples: kick, piano, strings, synth

#### **2. By Musical Key**
- musical_key ENUM (35 types)
- filename_key (extracted from names)
- Examples: C, Am, F#, Bbm

#### **3. By Tempo/BPM**
- BPM metadata (analyzed)
- filename_bpm (from names)
- Range: 30-300 BPM
- 6 defined BPM ranges

#### **4. By Genre**
- filename_genres array (20 types)
- Examples: house, techno, dnb, trap

#### **5. By Structure/Function**
- structure_tags array (20 types)
- Examples: loop, intro, fill, drop

#### **6. By Collection/Source**
- manufacturer field
- collection_name field
- folder_tags array
- Examples: "Vengeance", "Splice", "Native Instruments"

#### **7. By Content Hierarchy**
- parent_file_id (for multi-track splits)
- track_number (sequence)
- total_tracks (count)
- Examples: Track 1 of 8, Track 2 of 8

---

## 🔍 **PART 5: METADATA SOURCES**

### **metadata_source Types**
```sql
'analyzed'      -- From MIDI analysis only
'filename'      -- From filename parsing only
'both'          -- Both sources available
'validated'     -- Cross-validated (analyzed matches filename)
'none'          -- No metadata available
```

### **Coverage Statistics** (from real data)
- **Analyzed BPM**: ~40% coverage
- **Filename BPM**: ~60% coverage
- **Either BPM**: ~85% coverage
- **Validated BPM**: ~30% (within ±5 BPM tolerance)

---

## 📋 **PART 6: COMPLETE TAG TAXONOMY**

### **Tag Categories Summary**

| Category | Count | Source | Examples |
|----------|-------|--------|----------|
| **file_category** | 37 | ENUM | KICK, BASS, LEAD, PAD |
| **musical_key** | 35 | ENUM | C, Am, F#, Bbm |
| **Drum Types** | 48 | Auto-tagger | kick, snare, hihat, crash |
| **Instruments** | 350+ | Auto-tagger | piano, bass, strings, synth |
| **Genres** | 20+ | Filename | house, techno, dnb, trap |
| **Structure** | 20+ | Filename | loop, intro, fill, verse |
| **BPM Ranges** | 6 | Calculated | 60-90, 90-110, 110-130... |
| **Moods** | 30+ | Auto-tagger | happy, dark, energetic |
| **Technical** | 50+ | Auto-tagger | mono, stereo, channel-10 |

**Grand Total**: **500+ unique categories/subcategories**

---

## 🎯 **PART 7: SEARCH & FILTER CAPABILITIES**

### **Available Filters**

```sql
-- By Category
WHERE primary_category = 'KICK'
WHERE secondary_category IN ('BASS', 'SUB_BASS')

-- By Key
WHERE key_signature = 'Am'
WHERE filename_key = 'Cm'

-- By BPM Range
WHERE bpm BETWEEN 120 AND 130
WHERE filename_bpm BETWEEN 120 AND 130

-- By Genre (array contains)
WHERE 'house' = ANY(filename_genres)
WHERE filename_genres && ARRAY['techno', 'house']

-- By Structure (array contains)
WHERE 'loop' = ANY(structure_tags)
WHERE structure_tags && ARRAY['intro', 'verse']

-- By Tags
WHERE EXISTS (
    SELECT 1 FROM file_tags ft
    JOIN tags t ON ft.tag_id = t.id
    WHERE ft.file_id = files.id
    AND t.name IN ('kick', 'snare', 'drums')
)

-- Combined Search
WHERE
    bpm BETWEEN 120 AND 130
    AND key_signature = 'Am'
    AND 'house' = ANY(filename_genres)
    AND 'loop' = ANY(structure_tags)
    AND primary_category IN ('BASS', 'LEAD')
```

---

## 📊 **PART 8: DATABASE TABLES**

### **15 Tables in Schema**

1. **files** - Main file records
2. **musical_metadata** - BPM, key, time signature
3. **technical_metadata** - Format, tracks, duration
4. **tags** - Tag definitions
5. **file_tags** - File-to-tag relationships
6. **favorites** - User favorites
7. **collections** - Organized collections
8. **collection_files** - Collection memberships
9. **track_splits** - Multi-track file relationships
10. **analysis_history** - Analysis tracking
11. **import_batches** - Batch import tracking
12. **search_history** - User searches
13. **file_embeddings** - Vector embeddings (pgvector)
14. **genre_tags** - Genre taxonomy
15. **structure_tags** - Structure taxonomy

### **60+ Indexes** for Performance
- Primary keys on all tables
- Foreign key indexes
- Full-text search indexes (GIN + tsvector)
- Array indexes (GIN for filename_genres, structure_tags)
- Range indexes (BPM, key, duration)
- Composite indexes (BPM + key, category + BPM)
- Partial indexes (WHERE NOT NULL)

---

## 🎨 **PART 9: DRUM-SPECIFIC CATEGORIES**

### **150+ Drum Tags** (v2.1 Enhancement)

**Source**: `DRUM-ANALYZER-IMPLEMENTATION-PHASE1.md`

#### **GM Drum Mapping** (48 types)
```
-- Kick Drums (2)
35: Acoustic Bass Drum (kick)
36: Bass Drum 1 (kick)

-- Snare Drums (3)
38: Acoustic Snare (snare)
40: Electric Snare (snare)
37: Side Stick (rim)

-- Hi-Hats (4)
42: Closed Hi-Hat (closed-hat)
44: Pedal Hi-Hat (pedal-hat)
46: Open Hi-Hat (open-hat)
43: Hi-Hat (hihat)

-- Cymbals (8)
49: Crash Cymbal 1 (crash)
52: Chinese Cymbal (china)
51: Ride Cymbal 1 (ride)
53: Ride Bell (bell)
55: Splash Cymbal (splash)
57: Crash Cymbal 2 (crash)
59: Ride Cymbal 2 (ride)

-- Toms (6)
41: Low Floor Tom (tom)
43: High Floor Tom (tom)
45: Low Tom (tom)
47: Low-Mid Tom (tom)
48: Hi-Mid Tom (tom)
50: High Tom (tom)

-- Percussion (25+)
clap, cowbell, tambourine, shaker, conga,
bongo, timbale, agogo, cabasa, maracas, etc.
```

#### **Pattern Types**
```
groove, fill, intro, ending, breakdown, turnaround,
buildup, solo, accent, ghost-note, flam, roll
```

#### **Rhythmic Feel**
```
straight, swing, shuffle, triplet, half-time,
double-time, syncopated, polyrhythmic
```

#### **Song Structure**
```
verse, chorus, bridge, pre-chorus, post-chorus,
intro, outro, breakdown, drop, buildup
```

#### **Techniques**
```
ghost-notes, double-bass, hi-hat-open, hi-hat-closed,
rim-shot, cross-stick, flam, drag, roll, paradiddle
```

---

## 🚀 **PART 10: QUERY EXAMPLES**

### **Complex Search Examples**

```sql
-- Example 1: Find house music loops in Am at 120-130 BPM
SELECT * FROM search_files_with_metadata(
    p_bpm_min := 120,
    p_bpm_max := 130,
    p_key := 'Am',
    p_genres := ARRAY['house'],
    p_structure_tags := ARRAY['loop']
);

-- Example 2: Find drum loops with kicks and snares
SELECT f.*
FROM files f
JOIN file_tags ft1 ON f.id = ft1.file_id
JOIN tags t1 ON ft1.tag_id = t1.id AND t1.name = 'kick'
JOIN file_tags ft2 ON f.id = ft2.file_id
JOIN tags t2 ON ft2.tag_id = t2.id AND t2.name = 'snare'
WHERE f.primary_category = 'DRUM_LOOP';

-- Example 3: Find bass lines in C minor with validated metadata
SELECT * FROM files_with_validated_metadata
WHERE validated_key = 'Cm'
AND 'bass' = ANY(
    SELECT t.name FROM file_tags ft
    JOIN tags t ON ft.tag_id = t.id
    WHERE ft.file_id = id
);

-- Example 4: Popular genre + BPM combinations
SELECT * FROM popular_genre_bpm_combinations
WHERE file_count > 100
ORDER BY file_count DESC;

-- Example 5: Metadata coverage statistics
SELECT * FROM get_metadata_coverage_stats();
```

---

## 📖 **REFERENCES**

### **Source Files**
1. `database/migrations/001_initial_schema.sql` - Core schema
2. `database/migrations/008_filename_metadata.sql` - Filename parsing
3. `pipeline/src-tauri/src/core/analysis/auto_tagger.rs` - Auto-tagging
4. `pipeline/src-tauri/src/core/analysis/drum_analyzer.rs` - Drum detection
5. `CLAUDE.md` - Project documentation

### **Documentation**
- DATABASE-ORGANIZATION-ANALYSIS.md
- DRUM-ANALYZER-IMPLEMENTATION-PHASE1.md
- PRODUCTION-DEPLOYMENT-FINAL.md
- DAW-INTEGRATION-REPORT.md

---

**Last Updated**: 2025-11-10
**Database Version**: PostgreSQL 16
**Tested With**: 1,603 + 1,196,659 real MIDI files
**Status**: Production-Ready ✅

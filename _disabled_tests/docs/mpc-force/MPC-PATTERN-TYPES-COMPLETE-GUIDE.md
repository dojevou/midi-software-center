# MPC/Force Pattern Types - Complete Guide

**Date:** November 22, 2025
**Purpose:** Complete reference for all MPC/Force pattern types and file formats

---

## 🎯 Overview: 4 Pattern Type Categories

The Akai Force/MPC ecosystem supports **4 distinct pattern types**, each with different file formats, storage locations, and use cases:

| Type | File Format | Storage Location | Use Case |
|------|-------------|------------------|----------|
| **1. MIDI Patterns** | `.mpcpattern` | `/Expansions/[PackName]/Patterns/` | General MIDI clips/loops |
| **2. Chord Progressions** | `.mpcprog` | `/Progressions/` (root level) | Pad Perform > Progressions mode |
| **3. Arpeggiator Patterns** | `.mid` | `/Arp Patterns/` | Arpeggiator menu |
| **4. Rhythm Patterns** | `.mid` | `/Arp Patterns/` | Arpeggiator > Rhythm mode |

---

## 1️⃣ MIDI Patterns (.mpcpattern)

### **Description**
Self-contained MIDI clips/loops that can be loaded directly into tracks. These are what most people think of as "patterns" in MPC.

### **File Format**
- **Extension:** `.mpcpattern`
- **Format:** Proprietary JSON-based format
- **Contains:** MIDI events + MPC-specific features (ratchets, probability, 16 levels)
- **Example structure:**
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

### **Storage Locations**

**Standalone (Force/MPC One/Live/X):**
```
/Expansions/[Pack_Name]/Patterns/
```

**MPC Software:**
- Mac: `/Users/[username]/Library/Application Support/Akai/MPC/Expansions/`
- Windows: `C:\ProgramData\Akai\MPC\Expansions\`

### **How to Create**

**Option A: From MPC device**
1. Create a MIDI track
2. Record/edit your pattern
3. Press pencil icon on track
4. Select "Export as Pattern"
5. Save to Expansions folder

**Option B: Convert MIDI → .mpcpattern (Rust tool)**
```bash
# Our ultra-fast parallel converter
./target/release/midi_to_mpcpattern_parallel \
    input.mid output.mpcpattern

# Batch conversion
./target/release/midi_to_mpcpattern_parallel \
    --batch /input/dir /output/dir
```

### **Use Cases**
- ✅ Drum loops and fills
- ✅ Bass lines
- ✅ Melodic sequences
- ✅ Complete arrangements
- ✅ **This is what we're creating for the 97 instrument expansions**

---

## 2️⃣ Chord Progressions (.mpcprog)

### **Description**
Pre-configured chord sequences for use in **Pad Perform > Progressions mode**. Each pad plays a full chord in the progression.

### **File Format**
- **Extension:** `.mpcprog`
- **Format:** Proprietary format (likely JSON-based like .mpcpattern)
- **Structure:** Unknown (not publicly documented)

### **Storage Locations**

**Standalone (Force/MPC):**
```
/Progressions/                    # Root level of SD card/USB/SATA
└── [Category_Name]/              # Optional: organize by category
    ├── progression1.mpcprog
    ├── progression2.mpcprog
    └── ...
```

**System progressions (read-only):**
```
/usr/share/Akai/SME0/Progressions/
```

**MPC Software:**
- Mac: `/Users/[username]/Library/Application Support/Akai/MPC/Progressions/`
- Windows: `C:\ProgramData\Akai\MPC\Progressions\`

### **How to Create**

**Option A: From MPC Software (official method)**
1. Create chord progression as MIDI track
2. Go to **File > Export > As Progression**
3. Save as `.mpcprog` file
4. Copy to `/Progressions/` folder

**Option B: Web-based builder (third-party)**
- URL: https://midi.amitszone.com/FORCE/PBUILDER/
- Click scale/chord degrees to build progression
- Save as `.mpcprog` file
- **Note:** Not a MIDI converter, builds from scratch

**Option C: Extract from expansions**
- Some commercial expansions include `.mpcprog` files
- Copy from expansion's Progressions folder

### **Naming Convention**
```
CategoryName-ProgressionName.mpcprog
```
Examples:
- `Jazz-ii-V-I.mpcprog`
- `Pop-4ChordSong.mpcprog`
- `Rock-PowerChords.mpcprog`

### **Access on Device**
1. Go to **BROWSER > Pad Perform**
2. Select **Progressions** mode
3. Under **"Other"** tab, see custom progressions
4. Select key and octave
5. Play pads to trigger chords in sequence

### **Use Cases**
- ✅ Songwriting/composition
- ✅ Live performance chord triggers
- ✅ Quick harmonic sketching
- ✅ Learning music theory

### **⚠️ Key Limitation**
- **Cannot easily convert MIDI → .mpcprog** (no official tools)
- MPC Software required to create from MIDI
- Web builder only creates from scratch

---

## 3️⃣ Arpeggiator Patterns (.mid)

### **Description**
Custom arpeggio patterns that can be loaded into the MPC's built-in arpeggiator.

### **File Format**
- **Extension:** `.mid` (standard MIDI file)
- **Limitation:** Maximum 128 notes per pattern
- **Format:** Standard MIDI Type 0 or Type 1

### **Storage Locations**

**MPC Software:**
- Mac: `/Library/Application Support/Akai/MPC/Arp Patterns/`
- Windows: `C:\Program Files\Akai Pro\MPC\Arp Patterns\`

**⚠️ Standalone devices:**
- **Not directly supported** - must be created in MPC Software
- Requires MPC in controller mode connected to computer

### **How to Create**

**Method (MPC Software required):**
1. Open MPC 2.0 software with plugin instrument
2. Play/record arpeggio pattern in piano roll
3. Go to **Menu > File > Export as MIDI Track File**
4. Save `.mid` file to desktop
5. Move file to `Arp Patterns` folder
6. Restart MPC Software
7. Pattern appears in Arpeggiator menu

### **Arpeggiator Modes**

The MPC arpeggiator has **4 modes** that use different pattern types:

1. **Arp** - Traditional arpeggiator
   - Patterns: Up, Down, Up/Down, Random, etc.
   - Follows held chord
   - Variables: Pattern, Octave, Variation

2. **Note Repeat** - MPC classic note repeat
   - No patterns, just repeats held note

3. **Rhythm** - Rhythmic pattern from held chord
   - Uses custom rhythm patterns (also `.mid` files)
   - Spread parameter for glissando effect

4. **Pattern** - Melodic phrase transposition
   - Hold single note to trigger melodic pattern
   - Pattern transposes based on held note
   - **Uses custom arp patterns**

### **Use Cases**
- ✅ Custom arpeggiation patterns
- ✅ Melodic phrase triggers
- ✅ Rhythmic variations
- ✅ Performance effects

---

## 4️⃣ Rhythm Patterns (.mid)

### **Description**
Similar to arp patterns but specifically for **Arpeggiator > Rhythm mode**. These create rhythmic patterns from held chords.

### **File Format**
- **Extension:** `.mid` (standard MIDI file)
- **Storage:** Same as Arp Patterns (`/Arp Patterns/`)
- **Usage:** Selected in Arpeggiator > Rhythm mode

### **How They Work**
1. Load rhythm pattern in Arpeggiator menu
2. Select **Rhythm** mode
3. Hold down chord
4. Arpeggiator plays held notes in rhythmic pattern
5. Adjust **Spread** for timing variation

### **Use Cases**
- ✅ Rhythmic chord patterns (e.g., reggae skanks)
- ✅ Syncopated sequences
- ✅ Percussive chord hits
- ✅ Glissando effects (with Spread)

---

## 📊 Comparison Table

| Feature | MIDI Patterns | Progressions | Arp Patterns | Rhythm Patterns |
|---------|--------------|--------------|--------------|-----------------|
| **Format** | `.mpcpattern` | `.mpcprog` | `.mid` | `.mid` |
| **Standalone** | ✅ Yes | ✅ Yes | ❌ Software only | ❌ Software only |
| **MIDI Convert** | ✅ Easy | ❌ Difficult | ✅ Easy | ✅ Easy |
| **Our Tool** | ✅ Yes | ❌ No | ✅ Yes (MIDI) | ✅ Yes (MIDI) |
| **Max Notes** | ♾️ Unlimited | Unknown | 128 | 128 |
| **Location** | Expansions | Root/Progressions | Arp Patterns | Arp Patterns |
| **Access** | Browser | Pad Perform | Arpeggiator | Arpeggiator |

---

## 🎯 Recommendations for Our Project

### **What We Can Do Now** ✅

1. **MIDI Patterns (.mpcpattern)** - **CURRENT FOCUS**
   - ✅ Converter built and tested
   - ✅ 97 instrument expansions planned
   - ✅ 2.2M files ready for conversion
   - ✅ Perfect for our database organization

2. **Arpeggiator Patterns (.mid)** - **EASY TO ADD**
   - ✅ Files are already MIDI (no conversion needed!)
   - ✅ Just identify arpeggio-style patterns
   - ✅ Copy to `/Arp Patterns/` folder
   - Query: Files with ascending/descending note patterns

3. **Rhythm Patterns (.mid)** - **EASY TO ADD**
   - ✅ Files are already MIDI
   - ✅ Identify rhythmic chord patterns
   - ✅ Same `/Arp Patterns/` folder
   - Query: Files with repeated rhythmic structures

### **What We Cannot Easily Do** ❌

4. **Chord Progressions (.mpcprog)** - **BLOCKED**
   - ❌ No documented file format
   - ❌ No MIDI → .mpcprog converter
   - ❌ Requires MPC Software to create
   - ❌ Would need reverse engineering

---

## 💡 Proposed Expansion Strategy

### **Phase 1: MIDI Patterns** (Current)
Create **97 instrument-specific expansions** with `.mpcpattern` files:
- MIDI_KICK, MIDI_SNARE, MIDI_BASS, etc.
- ~2.2M single-track patterns
- Storage: `/Expansions/MIDI_[INSTRUMENT]/Patterns/`

### **Phase 2: Functional Categories** (New)
Create **3 functional expansions** for special use cases:

#### **A. Arpeggiator Patterns Expansion**
```
/Expansions/MIDI_ARPEGGIOS/
├── Patterns/              # Standard .mpcpattern files
└── Arp_Patterns/         # Copy to /Arp Patterns/ manually
    ├── ascending/
    ├── descending/
    ├── random/
    └── melodic/
```

**Database Query:**
```sql
SELECT filepath FROM files
WHERE num_tracks = 1
  AND (
    -- Ascending pattern detection
    -- Look for files where notes generally ascend
    -- Could analyze note sequences in analysis_results
  )
LIMIT 5000;
```

#### **B. Rhythm Patterns Expansion**
```
/Expansions/MIDI_RHYTHMS/
├── Patterns/              # Standard .mpcpattern files
└── Rhythm_Patterns/      # Copy to /Arp Patterns/ manually
    ├── straight/
    ├── swing/
    ├── syncopated/
    └── shuffle/
```

**Database Query:**
```sql
SELECT filepath FROM files f
JOIN drum_patterns dp ON f.id = dp.file_id
WHERE f.num_tracks = 1
  AND dp.pattern_type IN ('groove', 'pattern')
  AND dp.feel IN ('straight', 'swing', 'shuffle', 'syncopated')
LIMIT 5000;
```

#### **C. Chord Progressions Expansion**
```
/Expansions/MIDI_CHORDS/
└── Patterns/              # .mpcpattern files only (not .mpcprog)
    ├── major/
    ├── minor/
    ├── jazz/
    └── pop/
```

**Database Query:**
```sql
SELECT filepath FROM files f
JOIN analysis_results ar ON f.id = ar.file_id
WHERE f.num_tracks = 1
  -- Look for chord-heavy files
  -- Could check controller_data for sustained notes
  -- Or look for files with chord progressions in chords table
LIMIT 5000;
```

**Note:** These would be `.mpcpattern` files (usable in tracks), NOT `.mpcprog` files (Pad Perform progressions).

### **Phase 3: Advanced (Future)**
- Reverse engineer `.mpcprog` format
- Build MIDI → .mpcprog converter
- Create true Pad Perform chord progression packs

---

## 🛠️ Implementation Plan

### **Immediate Actions**

1. **Complete Phase 1 (97 instruments)** ← Current focus
   - Convert 2.2M single-track files → `.mpcpattern`
   - Organize into 97 expansions
   - Test on Force/MPC One

2. **Add Arp Patterns folder** (5-10 minutes)
   ```bash
   # Identify arp-style patterns from database
   psql $DB_URL -c "
     SELECT filepath FROM files
     WHERE num_tracks = 1
       AND filepath LIKE '%arp%'
        OR filepath LIKE '%ascending%'
        OR filepath LIKE '%descending%'
     LIMIT 1000;
   " > /tmp/arp_patterns.txt

   # Copy MIDI files to Arp Patterns folder
   mkdir -p /media/dojevou/NewSSD2/Arp_Patterns
   cat /tmp/arp_patterns.txt | while read file; do
     cp "$file" /media/dojevou/NewSSD2/Arp_Patterns/
   done
   ```

3. **Add Rhythm Patterns folder** (5-10 minutes)
   ```bash
   # Query rhythmic patterns from database
   psql $DB_URL -c "
     SELECT f.filepath
     FROM files f
     JOIN drum_patterns dp ON f.id = dp.file_id
     WHERE f.num_tracks = 1
       AND dp.pattern_type = 'groove'
     LIMIT 1000;
   " > /tmp/rhythm_patterns.txt

   # Copy to same Arp Patterns folder
   cat /tmp/rhythm_patterns.txt | while read file; do
     cp "$file" /media/dojevou/NewSSD2/Arp_Patterns/
   done
   ```

4. **Create 3 functional expansions** (20-30 minutes)
   - Build MIDI_ARPEGGIOS expansion (convert to .mpcpattern)
   - Build MIDI_RHYTHMS expansion (convert to .mpcpattern)
   - Build MIDI_CHORDS expansion (convert to .mpcpattern)

### **Future Research**

5. **Reverse engineer .mpcprog format**
   - Download sample .mpcprog files from expansions
   - Analyze binary/JSON structure
   - Document format specification
   - Build converter tool

---

## 📁 Final Directory Structure

```
/media/dojevou/NewSSD2/
├── Expansions/                      # For MPC device
│   ├── MIDI_KICK/
│   ├── MIDI_SNARE/
│   ├── MIDI_BASS/
│   ├── ... (94 more instrument packs)
│   ├── MIDI_ARPEGGIOS/             # Functional pack 1
│   ├── MIDI_RHYTHMS/               # Functional pack 2
│   └── MIDI_CHORDS/                # Functional pack 3
│
├── Arp_Patterns/                    # Copy to MPC device root
│   ├── ascending_001.mid
│   ├── descending_001.mid
│   ├── rhythm_straight_001.mid
│   └── ... (1000-2000 patterns)
│
└── Progressions/                    # Future: when we solve .mpcprog
    └── (empty for now)
```

**Copy to MPC device:**
```bash
# Mount MPC/Force as USB drive
# Copy Expansions folder
cp -r /media/dojevou/NewSSD2/Expansions /media/[MPC_DEVICE]/

# Copy Arp Patterns to root
cp -r /media/dojevou/NewSSD2/Arp_Patterns /media/[MPC_DEVICE]/

# Future: Copy Progressions
# cp -r /media/dojevou/NewSSD2/Progressions /media/[MPC_DEVICE]/
```

---

## 🔍 Database Queries for Pattern Types

### **1. Find Arpeggio-Style Patterns**
```sql
-- Patterns with ascending/descending note sequences
SELECT f.filepath, ar.note_range, ar.polyphony_avg
FROM files f
JOIN analysis_results ar ON f.id = ar.file_id
WHERE f.num_tracks = 1
  AND ar.polyphony_avg < 2.0  -- Mostly single notes
  AND ar.note_range > 12       -- Covers at least 1 octave
  AND (
    f.filename LIKE '%arp%'
    OR f.filename LIKE '%ascending%'
    OR f.filename LIKE '%descending%'
  )
LIMIT 5000;
```

### **2. Find Rhythm Patterns**
```sql
-- Patterns with rhythmic characteristics
SELECT f.filepath, dp.pattern_type, dp.feel
FROM files f
JOIN drum_patterns dp ON f.id = dp.file_id
WHERE f.num_tracks = 1
  AND dp.pattern_type IN ('groove', 'pattern', 'loop')
  AND dp.feel IS NOT NULL
LIMIT 5000;
```

### **3. Find Chord Progressions**
```sql
-- Patterns with multiple simultaneous notes (chords)
SELECT f.filepath, ar.polyphony_avg, ar.chord_complexity_score
FROM files f
JOIN analysis_results ar ON f.id = ar.file_id
WHERE f.num_tracks = 1
  AND ar.polyphony_avg >= 3.0  -- Average 3+ notes at once
  AND ar.chord_complexity_score > 0.5
LIMIT 5000;
```

### **4. Find Melodic Sequences**
```sql
-- Single-note melodic patterns
SELECT f.filepath, m.key_signature, ar.note_density
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
JOIN analysis_results ar ON f.id = ar.file_id
WHERE f.num_tracks = 1
  AND ar.polyphony_avg < 1.5   -- Mostly single notes
  AND ar.note_density > 0.3    -- Not too sparse
  AND m.key_signature IS NOT NULL
LIMIT 5000;
```

---

## 📝 Summary

### **What We're Building:**

1. ✅ **97 instrument expansions** with `.mpcpattern` files (current focus)
2. ✅ **Arpeggiator patterns** folder with `.mid` files (easy to add)
3. ✅ **Rhythm patterns** folder with `.mid` files (easy to add)
4. ❌ **Chord progressions** with `.mpcprog` files (future research needed)

### **Total Expansions:**
- 97 instrument-based expansions
- 3 functional expansions (Arps, Rhythms, Chords)
- **100 total expansion packs**

### **File Counts:**
- MIDI Patterns: ~2.2M files → `.mpcpattern`
- Arp Patterns: ~1,000-2,000 files → `.mid`
- Rhythm Patterns: ~1,000-2,000 files → `.mid`
- Chord Progressions: 0 (blocked until .mpcprog solved)

### **Timeline:**
- Phase 1 (97 instruments): 15-40 minutes (current)
- Phase 2 (arp/rhythm folders): 10-20 minutes
- Phase 3 (functional expansions): 20-30 minutes
- **Total: 45-90 minutes for complete library**

---

**Next Steps:** Do you want me to create the database queries to identify arp/rhythm patterns and build the functional expansions?

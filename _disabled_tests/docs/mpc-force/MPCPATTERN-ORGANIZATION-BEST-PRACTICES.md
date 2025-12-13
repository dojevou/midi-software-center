# .mpcpattern File Organization - Best Practices Guide

**Date:** November 22, 2025
**Based on:** Analysis of 100+ commercial MPC/Force expansion packs + community research

---

## 🏗️ Standard MPC Expansion Structure

### Root Location
```
/media/dojevou/RYXSTR/Expansions/
└── [Expansion Name]/
```

All expansions MUST be inside an `Expansions` folder in the root of your drive.

---

## 📁 Typical Expansion Folder Structure

### Option 1: Flat Structure (Most Common)
**Used by**: Deep House, Techno, Drum 'N' Bass packs

```
/Expansions/Deep House/
├── Cache.json
├── expansion-image.jpg
├── [Previews]/
│   ├── preview-01.mp3
│   └── preview-02.mp3
├── DeepHouse-Kit-01 120.xpm        # Program file
├── DeepHouse-Kit-01 120.sxq        # Sequence file
├── DeepHouse-Kit-01 120-Pattern.mpcpattern  # MIDI pattern
├── DeepHouse-Kick-01.WAV           # Sample
├── DeepHouse-Snare-01.WAV
└── [hundreds more WAV files...]
```

**Characteristics:**
- ✅ All files in root of expansion folder
- ✅ .mpcpattern files named matching their .xpm kit
- ✅ Samples use descriptive prefixes (genre-type-name)
- ✅ BPM in filename for patterns
- ✅ [Previews] subfolder for demo audio

---

###Option 2: Organized Subfolders (Professional)
**Used by**: 80s Nostalgia, Araab Muzik Vol 2

```
/Expansions/80s Nostalgia/
├── Cache.json
├── expansion-image.jpg
├── [Previews]/
│   └── demos.mp3
├── 80s Nostalgia Demo Project_[ProjectData]/  # Project folder
│   ├── project.xpj
│   └── [related files]
├── Samples/
│   ├── Drums/
│   │   ├── Kick/
│   │   ├── Snare/
│   │   └── HiHat/
│   ├── Synths/
│   └── Bass/
├── Programs/
│   ├── Kit-01.xpm
│   ├── Kit-01-Pattern.mpcpattern
│   └── Kit-02.xpm
└── Sequences/
    ├── Seq-01.sxq
    └── Seq-02.sxq
```

**Characteristics:**
- ✅ Organized by content type
- ✅ Samples grouped by instrument/category
- ✅ Programs in dedicated folder
- ✅ Easy to browse and manage
- ✅ More professional presentation

---

## 🎯 Naming Conventions

### .mpcpattern Files

**Format:** `[Genre]-Kit-[Name] [BPM]-Pattern.mpcpattern`

**Examples:**
```
DeepHouse-Kit-DH Kit 02 122-Pattern.mpcpattern
Techno-Kit-Techno Kit 05 127-Pattern.mpcpattern
TrapKit-TS 01 130 Ebmin-Pattern.mpcpattern
```

**Key Elements:**
1. **Genre prefix** - Identifies the pack
2. **Kit name/number** - Links to corresponding .xpm file
3. **BPM** - Essential for browsing
4. **Key** (optional) - For melodic patterns
5. **"-Pattern" suffix** - Distinguishes from .sxq sequences

---

### .xpm Program Files

**Format:** `[Genre]-Kit-[Name] [BPM].xpm`

**Examples:**
```
DeepHouse-Kit-DH Kit 02 122.xpm
Techno-Multi-Tech Kicks.xpm
TrapKit-TS 05 108 Ebmin.xpm
```

---

### WAV Samples

**Format:** `[Genre]-[Type]-[Name] [Key].WAV`

**Examples:**
```
DeepHouse-Kick-DH Kik 01.WAV
DeepHouse-Clap-DH Clps 24.WAV
DeepHouse-Loop-Wav-Bass-01LpG.WAV
DeepHouse-Hits-Synth-05Hit F#m.WAV
```

**Key Elements:**
1. **Genre prefix** - Pack identifier
2. **Type** - Kick, Snare, Loop, etc.
3. **Name** - Descriptive identifier
4. **Key/Note** - For tonal samples

---

## 🎵 Organization Strategies

### Strategy 1: By BPM (Recommended for Patterns)
```
/MIDI_Patterns/
├── 080-100_BPM/
│   ├── Pattern-085-DrumGroove.mpcpattern
│   └── Pattern-095-HipHop.mpcpattern
├── 100-120_BPM/
│   ├── Pattern-110-House.mpcpattern
│   └── Pattern-118-Breaks.mpcpattern
├── 120-140_BPM/
│   ├── Pattern-125-Techno.mpcpattern
│   └── Pattern-130-Trap.mpcpattern
└── 140-180_BPM/
    ├── Pattern-160-DnB.mpcpattern
    └── Pattern-174-Jungle.mpcpattern
```

**Best for:** Quick workflow, live performance, genre-agnostic

---

### Strategy 2: By Genre
```
/MIDI_Patterns/
├── House/
│   ├── DeepHouse-122.mpcpattern
│   └── TechHouse-127.mpcpattern
├── Hip-Hop/
│   ├── BoomBap-090.mpcpattern
│   └── Trap-130.mpcpattern
├── Drum-and-Bass/
│   ├── DnB-174-Neurofunk.mpcpattern
│   └── DnB-170-Liquid.mpcpattern
└── Techno/
    ├── Techno-127-Minimal.mpcpattern
    └── Techno-132-Hard.mpcpattern
```

**Best for:** Genre-specific production, style exploration

---

### Strategy 3: By Kit/Program (Paired)
```
/Expansions/My Pack/
├── Kit-01-HipHop-095.xpm
├── Kit-01-HipHop-095-Pattern.mpcpattern
├── Kit-01-HipHop-095.sxq
├── Kit-02-House-122.xpm
├── Kit-02-House-122-Pattern.mpcpattern
└── Kit-02-House-122.sxq
```

**Best for:** Cohesive kits, pattern-program pairs, expansion packs

---

### Strategy 4: By Function/Type
```
/MIDI_Patterns/
├── Grooves/
│   ├── 4-4-Groove-122.mpcpattern
│   └── Breakbeat-095.mpcpattern
├── Fills/
│   ├── Tom-Fill-01.mpcpattern
│   └── Crash-Fill-02.mpcpattern
├── Intros/
│   ├── Build-Up-128.mpcpattern
│   └── Riser-Pattern.mpcpattern
└── Endings/
    ├── Breakdown-120.mpcpattern
    └── Outro-Pattern.mpcpattern
```

**Best for:** Arrangement-focused workflow, building complete tracks

---

## 📊 Commercial Pack Analysis

### From Real Examples (Deep House Pack)

**Pattern Distribution:**
- 25 .mpcpattern files
- 25 matching .xpm programs
- 25 matching .sxq sequences
- ~500 WAV samples

**Naming Pattern:**
```
DeepHouse-Kit-DH Kit [01-25] [120-124]-Pattern.mpcpattern
```

**BPM Range:** 120-124 (genre-appropriate)
**Organization:** Flat (all files in root)
**Size:** ~150 MB per pack

---

## 🎯 Recommendations for Your Library

Based on 2.8M MIDI files + reverse-engineered format:

### Recommended Structure
```
/media/dojevou/RYXSTR/Expansions/
├── MIDI Patterns Collection/       # Custom expansion
│   ├── Cache.json
│   ├── [Previews]/
│   ├── By BPM/
│   │   ├── 080-100/
│   │   ├── 100-120/
│   │   ├── 120-140/
│   │   └── 140-180/
│   ├── By Genre/
│   │   ├── Hip-Hop/
│   │   ├── House/
│   │   ├── Techno/
│   │   └── DnB/
│   └── By Instrument/
│       ├── Drums/
│       ├── Bass/
│       └── Melodic/
```

---

## 💡 Best Practices from Community

### File Naming
1. **Include BPM** - Essential for browsing
2. **Use hyphens** - Better than underscores (MPC browser tags)
3. **Keep names short** - Easier to read on small screens
4. **Consistent prefixes** - Groups related files
5. **Include key** - For melodic/harmonic patterns

### Folder Organization
1. **Flat for small packs** (<50 files) - Easier to browse
2. **Subfolders for large packs** (50+ files) - Better management
3. **[Previews] folder** - Always include for demos
4. **Cache.json** - Required for proper expansion recognition
5. **expansion-image.jpg** - Visual identity in browser

### Workflow Tips
1. **Match .mpcpattern to .xpm** - Same name, easier to find pairs
2. **Group by BPM first** - Fastest workflow in production
3. **Sub-organize by genre** - Within BPM folders
4. **Use descriptive names** - Future-you will thank you
5. **Keep originals** - Separate folder for .mid sources

---

## 🚀 Implementation Strategy

### For Your Database

1. **Query PostgreSQL** for best patterns
   ```sql
   SELECT filepath, bpm, key_signature, tags
   FROM files f
   JOIN musical_metadata m ON f.id = m.file_id
   WHERE tags LIKE '%drum%' AND bpm BETWEEN 120 AND 140
   ORDER BY bpm, key_signature
   LIMIT 5000;
   ```

2. **Convert to .mpcpattern** (using our tool)
   ```bash
   cargo run --bin midi_to_mpcpattern -- --batch /path/to/midi /path/to/patterns
   ```

3. **Organize by BPM** (primary)
   ```bash
   # Group into BPM folders
   mv *-120-*.mpcpattern ./120-140_BPM/
   mv *-130-*.mpcpattern ./120-140_BPM/
   ```

4. **Create expansion metadata**
   ```bash
   # Generate Cache.json
   # Add expansion image
   # Create [Previews] folder
   ```

5. **Test on Force** hardware
   - Load expansion
   - Browse patterns
   - Validate playback
   - Check browser organization

---

## 📋 Checklist for Custom Expansion Pack

- [ ] Expansion folder in `/Expansions/`
- [ ] `Cache.json` file (required)
- [ ] Expansion image (.jpg)
- [ ] `[Previews]` folder with demo audio
- [ ] .mpcpattern files with BPM in name
- [ ] Matching .xpm files (if included)
- [ ] Consistent naming convention
- [ ] BPM range appropriate for genre
- [ ] Files organized logically (flat or subfolders)
- [ ] Total size reasonable (<500 MB recommended)

---

## 🔗 Related Documentation

- `MPCPATTERN-FORMAT-SPECIFICATION.md` - Technical format details
- `MPCPATTERN-REVERSE-ENGINEERING-SUMMARY.md` - Format analysis
- `FORCE-MIDI-INTEGRATION-GUIDE.md` - Force integration guide
- `FORCE-EXPORT-SUMMARY.md` - Export system documentation

---

## 💡 Key Takeaways

1. **Flat structure works best** for small-medium packs (<100 files)
2. **BPM in filename is essential** - Universal convention
3. **Match pattern names to programs** - Easier workflows
4. **Use hyphens for tags** - MPC browser categorization
5. **[Previews] folder recommended** - Helps users demo content
6. **Cache.json required** - Proper expansion recognition
7. **Keep it simple** - Don't over-organize

---

**Bottom Line:** Commercial packs use flat structures with descriptive names and BPM prominently featured. For large collections, organize by BPM first, then sub-categorize by genre or type. Match .mpcpattern files to their corresponding .xpm programs for cohesive kits.

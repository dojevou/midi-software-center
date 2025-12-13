# MIDI Expansion Pack System for Akai Force

**Created:** November 22, 2025
**Purpose:** Force-style MIDI expansion packs (like audio expansions, but for MIDI)

---

## 🎯 Concept Overview

**What:** Curated MIDI file collections organized like Akai Force expansions
**Why:** Force browser works best with organized, themed packs (not 2.7M files)
**How:** Database queries + smart organization + Force-friendly naming

---

## 📦 Pack Structure (Force-Style)

```
/media/dojevou/RYXSTR/midi/Expansions/
├── Drum_Grooves_Rock_100-140/
│   ├── _info.txt                           # Pack metadata
│   ├── 100-110_BPM/
│   │   ├── 100bpm_Cmaj_Kick_Basic_4-4.mid
│   │   ├── 105bpm_Gmaj_Snare_Rock_Groove.mid
│   │   └── ... (50-200 files)
│   ├── 110-120_BPM/
│   ├── 120-130_BPM/
│   └── 130-140_BPM/
│
├── Bass_Lines_Hip_Hop/
│   ├── _info.txt
│   ├── C_Major/
│   ├── D_Minor/
│   ├── G_Major/
│   └── ...
│
├── Chord_Progressions_Keys/
│   ├── _info.txt
│   ├── Jazz/
│   ├── Pop/
│   ├── Rock/
│   └── EDM/
│
└── [More expansion packs...]
```

---

## 🎨 Expansion Pack Templates

### Template 1: Genre + Instrument + BPM
**Name:** `{Genre}_{Instrument}_{BPM_Range}`
**Example:** `Hip_Hop_Drums_80-120`

**Contents:**
- 3-5 BPM subfolders (10-20 BPM increments)
- 50-200 files per subfolder
- Files named: `{BPM}bpm_{Key}_{Style}_{Description}.mid`

**Database Query:**
```sql
SELECT f.filepath, f.filename, m.bpm, m.key_signature
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE t.name IN ('hip-hop', 'drums')
  AND m.bpm BETWEEN 80 AND 120
  AND m.bpm IS NOT NULL
ORDER BY m.bpm;
```

---

### Template 2: Instrument + Key
**Name:** `{Instrument}_by_Key`
**Example:** `Bass_Lines_by_Key`

**Contents:**
- 24 key subfolders (12 major + 12 minor)
- 50-200 files per key
- Cross-referenced with genres

**Database Query:**
```sql
SELECT f.filepath, f.filename, m.key_signature, m.bpm
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE t.name IN ('bass', 'bassline', 'sub')
  AND m.key_signature IS NOT NULL
ORDER BY m.key_signature, m.bpm;
```

---

### Template 3: Pattern Type
**Name:** `{Pattern_Type}_{Style}`
**Example:** `Drum_Fills_All_Styles`

**Contents:**
- Organized by instrument/cymbal type
- Subfolders: Kick, Snare, Hat, Crash, Tom, Full_Kit
- Great for live performance

**Database Query:**
```sql
SELECT f.filepath, f.filename, m.bpm, t.name as instrument
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE t.name IN ('fill', 'fills')
ORDER BY t.name, m.bpm;
```

---

### Template 4: Curated Top Picks
**Name:** `Top_{N}_{Category}`
**Example:** `Top_100_Grooves`

**Contents:**
- Best files based on tag quality
- Files with 5+ tags (comprehensive metadata)
- Organized by usage frequency

**Database Query:**
```sql
SELECT f.filepath, f.filename, m.bpm, m.key_signature, COUNT(ft.tag_id) as tag_count
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
JOIN file_tags ft ON f.id = ft.file_id
WHERE m.bpm IS NOT NULL
GROUP BY f.id, f.filepath, f.filename, m.bpm, m.key_signature
HAVING COUNT(ft.tag_id) >= 5
ORDER BY tag_count DESC
LIMIT 100;
```

---

## 📋 Expansion Pack Ideas

### Drums:
1. **Drum_Grooves_Rock_100-140** (5,000 files)
   - 4/4 straight beats
   - Rock, metal, punk styles
   - Organized by BPM (10 BPM increments)

2. **Drum_Grooves_Hip_Hop_80-120** (3,000 files)
   - Boom-bap, trap, lo-fi
   - Swing and straight feels
   - By BPM range

3. **Drum_Fills_All_Instruments** (2,000 files)
   - Kick fills, snare rolls, hat patterns
   - Crash/ride cymbal hits
   - Organized by instrument

4. **Drum_Grooves_Jazz_60-180** (2,000 files)
   - Swing, bebop, Latin
   - Brushes, sticks, mallets
   - By tempo + feel

5. **Drum_Grooves_EDM_120-180** (3,000 files)
   - House, techno, dubstep, drum & bass
   - 4/4, half-time, double-time
   - By BPM + subgenre

### Bass:
6. **Bass_Lines_Hip_Hop** (2,000 files)
   - 808, 909, sub bass
   - Organized by key (24 folders)
   - 80-120 BPM range

7. **Bass_Lines_Funk** (1,500 files)
   - Slap, fingerstyle, picked
   - Syncopated grooves
   - By key + BPM

8. **Bass_Lines_EDM** (2,000 files)
   - Wobbles, subs, leads
   - House, techno, dubstep
   - By key + genre

### Keys & Chords:
9. **Chord_Progressions_Pop** (3,000 files)
   - I-V-vi-IV and variations
   - Piano, synth, pad
   - Organized by key

10. **Chord_Progressions_Jazz** (2,000 files)
    - ii-V-I, turnarounds
    - Complex voicings
    - Organized by key

11. **Piano_Melodies_Classical** (1,500 files)
    - Scales, arpeggios, runs
    - Organized by key
    - Various tempos

12. **Synth_Leads_EDM** (2,000 files)
    - Saw, square, sine waves
    - By key + BPM
    - Genre-tagged

### Melodic:
13. **Guitar_Riffs_Rock** (1,000 files)
    - Power chords, licks
    - By key + tuning
    - Organized by style

14. **String_Sections_Orchestral** (1,000 files)
    - Violin, cello, ensemble
    - By key + mood
    - Classical to cinematic

### Loops & Patterns:
15. **Multi_Instrument_Loops_120BPM** (2,000 files)
    - Full arrangements
    - Drums + bass + melody
    - Various genres at 120 BPM

16. **Percussion_Loops_Latin** (1,000 files)
    - Congas, bongos, shakers
    - Salsa, samba, reggaeton
    - By tempo + style

---

## 🎯 Starter Pack Collection

### "Force MIDI Essentials" (20,000 files, ~500MB)

**Pack 1:** Drum Grooves 100-140 (5,000)
**Pack 2:** Drum Fills All Styles (2,000)
**Pack 3:** Bass Lines by Key (3,000)
**Pack 4:** Chord Progressions Keys (5,000)
**Pack 5:** Multi-Instrument Loops (3,000)
**Pack 6:** Top 100 of Each Category (2,000)

**Export time:** 15-30 minutes
**Force browser:** Fast and responsive

---

## 🛠️ Pack Metadata (_info.txt)

Each expansion pack includes an `_info.txt` file:

```
MIDI Expansion Pack Information
================================

Pack Name: Drum Grooves Rock 100-140 BPM
Version: 1.0
Created: 2025-11-22
Files: 5,247

Description:
Rock drum grooves organized by BPM range (100-140).
Includes straight 4/4 beats, variations, and transitions.
Styles: Rock, Metal, Punk, Alternative.

Organization:
- 100-110_BPM/ (1,243 files)
- 110-120_BPM/ (1,456 files)
- 120-130_BPM/ (1,389 files)
- 130-140_BPM/ (1,159 files)

File Naming:
{BPM}bpm_{Key}_{Style}_{Description}.mid
Example: 120bpm_Cmaj_Rock_Basic_Groove.mid

Tags: drums, rock, groove, 4/4, straight
BPM Range: 100-140
Keys: All 24 keys represented

Database Query Used:
SELECT f.* FROM files f
JOIN musical_metadata m ON f.id = m.file_id
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE t.name IN ('drums', 'rock', 'groove')
  AND m.bpm BETWEEN 100 AND 140
ORDER BY m.bpm, m.key_signature;

Compatibility:
- Akai Force (OS 3.5+)
- Akai MPC (all models)
- Standard MIDI format (Type 0/1)
```

---

## 🚀 Export Script Features

### Core Functions:
1. **Pack Templates:** 16 predefined expansion pack types
2. **Custom Queries:** Build your own packs with SQL
3. **Smart Organization:** Auto-organize by BPM, key, genre, instrument
4. **File Renaming:** Add metadata to filenames for quick browsing
5. **Pack Generation:** Create multiple packs in one run
6. **Progress Tracking:** Real-time file count and ETA
7. **Dry Run Mode:** Preview before copying
8. **Resume Support:** Skip already-exported files

### Performance:
- **Parallel copying:** 16 threads for speed
- **Batch queries:** Efficient database access
- **Space check:** Verify drive capacity
- **Duplicate detection:** Skip redundant files

### Output:
- **Force-ready folders:** Browse immediately on hardware
- **Metadata files:** _info.txt in each pack
- **Index file:** master_pack_list.txt with all packs
- **Statistics:** pack_stats.json with export metrics

---

## 📊 Pack Size Guidelines

**Small Pack:** 500-1,000 files (~20-50 MB)
- Quick to browse
- Focused theme
- Perfect for specific sessions

**Medium Pack:** 1,000-3,000 files (~50-150 MB)
- Versatile collection
- Genre or instrument focused
- Good balance of variety and speed

**Large Pack:** 3,000-10,000 files (~150-500 MB)
- Comprehensive library
- Multiple subfolders
- May slow browser slightly

**Optimal:** 2,000 files per pack
- Fast browser performance
- Enough variety
- Easy to navigate

---

## 🎛️ Force Browser Optimization

### Best Practices:
1. **Shallow hierarchy:** 2-3 folder levels max
2. **Clear names:** Genre_Instrument_BPM format
3. **Limited files/folder:** 50-200 files per folder
4. **Consistent naming:** {BPM}bpm_{Key}_{Description}.mid
5. **Pack separation:** Each expansion in own folder

### File Naming for Force:
```
✅ Good: 120bpm_Cmaj_Kick_Rock_Groove.mid
❌ Bad: my super cool drum loop #5.mid

✅ Good: 140bpm_Amin_Bass_Techno_Loop.mid
❌ Bad: Bass - Loop (140).mid

✅ Good: 100bpm_Gmaj_Piano_Jazz_Chord_Prog.mid
❌ Bad: piano_chords_1.mid
```

### Benefits:
- Sortable by BPM (alphabetically)
- Key visible at a glance
- Instrument type clear
- Style/genre apparent
- Force browser shows full names

---

## 💾 Database Export Queries

### Pack 1: Drum Grooves by BPM
```sql
\COPY (
  SELECT
    f.filepath,
    f.filename,
    m.bpm,
    m.key_signature,
    string_agg(t.name, ', ') as tags
  FROM files f
  JOIN musical_metadata m ON f.id = m.file_id
  JOIN file_tags ft ON f.id = ft.file_id
  JOIN tags t ON ft.tag_id = t.id
  WHERE t.name IN ('drums', 'groove', 'rock')
    AND m.bpm BETWEEN 100 AND 140
  GROUP BY f.id, f.filepath, f.filename, m.bpm, m.key_signature
  ORDER BY m.bpm
) TO '/tmp/drum_grooves_rock_100-140.csv' CSV HEADER;
```

### Pack 2: Bass Lines by Key
```sql
\COPY (
  SELECT
    f.filepath,
    m.key_signature as key,
    m.bpm,
    string_agg(t.name, ', ') as tags
  FROM files f
  JOIN musical_metadata m ON f.id = m.file_id
  JOIN file_tags ft ON f.id = ft.file_id
  JOIN tags t ON ft.tag_id = t.id
  WHERE t.name IN ('bass', 'bassline', 'hip-hop')
  GROUP BY f.id, f.filepath, m.key_signature, m.bpm
  ORDER BY m.key_signature, m.bpm
) TO '/tmp/bass_hip_hop.csv' CSV HEADER;
```

### Pack 3: Top 100 Most Tagged
```sql
\COPY (
  SELECT
    f.filepath,
    f.filename,
    m.bpm,
    m.key_signature,
    COUNT(ft.tag_id) as tag_count,
    string_agg(t.name, ', ') as tags
  FROM files f
  JOIN musical_metadata m ON f.id = m.file_id
  JOIN file_tags ft ON f.id = ft.file_id
  JOIN tags t ON ft.tag_id = t.id
  WHERE m.bpm IS NOT NULL
  GROUP BY f.id, f.filepath, f.filename, m.bpm, m.key_signature
  HAVING COUNT(ft.tag_id) >= 5
  ORDER BY tag_count DESC
  LIMIT 100
) TO '/tmp/top_100_files.csv' CSV HEADER;
```

---

## 🎯 Next Steps

1. **Choose packs:** Select 5-10 expansion packs to create
2. **Build export script:** Automate pack generation
3. **Test on Force:** Verify browser performance
4. **Iterate:** Refine organization based on usage
5. **Create custom packs:** Build workflow-specific collections

---

**Ready to build the export script!**

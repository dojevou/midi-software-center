# Akai Force MIDI Export Strategy

**Created:** November 22, 2025
**Target:** `/media/dojevou/RYXSTR/midi`
**Drive Space:** 789GB available (plenty of room)

---

## 🎯 Akai Force Research Summary

### Key Findings:
1. **MIDI Support:** Force 3.1+ supports standard .mid/.midi files
2. **Folder Structure:** Uses "Progressions" folder for MIDI progressions
3. **Browser:** Grid-based clip browser - needs organized folders
4. **Workflow:** Users organize by genre, instrument, BPM, key for quick access
5. **Best Practice:** Create themed "packs" in subfolders (like Expansions)

### Force-Specific Optimizations:
- **Quick browsing:** Shallow folder hierarchy (2-3 levels max)
- **Clear names:** Genre/Instrument/BPM visible in folder/file names
- **Limited files per folder:** 50-200 files max for browser speed
- **Metadata in filename:** BPM and key for quick reference

---

## 📁 Proposed Folder Structure

```
/media/dojevou/RYXSTR/midi/
├── By_Genre/
│   ├── Rock/
│   │   ├── 100-120_BPM/
│   │   └── 120-140_BPM/
│   ├── Hip_Hop/
│   ├── EDM/
│   ├── Jazz/
│   └── ... (14 genres total)
│
├── By_Instrument/
│   ├── Drums/
│   │   ├── Kick/
│   │   ├── Snare/
│   │   ├── Hi_Hat/
│   │   ├── Full_Kits/
│   │   └── Fills/
│   ├── Bass/
│   ├── Keys/
│   │   ├── Piano/
│   │   ├── Synth/
│   │   └── Pads/
│   └── ... (97 instruments organized)
│
├── By_BPM/
│   ├── 60-80_BPM/
│   ├── 80-100_BPM/
│   ├── 100-120_BPM/
│   ├── 120-140_BPM/
│   ├── 140-160_BPM/
│   └── 160-180_BPM/
│
├── By_Key/
│   ├── C_Major/
│   ├── C_Minor/
│   ├── G_Major/
│   └── ... (24 keys total)
│
├── Curated_Packs/
│   ├── Drum_Grooves_120BPM/
│   ├── Bass_Lines_Hip_Hop/
│   ├── Chord_Progressions_Keys/
│   └── ... (themed collections)
│
└── Quick_Access/
    ├── Top_100_Loops/
    ├── Top_100_Fills/
    └── Top_100_Progressions/
```

---

## 🎨 File Naming Convention

**Format:** `{BPM}bpm_{Key}_{Instrument}_{Description}.mid`

**Examples:**
- `120bpm_Cmaj_Kick_Rock_Groove.mid`
- `140bpm_Amin_Bass_Funk_Loop.mid`
- `100bpm_Gmaj_Piano_Chord_Progression.mid`

**Benefits:**
- Sortable by BPM (alphabetically)
- Key visible at a glance
- Instrument type clear
- Descriptive enough for quick decisions

---

## 🎯 Export Strategy Options

### Option 1: Curated Selection (RECOMMENDED)
**Export:** ~10,000-50,000 best files
**Criteria:**
- High-quality tags (3+ tags per file)
- Popular patterns (grooves, loops, fills)
- Organized by workflow (genre + instrument + BPM)
- Limit per folder: 200 files max

**Why:** Force browser performs better with smaller, curated libraries
**Time:** ~30-60 minutes to export

### Option 2: Full Collection
**Export:** All 2.76M files
**Organization:** By primary tag only (genre OR instrument)
**Why:** Complete library access
**Time:** Several hours
**Warning:** May slow down Force browser

### Option 3: Dynamic Packs
**Export:** Query-based "packs" created on demand
**Examples:**
- "120 BPM Hip Hop Drums"
- "C Minor Bass Lines"
- "Rock Grooves 4/4"
**Why:** Custom workflow-specific collections
**Time:** Minutes per pack

---

## 🗄️ Database Queries for Export

### 1. Drum Patterns by BPM
```sql
SELECT f.filepath, f.filename, m.bpm, m.key_signature, t.name as tag
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE t.name IN ('drums', 'groove', 'loop', 'fill')
  AND m.bpm BETWEEN 100 AND 140
  AND m.bpm IS NOT NULL
ORDER BY m.bpm, t.name;
```

### 2. Bass Lines by Key
```sql
SELECT f.filepath, f.filename, m.bpm, m.key_signature
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE t.name IN ('bass', 'bassline', 'sub', '808', '909')
  AND m.key_signature IS NOT NULL
ORDER BY m.key_signature, m.bpm;
```

### 3. Chord Progressions
```sql
SELECT f.filepath, f.filename, m.bpm, m.key_signature
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE t.name IN ('chord', 'progression', 'piano', 'keys', 'pad')
  AND m.key_signature IS NOT NULL
ORDER BY m.key_signature;
```

### 4. Genre-Based Collections
```sql
SELECT f.filepath, f.filename, m.bpm, m.key_signature, t.name as genre
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE t.name IN ('rock', 'jazz', 'hip-hop', 'edm', 'funk', 'techno')
  AND m.bpm IS NOT NULL
ORDER BY t.name, m.bpm;
```

---

## 💾 Export Script Features

### Core Functions:
1. **Query Database:** Extract files by criteria (genre, instrument, BPM, key)
2. **Organize Folders:** Create Force-friendly folder structure
3. **Rename Files:** Add BPM/key metadata to filenames
4. **Copy Files:** Efficient batch copying with progress
5. **Create Index:** Generate text file listing of exported files
6. **Symlinks Option:** Link files instead of copy (save space)

### Performance:
- **Parallel copying:** Use 8-16 threads for fast transfer
- **Progress tracking:** Real-time file count and ETA
- **Resume support:** Skip already-copied files
- **Validation:** Verify file integrity after copy

### Safety:
- **Dry-run mode:** Preview what will be exported
- **Max files per folder:** Prevent browser slowdown
- **Duplicate detection:** Skip redundant files
- **Space check:** Verify drive has enough room

---

## 🎛️ Recommended Starter Export

### "Force Essentials Pack" (~20,000 files)
```
1. Drum Grooves (5,000):
   - 100-140 BPM
   - Rock, Hip Hop, EDM, Funk
   - Organized by BPM range

2. Drum Fills (2,000):
   - All BPM ranges
   - By instrument type (kick, snare, hat, crash)

3. Bass Lines (3,000):
   - Organized by key (12 major, 12 minor)
   - Hip Hop, Funk, EDM styles

4. Chord Progressions (5,000):
   - Organized by key
   - Piano, synth, pad variations

5. Loops & Patterns (3,000):
   - Multi-instrument loops
   - Organized by genre + BPM

6. Top Hits (2,000):
   - Most-tagged files
   - Highest quality patterns
```

**Total Size:** ~500MB - 1GB
**Export Time:** 15-30 minutes
**Force Browser:** Fast and responsive

---

## 🚀 Next Steps

1. **Review Strategy:** Choose export option (Curated vs Full)
2. **Run Export Script:** Generate organized MIDI library
3. **Test on Force:** Verify browser performance and workflow
4. **Iterate:** Add/remove categories based on usage
5. **Create Packs:** Build themed collections for specific projects

---

**Ready to create the export script!**

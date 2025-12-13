# .mpcpattern File Storage Location - Research Findings

**Date:** November 22, 2025
**Question:** Should .mpcpattern files be stored in expansion folders or separately?

---

## ✅ RECOMMENDATION: Store in Expansion Folders

**Bottom Line:** .mpcpattern files should be stored **INSIDE expansion folders** in the `/Expansions/` directory on your MPC/Force drive.

---

## 📊 Research Findings

### 1. Official MPC OS Structure

**Required Location:**
```
/Expansions/                    # Root "Expansions" folder (required)
└── [Your_Expansion_Name]/      # Individual expansion folder
    ├── Cache.json              # Metadata (required for recognition)
    ├── expansion-image.jpg     # Thumbnail (512x512)
    ├── [Previews]/             # Audio demos
    └── Patterns/               # .mpcpattern files go here
        ├── pattern1.mpcpattern
        ├── pattern2.mpcpattern
        └── ...
```

**Key Requirements:**
1. **Must be in `/Expansions/`** - Root folder on drive (internal, SD, or USB)
2. **Each expansion = separate folder** - One folder per expansion pack
3. **Cache.json required** - For MPC OS to recognize as expansion
4. **Flat or 1-level subfolders** - Max 1-2 levels of organization within expansion

---

### 2. How MPC OS Accesses Files

#### Option A: Expansion Browser (RECOMMENDED ✅)
**Path:** `BROWSER > Expansions`

**Advantages:**
- ✅ Organized, visual interface with thumbnails
- ✅ Metadata from Cache.json displayed
- ✅ Filter by file type (MIDI, samples, programs)
- ✅ Quick access to entire pack
- ✅ Professional presentation
- ✅ Easy to enable/disable expansion packs
- ✅ Works exactly like commercial expansion packs

**How patterns appear:**
- Expansion shows with custom thumbnail
- Click expansion to browse contents
- Use "MIDI" filter to show only .mpcpattern files
- Files sorted alphabetically or by metadata

#### Option B: File Browser (Places)
**Path:** `BROWSER > Places > [Drive] > Expansions`

**Disadvantages:**
- ❌ Raw file system view (no thumbnails)
- ❌ No metadata display
- ❌ Harder to navigate (more clicks)
- ❌ No filtering by expansion
- ❌ Less professional workflow
- ❌ Mixes all expansions together

**When to use:**
- Troubleshooting file locations
- Direct file access for editing
- Checking disk space

---

### 3. Commercial Expansion Pack Analysis

**From official Akai and third-party packs:**

**Typical Structure:**
```
/Expansions/Deep_House_Patterns/
├── Cache.json                          # Metadata
├── expansion-image.jpg                 # 512x512 thumbnail
├── [Previews]/
│   └── deep-house-demo.mp3
├── DeepHouse-Kit-01-120.xpm           # Drum program
├── DeepHouse-Kit-01-120-Pattern.mpcpattern  # MIDI pattern (ROOT)
├── DeepHouse-Kit-01-120.sxq           # Sequence
├── DeepHouse-Kick-01.WAV              # Samples
└── ... (more files)
```

**OR with subfolder:**
```
/Expansions/Deep_House_Patterns/
├── Cache.json
├── expansion-image.jpg
├── [Previews]/
├── Samples/
│   ├── Kick/
│   └── Snare/
├── Programs/
│   └── Kit-01.xpm
└── MIDI Patterns/                      # Subfolder
    ├── Pattern-01-120.mpcpattern
    ├── Pattern-02-122.mpcpattern
    └── ...
```

**Key Findings:**
- ✅ Patterns stored IN expansion folder (root or 1 level deep)
- ✅ Named to match programs (.xpm files) when applicable
- ✅ Include BPM in filename
- ✅ Use hyphens for better browser parsing
- ⚠️ Never stored separately outside expansions

---

### 4. Why NOT to Store Separately

**If patterns stored outside `/Expansions/` (e.g., in `/MIDI_Patterns/`):**

❌ **Problems:**
1. No Expansion Browser access
2. No thumbnails or metadata
3. Mixed with system/user files
4. Harder to organize large collections
5. No "load expansion" workflow
6. Can't easily share/backup entire pack
7. Doesn't follow MPC OS conventions
8. Looks unprofessional

**File Browser only:**
- Must navigate: `Places > [Drive] > MIDI_Patterns > ...`
- No filtering, no previews, no metadata
- All patterns mixed together (no categorization by expansion)

---

## 🎯 Best Practices for 97 Instrument Expansions

### Recommended Structure

**For your 97 instrument packs:**

```
/Expansions/
├── MIDI_RIDE/                  # Expansion 1
│   ├── Cache.json
│   ├── expansion-image.jpg     # "RIDE" thumbnail
│   ├── [Previews]/
│   └── Patterns/               # 103,591 .mpcpattern files
│       ├── ride_080bpm_001.mpcpattern
│       ├── ride_085bpm_002.mpcpattern
│       └── ...
│
├── MIDI_KICK/                  # Expansion 2
│   ├── Cache.json
│   ├── expansion-image.jpg     # "KICK" thumbnail
│   ├── [Previews]/
│   └── Patterns/               # 75,286 .mpcpattern files
│       ├── kick_095bpm_001.mpcpattern
│       └── ...
│
... (95 more expansions)
│
└── MIDI_HIP_HOP/               # Expansion 97
    ├── Cache.json
    ├── expansion-image.jpg
    ├── [Previews]/
    └── Patterns/               # 22 .mpcpattern files
```

### Workflow Benefits

**When you need kick patterns:**
1. Open MPC: `BROWSER > Expansions`
2. See "MIDI_KICK" thumbnail
3. Click to open (75,286 patterns available)
4. Filter by BPM/key if needed
5. Load pattern

**vs. Separate storage:**
1. Open MPC: `BROWSER > Places`
2. Navigate to drive
3. Find MIDI folder
4. Browse thousands of mixed files
5. Hope you find the right kick pattern

---

## 📏 Size Considerations

### Large Expansions (10K+ files)

**Concern:** Some instruments have 100K+ patterns (e.g., Ride: 103,591)

**Solutions:**

**Option 1: Single Large Expansion**
```
/Expansions/MIDI_RIDE/
└── Patterns/           # All 103,591 files
```
- ✅ Simple structure
- ⚠️ May be slow to load on hardware
- ⚠️ Hard to browse on small screen

**Option 2: Sub-Expansions by BPM**
```
/Expansions/
├── MIDI_RIDE_080-100/
│   └── Patterns/       # ~25K files
├── MIDI_RIDE_100-120/
│   └── Patterns/       # ~25K files
├── MIDI_RIDE_120-140/
│   └── Patterns/       # ~25K files
└── MIDI_RIDE_140-180/
    └── Patterns/       # ~25K files
```
- ✅ Faster loading
- ✅ Easier to browse
- ✅ Load only what you need
- ❌ More expansion folders (4 per large instrument)

**Option 3: Smart Limits (5K per expansion)**
```
/Expansions/MIDI_RIDE/
└── Patterns/           # Top 5,000 ride patterns
```
- ✅ Fast loading
- ✅ Manageable size
- ❌ Excludes some patterns
- ✅ Can always add "MIDI_RIDE_Extended" later

---

## 🚀 Implementation Recommendation

### For Your 97 Instruments:

**Strategy:** Tiered approach based on file count

**Tier 1: Large (>20K files)**
- Split into BPM ranges (4 expansions each)
- Example: MIDI_RIDE → MIDI_RIDE_080-100, MIDI_RIDE_100-120, etc.
- Result: ~84 total expansions (21 large × 4)

**Tier 2: Medium (1K-20K files)**
- Single expansion per instrument
- Example: MIDI_PIANO → MIDI_PIANO (21,932 files)
- Result: 49 expansions

**Tier 3: Small (<1K files)**
- Single expansion per instrument
- Example: MIDI_HIP_HOP → MIDI_HIP_HOP (22 files)
- Result: 27 expansions

**Total: ~160 expansions** (vs original 97)

**OR**

**Simpler: Apply 5K limit to all**
- Result: 97 expansions, all manageable sizes
- Fast loading on all hardware
- Can create "Extended" packs later for popular instruments

---

## ✅ Final Answer

**Store .mpcpattern files IN expansion folders:**
```
/Expansions/[Expansion_Name]/[Patterns or root]/
```

**NOT separately:**
```
❌ /MIDI_Patterns/
❌ /User/Patterns/
❌ /Data/MIDI/
```

**Why:**
- MPC OS Expansion Browser integration
- Professional presentation
- Easier workflow
- Follows industry standard
- Better organization at scale
- Shareable/portable expansion packs

---

## 📋 Action Items

1. ✅ Create expansion folders in `/Expansions/`
2. ✅ Store all .mpcpattern files inside expansion folders
3. ✅ Include Cache.json for each expansion
4. ✅ Add expansion images (512x512)
5. ✅ Use Expansion Browser for access (not file browser)
6. ✅ Consider BPM-based sub-expansions for large collections
7. ✅ Test on hardware to verify loading speed

---

**Conclusion:** The research is clear - expansion folders are the correct and recommended location for .mpcpattern files on MPC OS devices.

# MPC OS - 97 Instrument-Specific Expansion Packs

**Date:** November 22, 2025
**Strategy:** One expansion pack per instrument (97 total)
**Target Devices:** All MPC OS devices (Force, MPC One, MPC Live/II, MPC X, MPC Key 61)

---

## 🎯 Strategy Overview

Instead of organizing by BPM or genre, create **97 focused expansion packs** - one for each instrument type found in the database.

### Why This Works:

1. **Instrument-first workflow** - Load exactly what you need
2. **Fast browsing** - Smaller, focused packs load instantly
3. **Scalable** - Can load/unload specific instruments per project
4. **Database-aligned** - Matches existing tag structure
5. **Producer-friendly** - Think in terms of "I need kicks" not "I need 120 BPM"

---

## 📊 The 97 Instruments (by file count)

### Tier 1: Large Collections (10,000+ files)
1. **MIDI_Ride** - 103,591 files (6.04%)
2. **MIDI_Fill** - 88,142 files (5.14%)
3. **MIDI_Kick** - 75,286 files (4.39%)
4. **MIDI_Tom** - 64,351 files (3.75%)
5. **MIDI_Bass** - 52,917 files (3.08%)
6. **MIDI_Rock** - 40,209 files (2.34%)
7. **MIDI_Crash** - 39,690 files (2.31%)
8. **MIDI_Loop** - 31,736 files (1.85%)
9. **MIDI_Synth** - 26,556 files (1.55%)
10. **MIDI_Piano** - 21,932 files (1.28%)
11. **MIDI_Bell** - 20,861 files (1.22%)
12. **MIDI_Lead** - 19,496 files (1.14%)
13. **MIDI_Drum** - 17,380 files (1.01%)
14. **MIDI_Snare** - 16,341 files (0.95%)
15. **MIDI_Stick** - 16,144 files (0.94%)
16. **MIDI_Melody** - 15,282 files (0.89%)
17. **MIDI_Chord** - 14,766 files (0.86%)
18. **MIDI_Pad** - 12,956 files (0.76%)
19. **MIDI_HiHat** - 11,459 files (0.67%)
20. **MIDI_Funk** - 11,136 files (0.65%)
21. **MIDI_Groove** - 10,729 files (0.63%)

### Tier 2: Medium Collections (1,000-9,999 files)
22-70. (49 instruments ranging from EP: 9,108 files to Vocal: 1,279 files)

### Tier 3: Small Collections (100-999 files)
71-91. (21 instruments ranging from Bongo: 1,229 files to Saxophone: 110 files)

### Tier 4: Boutique Collections (<100 files)
92-97. (6 instruments: Wurlitzer: 94, Impact: 75, R&B: 42, Hip-Hop: 22)

---

## 📁 Expansion Structure

Each instrument gets its own expansion folder:

```
/media/[DRIVE]/Expansions/
├── MIDI_Ride/                    # 103,591 patterns
│   ├── Cache.json
│   ├── expansion-image.jpg
│   ├── [Previews]/
│   │   └── ride-demo.mp3
│   └── Patterns/
│       ├── ride_080bpm_001.mpcpattern
│       ├── ride_085bpm_002.mpcpattern
│       └── ... (103,591 total)
│
├── MIDI_Fill/                    # 88,142 patterns
│   ├── Cache.json
│   ├── expansion-image.jpg
│   ├── [Previews]/
│   └── Patterns/
│       └── ... (88,142 total)
│
├── MIDI_Kick/                    # 75,286 patterns
│   └── ...
│
... (97 expansions total)
│
└── MIDI_Hip-Hop/                 # 22 patterns (smallest)
    └── ...
```

### File Naming Within Each Expansion

**Format:** `{instrument}_{bpm}bpm_{key}_{id}.mpcpattern`

**Examples:**
```
ride_120bpm_Cmaj_00001.mpcpattern
kick_095bpm_00042.mpcpattern
bass_128bpm_Amin_00156.mpcpattern
piano_110bpm_Gmaj_00023.mpcpattern
```

---

## 🚀 Implementation Workflow

### Step 1: Create 97 Expansion Folders

```bash
#!/bin/bash
# create_instrument_expansions.sh

EXPANSIONS_DIR="/media/[DRIVE]/Expansions"
INSTRUMENTS_FILE="INSTRUMENT_LIST.txt"

# Read each instrument and create expansion folder
while IFS=':' read -r instrument count; do
    instrument=$(echo "$instrument" | tr '[:lower:]' '[:upper:]' | tr '-' '_')
    expansion_name="MIDI_${instrument}"
    expansion_dir="$EXPANSIONS_DIR/$expansion_name"

    # Create folder structure
    mkdir -p "$expansion_dir/Patterns"
    mkdir -p "$expansion_dir/[Previews]"

    # Create Cache.json
    cat > "$expansion_dir/Cache.json" << EOF
{
  "name": "$expansion_name",
  "version": "1.0.0",
  "author": "MIDI Software Center",
  "description": "$count ${instrument} patterns",
  "category": "Instrument",
  "instrument": "$instrument",
  "pattern_count": $count
}
EOF

    # Create placeholder image
    convert -size 512x512 xc:black \
        -fill white -pointsize 60 -gravity center \
        -annotate +0+0 "${instrument}" \
        "$expansion_dir/expansion-image.jpg" 2>/dev/null || \
        echo "ImageMagick not available, skipping image"

    echo "✅ Created: $expansion_name ($count patterns)"
done < "$INSTRUMENTS_FILE"

echo ""
echo "🎉 Created 97 expansion folders"
```

---

### Step 2: Query Database Per Instrument

For each instrument, extract file paths of single-track files:

```sql
-- Example: Export all "kick" files
\COPY (
    SELECT
        f.filepath,
        f.filename,
        m.bpm,
        m.key_signature,
        ROW_NUMBER() OVER (ORDER BY m.bpm, f.filename) as pattern_id
    FROM files f
    JOIN file_tags ft ON f.id = ft.file_id
    JOIN tags t ON ft.tag_id = t.id
    JOIN musical_metadata m ON f.id = m.file_id
    WHERE
        f.num_tracks = 1           -- Single track only
        AND t.name = 'kick'         -- Instrument filter
    ORDER BY m.bpm, f.filename
) TO '/tmp/kick_patterns.csv' WITH CSV HEADER;
```

---

### Step 3: Parallel Conversion Per Instrument

```bash
#!/bin/bash
# convert_instrument_parallel.sh

INSTRUMENT=$1  # e.g., "kick"
CSV_FILE="/tmp/${INSTRUMENT}_patterns.csv"
OUTPUT_DIR="/media/[DRIVE]/Expansions/MIDI_${INSTRUMENT^^}/Patterns"
CONVERTER="./target/ultra-fast/midi_to_mpcpattern_parallel"

echo "🚀 Converting ${INSTRUMENT} patterns..."
echo "📊 Reading from: $CSV_FILE"
echo "📁 Output to: $OUTPUT_DIR"
echo ""

# Create temp directory with file list
TEMP_LIST="/tmp/${INSTRUMENT}_files.txt"
tail -n +2 "$CSV_FILE" | cut -d',' -f1 > "$TEMP_LIST"

# Convert using parallel converter
# (It will find all .mid files in the list and convert them)
$CONVERTER --batch-from-list "$TEMP_LIST" "$OUTPUT_DIR"

echo ""
echo "✅ ${INSTRUMENT} conversion complete"
```

---

### Step 4: Master Conversion Script

Convert all 97 instruments sequentially (or parallelize by instrument):

```bash
#!/bin/bash
# convert_all_instruments.sh

DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"
INSTRUMENTS_FILE="INSTRUMENT_LIST.txt"
CONVERTER="./target/ultra-fast/midi_to_mpcpattern_parallel"

total_instruments=97
current=0

while IFS=':' read -r instrument count; do
    current=$((current + 1))

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "[$current/$total_instruments] Converting: $instrument ($count files)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    # Export file list from database
    psql "$DB_URL" -t -A -F',' -c "
        SELECT f.filepath, m.bpm, m.key_signature
        FROM files f
        JOIN file_tags ft ON f.id = ft.file_id
        JOIN tags t ON ft.tag_id = t.id
        LEFT JOIN musical_metadata m ON f.id = m.file_id
        WHERE f.num_tracks = 1
          AND t.name = '$instrument'
        ORDER BY m.bpm, f.filename
    " > "/tmp/${instrument}_patterns.csv"

    # Count actual files found
    file_count=$(wc -l < "/tmp/${instrument}_patterns.csv")
    echo "📊 Found $file_count files in database"

    # Convert
    instrument_upper=$(echo "$instrument" | tr '[:lower:]' '[:upper:]' | tr '-' '_')
    output_dir="/media/[DRIVE]/Expansions/MIDI_${instrument_upper}/Patterns"

    # Extract just file paths
    cut -d',' -f1 "/tmp/${instrument}_patterns.csv" > "/tmp/${instrument}_files.txt"

    # Convert with parallel processor
    $CONVERTER --batch-from-list "/tmp/${instrument}_files.txt" "$output_dir"

    echo "✅ Completed: $instrument"

done < "$INSTRUMENTS_FILE"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎉 ALL 97 INSTRUMENTS CONVERTED"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
```

---

## 📊 Estimated Results

### Total Files to Convert
- **All 97 instruments:** ~1.7M single-track files
- **With limits (5K per instrument):** ~485K files (97 × 5,000)

### Conversion Performance
**With ultra-fast parallel converter:**
- Speed: 2,000-5,000 files/sec
- **Full collection (1.7M):** 6-15 minutes
- **Limited (485K):** 2-4 minutes

### Expansion Pack Sizes

| Tier | Instruments | Files/Expansion | Size/Expansion | Total Size |
|------|-------------|-----------------|----------------|------------|
| 1 (Large) | 21 | 10K-103K | 50-500 MB | ~5 GB |
| 2 (Medium) | 49 | 1K-10K | 5-50 MB | ~1 GB |
| 3 (Small) | 21 | 100-999 | 0.5-5 MB | ~50 MB |
| 4 (Boutique) | 6 | <100 | <0.5 MB | ~2 MB |
| **TOTAL** | **97** | **Variable** | **Variable** | **~6 GB** |

---

## 🎯 Optimization Strategies

### Strategy 1: No Limits (Convert All)
- **Pros:** Complete collection, maximum choice
- **Cons:** Some expansions are huge (103K files)
- **Best for:** Desktop/studio use with large storage

### Strategy 2: Smart Limits (5K per instrument)
- **Pros:** Manageable sizes, fast browsing
- **Cons:** May miss some patterns
- **Best for:** Mobile MPC devices with limited storage
- **Implementation:** Add `LIMIT 5000` to database queries

### Strategy 3: Tiered Approach
- **Tier 1 (large):** Limit to 10K files each
- **Tier 2 (medium):** Limit to 5K files each
- **Tier 3 (small):** Include all files
- **Tier 4 (boutique):** Include all files
- **Result:** ~350K patterns total, ~1.5 GB

---

## ✅ Advantages of This Approach

1. **Focused Loading** - Load only "MIDI_Kick" when making a beat
2. **Fast Browsing** - Small expansion packs load instantly on hardware
3. **Scalable** - Add/remove instruments as needed
4. **Database-Aligned** - Uses existing tag structure
5. **Flexible** - Can create sub-expansions (e.g., "MIDI_Kick_HipHop_80-100BPM")
6. **Universal** - Works on all MPC OS devices
7. **Organized** - Clear instrument-based categorization
8. **Efficient** - No duplicate files across expansions (if tags are exclusive)

---

## 🚀 Quick Start

```bash
# 1. Create all 97 expansion folders
./scripts/create_instrument_expansions.sh

# 2. Build ultra-fast converter
./scripts/build-ultra-fast-converter.sh

# 3. Convert all instruments (1.7M files, 6-15 min)
./scripts/convert_all_instruments.sh

# 4. Or convert with limits (485K files, 2-4 min)
./scripts/convert_all_instruments.sh --limit 5000

# 5. Copy to MPC device
cp -r /media/[DRIVE]/Expansions /media/[MPC_DEVICE]/

# 6. Test on hardware
# (Load a few expansions and test browsing/playback)
```

---

## 📋 Next Steps

1. ✅ Approve 97-instrument expansion strategy
2. ⏳ Decide on limits (all files vs 5K per instrument)
3. ⏳ Build ultra-fast parallel converter
4. ⏳ Create expansion folder structure
5. ⏳ Run database exports per instrument
6. ⏳ Execute parallel conversions
7. ⏳ Test on MPC OS hardware
8. ⏳ Refine based on workflow feedback

---

## 🔗 Related Documentation

- `INSTRUMENT_ANALYSIS.md` - Full 97-instrument breakdown
- `INSTRUMENT_LIST.txt` - Complete instrument list with counts
- `AKAI-FORCE-EXPANSION-STRATEGY.md` - Previous 8-expansion plan (deprecated)
- `MPCPATTERN-FORMAT-SPECIFICATION.md` - Technical format details
- `MPCPATTERN-ORGANIZATION-BEST-PRACTICES.md` - Commercial pack analysis

---

**Ready to proceed?** This strategy creates 97 focused, instrument-specific expansion packs optimized for MPC OS workflow. Each expansion loads fast, browses easily, and contains exactly what you need for that instrument.

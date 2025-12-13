# Akai MPC OS Expansion Strategy - Complete Plan

**Date:** November 22, 2025
**Target Devices:** Akai Force, MPC One, MPC Live, MPC Live II, MPC X, MPC Key 61 (all MPC OS devices)
**Goal:** Convert 2.2M single-track MIDI files to .mpcpattern format and organize into optimized MPC OS expansion packs

---

## 📊 Database Analysis

### Current Collection Stats
- **Total files:** 2,806,055
- **Single-track files:** 2,220,812 (79.1%) ✅ **TARGET FOR CONVERSION**
- **Multi-track files:** 585,241 (20.9%) ❌ Skip these
- **Average tracks per file:** 1.43
- **Max tracks:** 293

### File Distribution by Track Count
```
1 track:  2,220,812 files (79.1%)
2 tracks:   496,533 files (17.7%)
3 tracks:    15,074 files (0.5%)
4+ tracks:   73,636 files (2.6%)
```

---

## 🎯 Conversion Strategy

### Phase 1: Database Query & Selection

**Recommended Filters** (reduce 2.2M to manageable set):

#### Option A: Quality-Based Selection (Recommended)
```sql
-- Get top-quality patterns across all categories
SELECT
    f.id,
    f.filepath,
    f.filename,
    m.bpm,
    m.key_signature,
    m.time_signature,
    array_to_string(f.track_names, ', ') as instruments,
    dp.pattern_type,
    dp.feel
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
LEFT JOIN drum_patterns dp ON f.id = dp.file_id
LEFT JOIN file_tags ft ON f.id = ft.file_id
LEFT JOIN tags t ON ft.tag_id = t.id
WHERE
    f.num_tracks = 1                    -- Single track only
    AND m.bpm BETWEEN 60 AND 200       -- Valid BPM range
    AND m.bpm IS NOT NULL
    AND m.duration_seconds BETWEEN 1 AND 30  -- Reasonable pattern length
ORDER BY
    m.bpm,
    m.key_signature,
    f.filename
LIMIT 50000;  -- Manageable starting size
```

#### Option B: Category-Based Selection
```sql
-- Separate queries for different categories

-- 1. DRUMS (80-180 BPM, ~20K patterns)
SELECT f.id, f.filepath, m.bpm, dp.pattern_type, dp.feel
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
LEFT JOIN drum_patterns dp ON f.id = dp.file_id
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE
    f.num_tracks = 1
    AND t.name = 'drums'
    AND m.bpm BETWEEN 80 AND 180
    AND m.duration_seconds BETWEEN 1 AND 16
ORDER BY m.bpm
LIMIT 20000;

-- 2. BASS (90-150 BPM, ~5K patterns)
SELECT f.id, f.filepath, m.bpm, m.key_signature
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE
    f.num_tracks = 1
    AND t.name = 'bass'
    AND m.bpm BETWEEN 90 AND 150
    AND m.key_signature IS NOT NULL
ORDER BY m.bpm, m.key_signature
LIMIT 5000;

-- 3. MELODIC (100-140 BPM, ~10K patterns)
SELECT f.id, f.filepath, m.bpm, m.key_signature
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE
    f.num_tracks = 1
    AND t.name IN ('piano', 'synth', 'keys', 'lead', 'pad')
    AND m.bpm BETWEEN 100 AND 140
    AND m.key_signature IS NOT NULL
ORDER BY m.bpm, m.key_signature
LIMIT 10000;

-- 4. LOOPS (various BPM, ~10K patterns)
SELECT f.id, f.filepath, m.bpm, m.key_signature
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE
    f.num_tracks = 1
    AND t.name = 'loop'
    AND m.bpm IS NOT NULL
    AND m.duration_seconds BETWEEN 2 AND 16
ORDER BY m.bpm
LIMIT 10000;

-- 5. FILLS (any BPM, ~5K patterns)
SELECT f.id, f.filepath, m.bpm, dp.pattern_type
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
LEFT JOIN drum_patterns dp ON f.id = dp.file_id
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE
    f.num_tracks = 1
    AND (t.name = 'fill' OR dp.pattern_type = 'fill')
    AND m.duration_seconds BETWEEN 0.5 AND 4
ORDER BY m.bpm
LIMIT 5000;
```

**Total Selection:** ~50,000 patterns (manageable size, ~2.2% of single-track collection)

---

## 📁 MPC OS Expansion Structure

### Recommended Organization: **Multi-Expansion Strategy**

Instead of one massive expansion, create **multiple themed expansion packs** for better workflow.

**Compatible with:** All MPC OS devices (Force, MPC One, MPC Live/II, MPC X, MPC Key 61)

```
/media/[DRIVE]/Expansions/
├── MIDI_Drums_080-100_BPM/          # Expansion 1: Slow drums
│   ├── Cache.json
│   ├── expansion-image.jpg
│   ├── [Previews]/
│   │   └── demo-080-100-drums.mp3
│   ├── Grooves/
│   │   ├── 080BPM-Funk-Groove-01.mpcpattern
│   │   ├── 085BPM-HipHop-Groove-02.mpcpattern
│   │   └── 095BPM-Breaks-Groove-03.mpcpattern
│   └── Fills/
│       ├── 090BPM-Tom-Fill-01.mpcpattern
│       └── 095BPM-Crash-Fill-02.mpcpattern
│
├── MIDI_Drums_100-120_BPM/          # Expansion 2: Mid-tempo drums
│   ├── Cache.json
│   ├── expansion-image.jpg
│   ├── [Previews]/
│   ├── Grooves/
│   └── Fills/
│
├── MIDI_Drums_120-140_BPM/          # Expansion 3: Fast drums
│   ├── Cache.json
│   ├── expansion-image.jpg
│   ├── [Previews]/
│   ├── Grooves/
│   └── Fills/
│
├── MIDI_Drums_140-180_BPM/          # Expansion 4: Very fast drums
│   ├── Cache.json
│   ├── expansion-image.jpg
│   ├── [Previews]/
│   ├── Grooves/
│   └── Fills/
│
├── MIDI_Bass_By_Key/                # Expansion 5: Bass patterns
│   ├── Cache.json
│   ├── expansion-image.jpg
│   ├── [Previews]/
│   ├── C_Major/
│   │   ├── 100BPM-Bass-C-01.mpcpattern
│   │   └── 120BPM-Bass-C-02.mpcpattern
│   ├── G_Major/
│   ├── D_Minor/
│   └── A_Minor/
│
├── MIDI_Melodic_By_Key/             # Expansion 6: Melodic patterns
│   ├── Cache.json
│   ├── expansion-image.jpg
│   ├── [Previews]/
│   ├── Piano/
│   │   ├── C_Major/
│   │   ├── G_Major/
│   │   └── D_Minor/
│   ├── Synth/
│   │   ├── C_Major/
│   │   └── A_Minor/
│   └── Keys/
│
├── MIDI_Loops_House_Techno/        # Expansion 7: Electronic loops
│   ├── Cache.json
│   ├── expansion-image.jpg
│   ├── [Previews]/
│   ├── 120-125_BPM/
│   └── 126-132_BPM/
│
└── MIDI_Loops_HipHop_RnB/          # Expansion 8: Hip-hop loops
    ├── Cache.json
    ├── expansion-image.jpg
    ├── [Previews]/
    ├── 080-095_BPM/
    └── 096-110_BPM/
```

### Why Multiple Expansions?

1. **Faster browsing** - Smaller, focused packs load faster on MPC hardware
2. **Workflow optimization** - Load only what you need for current project
3. **Genre-specific production** - Quick access to relevant patterns
4. **Better organization** - Clear categorization by BPM/key/type
5. **Hardware performance** - Smaller packs perform better on all MPC OS devices
6. **Universal compatibility** - Works on Force, MPC One, MPC Live, MPC X, etc.

---

## 🏷️ Naming Conventions

### .mpcpattern File Naming Format

**Pattern:** `{BPM}BPM-{Type}-{Key}-{Genre}-{ID}.mpcpattern`

**Examples:**
```
120BPM-Groove-Cmaj-House-001.mpcpattern
095BPM-Fill-Tom-HipHop-042.mpcpattern
128BPM-Bass-Amin-Techno-015.mpcpattern
110BPM-Loop-Piano-Jazz-023.mpcpattern
140BPM-Groove-Kick-DnB-078.mpcpattern
```

**Field Descriptions:**
- **BPM** (3 digits) - Essential for browsing (e.g., 095, 120, 140)
- **Type** - Groove, Fill, Loop, Bass, Lead, Chord, Arp
- **Key** (optional) - Cmaj, Amin, Gmaj, Dmin (for melodic patterns)
- **Genre** (optional) - House, Techno, HipHop, DnB, Jazz
- **ID** (3 digits) - Unique identifier within category (001-999)

### Expansion Naming Format

**Pattern:** `MIDI_{Category}_{Descriptor}`

**Examples:**
```
MIDI_Drums_080-100_BPM
MIDI_Drums_120-140_BPM
MIDI_Bass_By_Key
MIDI_Melodic_Piano_Jazz
MIDI_Loops_House_Techno
MIDI_Fills_Collection
```

---

## 🚀 Implementation Workflow

### Step 1: Export File Lists from Database
```bash
# Export drum patterns by BPM range
psql $DB_URL -c "\COPY (
  SELECT f.filepath, m.bpm, dp.pattern_type, dp.feel
  FROM files f
  JOIN musical_metadata m ON f.id = m.file_id
  LEFT JOIN drum_patterns dp ON f.id = dp.file_id
  JOIN file_tags ft ON f.id = ft.file_id
  JOIN tags t ON ft.tag_id = t.id
  WHERE f.num_tracks = 1
    AND t.name = 'drums'
    AND m.bpm BETWEEN 80 AND 100
  ORDER BY m.bpm
) TO '/tmp/drums_080-100.csv' WITH CSV HEADER;"

# Repeat for other BPM ranges: 100-120, 120-140, 140-180
```

### Step 2: Convert MIDI to .mpcpattern
```bash
# Build converter (already exists)
cargo build --release --bin midi_to_mpcpattern

# Convert in batches (example: drums 80-100 BPM)
#!/bin/bash
OUTPUT_DIR="/media/[DRIVE]/Expansions/MIDI_Drums_080-100_BPM/Grooves"
mkdir -p "$OUTPUT_DIR"

while IFS=',' read -r filepath bpm pattern_type feel; do
    # Generate output filename
    filename=$(basename "$filepath" .mid)
    output_name="${bpm}BPM-Groove-${pattern_type}-$(printf '%03d' $COUNTER).mpcpattern"

    # Convert
    ./target/release/midi_to_mpcpattern \
        "$filepath" \
        "$OUTPUT_DIR/$output_name"

    COUNTER=$((COUNTER + 1))

    if [ $((COUNTER % 100)) -eq 0 ]; then
        echo "Converted $COUNTER patterns..."
    fi
done < /tmp/drums_080-100.csv

echo "✅ Converted $COUNTER patterns to $OUTPUT_DIR"
```

### Step 3: Create Expansion Metadata

#### Cache.json Template
```json
{
  "name": "MIDI Drums 080-100 BPM",
  "version": "1.0.0",
  "author": "MIDI Software Center",
  "description": "2,500 drum patterns organized by BPM (80-100 range)",
  "category": "Drums",
  "tags": ["drums", "patterns", "80-100bpm", "grooves", "fills"],
  "file_count": 2500,
  "created": "2025-11-22",
  "preview_audio": "Previews/demo-080-100-drums.mp3"
}
```

#### Create expansion metadata script
```bash
#!/bin/bash
EXPANSION_DIR="/media/[DRIVE]/Expansions/MIDI_Drums_080-100_BPM"

# Create Cache.json
cat > "$EXPANSION_DIR/Cache.json" << 'EOF'
{
  "name": "MIDI Drums 080-100 BPM",
  "version": "1.0.0",
  "author": "MIDI Software Center",
  "description": "Drum patterns 80-100 BPM range",
  "category": "Drums"
}
EOF

# Create Previews folder
mkdir -p "$EXPANSION_DIR/[Previews]"

# Create placeholder image (512x512 recommended)
# You can generate this with ImageMagick or add custom artwork
convert -size 512x512 xc:black \
    -fill white -pointsize 48 -gravity center \
    -annotate +0+0 "MIDI Drums\n80-100 BPM" \
    "$EXPANSION_DIR/expansion-image.jpg"

echo "✅ Created expansion metadata for $EXPANSION_DIR"
```

### Step 4: Organize Patterns into Expansions

**Create organization script:**
```bash
#!/bin/bash
# organize_force_expansions.sh

BASE_DIR="/media/[DRIVE]/Expansions"
DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

# Function to create expansion structure
create_expansion() {
    local name=$1
    local dir="$BASE_DIR/$name"

    mkdir -p "$dir/Grooves"
    mkdir -p "$dir/Fills"
    mkdir -p "$dir/[Previews]"

    # Create Cache.json
    cat > "$dir/Cache.json" << EOF
{
  "name": "$name",
  "version": "1.0.0",
  "author": "MIDI Software Center",
  "category": "Patterns"
}
EOF

    # Create placeholder image
    convert -size 512x512 xc:black \
        -fill white -pointsize 40 -gravity center \
        -annotate +0+0 "${name//_/\\n}" \
        "$dir/expansion-image.jpg" 2>/dev/null || echo "ImageMagick not found, skip image"

    echo "✅ Created expansion: $name"
}

# Create all drum expansions
create_expansion "MIDI_Drums_080-100_BPM"
create_expansion "MIDI_Drums_100-120_BPM"
create_expansion "MIDI_Drums_120-140_BPM"
create_expansion "MIDI_Drums_140-180_BPM"
create_expansion "MIDI_Bass_By_Key"
create_expansion "MIDI_Melodic_By_Key"
create_expansion "MIDI_Loops_House_Techno"
create_expansion "MIDI_Loops_HipHop_RnB"

echo ""
echo "🎉 All expansion folders created!"
echo "Next: Run conversion scripts to populate with .mpcpattern files"
```

### Step 5: Batch Conversion Script

**Complete batch converter:**
```bash
#!/bin/bash
# batch_convert_to_mpcpattern.sh

DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"
CONVERTER="./target/release/midi_to_mpcpattern"
BASE_DIR="/media/[DRIVE]/Expansions"

# Function to convert category
convert_category() {
    local category=$1
    local bpm_min=$2
    local bpm_max=$3
    local output_dir=$4
    local limit=$5

    echo "🔄 Converting $category ($bpm_min-$bpm_max BPM)..."

    # Export file list from database
    psql "$DB_URL" -t -A -F',' -c "
        SELECT f.filepath, m.bpm
        FROM files f
        JOIN musical_metadata m ON f.id = m.file_id
        JOIN file_tags ft ON f.id = ft.file_id
        JOIN tags t ON ft.tag_id = t.id
        WHERE f.num_tracks = 1
          AND t.name = '$category'
          AND m.bpm BETWEEN $bpm_min AND $bpm_max
        ORDER BY m.bpm
        LIMIT $limit;
    " > "/tmp/${category}_${bpm_min}-${bpm_max}.csv"

    # Convert each file
    counter=1
    while IFS=',' read -r filepath bpm; do
        if [ -f "$filepath" ]; then
            output_name="${bpm}BPM-${category}-$(printf '%04d' $counter).mpcpattern"

            if "$CONVERTER" "$filepath" "$output_dir/$output_name" 2>/dev/null; then
                counter=$((counter + 1))

                if [ $((counter % 100)) -eq 0 ]; then
                    echo "  Converted $counter patterns..."
                fi
            fi
        fi
    done < "/tmp/${category}_${bpm_min}-${bpm_max}.csv"

    echo "✅ Converted $counter $category patterns ($bpm_min-$bpm_max BPM)"
    echo ""
}

# Convert drums by BPM ranges
convert_category "drums" 80 100 "$BASE_DIR/MIDI_Drums_080-100_BPM/Grooves" 5000
convert_category "drums" 100 120 "$BASE_DIR/MIDI_Drums_100-120_BPM/Grooves" 5000
convert_category "drums" 120 140 "$BASE_DIR/MIDI_Drums_120-140_BPM/Grooves" 5000
convert_category "drums" 140 180 "$BASE_DIR/MIDI_Drums_140-180_BPM/Grooves" 5000

# Convert bass patterns
convert_category "bass" 90 150 "$BASE_DIR/MIDI_Bass_By_Key" 5000

# Convert melodic patterns
convert_category "piano" 100 140 "$BASE_DIR/MIDI_Melodic_By_Key/Piano" 3000
convert_category "synth" 100 140 "$BASE_DIR/MIDI_Melodic_By_Key/Synth" 3000

# Convert loops
convert_category "loop" 120 132 "$BASE_DIR/MIDI_Loops_House_Techno" 5000
convert_category "loop" 80 110 "$BASE_DIR/MIDI_Loops_HipHop_RnB" 5000

echo ""
echo "🎉 Batch conversion complete!"
echo "📊 Total expansions created: 8"
echo "📁 Location: $BASE_DIR"
```

---

## 📊 Estimated Results

### Expansion Pack Sizes
| Expansion | Patterns | Size (est.) | Description |
|-----------|----------|-------------|-------------|
| Drums 80-100 BPM | 5,000 | ~25 MB | Slow/mid tempo grooves & fills |
| Drums 100-120 BPM | 5,000 | ~25 MB | House, funk, hip-hop grooves |
| Drums 120-140 BPM | 5,000 | ~25 MB | Techno, trap, house grooves |
| Drums 140-180 BPM | 5,000 | ~25 MB | DnB, jungle, fast patterns |
| Bass By Key | 5,000 | ~25 MB | Bass lines in all keys |
| Melodic By Key | 6,000 | ~30 MB | Piano, synth, keys patterns |
| Loops House/Techno | 5,000 | ~25 MB | Electronic loops 120-132 BPM |
| Loops Hip-Hop/R&B | 5,000 | ~25 MB | Hip-hop loops 80-110 BPM |
| **TOTAL** | **41,000** | **~205 MB** | **8 expansion packs** |

### Performance Estimates
- **Conversion time:** ~2-4 hours (41,000 files @ 200-400 files/min)
- **Organization time:** ~30 minutes (folder creation, metadata)
- **Testing time:** ~1 hour (load expansions on Force, verify browsing)
- **Total time:** **3.5-5.5 hours** for complete workflow

---

## 🎯 Optimization Strategies

### 1. Smart Selection (Reduce 2.2M → 41K)
- Filter by BPM range (valid patterns only)
- Filter by duration (1-30 seconds)
- Filter by quality (exclude corrupted files)
- Prioritize tagged/analyzed files
- Balance across categories

### 2. BPM-First Organization
- Primary sort by BPM (fastest workflow on Force)
- Secondary sort by key/type
- Matches how producers work (start with tempo)

### 3. Genre-Specific Packs
- Electronic (House/Techno 120-140)
- Hip-Hop (80-110)
- DnB (140-180)
- Allows loading relevant content per project

### 4. Flat Structure Within Packs
- Keeps browsing fast on hardware
- Max 1-2 subfolder levels
- Descriptive filenames do the organizing

---

## ✅ Success Criteria

1. **All patterns playable** - All MPC OS devices can load and play every .mpcpattern
2. **Proper categorization** - Patterns sorted by BPM/key/type
3. **Fast browsing** - Expansion packs load quickly on all MPC hardware
4. **Clear naming** - Files easy to identify from name alone
5. **Complete metadata** - Cache.json and images for all packs
6. **Organized structure** - Logical folder hierarchy
7. **Size optimized** - Packs sized for optimal MPC OS performance (<500 MB each)
8. **Universal compatibility** - Tested on Force, MPC One, and other MPC OS devices

---

## 🚀 Quick Start Commands

```bash
# 1. Create expansion folders
./scripts/organize_force_expansions.sh

# 2. Build converter
cargo build --release --bin midi_to_mpcpattern

# 3. Run batch conversion
./scripts/batch_convert_to_mpcpattern.sh

# 4. Verify results
find /media/[DRIVE]/Expansions -name "*.mpcpattern" | wc -l

# 5. Copy to MPC OS device
# (Manual: plug in Force/MPC One/MPC Live/etc, copy Expansions folder to drive root)
```

---

## 📋 Next Steps

1. ✅ Review and approve this plan
2. ⏳ Determine target drive location (Force/MPC device drive)
3. ⏳ Create organization scripts
4. ⏳ Run batch conversion (3.5-5.5 hours)
5. ⏳ Test on MPC OS hardware (Force, MPC One, or other device)
6. ⏳ Iterate based on workflow feedback

---

## 🔗 Related Documentation

- `MPCPATTERN-FORMAT-SPECIFICATION.md` - Technical format details
- `MPCPATTERN-ORGANIZATION-BEST-PRACTICES.md` - Commercial pack analysis
- `pipeline/src-tauri/src/bin/midi_to_mpcpattern.rs` - Converter source code
- `DATABASE-INSTRUMENT-ORGANIZATION-GUIDE.md` - Database query examples

---

**Ready to proceed with implementation?** This plan converts 41,000 carefully selected single-track MIDI files into 8 themed MPC OS expansion packs optimized for production workflow on all Akai MPC OS devices (Force, MPC One, MPC Live/II, MPC X, MPC Key 61).

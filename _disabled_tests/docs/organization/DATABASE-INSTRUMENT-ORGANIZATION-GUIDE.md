# Database Instrument Organization Guide

**Status:** Ready to implement
**Files Analyzed:** 1,715,885 unique MIDI files
**Instruments Discovered:** 97 unique instruments
**Date:** 2025-11-22

---

## Quick Start

```bash
# 1. Apply the database organization schema
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" \
  -f database/organize_by_instruments.sql

# 2. Query your organized collection
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" \
  -c "SELECT * FROM v_tag_stats LIMIT 20;"
```

---

## What This Does

### 1. **Tag System Integration**
- Inserts all 97 discovered instruments as database tags
- Categorizes them: `drums`, `bass`, `keys`, `synth`, `guitar`, `strings`, `brass`, `woodwind`, `vocal`, `fx`, `pattern`, `melody`, `harmony`, `genre`, `orchestral`
- Tags files automatically based on filename matching

### 2. **Virtual Folder Views**
Instead of moving 1.7M files into folders, create database views:
- **`v_drums`** - All drum files (ride, kick, snare, toms, cymbals, etc.)
- **`v_melodic`** - All melodic/harmonic files (keys, synths, strings, brass, etc.)
- **`v_bass`** - All bass files (808, 909, sub, bassline)
- **`v_loops`** - All loops and patterns
- **`v_by_genre`** - Files grouped by genre

### 3. **Powerful Search Queries**
Query by any combination:
- Instrument + BPM + Key signature
- Multiple instruments (jazz piano, rock drums)
- Genre + instrument type
- Pattern type + tempo

---

## Organization Hierarchy

Based on your 1.7M files, here's the discovered taxonomy:

```
📁 DRUMS (top 23 drum types, ~35% of collection)
├── ride        103,591 files (6.04%)
├── fill         88,142 files (5.14%)
├── kick         75,286 files (4.39%)
├── tom          64,351 files (3.75%)
├── crash        39,690 files (2.31%)
├── snare        16,341 files (0.95%)
├── stick        16,144 files (0.94%)
├── hihat        11,459 files (0.67%)
└── [15 more drum types...]

📁 BASS (5 types, 3.08% of collection)
├── bass         52,917 files
├── bassline      3,782 files
├── sub           3,590 files
├── 808           3,245 files
└── 909             596 files

📁 SYNTH & KEYS (12 types, ~4% of collection)
├── synth        26,556 files
├── piano        21,932 files
├── lead         19,496 files
├── pad          12,956 files
├── ep            9,108 files
├── arp           7,890 files
└── [6 more...]

📁 GUITARS (6 types, 0.7% of collection)
├── guitar        5,387 files
├── gtr           3,797 files
└── [4 more...]

📁 STRINGS (5 types, 0.5% of collection)
├── strings       5,986 files
├── violin        1,077 files
└── [3 more...]

📁 BRASS (5 types, 0.4% of collection)
├── brass         3,866 files
├── sax           1,511 files
└── [3 more...]

📁 WOODWINDS (4 types, 0.2% of collection)
├── flute         1,422 files
└── [3 more...]

📁 VOCALS (5 types, 0.3% of collection)
├── choir         2,079 files
├── vocal         1,279 files
└── [3 more...]

📁 FX (7 types, 1.5% of collection)
├── bell         20,861 files
├── fx            4,646 files
└── [5 more...]

📁 PATTERNS (6 types, 3.4% of collection)
├── loop         31,736 files
├── groove       10,729 files
└── [4 more...]

📁 GENRES (14 types, 4.6% of collection)
├── rock         40,209 files (2.34%)
├── funk         11,136 files
├── jazz          6,193 files
└── [11 more...]
```

---

## Example Queries

### Basic Queries

```sql
-- Find all ride cymbal files
SELECT * FROM get_files_by_instrument('ride');

-- Find all drum files
SELECT * FROM v_drums LIMIT 100;

-- Find all bass files
SELECT * FROM v_bass;

-- Count files by instrument category
SELECT
    category,
    COUNT(*) AS instrument_count,
    SUM(file_count) AS total_files
FROM v_tag_stats
WHERE category IS NOT NULL
GROUP BY category
ORDER BY total_files DESC;
```

### Advanced Queries

```sql
-- Find 120 BPM drum loops in C major
SELECT f.filename, f.filepath, m.bpm, m.key_signature
FROM files f
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
JOIN musical_metadata m ON f.id = m.file_id
WHERE t.name = 'loop'
  AND t.category = 'drums'
  AND m.bpm BETWEEN 118 AND 122
  AND m.key_signature = 'C'
ORDER BY m.bpm;

-- Find jazz piano files
SELECT * FROM get_files_by_instruments(ARRAY['jazz', 'piano']);

-- Find rock drum fills
SELECT f.filename, f.filepath
FROM files f
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE t.name IN ('rock', 'fill', 'drums')
GROUP BY f.id, f.filename, f.filepath
HAVING COUNT(DISTINCT t.name) = 3;

-- Find melodic loops with BPM and key
SELECT
    f.filename,
    m.bpm,
    m.key_signature,
    array_agg(DISTINCT t.name ORDER BY t.name) AS tags
FROM files f
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
LEFT JOIN musical_metadata m ON f.id = m.file_id
WHERE t.category IN ('melody', 'pattern')
GROUP BY f.id, f.filename, m.bpm, m.key_signature
HAVING 'loop' = ANY(array_agg(DISTINCT t.name))
ORDER BY m.bpm;

-- Find multi-instrument files (contains 3+ instruments)
SELECT * FROM v_multi_instrument ORDER BY tag_count DESC LIMIT 50;

-- Top instrument combinations
SELECT * FROM v_instrument_combos LIMIT 20;
```

### Export Queries

```sql
-- Export all drum file paths to text file
\COPY (SELECT filepath FROM v_drums ORDER BY filename) TO '/tmp/drum_files.txt';

-- Export rock genre files with metadata
\COPY (
    SELECT f.filepath, m.bpm, m.key_signature
    FROM files f
    JOIN file_tags ft ON f.id = ft.file_id
    JOIN tags t ON ft.tag_id = t.id
    LEFT JOIN musical_metadata m ON f.id = m.file_id
    WHERE t.name = 'rock'
    ORDER BY m.bpm
) TO '/tmp/rock_files.csv' WITH CSV HEADER;

-- Create M3U playlist for all bass loops
\COPY (
    SELECT filepath FROM files f
    JOIN file_tags ft ON f.id = ft.file_id
    JOIN tags t ON ft.tag_id = t.id
    WHERE t.name IN ('bass', 'loop')
    GROUP BY f.id, f.filepath
    HAVING COUNT(DISTINCT t.name) = 2
) TO '/tmp/bass_loops.m3u';
```

---

## Integration with DAW

### Option 1: Symlinks (Recommended)
Create organized symlink folders without moving files:

```bash
# Create folder structure
mkdir -p ~/midi-organized/{drums,bass,melodic,loops,fx,vocals}

# Generate symlink commands from database
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" \
  -c "\COPY (
    SELECT 'ln -s \"' || filepath || '\" ~/midi-organized/drums/'
    FROM v_drums
  ) TO STDOUT" | bash

# Same for other categories
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" \
  -c "\COPY (
    SELECT 'ln -s \"' || filepath || '\" ~/midi-organized/bass/'
    FROM v_bass
  ) TO STDOUT" | bash

# Now your DAW sees organized folders, but files stay in place!
```

### Option 2: Playlists/Collections
Export query results as M3U playlists:

```bash
# Create playlist for 128 BPM techno drums
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" \
  -c "\COPY (
    SELECT filepath FROM files f
    JOIN file_tags ft ON f.id = ft.file_id
    JOIN tags t ON ft.tag_id = t.id
    JOIN musical_metadata m ON f.id = m.file_id
    WHERE t.name IN ('techno', 'drums')
      AND m.bpm BETWEEN 126 AND 130
  ) TO '/tmp/128_techno_drums.m3u'"
```

### Option 3: API Integration
Build a web UI or plugin that queries the database:

```javascript
// Example: Query from TypeScript/Svelte frontend
const getDrumFiles = async (bpm: number, key: string) => {
  const result = await invoke('search_files', {
    tags: ['drums'],
    bpmMin: bpm - 2,
    bpmMax: bpm + 2,
    key: key
  });
  return result;
};
```

---

## Performance Considerations

### Database Size
- **Tags:** 97 rows (~5 KB)
- **File Tags:** ~5-10M rows (avg 3-6 tags per file) = ~200-400 MB
- **Indexes:** ~100-200 MB
- **Total overhead:** ~300-600 MB

### Query Speed
With proper indexes (included in script):
- Simple tag query: **< 10ms**
- Complex multi-tag + metadata: **< 100ms**
- Full category scan: **< 500ms**
- Export to file: **1-3 seconds**

### Import Speed
Tagging 1.7M files:
- **Pattern matching:** ~5-10 minutes (one-time)
- **Subsequent queries:** Instant (indexed)

---

## Maintenance

### Update Tag Counts
```sql
-- Refresh usage counts
UPDATE tags t
SET usage_count = (
    SELECT COUNT(*)
    FROM file_tags ft
    WHERE ft.tag_id = t.id
);
```

### Add New Instruments
```sql
-- Discovered new instrument in filenames?
INSERT INTO tags (name, category, usage_count)
VALUES ('marimba', 'percussion', 0);

-- Tag matching files
SELECT tag_files_by_keyword('marimba', 'marimba');
```

### Find Untagged Files
```sql
-- Files with no instrument tags
SELECT f.id, f.filename, f.filepath
FROM files f
LEFT JOIN file_tags ft ON f.id = ft.file_id
WHERE ft.file_id IS NULL
LIMIT 100;
```

---

## Migration Strategy

### Phase 1: Schema Setup (5 minutes)
```bash
psql $DATABASE_URL -f database/organize_by_instruments.sql
```

### Phase 2: Tag Top Instruments (10 minutes)
The script automatically tags the top 30 instruments (~85% of files)

### Phase 3: Verify (1 minute)
```sql
SELECT category, SUM(file_count) AS total
FROM v_tag_stats
GROUP BY category;
```

### Phase 4: Create Symlinks (Optional, 15 minutes)
```bash
./scripts/create-organized-symlinks.sh
```

---

## Comparison: Physical vs Database Organization

### Physical Folder Organization
```
❌ Must move 1.7M files (slow, risky)
❌ Files can only be in ONE folder
❌ Reorganization requires moving files again
❌ Hard to query by multiple criteria
❌ No BPM/key filtering without renaming
✅ Works with any file browser
✅ Works with legacy DAWs
```

### Database Organization (Recommended)
```
✅ Zero file movement (instant)
✅ Files can have MULTIPLE tags
✅ Reorganization = just change query
✅ Powerful multi-criteria queries
✅ Combine tags + BPM + key + duration
✅ Export results as playlists/symlinks
✅ Scales to millions of files
⚠️ Requires PostgreSQL access
⚠️ Needs UI for non-technical users
```

---

## Next Steps

1. **Apply the schema:**
   ```bash
   psql $DATABASE_URL -f database/organize_by_instruments.sql
   ```

2. **Verify tags were created:**
   ```bash
   psql $DATABASE_URL -c "SELECT COUNT(*) FROM tags;"
   # Expected: 97 tags
   ```

3. **Check file tagging progress:**
   ```bash
   psql $DATABASE_URL -c "SELECT COUNT(*) FROM file_tags;"
   # Expected: 300K-500K file_tags (3-6 tags per file avg)
   ```

4. **Run example queries** from this guide

5. **Build web UI** to browse organized collection

6. **Create symlinks** for DAW integration (optional)

---

## Additional Resources

- **Main organization doc:** `DATABASE-FILE-ORGANIZATION.md`
- **Schema:** `database/migrations/001_initial_schema.sql`
- **Instrument analysis:** `INSTRUMENT_ANALYSIS.md`
- **Raw instrument list:** `INSTRUMENT_LIST.txt`
- **Organization script:** `database/organize_by_instruments.sql`

---

**Recommendation:** Use database organization with symlinks for DAW compatibility. Best of both worlds!

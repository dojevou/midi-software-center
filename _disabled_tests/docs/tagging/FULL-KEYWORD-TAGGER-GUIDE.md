# Full Keyword Tagger Guide

**Created:** November 22, 2025
**Purpose:** Tag all 2.4M MIDI files using actual folder/filename keywords from your collection

---

## The Problem with the Original Tagger

**Original Tagger:** `fast_tagger.rs`
- Used only 1,640 curated keywords from `/tmp/master_tag_list.txt`
- Result: 2.1 tags/file average (low coverage)
- Missing: 99.7% of actual keywords in your collection!

---

## The New Full Keyword Tagger

**New Tagger:** `fast_tagger_full.rs`
- Uses **ALL** actual keywords from your collection's folders and filenames
- Sources: 3 raw data files with full keyword lists
- Expected result: **5-15 tags/file** (much better coverage!)

---

## Keyword Sources

### 1. Grandparent Folder Keywords
**File:** `/tmp/grandparent_folders.txt`
**Count:** 10,266 unique folder names (2 levels up from files)
**Top Examples:**
- Variation-0 (83,179 files)
- Ultra New Midi Pack (66,513 files)
- 4-4 Grooves (26,952 files)
- Grooves (26,201 files)
- Fills (13,301 files)

### 2. Parent Folder Keywords
**File:** `/tmp/parent_folders.txt`
**Count:** 133,231 unique folder names (immediate containing directories)
**Top Examples:**
- splits (137,451 files)
- Straight (22,935 files)
- EDM Midi pack (13,408 files)
- Fills (11,311 files)

### 3. Filename Keywords
**File:** `/tmp/filenames.txt`
**Count:** 524,894 unique filenames (without extensions)
**Top Examples:**
- track_01 (387,261 files - 22.9% of all files!)
- track_02 (15,228 files)
- Variation_01 (9,300 files)
- bass (894 files)
- kick (321 files)

**Total Potential Keywords:** 10,266 + 133,231 + 524,894 = **668,391 raw keywords**

---

## How the New Tagger Works

### Keyword Normalization
For each folder/filename keyword:
1. Convert to lowercase
2. Replace delimiters (`_`, `-`, `(`, `)`, etc.) with spaces
3. Split into individual words
4. Filter out:
   - Words shorter than 2 characters
   - Pure numbers (e.g., "01", "123")
   - Noise words ("the", "and", "midi", "file", etc.)
   - Words longer than 50 characters

**Example:**
```
Folder: "4-4 Grooves"
→ Keywords: ["grooves"]

Filename: "Oakland3013_15.mid"
→ Keywords: ["oakland3013"]

Folder: "EDM Midi pack"
→ Keywords: ["edm"] (filtered "midi", "pack" as noise)
```

### Minimum Frequency Filter
By default, skips keywords that appear less than 5 times (`--min-frequency 5`)
- Reduces noise from one-off folder names
- Still captures ~50,000-100,000 unique keywords
- Adjust with `--min-frequency` flag

---

## Expected Results

### Before (Original Tagger)
- Tags per file: 2.1 average
- Total tags: 1,640
- Coverage: Low (~0.3% of potential keywords)

### After (Full Keyword Tagger)
- Tags per file: **5-15 average** (estimated)
- Total tags: **50,000-100,000** unique keywords
- Coverage: High (captures actual collection structure)

### Example File Tagging
```
File: ./archives/Linear Drums_extracted/Linear Drums/Linear Drums/94_Oakland_3013/Oakland3013_15.mid

Tags extracted:
- Grandparent: "linear", "drums"
- Parent: "oakland"
- Filename: "oakland3013"

Total tags for this file: 4 tags
```

---

## Usage

### Basic Usage
```bash
./target/release/fast_tagger_full
```

### With Custom Options
```bash
./target/release/fast_tagger_full \
  --grandparent-file /tmp/grandparent_folders.txt \
  --parent-file /tmp/parent_folders.txt \
  --filename-file /tmp/filenames.txt \
  --min-frequency 5 \
  --workers 16 \
  --batch-size 10000 \
  --chunk-size 5000
```

### Parameters
- `--grandparent-file`: Path to grandparent folders file (default: `/tmp/grandparent_folders.txt`)
- `--parent-file`: Path to parent folders file (default: `/tmp/parent_folders.txt`)
- `--filename-file`: Path to filenames file (default: `/tmp/filenames.txt`)
- `--min-frequency`: Minimum keyword frequency threshold (default: 5)
- `--workers`: Number of parallel workers (default: 16)
- `--batch-size`: Database batch insert size (default: 10000)
- `--chunk-size`: File processing chunk size (default: 5000)
- `--database-url`: PostgreSQL connection string

---

## Processing Steps

### Step 1: Load Keywords (1-2 minutes)
```
📚 Loading ALL keywords from collection...
1️⃣ Grandparent Folder Keywords: 10,266 → ~8,000 normalized
2️⃣ Parent Folder Keywords: 133,231 → ~60,000 normalized
3️⃣ Filename Keywords: 524,894 → ~80,000 normalized
✅ Total unique keywords loaded: ~100,000
```

### Step 2: Insert Tags to Database (5-10 minutes)
```
💾 Inserting keywords as tags into database...
  Batch 10/20 | Tags: 50,000 | Elapsed: 120s
  Batch 20/20 | Tags: 100,000 | Elapsed: 240s
✅ Inserted 100,000 tags in 240s
```

### Step 3: Process Files (40-60 minutes)
```
Processing chunk 485/485 (5000 files)...
⚙️ Processing files for tag matching...
✅ Matched 52,843 tag relationships
💾 Inserting 52,843 tag relationships...
Progress: 2420096/2420096 (100.0%) | Tagged: 2420096 | Total tags: 15,234,567 (6.3 avg)
```

---

## Performance Estimates

**Total Time:** 45-70 minutes
- Step 1 (Load keywords): 1-2 min
- Step 2 (Insert tags): 5-10 min
- Step 3 (Tag files): 40-60 min

**Processing Rate:** 600-800 files/sec
**Database Size Increase:** +500 MB - 1 GB (tag relationships)

---

## Comparison to Original

| Metric | Original Tagger | Full Keyword Tagger |
|--------|----------------|---------------------|
| **Keywords** | 1,640 | ~100,000 |
| **Tags/File** | 2.1 avg | 5-15 avg (estimated) |
| **Coverage** | Low (curated only) | High (actual collection) |
| **Processing Time** | ~30 min | ~60 min |
| **Database Growth** | +200 MB | +1 GB |

---

## Benefits

### Better Search
```sql
-- Find all "groove" files across all levels
SELECT f.* FROM files f
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE t.name = 'groove';

-- Find drum files from "Linear Drums" collection
SELECT f.* FROM files f
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE t.name IN ('linear', 'drums');
```

### Multi-Level Organization
Files inherit tags from:
1. Grandparent folder (collection/archive name)
2. Parent folder (category/style)
3. Filename (specific identifier)

### Example Query Results
```sql
-- Before: "groove" → 2,000 results (only filenames with "groove")
-- After: "groove" → 26,000+ results (Grooves folder + groove filenames)
```

---

## Monitoring Progress

### Watch Log
```bash
# In separate terminal
tail -f /tmp/full_tagger_log.txt
```

### Check Database Stats
```sql
SELECT
  COUNT(*) as total_tags,
  COUNT(DISTINCT name) as unique_tags
FROM tags
WHERE category = 'auto_extracted';

SELECT
  COUNT(DISTINCT file_id) as files_tagged,
  COUNT(*) as total_tag_relationships,
  ROUND(AVG(tag_count), 2) as avg_tags_per_file
FROM (
  SELECT file_id, COUNT(*) as tag_count
  FROM file_tags
  GROUP BY file_id
) sub;
```

---

## Running Now

**Command:**
```bash
nohup ./target/release/fast_tagger_full > /tmp/full_tagger_log.txt 2>&1 &
```

**Monitor:**
```bash
tail -f /tmp/full_tagger_log.txt
```

---

## Next Steps After Completion

1. **Verify results:**
   ```sql
   SELECT COUNT(DISTINCT file_id) FROM file_tags;
   SELECT AVG(tag_count) FROM (
     SELECT COUNT(*) as tag_count FROM file_tags GROUP BY file_id
   ) sub;
   ```

2. **Test search queries** with new tags
3. **Update CLAUDE.md** with new tagging stats
4. **Create views** for common tag combinations

---

**Ready to run!** This will provide MUCH better tag coverage of your entire collection.

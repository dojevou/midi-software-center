# Fast Multi-Level Tagging Strategy

**Goal:** Tag 1.79M files with keywords from folders and filenames in **minutes instead of hours**

---

## Current Bottleneck

**Sequential Tagger (`tag_files_sequential`):**
- Time: ~8 hours (97 keywords × 5 min/keyword)
- Method: 97 separate queries, each doing full table scan
- Query: `SELECT * FROM files WHERE LOWER(filename) LIKE '%keyword%'`
- Problems:
  - ❌ 97 full table scans (1.79M rows each)
  - ❌ LIKE '%keyword%' cannot use indexes
  - ❌ Sequential processing (one keyword at a time)
  - ❌ No batch inserts
  - ❌ Doesn't use grandparent/parent folder keywords

---

## Optimized Strategy

### Three-Level Keyword Matching

**Level 1: Grandparent Folder** (2 levels up)
- Example: `./midi-library/archives/Ultra New Midi Pack/EDM/House/track.mid`
- Grandparent: `Ultra New Midi Pack`
- Extract: "ultra", "new", "midi", "pack"

**Level 2: Parent Folder** (1 level up)
- Parent: `House`
- Extract: "house"

**Level 3: Filename** (file itself, no extension)
- Filename: `120bpm_bass_loop.mid`
- Extract: "120bpm", "bass", "loop"

### Algorithm: Single-Pass Multi-Level Extraction

```rust
for each file in database {
    // Parse path once
    let (grandparent, parent, filename) = parse_path(file.filepath);

    // Extract keywords from all three levels
    let keywords = HashSet::new();
    keywords.extend(extract_keywords(grandparent, grandparent_keywords));
    keywords.extend(extract_keywords(parent, parent_keywords));
    keywords.extend(extract_keywords(filename, filename_keywords));

    // Batch collect tags
    for keyword in keywords {
        batch.push((file_id, tag_id[keyword]));
    }

    // Batch insert when batch is full
    if batch.len() >= 10000 {
        insert_batch(&batch);
        batch.clear();
    }
}
```

### Performance Optimizations

**1. Pre-load Keywords into Memory**
```rust
// Load from extracted lists
let grandparent_keywords: HashSet<String> = load_keywords("/tmp/grandparent_folders.txt");
let parent_keywords: HashSet<String> = load_keywords("/tmp/parent_folders.txt");
let filename_keywords: HashSet<String> = load_keywords("/tmp/filenames.txt");

// O(1) lookups instead of database queries
```

**2. Keyword Normalization**
```rust
fn normalize_keyword(s: &str) -> Vec<String> {
    s.to_lowercase()
     .replace("_", " ")
     .replace("-", " ")
     .split_whitespace()
     .map(|w| w.trim())
     .filter(|w| w.len() >= 3)  // Skip very short words
     .map(String::from)
     .collect()
}

// "120bpm_Bass_Loop" -> ["120bpm", "bass", "loop"]
// "4-4 Grooves" -> ["grooves"]  (skip "4-4")
```

**3. Database Indexes**
```sql
-- Add indexes for faster queries
CREATE INDEX CONCURRENTLY idx_files_filepath_lower ON files (LOWER(filepath));
CREATE INDEX CONCURRENTLY idx_files_filename_lower ON files (LOWER(filename));
CREATE INDEX CONCURRENTLY idx_tags_name_lower ON tags (LOWER(name));
CREATE INDEX CONCURRENTLY idx_file_tags_composite ON file_tags (file_id, tag_id);

-- Prevent duplicate tags
CREATE UNIQUE INDEX CONCURRENTLY idx_file_tags_unique ON file_tags (file_id, tag_id);
```

**4. Batch Processing**
```rust
const BATCH_SIZE: usize = 10000;  // Insert 10k tags at once
const FILE_CHUNK_SIZE: usize = 5000;  // Process 5k files per chunk
const NUM_WORKERS: usize = 16;  // Parallel workers
```

**5. Skip Already-Tagged Files**
```sql
-- Only process untagged or partially-tagged files
SELECT f.id, f.filepath, f.filename
FROM files f
LEFT JOIN file_tags ft ON f.id = ft.file_id
GROUP BY f.id
HAVING COUNT(ft.tag_id) < 3  -- Skip files with 3+ tags already
```

---

## Implementation Plan

### Step 1: Create Curated Tag Dictionary

Instead of using ALL 524,894 filenames as tags, curate a meaningful subset:

```bash
# Extract high-frequency, meaningful keywords
# Minimum frequency: 100 files
# Exclude: numbers, track_01, variation_01, etc.

# From grandparent folders (10,266 → ~500 tags)
grep -v "Variation-" /tmp/grandparent_folders.txt | \
  awk '$1 >= 100 {for(i=2;i<=NF;i++) print $i}' > /tmp/curated_grandparent_tags.txt

# From parent folders (133,231 → ~2,000 tags)
grep -v "Variation-" /tmp/parent_folders.txt | \
  awk '$1 >= 100 {for(i=2;i<=NF;i++) print $i}' > /tmp/curated_parent_tags.txt

# From filenames (524,894 → ~1,000 tags)
grep -v -E "track_|Variation_|Beat-[0-9]" /tmp/filenames.txt | \
  awk '$1 >= 100 {for(i=2;i<=NF;i++) print $i}' > /tmp/curated_filename_tags.txt
```

**Result:** ~3,500 curated tags instead of 524,894

### Step 2: Create Tag Dictionary Binary

```bash
# Binary: pipeline/src-tauri/src/bin/create_tag_dictionary.rs
# Purpose: Load curated keywords, normalize, insert into tags table

cargo build --release --bin create_tag_dictionary
DATABASE_URL="..." target/release/create_tag_dictionary
```

**Features:**
- Load curated keywords from three lists
- Normalize keywords (lowercase, split on delimiters)
- Remove duplicates
- Insert into `tags` table with categories
- Categories: "grandparent_folder", "parent_folder", "filename"

### Step 3: Build Fast Multi-Level Tagger

```bash
# Binary: pipeline/src-tauri/src/bin/fast_multi_level_tagger.rs
# Purpose: Single-pass extraction of ALL keywords per file

cargo build --release --bin fast_multi_level_tagger
DATABASE_URL="..." target/release/fast_multi_level_tagger \
  --workers 16 \
  --batch-size 10000 \
  --chunk-size 5000
```

**Features:**
- Load ALL tags into HashSets (in-memory, O(1) lookup)
- Query files in chunks (5,000 at a time)
- For each file: parse path, extract grandparent/parent/filename
- Match extracted keywords against tag HashSets
- Batch insert file_tags (10,000 at a time)
- Progress reporting every 10k files
- Parallel processing (16 workers)

### Step 4: Add Database Indexes

```bash
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" << 'EOF'
-- Add indexes CONCURRENTLY (won't block other operations)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_files_filepath_lower
  ON files (LOWER(filepath));

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_files_filename_lower
  ON files (LOWER(filename));

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tags_name_lower
  ON tags (LOWER(name));

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_file_tags_composite
  ON file_tags (file_id, tag_id);

CREATE UNIQUE INDEX CONCURRENTLY IF NOT EXISTS idx_file_tags_unique
  ON file_tags (file_id, tag_id);
EOF
```

---

## Expected Performance

### Current Sequential Tagger
- **Time:** ~8 hours
- **Tags per file:** ~2-3 (limited by slowness)
- **Files tagged:** 270,187 (15.75%)
- **Method:** 97 separate full table scans

### New Multi-Level Tagger
- **Time:** ~5-15 minutes (estimated)
- **Tags per file:** ~5-10 (grandparent + parent + filename)
- **Files tagged:** 1,790,269 (100%)
- **Method:** Single pass, O(1) keyword lookups, batch inserts

### Performance Breakdown
```
Load tags into memory:          ~1-2 seconds (3,500 tags)
Query files (chunked):          ~30 seconds (1.79M rows)
Parse paths + match keywords:   ~2-3 minutes (1.79M × 3 levels)
Batch insert tags:              ~2-5 minutes (5-10M inserts)
Total:                          ~5-10 minutes
```

**Speed improvement:** 48-96x faster (8 hours → 5-10 minutes)

---

## Execution Steps

```bash
# 1. Add database indexes (run FIRST, takes 2-5 minutes)
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" \
  -f /tmp/add_tagging_indexes.sql

# 2. Create curated tag dictionary
./scripts/create-curated-tags.sh

# 3. Build binaries
cd /home/dojevou/projects/midi-software-center
cargo build --release --bin create_tag_dictionary
cargo build --release --bin fast_multi_level_tagger

# 4. Insert tags into database
DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library" \
  target/release/create_tag_dictionary

# 5. Run fast tagger
DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library" \
  target/release/fast_multi_level_tagger \
  --workers 16 \
  --batch-size 10000 \
  --chunk-size 5000 \
  --log /tmp/fast_tagging.log

# 6. Verify results
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -c "
SELECT
  COUNT(DISTINCT file_id) as files_tagged,
  COUNT(*) as total_tags,
  ROUND(AVG(tags_per_file), 2) as avg_tags_per_file
FROM (
  SELECT file_id, COUNT(*) as tags_per_file
  FROM file_tags
  GROUP BY file_id
) t;"
```

---

## Advantages Over Current Approach

| Feature | Current (Sequential) | New (Multi-Level) |
|:--------|:---------------------|:------------------|
| **Time** | ~8 hours | ~5-10 minutes |
| **Table Scans** | 97 full scans | 1 single scan |
| **Tags per File** | ~2-3 | ~5-10 |
| **Coverage** | 15.75% of files | 100% of files |
| **Keyword Sources** | Filenames only | Grandparent + Parent + Filename |
| **Parallelization** | None | 16 workers |
| **Batch Inserts** | No | Yes (10k at a time) |
| **Memory Usage** | Low (query one keyword at a time) | Medium (3.5k tags in RAM) |
| **Indexes** | None | 5 optimized indexes |
| **Duplicate Prevention** | Manual checks | Unique constraint |

---

## Next Steps

1. **Stop current slow tagger** (if still running)
2. **Add database indexes** (2-5 min, concurrent)
3. **Create curated tag lists** (filter by frequency ≥ 100)
4. **Build tag dictionary binary** (inserts 3.5k tags)
5. **Build fast multi-level tagger** (single-pass extraction)
6. **Run fast tagger** (5-10 minutes total)
7. **Verify results** (check tag counts and coverage)

**Ready to implement!**

# MIDI Library Database & File Organization

**Project Scale:** 1.7M+ original files → 6.7M after processing (splits, extractions)

**Last Updated:** 2025-11-21

---

## Directory Structure Decision

### ✅ RECOMMENDED: Keep Separate

```
/home/dojevou/projects/midi-software-center/
├── database/                    # PostgreSQL migrations, schemas, backups ONLY
│   ├── migrations/             # SQL migration files
│   ├── backups/                # Database dumps (.sql.gz)
│   └── optimizations/          # Index creation, performance tuning
│
├── midi-library/               # ALL MIDI FILES (6.7M files, 71+ GB)
│   ├── archives/               # Source of truth - original + extracted
│   ├── files/                  # Consolidated working library
│   └── temp/                   # Temporary processing
│
└── [application code...]       # Rust, TypeScript, etc.
```

### ❌ NOT RECOMMENDED: Moving to database/

```
database/
├── midi-library/               # ❌ BAD - mixing data with schema
```

**Why Keep Separate:**
- **Flexibility**: Can move files to different drives (SSD vs HDD)
- **Backups**: Separate backup strategies (DB dumps vs file sync)
- **Performance**: Can mount `midi-library/` on faster storage
- **Clarity**: Clear separation of concerns (metadata vs content)
- **Scale**: Easier to manage 6.7M files independently

---

## Recommended File Organization

### Option A: Minimal Structure (RECOMMENDED for 6.7M files)

```
midi-library/
├── archives/                    # 1.5M files - Source archives + extracted
│   ├── [preserve original structure]
│   └── [extracted content in subdirs]
│
├── files/                       # 6.7M files - Consolidated library
│   ├── drums/                  # ~1.2M drum MIDI files
│   ├── melodic/                # ~3.5M melodic/harmonic files
│   ├── loops/                  # ~1.5M loops and patterns
│   ├── splits/                 # ~152K split multi-track files
│   └── other/                  # Uncategorized
│
└── temp/                        # Temporary only (excluded from backup)
    ├── extraction/             # Archive extraction workspace
    ├── analysis/               # Analysis temp files
    └── processing/             # Import/split processing
```

**Pros:**
- Simple, flat structure
- Easy to navigate with file browser
- Fast imports (minimal directory traversal)
- Clear categorization

**Cons:**
- Still ~6.7M files in 5 folders
- Manual categorization needed

---

### Option B: Database-Centric (RECOMMENDED for professionals)

```
midi-library/
├── archives/                    # 1.5M files - Original archives (backup)
│   └── [keep as-is, read-only]
│
├── files/                       # 6.7M files - Flat or by first letter
│   ├── a/                      # Files starting with 'a'
│   ├── b/                      # Files starting with 'b'
│   ├── ...
│   └── z/
│   └── 0-9/                    # Files starting with numbers
│
└── temp/                        # Processing workspace
```

**Organization Method:** Database + Meilisearch

Instead of physical folders, use:
- **Database queries** for filtering (BPM, key, tags)
- **Meilisearch** for full-text search
- **Saved views** in database for virtual folders
- **Symlinks** for DAW integration

**Example Queries (Virtual Folders):**
```sql
-- "Drums 120-130 BPM in C"
SELECT filepath FROM files f
JOIN musical_metadata m ON f.id = m.file_id
WHERE f.tags @> ARRAY['drums']
  AND m.bpm BETWEEN 120 AND 130
  AND m.key_signature = 'C';

-- "Melodic loops with 7th chords"
SELECT filepath FROM files f
JOIN musical_metadata m ON f.id = m.file_id
WHERE f.tags && ARRAY['melodic', 'loop']
  AND m.has_seventh_chords = true;
```

**Pros:**
- No need to physically move 6.7M files
- Flexible, instant reorganization (just change query)
- Multiple ways to "view" same files
- Database handles all organization

**Cons:**
- Requires database queries for browsing
- Need UI for non-technical users

---

### Option C: Hybrid (Good balance)

```
midi-library/
├── archives/                    # Original + extracted archives
│
├── library/                     # Main working library
│   ├── by-type/                # Primary organization
│   │   ├── drums/
│   │   ├── bass/
│   │   ├── melodic/
│   │   └── loops/
│   │
│   └── by-collection/          # Secondary (symlinks)
│       ├── groove-monkee/
│       ├── superior-drummer/
│       └── [other collections]
│
└── temp/                        # Processing
```

**Organization:**
- Physical folders for major categories (drums, bass, etc.)
- Symlinks for alternate views (by collection, by artist)
- Database for advanced queries

---

## Current State (Need to Consolidate)

```
midi-library/
├── archives/        # 1,541,576 files (34 GB)
├── extracted/       # 5,052,093 files (37 GB) ← CONSOLIDATE THIS
├── splits/          # 152,678 files (772 MB)  ← MOVE TO files/splits/
├── organized/       # Empty
└── repaired/        # ~20 KB
```

**Issues:**
1. Duplication between `archives/` and `extracted/`
2. Only 1.17M files in database, but 6.7M on disk
3. No clear organization structure
4. `extracted/` is 5M files not tracked

---

## Migration Plan: Current → Recommended

### Phase 1: Consolidate Files

```bash
# Create new structure
mkdir -p midi-library/files/{drums,melodic,loops,splits,other}
mkdir -p midi-library/temp/{extraction,processing}

# Move splits
mv midi-library/splits/* midi-library/files/splits/

# Archive extracted (decide: merge or keep)
# Option A: Keep extracted separate for now
# Option B: Merge into archives
```

### Phase 2: Database Import

```bash
# Import all files from all directories
./target/release/import \
  --source /home/dojevou/projects/midi-software-center/midi-library/archives \
  --source /home/dojevou/projects/midi-software-center/midi-library/extracted \
  --source /home/dojevou/projects/midi-software-center/midi-library/files

# This will:
# - Hash each file (detect duplicates)
# - Extract metadata
# - Populate database
# - Auto-tag based on content
```

### Phase 3: Analysis

```bash
# Run Phase 2 analysis on all imported files
./target/release/analyze

# This will:
# - Analyze BPM, key, chords
# - Extract controller data, articulation
# - Structure/form analysis
# - Populate musical_metadata table
```

### Phase 4: Organization (Optional)

```bash
# Categorize by drum detection
UPDATE files
SET tags = array_append(tags, 'drums')
WHERE EXISTS (
    SELECT 1 FROM musical_metadata m
    WHERE m.file_id = files.id
    AND m.is_percussive = true
);

# Create symlinks for organized browsing
./scripts/create-organized-symlinks.sh
```

---

## Database Schema Overview

### Files Table (Core)
```sql
files
├── id (PK)
├── filepath (unique, indexed)
├── filename
├── blake3_hash (unique, deduplication)
├── tags[] (GIN indexed)
├── imported_at
└── analyzed_at
```

### Musical Metadata Table (Analysis)
```sql
musical_metadata
├── file_id (FK → files.id)
├── bpm
├── key_signature (ENUM)
├── time_signature
├── duration_seconds
├── is_percussive
├── has_chords
├── chord_progression (JSONB)
├── controller_data (JSONB)
├── articulation_data (JSONB)
└── structure_data (JSONB)
```

### Search Integration
- **Meilisearch**: Full-text search on tags, filename, metadata
- **pgvector**: Semantic similarity search
- **GIN indexes**: Fast array/JSONB queries

---

## Storage Requirements

### Current
- **archives/**: 34 GB
- **extracted/**: 37 GB
- **splits/**: 772 MB
- **Total**: ~71 GB

### After Consolidation
- **files/**: ~71 GB (all MIDI files)
- **archives/**: Delete or keep as backup
- **Database**: ~10-20 GB (metadata, indexes)
- **Total**: 80-90 GB

### Recommended Storage
- **Fast SSD**: Database + frequently used files (20-30 GB)
- **HDD/NAS**: Archive + full library (70+ GB)

---

## Backup Strategy

### Daily
- Database: `pg_dump` → compressed backup
- Config: Git commit

### Weekly
- Full library: rsync to backup drive
- Meilisearch index: Snapshot

### Monthly
- Archive original source files
- Full system backup

---

## Performance Considerations

### File System
- **ext4**: Good for millions of small files
- **ZFS**: Better with compression (MIDI files compress well)
- **XFS**: Good performance, less overhead

### Database
- **Connection pooling**: 32 workers + 2 utility
- **Indexes**: All major search fields
- **Partitioning**: Consider for 10M+ files

### Import Speed
- Current: ~7,800 files/sec (LUDICROUS mode)
- Time for 6.7M files: ~14 minutes

### Analysis Speed
- Current: ~200-400 files/sec
- Time for 6.7M files: ~5-9 hours

---

## Recommended: Option B (Database-Centric)

**Final Structure:**
```
/home/dojevou/projects/midi-software-center/
├── midi-library/
│   ├── files/              # 6.7M files, alphabetically organized
│   │   ├── 0-9/
│   │   ├── a/
│   │   ├── b/
│   │   └── ...
│   ├── archives/           # Backup of original archives
│   └── temp/              # Processing workspace
│
├── database/               # PostgreSQL schema only
│   ├── migrations/
│   └── backups/
│
└── [application code]
```

**Access Method:**
- Use database queries and Meilisearch
- Build web UI for browsing
- Export results as file lists or playlists
- Create symlinks on-demand for DAW projects

---

## Next Steps

1. **Decide**: Keep `midi-library/` at project root? ✅ YES
2. **Import**: Run import on all 6.7M files
3. **Analyze**: Run Phase 2 analysis
4. **Organize**: Use database for virtual folders
5. **UI**: Build browser interface using database

---

## Commands Reference

```bash
# Import all directories
./target/release/import --source /path/to/midi-library

# Analyze all unanalyzed files
./target/release/analyze

# Search by BPM
psql $DATABASE_URL -c "SELECT filepath FROM files f JOIN musical_metadata m ON f.id = m.file_id WHERE m.bpm BETWEEN 120 AND 130;"

# Tag all drums
psql $DATABASE_URL -c "UPDATE files SET tags = array_append(tags, 'drums') WHERE EXISTS (SELECT 1 FROM musical_metadata WHERE file_id = files.id AND is_percussive = true);"

# Count by category
psql $DATABASE_URL -c "SELECT unnest(tags) as tag, COUNT(*) FROM files GROUP BY tag ORDER BY count DESC;"
```

---

**Recommendation:** Keep `midi-library/` at project root, use Option B (Database-Centric) organization.

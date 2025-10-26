# MIDI Library Database Documentation

**Version**: 1.0
**Database**: PostgreSQL 16 with pgvector extension
**Search Engine**: Meilisearch
**Status**: Production Ready

---

## Overview

Complete database system for managing 3,000,000+ MIDI files with:
- Full metadata extraction and storage
- Vector similarity search (pgvector)
- Full-text search (Meilisearch)
- Musical analysis and categorization
- User collections and playlists
- Tag system and favorites

---

## Quick Start

### Setup

```bash
# Start database and load sample data
./scripts/setup.sh

# Run tests
./scripts/test_database.sh

# Connect manually
psql -h localhost -U midiuser -d midi_library
# Password: midipass
```

### Docker Services

- **PostgreSQL**: localhost:5432
- **Meilisearch**: localhost:7700

### Stop Services

```bash
cd database/
docker-compose down
```

---

## Database Schema

### Core Tables

#### `files`
Primary table for all MIDI files.

```sql
CREATE TABLE files (
    id BIGSERIAL PRIMARY KEY,
    filename TEXT NOT NULL,
    filepath TEXT NOT NULL UNIQUE,
    file_hash TEXT NOT NULL UNIQUE,
    file_size_bytes BIGINT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    indexed_at TIMESTAMPTZ,
    analysis_version TEXT,
    processing_status processing_status DEFAULT 'pending'
);
```

**Indexes**:
- `idx_files_hash` (HASH on file_hash)
- `idx_files_filename` (GIN on filename with trigram)
- `idx_files_status` (B-tree on processing_status)

---

#### `musical_metadata`
Musical characteristics extracted from MIDI files.

```sql
CREATE TABLE musical_metadata (
    id BIGSERIAL PRIMARY KEY,
    file_id BIGINT REFERENCES files(id) ON DELETE CASCADE,
    bpm REAL,
    key_signature TEXT,
    time_signature TEXT,
    duration_seconds REAL,
    total_notes INTEGER,
    track_count INTEGER,
    has_drums BOOLEAN DEFAULT FALSE,
    has_melody BOOLEAN DEFAULT FALSE,
    embedding vector(384)  -- For similarity search
);
```

**Key Features**:
- BPM detection
- Key/time signature analysis
- Duration and note counting
- 384-dimensional embedding vectors for similarity

**Indexes**:
- `idx_musical_metadata_file_id` (B-tree)
- `idx_musical_metadata_bpm` (B-tree)
- `idx_musical_metadata_key` (B-tree)
- `idx_musical_metadata_embedding` (HNSW for vector search)

---

#### `instruments`
General MIDI instrument mappings (128 standard instruments).

```sql
CREATE TABLE instruments (
    id SERIAL PRIMARY KEY,
    program_number INTEGER UNIQUE NOT NULL,
    name TEXT NOT NULL,
    category instrument_category NOT NULL,
    description TEXT
);
```

**Categories** (enum):
- piano, chromatic_percussion, organ, guitar, bass
- strings, ensemble, brass, reed, pipe
- synth_lead, synth_pad, synth_effects, ethnic, percussive, sound_effects

---

#### `file_instruments`
Many-to-many relationship between files and instruments.

```sql
CREATE TABLE file_instruments (
    id BIGSERIAL PRIMARY KEY,
    file_id BIGINT REFERENCES files(id) ON DELETE CASCADE,
    instrument_id INTEGER REFERENCES instruments(id),
    channel INTEGER,
    note_count INTEGER,
    is_primary BOOLEAN DEFAULT FALSE,
    UNIQUE(file_id, instrument_id, channel)
);
```

---

#### `tags`
User-defined categorization system.

```sql
CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    category TEXT,
    color TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

**Built-in Categories**:
- genre (e.g., jazz, classical, rock)
- mood (e.g., happy, sad, energetic)
- style (e.g., ambient, aggressive, melodic)
- usage (e.g., intro, outro, breakdown)

---

#### `file_tags`
Many-to-many: files ↔ tags.

```sql
CREATE TABLE file_tags (
    id BIGSERIAL PRIMARY KEY,
    file_id BIGINT REFERENCES files(id) ON DELETE CASCADE,
    tag_id INTEGER REFERENCES tags(id) ON DELETE CASCADE,
    confidence REAL DEFAULT 1.0,
    UNIQUE(file_id, tag_id)
);
```

**Confidence**: Auto-tagged items may have < 1.0 confidence.

---

#### `collections`
User-created collections/folders.

```sql
CREATE TABLE collections (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    parent_id INTEGER REFERENCES collections(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

**Hierarchy**: Supports nested collections via `parent_id`.

---

#### `collection_files`
Files in collections.

```sql
CREATE TABLE collection_files (
    id BIGSERIAL PRIMARY KEY,
    collection_id INTEGER REFERENCES collections(id) ON DELETE CASCADE,
    file_id BIGINT REFERENCES files(id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ DEFAULT NOW(),
    sort_order INTEGER,
    UNIQUE(collection_id, file_id)
);
```

---

#### `favorites`
Quick access to favorite files.

```sql
CREATE TABLE favorites (
    id BIGSERIAL PRIMARY KEY,
    file_id BIGINT REFERENCES files(id) ON DELETE CASCADE UNIQUE,
    added_at TIMESTAMPTZ DEFAULT NOW()
);
```

---

#### `search_history`
Track search queries for analytics.

```sql
CREATE TABLE search_history (
    id BIGSERIAL PRIMARY KEY,
    query TEXT NOT NULL,
    filters JSONB,
    result_count INTEGER,
    searched_at TIMESTAMPTZ DEFAULT NOW()
);
```

---

### Views

#### `v_file_details`
Complete file information with metadata.

```sql
SELECT
    f.id, f.filename, f.filepath,
    m.bpm, m.key_signature, m.time_signature,
    m.duration_seconds, m.total_notes,
    array_agg(DISTINCT t.name) as tags,
    array_agg(DISTINCT i.name) as instruments
FROM files f
LEFT JOIN musical_metadata m ON f.id = m.file_id
-- ... (with aggregations)
```

---

#### `v_database_stats`
Database health and statistics.

```sql
SELECT
    (SELECT COUNT(*) FROM files) as total_files,
    (SELECT COUNT(*) FROM files WHERE processing_status = 'completed') as processed_files,
    (SELECT COUNT(*) FROM tags) as total_tags,
    (SELECT COUNT(*) FROM collections) as total_collections,
    (SELECT COUNT(*) FROM favorites) as total_favorites,
    (SELECT pg_size_pretty(pg_database_size('midi_library'))) as database_size
```

---

## Common Queries

All common queries are in `queries/common_queries.sql`.

### Search by BPM Range

```sql
SELECT f.filename, m.bpm, m.key_signature
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
WHERE m.bpm BETWEEN 120 AND 140
ORDER BY m.bpm;
```

### Search by Key Signature

```sql
SELECT f.filename, m.key_signature, m.bpm
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
WHERE m.key_signature = 'C major'
ORDER BY f.filename;
```

### Find Similar Files (Vector Search)

```sql
SELECT
    f.filename,
    m.bpm,
    m.key_signature,
    1 - (m.embedding <=> target.embedding) AS similarity
FROM musical_metadata m
JOIN files f ON m.file_id = f.id
CROSS JOIN (
    SELECT embedding FROM musical_metadata WHERE file_id = 1
) target
WHERE m.embedding IS NOT NULL
ORDER BY m.embedding <=> target.embedding
LIMIT 10;
```

**Operator `<=>`**: Cosine distance (requires pgvector).

### Find Files by Instrument

```sql
SELECT DISTINCT f.filename, i.name as instrument
FROM files f
JOIN file_instruments fi ON f.id = fi.file_id
JOIN instruments i ON fi.instrument_id = i.id
WHERE i.name = 'Acoustic Grand Piano'
ORDER BY f.filename;
```

### Get Collection Contents

```sql
SELECT
    c.name as collection,
    f.filename,
    cf.added_at
FROM collections c
JOIN collection_files cf ON c.id = cf.collection_id
JOIN files f ON cf.file_id = f.id
WHERE c.name = 'Favorites'
ORDER BY cf.sort_order, cf.added_at;
```

---

## Meilisearch Integration

### Index Configuration

Configuration in `config/meilisearch-index.json`:

```json
{
  "searchableAttributes": [
    "filename",
    "tags",
    "key_signature",
    "instruments"
  ],
  "filterableAttributes": [
    "bpm",
    "key_signature",
    "time_signature",
    "has_drums",
    "has_melody",
    "duration_seconds"
  ],
  "sortableAttributes": [
    "bpm",
    "duration_seconds",
    "total_notes",
    "created_at"
  ]
}
```

### Creating the Index

```bash
# Create index
curl -X POST 'http://localhost:7700/indexes' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "uid": "midi_files",
    "primaryKey": "id"
  }'

# Update settings
curl -X PATCH 'http://localhost:7700/indexes/midi_files/settings' \
  -H 'Content-Type: application/json' \
  --data-binary @config/meilisearch-index.json
```

### Searching

```bash
# Search for files
curl -X POST 'http://localhost:7700/indexes/midi_files/search' \
  -H 'Content-Type: application/json' \
  --data-binary '{
    "q": "jazz piano",
    "filter": "bpm > 100 AND has_drums = true",
    "sort": ["bpm:asc"],
    "limit": 20
  }'
```

---

## Maintenance

### Backup Database

```bash
# Create backup
docker exec midi_library_db pg_dump -U midiuser midi_library > backup_$(date +%Y%m%d).sql

# Restore backup
docker exec -i midi_library_db psql -U midiuser -d midi_library < backup_20250101.sql
```

### Database Statistics

```sql
-- Check database size
SELECT pg_size_pretty(pg_database_size('midi_library'));

-- Check table sizes
SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

-- Index usage
SELECT
    schemaname,
    tablename,
    indexname,
    idx_scan,
    pg_size_pretty(pg_relation_size(indexrelid)) AS index_size
FROM pg_stat_user_indexes
ORDER BY idx_scan DESC;
```

### Vacuum and Analyze

```bash
# Full vacuum (reclaim space)
docker exec midi_library_db psql -U midiuser -d midi_library -c "VACUUM FULL ANALYZE;"

# Regular maintenance
docker exec midi_library_db psql -U midiuser -d midi_library -c "VACUUM ANALYZE;"
```

---

## Performance Tuning

### For Large Datasets (1M+ files)

Edit `docker-compose.yml` PostgreSQL settings:

```yaml
command:
  - "postgres"
  - "-c"
  - "shared_buffers=2GB"          # 25% of RAM
  - "-c"
  - "effective_cache_size=6GB"    # 50-75% of RAM
  - "-c"
  - "work_mem=64MB"                # For sorting/hashing
  - "-c"
  - "maintenance_work_mem=512MB"   # For VACUUM, CREATE INDEX
  - "-c"
  - "max_connections=100"
```

### Index Maintenance

```sql
-- Rebuild indexes (if performance degrades)
REINDEX TABLE files;
REINDEX TABLE musical_metadata;

-- Update statistics
ANALYZE files;
ANALYZE musical_metadata;
```

---

## Troubleshooting

### Connection Issues

```bash
# Check if container is running
docker ps | grep midi_library_db

# Check logs
docker logs midi_library_db

# Restart container
docker-compose restart
```

### Permission Issues

```bash
# Grant permissions manually
docker exec -it midi_library_db psql -U midiuser -d midi_library
# Then: GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO midiuser;
```

### Slow Queries

```sql
-- Enable query logging
ALTER SYSTEM SET log_min_duration_statement = 1000; -- Log queries > 1s
SELECT pg_reload_conf();

-- Check slow queries
SELECT * FROM pg_stat_statements ORDER BY total_exec_time DESC LIMIT 10;
```

---

## Architecture Notes

### Design Principles

1. **Normalized schema**: Minimize redundancy
2. **Indexed appropriately**: Fast searches on common fields
3. **Vector embeddings**: For ML-based similarity
4. **Full-text search**: Offloaded to Meilisearch
5. **Flexible tagging**: User-defined organization
6. **Scalable**: Handles millions of files

### File Processing Workflow

1. **Ingest**: File discovered, hash computed, added to `files`
2. **Parse**: MIDI file parsed (separate Rust module)
3. **Analyze**: Musical metadata extracted
4. **Store**: Metadata saved to database
5. **Index**: Document sent to Meilisearch
6. **Embed**: (Optional) Generate vector embedding for similarity

### Status Values

Files use `processing_status` enum:
- `pending`: Awaiting processing
- `processing`: Currently being analyzed
- `completed`: Successfully processed
- `failed`: Processing error
- `skipped`: Intentionally not processed

---

## Sample Data

The `scripts/insert_sample_data.sql` includes:

- **128 GM instruments** (full General MIDI set)
- **30 common tags** (genres, moods, styles)
- **10 collections** (e.g., "Jazz Standards", "Electronic")
- **100 sample MIDI files** (with realistic metadata)
- **Favorites** and **search history** examples

---

## Next Steps

1. **Integration**: Connect Rust/Tauri pipeline to this database
2. **Embeddings**: Generate vector embeddings during analysis
3. **Meilisearch sync**: Automatically sync new files to search index
4. **Batch processing**: Process existing 3M MIDI files
5. **UI**: Build Svelte interface for browsing/searching

---

## Reference

- PostgreSQL: https://www.postgresql.org/docs/16/
- pgvector: https://github.com/pgvector/pgvector
- Meilisearch: https://www.meilisearch.com/docs
- General MIDI: https://www.midi.org/specifications

---

**Database is production-ready and tested. Ready for integration with Pipeline and DAW applications.**

# Database Test Fixtures

This directory contains comprehensive test data for all 18 database tables in the MIDI Software Center.

## Files

- **test_data.sql** - Complete test fixtures with 920 lines of sample data

## Coverage

The test fixtures provide comprehensive coverage across all database tables:

### Core Tables (15 files)
1. **files** - 15 MIDI files covering various scenarios
2. **musical_metadata** - 11 metadata records with BPM, key, time signature
3. **file_categories** - 12 category classifications
4. **file_instruments** - 10 instrument detections
5. **tags** - 15 tag definitions across 4 categories
6. **file_tags** - 26 file-tag relationships
7. **favorites** - 4 favorited files
8. **track_splits** - 2 parent-child track relationships
9. **duplicate_groups** - 1 duplicate group
10. **duplicate_files** - 2 files in duplicate group
11. **file_embeddings** - 3 vector embeddings (sample)
12. **file_compatibility** - 3 compatibility pairs
13. **rhythm_patterns** - 2 rhythmic analyses
14. **harmonic_patterns** - 2 chord progressions
15. **melodic_patterns** - 2 melodic analyses
16. **processing_jobs** - 3 jobs (completed, running, failed)
17. **processing_errors** - 5 error records
18. **schema_migrations** - (managed by migrations)

## Test Data Scenarios

### BPM Coverage
- 60 BPM - Slow bass line
- 80 BPM - Ambient pad
- 90 BPM - Drum loop
- 100 BPM - Full song with tempo changes
- 110 BPM - Vocal melody
- 120 BPM - Piano melody
- 128 BPM - Lead synth (house standard)
- 140 BPM - House chord
- 174 BPM - DnB arpeggio

### Musical Keys
- C major, C minor
- D minor
- E minor
- F major
- G major
- A minor, A major
- UNKNOWN (percussive)

### MIDI Formats
- Format 0 (single track)
- Format 1 (multi-track)
- Format 2 (not included - rare)

### File Sizes
- Minimal: 100 bytes
- Small: 512-1024 bytes
- Medium: 2-4 KB
- Large: 8-10 KB
- Very large: 100 KB (full song)

### Special Scenarios

#### Multi-track Files
- File 7: Parent file (8 tracks)
- Files 8-9: Split tracks from parent

#### Duplicates
- Files 1 & 10: Same content_hash (duplicate detection)

#### Edge Cases
- File 11: Minimal file (100 bytes)
- File 12: NULL manufacturer/collection
- File 14: FX riser (variable BPM)

#### Musical Characteristics
- Monophonic: Bass, DnB arp, vocal
- Polyphonic: Piano, house chord, pad
- Percussive: Drums
- Chords: Piano, house chord, pad
- Melody: Bass, piano, vocal, lead

## Usage

### Load Test Data

```bash
# Option 1: Using psql directly
psql -U midiuser -d midi_library -h localhost -p 5433 < tests/fixtures/database/test_data.sql

# Option 2: Using Docker
docker exec -i midi-postgres psql -U midiuser -d midi_library < tests/fixtures/database/test_data.sql

# Option 3: Using make (if target exists)
make db-load-fixtures
```

### Clean Test Data

```bash
# Remove all test data (WARNING: This will delete data!)
psql -U midiuser -d midi_library -h localhost -p 5433 -c "
BEGIN;
TRUNCATE files, tags, processing_jobs CASCADE;
COMMIT;
"
```

## Query Examples

### Find Files by BPM Range

```sql
SELECT f.filename, m.bpm, m.key_signature
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
WHERE m.bpm BETWEEN 120 AND 140;
```

### Get Tagged Files

```sql
SELECT f.filename, array_agg(t.name) as tags
FROM files f
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
GROUP BY f.id, f.filename;
```

### Find Compatible Files

```sql
SELECT
    f1.filename as file_a,
    f2.filename as file_b,
    fc.overall_score,
    fc.key_compatible,
    fc.bpm_compatible
FROM file_compatibility fc
JOIN files f1 ON fc.file_id_a = f1.id
JOIN files f2 ON fc.file_id_b = f2.id
ORDER BY fc.overall_score DESC;
```

### Get User Favorites

```sql
SELECT f.filename, f.filepath, fav.created_at
FROM favorites fav
JOIN files f ON fav.file_id = f.id
ORDER BY fav.created_at DESC;
```

### Find Track Splits

```sql
SELECT
    parent.filename as parent_file,
    ts.track_number,
    ts.track_name,
    ts.instrument,
    split.filename as split_file
FROM track_splits ts
JOIN files parent ON ts.parent_file_id = parent.id
JOIN files split ON ts.split_file_id = split.id
ORDER BY parent.id, ts.track_number;
```

### Duplicate Detection

```sql
SELECT
    dg.duplicate_count,
    array_agg(f.filepath) as duplicate_paths,
    dg.total_size_bytes
FROM duplicate_groups dg
JOIN duplicate_files df ON dg.id = df.group_id
JOIN files f ON df.file_id = f.id
GROUP BY dg.id, dg.duplicate_count, dg.total_size_bytes;
```

## Integration with Tests

Use these fixtures in Rust tests with `sqlx::test`:

```rust
use sqlx::PgPool;

#[sqlx::test(fixtures("test_data"))]
async fn test_find_files_by_bpm(pool: PgPool) {
    let files = find_files_by_bpm_range(&pool, 120.0, 140.0).await.unwrap();
    assert_eq!(files.len(), 3); // Files 3, 4, 15
}

#[sqlx::test(fixtures("test_data"))]
async fn test_get_favorites(pool: PgPool) {
    let favorites = get_user_favorites(&pool).await.unwrap();
    assert_eq!(favorites.len(), 4); // Files 3, 5, 7, 15
}
```

## Maintenance

When adding new tables or test scenarios:

1. Update `test_data.sql` with new sample data
2. Update this README with coverage details
3. Run verification queries to ensure data integrity
4. Update integration tests to use new fixtures

## Notes

- All timestamps use `NOW()` with intervals for realistic relative dates
- Content hashes are hex-decoded for proper BYTEA storage
- Vector embeddings use `array_fill()` for testing (not real embeddings)
- Foreign key constraints are properly maintained
- Triggers automatically update `usage_count`, `search_vector`, etc.

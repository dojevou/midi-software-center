# Migration 006: Track Splits

## Overview

**File**: `006_track_splits.sql`
**Purpose**: Track relationships between parent multi-track MIDI files and their split single-track versions
**Date**: 2025-10-12

---

## What This Migration Does

Creates the `track_splits` table to maintain relationships when multi-track MIDI files are split into individual track files. This enables:

- Finding all split tracks from a parent file
- Finding the parent file of a split track
- Tracking metadata about each split (track number, name, instrument)
- Performance analysis (note counts per track)

---

## Schema Details

### Table: `track_splits`

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | BIGSERIAL | PRIMARY KEY | Auto-incrementing unique identifier |
| `parent_file_id` | BIGINT | NOT NULL, FK → files(id) | Original multi-track file |
| `split_file_id` | BIGINT | NOT NULL, FK → files(id) | Split single-track file |
| `track_number` | INT | NOT NULL, ≥ 0 | Track index in parent (0-based) |
| `track_name` | TEXT | NULL | Track name from MIDI metadata |
| `instrument` | TEXT | NULL | Instrument/program name |
| `note_count` | INT | NULL, ≥ 0 | Number of note events in track |
| `created_at` | TIMESTAMP | DEFAULT NOW() | Split creation timestamp |

### Constraints

- **UNIQUE**: `(parent_file_id, split_file_id)` - No duplicate relationships
- **CASCADE DELETE**: If parent or split file is deleted, relationship is deleted
- **CHECK**: `track_number >= 0`
- **CHECK**: `note_count >= 0` (if specified)

### Indexes

1. `idx_track_splits_parent_file_id` - Find all splits of a parent
2. `idx_track_splits_split_file_id` - Find parent of a split
3. `idx_track_splits_parent_track_number` - Order splits by track number
4. `idx_track_splits_instrument` - Find splits by instrument (partial index)

---

## Applying the Migration

### Method 1: Using Docker (Recommended)

```bash
# Navigate to migrations directory
cd ~/projects/midi-library-system/database/migrations

# Apply migration using Docker
docker exec -i midi_postgres psql -U postgres -d midi_library < 006_track_splits.sql

# Expected output:
# CREATE TABLE
# CREATE INDEX
# CREATE INDEX
# CREATE INDEX
# CREATE INDEX
# COMMENT
# COMMENT
# ...
```

### Method 2: Using psql directly

```bash
# Connect to database
psql -U postgres -d midi_library

# Run migration
\i /home/dojevou/projects/midi-library-system/database/migrations/006_track_splits.sql

# Exit
\q
```

### Method 3: Using environment variable

```bash
# Set DATABASE_URL
export DATABASE_URL="postgresql://postgres:password@localhost:5432/midi_library"

# Apply with psql
psql $DATABASE_URL -f 006_track_splits.sql
```

---

## Verification

### 1. Verify Table Exists

```sql
SELECT table_name, table_type
FROM information_schema.tables
WHERE table_name = 'track_splits';
```

**Expected**: One row with `track_splits` table

### 2. Verify Table Structure

```sql
\d track_splits
```

**Expected Output**:
```
                                      Table "public.track_splits"
     Column      |            Type             | Collation | Nullable |                Default
-----------------+-----------------------------+-----------+----------+----------------------------------------
 id              | bigint                      |           | not null | nextval('track_splits_id_seq'::regclass)
 parent_file_id  | bigint                      |           | not null |
 split_file_id   | bigint                      |           | not null |
 track_number    | integer                     |           | not null |
 track_name      | text                        |           |          |
 instrument      | text                        |           |          |
 note_count      | integer                     |           |          |
 created_at      | timestamp without time zone |           |          | now()
Indexes:
    "track_splits_pkey" PRIMARY KEY, btree (id)
    "uq_parent_split_pair" UNIQUE CONSTRAINT, btree (parent_file_id, split_file_id)
    "idx_track_splits_instrument" btree (instrument) WHERE instrument IS NOT NULL
    "idx_track_splits_parent_file_id" btree (parent_file_id)
    "idx_track_splits_parent_track_number" btree (parent_file_id, track_number)
    "idx_track_splits_split_file_id" btree (split_file_id)
Foreign-key constraints:
    "track_splits_parent_file_id_fkey" FOREIGN KEY (parent_file_id) REFERENCES files(id) ON DELETE CASCADE
    "track_splits_split_file_id_fkey" FOREIGN KEY (split_file_id) REFERENCES files(id) ON DELETE CASCADE
Check constraints:
    "chk_note_count_positive" CHECK (note_count IS NULL OR note_count >= 0)
    "chk_track_number_positive" CHECK (track_number >= 0)
```

### 3. Verify Indexes

```sql
SELECT indexname, indexdef
FROM pg_indexes
WHERE tablename = 'track_splits'
ORDER BY indexname;
```

**Expected**: 6 indexes (1 primary key + 1 unique + 4 performance indexes)

### 4. Verify Comments

```sql
SELECT
    col_description('track_splits'::regclass, 0) AS table_comment,
    col_description('track_splits'::regclass, 1) AS id_comment;
```

**Expected**: Non-null descriptions

---

## Example Usage

### Insert a Split Relationship

```sql
-- Assuming parent file ID = 100, split file ID = 201
INSERT INTO track_splits (
    parent_file_id,
    split_file_id,
    track_number,
    track_name,
    instrument,
    note_count
) VALUES (
    100,
    201,
    0,
    'Lead Melody',
    'Piano',
    1247
);
```

### Find All Splits of a Parent File

```sql
-- Find all tracks split from parent file 100
SELECT
    ts.track_number,
    ts.track_name,
    ts.instrument,
    ts.note_count,
    f.filename AS split_filename,
    f.file_path AS split_path
FROM track_splits ts
JOIN files f ON ts.split_file_id = f.id
WHERE ts.parent_file_id = 100
ORDER BY ts.track_number;
```

### Find Parent of a Split File

```sql
-- Find parent of split file 201
SELECT
    parent.filename AS parent_filename,
    parent.file_path AS parent_path,
    ts.track_number,
    ts.track_name,
    ts.instrument
FROM track_splits ts
JOIN files parent ON ts.parent_file_id = parent.id
WHERE ts.split_file_id = 201;
```

### Find All Splits with Piano Tracks

```sql
SELECT
    parent.filename AS parent_file,
    ts.track_number,
    ts.track_name,
    ts.note_count,
    split.filename AS split_file
FROM track_splits ts
JOIN files parent ON ts.parent_file_id = parent.id
JOIN files split ON ts.split_file_id = split.id
WHERE ts.instrument ILIKE '%piano%'
ORDER BY ts.note_count DESC;
```

### Find Parents with Most Tracks

```sql
SELECT
    parent.filename,
    COUNT(*) AS num_splits,
    SUM(ts.note_count) AS total_notes
FROM track_splits ts
JOIN files parent ON ts.parent_file_id = parent.id
GROUP BY parent.id, parent.filename
ORDER BY num_splits DESC
LIMIT 10;
```

### Find Empty or Sparse Tracks

```sql
-- Find tracks with fewer than 10 notes
SELECT
    parent.filename AS parent_file,
    ts.track_number,
    ts.track_name,
    ts.instrument,
    ts.note_count
FROM track_splits ts
JOIN files parent ON ts.parent_file_id = parent.id
WHERE ts.note_count IS NOT NULL AND ts.note_count < 10
ORDER BY parent.filename, ts.track_number;
```

---

## Rollback (If Needed)

If you need to undo this migration:

```sql
-- Drop the table (will cascade to indexes and constraints)
DROP TABLE IF EXISTS track_splits CASCADE;
```

**Warning**: This will permanently delete all split relationship data!

---

## Integration with Application

### Rust (SQLx)

```rust
// Model
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TrackSplit {
    pub id: i64,
    pub parent_file_id: i64,
    pub split_file_id: i64,
    pub track_number: i32,
    pub track_name: Option<String>,
    pub instrument: Option<String>,
    pub note_count: Option<i32>,
    pub created_at: chrono::NaiveDateTime,
}

// Insert
async fn insert_track_split(
    pool: &PgPool,
    parent_id: i64,
    split_id: i64,
    track_num: i32,
    track_name: Option<&str>,
    instrument: Option<&str>,
    note_count: Option<i32>,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        INSERT INTO track_splits
            (parent_file_id, split_file_id, track_number, track_name, instrument, note_count)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
        parent_id,
        split_id,
        track_num,
        track_name,
        instrument,
        note_count
    )
    .fetch_one(pool)
    .await?;

    Ok(row.id)
}

// Query
async fn get_splits_for_parent(
    pool: &PgPool,
    parent_id: i64,
) -> Result<Vec<TrackSplit>, sqlx::Error> {
    sqlx::query_as!(
        TrackSplit,
        r#"
        SELECT id, parent_file_id, split_file_id, track_number,
               track_name, instrument, note_count, created_at
        FROM track_splits
        WHERE parent_file_id = $1
        ORDER BY track_number
        "#,
        parent_id
    )
    .fetch_all(pool)
    .await
}
```

---

## Testing

### Test Data

```sql
-- Insert test parent file
INSERT INTO files (filename, file_path, file_hash, file_size)
VALUES ('multi_track_test.mid', '/test/multi_track_test.mid', 'hash123', 10240)
RETURNING id; -- Assume returns 1000

-- Insert test split files
INSERT INTO files (filename, file_path, file_hash, file_size)
VALUES
    ('multi_track_test_track_0.mid', '/test/splits/track_0.mid', 'hash124', 2048),
    ('multi_track_test_track_1.mid', '/test/splits/track_1.mid', 'hash125', 3072),
    ('multi_track_test_track_2.mid', '/test/splits/track_2.mid', 'hash126', 1024)
RETURNING id; -- Assume returns 1001, 1002, 1003

-- Insert split relationships
INSERT INTO track_splits (parent_file_id, split_file_id, track_number, track_name, instrument, note_count)
VALUES
    (1000, 1001, 0, 'Drums', 'Drum Kit', 847),
    (1000, 1002, 1, 'Bass', 'Electric Bass', 423),
    (1000, 1003, 2, 'Piano', 'Acoustic Piano', 1247);

-- Verify
SELECT * FROM track_splits WHERE parent_file_id = 1000;
```

### Constraint Tests

```sql
-- Test: Duplicate prevention (should fail)
INSERT INTO track_splits (parent_file_id, split_file_id, track_number)
VALUES (1000, 1001, 0);
-- Expected: ERROR: duplicate key value violates unique constraint "uq_parent_split_pair"

-- Test: Negative track number (should fail)
INSERT INTO track_splits (parent_file_id, split_file_id, track_number)
VALUES (1000, 1004, -1);
-- Expected: ERROR: new row violates check constraint "chk_track_number_positive"

-- Test: Negative note count (should fail)
INSERT INTO track_splits (parent_file_id, split_file_id, track_number, note_count)
VALUES (1000, 1004, 3, -10);
-- Expected: ERROR: new row violates check constraint "chk_note_count_positive"

-- Test: Cascade delete (should succeed)
DELETE FROM files WHERE id = 1000;
SELECT * FROM track_splits WHERE parent_file_id = 1000;
-- Expected: 0 rows (all splits deleted)
```

---

## Performance Notes

- **Parent lookups**: O(log n) via `idx_track_splits_parent_file_id`
- **Split lookups**: O(log n) via `idx_track_splits_split_file_id`
- **Ordered track retrieval**: O(log n) via `idx_track_splits_parent_track_number`
- **Instrument search**: O(log n) via partial index (only non-null values)

Expected query times for 1M split relationships:
- Find all splits of parent: < 10ms
- Find parent of split: < 5ms
- Instrument search: < 20ms

---

## Next Steps

1. ✅ Apply migration
2. ✅ Verify with test data
3. Create Rust models (`db/models/track_split.rs`)
4. Create repository (`db/repositories/track_split_repository.rs`)
5. Create Tauri command for split operations
6. Update UI to show split relationships

---

## Related Files

- **Schema**: `001_initial_schema.sql` (files table)
- **Documentation**: `docs/implementation-guides/01-DATABASE-SCHEMA.md`
- **Models**: `pipeline/src-tauri/src/db/models/` (to be created)

---

**Migration Status**: Ready to apply
**Breaking Changes**: None
**Rollback Available**: Yes (DROP TABLE)

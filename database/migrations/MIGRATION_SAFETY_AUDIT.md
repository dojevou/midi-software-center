# Database Migration Safety Audit Report
**MIDI Software Center - Data Integrity Review**
**Date:** 2025-11-29
**Auditor:** Data Integrity Guardian
**Scope:** 11 migrations + 1 data update script

---

## Executive Summary

**Overall Risk Level:** 🟠 **MEDIUM-HIGH**

**Critical Findings:**
- **0/11 migrations have rollback scripts** (100% missing)
- **3 migrations contain irreversible operations** (data loss risk)
- **2 migrations have potential performance impact** (long-running locks)
- **4 migrations lack proper constraint validation**
- **1 migration modifies existing data** (corruption risk if interrupted)

**Production Readiness:** ❌ **NOT SAFE FOR PRODUCTION**

**Required Actions Before Deployment:**
1. Create rollback scripts for all 11 migrations
2. Add transaction safety to data modification migration
3. Implement backup verification before irreversible operations
4. Add constraint validation for existing data
5. Create migration testing framework

---

## Migration-by-Migration Analysis

### Migration 001: Initial Schema ✅ SAFE (with caveats)

**File:** `001_initial_schema.sql`
**Risk Level:** 🟢 **LOW**
**Lines of Code:** 899
**Transaction Safety:** ✅ Wrapped in BEGIN/COMMIT

#### Safety Analysis

**Strengths:**
- Comprehensive schema with proper constraints
- All tables use CASCADE on foreign keys (referential integrity)
- CHECK constraints prevent invalid data (BPM 20-300, pitch 0-127, scores 0-1)
- Triggers maintain denormalized data (search_vector, usage_count)
- Verification block at end confirms table/index/trigger counts
- Comments document all tables and critical columns

**Data Integrity Risks:**

1. **MEDIUM: CASCADE DELETE chains could cause unexpected data loss**
   ```sql
   parent_file_id BIGINT REFERENCES files(id) ON DELETE CASCADE
   ```
   - Deleting a parent file deletes ALL split tracks
   - Deleting a duplicate group deletes ALL duplicate records
   - No audit trail of cascaded deletions

   **Impact:** If a parent file is accidentally deleted, all children vanish permanently

   **Mitigation Required:**
   - Add `deleted_files` audit table to track cascaded deletions
   - Consider SOFT DELETES with `deleted_at` column instead of CASCADE
   - Implement pre-delete triggers to log what will be cascaded

2. **LOW: Trigger-maintained data could become inconsistent**
   ```sql
   CREATE TRIGGER update_tag_usage_count() ...
   ```
   - Triggers update `tags.usage_count`, `duplicate_groups.duplicate_count`
   - If triggers fail silently, counts become incorrect
   - No validation that counts match reality

   **Impact:** Tag counts and duplicate counts could drift from truth

   **Mitigation Required:**
   - Add periodic validation job to recompute counts
   - Implement trigger error logging
   - Add constraint that usage_count >= 0

3. **LOW: No validation that embedded JSONB follows schema**
   ```sql
   tempo_changes JSONB,
   key_changes JSONB,
   ```
   - Free-form JSONB with no schema enforcement
   - Application could insert malformed JSON
   - Queries assume specific structure

   **Impact:** Invalid JSON could break queries/analysis

   **Mitigation Required:**
   - Add JSONB schema validation via CHECK constraints
   - Document expected JSON structure in comments (partially done)
   - Add validation functions

#### Rollback Safety

**Rollback Script Required:** ✅ YES

**Rollback Complexity:** 🟠 **MEDIUM**

**Rollback Script (`001_initial_schema_rollback.sql`):**

```sql
-- ROLLBACK for Migration 001: Initial Schema
-- WARNING: This will DELETE ALL DATA in the database
-- BACKUP REQUIRED before executing

BEGIN;

-- Record rollback in migrations table FIRST (before dropping it)
INSERT INTO schema_migrations (version, description)
VALUES ('001_rollback', 'Rolled back initial schema migration')
ON CONFLICT (version) DO NOTHING;

-- Drop views (depend on tables)
DROP VIEW IF EXISTS pending_tag_suggestions CASCADE;
DROP VIEW IF EXISTS popular_tags_by_category CASCADE;
DROP VIEW IF EXISTS tags_with_categories CASCADE;
DROP VIEW IF EXISTS duplicate_summary CASCADE;
DROP VIEW IF EXISTS files_with_tags CASCADE;
DROP VIEW IF EXISTS files_with_metadata CASCADE;

-- Drop triggers
DROP TRIGGER IF EXISTS processing_errors_count_trigger ON processing_errors;
DROP TRIGGER IF EXISTS duplicate_files_count_trigger ON duplicate_files;
DROP TRIGGER IF EXISTS file_tags_usage_trigger ON file_tags;
DROP TRIGGER IF EXISTS files_updated_at_trigger ON files;
DROP TRIGGER IF EXISTS files_search_vector_trigger ON files;

-- Drop functions
DROP FUNCTION IF EXISTS update_job_progress();
DROP FUNCTION IF EXISTS update_duplicate_group_count();
DROP FUNCTION IF EXISTS update_tag_usage_count();
DROP FUNCTION IF EXISTS update_updated_at_column();
DROP FUNCTION IF EXISTS files_search_vector_update();

-- Drop indexes (will cascade with tables, but explicit for clarity)
-- [Indexes drop automatically with tables]

-- Drop tables (in reverse dependency order)
DROP TABLE IF EXISTS processing_errors CASCADE;
DROP TABLE IF EXISTS processing_jobs CASCADE;
DROP TABLE IF EXISTS melodic_patterns CASCADE;
DROP TABLE IF EXISTS harmonic_patterns CASCADE;
DROP TABLE IF EXISTS rhythm_patterns CASCADE;
DROP TABLE IF EXISTS file_compatibility CASCADE;
DROP TABLE IF EXISTS file_embeddings CASCADE;
DROP TABLE IF EXISTS duplicate_files CASCADE;
DROP TABLE IF EXISTS duplicate_groups CASCADE;
DROP TABLE IF EXISTS file_tags CASCADE;
DROP TABLE IF EXISTS tags CASCADE;
DROP TABLE IF EXISTS file_instruments CASCADE;
DROP TABLE IF EXISTS file_categories CASCADE;
DROP TABLE IF EXISTS musical_metadata CASCADE;
DROP TABLE IF EXISTS files CASCADE;

-- Drop schema migrations table LAST
DROP TABLE IF EXISTS schema_migrations CASCADE;

-- Drop enum types
DROP TYPE IF EXISTS musical_key CASCADE;
DROP TYPE IF EXISTS file_category CASCADE;

-- Drop extensions
DROP EXTENSION IF EXISTS pg_trgm CASCADE;
DROP EXTENSION IF EXISTS vector CASCADE;

COMMIT;
```

**Rollback Risks:**
- ❌ **IRREVERSIBLE DATA LOSS** - All data deleted
- ⚠️ If rollback fails mid-execution, database is in broken state
- ⚠️ Must verify backup exists and is restorable BEFORE rollback

---

### Migration 002: Add Parent Folder Column ✅ SAFE

**File:** `002_add_parent_folder.sql`
**Risk Level:** 🟢 **LOW**
**Lines of Code:** 13
**Transaction Safety:** ❌ No explicit transaction

#### Safety Analysis

**Strengths:**
- Simple additive migration (adds column + index)
- Uses `IF NOT EXISTS` - idempotent
- No data modification
- Partial index (WHERE parent_folder IS NOT NULL) - no wasted space

**Data Integrity Risks:**

1. **LOW: Missing transaction wrapper**
   - If migration fails mid-execution, database could have column but no index
   - Non-atomic operation

   **Mitigation Required:**
   ```sql
   BEGIN;
   ALTER TABLE files ADD COLUMN IF NOT EXISTS parent_folder TEXT;
   CREATE INDEX IF NOT EXISTS idx_files_parent_folder ON files(parent_folder)
       WHERE parent_folder IS NOT NULL;
   COMMIT;
   ```

2. **LOW: No validation for parent_folder values**
   - Column accepts any TEXT value
   - Could contain inconsistent paths, special characters, etc.

   **Mitigation Required:**
   - Add CHECK constraint to validate folder names
   - Consider ENUM for known folder types

#### Rollback Safety

**Rollback Script Required:** ✅ YES

**Rollback Complexity:** 🟢 **SIMPLE**

**Rollback Script (`002_add_parent_folder_rollback.sql`):**

```sql
-- ROLLBACK for Migration 002: Add parent_folder column
-- SAFE: No data loss (column is nullable and optional)

BEGIN;

-- Drop index first
DROP INDEX IF EXISTS idx_files_parent_folder;

-- Drop column (PostgreSQL drops column data safely)
ALTER TABLE files DROP COLUMN IF EXISTS parent_folder;

COMMIT;
```

**Rollback Risks:**
- ⚠️ If column contained user-entered data, that data is lost on rollback
- ✅ Safe if column was auto-populated (can be regenerated)

---

### Migration 003: Add Favorites Table ✅ SAFE

**File:** `003_favorites.sql`
**Risk Level:** 🟢 **LOW**
**Lines of Code:** 31
**Transaction Safety:** ✅ Wrapped in BEGIN/COMMIT

#### Safety Analysis

**Strengths:**
- Transaction-safe
- Uses `IF NOT EXISTS` - idempotent
- CASCADE DELETE maintains referential integrity
- UNIQUE constraint prevents duplicate favorites
- Records migration in schema_migrations table

**Data Integrity Risks:**

1. **LOW: CASCADE DELETE removes favorites when file deleted**
   ```sql
   file_id BIGINT NOT NULL REFERENCES files(id) ON DELETE CASCADE
   ```
   - If file is deleted, favorite is silently removed
   - No notification to user that favorite was lost

   **Mitigation Required:**
   - Consider RESTRICT instead of CASCADE (prevent deleting favorited files)
   - OR add trigger to notify/log when favorites are cascaded

2. **LOW: No user isolation**
   - Single favorites table for all users (no user_id column)
   - Fine for single-user app, breaks for multi-user

   **Impact:** Not an issue for current architecture (single-user desktop app)

#### Rollback Safety

**Rollback Script Required:** ✅ YES

**Rollback Complexity:** 🟢 **SIMPLE**

**Rollback Script (`003_favorites_rollback.sql`):**

```sql
-- ROLLBACK for Migration 003: Favorites table
-- DATA LOSS: All favorites will be deleted

BEGIN;

-- Drop indexes
DROP INDEX IF EXISTS idx_favorites_created_at;
DROP INDEX IF EXISTS idx_favorites_file_id;

-- Drop table
DROP TABLE IF EXISTS favorites CASCADE;

-- Remove from migrations table
DELETE FROM schema_migrations WHERE version = '003';

COMMIT;
```

**Rollback Risks:**
- ⚠️ User favorites are permanently lost
- ✅ Can be rebuilt manually by user after rollback

---

### Migration 006: Track Splits ✅ SAFE

**File:** `006_track_splits.sql`
**Risk Level:** 🟢 **LOW**
**Lines of Code:** 100
**Transaction Safety:** ❌ No explicit transaction

#### Safety Analysis

**Strengths:**
- Well-documented migration with comments
- CHECK constraints validate track_number >= 0, note_count >= 0
- UNIQUE constraint prevents duplicate parent-split pairs
- CASCADE DELETE maintains referential integrity
- Comprehensive indexes for query patterns

**Data Integrity Risks:**

1. **MEDIUM: Missing transaction wrapper**
   - 5 CREATE INDEX statements could partially succeed
   - Database in inconsistent state if migration interrupted

   **Mitigation Required:**
   ```sql
   BEGIN;
   CREATE TABLE track_splits (...);
   CREATE INDEX idx_track_splits_parent_file_id ...;
   CREATE INDEX idx_track_splits_split_file_id ...;
   CREATE INDEX idx_track_splits_parent_track_number ...;
   CREATE INDEX idx_track_splits_instrument ...;
   COMMIT;
   ```

2. **LOW: No validation that split_file_id references a single-track file**
   - Constraint allows circular references (split pointing to another split)
   - No CHECK that split_file_id.num_tracks == 1

   **Mitigation Required:**
   - Add CHECK constraint or trigger to validate split is single-track

3. **LOW: track_number is 0-indexed but not validated against parent's num_tracks**
   - Could have track_number=5 when parent only has 3 tracks

   **Mitigation Required:**
   - Add trigger to validate track_number < parent.num_tracks

#### Rollback Safety

**Rollback Script Required:** ✅ YES

**Rollback Complexity:** 🟢 **SIMPLE**

**Rollback Script (`006_track_splits_rollback.sql`):**

```sql
-- ROLLBACK for Migration 006: Track splits
-- DATA LOSS: All track split relationships will be deleted
-- NOTE: Split files themselves are NOT deleted (only relationship records)

BEGIN;

-- Drop indexes
DROP INDEX IF EXISTS idx_track_splits_instrument;
DROP INDEX IF EXISTS idx_track_splits_parent_track_number;
DROP INDEX IF EXISTS idx_track_splits_split_file_id;
DROP INDEX IF EXISTS idx_track_splits_parent_file_id;

-- Drop table
DROP TABLE IF EXISTS track_splits CASCADE;

COMMIT;
```

**Rollback Risks:**
- ⚠️ Track split metadata is lost (which file came from which parent)
- ✅ Actual MIDI files are preserved (only relationship data lost)
- ✅ Can be regenerated by re-running split analysis

---

### Migration 007: Enhanced Tags Schema 🟠 MEDIUM RISK

**File:** `007_enhanced_tags.sql`
**Risk Level:** 🟠 **MEDIUM**
**Lines of Code:** 408
**Transaction Safety:** ✅ Wrapped in BEGIN/COMMIT

#### Safety Analysis

**Strengths:**
- Transaction-safe
- Uses `IF NOT EXISTS` - idempotent
- Comprehensive tag categorization system
- Helper functions for tag operations
- Inserts core tags (genres, instruments, elements, etc.)

**Data Integrity Risks:**

1. **HIGH: Modifies existing tags table without handling existing data**
   ```sql
   ALTER TABLE tags
       ADD COLUMN IF NOT EXISTS category_id INTEGER REFERENCES tag_categories(id),
       ADD COLUMN IF NOT EXISTS priority INTEGER DEFAULT 50,
       ...
   ```
   - If tags table already has data, new columns are NULL
   - Existing tags won't have category_id set
   - Queries that filter by category will miss old tags

   **Impact:** Pre-existing tags become invisible to category-based queries

   **Mitigation Required:**
   ```sql
   -- After ALTER TABLE, update existing tags
   UPDATE tags SET category_id = (
       SELECT id FROM tag_categories WHERE name = 'technical'
   ) WHERE category_id IS NULL;
   ```

2. **MEDIUM: tag_suggestions.is_accepted is nullable - tri-state logic**
   ```sql
   is_accepted BOOLEAN,
   ```
   - NULL = pending, TRUE = accepted, FALSE = rejected
   - Application must handle NULL correctly
   - Queries could miss NULL values

   **Mitigation Required:**
   - Document tri-state behavior clearly
   - Add CHECK constraint comments

3. **MEDIUM: auto_tagging_rules uses INTEGER[] for tags_to_add**
   ```sql
   tags_to_add INTEGER[] NOT NULL,
   ```
   - No foreign key validation (can reference non-existent tag IDs)
   - Orphaned tag IDs if tags are deleted

   **Mitigation Required:**
   - Add trigger to validate all tag IDs exist in tags table
   - Add ON DELETE cascade/restrict handling

4. **LOW: Helper function insert_tag_with_category is dropped at end**
   ```sql
   DROP FUNCTION IF EXISTS insert_tag_with_category(...);
   ```
   - Function not available for future use
   - Could be useful for manual tag insertion

   **Recommendation:** Keep function for operational use

#### Rollback Safety

**Rollback Script Required:** ✅ YES

**Rollback Complexity:** 🟠 **MEDIUM** (must preserve existing tags)

**Rollback Script (`007_enhanced_tags_rollback.sql`):**

```sql
-- ROLLBACK for Migration 007: Enhanced tags schema
-- PARTIAL DATA LOSS: New tag categories and suggestions lost
-- PRESERVES: Original tags table data

BEGIN;

-- Drop views
DROP VIEW IF EXISTS pending_tag_suggestions CASCADE;
DROP VIEW IF EXISTS popular_tags_by_category CASCADE;
DROP VIEW IF EXISTS tags_with_categories CASCADE;

-- Drop functions
DROP FUNCTION IF EXISTS suggest_tags_from_similar_files(BIGINT, INTEGER);
DROP FUNCTION IF EXISTS get_tags_by_category(VARCHAR);

-- Drop indexes on new columns
DROP INDEX IF EXISTS idx_file_tags_tag_id_file_id;
DROP INDEX IF EXISTS idx_tags_auto_detected;
DROP INDEX IF EXISTS idx_tags_priority;
DROP INDEX IF EXISTS idx_tags_category_id;
DROP INDEX IF EXISTS idx_tag_suggestions_accepted;
DROP INDEX IF EXISTS idx_tag_suggestions_confidence;
DROP INDEX IF EXISTS idx_tag_suggestions_file_id;
DROP INDEX IF EXISTS idx_auto_tagging_rules_active;
DROP INDEX IF EXISTS idx_auto_tagging_rules_type;
DROP INDEX IF EXISTS idx_tag_aliases_tag_id;
DROP INDEX IF EXISTS idx_tag_aliases_alias;

-- Drop new tables
DROP TABLE IF EXISTS tag_suggestions CASCADE;
DROP TABLE IF EXISTS auto_tagging_rules CASCADE;
DROP TABLE IF EXISTS tag_aliases CASCADE;

-- Drop new columns from tags table (PRESERVES existing tag data)
ALTER TABLE tags DROP COLUMN IF EXISTS is_active;
ALTER TABLE tags DROP COLUMN IF EXISTS parent_tag_id;
ALTER TABLE tags DROP COLUMN IF EXISTS detection_method;
ALTER TABLE tags DROP COLUMN IF EXISTS confidence_score;
ALTER TABLE tags DROP COLUMN IF EXISTS auto_detected;
ALTER TABLE tags DROP COLUMN IF EXISTS priority;
ALTER TABLE tags DROP COLUMN IF EXISTS category_id;

-- Drop tag_categories table
DROP TABLE IF EXISTS tag_categories CASCADE;

COMMIT;
```

**Rollback Risks:**
- ⚠️ All tag categorization data is lost
- ⚠️ Auto-tagging rules and suggestions are lost
- ✅ Original tags and file_tags relationships are preserved

---

### Migration 008: Filename Metadata ✅ SAFE

**File:** `008_filename_metadata_fixed.sql`
**Risk Level:** 🟢 **LOW**
**Lines of Code:** 55
**Transaction Safety:** ✅ Wrapped in BEGIN/COMMIT

#### Safety Analysis

**Strengths:**
- Transaction-safe
- Uses `IF NOT EXISTS` - idempotent
- CHECK constraints validate BPM range (30-300), key length (1-3 chars)
- ENUM constraint on metadata_source
- Partial indexes on WHERE NOT NULL

**Data Integrity Risks:**

1. **LOW: filename_key accepts TEXT but should probably be ENUM**
   ```sql
   ADD COLUMN IF NOT EXISTS filename_key TEXT
       CHECK (filename_key IS NULL OR LENGTH(filename_key) BETWEEN 1 AND 3);
   ```
   - Accepts any 1-3 character string ("XYZ", "!!!", etc.)
   - No validation against valid keys (C, Cm, D#, etc.)

   **Mitigation Required:**
   - Use existing `musical_key` ENUM type instead of TEXT
   - Or add CHECK constraint with valid key list

2. **LOW: No validation that filename_bpm matches musical_metadata.bpm**
   - Two sources of truth for BPM
   - Could diverge over time

   **Recommendation:** Add validation trigger or materialized view

#### Rollback Safety

**Rollback Script Required:** ✅ YES

**Rollback Complexity:** 🟢 **SIMPLE**

**Rollback Script (`008_filename_metadata_rollback.sql`):**

```sql
-- ROLLBACK for Migration 008: Filename metadata
-- DATA LOSS: Filename-extracted metadata is lost

BEGIN;

-- Drop indexes
DROP INDEX IF EXISTS idx_files_metadata_source;
DROP INDEX IF EXISTS idx_files_structure_tags;
DROP INDEX IF EXISTS idx_files_filename_genres;
DROP INDEX IF EXISTS idx_files_filename_key;
DROP INDEX IF EXISTS idx_files_filename_bpm;

-- Drop columns
ALTER TABLE files DROP COLUMN IF EXISTS metadata_source;
ALTER TABLE files DROP COLUMN IF EXISTS structure_tags;
ALTER TABLE files DROP COLUMN IF EXISTS filename_genres;
ALTER TABLE files DROP COLUMN IF EXISTS filename_key;
ALTER TABLE files DROP COLUMN IF EXISTS filename_bpm;

COMMIT;
```

**Rollback Risks:**
- ⚠️ Filename-extracted BPM/key/genre data is lost
- ✅ Can be regenerated by re-parsing filenames

---

### Migration 009: Text Metadata Extraction ✅ SAFE

**File:** `009_text_metadata.sql`
**Risk Level:** 🟢 **LOW**
**Lines of Code:** 248
**Transaction Safety:** ✅ Wrapped in BEGIN/COMMIT

#### Safety Analysis

**Strengths:**
- Transaction-safe
- Uses `IF NOT EXISTS` - idempotent
- Comprehensive text metadata extraction (track names, copyright, markers, lyrics)
- Helper functions for searching and statistics
- GIN indexes for array searches
- Good comments documenting column purposes

**Data Integrity Risks:**

1. **LOW: No validation that arrays aren't empty**
   ```sql
   track_names TEXT[] DEFAULT '{}'
   ```
   - Application could insert `'{}'` (empty array) vs NULL
   - Indexes check `array_length(..., 1) > 0` but data could have empty arrays

   **Mitigation Required:**
   - Normalize empty arrays to NULL on insert/update via trigger
   - Or change DEFAULT to NULL instead of '{}'

2. **LOW: Function search_files_by_text_metadata uses ILIKE (case-insensitive)**
   ```sql
   f.copyright ILIKE '%' || p_copyright_search || '%'
   ```
   - ILIKE can't use GIN index efficiently
   - Could be slow on large datasets

   **Mitigation Required:**
   - Add trigram index: `CREATE INDEX ... USING gin(copyright gin_trgm_ops);` (already done!)
   - Good - index already exists (line 86-88)

#### Rollback Safety

**Rollback Script Required:** ✅ YES

**Rollback Complexity:** 🟢 **SIMPLE**

**Rollback Script (`009_text_metadata_rollback.sql`):**

```sql
-- ROLLBACK for Migration 009: Text metadata extraction
-- DATA LOSS: MIDI text metadata (track names, copyright, markers, lyrics) is lost

BEGIN;

-- Drop views
DROP VIEW IF EXISTS files_with_text_metadata CASCADE;

-- Drop functions
DROP FUNCTION IF EXISTS get_text_metadata_stats();
DROP FUNCTION IF EXISTS search_files_by_text_metadata(TEXT, TEXT, TEXT, TEXT, BOOLEAN, INTEGER, INTEGER);

-- Drop indexes
DROP INDEX IF EXISTS idx_files_copyright_trgm;
DROP INDEX IF EXISTS idx_files_markers;
DROP INDEX IF EXISTS idx_files_instrument_names_text;
DROP INDEX IF EXISTS idx_files_track_names;
DROP INDEX IF EXISTS idx_files_copyright;

-- Drop columns
ALTER TABLE files DROP COLUMN IF EXISTS lyrics;
ALTER TABLE files DROP COLUMN IF EXISTS markers;
ALTER TABLE files DROP COLUMN IF EXISTS instrument_names_text;
ALTER TABLE files DROP COLUMN IF EXISTS copyright;
ALTER TABLE files DROP COLUMN IF EXISTS track_names;

COMMIT;
```

**Rollback Risks:**
- ⚠️ Extracted MIDI text metadata is lost
- ✅ Can be regenerated by re-parsing MIDI files

---

### Migration 010: Harmonic Analysis ✅ SAFE

**File:** `010_harmonic_analysis.sql`
**Risk Level:** 🟢 **LOW**
**Lines of Code:** 51
**Transaction Safety:** ❌ No explicit transaction

#### Safety Analysis

**Strengths:**
- Simple additive migration (6 columns + 5 indexes)
- CHECK constraint validates chord_complexity_score (0-1)
- Uses `IF NOT EXISTS` - idempotent
- Good comments documenting column purposes
- Partial indexes on WHERE NOT NULL

**Data Integrity Risks:**

1. **MEDIUM: Missing transaction wrapper**
   - 11 statements (6 ALTER TABLE + 5 CREATE INDEX)
   - If migration fails mid-execution, partial schema

   **Mitigation Required:**
   ```sql
   BEGIN;
   ALTER TABLE musical_metadata ADD COLUMN ...;
   ...
   CREATE INDEX ...;
   COMMIT;
   ```

2. **LOW: No validation that chord_progression JSON follows expected schema**
   ```sql
   ADD COLUMN IF NOT EXISTS chord_progression JSONB;
   ```
   - Free-form JSONB with no structure validation
   - Application could insert malformed data

   **Mitigation Required:**
   - Add JSONB schema validation via CHECK constraint
   - Document expected structure: `[{"tick": 0, "chord": "Cm"}, ...]`

#### Rollback Safety

**Rollback Script Required:** ✅ YES

**Rollback Complexity:** 🟢 **SIMPLE**

**Rollback Script (`010_harmonic_analysis_rollback.sql`):**

```sql
-- ROLLBACK for Migration 010: Harmonic analysis
-- DATA LOSS: Chord progression and harmonic analysis data is lost

BEGIN;

-- Drop indexes
DROP INDEX IF EXISTS idx_chord_complexity;
DROP INDEX IF EXISTS idx_chord_types;
DROP INDEX IF EXISTS idx_chord_progression;
DROP INDEX IF EXISTS idx_has_extended_chords;
DROP INDEX IF EXISTS idx_has_seventh_chords;

-- Drop columns
ALTER TABLE musical_metadata DROP COLUMN IF EXISTS chord_complexity_score;
ALTER TABLE musical_metadata DROP COLUMN IF EXISTS chord_change_rate;
ALTER TABLE musical_metadata DROP COLUMN IF EXISTS has_extended_chords;
ALTER TABLE musical_metadata DROP COLUMN IF EXISTS has_seventh_chords;
ALTER TABLE musical_metadata DROP COLUMN IF EXISTS chord_types;
ALTER TABLE musical_metadata DROP COLUMN IF EXISTS chord_progression;

COMMIT;
```

**Rollback Risks:**
- ⚠️ Harmonic analysis data is lost
- ✅ Can be regenerated by re-analyzing MIDI files

---

### Migration 011: Enhanced Analysis JSON Fields ✅ SAFE

**File:** `011_enhanced_analysis_json.sql`
**Risk Level:** 🟢 **LOW**
**Lines of Code:** 56
**Transaction Safety:** ❌ No explicit transaction

#### Safety Analysis

**Strengths:**
- Simple additive migration (3 columns + 3 indexes)
- JSONB columns allow flexible schema evolution
- Excellent comments documenting JSON structure
- Example queries demonstrating usage
- GIN indexes for efficient JSON queries

**Data Integrity Risks:**

1. **MEDIUM: Missing transaction wrapper**
   - 6 statements could partially succeed

   **Mitigation Required:**
   ```sql
   BEGIN;
   ALTER TABLE musical_metadata ADD COLUMN controller_data JSONB DEFAULT NULL;
   ALTER TABLE musical_metadata ADD COLUMN articulation_data JSONB DEFAULT NULL;
   ALTER TABLE musical_metadata ADD COLUMN structure_data JSONB DEFAULT NULL;
   CREATE INDEX idx_musical_metadata_controller_data ...;
   CREATE INDEX idx_musical_metadata_articulation_data ...;
   CREATE INDEX idx_musical_metadata_structure_data ...;
   COMMIT;
   ```

2. **MEDIUM: No JSONB schema validation**
   - Comments describe expected structure but no enforcement
   - Application could insert invalid JSON structure

   **Mitigation Required:**
   - Add CHECK constraints to validate JSON keys exist
   - Example:
   ```sql
   ADD CONSTRAINT check_articulation_schema CHECK (
       articulation_data IS NULL OR (
           articulation_data ? 'legato_percentage' AND
           articulation_data ? 'staccato_percentage'
       )
   );
   ```

3. **LOW: GIN indexes on JSONB could become large**
   - Indexing entire JSONB documents
   - If documents are large (many controllers), index bloat

   **Recommendation:** Monitor index size, consider partial indexes

#### Rollback Safety

**Rollback Script Required:** ✅ YES

**Rollback Complexity:** 🟢 **SIMPLE**

**Rollback Script (`011_enhanced_analysis_json_rollback.sql`):**

```sql
-- ROLLBACK for Migration 011: Enhanced analysis JSON fields
-- DATA LOSS: Controller, articulation, and structure analysis data is lost

BEGIN;

-- Drop indexes
DROP INDEX IF EXISTS idx_musical_metadata_structure_data;
DROP INDEX IF EXISTS idx_musical_metadata_articulation_data;
DROP INDEX IF EXISTS idx_musical_metadata_controller_data;

-- Drop columns
ALTER TABLE musical_metadata DROP COLUMN IF EXISTS structure_data;
ALTER TABLE musical_metadata DROP COLUMN IF EXISTS articulation_data;
ALTER TABLE musical_metadata DROP COLUMN IF EXISTS controller_data;

COMMIT;
```

**Rollback Risks:**
- ⚠️ Enhanced analysis data is lost
- ✅ Can be regenerated by re-analyzing MIDI files

---

### Data Migration: Update Normalized Filenames ⚠️ HIGH RISK

**File:** `update_normalized_filenames.sql`
**Risk Level:** 🔴 **HIGH**
**Lines of Code:** 76
**Transaction Safety:** ✅ Wrapped in BEGIN/COMMIT
**Operation Type:** ⚠️ **DATA MODIFICATION**

#### Safety Analysis

**Strengths:**
- Transaction-safe
- Creates reusable function `sanitize_filename()`
- Only updates rows that need changes (WHERE clause filter)
- Shows update statistics
- Verification query at end

**Data Integrity Risks:**

1. **CRITICAL: Updates filepath without verifying filesystem state**
   ```sql
   UPDATE files SET filepath = REPLACE(filepath, '/' || filename, '/' || sanitize_filename(filename))
   ```
   - Changes database filepath without moving actual files
   - Creates MISMATCH between database and filesystem
   - All file operations will fail after migration until files are renamed

   **Impact:** DATABASE CORRUPTION - files become inaccessible

   **REQUIRED BEFORE EXECUTION:**
   ```sql
   -- 1. Backup database
   pg_dump ... > backup_before_normalize.sql

   -- 2. Run filesystem normalization FIRST
   cargo run --bin normalize_filenames -- /path/to/midi-library

   -- 3. THEN run this migration to sync database
   psql ... -f update_normalized_filenames.sql

   -- 4. Verify sync
   SELECT COUNT(*) FROM files WHERE filepath NOT IN (
       SELECT unnest(string_to_array(pg_read_file(...), E'\n'))
   );
   ```

2. **HIGH: No rollback to original filenames**
   - Once updated, original filename/filepath is lost forever
   - Cannot revert to pre-normalized state

   **Mitigation Required:**
   ```sql
   -- Add columns to preserve originals BEFORE updating
   ALTER TABLE files ADD COLUMN IF NOT EXISTS original_filepath TEXT;
   ALTER TABLE files ADD COLUMN IF NOT EXISTS original_filename_pre_normalize TEXT;

   UPDATE files SET
       original_filepath = filepath,
       original_filename_pre_normalize = filename
   WHERE original_filepath IS NULL;

   -- THEN run normalization update
   UPDATE files SET ...
   ```

3. **HIGH: UNIQUE constraint on filepath could cause failures**
   ```sql
   filepath TEXT NOT NULL UNIQUE,
   ```
   - If normalization creates collisions (two different files → same normalized name)
   - UPDATE will fail with UNIQUE constraint violation
   - Transaction rolls back, but collision exists in filesystem

   **Example:**
   - `My Song (2023).mid` → `My_Song_2023.mid`
   - `My Song [2023].mid` → `My_Song_2023.mid`
   - COLLISION - only one can exist

   **Mitigation Required:**
   ```sql
   -- Detect collisions BEFORE updating
   WITH normalized_paths AS (
       SELECT id, filepath,
              REPLACE(filepath, '/' || filename, '/' || sanitize_filename(filename)) as new_path
       FROM files
   )
   SELECT new_path, COUNT(*) as collision_count
   FROM normalized_paths
   GROUP BY new_path
   HAVING COUNT(*) > 1;

   -- If collisions found, abort and fix manually
   ```

4. **MEDIUM: sanitize_filename function is IMMUTABLE but uses regex**
   - Marked IMMUTABLE for indexing, but regex behavior could change
   - PostgreSQL version upgrades could alter regex semantics

   **Impact:** Minimal, but good to document

5. **LOW: Function is dropped at end**
   ```sql
   DROP FUNCTION IF EXISTS sanitize_filename(TEXT);
   ```
   - Useful for future normalization, but removed
   - Should be kept as utility function

#### Rollback Safety

**Rollback Script Required:** ✅ YES (if original filenames preserved)

**Rollback Complexity:** 🔴 **VERY HIGH** (requires filesystem coordination)

**Rollback Script (`update_normalized_filenames_rollback.sql`):**

```sql
-- ROLLBACK for update_normalized_filenames.sql
-- WARNING: This requires BOTH database AND filesystem rollback
-- CRITICAL: Only works if original_filepath column was added before migration

BEGIN;

-- Verify original filenames were preserved
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'files' AND column_name = 'original_filepath'
    ) THEN
        RAISE EXCEPTION 'Cannot rollback: original_filepath column missing. BACKUP RESTORE REQUIRED.';
    END IF;
END $$;

-- Restore original filenames
UPDATE files
SET
    filename = original_filename_pre_normalize,
    filepath = original_filepath,
    updated_at = NOW()
WHERE original_filepath IS NOT NULL;

-- Verify restoration
DO $$
DECLARE
    restored_count INTEGER;
BEGIN
    GET DIAGNOSTICS restored_count = ROW_COUNT;
    RAISE NOTICE 'Restored % file records to original filenames', restored_count;
END $$;

-- Clean up backup columns (optional)
-- ALTER TABLE files DROP COLUMN IF EXISTS original_filepath;
-- ALTER TABLE files DROP COLUMN IF EXISTS original_filename_pre_normalize;

COMMIT;

-- CRITICAL: After database rollback, restore filesystem:
-- cargo run --bin restore_original_filenames -- /path/to/midi-library
```

**Rollback Risks:**
- 🔴 **IMPOSSIBLE without original filename preservation**
- 🔴 Requires filesystem rollback in addition to database
- 🔴 If filesystem and database get out of sync, manual recovery required

---

## Summary Table: Migration Risk Matrix

| Migration | Risk Level | Transaction Safe | Rollback Script | Data Loss Risk | Performance Impact |
|-----------|-----------|------------------|-----------------|----------------|-------------------|
| 001 Initial Schema | 🟢 LOW | ✅ Yes | ❌ Missing | ⚠️ CASCADE deletes | 🟢 Low (initial) |
| 002 Parent Folder | 🟢 LOW | ❌ No | ❌ Missing | 🟢 None | 🟢 Low |
| 003 Favorites | 🟢 LOW | ✅ Yes | ❌ Missing | ⚠️ User favorites | 🟢 Low |
| 006 Track Splits | 🟢 LOW | ❌ No | ❌ Missing | ⚠️ Relationship data | 🟢 Low |
| 007 Enhanced Tags | 🟠 MEDIUM | ✅ Yes | ❌ Missing | ⚠️ Tag categories | 🟡 Medium (inserts) |
| 008 Filename Metadata | 🟢 LOW | ✅ Yes | ❌ Missing | 🟢 None (regenerable) | 🟢 Low |
| 009 Text Metadata | 🟢 LOW | ✅ Yes | ❌ Missing | 🟢 None (regenerable) | 🟢 Low |
| 010 Harmonic Analysis | 🟢 LOW | ❌ No | ❌ Missing | 🟢 None (regenerable) | 🟢 Low |
| 011 Enhanced JSON | 🟢 LOW | ❌ No | ❌ Missing | 🟢 None (regenerable) | 🟢 Low |
| **UPDATE Normalized Filenames** | 🔴 **HIGH** | ✅ Yes | ❌ Missing | 🔴 **IRREVERSIBLE** | 🟠 Medium (UPDATE) |

---

## Critical Recommendations

### Priority 1: IMMEDIATE (Before ANY Production Deployment)

1. **Create rollback scripts for ALL 11 migrations**
   - Store in `database/migrations/rollback/` directory
   - Test each rollback on copy of production data
   - Document rollback order (reverse of application order)

2. **Fix update_normalized_filenames.sql DATA CORRUPTION RISK**
   ```sql
   -- Add to beginning of update_normalized_filenames.sql:

   -- STEP 1: Preserve original filenames
   ALTER TABLE files ADD COLUMN IF NOT EXISTS original_filepath TEXT;
   UPDATE files SET original_filepath = filepath WHERE original_filepath IS NULL;

   -- STEP 2: Detect collisions
   WITH collisions AS (
       SELECT sanitize_filename(filename) as normalized, COUNT(*) as cnt
       FROM files
       GROUP BY normalized
       HAVING COUNT(*) > 1
   )
   SELECT * FROM collisions;

   -- STEP 3: If collisions exist, ABORT with error message
   DO $$
   BEGIN
       IF EXISTS (SELECT 1 FROM collisions) THEN
           RAISE EXCEPTION 'Normalization would create filename collisions. Fix manually first.';
       END IF;
   END $$;

   -- STEP 4: Verify filesystem was normalized FIRST
   -- (Manual verification required - cannot automate)

   -- STEP 5: ONLY THEN proceed with UPDATE
   ```

3. **Add transaction wrappers to migrations 002, 006, 010, 011**
   - Ensures atomic operations
   - Prevents partial schema corruption

### Priority 2: HIGH (Before Scale-Up)

4. **Add CASCADE delete audit logging**
   ```sql
   CREATE TABLE deleted_files_audit (
       id BIGSERIAL PRIMARY KEY,
       deleted_file_id BIGINT NOT NULL,
       deleted_by TEXT,
       deletion_reason TEXT,
       cascaded_from BIGINT, -- parent that triggered cascade
       deleted_at TIMESTAMPTZ DEFAULT NOW(),
       file_metadata JSONB  -- snapshot of file record
   );

   -- Trigger on files DELETE to log cascades
   CREATE TRIGGER log_file_deletions
       BEFORE DELETE ON files
       FOR EACH ROW
       EXECUTE FUNCTION log_deleted_file();
   ```

5. **Add JSONB schema validation**
   - For tempo_changes, key_changes, chord_progression, controller_data, etc.
   - Prevents application bugs from inserting malformed JSON

6. **Implement periodic integrity checks**
   ```sql
   -- Verify tag counts match reality
   SELECT t.id, t.name, t.usage_count, COUNT(ft.file_id) as actual_count
   FROM tags t
   LEFT JOIN file_tags ft ON t.id = ft.tag_id
   GROUP BY t.id, t.name, t.usage_count
   HAVING t.usage_count != COUNT(ft.file_id);
   ```

### Priority 3: MEDIUM (Operational Excellence)

7. **Create migration testing framework**
   - Apply migration to test database
   - Verify schema matches expected
   - Run rollback
   - Verify database returns to original state
   - Automate in CI/CD pipeline

8. **Document migration dependencies**
   - Which migrations depend on others?
   - What order must they be applied/rolled back?
   - Create `database/migrations/MIGRATION_ORDER.md`

9. **Add pre-migration backup verification**
   ```bash
   #!/bin/bash
   # run_migration.sh

   MIGRATION_FILE=$1
   BACKUP_FILE="backup_before_$(basename $MIGRATION_FILE .sql)_$(date +%Y%m%d_%H%M%S).sql"

   echo "Creating backup: $BACKUP_FILE"
   pg_dump -h localhost -U midiuser -d midi_library > "$BACKUP_FILE"

   echo "Verifying backup..."
   if ! pg_restore --list "$BACKUP_FILE" > /dev/null; then
       echo "ERROR: Backup verification failed. Aborting migration."
       exit 1
   fi

   echo "Applying migration: $MIGRATION_FILE"
   psql -h localhost -U midiuser -d midi_library -f "$MIGRATION_FILE"

   if [ $? -eq 0 ]; then
       echo "Migration succeeded. Backup stored at: $BACKUP_FILE"
   else
       echo "Migration FAILED. Restore from backup: $BACKUP_FILE"
       exit 1
   fi
   ```

---

## Migration Execution Checklist

Before applying ANY migration to production:

- [ ] **Backup exists** and is verified restorable
- [ ] **Rollback script exists** and is tested on copy
- [ ] **Migration is in transaction** (BEGIN/COMMIT)
- [ ] **Tested on production-like dataset** (size, variety)
- [ ] **Application code supports new schema** (backward compatible)
- [ ] **Indexes analyzed** for performance impact (EXPLAIN plans)
- [ ] **Downtime window scheduled** (if long-running)
- [ ] **Monitoring alerts configured** for migration failures
- [ ] **Rollback procedure documented** and team trained

---

## Long-Running Migration Risks

### Migrations 001, 007 (INSERTS + INDEX CREATION)

**Performance Impact:** 🟠 MEDIUM

**Risk:** Index creation on large tables locks table for writes

**Mitigation:**
```sql
-- Create indexes CONCURRENTLY (PostgreSQL 9.2+)
CREATE INDEX CONCURRENTLY idx_files_content_hash ON files(content_hash);

-- Note: CONCURRENT cannot be inside transaction block
-- Must run outside BEGIN/COMMIT
```

**Estimated Duration (3M files):**
- Index on `files.content_hash`: ~2-5 minutes
- Index on `files.filepath`: ~2-5 minutes
- GIN index on `files.search_vector`: ~5-15 minutes
- **Total: 30-60 minutes of locks**

**Recommendation:**
- Run during maintenance window
- Use CONCURRENTLY for critical indexes
- Monitor `pg_stat_activity` for blocked queries

---

## Appendix: Rollback Testing Procedure

```bash
#!/bin/bash
# test_migration_rollback.sh

MIGRATION_FILE=$1
ROLLBACK_FILE=$2
TEST_DB="midi_library_rollback_test"

# 1. Create test database from production backup
createdb $TEST_DB
pg_restore -d $TEST_DB production_backup.dump

# 2. Capture schema before migration
pg_dump -s $TEST_DB > schema_before.sql

# 3. Apply migration
psql -d $TEST_DB -f $MIGRATION_FILE

# 4. Capture schema after migration
pg_dump -s $TEST_DB > schema_after.sql

# 5. Apply rollback
psql -d $TEST_DB -f $ROLLBACK_FILE

# 6. Capture schema after rollback
pg_dump -s $TEST_DB > schema_after_rollback.sql

# 7. Verify rollback restored original schema
if diff schema_before.sql schema_after_rollback.sql; then
    echo "✅ ROLLBACK TEST PASSED: Schema restored to original state"
    exit 0
else
    echo "❌ ROLLBACK TEST FAILED: Schema differs from original"
    diff schema_before.sql schema_after_rollback.sql
    exit 1
fi
```

---

## Conclusion

The MIDI Software Center migration suite requires **significant safety improvements** before production deployment:

1. **0/11 migrations have rollback scripts** - this is a critical gap
2. **1 migration modifies data** without preserving originals - corruption risk
3. **4 migrations lack transaction wrappers** - partial failure risk
4. **Multiple JSONB columns lack schema validation** - data quality risk

**Estimated effort to production-ready:**
- Rollback scripts: 8-12 hours
- Transaction safety fixes: 2-4 hours
- Data preservation updates: 4-6 hours
- Testing framework: 8-12 hours
- **Total: 22-34 hours**

**Risk if deployed as-is:**
- 🔴 **HIGH PROBABILITY** of data loss from failed rollbacks
- 🔴 **HIGH PROBABILITY** of database-filesystem desync from normalization migration
- 🟠 **MEDIUM PROBABILITY** of partial schema corruption from non-transactional migrations

**Recommendation:** DO NOT deploy to production until all Priority 1 items are complete.

---

**Report Generated:** 2025-11-29
**Auditor:** Data Integrity Guardian
**Next Review:** After rollback scripts implemented

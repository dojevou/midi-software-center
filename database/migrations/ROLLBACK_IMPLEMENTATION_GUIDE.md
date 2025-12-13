# Database Migration Rollback Implementation Guide

**MIDI Software Center - Production Safety Protocol**

---

## Executive Summary

**Current State:** 0/11 migrations have rollback scripts
**Target State:** 11/11 migrations with tested rollback scripts
**Estimated Effort:** 22-34 hours
**Priority:** CRITICAL - Required before production deployment

---

## Quick Start: Create Rollback Directory Structure

```bash
cd /home/dojevou/projects/midi-software-center/database/migrations

# Create rollback directory
mkdir -p rollback

# Create rollback tracking
cat > ROLLBACK_STATUS.md << 'EOF'
# Rollback Script Status

| Migration | Rollback Script | Tested | Notes |
|-----------|----------------|--------|-------|
| 001_initial_schema.sql | ❌ Missing | ❌ No | Critical - drops all data |
| 002_add_parent_folder.sql | ❌ Missing | ❌ No | Safe - drops column |
| 003_favorites.sql | ❌ Missing | ❌ No | Safe - drops table |
| 006_track_splits.sql | ❌ Missing | ❌ No | Safe - drops table |
| 007_enhanced_tags.sql | ❌ Missing | ❌ No | Medium - preserves tags |
| 008_filename_metadata_fixed.sql | ❌ Missing | ❌ No | Safe - drops columns |
| 009_text_metadata.sql | ❌ Missing | ❌ No | Safe - drops columns |
| 010_harmonic_analysis.sql | ❌ Missing | ❌ No | Safe - drops columns |
| 011_enhanced_analysis_json.sql | ❌ Missing | ❌ No | Safe - drops columns |
| update_normalized_filenames.sql | ❌ Missing | ❌ No | **CRITICAL - irreversible without prep** |
EOF
```

---

## Rollback Script Template

All rollback scripts follow this structure:

```sql
-- ============================================================================
-- ROLLBACK SCRIPT TEMPLATE
-- ============================================================================
-- Migration: [MIGRATION_NUMBER]_[MIGRATION_NAME].sql
-- Rollback: rollback/[MIGRATION_NUMBER]_[MIGRATION_NAME]_rollback.sql
-- Created: [DATE]
-- Risk Level: [LOW|MEDIUM|HIGH|CRITICAL]
-- Data Loss: [NONE|PARTIAL|COMPLETE]
-- ============================================================================

-- ============================================================================
-- PRE-ROLLBACK VALIDATION
-- ============================================================================

DO $$
BEGIN
    -- Verify migration was actually applied
    IF NOT EXISTS (
        SELECT 1 FROM schema_migrations WHERE version = '[VERSION]'
    ) THEN
        RAISE NOTICE 'Migration [VERSION] was never applied. Nothing to rollback.';
        -- Exit gracefully
        RETURN;
    END IF;

    -- Verify dependent migrations are not applied
    IF EXISTS (
        SELECT 1 FROM schema_migrations WHERE version IN ('[DEPENDENT_VERSIONS]')
    ) THEN
        RAISE EXCEPTION 'Cannot rollback: dependent migrations exist. Rollback those first.';
    END IF;

    RAISE NOTICE 'Pre-rollback validation passed. Proceeding with rollback.';
END $$;

-- ============================================================================
-- BACKUP CHECKPOINT
-- ============================================================================

-- Record rollback start
INSERT INTO schema_migrations (version, description)
VALUES ('[VERSION]_rollback_start', 'Started rollback of migration [VERSION] at ' || NOW())
ON CONFLICT (version) DO NOTHING;

-- ============================================================================
-- ROLLBACK OPERATIONS (Reverse order of migration)
-- ============================================================================

BEGIN;

-- [ROLLBACK STEPS HERE - IN REVERSE ORDER OF MIGRATION]

-- Example:
-- 1. Drop views (depend on tables/columns)
-- DROP VIEW IF EXISTS [view_name] CASCADE;

-- 2. Drop functions (depend on types/tables)
-- DROP FUNCTION IF EXISTS [function_name]([args]) CASCADE;

-- 3. Drop triggers
-- DROP TRIGGER IF EXISTS [trigger_name] ON [table_name];

-- 4. Drop indexes
-- DROP INDEX IF EXISTS [index_name];

-- 5. Drop constraints
-- ALTER TABLE [table] DROP CONSTRAINT IF EXISTS [constraint_name];

-- 6. Drop columns
-- ALTER TABLE [table] DROP COLUMN IF EXISTS [column_name];

-- 7. Drop tables
-- DROP TABLE IF EXISTS [table_name] CASCADE;

-- 8. Drop types
-- DROP TYPE IF EXISTS [type_name] CASCADE;

-- Record rollback completion
DELETE FROM schema_migrations WHERE version = '[VERSION]';

INSERT INTO schema_migrations (version, description)
VALUES ('[VERSION]_rollback_complete', 'Completed rollback of migration [VERSION] at ' || NOW())
ON CONFLICT (version) DO NOTHING;

COMMIT;

-- ============================================================================
-- POST-ROLLBACK VERIFICATION
-- ============================================================================

DO $$
DECLARE
    v_table_count INTEGER;
    v_column_count INTEGER;
BEGIN
    -- Verify objects were removed
    -- Example: Check table doesn't exist
    SELECT COUNT(*) INTO v_table_count
    FROM information_schema.tables
    WHERE table_schema = 'public' AND table_name = '[table_name]';

    IF v_table_count > 0 THEN
        RAISE WARNING 'Table [table_name] still exists after rollback';
    ELSE
        RAISE NOTICE 'Rollback verified: Table [table_name] removed';
    END IF;
END $$;

-- ============================================================================
-- ROLLBACK COMPLETE
-- ============================================================================
```

---

## Priority 1: Critical Rollback Scripts (IMMEDIATE)

### 1. update_normalized_filenames_rollback.sql (CRITICAL)

**Risk:** This migration modifies data irreversibly without preservation

**Required changes to original migration:**

```sql
-- FILE: update_normalized_filenames_SAFE.sql
-- MODIFIED VERSION with data preservation

BEGIN;

-- STEP 1: Add preservation columns BEFORE any updates
ALTER TABLE files ADD COLUMN IF NOT EXISTS original_filepath_backup TEXT;
ALTER TABLE files ADD COLUMN IF NOT EXISTS original_filename_backup TEXT;

-- STEP 2: Preserve original values
UPDATE files SET
    original_filepath_backup = filepath,
    original_filename_backup = filename
WHERE original_filepath_backup IS NULL;

-- STEP 3: Create sanitize function
CREATE OR REPLACE FUNCTION sanitize_filename(input TEXT) RETURNS TEXT AS $$
DECLARE
    result TEXT;
BEGIN
    result := input;
    result := REPLACE(result, ' ', '_');
    result := REGEXP_REPLACE(result, '[^a-zA-Z0-9_\-.]+', '', 'g');
    result := REGEXP_REPLACE(result, '\.(mid|MID|MIDI|midi|Mid|MiD)$', '.mid', 'i');
    result := REGEXP_REPLACE(result, '_+', '_', 'g');
    result := REGEXP_REPLACE(result, '-+', '-', 'g');
    result := REGEXP_REPLACE(result, '^[_-]+', '', 'g');
    result := REGEXP_REPLACE(result, '[_-]+$', '', 'g');
    RETURN result;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- STEP 4: Detect collisions BEFORE updating
DO $$
DECLARE
    v_collision_count INTEGER;
BEGIN
    WITH normalized AS (
        SELECT id, filename, sanitize_filename(filename) as new_filename
        FROM files
    ),
    collisions AS (
        SELECT new_filename, COUNT(*) as cnt
        FROM normalized
        GROUP BY new_filename
        HAVING COUNT(*) > 1
    )
    SELECT COUNT(*) INTO v_collision_count FROM collisions;

    IF v_collision_count > 0 THEN
        RAISE EXCEPTION 'Found % filename collisions after normalization. Aborting. Run query to see: WITH normalized AS (SELECT id, filename, sanitize_filename(filename) as new_filename FROM files), collisions AS (SELECT new_filename, COUNT(*) as cnt FROM normalized GROUP BY new_filename HAVING COUNT(*) > 1) SELECT * FROM collisions;', v_collision_count;
    END IF;

    RAISE NOTICE 'No collisions detected. Safe to proceed.';
END $$;

-- STEP 5: Update filenames (only if no collisions)
UPDATE files
SET
    filename = sanitize_filename(filename),
    filepath = REPLACE(
        filepath,
        '/' || filename,
        '/' || sanitize_filename(filename)
    ),
    updated_at = NOW()
WHERE
    filename != sanitize_filename(filename);

-- STEP 6: Verify update
DO $$
DECLARE
    updated_count INTEGER;
BEGIN
    GET DIAGNOSTICS updated_count = ROW_COUNT;
    RAISE NOTICE 'Updated % file records', updated_count;
END $$;

-- Keep function for operational use
-- DROP FUNCTION IF EXISTS sanitize_filename(TEXT);

COMMIT;
```

**Rollback script:**

```sql
-- FILE: rollback/update_normalized_filenames_rollback.sql

BEGIN;

-- Verify backup columns exist
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'files' AND column_name = 'original_filepath_backup'
    ) THEN
        RAISE EXCEPTION 'ROLLBACK IMPOSSIBLE: original_filepath_backup column missing. Database restore from backup required.';
    END IF;
END $$;

-- Restore original filenames
UPDATE files
SET
    filename = original_filename_backup,
    filepath = original_filepath_backup,
    updated_at = NOW()
WHERE original_filepath_backup IS NOT NULL;

-- Report restoration
DO $$
DECLARE
    restored_count INTEGER;
BEGIN
    GET DIAGNOSTICS restored_count = ROW_COUNT;
    RAISE NOTICE 'Restored % files to original filenames', restored_count;
END $$;

-- Optional: Clean up backup columns (keep for audit trail)
-- ALTER TABLE files DROP COLUMN IF EXISTS original_filepath_backup;
-- ALTER TABLE files DROP COLUMN IF EXISTS original_filename_backup;

COMMIT;

-- CRITICAL POST-ROLLBACK STEP:
-- Restore filesystem to original filenames:
-- cargo run --bin restore_original_filenames -- /home/dojevou/tmp/midi_splits_fast
```

---

### 2. 001_initial_schema_rollback.sql (CRITICAL - DATA LOSS)

```sql
-- FILE: rollback/001_initial_schema_rollback.sql
-- WARNING: This will DELETE ALL DATA in the database
-- BACKUP VERIFICATION REQUIRED before executing

DO $$
BEGIN
    RAISE WARNING 'This rollback will DELETE ALL DATA. Verify backup exists before continuing.';
    RAISE WARNING 'Type ROLLBACK; to abort, or wait 10 seconds...';
    PERFORM pg_sleep(10);
END $$;

BEGIN;

-- Record rollback in migrations table FIRST
INSERT INTO schema_migrations (version, description)
VALUES ('001_rollback_start', 'Started rollback of initial schema at ' || NOW())
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
DROP FUNCTION IF EXISTS update_job_progress() CASCADE;
DROP FUNCTION IF EXISTS update_duplicate_group_count() CASCADE;
DROP FUNCTION IF EXISTS update_tag_usage_count() CASCADE;
DROP FUNCTION IF EXISTS update_updated_at_column() CASCADE;
DROP FUNCTION IF EXISTS files_search_vector_update() CASCADE;

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

-- Drop enum types
DROP TYPE IF EXISTS musical_key CASCADE;
DROP TYPE IF EXISTS file_category CASCADE;

-- Drop extensions
DROP EXTENSION IF EXISTS pg_trgm CASCADE;
DROP EXTENSION IF EXISTS vector CASCADE;

-- Remove migration record
DELETE FROM schema_migrations WHERE version = '001';

-- Drop schema_migrations table LAST
DROP TABLE IF EXISTS schema_migrations CASCADE;

COMMIT;
```

---

## Priority 2: Simple Rollback Scripts (HIGH)

### Template for simple column additions (002, 008, 009, 010, 011)

```sql
-- FILE: rollback/[NUMBER]_[NAME]_rollback.sql

BEGIN;

-- Drop indexes
DROP INDEX IF EXISTS [index_name_1];
DROP INDEX IF EXISTS [index_name_2];
-- ... (all indexes created by migration)

-- Drop columns
ALTER TABLE [table_name] DROP COLUMN IF EXISTS [column_1];
ALTER TABLE [table_name] DROP COLUMN IF EXISTS [column_2];
-- ... (all columns added by migration)

-- Remove migration record
DELETE FROM schema_migrations WHERE version = '[VERSION]';

COMMIT;
```

### Example: 002_add_parent_folder_rollback.sql

```sql
-- FILE: rollback/002_add_parent_folder_rollback.sql

BEGIN;

-- Drop index
DROP INDEX IF EXISTS idx_files_parent_folder;

-- Drop column
ALTER TABLE files DROP COLUMN IF EXISTS parent_folder;

-- No schema_migrations record to remove (migration didn't add one)

COMMIT;
```

---

## Priority 3: Medium Complexity Rollbacks (003, 006, 007)

### 003_favorites_rollback.sql

```sql
-- FILE: rollback/003_favorites_rollback.sql

BEGIN;

-- Drop indexes
DROP INDEX IF EXISTS idx_favorites_created_at;
DROP INDEX IF EXISTS idx_favorites_file_id;

-- Drop table
DROP TABLE IF EXISTS favorites CASCADE;

-- Remove migration record
DELETE FROM schema_migrations WHERE version = '003';

COMMIT;
```

### 006_track_splits_rollback.sql

```sql
-- FILE: rollback/006_track_splits_rollback.sql

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

### 007_enhanced_tags_rollback.sql (Most Complex)

```sql
-- FILE: rollback/007_enhanced_tags_rollback.sql

BEGIN;

-- Drop views
DROP VIEW IF EXISTS pending_tag_suggestions CASCADE;
DROP VIEW IF EXISTS popular_tags_by_category CASCADE;
DROP VIEW IF EXISTS tags_with_categories CASCADE;

-- Drop functions
DROP FUNCTION IF EXISTS suggest_tags_from_similar_files(BIGINT, INTEGER) CASCADE;
DROP FUNCTION IF EXISTS get_tags_by_category(VARCHAR) CASCADE;

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

-- Drop new columns from tags table (PRESERVES original tag data)
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

---

## Rollback Testing Procedure

### 1. Create Test Database

```bash
#!/bin/bash
# test_rollback.sh [migration_number]

MIGRATION_NUM=$1
MIGRATION_FILE="database/migrations/${MIGRATION_NUM}_*.sql"
ROLLBACK_FILE="database/migrations/rollback/${MIGRATION_NUM}_*_rollback.sql"
TEST_DB="midi_test_rollback_${MIGRATION_NUM}"

# Create test database
echo "Creating test database: $TEST_DB"
createdb $TEST_DB

# Initialize with base schema
psql -d $TEST_DB -f database/migrations/001_initial_schema.sql

# Apply migrations up to target
for migration in database/migrations/00{2..9}_*.sql; do
    if [[ $migration < $MIGRATION_FILE ]]; then
        echo "Applying prerequisite: $migration"
        psql -d $TEST_DB -f $migration
    fi
done

# Capture schema before target migration
echo "Capturing schema before migration..."
pg_dump -s $TEST_DB > /tmp/schema_before_${MIGRATION_NUM}.sql

# Apply target migration
echo "Applying target migration: $MIGRATION_FILE"
psql -d $TEST_DB -f $MIGRATION_FILE

# Capture schema after migration
echo "Capturing schema after migration..."
pg_dump -s $TEST_DB > /tmp/schema_after_${MIGRATION_NUM}.sql

# Apply rollback
echo "Applying rollback: $ROLLBACK_FILE"
psql -d $TEST_DB -f $ROLLBACK_FILE

# Capture schema after rollback
echo "Capturing schema after rollback..."
pg_dump -s $TEST_DB > /tmp/schema_after_rollback_${MIGRATION_NUM}.sql

# Compare schemas
echo "Comparing schemas..."
if diff /tmp/schema_before_${MIGRATION_NUM}.sql /tmp/schema_after_rollback_${MIGRATION_NUM}.sql > /tmp/schema_diff_${MIGRATION_NUM}.txt; then
    echo "✅ ROLLBACK TEST PASSED: Schema restored to original state"
    dropdb $TEST_DB
    exit 0
else
    echo "❌ ROLLBACK TEST FAILED: Schema differs from original"
    echo "Differences saved to: /tmp/schema_diff_${MIGRATION_NUM}.txt"
    echo "Test database preserved: $TEST_DB"
    cat /tmp/schema_diff_${MIGRATION_NUM}.txt
    exit 1
fi
```

### 2. Test All Rollbacks

```bash
#!/bin/bash
# test_all_rollbacks.sh

MIGRATIONS=(001 002 003 006 007 008 009 010 011)

for migration in "${MIGRATIONS[@]}"; do
    echo "========================================="
    echo "Testing rollback for migration: $migration"
    echo "========================================="

    ./test_rollback.sh $migration

    if [ $? -ne 0 ]; then
        echo "❌ Rollback test failed for migration $migration"
        echo "Fix rollback script before continuing"
        exit 1
    fi
done

echo "✅ ALL ROLLBACK TESTS PASSED"
```

---

## Deployment Checklist

Before applying ANY migration to production:

- [ ] Rollback script exists in `rollback/` directory
- [ ] Rollback script tested on copy of production data
- [ ] Full database backup created and verified restorable
- [ ] Migration wrapped in transaction (BEGIN/COMMIT)
- [ ] Dependent migrations identified and documented
- [ ] Downtime window scheduled (if needed)
- [ ] Rollback procedure documented and team trained
- [ ] Monitoring configured to detect migration failures

**Production Deployment Order:**

1. Schedule maintenance window
2. Announce downtime to users
3. Create full database backup
4. Verify backup is restorable
5. Apply migration
6. Verify migration success (check counts, query data)
7. Run smoke tests on application
8. If any issues: ROLLBACK immediately
9. Announce service restored

---

## Emergency Rollback Procedure

If migration fails in production:

```bash
#!/bin/bash
# emergency_rollback.sh [migration_number]

MIGRATION_NUM=$1
ROLLBACK_SCRIPT="database/migrations/rollback/${MIGRATION_NUM}_*_rollback.sql"
DB_NAME="midi_library"
BACKUP_FILE="backup_before_migration_${MIGRATION_NUM}_*.sql"

echo "🚨 EMERGENCY ROLLBACK INITIATED 🚨"
echo "Migration: $MIGRATION_NUM"
echo "Database: $DB_NAME"

# Verify rollback script exists
if [ ! -f $ROLLBACK_SCRIPT ]; then
    echo "❌ CRITICAL: Rollback script not found: $ROLLBACK_SCRIPT"
    echo "Manual database restore required from backup: $BACKUP_FILE"
    exit 1
fi

# Create emergency backup BEFORE rollback
echo "Creating emergency backup before rollback..."
pg_dump -h localhost -U midiuser -d $DB_NAME > "emergency_backup_before_rollback_$(date +%Y%m%d_%H%M%S).sql"

# Apply rollback
echo "Applying rollback script..."
psql -h localhost -U midiuser -d $DB_NAME -f $ROLLBACK_SCRIPT

if [ $? -eq 0 ]; then
    echo "✅ Rollback completed successfully"
    echo "Verify application functionality before resuming service"
else
    echo "❌ ROLLBACK FAILED"
    echo "Restore from backup: $BACKUP_FILE"
    exit 1
fi
```

---

## Next Steps

1. **Create all rollback scripts** (use templates above)
2. **Test each rollback** using test_rollback.sh
3. **Update original migrations** (add transactions, data preservation)
4. **Document migration dependencies** in MIGRATION_ORDER.md
5. **Create deployment runbook** for production team

**Estimated Timeline:**
- Day 1: Create rollback scripts (001, 002, 003) - 4 hours
- Day 2: Create rollback scripts (006, 007, 008, 009) - 4 hours
- Day 3: Create rollback scripts (010, 011, update_normalized) - 4 hours
- Day 4: Test all rollbacks - 4 hours
- Day 5: Update original migrations, create runbook - 4 hours
- **Total: 20 hours over 5 days**

---

**Document Version:** 1.0
**Last Updated:** 2025-11-29
**Owner:** Data Integrity Guardian

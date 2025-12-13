-- Add indexes for fast tagging operations
-- Run with: psql "postgresql://..." -f add_tagging_indexes.sql

\echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
\echo '  Adding Tagging Performance Indexes'
\echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
\echo ''

\timing on

-- 1. Index on lowercase filepath for pattern matching
\echo 'Creating index on LOWER(filepath)...'
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_files_filepath_lower
  ON files (LOWER(filepath));

-- 2. Index on lowercase filename for pattern matching
\echo 'Creating index on LOWER(filename)...'
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_files_filename_lower
  ON files (LOWER(filename));

-- 3. Index on lowercase tag name for lookups
\echo 'Creating index on LOWER(name) in tags...'
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tags_name_lower
  ON tags (LOWER(name));

-- 4. Composite index on file_tags for efficient joins
\echo 'Creating composite index on file_tags...'
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_file_tags_composite
  ON file_tags (file_id, tag_id);

-- 5. Unique constraint to prevent duplicate tags
\echo 'Creating unique constraint on file_tags...'
CREATE UNIQUE INDEX CONCURRENTLY IF NOT EXISTS idx_file_tags_unique
  ON file_tags (file_id, tag_id);

-- 6. Index on tag category for filtering
\echo 'Creating index on tag category...'
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tags_category
  ON tags (category);

\echo ''
\echo '✅ All tagging indexes created successfully!'
\echo ''

-- Show index sizes
\echo '📊 Index Sizes:'
SELECT
  schemaname,
  tablename,
  indexname,
  pg_size_pretty(pg_relation_size(indexrelid)) as index_size
FROM pg_stat_user_indexes
WHERE indexname LIKE 'idx_files_%'
   OR indexname LIKE 'idx_tags_%'
   OR indexname LIKE 'idx_file_tags_%'
ORDER BY indexname;

\echo ''
\echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
\echo '  Indexes Ready for Fast Tagging!'
\echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'

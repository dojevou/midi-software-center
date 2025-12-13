-- ============================================================================
-- RESTORE SAFETY - Return PostgreSQL to Safe Production Settings
-- ============================================================================
-- Created: November 18, 2025
-- Purpose: Restore crash safety after bulk import
-- Run this IMMEDIATELY after bulk import completes!
-- ============================================================================

-- ============================================================================
-- 1. RESTORE CRASH SAFETY
-- ============================================================================

-- Re-enable disk synchronization (safe writes)
ALTER SYSTEM SET fsync = 'on';

-- Synchronous commits (full durability)
ALTER SYSTEM SET synchronous_commit = 'on';

-- Full page writes (prevent corruption)
ALTER SYSTEM SET full_page_writes = 'on';

-- Standard WAL logging
ALTER SYSTEM SET wal_level = 'replica';

-- Standard WAL size
ALTER SYSTEM SET max_wal_size = '2GB';

-- Normal checkpoints (5 minutes)
ALTER SYSTEM SET checkpoint_timeout = '5min';

-- ============================================================================
-- 2. RE-ENABLE MAINTENANCE
-- ============================================================================

-- Re-enable autovacuum (cleanup and statistics)
ALTER SYSTEM SET autovacuum = 'on';

-- Re-enable activity tracking
ALTER SYSTEM SET track_activities = 'on';

-- Re-enable statistics
ALTER SYSTEM SET track_counts = 'on';

-- ============================================================================
-- 3. KEEP OPTIMIZED MEMORY SETTINGS (Safe to keep)
-- ============================================================================

-- These settings are safe and improve performance:
-- - shared_buffers = 16GB
-- - effective_cache_size = 45GB
-- - maintenance_work_mem = 4GB
-- - work_mem = 512MB
-- - max_worker_processes = 64

-- ============================================================================
-- 4. RELOAD CONFIGURATION
-- ============================================================================

SELECT pg_reload_conf();

-- ============================================================================
-- 5. CONVERT TABLES BACK TO LOGGED (If using UNLOGGED)
-- ============================================================================

-- Uncomment if you used UNLOGGED tables:
-- ALTER TABLE files SET LOGGED;
-- ALTER TABLE musical_metadata SET LOGGED;
-- ALTER TABLE tags SET LOGGED;
-- ALTER TABLE file_tags SET LOGGED;

-- ============================================================================
-- 6. REBUILD INDEXES (If dropped during import)
-- ============================================================================

-- Run this if you dropped indexes before import:
-- \i /path/to/INDEX_BACKUP.sql

-- ============================================================================
-- 7. ANALYZE TABLES (Update statistics for query planner)
-- ============================================================================

ANALYZE files;
ANALYZE musical_metadata;
ANALYZE tags;
ANALYZE file_tags;
ANALYZE track_splits;
ANALYZE search_index;

-- Verbose analyze for verification
ANALYZE VERBOSE files;

-- ============================================================================
-- 8. VACUUM TABLES (Reclaim space and update visibility map)
-- ============================================================================

VACUUM ANALYZE files;
VACUUM ANALYZE musical_metadata;
VACUUM ANALYZE tags;
VACUUM ANALYZE file_tags;

-- ============================================================================
-- 9. VERIFY SAFETY RESTORED
-- ============================================================================

-- Check critical settings are restored
SELECT name, setting, unit, boot_val, reset_val
FROM pg_settings
WHERE name IN (
    'fsync',
    'synchronous_commit',
    'full_page_writes',
    'wal_level',
    'autovacuum',
    'track_activities',
    'track_counts'
)
ORDER BY name;

-- Should see:
-- fsync = on
-- synchronous_commit = on
-- full_page_writes = on
-- wal_level = replica
-- autovacuum = on

-- ============================================================================
-- 10. DATABASE HEALTH CHECK
-- ============================================================================

-- Check table sizes
SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size,
    pg_size_pretty(pg_relation_size(schemaname||'.'||tablename)) AS table_size,
    pg_size_pretty(pg_indexes_size(schemaname||'.'||tablename)) AS indexes_size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

-- Check index count
SELECT
    tablename,
    COUNT(*) as index_count
FROM pg_indexes
WHERE schemaname = 'public'
GROUP BY tablename
ORDER BY tablename;

-- ============================================================================
-- SAFETY RESTORED!
-- ============================================================================
-- PostgreSQL is now safe for production use.
-- - Crash safety: ENABLED
-- - Autovacuum: ENABLED
-- - Statistics: ENABLED
-- - Performance: OPTIMIZED (memory settings retained)
-- ============================================================================

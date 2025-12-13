-- ============================================================================
-- ULTRA-FAST PostgreSQL Configuration for MIDI Pipeline
-- ============================================================================
-- Created: November 18, 2025
-- Purpose: Maximum import speed configuration
-- WARNING: UNSAFE for production! Use ONLY during bulk import!
-- ============================================================================

-- ============================================================================
-- 1. ULTRA-FAST BULK LOADING (UNSAFE - Import Only!)
-- ============================================================================

-- Disable disk synchronization (10x faster writes, NO crash safety!)
ALTER SYSTEM SET fsync = 'off';

-- Async commits (3x faster, small data loss window if crash)
ALTER SYSTEM SET synchronous_commit = 'off';

-- No full page writes (faster, risk of corruption if crash)
ALTER SYSTEM SET full_page_writes = 'off';

-- Minimal WAL logging
ALTER SYSTEM SET wal_level = 'minimal';

-- Huge WAL buffer (16GB)
ALTER SYSTEM SET max_wal_size = '16GB';

-- Rare checkpoints (1 hour)
ALTER SYSTEM SET checkpoint_timeout = '1h';

-- Smooth checkpoints
ALTER SYSTEM SET checkpoint_completion_target = 0.9;

-- ============================================================================
-- 2. MEMORY ALLOCATION (Optimized for 60GB RAM System)
-- ============================================================================

-- 16GB shared buffer cache (25% of RAM)
ALTER SYSTEM SET shared_buffers = '16GB';

-- 45GB effective cache (75% of RAM estimate)
ALTER SYSTEM SET effective_cache_size = '45GB';

-- 4GB maintenance memory (index building, VACUUM, etc.)
ALTER SYSTEM SET maintenance_work_mem = '4GB';

-- 512MB per-operation memory (sorts, hash joins)
ALTER SYSTEM SET work_mem = '512MB';

-- 256MB temp table cache
ALTER SYSTEM SET temp_buffers = '256MB';

-- ============================================================================
-- 3. DISABLE DURING IMPORT
-- ============================================================================

-- No autovacuum during import
ALTER SYSTEM SET autovacuum = 'off';

-- No activity tracking
ALTER SYSTEM SET track_activities = 'off';

-- No statistics
ALTER SYSTEM SET track_counts = 'off';

-- ============================================================================
-- 4. CONNECTION POOL (Support 48+ workers)
-- ============================================================================

ALTER SYSTEM SET max_connections = 200;
ALTER SYSTEM SET superuser_reserved_connections = 10;

-- ============================================================================
-- 5. PARALLEL QUERY EXECUTION (Use All 16 Cores)
-- ============================================================================

-- All cores available
ALTER SYSTEM SET max_worker_processes = 64;
ALTER SYSTEM SET max_parallel_workers = 64;
ALTER SYSTEM SET max_parallel_workers_per_gather = 32;
ALTER SYSTEM SET max_parallel_maintenance_workers = 16;

-- Prefer parallel plans
ALTER SYSTEM SET parallel_setup_cost = 0;
ALTER SYSTEM SET parallel_tuple_cost = 0;
ALTER SYSTEM SET min_parallel_table_scan_size = '1MB';
ALTER SYSTEM SET min_parallel_index_scan_size = '512kB';

-- ============================================================================
-- 6. QUERY PLANNER OPTIMIZATIONS
-- ============================================================================

-- Enable all join types
ALTER SYSTEM SET enable_hashjoin = 'on';
ALTER SYSTEM SET enable_mergejoin = 'on';
ALTER SYSTEM SET enable_nestloop = 'on';

-- Enable bitmapscan (faster for large result sets)
ALTER SYSTEM SET enable_bitmapscan = 'on';

-- Increase cost threshold (allow expensive plans for faster overall)
ALTER SYSTEM SET from_collapse_limit = 20;
ALTER SYSTEM SET join_collapse_limit = 20;

-- ============================================================================
-- 7. RELOAD CONFIGURATION
-- ============================================================================

SELECT pg_reload_conf();

-- ============================================================================
-- 8. VERIFY CONFIGURATION
-- ============================================================================

-- Check critical settings
SELECT name, setting, unit, boot_val, reset_val
FROM pg_settings
WHERE name IN (
    'fsync',
    'synchronous_commit',
    'shared_buffers',
    'effective_cache_size',
    'max_connections',
    'max_worker_processes',
    'autovacuum'
)
ORDER BY name;

-- ============================================================================
-- REMINDER: RESTORE SAFETY AFTER IMPORT!
-- ============================================================================
-- Run ../restore-safety.sql after bulk import completes!
-- ============================================================================

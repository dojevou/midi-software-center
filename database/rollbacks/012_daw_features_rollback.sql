-- ============================================================================
-- ROLLBACK: Migration 012 - DAW Features
-- ============================================================================
-- Description: Rollback script for DAW features migration
-- Version: 012
-- Date: 2025-11-30
--
-- WARNING: This rollback will PERMANENTLY DELETE all data in the affected tables.
-- Ensure you have a backup before proceeding.
--
-- Usage:
--   psql "postgresql://midiuser:145278963@localhost:5433/midi_library" \
--     -f database/rollbacks/012_daw_features_rollback.sql
--
-- Verification:
--   After rollback, verify migration_012_daw_features_applied is FALSE
-- ============================================================================

-- Wrap in transaction for atomic rollback
BEGIN;

-- ============================================================================
-- PRE-ROLLBACK VERIFICATION
-- ============================================================================

DO $$
DECLARE
    migration_applied BOOLEAN;
BEGIN
    -- Check if migration was applied
    SELECT EXISTS (
        SELECT 1 FROM information_schema.tables
        WHERE table_schema = 'public'
        AND table_name = 'daw_projects'
    ) INTO migration_applied;

    IF NOT migration_applied THEN
        RAISE NOTICE 'Migration 012 does not appear to be applied. Nothing to rollback.';
        -- We'll continue anyway to clean up any partial state
    ELSE
        RAISE NOTICE 'Migration 012 detected. Proceeding with rollback...';
    END IF;
END $$;

-- ============================================================================
-- DROP VIEWS (must be dropped before tables they depend on)
-- ============================================================================

DROP VIEW IF EXISTS repair_statistics CASCADE;
DROP VIEW IF EXISTS daw_notes_with_track CASCADE;

RAISE NOTICE 'Views dropped successfully';

-- ============================================================================
-- DROP TRIGGERS
-- ============================================================================

-- Drop triggers on daw_projects
DROP TRIGGER IF EXISTS update_daw_projects_updated_at ON daw_projects;

-- Drop triggers on daw_tracks
DROP TRIGGER IF EXISTS update_daw_tracks_updated_at ON daw_tracks;

-- Drop triggers on effect_presets
DROP TRIGGER IF EXISTS update_effect_presets_updated_at ON effect_presets;

-- Drop triggers on track_effects
DROP TRIGGER IF EXISTS update_track_effects_updated_at ON track_effects;

-- Drop triggers on settings_profiles
DROP TRIGGER IF EXISTS update_settings_profiles_updated_at ON settings_profiles;

-- Drop triggers on repair_batch_jobs
DROP TRIGGER IF EXISTS update_repair_batch_jobs_updated_at ON repair_batch_jobs;

DO $$ BEGIN RAISE NOTICE 'Triggers dropped successfully'; END $$;

-- ============================================================================
-- DROP INDEXES (explicit drop for clarity, CASCADE on tables would handle these)
-- ============================================================================

-- daw_projects indexes
DROP INDEX IF EXISTS idx_daw_projects_name;
DROP INDEX IF EXISTS idx_daw_projects_created_at;
DROP INDEX IF EXISTS idx_daw_projects_updated_at;

-- daw_tracks indexes
DROP INDEX IF EXISTS idx_daw_tracks_project_id;
DROP INDEX IF EXISTS idx_daw_tracks_channel;
DROP INDEX IF EXISTS idx_daw_tracks_instrument_name;
DROP INDEX IF EXISTS idx_daw_tracks_project_position;
DROP INDEX IF EXISTS idx_daw_tracks_muted_soloed;

-- daw_notes indexes
DROP INDEX IF EXISTS idx_daw_notes_track_id;
DROP INDEX IF EXISTS idx_daw_notes_pitch;
DROP INDEX IF EXISTS idx_daw_notes_start_tick;
DROP INDEX IF EXISTS idx_daw_notes_channel;
DROP INDEX IF EXISTS idx_daw_notes_track_pitch;
DROP INDEX IF EXISTS idx_daw_notes_track_start;
DROP INDEX IF EXISTS idx_daw_notes_velocity;

-- daw_automation_lanes indexes
DROP INDEX IF EXISTS idx_daw_automation_lanes_track_id;
DROP INDEX IF EXISTS idx_daw_automation_lanes_parameter;
DROP INDEX IF EXISTS idx_daw_automation_lanes_track_parameter;

-- daw_automation_points indexes
DROP INDEX IF EXISTS idx_daw_automation_points_lane_id;
DROP INDEX IF EXISTS idx_daw_automation_points_tick;
DROP INDEX IF EXISTS idx_daw_automation_points_lane_tick;

-- effect_presets indexes
DROP INDEX IF EXISTS idx_effect_presets_effect_type;
DROP INDEX IF EXISTS idx_effect_presets_name;
DROP INDEX IF EXISTS idx_effect_presets_is_factory;
DROP INDEX IF EXISTS idx_effect_presets_parameters;

-- track_effects indexes
DROP INDEX IF EXISTS idx_track_effects_track_id;
DROP INDEX IF EXISTS idx_track_effects_effect_type;
DROP INDEX IF EXISTS idx_track_effects_preset_id;
DROP INDEX IF EXISTS idx_track_effects_track_position;
DROP INDEX IF EXISTS idx_track_effects_enabled;
DROP INDEX IF EXISTS idx_track_effects_parameters;

-- settings_profiles indexes
DROP INDEX IF EXISTS idx_settings_profiles_name;
DROP INDEX IF EXISTS idx_settings_profiles_is_default;
DROP INDEX IF EXISTS idx_settings_profiles_category;
DROP INDEX IF EXISTS idx_settings_profiles_settings;

-- settings_history indexes
DROP INDEX IF EXISTS idx_settings_history_profile_id;
DROP INDEX IF EXISTS idx_settings_history_changed_at;
DROP INDEX IF EXISTS idx_settings_history_setting_key;
DROP INDEX IF EXISTS idx_settings_history_profile_changed;

-- midi_repair_log indexes
DROP INDEX IF EXISTS idx_midi_repair_log_file_id;
DROP INDEX IF EXISTS idx_midi_repair_log_corruption_type;
DROP INDEX IF EXISTS idx_midi_repair_log_repair_strategy;
DROP INDEX IF EXISTS idx_midi_repair_log_success;
DROP INDEX IF EXISTS idx_midi_repair_log_repaired_at;
DROP INDEX IF EXISTS idx_midi_repair_log_batch_job_id;
DROP INDEX IF EXISTS idx_midi_repair_log_severity;

-- repair_batch_jobs indexes
DROP INDEX IF EXISTS idx_repair_batch_jobs_status;
DROP INDEX IF EXISTS idx_repair_batch_jobs_started_at;
DROP INDEX IF EXISTS idx_repair_batch_jobs_completed_at;

-- undo_history indexes
DROP INDEX IF EXISTS idx_undo_history_project_id;
DROP INDEX IF EXISTS idx_undo_history_action_type;
DROP INDEX IF EXISTS idx_undo_history_created_at;
DROP INDEX IF EXISTS idx_undo_history_undone_at;
DROP INDEX IF EXISTS idx_undo_history_project_created;

DO $$ BEGIN RAISE NOTICE 'Indexes dropped successfully'; END $$;

-- ============================================================================
-- DROP TABLES (in reverse dependency order)
-- ============================================================================

-- Undo/redo system
DROP TABLE IF EXISTS undo_history CASCADE;

-- Repair system (child tables first)
DROP TABLE IF EXISTS midi_repair_log CASCADE;
DROP TABLE IF EXISTS repair_batch_jobs CASCADE;

-- Settings system (child tables first)
DROP TABLE IF EXISTS settings_history CASCADE;
DROP TABLE IF EXISTS settings_profiles CASCADE;

-- Effects system (child tables first)
DROP TABLE IF EXISTS track_effects CASCADE;
DROP TABLE IF EXISTS effect_presets CASCADE;

-- DAW/Piano roll system (child tables first, respecting foreign keys)
DROP TABLE IF EXISTS daw_automation_points CASCADE;
DROP TABLE IF EXISTS daw_automation_lanes CASCADE;
DROP TABLE IF EXISTS daw_notes CASCADE;
DROP TABLE IF EXISTS daw_tracks CASCADE;
DROP TABLE IF EXISTS daw_projects CASCADE;

DO $$ BEGIN RAISE NOTICE 'Tables dropped successfully'; END $$;

-- ============================================================================
-- DROP ENUM TYPES (after tables that use them are dropped)
-- ============================================================================

DROP TYPE IF EXISTS repair_strategy CASCADE;
DROP TYPE IF EXISTS corruption_severity CASCADE;
DROP TYPE IF EXISTS corruption_type CASCADE;
DROP TYPE IF EXISTS effect_type CASCADE;

DO $$ BEGIN RAISE NOTICE 'Enum types dropped successfully'; END $$;

-- ============================================================================
-- DROP HELPER FUNCTIONS (if any were created)
-- ============================================================================

-- Drop the update_updated_at_column function if it was created by this migration
-- Note: Only drop if no other tables use it
DO $$
DECLARE
    trigger_count INTEGER;
BEGIN
    -- Count how many triggers use this function
    SELECT COUNT(*) INTO trigger_count
    FROM information_schema.triggers
    WHERE action_statement LIKE '%update_updated_at_column%';

    IF trigger_count = 0 THEN
        -- Safe to drop the function
        DROP FUNCTION IF EXISTS update_updated_at_column() CASCADE;
        RAISE NOTICE 'Helper function update_updated_at_column dropped';
    ELSE
        RAISE NOTICE 'Helper function update_updated_at_column kept (used by % other triggers)', trigger_count;
    END IF;
END $$;

-- ============================================================================
-- UPDATE MIGRATION TRACKING (if you have a migrations table)
-- ============================================================================

-- If you have a schema_migrations or similar table, remove this migration record
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.tables
        WHERE table_schema = 'public'
        AND table_name = 'schema_migrations'
    ) THEN
        DELETE FROM schema_migrations WHERE version = '012';
        RAISE NOTICE 'Removed migration 012 from schema_migrations table';
    END IF;
END $$;

-- ============================================================================
-- POST-ROLLBACK VERIFICATION
-- ============================================================================

DO $$
DECLARE
    remaining_tables TEXT[];
    remaining_types TEXT[];
    table_name TEXT;
    type_name TEXT;
BEGIN
    -- Check for any remaining tables
    SELECT ARRAY_AGG(t.table_name) INTO remaining_tables
    FROM information_schema.tables t
    WHERE t.table_schema = 'public'
    AND t.table_name IN (
        'daw_projects', 'daw_tracks', 'daw_notes',
        'daw_automation_lanes', 'daw_automation_points',
        'effect_presets', 'track_effects',
        'settings_profiles', 'settings_history',
        'midi_repair_log', 'repair_batch_jobs',
        'undo_history'
    );

    IF remaining_tables IS NOT NULL AND array_length(remaining_tables, 1) > 0 THEN
        RAISE WARNING 'Some tables were not dropped: %', remaining_tables;
    END IF;

    -- Check for any remaining enum types
    SELECT ARRAY_AGG(t.typname) INTO remaining_types
    FROM pg_type t
    JOIN pg_catalog.pg_namespace n ON n.oid = t.typnamespace
    WHERE n.nspname = 'public'
    AND t.typname IN ('effect_type', 'corruption_type', 'corruption_severity', 'repair_strategy');

    IF remaining_types IS NOT NULL AND array_length(remaining_types, 1) > 0 THEN
        RAISE WARNING 'Some enum types were not dropped: %', remaining_types;
    END IF;

    -- Final status
    IF (remaining_tables IS NULL OR array_length(remaining_tables, 1) IS NULL)
       AND (remaining_types IS NULL OR array_length(remaining_types, 1) IS NULL) THEN
        RAISE NOTICE '========================================';
        RAISE NOTICE 'ROLLBACK SUCCESSFUL: Migration 012 completely reverted';
        RAISE NOTICE '========================================';
    ELSE
        RAISE WARNING '========================================';
        RAISE WARNING 'ROLLBACK INCOMPLETE: Some objects may remain';
        RAISE WARNING '========================================';
    END IF;
END $$;

-- ============================================================================
-- COMMIT TRANSACTION
-- ============================================================================

COMMIT;

-- ============================================================================
-- POST-COMMIT MAINTENANCE (optional, run manually if needed)
-- ============================================================================

-- After rollback, you may want to run:
-- VACUUM ANALYZE;
-- REINDEX DATABASE midi_library;

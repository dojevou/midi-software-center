-- =============================================================================
-- MIDI Library System - DAW Features Migration
-- =============================================================================
-- Migration: 012_daw_features.sql
-- Version: 1.0
-- PostgreSQL: 16+
-- Date: 2025-12-01
-- Description: Adds tables for piano roll, effects, settings profiles, and repair operations
-- =============================================================================

BEGIN;

-- =============================================================================
-- ENUM TYPES
-- =============================================================================

-- Effect types supported by the DAW
CREATE TYPE effect_type AS ENUM (
    'reverb',
    'delay',
    'chorus',
    'flanger',
    'phaser',
    'tremolo',
    'vibrato',
    'distortion',
    'overdrive',
    'bitcrusher',
    'compressor',
    'limiter',
    'gate',
    'expander',
    'equalizer',
    'filter',
    'lowpass',
    'highpass',
    'bandpass',
    'notch',
    'pitch_shift',
    'harmonizer',
    'vocoder',
    'autopan',
    'stereo_widener',
    'gain',
    'custom'
);

-- MIDI corruption types
CREATE TYPE corruption_type AS ENUM (
    'missing_end_of_track',
    'invalid_header',
    'truncated_file',
    'invalid_track_length',
    'garbage_at_end',
    'invalid_delta_time',
    'invalid_event_type',
    'invalid_running_status',
    'invalid_meta_event',
    'invalid_sysex_event',
    'missing_track_header',
    'track_count_mismatch',
    'nested_sysex',
    'invalid_channel',
    'invalid_note',
    'invalid_velocity',
    'file_too_large',
    'unknown'
);

-- Corruption severity levels
CREATE TYPE corruption_severity AS ENUM (
    'low',
    'medium',
    'high',
    'critical'
);

-- Repair strategies
CREATE TYPE repair_strategy AS ENUM (
    'add_end_of_track',
    'trim_garbage',
    'fix_track_length',
    'fix_track_count',
    'remove_invalid_events',
    'clamp_values',
    'rebuild',
    'conservative',
    'aggressive',
    'auto'
);

-- =============================================================================
-- PIANO ROLL TABLES
-- =============================================================================

-- -----------------------------------------------------------------------------
-- TABLE: daw_projects
-- PURPOSE: DAW project files for session management
-- EXPECTED ROWS: 10,000+
-- -----------------------------------------------------------------------------
CREATE TABLE daw_projects (
    id BIGSERIAL PRIMARY KEY,

    -- Project identification
    name TEXT NOT NULL,
    description TEXT,

    -- Project settings
    tempo NUMERIC(6, 2) DEFAULT 120.0 CHECK (tempo >= 20 AND tempo <= 400),
    time_signature_numerator SMALLINT DEFAULT 4 CHECK (time_signature_numerator >= 1 AND time_signature_numerator <= 32),
    time_signature_denominator SMALLINT DEFAULT 4 CHECK (time_signature_denominator IN (1, 2, 4, 8, 16, 32)),
    ppqn INTEGER DEFAULT 480 CHECK (ppqn >= 24 AND ppqn <= 960),

    -- Loop/range settings
    loop_enabled BOOLEAN DEFAULT FALSE,
    loop_start_tick BIGINT DEFAULT 0,
    loop_end_tick BIGINT DEFAULT 1920,

    -- Project state
    current_position BIGINT DEFAULT 0,
    is_playing BOOLEAN DEFAULT FALSE,
    is_recording BOOLEAN DEFAULT FALSE,

    -- Metadata
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_saved_at TIMESTAMPTZ,
    version INTEGER DEFAULT 1
);

-- -----------------------------------------------------------------------------
-- TABLE: daw_tracks
-- PURPOSE: Track definitions within a DAW project
-- EXPECTED ROWS: 100,000+
-- -----------------------------------------------------------------------------
CREATE TABLE daw_tracks (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT NOT NULL REFERENCES daw_projects(id) ON DELETE CASCADE,

    -- Track identification
    name TEXT NOT NULL,
    track_number INTEGER NOT NULL,

    -- MIDI settings
    channel SMALLINT DEFAULT 0 CHECK (channel >= 0 AND channel <= 15),
    program_number SMALLINT DEFAULT 0 CHECK (program_number >= 0 AND program_number <= 127),
    bank_msb SMALLINT DEFAULT 0 CHECK (bank_msb >= 0 AND bank_msb <= 127),
    bank_lsb SMALLINT DEFAULT 0 CHECK (bank_lsb >= 0 AND bank_lsb <= 127),

    -- Track state
    is_muted BOOLEAN DEFAULT FALSE,
    is_solo BOOLEAN DEFAULT FALSE,
    is_armed BOOLEAN DEFAULT FALSE,

    -- Volume and pan
    volume NUMERIC(5, 3) DEFAULT 1.0 CHECK (volume >= 0 AND volume <= 2.0),
    pan NUMERIC(5, 3) DEFAULT 0.0 CHECK (pan >= -1.0 AND pan <= 1.0),

    -- Display
    color TEXT DEFAULT '#3B82F6',
    height INTEGER DEFAULT 100,
    collapsed BOOLEAN DEFAULT FALSE,

    -- Reference to source file (if imported)
    source_file_id BIGINT REFERENCES files(id) ON DELETE SET NULL,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(project_id, track_number)
);

-- -----------------------------------------------------------------------------
-- TABLE: daw_notes
-- PURPOSE: MIDI notes in the piano roll
-- EXPECTED ROWS: 10,000,000+
-- -----------------------------------------------------------------------------
CREATE TABLE daw_notes (
    id BIGSERIAL PRIMARY KEY,
    track_id BIGINT NOT NULL REFERENCES daw_tracks(id) ON DELETE CASCADE,

    -- Note properties
    pitch SMALLINT NOT NULL CHECK (pitch >= 0 AND pitch <= 127),
    velocity SMALLINT NOT NULL DEFAULT 100 CHECK (velocity >= 0 AND velocity <= 127),
    start_tick BIGINT NOT NULL CHECK (start_tick >= 0),
    duration_ticks INTEGER NOT NULL CHECK (duration_ticks > 0),

    -- MIDI channel (can override track default)
    channel SMALLINT CHECK (channel IS NULL OR (channel >= 0 AND channel <= 15)),

    -- Selection/editing state (transient, but useful for undo/redo)
    is_selected BOOLEAN DEFAULT FALSE,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- -----------------------------------------------------------------------------
-- TABLE: daw_automation_lanes
-- PURPOSE: Automation data for tracks
-- EXPECTED ROWS: 500,000+
-- -----------------------------------------------------------------------------
CREATE TABLE daw_automation_lanes (
    id BIGSERIAL PRIMARY KEY,
    track_id BIGINT NOT NULL REFERENCES daw_tracks(id) ON DELETE CASCADE,

    -- Lane identification
    name TEXT NOT NULL,
    parameter_type TEXT NOT NULL, -- 'volume', 'pan', 'cc1', 'cc7', etc.
    parameter_id INTEGER, -- CC number, or NULL for built-in parameters

    -- Display settings
    height INTEGER DEFAULT 60,
    collapsed BOOLEAN DEFAULT FALSE,
    color TEXT,

    -- Range settings
    min_value NUMERIC(8, 4) DEFAULT 0,
    max_value NUMERIC(8, 4) DEFAULT 1,
    default_value NUMERIC(8, 4) DEFAULT 0.5,

    -- State
    is_enabled BOOLEAN DEFAULT TRUE,
    is_visible BOOLEAN DEFAULT TRUE,

    created_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(track_id, parameter_type, parameter_id)
);

-- -----------------------------------------------------------------------------
-- TABLE: daw_automation_points
-- PURPOSE: Individual automation points within lanes
-- EXPECTED ROWS: 5,000,000+
-- -----------------------------------------------------------------------------
CREATE TABLE daw_automation_points (
    id BIGSERIAL PRIMARY KEY,
    lane_id BIGINT NOT NULL REFERENCES daw_automation_lanes(id) ON DELETE CASCADE,

    -- Point properties
    tick BIGINT NOT NULL CHECK (tick >= 0),
    value NUMERIC(8, 4) NOT NULL,

    -- Curve properties
    curve_type TEXT DEFAULT 'linear' CHECK (curve_type IN ('linear', 'bezier', 'step', 'smooth', 'exponential', 'logarithmic')),
    tension NUMERIC(3, 2) DEFAULT 0.0 CHECK (tension >= -1.0 AND tension <= 1.0),

    -- For bezier curves
    control_x1 NUMERIC(8, 4),
    control_y1 NUMERIC(8, 4),
    control_x2 NUMERIC(8, 4),
    control_y2 NUMERIC(8, 4),

    created_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(lane_id, tick)
);

-- =============================================================================
-- EFFECTS TABLES
-- =============================================================================

-- -----------------------------------------------------------------------------
-- TABLE: effect_presets
-- PURPOSE: Reusable effect presets
-- EXPECTED ROWS: 50,000+
-- -----------------------------------------------------------------------------
CREATE TABLE effect_presets (
    id BIGSERIAL PRIMARY KEY,

    -- Preset identification
    name TEXT NOT NULL,
    description TEXT,
    effect_type effect_type NOT NULL,

    -- Preset data
    parameters JSONB NOT NULL DEFAULT '{}',

    -- Categorization
    category TEXT,
    tags TEXT[],

    -- Metadata
    author TEXT,
    is_factory BOOLEAN DEFAULT FALSE,
    is_favorite BOOLEAN DEFAULT FALSE,
    usage_count INTEGER DEFAULT 0,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- -----------------------------------------------------------------------------
-- TABLE: track_effects
-- PURPOSE: Effects applied to tracks
-- EXPECTED ROWS: 200,000+
-- -----------------------------------------------------------------------------
CREATE TABLE track_effects (
    id BIGSERIAL PRIMARY KEY,
    track_id BIGINT NOT NULL REFERENCES daw_tracks(id) ON DELETE CASCADE,

    -- Effect properties
    effect_type effect_type NOT NULL,
    name TEXT NOT NULL,
    slot_index INTEGER NOT NULL CHECK (slot_index >= 0),

    -- Effect state
    is_enabled BOOLEAN DEFAULT TRUE,
    is_bypassed BOOLEAN DEFAULT FALSE,

    -- Current parameters
    parameters JSONB NOT NULL DEFAULT '{}',

    -- Mix settings
    wet_mix NUMERIC(3, 2) DEFAULT 1.0 CHECK (wet_mix >= 0 AND wet_mix <= 1.0),
    dry_mix NUMERIC(3, 2) DEFAULT 0.0 CHECK (dry_mix >= 0 AND dry_mix <= 1.0),
    output_gain NUMERIC(4, 2) DEFAULT 0.0, -- dB

    -- Link to preset (optional)
    preset_id BIGINT REFERENCES effect_presets(id) ON DELETE SET NULL,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(track_id, slot_index)
);

-- =============================================================================
-- SETTINGS TABLES
-- =============================================================================

-- -----------------------------------------------------------------------------
-- TABLE: settings_profiles
-- PURPOSE: User settings profiles for backup/restore
-- EXPECTED ROWS: 1,000+
-- -----------------------------------------------------------------------------
CREATE TABLE settings_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Profile identification
    name TEXT NOT NULL,
    description TEXT,

    -- Profile data (full settings JSON)
    settings_data JSONB NOT NULL,

    -- Categories included (for partial profiles)
    categories_included TEXT[] DEFAULT ARRAY['all'],

    -- Metadata
    version TEXT NOT NULL DEFAULT '1.0',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(name)
);

-- -----------------------------------------------------------------------------
-- TABLE: settings_history
-- PURPOSE: Track settings changes for audit/undo
-- EXPECTED ROWS: 100,000+
-- -----------------------------------------------------------------------------
CREATE TABLE settings_history (
    id BIGSERIAL PRIMARY KEY,

    -- Change details
    category TEXT NOT NULL,
    setting_key TEXT NOT NULL,
    old_value JSONB,
    new_value JSONB,

    -- Change metadata
    changed_at TIMESTAMPTZ DEFAULT NOW(),
    changed_by TEXT DEFAULT 'user',
    change_reason TEXT
);

-- =============================================================================
-- REPAIR TABLES
-- =============================================================================

-- -----------------------------------------------------------------------------
-- TABLE: midi_repair_log
-- PURPOSE: Track MIDI file repairs
-- EXPECTED ROWS: 500,000+
-- -----------------------------------------------------------------------------
CREATE TABLE midi_repair_log (
    id BIGSERIAL PRIMARY KEY,
    file_id BIGINT REFERENCES files(id) ON DELETE SET NULL,

    -- Original file info
    original_filepath TEXT NOT NULL,
    original_size BIGINT,
    original_hash BYTEA,

    -- Corruption detected
    corruption_types corruption_type[] NOT NULL,
    corruption_severity corruption_severity NOT NULL,
    corruption_details JSONB,

    -- Repair action
    strategy_used repair_strategy NOT NULL,

    -- Results
    success BOOLEAN NOT NULL,
    error_message TEXT,

    -- Repaired file info
    repaired_filepath TEXT,
    repaired_size BIGINT,
    repaired_hash BYTEA,

    -- Backup info
    backup_filepath TEXT,

    -- Statistics
    events_removed INTEGER DEFAULT 0,
    events_fixed INTEGER DEFAULT 0,
    bytes_changed INTEGER DEFAULT 0,

    -- Timing
    repair_duration_ms INTEGER,
    repaired_at TIMESTAMPTZ DEFAULT NOW()
);

-- -----------------------------------------------------------------------------
-- TABLE: repair_batch_jobs
-- PURPOSE: Track batch repair operations
-- EXPECTED ROWS: 10,000+
-- -----------------------------------------------------------------------------
CREATE TABLE repair_batch_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Job details
    source_directory TEXT NOT NULL,
    options JSONB NOT NULL DEFAULT '{}',

    -- Progress
    total_files INTEGER DEFAULT 0,
    processed_files INTEGER DEFAULT 0,
    successful_repairs INTEGER DEFAULT 0,
    failed_repairs INTEGER DEFAULT 0,
    skipped_files INTEGER DEFAULT 0,

    -- Status
    status TEXT DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed', 'cancelled')),
    error_message TEXT,

    -- Timing
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,

    -- Statistics
    total_bytes_processed BIGINT DEFAULT 0,
    total_bytes_saved BIGINT DEFAULT 0,
    avg_repair_time_ms NUMERIC(8, 2),

    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- UNDO/REDO TABLES
-- =============================================================================

-- -----------------------------------------------------------------------------
-- TABLE: undo_history
-- PURPOSE: Track undo/redo operations for the DAW
-- EXPECTED ROWS: 1,000,000+
-- -----------------------------------------------------------------------------
CREATE TABLE undo_history (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT NOT NULL REFERENCES daw_projects(id) ON DELETE CASCADE,

    -- Operation details
    operation_type TEXT NOT NULL,
    operation_name TEXT NOT NULL,

    -- State snapshots (for complex operations)
    before_state JSONB,
    after_state JSONB,

    -- For simple operations, just track the changes
    changes JSONB,

    -- Position in undo stack
    sequence_number INTEGER NOT NULL,

    -- Timing
    performed_at TIMESTAMPTZ DEFAULT NOW(),

    -- Status
    is_undone BOOLEAN DEFAULT FALSE,
    undone_at TIMESTAMPTZ
);

-- =============================================================================
-- INDEXES
-- =============================================================================

-- DAW Projects indexes
CREATE INDEX idx_daw_projects_name ON daw_projects(name);
CREATE INDEX idx_daw_projects_updated ON daw_projects(updated_at DESC);

-- DAW Tracks indexes
CREATE INDEX idx_daw_tracks_project ON daw_tracks(project_id);
CREATE INDEX idx_daw_tracks_source ON daw_tracks(source_file_id) WHERE source_file_id IS NOT NULL;
CREATE INDEX idx_daw_tracks_channel ON daw_tracks(channel);

-- DAW Notes indexes
CREATE INDEX idx_daw_notes_track ON daw_notes(track_id);
CREATE INDEX idx_daw_notes_pitch ON daw_notes(pitch);
CREATE INDEX idx_daw_notes_start ON daw_notes(start_tick);
CREATE INDEX idx_daw_notes_range ON daw_notes(track_id, start_tick, (start_tick + duration_ticks));
CREATE INDEX idx_daw_notes_selected ON daw_notes(track_id, is_selected) WHERE is_selected = TRUE;

-- Automation indexes
CREATE INDEX idx_automation_lanes_track ON daw_automation_lanes(track_id);
CREATE INDEX idx_automation_lanes_param ON daw_automation_lanes(parameter_type);
CREATE INDEX idx_automation_points_lane ON daw_automation_points(lane_id);
CREATE INDEX idx_automation_points_tick ON daw_automation_points(lane_id, tick);

-- Effects indexes
CREATE INDEX idx_effect_presets_type ON effect_presets(effect_type);
CREATE INDEX idx_effect_presets_category ON effect_presets(category) WHERE category IS NOT NULL;
CREATE INDEX idx_effect_presets_favorite ON effect_presets(is_favorite) WHERE is_favorite = TRUE;
CREATE INDEX idx_effect_presets_tags ON effect_presets USING GIN(tags);
CREATE INDEX idx_track_effects_track ON track_effects(track_id);
CREATE INDEX idx_track_effects_type ON track_effects(effect_type);
CREATE INDEX idx_track_effects_preset ON track_effects(preset_id) WHERE preset_id IS NOT NULL;

-- Settings indexes
CREATE INDEX idx_settings_profiles_name ON settings_profiles(name);
CREATE INDEX idx_settings_history_category ON settings_history(category);
CREATE INDEX idx_settings_history_time ON settings_history(changed_at DESC);

-- Repair indexes
CREATE INDEX idx_repair_log_file ON midi_repair_log(file_id) WHERE file_id IS NOT NULL;
CREATE INDEX idx_repair_log_path ON midi_repair_log(original_filepath);
CREATE INDEX idx_repair_log_success ON midi_repair_log(success);
CREATE INDEX idx_repair_log_severity ON midi_repair_log(corruption_severity);
CREATE INDEX idx_repair_log_time ON midi_repair_log(repaired_at DESC);
CREATE INDEX idx_repair_batch_status ON repair_batch_jobs(status);
CREATE INDEX idx_repair_batch_created ON repair_batch_jobs(created_at DESC);

-- Undo indexes
CREATE INDEX idx_undo_project ON undo_history(project_id);
CREATE INDEX idx_undo_sequence ON undo_history(project_id, sequence_number DESC);
CREATE INDEX idx_undo_undone ON undo_history(project_id, is_undone);

-- =============================================================================
-- TRIGGERS
-- =============================================================================

-- Update timestamps for DAW projects
CREATE OR REPLACE FUNCTION update_daw_project_timestamp() RETURNS trigger AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER daw_projects_updated_at
    BEFORE UPDATE ON daw_projects
    FOR EACH ROW
    EXECUTE FUNCTION update_daw_project_timestamp();

-- Update timestamps for DAW tracks
CREATE TRIGGER daw_tracks_updated_at
    BEFORE UPDATE ON daw_tracks
    FOR EACH ROW
    EXECUTE FUNCTION update_daw_project_timestamp();

-- Update timestamps for DAW notes
CREATE TRIGGER daw_notes_updated_at
    BEFORE UPDATE ON daw_notes
    FOR EACH ROW
    EXECUTE FUNCTION update_daw_project_timestamp();

-- Update timestamps for effect presets
CREATE TRIGGER effect_presets_updated_at
    BEFORE UPDATE ON effect_presets
    FOR EACH ROW
    EXECUTE FUNCTION update_daw_project_timestamp();

-- Update timestamps for track effects
CREATE TRIGGER track_effects_updated_at
    BEFORE UPDATE ON track_effects
    FOR EACH ROW
    EXECUTE FUNCTION update_daw_project_timestamp();

-- Update timestamps for settings profiles
CREATE TRIGGER settings_profiles_updated_at
    BEFORE UPDATE ON settings_profiles
    FOR EACH ROW
    EXECUTE FUNCTION update_daw_project_timestamp();

-- =============================================================================
-- VIEWS
-- =============================================================================

-- View for track notes with track info
CREATE VIEW daw_notes_with_track AS
SELECT
    n.id,
    n.track_id,
    n.pitch,
    n.velocity,
    n.start_tick,
    n.duration_ticks,
    n.channel,
    n.is_selected,
    t.name as track_name,
    t.project_id,
    t.color as track_color,
    t.is_muted,
    t.is_solo
FROM daw_notes n
JOIN daw_tracks t ON n.track_id = t.id;

-- View for repair statistics
CREATE VIEW repair_statistics AS
SELECT
    COUNT(*) as total_repairs,
    COUNT(*) FILTER (WHERE success = TRUE) as successful_repairs,
    COUNT(*) FILTER (WHERE success = FALSE) as failed_repairs,
    SUM(events_removed) as total_events_removed,
    SUM(events_fixed) as total_events_fixed,
    SUM(bytes_changed) as total_bytes_changed,
    AVG(repair_duration_ms) as avg_repair_duration_ms,
    MAX(repaired_at) as last_repair_at
FROM midi_repair_log;

-- =============================================================================
-- COMMENTS
-- =============================================================================

COMMENT ON TABLE daw_projects IS 'DAW project files with tempo, time signature, and loop settings';
COMMENT ON TABLE daw_tracks IS 'Track definitions within DAW projects including MIDI settings';
COMMENT ON TABLE daw_notes IS 'MIDI notes for piano roll editing';
COMMENT ON TABLE daw_automation_lanes IS 'Automation lanes for track parameters (volume, pan, CC)';
COMMENT ON TABLE daw_automation_points IS 'Individual automation points with curve types';
COMMENT ON TABLE effect_presets IS 'Reusable effect parameter presets';
COMMENT ON TABLE track_effects IS 'Effects applied to tracks with parameters and routing';
COMMENT ON TABLE settings_profiles IS 'User settings backup/restore profiles';
COMMENT ON TABLE settings_history IS 'Audit log of settings changes';
COMMENT ON TABLE midi_repair_log IS 'Log of MIDI file repairs with before/after details';
COMMENT ON TABLE repair_batch_jobs IS 'Batch repair job tracking';
COMMENT ON TABLE undo_history IS 'Undo/redo stack for DAW operations';

-- =============================================================================
-- RECORD MIGRATION
-- =============================================================================

INSERT INTO schema_migrations (version, description)
VALUES ('012', 'DAW features - piano roll, effects, settings profiles, repair operations, undo/redo');

-- =============================================================================
-- VERIFY MIGRATION
-- =============================================================================

DO $$
DECLARE
    table_count INTEGER;
    index_count INTEGER;
BEGIN
    -- Count new tables
    SELECT COUNT(*) INTO table_count
    FROM information_schema.tables
    WHERE table_schema = 'public'
    AND table_name IN (
        'daw_projects', 'daw_tracks', 'daw_notes',
        'daw_automation_lanes', 'daw_automation_points',
        'effect_presets', 'track_effects',
        'settings_profiles', 'settings_history',
        'midi_repair_log', 'repair_batch_jobs',
        'undo_history'
    );

    -- Count new indexes (rough count)
    SELECT COUNT(*) INTO index_count
    FROM pg_indexes
    WHERE schemaname = 'public'
    AND indexname LIKE 'idx_daw_%'
       OR indexname LIKE 'idx_effect_%'
       OR indexname LIKE 'idx_track_effects_%'
       OR indexname LIKE 'idx_settings_%'
       OR indexname LIKE 'idx_repair_%'
       OR indexname LIKE 'idx_undo_%'
       OR indexname LIKE 'idx_automation_%';

    RAISE NOTICE 'Migration 012 verification:';
    RAISE NOTICE '  New tables created: % (expected 12)', table_count;
    RAISE NOTICE '  New indexes created: % (expected 25+)', index_count;

    IF table_count < 12 THEN
        RAISE EXCEPTION 'Expected 12 new tables, only created %', table_count;
    END IF;

    RAISE NOTICE 'Migration 012 completed successfully';
END $$;

COMMIT;

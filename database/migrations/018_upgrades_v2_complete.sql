-- =============================================================================
-- MIDI Software Center - Upgrades V2 Complete Migration
-- =============================================================================
-- Migration: 018_upgrades_v2_complete.sql
-- Version: 2.0
-- PostgreSQL: 16+
-- Date: 2025-12-11
-- Description: Complete implementation of all tables from upgrades.txt specification
--              Adds Mixer, MIDI I/O, Preferences, Gear, Presets, and History tables
-- =============================================================================

BEGIN;

-- =============================================================================
-- SECTION 1: PROJECTS DOMAIN (Extensions)
-- =============================================================================

-- Project markers (song sections, cue points)
CREATE TABLE IF NOT EXISTS project_markers (
    id              BIGSERIAL PRIMARY KEY,
    project_id      BIGINT NOT NULL REFERENCES daw_projects(id) ON DELETE CASCADE,

    name            VARCHAR(100) NOT NULL,
    tick            BIGINT NOT NULL,
    color           VARCHAR(7),
    marker_type     VARCHAR(20) DEFAULT 'marker'     -- marker, loop_start, loop_end, cue
);

-- Project clips (references to MIDI files placed on timeline)
CREATE TABLE IF NOT EXISTS project_clips (
    id              BIGSERIAL PRIMARY KEY,
    track_id        BIGINT NOT NULL REFERENCES daw_tracks(id) ON DELETE CASCADE,
    midi_file_id    BIGINT REFERENCES files(id),     -- NULL if recorded/edited in place

    -- Timeline position
    start_tick      BIGINT NOT NULL,                 -- Position on timeline
    length_ticks    BIGINT NOT NULL,                 -- Duration (may differ from source)

    -- Clip adjustments
    offset_ticks    BIGINT DEFAULT 0,                -- Offset into source MIDI
    loop_count      SMALLINT DEFAULT 1,              -- How many times to loop

    -- Clip-level transforms
    transpose       SMALLINT DEFAULT 0,
    velocity_scale  SMALLINT DEFAULT 100,

    -- Visual
    name            VARCHAR(100),                    -- Override display name
    color           VARCHAR(7),                      -- Override color

    -- If edited (not from library)
    custom_midi_data BYTEA,                          -- Inline MIDI if modified

    created_at      TIMESTAMPTZ DEFAULT NOW(),
    modified_at     TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- SECTION 2: MIDI I/O CONFIGURATION
-- =============================================================================

-- Physical MIDI ports (discovered from system)
CREATE TABLE IF NOT EXISTS midi_ports (
    id              BIGSERIAL PRIMARY KEY,

    -- System identification
    system_name     VARCHAR(255) NOT NULL,           -- OS-level port name
    system_id       VARCHAR(255),                    -- Unique system identifier

    -- User customization
    display_name    VARCHAR(100),                    -- User-friendly name
    alias           VARCHAR(50),                     -- Short name: "Port A", "Synth Out"

    -- Port type
    direction       VARCHAR(10) NOT NULL,            -- 'input' or 'output'
    port_type       VARCHAR(20) DEFAULT 'hardware',  -- hardware, virtual, network

    -- Connection state (updated at runtime, persisted for UI)
    is_connected    BOOLEAN DEFAULT FALSE,
    last_seen_at    TIMESTAMPTZ,

    -- Default behavior
    enabled         BOOLEAN DEFAULT TRUE,            -- User can disable ports
    auto_connect    BOOLEAN DEFAULT TRUE,            -- Reconnect on startup

    -- For outputs: default clock/transport behavior
    send_clock      BOOLEAN DEFAULT FALSE,
    send_transport  BOOLEAN DEFAULT TRUE,            -- Start/Stop/Continue

    -- For inputs: routing
    route_to        VARCHAR(20) DEFAULT 'selected',  -- selected, all, none

    UNIQUE(system_name, direction)
);

-- Port groups (for multi-port interfaces)
CREATE TABLE IF NOT EXISTS midi_port_groups (
    id              BIGSERIAL PRIMARY KEY,
    name            VARCHAR(100) NOT NULL,           -- "Focusrite Scarlett", "MOTU 828"
    icon            VARCHAR(50)
);

CREATE TABLE IF NOT EXISTS midi_port_group_members (
    port_group_id   BIGINT REFERENCES midi_port_groups(id) ON DELETE CASCADE,
    port_id         BIGINT REFERENCES midi_ports(id) ON DELETE CASCADE,
    sort_order      SMALLINT,
    PRIMARY KEY (port_group_id, port_id)
);

-- MIDI routing rules (advanced)
CREATE TABLE IF NOT EXISTS midi_routes (
    id              BIGSERIAL PRIMARY KEY,
    name            VARCHAR(100),
    enabled         BOOLEAN DEFAULT TRUE,

    -- Source
    input_port_id   BIGINT REFERENCES midi_ports(id),
    input_channel   SMALLINT,                        -- NULL = all channels

    -- Destination
    output_port_id  BIGINT REFERENCES midi_ports(id),
    output_channel  SMALLINT,                        -- NULL = pass through

    -- Filter
    filter_type     VARCHAR(20),                     -- notes, cc, program, all

    -- Transform
    transpose       SMALLINT DEFAULT 0,
    velocity_scale  SMALLINT DEFAULT 100
);

-- =============================================================================
-- SECTION 3: MIDI MIXER STATE
-- =============================================================================

-- Mixer channel strips (linked to tracks, stored separately for reuse)
CREATE TABLE IF NOT EXISTS mixer_channels (
    id              BIGSERIAL PRIMARY KEY,
    track_id        BIGINT UNIQUE REFERENCES daw_tracks(id) ON DELETE CASCADE,

    -- MIDI Output routing
    midi_channel    SMALLINT NOT NULL DEFAULT 1 CHECK (midi_channel >= 1 AND midi_channel <= 16),
    midi_port_id    BIGINT REFERENCES midi_ports(id),

    -- ══════════════════════════════════════════════════════════════════
    -- MIDI PARAMETERS (all the knobs from the mixer window)
    -- ══════════════════════════════════════════════════════════════════

    -- Transpose
    transpose       SMALLINT DEFAULT 0 CHECK (transpose >= -48 AND transpose <= 48),

    -- Velocity
    velocity_scale  SMALLINT DEFAULT 100 CHECK (velocity_scale >= 1 AND velocity_scale <= 200),
    velocity_min    SMALLINT DEFAULT 0 CHECK (velocity_min >= 0 AND velocity_min <= 127),
    velocity_max    SMALLINT DEFAULT 127 CHECK (velocity_max >= 0 AND velocity_max <= 127),
    velocity_offset SMALLINT DEFAULT 0 CHECK (velocity_offset >= -127 AND velocity_offset <= 127),

    -- Pitch Bend
    pitch_bend      SMALLINT DEFAULT 0 CHECK (pitch_bend >= -8192 AND pitch_bend <= 8191),

    -- Note Filter
    note_low        SMALLINT DEFAULT 0 CHECK (note_low >= 0 AND note_low <= 127),
    note_high       SMALLINT DEFAULT 127 CHECK (note_high >= 0 AND note_high <= 127),

    -- Timing
    quantize_grid   VARCHAR(10),                     -- NULL, '1/4', '1/8', '1/16', '1/32'
    quantize_strength SMALLINT DEFAULT 100 CHECK (quantize_strength >= 0 AND quantize_strength <= 100),
    swing           SMALLINT DEFAULT 0 CHECK (swing >= 0 AND swing <= 100),
    delay_ms        SMALLINT DEFAULT 0 CHECK (delay_ms >= -100 AND delay_ms <= 100),
    humanize        SMALLINT DEFAULT 0 CHECK (humanize >= 0 AND humanize <= 100),

    -- Channel state
    muted           BOOLEAN DEFAULT FALSE,
    solo            BOOLEAN DEFAULT FALSE,

    -- Program change (for selecting patches on outboard gear)
    program_change  SMALLINT CHECK (program_change IS NULL OR (program_change >= 0 AND program_change <= 127)),
    bank_msb        SMALLINT CHECK (bank_msb IS NULL OR (bank_msb >= 0 AND bank_msb <= 127)),
    bank_lsb        SMALLINT CHECK (bank_lsb IS NULL OR (bank_lsb >= 0 AND bank_lsb <= 127)),

    modified_at     TIMESTAMPTZ DEFAULT NOW()
);

-- Master channel (applies to all tracks)
CREATE TABLE IF NOT EXISTS mixer_master (
    id              BIGSERIAL PRIMARY KEY,
    project_id      BIGINT UNIQUE REFERENCES daw_projects(id) ON DELETE CASCADE,

    -- Master MIDI transforms (applied after individual channels)
    transpose       SMALLINT DEFAULT 0 CHECK (transpose >= -48 AND transpose <= 48),
    velocity_scale  SMALLINT DEFAULT 100 CHECK (velocity_scale >= 1 AND velocity_scale <= 200),
    velocity_min    SMALLINT DEFAULT 0 CHECK (velocity_min >= 0 AND velocity_min <= 127),
    velocity_max    SMALLINT DEFAULT 127 CHECK (velocity_max >= 0 AND velocity_max <= 127),
    pitch_bend      SMALLINT DEFAULT 0 CHECK (pitch_bend >= -8192 AND pitch_bend <= 8191),
    note_low        SMALLINT DEFAULT 0 CHECK (note_low >= 0 AND note_low <= 127),
    note_high       SMALLINT DEFAULT 127 CHECK (note_high >= 0 AND note_high <= 127),
    quantize_grid   VARCHAR(10),
    quantize_strength SMALLINT DEFAULT 100 CHECK (quantize_strength >= 0 AND quantize_strength <= 100),
    swing           SMALLINT DEFAULT 0 CHECK (swing >= 0 AND swing <= 100),
    delay_ms        SMALLINT DEFAULT 0 CHECK (delay_ms >= -100 AND delay_ms <= 100),

    -- Master mute
    muted           BOOLEAN DEFAULT FALSE,

    -- Tempo
    tempo           NUMERIC(6, 2) DEFAULT 120.0 CHECK (tempo >= 20 AND tempo <= 400),
    tempo_multiplier NUMERIC(3, 2) DEFAULT 1.0 CHECK (tempo_multiplier >= 0.25 AND tempo_multiplier <= 4.0),

    -- Sync settings
    sync_source     VARCHAR(20) DEFAULT 'internal',  -- internal, external, midi_clock
    send_midi_clock BOOLEAN DEFAULT TRUE,
    send_transport  BOOLEAN DEFAULT TRUE,

    modified_at     TIMESTAMPTZ DEFAULT NOW()
);

-- CC assignments for each channel (which CCs are being sent)
CREATE TABLE IF NOT EXISTS mixer_channel_ccs (
    id              BIGSERIAL PRIMARY KEY,
    mixer_channel_id BIGINT REFERENCES mixer_channels(id) ON DELETE CASCADE,

    cc_number       SMALLINT NOT NULL CHECK (cc_number >= 0 AND cc_number <= 127),
    cc_value        SMALLINT NOT NULL DEFAULT 0 CHECK (cc_value >= 0 AND cc_value <= 127),
    cc_name         VARCHAR(50),                     -- User label: "Filter Cutoff"

    UNIQUE(mixer_channel_id, cc_number)
);

-- =============================================================================
-- SECTION 4: USER PREFERENCES
-- =============================================================================

-- Single-row settings table (key-value with types)
CREATE TABLE IF NOT EXISTS settings (
    key             VARCHAR(100) PRIMARY KEY,
    value           TEXT,
    value_type      VARCHAR(20) DEFAULT 'string',    -- string, int, float, bool, json
    category        VARCHAR(50),                     -- general, audio, midi, ui, keyboard
    modified_at     TIMESTAMPTZ DEFAULT NOW()
);

-- Window layouts (Pro Tools style)
CREATE TABLE IF NOT EXISTS window_layouts (
    id              BIGSERIAL PRIMARY KEY,
    name            VARCHAR(100) NOT NULL,
    is_default      BOOLEAN DEFAULT FALSE,

    -- Store complete window state as JSON
    layout_data     JSONB NOT NULL,

    created_at      TIMESTAMPTZ DEFAULT NOW(),
    modified_at     TIMESTAMPTZ DEFAULT NOW()
);

-- Keyboard shortcuts (customizable)
CREATE TABLE IF NOT EXISTS keyboard_shortcuts (
    id              BIGSERIAL PRIMARY KEY,
    action          VARCHAR(100) NOT NULL UNIQUE,    -- 'transport.play', 'edit.undo', etc.
    shortcut        VARCHAR(50) NOT NULL,            -- 'Space', 'Cmd+Z', 'Ctrl+Shift+S'
    category        VARCHAR(50),
    description     TEXT,
    is_custom       BOOLEAN DEFAULT FALSE            -- TRUE if user modified
);

-- Recent projects
CREATE TABLE IF NOT EXISTS recent_projects (
    id              BIGSERIAL PRIMARY KEY,
    project_id      BIGINT REFERENCES daw_projects(id) ON DELETE CASCADE,
    file_path       TEXT,                            -- For projects saved to disk
    opened_at       TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- SECTION 5: OUTBOARD GEAR PROFILES
-- =============================================================================

-- Hardware device definitions
CREATE TABLE IF NOT EXISTS gear_profiles (
    id              BIGSERIAL PRIMARY KEY,

    -- Basic info
    name            VARCHAR(100) NOT NULL,           -- "Roland TR-808"
    manufacturer    VARCHAR(100),                    -- "Roland"
    category        VARCHAR(50),                     -- synth, drum_machine, sampler, effect

    -- Image/icon
    image_path      TEXT,
    icon            VARCHAR(50),

    -- MIDI specs
    midi_channels   SMALLINT[] DEFAULT '{1}',        -- Channels it responds to
    is_multitimbral BOOLEAN DEFAULT FALSE,

    -- User data
    notes           TEXT,
    is_favorite     BOOLEAN DEFAULT FALSE,
    is_custom       BOOLEAN DEFAULT TRUE,            -- FALSE for factory presets

    created_at      TIMESTAMPTZ DEFAULT NOW()
);

-- CC mappings for each gear profile
CREATE TABLE IF NOT EXISTS gear_cc_mappings (
    id              BIGSERIAL PRIMARY KEY,
    gear_profile_id BIGINT REFERENCES gear_profiles(id) ON DELETE CASCADE,

    cc_number       SMALLINT NOT NULL CHECK (cc_number >= 0 AND cc_number <= 127),
    name            VARCHAR(100) NOT NULL,           -- "Filter Cutoff"
    min_value       SMALLINT DEFAULT 0 CHECK (min_value >= 0 AND min_value <= 127),
    max_value       SMALLINT DEFAULT 127 CHECK (max_value >= 0 AND max_value <= 127),
    default_value   SMALLINT DEFAULT 64 CHECK (default_value >= 0 AND default_value <= 127),

    -- For display
    display_type    VARCHAR(20) DEFAULT 'knob',      -- knob, slider, switch, dropdown

    UNIQUE(gear_profile_id, cc_number)
);

-- Program/Patch names for gear
CREATE TABLE IF NOT EXISTS gear_programs (
    id              BIGSERIAL PRIMARY KEY,
    gear_profile_id BIGINT REFERENCES gear_profiles(id) ON DELETE CASCADE,

    program_number  SMALLINT NOT NULL CHECK (program_number >= 0 AND program_number <= 127),
    bank_msb        SMALLINT DEFAULT 0 CHECK (bank_msb >= 0 AND bank_msb <= 127),
    bank_lsb        SMALLINT DEFAULT 0 CHECK (bank_lsb >= 0 AND bank_lsb <= 127),
    name            VARCHAR(100) NOT NULL,           -- "Acid Bass"
    category        VARCHAR(50),

    UNIQUE(gear_profile_id, bank_msb, bank_lsb, program_number)
);

-- User's gear instances (their actual hardware)
CREATE TABLE IF NOT EXISTS user_gear (
    id              BIGSERIAL PRIMARY KEY,
    gear_profile_id BIGINT REFERENCES gear_profiles(id),

    -- User customization
    nickname        VARCHAR(100),                    -- "My 808 Clone"

    -- MIDI routing
    midi_port_id    BIGINT REFERENCES midi_ports(id),
    midi_channel    SMALLINT DEFAULT 1 CHECK (midi_channel >= 1 AND midi_channel <= 16),

    -- Notes
    notes           TEXT,

    created_at      TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- SECTION 6: PRESETS & TEMPLATES
-- =============================================================================

-- Mixer channel presets (save/recall channel strip settings)
CREATE TABLE IF NOT EXISTS mixer_presets (
    id              BIGSERIAL PRIMARY KEY,
    name            VARCHAR(100) NOT NULL,
    category        VARCHAR(50),                     -- bass, drums, synth, etc.

    -- All the mixer settings as JSON
    settings        JSONB NOT NULL,

    -- For gear association
    gear_profile_id BIGINT REFERENCES gear_profiles(id),

    is_factory      BOOLEAN DEFAULT FALSE,
    created_at      TIMESTAMPTZ DEFAULT NOW()
);

-- Track templates (track + mixer settings)
CREATE TABLE IF NOT EXISTS track_templates (
    id              BIGSERIAL PRIMARY KEY,
    name            VARCHAR(100) NOT NULL,
    category        VARCHAR(50),

    -- Track settings
    track_settings  JSONB NOT NULL,                  -- name, color, icon
    mixer_settings  JSONB NOT NULL,                  -- All mixer params

    -- Associated gear
    gear_profile_id BIGINT REFERENCES gear_profiles(id),

    is_factory      BOOLEAN DEFAULT FALSE,
    created_at      TIMESTAMPTZ DEFAULT NOW()
);

-- Project templates (complete project setup)
CREATE TABLE IF NOT EXISTS project_templates (
    id              BIGSERIAL PRIMARY KEY,
    name            VARCHAR(100) NOT NULL,
    description     TEXT,
    category        VARCHAR(50),                     -- genre or purpose

    -- Complete project as JSON (tracks, mixer, markers)
    template_data   JSONB NOT NULL,

    -- Preview
    thumbnail_path  TEXT,

    is_factory      BOOLEAN DEFAULT FALSE,
    created_at      TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- SECTION 7: HISTORY & UNDO
-- =============================================================================

-- Undo stack per project (in-memory during session, persisted for crash recovery)
CREATE TABLE IF NOT EXISTS undo_stack (
    id              BIGSERIAL PRIMARY KEY,
    project_id      BIGINT REFERENCES daw_projects(id) ON DELETE CASCADE,

    -- Action info
    action_type     VARCHAR(50) NOT NULL,            -- 'clip.move', 'track.add', 'mixer.change'
    action_name     VARCHAR(100) NOT NULL,           -- Human readable: "Move clip"

    -- State before action (for undo)
    before_state    JSONB NOT NULL,

    -- State after action (for redo)
    after_state     JSONB NOT NULL,

    -- Stack position
    stack_index     INTEGER NOT NULL,

    created_at      TIMESTAMPTZ DEFAULT NOW()
);

-- Activity log (non-undoable events, for analytics/debugging)
CREATE TABLE IF NOT EXISTS activity_log (
    id              BIGSERIAL PRIMARY KEY,

    event_type      VARCHAR(50) NOT NULL,            -- 'file.import', 'project.open', 'export'
    event_data      JSONB,

    -- Context
    project_id      BIGINT REFERENCES daw_projects(id) ON DELETE SET NULL,
    midi_file_id    BIGINT REFERENCES files(id) ON DELETE SET NULL,

    created_at      TIMESTAMPTZ DEFAULT NOW()
);

-- Import history (for debugging import issues)
CREATE TABLE IF NOT EXISTS import_history (
    id              BIGSERIAL PRIMARY KEY,

    source_path     TEXT NOT NULL,
    file_count      INTEGER,
    success_count   INTEGER,
    error_count     INTEGER,

    -- Errors as JSON array
    errors          JSONB,

    started_at      TIMESTAMPTZ DEFAULT NOW(),
    completed_at    TIMESTAMPTZ
);

-- =============================================================================
-- SECTION 8: INDEXES
-- =============================================================================

-- Project markers
CREATE INDEX IF NOT EXISTS idx_project_markers_project ON project_markers(project_id, tick);

-- Project clips
CREATE INDEX IF NOT EXISTS idx_project_clips_track ON project_clips(track_id);
CREATE INDEX IF NOT EXISTS idx_project_clips_position ON project_clips(track_id, start_tick);
CREATE INDEX IF NOT EXISTS idx_project_clips_file ON project_clips(midi_file_id) WHERE midi_file_id IS NOT NULL;

-- MIDI I/O
CREATE INDEX IF NOT EXISTS idx_midi_ports_direction ON midi_ports(direction);
CREATE INDEX IF NOT EXISTS idx_midi_ports_connected ON midi_ports(is_connected) WHERE is_connected = TRUE;
CREATE INDEX IF NOT EXISTS idx_midi_routes_input ON midi_routes(input_port_id);
CREATE INDEX IF NOT EXISTS idx_midi_routes_output ON midi_routes(output_port_id);

-- Mixer
CREATE INDEX IF NOT EXISTS idx_mixer_channels_track ON mixer_channels(track_id);
CREATE INDEX IF NOT EXISTS idx_mixer_channels_port ON mixer_channels(midi_port_id) WHERE midi_port_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_mixer_channel_ccs_channel ON mixer_channel_ccs(mixer_channel_id);

-- Settings
CREATE INDEX IF NOT EXISTS idx_settings_category ON settings(category);
CREATE INDEX IF NOT EXISTS idx_keyboard_shortcuts_category ON keyboard_shortcuts(category);
CREATE INDEX IF NOT EXISTS idx_recent_projects_opened ON recent_projects(opened_at DESC);

-- Gear
CREATE INDEX IF NOT EXISTS idx_gear_profiles_category ON gear_profiles(category);
CREATE INDEX IF NOT EXISTS idx_gear_profiles_manufacturer ON gear_profiles(manufacturer);
CREATE INDEX IF NOT EXISTS idx_gear_cc_mappings_profile ON gear_cc_mappings(gear_profile_id);
CREATE INDEX IF NOT EXISTS idx_gear_programs_profile ON gear_programs(gear_profile_id);
CREATE INDEX IF NOT EXISTS idx_user_gear_profile ON user_gear(gear_profile_id);

-- Presets
CREATE INDEX IF NOT EXISTS idx_mixer_presets_category ON mixer_presets(category);
CREATE INDEX IF NOT EXISTS idx_mixer_presets_gear ON mixer_presets(gear_profile_id) WHERE gear_profile_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_track_templates_category ON track_templates(category);
CREATE INDEX IF NOT EXISTS idx_project_templates_category ON project_templates(category);

-- History
CREATE INDEX IF NOT EXISTS idx_undo_stack_project ON undo_stack(project_id, stack_index);
CREATE INDEX IF NOT EXISTS idx_activity_log_type ON activity_log(event_type);
CREATE INDEX IF NOT EXISTS idx_activity_log_time ON activity_log(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_activity_log_project ON activity_log(project_id) WHERE project_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_import_history_time ON import_history(started_at DESC);

-- =============================================================================
-- SECTION 9: DEFAULT DATA
-- =============================================================================

-- Default settings
INSERT INTO settings (key, value, value_type, category) VALUES
    -- General
    ('app.language', 'en', 'string', 'general'),
    ('app.theme', 'dark', 'string', 'general'),
    ('app.auto_save', 'true', 'bool', 'general'),
    ('app.auto_save_interval', '60', 'int', 'general'),
    ('app.recent_projects_count', '10', 'int', 'general'),

    -- MIDI
    ('midi.default_channel', '1', 'int', 'midi'),
    ('midi.default_velocity', '100', 'int', 'midi'),
    ('midi.clock_source', 'internal', 'string', 'midi'),
    ('midi.send_clock', 'true', 'bool', 'midi'),
    ('midi.send_transport', 'true', 'bool', 'midi'),
    ('midi.thru_enabled', 'true', 'bool', 'midi'),
    ('midi.panic_on_disconnect', 'true', 'bool', 'midi'),
    ('midi.auto_reconnect', 'true', 'bool', 'midi'),

    -- UI
    ('ui.browser_position', 'left', 'string', 'ui'),
    ('ui.preview_visible', 'true', 'bool', 'ui'),
    ('ui.mixer_visible', 'true', 'bool', 'ui'),
    ('ui.sequencer_zoom', '1.0', 'float', 'ui'),
    ('ui.track_height_default', '80', 'int', 'ui'),

    -- Browser
    ('browser.view_mode', 'list', 'string', 'browser'),
    ('browser.sort_by', 'name', 'string', 'browser'),
    ('browser.sort_order', 'asc', 'string', 'browser'),
    ('browser.page_size', '25', 'int', 'browser'),
    ('browser.show_preview', 'true', 'bool', 'browser'),

    -- Import
    ('import.auto_tag', 'true', 'bool', 'import'),
    ('import.analyze_bpm', 'true', 'bool', 'import'),
    ('import.analyze_key', 'true', 'bool', 'import'),
    ('import.duplicate_action', 'skip', 'string', 'import')
ON CONFLICT (key) DO NOTHING;

-- Default window layouts
INSERT INTO window_layouts (name, is_default, layout_data) VALUES
    ('Browse & Edit', TRUE, '{"windows": [
        {"id": "database", "x": 0, "y": 0, "width": 400, "height": 600, "state": "normal", "docked": "left"},
        {"id": "preview", "x": 0, "y": 600, "width": 400, "height": 300, "state": "normal", "docked": "left"},
        {"id": "sequencer", "x": 400, "y": 0, "width": 800, "height": 900, "state": "normal", "docked": "float"}
    ]}'),
    ('Performance', FALSE, '{"windows": [
        {"id": "sequencer", "x": 0, "y": 0, "width": 1000, "height": 500, "state": "normal"},
        {"id": "mixer", "x": 0, "y": 500, "width": 1000, "height": 400, "state": "normal"}
    ]}'),
    ('Full Edit', FALSE, '{"windows": [
        {"id": "database", "x": 0, "y": 0, "width": 300, "height": 450, "state": "normal"},
        {"id": "sequencer", "x": 300, "y": 0, "width": 700, "height": 450, "state": "normal"},
        {"id": "mixer", "x": 0, "y": 450, "width": 500, "height": 350, "state": "normal"},
        {"id": "piano_roll", "x": 500, "y": 450, "width": 500, "height": 350, "state": "normal"}
    ]}')
ON CONFLICT DO NOTHING;

-- Default keyboard shortcuts
INSERT INTO keyboard_shortcuts (action, shortcut, category, description) VALUES
    -- Transport
    ('transport.play', 'Space', 'transport', 'Play / Pause'),
    ('transport.stop', 'Enter', 'transport', 'Stop'),
    ('transport.record', 'R', 'transport', 'Toggle Record'),
    ('transport.loop', 'L', 'transport', 'Toggle Loop'),
    ('transport.rewind', ',', 'transport', 'Rewind'),
    ('transport.forward', '.', 'transport', 'Forward'),
    ('transport.goto_start', 'Home', 'transport', 'Go to Start'),
    ('transport.goto_end', 'End', 'transport', 'Go to End'),

    -- Edit
    ('edit.undo', 'Cmd+Z', 'edit', 'Undo'),
    ('edit.redo', 'Cmd+Shift+Z', 'edit', 'Redo'),
    ('edit.cut', 'Cmd+X', 'edit', 'Cut'),
    ('edit.copy', 'Cmd+C', 'edit', 'Copy'),
    ('edit.paste', 'Cmd+V', 'edit', 'Paste'),
    ('edit.delete', 'Backspace', 'edit', 'Delete'),
    ('edit.select_all', 'Cmd+A', 'edit', 'Select All'),
    ('edit.duplicate', 'Cmd+D', 'edit', 'Duplicate'),

    -- Track
    ('track.mute', 'M', 'track', 'Mute Selected Track'),
    ('track.solo', 'S', 'track', 'Solo Selected Track'),
    ('track.arm', 'Shift+R', 'track', 'Arm Selected Track'),
    ('track.new', 'Cmd+T', 'track', 'New Track'),

    -- Windows
    ('window.database', 'Cmd+1', 'window', 'Show/Hide Database'),
    ('window.sequencer', 'Cmd+2', 'window', 'Show/Hide Sequencer'),
    ('window.mixer', 'Cmd+3', 'window', 'Show/Hide Mixer'),
    ('window.piano_roll', 'Cmd+4', 'window', 'Show/Hide Piano Roll'),
    ('window.monitor', 'Cmd+5', 'window', 'Show/Hide MIDI Monitor'),
    ('window.io_setup', 'Cmd+6', 'window', 'Show/Hide I/O Setup'),
    ('window.command_palette', 'Cmd+K', 'window', 'Command Palette'),

    -- File
    ('file.new', 'Cmd+N', 'file', 'New Project'),
    ('file.open', 'Cmd+O', 'file', 'Open Project'),
    ('file.save', 'Cmd+S', 'file', 'Save Project'),
    ('file.save_as', 'Cmd+Shift+S', 'file', 'Save Project As'),
    ('file.export', 'Cmd+E', 'file', 'Export MIDI'),

    -- Browser
    ('browser.search', 'Cmd+F', 'browser', 'Focus Search'),
    ('browser.preview', 'P', 'browser', 'Preview Selected')
ON CONFLICT (action) DO NOTHING;

-- Factory gear profiles
INSERT INTO gear_profiles (name, manufacturer, category, is_custom) VALUES
    ('TR-808', 'Roland', 'drum_machine', FALSE),
    ('TR-909', 'Roland', 'drum_machine', FALSE),
    ('TB-303', 'Roland', 'synth', FALSE),
    ('Juno-106', 'Roland', 'synth', FALSE),
    ('DX7', 'Yamaha', 'synth', FALSE),
    ('Minimoog', 'Moog', 'synth', FALSE),
    ('Sub 37', 'Moog', 'synth', FALSE),
    ('MicroKorg', 'Korg', 'synth', FALSE),
    ('Volca Keys', 'Korg', 'synth', FALSE),
    ('Volca Beats', 'Korg', 'drum_machine', FALSE),
    ('Digitakt', 'Elektron', 'sampler', FALSE),
    ('Digitone', 'Elektron', 'synth', FALSE),
    ('Prophet-5', 'Sequential', 'synth', FALSE),
    ('OB-6', 'Sequential', 'synth', FALSE),
    ('Bass Station II', 'Novation', 'synth', FALSE),
    ('Circuit Tracks', 'Novation', 'groovebox', FALSE),
    ('MPC One', 'Akai', 'sampler', FALSE),
    ('Hydrasynth', 'ASM', 'synth', FALSE),
    ('Minilogue XD', 'Korg', 'synth', FALSE),
    ('Peak', 'Novation', 'synth', FALSE)
ON CONFLICT DO NOTHING;

-- Factory project templates
INSERT INTO project_templates (name, description, category, template_data, is_factory) VALUES
    ('Empty Project', 'Blank project with no tracks', 'basic', '{"tracks": [], "bpm": 120}', TRUE),
    ('4 Track', '4 tracks for drums, bass, lead, pad', 'basic', '{"tracks": [
        {"name": "Drums", "channel": 10, "color": "#FF5733"},
        {"name": "Bass", "channel": 2, "color": "#3498DB"},
        {"name": "Lead", "channel": 1, "color": "#2ECC71"},
        {"name": "Pad", "channel": 3, "color": "#9B59B6"}
    ], "bpm": 120}', TRUE),
    ('8 Track', '8 tracks for full arrangement', 'basic', '{"tracks": [
        {"name": "Kick", "channel": 10},
        {"name": "Snare", "channel": 10},
        {"name": "Hats", "channel": 10},
        {"name": "Perc", "channel": 10},
        {"name": "Bass", "channel": 2},
        {"name": "Lead", "channel": 1},
        {"name": "Pad", "channel": 3},
        {"name": "FX", "channel": 4}
    ], "bpm": 120}', TRUE),
    ('Hip-Hop', 'Hip-Hop template at 90 BPM', 'genre', '{"tracks": [
        {"name": "Drums", "channel": 10},
        {"name": "808 Bass", "channel": 2},
        {"name": "Melody", "channel": 1},
        {"name": "Chops", "channel": 3}
    ], "bpm": 90}', TRUE),
    ('Techno', 'Techno template at 130 BPM', 'genre', '{"tracks": [
        {"name": "Kick", "channel": 10},
        {"name": "Hats", "channel": 10},
        {"name": "Perc", "channel": 10},
        {"name": "Bass", "channel": 2},
        {"name": "Lead", "channel": 1},
        {"name": "Pad", "channel": 3}
    ], "bpm": 130}', TRUE)
ON CONFLICT DO NOTHING;

-- =============================================================================
-- SECTION 10: TRIGGERS
-- =============================================================================

-- Update modified_at for mixer_channels
CREATE OR REPLACE FUNCTION update_mixer_channel_timestamp() RETURNS trigger AS $$
BEGIN
    NEW.modified_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER mixer_channels_updated_at
    BEFORE UPDATE ON mixer_channels
    FOR EACH ROW
    EXECUTE FUNCTION update_mixer_channel_timestamp();

CREATE TRIGGER mixer_master_updated_at
    BEFORE UPDATE ON mixer_master
    FOR EACH ROW
    EXECUTE FUNCTION update_mixer_channel_timestamp();

-- Update modified_at for project_clips
CREATE TRIGGER project_clips_updated_at
    BEFORE UPDATE ON project_clips
    FOR EACH ROW
    EXECUTE FUNCTION update_mixer_channel_timestamp();

-- Update modified_at for window_layouts
CREATE TRIGGER window_layouts_updated_at
    BEFORE UPDATE ON window_layouts
    FOR EACH ROW
    EXECUTE FUNCTION update_mixer_channel_timestamp();

-- Update modified_at for settings
CREATE TRIGGER settings_updated_at
    BEFORE UPDATE ON settings
    FOR EACH ROW
    EXECUTE FUNCTION update_mixer_channel_timestamp();

-- =============================================================================
-- SECTION 11: COMMENTS
-- =============================================================================

COMMENT ON TABLE project_markers IS 'Song markers, cue points, and loop markers within projects';
COMMENT ON TABLE project_clips IS 'MIDI clips placed on tracks referencing library files';
COMMENT ON TABLE midi_ports IS 'Physical and virtual MIDI ports discovered from the system';
COMMENT ON TABLE midi_port_groups IS 'Groups for multi-port interfaces (e.g., MOTU 828)';
COMMENT ON TABLE midi_routes IS 'MIDI routing rules for input to output mapping';
COMMENT ON TABLE mixer_channels IS 'Per-track MIDI mixer parameters (transpose, velocity, timing)';
COMMENT ON TABLE mixer_master IS 'Master channel MIDI parameters and sync settings';
COMMENT ON TABLE mixer_channel_ccs IS 'CC assignments per mixer channel';
COMMENT ON TABLE settings IS 'Application settings key-value store';
COMMENT ON TABLE window_layouts IS 'Saved window arrangements (Pro Tools style)';
COMMENT ON TABLE keyboard_shortcuts IS 'Customizable keyboard shortcuts';
COMMENT ON TABLE recent_projects IS 'Recent project history';
COMMENT ON TABLE gear_profiles IS 'Hardware device definitions with MIDI specs';
COMMENT ON TABLE gear_cc_mappings IS 'CC parameter mappings for gear profiles';
COMMENT ON TABLE gear_programs IS 'Program/patch names for gear';
COMMENT ON TABLE user_gear IS 'User instances of gear profiles with routing';
COMMENT ON TABLE mixer_presets IS 'Reusable mixer channel presets';
COMMENT ON TABLE track_templates IS 'Track + mixer settings templates';
COMMENT ON TABLE project_templates IS 'Complete project templates';
COMMENT ON TABLE undo_stack IS 'Per-project undo/redo stack';
COMMENT ON TABLE activity_log IS 'Non-undoable activity log for analytics';
COMMENT ON TABLE import_history IS 'Import operation history with error tracking';

-- =============================================================================
-- SECTION 12: RECORD MIGRATION
-- =============================================================================

INSERT INTO schema_migrations (version, description)
VALUES ('018', 'Upgrades V2 Complete - Mixer, MIDI I/O, Preferences, Gear, Presets, History')
ON CONFLICT DO NOTHING;

-- =============================================================================
-- SECTION 13: VERIFY MIGRATION
-- =============================================================================

DO $$
DECLARE
    table_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO table_count
    FROM information_schema.tables
    WHERE table_schema = 'public'
    AND table_name IN (
        'project_markers', 'project_clips',
        'midi_ports', 'midi_port_groups', 'midi_port_group_members', 'midi_routes',
        'mixer_channels', 'mixer_master', 'mixer_channel_ccs',
        'settings', 'window_layouts', 'keyboard_shortcuts', 'recent_projects',
        'gear_profiles', 'gear_cc_mappings', 'gear_programs', 'user_gear',
        'mixer_presets', 'track_templates', 'project_templates',
        'undo_stack', 'activity_log', 'import_history'
    );

    RAISE NOTICE 'Migration 018 verification:';
    RAISE NOTICE '  Tables created: % (expected 22)', table_count;

    IF table_count < 22 THEN
        RAISE EXCEPTION 'Expected 22 tables, only created %', table_count;
    END IF;

    RAISE NOTICE 'Migration 018 completed successfully';
END $$;

COMMIT;

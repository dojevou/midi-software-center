-- Migration 026: Mixer Presets System
-- Stream B Day 4 - Preset save/load functionality
-- Created: 2025-12-17

-- =============================================================================
-- MIXER PRESETS TABLE
-- =============================================================================
-- Stores mixer channel/track presets that can be saved and loaded
-- Supports both channel presets (single track) and full mixer presets (all tracks)

CREATE TABLE IF NOT EXISTS mixer_presets (
    id BIGSERIAL PRIMARY KEY,

    -- Preset identification
    name VARCHAR(255) NOT NULL,
    description TEXT,

    -- Preset type: 'channel' for single track, 'mixer' for full mixer state
    preset_type VARCHAR(50) NOT NULL DEFAULT 'channel',

    -- Category for organization (e.g., 'drums', 'bass', 'vocals', 'master')
    category VARCHAR(100),

    -- Tags for searchability (stored as array)
    tags TEXT[],

    -- The actual preset data stored as JSONB
    -- For channel presets: single MixerChannel serialized
    -- For mixer presets: full MixerState with all channels
    preset_data JSONB NOT NULL,

    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Usage tracking
    use_count INTEGER NOT NULL DEFAULT 0,
    last_used_at TIMESTAMPTZ,

    -- Favorite flag for quick access
    is_favorite BOOLEAN NOT NULL DEFAULT FALSE,

    -- Factory presets vs user presets
    is_factory BOOLEAN NOT NULL DEFAULT FALSE
);

-- =============================================================================
-- INDEXES
-- =============================================================================

-- Primary lookup patterns
CREATE INDEX idx_mixer_presets_name ON mixer_presets(name);
CREATE INDEX idx_mixer_presets_type ON mixer_presets(preset_type);
CREATE INDEX idx_mixer_presets_category ON mixer_presets(category);
CREATE INDEX idx_mixer_presets_favorite ON mixer_presets(is_favorite) WHERE is_favorite = TRUE;
CREATE INDEX idx_mixer_presets_factory ON mixer_presets(is_factory);

-- Tag search using GIN
CREATE INDEX idx_mixer_presets_tags ON mixer_presets USING GIN(tags);

-- JSONB search for preset data
CREATE INDEX idx_mixer_presets_data ON mixer_presets USING GIN(preset_data);

-- Sort by usage
CREATE INDEX idx_mixer_presets_use_count ON mixer_presets(use_count DESC);
CREATE INDEX idx_mixer_presets_last_used ON mixer_presets(last_used_at DESC NULLS LAST);

-- =============================================================================
-- TRIGGER FOR updated_at
-- =============================================================================

CREATE OR REPLACE FUNCTION update_mixer_presets_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_mixer_presets_updated_at
    BEFORE UPDATE ON mixer_presets
    FOR EACH ROW
    EXECUTE FUNCTION update_mixer_presets_updated_at();

-- =============================================================================
-- HELPER FUNCTIONS
-- =============================================================================

-- Get presets by type with optional category filter
CREATE OR REPLACE FUNCTION get_mixer_presets(
    p_type VARCHAR DEFAULT NULL,
    p_category VARCHAR DEFAULT NULL,
    p_favorites_only BOOLEAN DEFAULT FALSE,
    p_limit INTEGER DEFAULT 50,
    p_offset INTEGER DEFAULT 0
)
RETURNS TABLE (
    id BIGINT,
    name VARCHAR,
    description TEXT,
    preset_type VARCHAR,
    category VARCHAR,
    tags TEXT[],
    use_count INTEGER,
    last_used_at TIMESTAMPTZ,
    is_favorite BOOLEAN,
    is_factory BOOLEAN,
    created_at TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        mp.id,
        mp.name,
        mp.description,
        mp.preset_type,
        mp.category,
        mp.tags,
        mp.use_count,
        mp.last_used_at,
        mp.is_favorite,
        mp.is_factory,
        mp.created_at
    FROM mixer_presets mp
    WHERE
        (p_type IS NULL OR mp.preset_type = p_type)
        AND (p_category IS NULL OR mp.category = p_category)
        AND (NOT p_favorites_only OR mp.is_favorite = TRUE)
    ORDER BY
        mp.is_favorite DESC,
        mp.use_count DESC,
        mp.name ASC
    LIMIT p_limit
    OFFSET p_offset;
END;
$$ LANGUAGE plpgsql;

-- Increment use count and update last_used_at
CREATE OR REPLACE FUNCTION use_mixer_preset(p_id BIGINT)
RETURNS VOID AS $$
BEGIN
    UPDATE mixer_presets
    SET
        use_count = use_count + 1,
        last_used_at = NOW()
    WHERE id = p_id;
END;
$$ LANGUAGE plpgsql;

-- =============================================================================
-- FACTORY PRESETS (Built-in defaults)
-- =============================================================================

-- Insert some factory presets for common use cases
INSERT INTO mixer_presets (name, description, preset_type, category, tags, preset_data, is_factory) VALUES

-- Channel presets
('Clean Channel', 'Default clean channel with no effects', 'channel', 'utility',
 ARRAY['clean', 'default', 'utility'],
 '{"track_id": 0, "name": "Clean", "enabled": true, "midi_channel": 1, "transpose": 0, "octave_shift": 0, "velocity": 100, "velocity_scale": 100, "velocity_min": 1, "velocity_max": 127, "volume": 100, "pan": 64, "muted": false, "solo": false, "effects": [], "sends": [], "monitoring_mode": "Off", "record_armed": false}'::jsonb,
 TRUE),

('Drums Channel', 'Optimized for drum tracks with compression', 'channel', 'drums',
 ARRAY['drums', 'percussion', 'rhythm'],
 '{"track_id": 0, "name": "Drums", "enabled": true, "midi_channel": 10, "transpose": 0, "octave_shift": 0, "velocity": 110, "velocity_scale": 90, "velocity_min": 20, "velocity_max": 127, "volume": 105, "pan": 64, "muted": false, "solo": false, "effects": [{"id": 1, "name": "Compressor", "enabled": true, "wet_dry": 1.0, "parameters": {"threshold": -15.0, "ratio": 4.0, "attack": 5.0, "release": 100.0, "makeup": 3.0}}], "sends": [], "monitoring_mode": "Off", "record_armed": false}'::jsonb,
 TRUE),

('Bass Channel', 'Warm bass with subtle compression', 'channel', 'bass',
 ARRAY['bass', 'low-end', 'rhythm'],
 '{"track_id": 0, "name": "Bass", "enabled": true, "midi_channel": 2, "transpose": 0, "octave_shift": 0, "velocity": 100, "velocity_scale": 100, "velocity_min": 30, "velocity_max": 120, "volume": 100, "pan": 64, "muted": false, "solo": false, "effects": [{"id": 1, "name": "Compressor", "enabled": true, "wet_dry": 1.0, "parameters": {"threshold": -18.0, "ratio": 3.0, "attack": 10.0, "release": 150.0, "makeup": 2.0}}, {"id": 2, "name": "EQ3Band", "enabled": true, "wet_dry": 1.0, "parameters": {"low_gain": 2.0, "mid_gain": -1.0, "high_gain": -3.0, "low_freq": 80.0, "mid_freq": 800.0, "high_freq": 4000.0}}], "sends": [], "monitoring_mode": "Off", "record_armed": false}'::jsonb,
 TRUE),

('Piano Channel', 'Classic piano with reverb', 'channel', 'keys',
 ARRAY['piano', 'keys', 'melodic'],
 '{"track_id": 0, "name": "Piano", "enabled": true, "midi_channel": 1, "transpose": 0, "octave_shift": 0, "velocity": 100, "velocity_scale": 100, "velocity_min": 1, "velocity_max": 127, "volume": 95, "pan": 64, "muted": false, "solo": false, "effects": [{"id": 1, "name": "EQ3Band", "enabled": true, "wet_dry": 1.0, "parameters": {"low_gain": -2.0, "mid_gain": 1.0, "high_gain": 2.0, "low_freq": 150.0, "mid_freq": 1000.0, "high_freq": 6000.0}}, {"id": 2, "name": "Reverb", "enabled": true, "wet_dry": 0.25, "parameters": {"room_size": 0.6, "damping": 0.4, "width": 0.8, "wet": 0.3, "dry": 0.7}}], "sends": [], "monitoring_mode": "Off", "record_armed": false}'::jsonb,
 TRUE),

('Synth Lead', 'Bright synth lead with delay', 'channel', 'synth',
 ARRAY['synth', 'lead', 'melodic'],
 '{"track_id": 0, "name": "Synth Lead", "enabled": true, "midi_channel": 3, "transpose": 0, "octave_shift": 0, "velocity": 100, "velocity_scale": 100, "velocity_min": 1, "velocity_max": 127, "volume": 90, "pan": 64, "muted": false, "solo": false, "effects": [{"id": 1, "name": "EQ3Band", "enabled": true, "wet_dry": 1.0, "parameters": {"low_gain": -4.0, "mid_gain": 2.0, "high_gain": 3.0, "low_freq": 200.0, "mid_freq": 2000.0, "high_freq": 8000.0}}, {"id": 2, "name": "Delay", "enabled": true, "wet_dry": 0.3, "parameters": {"time_l": 375.0, "time_r": 500.0, "feedback": 0.4, "wet": 0.3, "dry": 0.7, "ping_pong": true}}], "sends": [], "monitoring_mode": "Off", "record_armed": false}'::jsonb,
 TRUE),

('Pad Channel', 'Atmospheric pad with lush reverb', 'channel', 'synth',
 ARRAY['pad', 'synth', 'atmospheric', 'ambient'],
 '{"track_id": 0, "name": "Pad", "enabled": true, "midi_channel": 4, "transpose": 0, "octave_shift": 0, "velocity": 80, "velocity_scale": 80, "velocity_min": 20, "velocity_max": 100, "volume": 85, "pan": 64, "muted": false, "solo": false, "effects": [{"id": 1, "name": "Reverb", "enabled": true, "wet_dry": 0.5, "parameters": {"room_size": 0.85, "damping": 0.3, "width": 1.0, "wet": 0.5, "dry": 0.5}}, {"id": 2, "name": "Delay", "enabled": true, "wet_dry": 0.2, "parameters": {"time_l": 500.0, "time_r": 750.0, "feedback": 0.3, "wet": 0.2, "dry": 0.8, "ping_pong": false}}], "sends": [], "monitoring_mode": "Off", "record_armed": false}'::jsonb,
 TRUE);

-- =============================================================================
-- COMMENTS
-- =============================================================================

COMMENT ON TABLE mixer_presets IS 'Stores mixer channel and full mixer state presets';
COMMENT ON COLUMN mixer_presets.preset_type IS 'Type of preset: channel (single track) or mixer (full state)';
COMMENT ON COLUMN mixer_presets.preset_data IS 'JSONB containing serialized MixerChannel or MixerState';
COMMENT ON COLUMN mixer_presets.is_factory IS 'TRUE for built-in presets, FALSE for user-created';

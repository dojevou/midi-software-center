-- =============================================================================
-- MIDI Library System - Complete Database Schema
-- =============================================================================
-- Migration: 001_initial_schema.sql
-- Version: 1.0
-- PostgreSQL: 16+
-- Target Scale: 3,000,000+ MIDI files
-- Description: Complete database structure with all tables, indexes, and triggers
-- =============================================================================

BEGIN;

-- =============================================================================
-- EXTENSIONS
-- =============================================================================

CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- =============================================================================
-- ENUM TYPES
-- =============================================================================

-- File categories for organization
CREATE TYPE file_category AS ENUM (
    -- Drums
    'KICK', 'SNARE', 'HIHAT', 'CLAP', 'PERC', 'TOM', 'CYMBAL',
    'DRUM_LOOP', 'DRUM_PATTERN',

    -- Bass
    'BASS', 'SUB_BASS', 'BASS_LOOP',

    -- Chords
    'CHORD', 'PROGRESSION', 'STAB',

    -- Pads
    'PAD', 'TEXTURE', 'ATMOSPHERE',

    -- Leads
    'LEAD', 'MELODY', 'HOOK', 'RIFF',

    -- Sequences
    'ARP', 'SEQUENCE',

    -- Keys
    'PIANO', 'KEYS', 'ORGAN',

    -- Orchestral
    'STRING', 'BRASS', 'WOODWIND',

    -- FX
    'FX', 'RISER', 'IMPACT', 'SWEEP', 'TRANSITION',

    -- Vocal
    'VOCAL', 'VOX', 'SAMPLE',

    -- Other
    'MOTIF', 'THEME', 'FULL_MIX', 'STEM', 'UNKNOWN'
);

-- Musical key signatures
CREATE TYPE musical_key AS ENUM (
    'C', 'Cm', 'C#', 'C#m', 'Db', 'Dbm',
    'D', 'Dm', 'D#', 'D#m', 'Eb', 'Ebm',
    'E', 'Em', 'F', 'Fm', 'F#', 'F#m',
    'Gb', 'Gbm', 'G', 'Gm', 'G#', 'G#m',
    'Ab', 'Abm', 'A', 'Am', 'A#', 'A#m',
    'Bb', 'Bbm', 'B', 'Bm', 'UNKNOWN'
);

-- =============================================================================
-- CORE TABLES
-- =============================================================================

-- -----------------------------------------------------------------------------
-- TABLE: files
-- PURPOSE: Primary table for all MIDI files
-- EXPECTED ROWS: 3,000,000+
-- -----------------------------------------------------------------------------
CREATE TABLE files (
    id BIGSERIAL PRIMARY KEY,

    -- File identification
    filename TEXT NOT NULL,
    filepath TEXT NOT NULL UNIQUE,
    original_filename TEXT NOT NULL,
    content_hash BYTEA NOT NULL,
    file_size_bytes BIGINT NOT NULL,

    -- MIDI format
    format SMALLINT CHECK (format IN (0, 1, 2)),
    num_tracks SMALLINT NOT NULL DEFAULT 1,
    ticks_per_quarter_note INTEGER,

    -- Duration
    duration_seconds NUMERIC(10, 3),
    duration_ticks BIGINT,

    -- Multi-track handling
    is_multi_track BOOLEAN DEFAULT FALSE,
    parent_file_id BIGINT REFERENCES files(id) ON DELETE CASCADE,
    track_number SMALLINT,
    total_tracks SMALLINT,

    -- Extracted context (from path/filename)
    manufacturer TEXT,
    collection_name TEXT,
    folder_tags TEXT[],

    -- Full-text search
    search_vector tsvector,

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    analyzed_at TIMESTAMPTZ,

    -- Processing
    import_batch_id UUID,

    -- Constraints
    CONSTRAINT valid_multi_track CHECK (
        (is_multi_track = FALSE AND parent_file_id IS NULL) OR
        (is_multi_track = TRUE AND parent_file_id IS NOT NULL)
    )
);

-- -----------------------------------------------------------------------------
-- TABLE: musical_metadata
-- PURPOSE: Musical properties of each file
-- EXPECTED ROWS: 3,000,000+
-- -----------------------------------------------------------------------------
CREATE TABLE musical_metadata (
    file_id BIGINT PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,

    -- Tempo
    bpm NUMERIC(6, 2) CHECK (bpm IS NULL OR (bpm >= 20 AND bpm <= 300)),
    bpm_confidence REAL,
    has_tempo_changes BOOLEAN DEFAULT FALSE,
    tempo_changes JSONB,

    -- Key signature
    key_signature musical_key,
    key_confidence REAL,
    has_key_changes BOOLEAN DEFAULT FALSE,
    key_changes JSONB,

    -- Time signature
    time_signature_numerator SMALLINT DEFAULT 4,
    time_signature_denominator SMALLINT DEFAULT 4,
    has_time_signature_changes BOOLEAN DEFAULT FALSE,
    time_signature_changes JSONB,

    -- Note statistics
    total_notes INTEGER NOT NULL DEFAULT 0,
    unique_pitches INTEGER,
    pitch_range_min SMALLINT CHECK (pitch_range_min IS NULL OR (pitch_range_min >= 0 AND pitch_range_min <= 127)),
    pitch_range_max SMALLINT CHECK (pitch_range_max IS NULL OR (pitch_range_max >= 0 AND pitch_range_max <= 127)),
    avg_velocity NUMERIC(5, 2),

    -- Density metrics
    note_density NUMERIC(8, 3),
    polyphony_max SMALLINT,
    polyphony_avg NUMERIC(5, 2),

    -- Musical characteristics
    is_monophonic BOOLEAN DEFAULT FALSE,
    is_polyphonic BOOLEAN DEFAULT TRUE,
    is_percussive BOOLEAN DEFAULT FALSE,

    -- Chord analysis
    has_chords BOOLEAN DEFAULT FALSE,
    chord_complexity REAL,

    -- Melody analysis
    has_melody BOOLEAN DEFAULT FALSE,
    melodic_range SMALLINT,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- -----------------------------------------------------------------------------
-- TABLE: file_categories
-- PURPOSE: Category classification for files
-- EXPECTED ROWS: 3,000,000+
-- -----------------------------------------------------------------------------
CREATE TABLE file_categories (
    file_id BIGINT PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,

    primary_category file_category NOT NULL,
    secondary_category file_category,
    tertiary_category file_category,

    -- Confidence scores
    confidence_score REAL CHECK (confidence_score IS NULL OR (confidence_score >= 0 AND confidence_score <= 1)),

    -- Source tracking
    is_manual BOOLEAN DEFAULT FALSE,
    detected_from TEXT,

    created_at TIMESTAMPTZ DEFAULT NOW(),

    -- Constraint: categories should be different
    CONSTRAINT different_categories CHECK (
        (secondary_category IS NULL OR secondary_category != primary_category) AND
        (tertiary_category IS NULL OR (tertiary_category != primary_category AND tertiary_category != secondary_category))
    )
);

-- -----------------------------------------------------------------------------
-- TABLE: file_instruments
-- PURPOSE: Detected instruments in MIDI files
-- EXPECTED ROWS: 10,000,000+
-- -----------------------------------------------------------------------------
CREATE TABLE file_instruments (
    id BIGSERIAL PRIMARY KEY,
    file_id BIGINT NOT NULL REFERENCES files(id) ON DELETE CASCADE,

    -- MIDI program
    channel SMALLINT NOT NULL CHECK (channel >= 0 AND channel <= 15),
    program_number SMALLINT NOT NULL CHECK (program_number >= 0 AND program_number <= 127),
    program_name TEXT,

    -- Categorization
    instrument_family TEXT,
    instrument_type TEXT,

    -- Usage statistics
    note_count INTEGER DEFAULT 0,
    is_primary BOOLEAN DEFAULT FALSE,
    avg_velocity NUMERIC(5, 2),
    pitch_range_low SMALLINT CHECK (pitch_range_low IS NULL OR (pitch_range_low >= 0 AND pitch_range_low <= 127)),
    pitch_range_high SMALLINT CHECK (pitch_range_high IS NULL OR (pitch_range_high >= 0 AND pitch_range_high <= 127)),

    created_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(file_id, channel, program_number)
);

-- =============================================================================
-- TAGGING SYSTEM
-- =============================================================================

-- -----------------------------------------------------------------------------
-- TABLE: tags
-- PURPOSE: Tag definitions
-- EXPECTED ROWS: 10,000
-- -----------------------------------------------------------------------------
CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    category TEXT,
    usage_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- -----------------------------------------------------------------------------
-- TABLE: file_tags
-- PURPOSE: Many-to-many relationship between files and tags
-- EXPECTED ROWS: 15,000,000+
-- -----------------------------------------------------------------------------
CREATE TABLE file_tags (
    file_id BIGINT NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ DEFAULT NOW(),
    added_by TEXT DEFAULT 'system',

    PRIMARY KEY (file_id, tag_id)
);

-- =============================================================================
-- VECTOR SIMILARITY & EMBEDDINGS
-- =============================================================================

-- -----------------------------------------------------------------------------
-- TABLE: file_embeddings
-- PURPOSE: Vector embeddings for similarity search
-- EXPECTED ROWS: 3,000,000+
-- -----------------------------------------------------------------------------
CREATE TABLE file_embeddings (
    file_id BIGINT PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,

    -- Different embedding types
    overall_embedding vector(768),
    rhythmic_embedding vector(256),
    harmonic_embedding vector(256),
    melodic_embedding vector(256),

    -- Embedding metadata
    model_version TEXT,
    generated_at TIMESTAMPTZ DEFAULT NOW(),

    -- Quality metrics
    embedding_quality REAL CHECK (embedding_quality IS NULL OR (embedding_quality >= 0 AND embedding_quality <= 1))
);

-- -----------------------------------------------------------------------------
-- TABLE: file_compatibility
-- PURPOSE: Pre-computed compatibility scores between files
-- EXPECTED ROWS: 50,000,000+
-- -----------------------------------------------------------------------------
CREATE TABLE file_compatibility (
    file_id_a BIGINT NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    file_id_b BIGINT NOT NULL REFERENCES files(id) ON DELETE CASCADE,

    -- Compatibility scores
    overall_score NUMERIC(3, 2) CHECK (overall_score IS NULL OR (overall_score >= 0 AND overall_score <= 1)),
    rhythmic_score NUMERIC(3, 2),
    harmonic_score NUMERIC(3, 2),
    melodic_score NUMERIC(3, 2),
    timbral_score NUMERIC(3, 2),

    -- Compatibility reasons
    key_compatible BOOLEAN,
    bpm_compatible BOOLEAN,
    time_signature_compatible BOOLEAN,

    -- Metadata
    computed_at TIMESTAMPTZ DEFAULT NOW(),

    PRIMARY KEY (file_id_a, file_id_b),
    CONSTRAINT ordered_pair CHECK (file_id_a < file_id_b)
);

-- =============================================================================
-- DEDUPLICATION
-- =============================================================================

-- -----------------------------------------------------------------------------
-- TABLE: duplicate_groups
-- PURPOSE: Groups of duplicate files by content hash
-- EXPECTED ROWS: 100,000+
-- -----------------------------------------------------------------------------
CREATE TABLE duplicate_groups (
    id SERIAL PRIMARY KEY,
    content_hash BYTEA NOT NULL UNIQUE,
    canonical_file_id BIGINT REFERENCES files(id) ON DELETE SET NULL,
    duplicate_count INTEGER DEFAULT 1,
    total_size_bytes BIGINT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- -----------------------------------------------------------------------------
-- TABLE: duplicate_files
-- PURPOSE: Individual files within duplicate groups
-- EXPECTED ROWS: 500,000+
-- -----------------------------------------------------------------------------
CREATE TABLE duplicate_files (
    id BIGSERIAL PRIMARY KEY,
    group_id INTEGER NOT NULL REFERENCES duplicate_groups(id) ON DELETE CASCADE,
    file_id BIGINT NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    is_canonical BOOLEAN DEFAULT FALSE,
    added_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(file_id)
);

-- =============================================================================
-- MUSICAL ANALYSIS PATTERNS
-- =============================================================================

-- -----------------------------------------------------------------------------
-- TABLE: rhythm_patterns
-- PURPOSE: Rhythmic analysis data
-- EXPECTED ROWS: 3,000,000+
-- -----------------------------------------------------------------------------
CREATE TABLE rhythm_patterns (
    id BIGSERIAL PRIMARY KEY,
    file_id BIGINT NOT NULL REFERENCES files(id) ON DELETE CASCADE,

    -- Pattern identification
    pattern_type TEXT,
    pattern_signature BYTEA,

    -- Timing analysis
    onset_times INTEGER[],
    inter_onset_intervals INTEGER[],
    swing_factor NUMERIC(3, 2),

    -- Groove metrics
    groove_template vector(16),
    syncopation_score NUMERIC(3, 2),

    -- Pattern properties
    pattern_length_beats INTEGER,
    pattern_complexity REAL,

    created_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(file_id, pattern_type)
);

-- -----------------------------------------------------------------------------
-- TABLE: harmonic_patterns
-- PURPOSE: Harmonic/chord progression analysis
-- EXPECTED ROWS: 1,000,000+
-- -----------------------------------------------------------------------------
CREATE TABLE harmonic_patterns (
    id BIGSERIAL PRIMARY KEY,
    file_id BIGINT NOT NULL REFERENCES files(id) ON DELETE CASCADE,

    -- Chord sequence
    chord_sequence TEXT[],
    chord_types TEXT[],
    chord_roots INTEGER[],

    -- Harmonic analysis
    roman_numerals TEXT[],
    harmonic_rhythm INTEGER[],

    -- Pattern properties
    progression_length INTEGER,
    harmonic_complexity REAL,
    uses_seventh_chords BOOLEAN DEFAULT FALSE,
    uses_extended_chords BOOLEAN DEFAULT FALSE,

    -- Fingerprint for similarity
    progression_hash BYTEA,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- -----------------------------------------------------------------------------
-- TABLE: melodic_patterns
-- PURPOSE: Melodic analysis
-- EXPECTED ROWS: 2,000,000+
-- -----------------------------------------------------------------------------
CREATE TABLE melodic_patterns (
    id BIGSERIAL PRIMARY KEY,
    file_id BIGINT NOT NULL REFERENCES files(id) ON DELETE CASCADE,

    -- Melodic contour
    pitch_sequence SMALLINT[],
    interval_sequence SMALLINT[],
    contour_direction TEXT[],

    -- Rhythmic contour
    note_durations INTEGER[],
    rhythmic_motif TEXT,

    -- Pattern analysis
    motif_count INTEGER,
    sequence_count INTEGER,
    repetition_score REAL,

    -- Melodic properties
    melodic_range SMALLINT,
    avg_interval_size NUMERIC(4, 2),
    stepwise_motion_ratio NUMERIC(3, 2),

    -- Fingerprint
    melodic_hash BYTEA,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- PROCESSING & JOBS
-- =============================================================================

-- -----------------------------------------------------------------------------
-- TABLE: processing_jobs
-- PURPOSE: Track batch processing jobs
-- EXPECTED ROWS: 10,000+
-- -----------------------------------------------------------------------------
CREATE TABLE processing_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Job information
    job_type TEXT NOT NULL,
    source_directory TEXT NOT NULL,

    -- Progress tracking
    total_files INTEGER NOT NULL DEFAULT 0,
    processed_files INTEGER NOT NULL DEFAULT 0,
    failed_files INTEGER NOT NULL DEFAULT 0,
    skipped_files INTEGER NOT NULL DEFAULT 0,

    -- Status
    status TEXT DEFAULT 'pending',
    error_message TEXT,

    -- Timing
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    estimated_completion TIMESTAMPTZ,

    -- Settings
    settings JSONB DEFAULT '{}'::jsonb,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- -----------------------------------------------------------------------------
-- TABLE: processing_errors
-- PURPOSE: Track errors during processing
-- EXPECTED ROWS: 50,000+
-- -----------------------------------------------------------------------------
CREATE TABLE processing_errors (
    id BIGSERIAL PRIMARY KEY,
    job_id UUID NOT NULL REFERENCES processing_jobs(id) ON DELETE CASCADE,

    filepath TEXT NOT NULL,
    error_type TEXT,
    error_message TEXT,
    stack_trace TEXT,

    occurred_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- SCHEMA MIGRATIONS
-- =============================================================================

CREATE TABLE schema_migrations (
    id SERIAL PRIMARY KEY,
    version TEXT NOT NULL UNIQUE,
    description TEXT,
    applied_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- INDEXES
-- =============================================================================

-- -----------------------------------------------------------------------------
-- FILES table indexes
-- -----------------------------------------------------------------------------
CREATE UNIQUE INDEX idx_files_content_hash ON files(content_hash);
CREATE INDEX idx_files_filepath ON files(filepath);
CREATE INDEX idx_files_manufacturer ON files(manufacturer) WHERE manufacturer IS NOT NULL;
CREATE INDEX idx_files_collection ON files(collection_name) WHERE collection_name IS NOT NULL;
CREATE INDEX idx_files_parent ON files(parent_file_id) WHERE parent_file_id IS NOT NULL;
CREATE INDEX idx_files_search ON files USING gin(search_vector);
CREATE INDEX idx_files_folder_tags ON files USING gin(folder_tags);
CREATE INDEX idx_files_created ON files(created_at DESC);
CREATE INDEX idx_files_batch ON files(import_batch_id) WHERE import_batch_id IS NOT NULL;
CREATE INDEX idx_files_format ON files(format);
CREATE INDEX idx_files_num_tracks ON files(num_tracks);
CREATE INDEX idx_files_duration ON files(duration_seconds) WHERE duration_seconds IS NOT NULL;

-- -----------------------------------------------------------------------------
-- MUSICAL_METADATA table indexes
-- -----------------------------------------------------------------------------
CREATE INDEX idx_metadata_bpm ON musical_metadata(bpm) WHERE bpm IS NOT NULL;
CREATE INDEX idx_metadata_key ON musical_metadata(key_signature) WHERE key_signature != 'UNKNOWN';
CREATE INDEX idx_metadata_time_sig ON musical_metadata(time_signature_numerator, time_signature_denominator);
CREATE INDEX idx_metadata_notes ON musical_metadata(total_notes DESC);
CREATE INDEX idx_metadata_density ON musical_metadata(note_density DESC);
CREATE INDEX idx_metadata_characteristics ON musical_metadata(is_percussive, is_monophonic, has_chords);
CREATE INDEX idx_metadata_polyphony ON musical_metadata(polyphony_max) WHERE polyphony_max IS NOT NULL;
CREATE INDEX idx_metadata_pitch_range ON musical_metadata(pitch_range_min, pitch_range_max);
CREATE INDEX idx_metadata_has_melody ON musical_metadata(has_melody) WHERE has_melody = TRUE;

-- -----------------------------------------------------------------------------
-- FILE_CATEGORIES table indexes
-- -----------------------------------------------------------------------------
CREATE INDEX idx_categories_primary ON file_categories(primary_category);
CREATE INDEX idx_categories_secondary ON file_categories(secondary_category) WHERE secondary_category IS NOT NULL;
CREATE INDEX idx_categories_tertiary ON file_categories(tertiary_category) WHERE tertiary_category IS NOT NULL;
CREATE INDEX idx_categories_confidence ON file_categories(confidence_score DESC);
CREATE INDEX idx_categories_manual ON file_categories(is_manual) WHERE is_manual = TRUE;

-- -----------------------------------------------------------------------------
-- FILE_INSTRUMENTS table indexes
-- -----------------------------------------------------------------------------
CREATE INDEX idx_instruments_file ON file_instruments(file_id);
CREATE INDEX idx_instruments_program ON file_instruments(program_number);
CREATE INDEX idx_instruments_family ON file_instruments(instrument_family) WHERE instrument_family IS NOT NULL;
CREATE INDEX idx_instruments_primary ON file_instruments(file_id, is_primary) WHERE is_primary = TRUE;
CREATE INDEX idx_instruments_channel ON file_instruments(channel);
CREATE INDEX idx_instruments_type ON file_instruments(instrument_type) WHERE instrument_type IS NOT NULL;

-- -----------------------------------------------------------------------------
-- TAGS table indexes
-- -----------------------------------------------------------------------------
CREATE INDEX idx_tags_name_trgm ON tags USING gin(name gin_trgm_ops);
CREATE INDEX idx_tags_category ON tags(category) WHERE category IS NOT NULL;
CREATE INDEX idx_tags_usage ON tags(usage_count DESC);

-- -----------------------------------------------------------------------------
-- FILE_TAGS table indexes
-- -----------------------------------------------------------------------------
CREATE INDEX idx_file_tags_tag ON file_tags(tag_id);
CREATE INDEX idx_file_tags_file ON file_tags(file_id);
CREATE INDEX idx_file_tags_added_at ON file_tags(added_at DESC);

-- -----------------------------------------------------------------------------
-- FILE_EMBEDDINGS table indexes (IVFFlat for vector similarity search)
-- -----------------------------------------------------------------------------
CREATE INDEX idx_embeddings_overall ON file_embeddings USING ivfflat (overall_embedding vector_cosine_ops) WITH (lists = 100);
CREATE INDEX idx_embeddings_rhythmic ON file_embeddings USING ivfflat (rhythmic_embedding vector_cosine_ops) WITH (lists = 100);
CREATE INDEX idx_embeddings_harmonic ON file_embeddings USING ivfflat (harmonic_embedding vector_cosine_ops) WITH (lists = 100);
CREATE INDEX idx_embeddings_melodic ON file_embeddings USING ivfflat (melodic_embedding vector_cosine_ops) WITH (lists = 100);

-- -----------------------------------------------------------------------------
-- FILE_COMPATIBILITY table indexes
-- -----------------------------------------------------------------------------
CREATE INDEX idx_compat_file_a ON file_compatibility(file_id_a);
CREATE INDEX idx_compat_file_b ON file_compatibility(file_id_b);
CREATE INDEX idx_compat_overall ON file_compatibility(overall_score DESC);
CREATE INDEX idx_compat_key ON file_compatibility(key_compatible) WHERE key_compatible = TRUE;
CREATE INDEX idx_compat_bpm ON file_compatibility(bpm_compatible) WHERE bpm_compatible = TRUE;

-- -----------------------------------------------------------------------------
-- DUPLICATE_GROUPS table indexes
-- -----------------------------------------------------------------------------
CREATE INDEX idx_dup_groups_hash ON duplicate_groups(content_hash);
CREATE INDEX idx_dup_groups_canonical ON duplicate_groups(canonical_file_id) WHERE canonical_file_id IS NOT NULL;
CREATE INDEX idx_dup_groups_count ON duplicate_groups(duplicate_count DESC);

-- -----------------------------------------------------------------------------
-- DUPLICATE_FILES table indexes
-- -----------------------------------------------------------------------------
CREATE INDEX idx_dup_files_group ON duplicate_files(group_id);
CREATE INDEX idx_dup_files_file ON duplicate_files(file_id);
CREATE INDEX idx_dup_files_canonical ON duplicate_files(is_canonical) WHERE is_canonical = TRUE;

-- -----------------------------------------------------------------------------
-- RHYTHM_PATTERNS table indexes
-- -----------------------------------------------------------------------------
CREATE INDEX idx_rhythm_file ON rhythm_patterns(file_id);
CREATE INDEX idx_rhythm_type ON rhythm_patterns(pattern_type) WHERE pattern_type IS NOT NULL;
CREATE INDEX idx_rhythm_signature ON rhythm_patterns USING hash(pattern_signature);
CREATE INDEX idx_rhythm_groove ON rhythm_patterns USING ivfflat (groove_template vector_cosine_ops) WITH (lists = 50);

-- -----------------------------------------------------------------------------
-- HARMONIC_PATTERNS table indexes
-- -----------------------------------------------------------------------------
CREATE INDEX idx_harmonic_file ON harmonic_patterns(file_id);
CREATE INDEX idx_harmonic_hash ON harmonic_patterns USING hash(progression_hash);
CREATE INDEX idx_harmonic_length ON harmonic_patterns(progression_length);
CREATE INDEX idx_harmonic_complexity ON harmonic_patterns(harmonic_complexity) WHERE harmonic_complexity IS NOT NULL;

-- -----------------------------------------------------------------------------
-- MELODIC_PATTERNS table indexes
-- -----------------------------------------------------------------------------
CREATE INDEX idx_melodic_file ON melodic_patterns(file_id);
CREATE INDEX idx_melodic_hash ON melodic_patterns USING hash(melodic_hash);
CREATE INDEX idx_melodic_range ON melodic_patterns(melodic_range) WHERE melodic_range IS NOT NULL;
CREATE INDEX idx_melodic_motion ON melodic_patterns(stepwise_motion_ratio);

-- -----------------------------------------------------------------------------
-- PROCESSING_JOBS table indexes
-- -----------------------------------------------------------------------------
CREATE INDEX idx_jobs_status ON processing_jobs(status);
CREATE INDEX idx_jobs_type ON processing_jobs(job_type);
CREATE INDEX idx_jobs_created ON processing_jobs(created_at DESC);
CREATE INDEX idx_jobs_started ON processing_jobs(started_at DESC) WHERE started_at IS NOT NULL;

-- -----------------------------------------------------------------------------
-- PROCESSING_ERRORS table indexes
-- -----------------------------------------------------------------------------
CREATE INDEX idx_errors_job ON processing_errors(job_id);
CREATE INDEX idx_errors_type ON processing_errors(error_type);
CREATE INDEX idx_errors_occurred ON processing_errors(occurred_at DESC);

-- =============================================================================
-- TRIGGERS
-- =============================================================================

-- -----------------------------------------------------------------------------
-- TRIGGER: Update search_vector on files table
-- -----------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION files_search_vector_update() RETURNS trigger AS $$
BEGIN
    NEW.search_vector :=
        setweight(to_tsvector('english', COALESCE(NEW.filename, '')), 'A') ||
        setweight(to_tsvector('english', COALESCE(NEW.manufacturer, '')), 'B') ||
        setweight(to_tsvector('english', COALESCE(NEW.collection_name, '')), 'B') ||
        setweight(to_tsvector('english', COALESCE(array_to_string(NEW.folder_tags, ' '), '')), 'C');
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER files_search_vector_trigger
    BEFORE INSERT OR UPDATE ON files
    FOR EACH ROW
    EXECUTE FUNCTION files_search_vector_update();

-- -----------------------------------------------------------------------------
-- TRIGGER: Update updated_at timestamp in files table
-- -----------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION update_updated_at_column() RETURNS trigger AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER files_updated_at_trigger
    BEFORE UPDATE ON files
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- -----------------------------------------------------------------------------
-- TRIGGER: Update tag usage_count
-- -----------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION update_tag_usage_count() RETURNS trigger AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE tags SET usage_count = usage_count + 1 WHERE id = NEW.tag_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE tags SET usage_count = usage_count - 1 WHERE id = OLD.tag_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER file_tags_usage_trigger
    AFTER INSERT OR DELETE ON file_tags
    FOR EACH ROW
    EXECUTE FUNCTION update_tag_usage_count();

-- -----------------------------------------------------------------------------
-- TRIGGER: Update duplicate_groups count when duplicate_files change
-- -----------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION update_duplicate_group_count() RETURNS trigger AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE duplicate_groups
        SET duplicate_count = duplicate_count + 1
        WHERE id = NEW.group_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE duplicate_groups
        SET duplicate_count = duplicate_count - 1
        WHERE id = OLD.group_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER duplicate_files_count_trigger
    AFTER INSERT OR DELETE ON duplicate_files
    FOR EACH ROW
    EXECUTE FUNCTION update_duplicate_group_count();

-- -----------------------------------------------------------------------------
-- TRIGGER: Update processing_jobs progress on errors
-- -----------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION update_job_progress() RETURNS trigger AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE processing_jobs
        SET failed_files = failed_files + 1
        WHERE id = NEW.job_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER processing_errors_count_trigger
    AFTER INSERT ON processing_errors
    FOR EACH ROW
    EXECUTE FUNCTION update_job_progress();

-- =============================================================================
-- VIEWS FOR COMMON QUERIES
-- =============================================================================

-- -----------------------------------------------------------------------------
-- VIEW: Complete file information with metadata
-- -----------------------------------------------------------------------------
CREATE VIEW files_with_metadata AS
SELECT
    f.id,
    f.filename,
    f.filepath,
    f.manufacturer,
    f.collection_name,
    f.duration_seconds,
    m.bpm,
    m.key_signature,
    m.time_signature_numerator,
    m.time_signature_denominator,
    m.total_notes,
    m.is_percussive,
    m.has_chords,
    m.has_melody,
    c.primary_category,
    c.secondary_category,
    f.created_at
FROM files f
LEFT JOIN musical_metadata m ON f.id = m.file_id
LEFT JOIN file_categories c ON f.id = c.file_id;

-- -----------------------------------------------------------------------------
-- VIEW: Files with tag names
-- -----------------------------------------------------------------------------
CREATE VIEW files_with_tags AS
SELECT
    f.id,
    f.filename,
    f.filepath,
    array_agg(t.name) FILTER (WHERE t.name IS NOT NULL) as tag_names
FROM files f
LEFT JOIN file_tags ft ON f.id = ft.file_id
LEFT JOIN tags t ON ft.tag_id = t.id
GROUP BY f.id, f.filename, f.filepath;

-- -----------------------------------------------------------------------------
-- VIEW: Duplicate file summary
-- -----------------------------------------------------------------------------
CREATE VIEW duplicate_summary AS
SELECT
    dg.id as group_id,
    dg.duplicate_count,
    dg.total_size_bytes,
    array_agg(f.filepath) as filepaths
FROM duplicate_groups dg
JOIN duplicate_files df ON dg.id = df.group_id
JOIN files f ON df.file_id = f.id
GROUP BY dg.id, dg.duplicate_count, dg.total_size_bytes;

-- =============================================================================
-- COMMENTS
-- =============================================================================

COMMENT ON TABLE files IS 'Main table storing all MIDI file metadata and filesystem information';
COMMENT ON TABLE musical_metadata IS 'Musical analysis data including BPM, key, time signature, and note statistics';
COMMENT ON TABLE file_categories IS 'Primary/secondary/tertiary category classification for files';
COMMENT ON TABLE file_instruments IS 'MIDI instruments detected in each file with usage statistics';
COMMENT ON TABLE tags IS 'Tag definitions for flexible file organization';
COMMENT ON TABLE file_tags IS 'Many-to-many relationship between files and tags';
COMMENT ON TABLE file_embeddings IS 'Vector embeddings for similarity search using pgvector';
COMMENT ON TABLE duplicate_groups IS 'Groups of duplicate files identified by content hash';
COMMENT ON TABLE duplicate_files IS 'Individual files within duplicate groups';
COMMENT ON TABLE rhythm_patterns IS 'Rhythmic analysis including onset times and groove templates';
COMMENT ON TABLE harmonic_patterns IS 'Chord progressions and harmonic analysis';
COMMENT ON TABLE melodic_patterns IS 'Melodic contours and interval sequences';
COMMENT ON TABLE file_compatibility IS 'Pre-computed compatibility scores between file pairs';
COMMENT ON TABLE processing_jobs IS 'Track batch processing jobs and their status';
COMMENT ON TABLE processing_errors IS 'Errors encountered during batch processing';

COMMENT ON COLUMN files.content_hash IS 'SHA-256 hash for deduplication';
COMMENT ON COLUMN files.search_vector IS 'Full-text search vector (auto-maintained by trigger)';
COMMENT ON COLUMN files.folder_tags IS 'Tags extracted from folder structure';
COMMENT ON COLUMN musical_metadata.tempo_changes IS 'JSONB array: [{tick: 0, bpm: 120}, ...]';
COMMENT ON COLUMN musical_metadata.key_changes IS 'JSONB array: [{tick: 0, key: "C"}, ...]';
COMMENT ON COLUMN file_embeddings.overall_embedding IS 'Combined embedding for overall similarity (768-dim)';
COMMENT ON COLUMN file_compatibility.overall_score IS 'Combined compatibility score (0-1)';

-- =============================================================================
-- RECORD MIGRATION
-- =============================================================================

INSERT INTO schema_migrations (version, description)
VALUES ('001', 'Initial schema - complete database structure with all tables, indexes, and triggers');

-- =============================================================================
-- VERIFY SCHEMA
-- =============================================================================

DO $$
DECLARE
    table_count INTEGER;
    index_count INTEGER;
    trigger_count INTEGER;
BEGIN
    -- Count tables
    SELECT COUNT(*) INTO table_count
    FROM information_schema.tables
    WHERE table_schema = 'public'
    AND table_type = 'BASE TABLE'
    AND table_name NOT IN ('spatial_ref_sys');

    -- Count indexes
    SELECT COUNT(*) INTO index_count
    FROM pg_indexes
    WHERE schemaname = 'public';

    -- Count triggers
    SELECT COUNT(*) INTO trigger_count
    FROM information_schema.triggers
    WHERE trigger_schema = 'public';

    RAISE NOTICE 'Schema verification:';
    RAISE NOTICE '  Tables created: %', table_count;
    RAISE NOTICE '  Indexes created: %', index_count;
    RAISE NOTICE '  Triggers created: %', trigger_count;

    IF table_count < 15 THEN
        RAISE EXCEPTION 'Expected at least 15 tables, only created %', table_count;
    END IF;

    IF index_count < 60 THEN
        RAISE WARNING 'Expected at least 60 indexes, only created %', index_count;
    END IF;

    IF trigger_count < 5 THEN
        RAISE WARNING 'Expected at least 5 triggers, only created %', trigger_count;
    END IF;

    RAISE NOTICE 'Schema migration 001 completed successfully';
    RAISE NOTICE 'Database ready for MIDI Library System';
END $$;

COMMIT;

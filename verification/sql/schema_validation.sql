-- Schema Validation for MIDI Software Center
-- Run with: psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -f verification/sql/schema_validation.sql
--
-- This validation script checks that the database schema matches the expected
-- structure defined in the model files (shared/rust/src/db/models/).

-- ============================================================================
-- TABLE EXISTENCE VERIFICATION
-- ============================================================================

DO $$
DECLARE
    missing_tables TEXT := '';
    optional_missing TEXT := '';
    -- Core required tables (from 001_initial_schema.sql)
    required_tables TEXT[] := ARRAY[
        'files',
        'musical_metadata',
        'tags',
        'file_tags',
        'file_instruments',
        'file_categories',
        'file_embeddings',
        'duplicate_groups',
        'duplicate_files',
        'rhythm_patterns',
        'harmonic_patterns',
        'melodic_patterns',
        'processing_jobs',
        'processing_errors',
        'schema_migrations'
    ];
    -- Optional tables (may be added in later migrations)
    optional_tables TEXT[] := ARRAY[
        'analysis_results',
        'drum_patterns',
        'import_batches',
        'midi_tracks',
        'search_index'
    ];
    tbl TEXT;
BEGIN
    FOREACH tbl IN ARRAY required_tables LOOP
        IF NOT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'public' AND table_name = tbl
        ) THEN
            missing_tables := missing_tables || tbl || ', ';
        END IF;
    END LOOP;

    FOREACH tbl IN ARRAY optional_tables LOOP
        IF NOT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'public' AND table_name = tbl
        ) THEN
            optional_missing := optional_missing || tbl || ', ';
        END IF;
    END LOOP;

    IF missing_tables != '' THEN
        RAISE EXCEPTION 'Missing required tables: %', rtrim(missing_tables, ', ');
    ELSE
        RAISE NOTICE 'All required tables exist';
    END IF;

    IF optional_missing != '' THEN
        RAISE NOTICE 'Optional tables not present (may be added later): %', rtrim(optional_missing, ', ');
    END IF;
END $$;

-- ============================================================================
-- COLUMN VERIFICATION FOR FILES TABLE
-- ============================================================================

DO $$
DECLARE
    missing_columns TEXT := '';
    -- Matches shared/rust/src/db/models/midi_file.rs MidiFile struct
    -- and 001_initial_schema.sql files table
    required_columns TEXT[] := ARRAY[
        'id',
        'filename',
        'filepath',
        'original_filename',
        'content_hash',
        'file_size_bytes',
        'format',
        'num_tracks',
        'ticks_per_quarter_note',
        'duration_seconds',
        'duration_ticks',
        'is_multi_track',
        'parent_file_id',
        'track_number',
        'total_tracks',
        'manufacturer',
        'collection_name',
        'folder_tags',
        'search_vector',
        'created_at',
        'updated_at',
        'analyzed_at',
        'import_batch_id'
    ];
    col TEXT;
BEGIN
    FOREACH col IN ARRAY required_columns LOOP
        IF NOT EXISTS (
            SELECT FROM information_schema.columns
            WHERE table_name = 'files' AND column_name = col
        ) THEN
            missing_columns := missing_columns || col || ', ';
        END IF;
    END LOOP;

    IF missing_columns != '' THEN
        RAISE EXCEPTION 'Missing columns in files table: %', rtrim(missing_columns, ', ');
    ELSE
        RAISE NOTICE 'Files table has all required columns';
    END IF;
END $$;

-- ============================================================================
-- COLUMN VERIFICATION FOR MUSICAL_METADATA TABLE
-- ============================================================================

DO $$
DECLARE
    missing_columns TEXT := '';
    -- Matches shared/rust/src/db/models/midi.rs MidiMetadata struct
    -- and 001_initial_schema.sql musical_metadata table
    -- Note: file_id is PRIMARY KEY (no separate id column)
    required_columns TEXT[] := ARRAY[
        'file_id',
        'bpm',
        'bpm_confidence',
        'has_tempo_changes',
        'tempo_changes',
        'key_signature',
        'key_confidence',
        'has_key_changes',
        'key_changes',
        'time_signature_numerator',
        'time_signature_denominator',
        'has_time_signature_changes',
        'time_signature_changes',
        'total_notes',
        'unique_pitches',
        'pitch_range_min',
        'pitch_range_max',
        'avg_velocity',
        'note_density',
        'polyphony_max',
        'polyphony_avg',
        'is_monophonic',
        'is_polyphonic',
        'is_percussive',
        'has_chords',
        'chord_complexity',
        'has_melody',
        'melodic_range',
        'created_at'
    ];
    col TEXT;
BEGIN
    FOREACH col IN ARRAY required_columns LOOP
        IF NOT EXISTS (
            SELECT FROM information_schema.columns
            WHERE table_name = 'musical_metadata' AND column_name = col
        ) THEN
            missing_columns := missing_columns || col || ', ';
        END IF;
    END LOOP;

    IF missing_columns != '' THEN
        RAISE EXCEPTION 'Missing columns in musical_metadata table: %', rtrim(missing_columns, ', ');
    ELSE
        RAISE NOTICE 'Musical_metadata table has all required columns';
    END IF;
END $$;

-- ============================================================================
-- COLUMN VERIFICATION FOR FILE_INSTRUMENTS TABLE
-- ============================================================================

DO $$
DECLARE
    missing_columns TEXT := '';
    -- Matches shared/rust/src/db/models/midi.rs FileInstrument struct
    required_columns TEXT[] := ARRAY[
        'id',
        'file_id',
        'channel',
        'program_number',
        'program_name',
        'instrument_family',
        'instrument_type',
        'note_count',
        'is_primary',
        'avg_velocity',
        'pitch_range_low',
        'pitch_range_high',
        'created_at'
    ];
    col TEXT;
BEGIN
    FOREACH col IN ARRAY required_columns LOOP
        IF NOT EXISTS (
            SELECT FROM information_schema.columns
            WHERE table_name = 'file_instruments' AND column_name = col
        ) THEN
            missing_columns := missing_columns || col || ', ';
        END IF;
    END LOOP;

    IF missing_columns != '' THEN
        RAISE EXCEPTION 'Missing columns in file_instruments table: %', rtrim(missing_columns, ', ');
    ELSE
        RAISE NOTICE 'File_instruments table has all required columns';
    END IF;
END $$;

-- ============================================================================
-- COLUMN VERIFICATION FOR TAGS TABLE
-- ============================================================================

DO $$
DECLARE
    missing_columns TEXT := '';
    -- Matches shared/rust/src/db/models/analysis.rs Tag struct
    required_columns TEXT[] := ARRAY[
        'id',
        'name',
        'category',
        'usage_count',
        'created_at'
    ];
    col TEXT;
BEGIN
    FOREACH col IN ARRAY required_columns LOOP
        IF NOT EXISTS (
            SELECT FROM information_schema.columns
            WHERE table_name = 'tags' AND column_name = col
        ) THEN
            missing_columns := missing_columns || col || ', ';
        END IF;
    END LOOP;

    IF missing_columns != '' THEN
        RAISE EXCEPTION 'Missing columns in tags table: %', rtrim(missing_columns, ', ');
    ELSE
        RAISE NOTICE 'Tags table has all required columns';
    END IF;
END $$;

-- ============================================================================
-- INDEX VERIFICATION
-- ============================================================================

DO $$
DECLARE
    missing_indexes TEXT := '';
    -- Key indexes from 001_initial_schema.sql
    required_indexes TEXT[] := ARRAY[
        'idx_files_content_hash',
        'idx_files_filepath',
        'idx_files_search',
        'idx_files_created',
        'idx_metadata_bpm',
        'idx_metadata_key',
        'idx_file_tags_tag',
        'idx_file_tags_file',
        'idx_instruments_file',
        'idx_tags_name_trgm'
    ];
    idx TEXT;
BEGIN
    FOREACH idx IN ARRAY required_indexes LOOP
        IF NOT EXISTS (
            SELECT FROM pg_indexes
            WHERE schemaname = 'public' AND indexname = idx
        ) THEN
            missing_indexes := missing_indexes || idx || ', ';
        END IF;
    END LOOP;

    IF missing_indexes != '' THEN
        RAISE WARNING 'Missing performance indexes: %', rtrim(missing_indexes, ', ');
    ELSE
        RAISE NOTICE 'All required indexes exist';
    END IF;
END $$;

-- ============================================================================
-- FOREIGN KEY VERIFICATION
-- ============================================================================

DO $$
DECLARE
    fk_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO fk_count
    FROM information_schema.table_constraints
    WHERE constraint_type = 'FOREIGN KEY'
    AND table_schema = 'public';

    IF fk_count < 5 THEN
        RAISE WARNING 'Low foreign key count (%). Data integrity may be at risk.', fk_count;
    ELSE
        RAISE NOTICE 'Foreign key constraints verified: % constraints found', fk_count;
    END IF;
END $$;

-- ============================================================================
-- DATA INTEGRITY CHECKS
-- ============================================================================

DO $$
DECLARE
    file_count BIGINT;
    metadata_count BIGINT;
    orphan_count BIGINT;
BEGIN
    -- Check file count
    SELECT COUNT(*) INTO file_count FROM files;
    RAISE NOTICE 'Total files in database: %', file_count;

    IF file_count = 0 THEN
        RAISE WARNING 'No files in database - migration or import may have failed';
    END IF;

    -- Check metadata coverage
    SELECT COUNT(*) INTO metadata_count FROM musical_metadata;
    RAISE NOTICE 'Files with musical metadata: %', metadata_count;

    IF file_count > 0 AND metadata_count::FLOAT / file_count::FLOAT < 0.5 THEN
        RAISE WARNING 'Less than 50%% of files have metadata. Analysis may be incomplete.';
    END IF;

    -- Check for orphaned metadata
    SELECT COUNT(*) INTO orphan_count
    FROM musical_metadata m
    LEFT JOIN files f ON m.file_id = f.id
    WHERE f.id IS NULL;

    IF orphan_count > 0 THEN
        RAISE WARNING 'Found % orphaned metadata records without corresponding files', orphan_count;
    ELSE
        RAISE NOTICE 'No orphaned metadata records found';
    END IF;
END $$;

-- ============================================================================
-- TAG SYSTEM VERIFICATION
-- ============================================================================

DO $$
DECLARE
    tag_count INTEGER;
    relationship_count BIGINT;
BEGIN
    SELECT COUNT(*) INTO tag_count FROM tags;
    SELECT COUNT(*) INTO relationship_count FROM file_tags;

    RAISE NOTICE 'Tags defined: %, File-tag relationships: %', tag_count, relationship_count;

    IF tag_count < 50 THEN
        RAISE WARNING 'Low tag count (%). Consider running tagging system.', tag_count;
    END IF;

    IF relationship_count = 0 AND tag_count > 0 THEN
        RAISE WARNING 'Tags exist but no files are tagged. Run tagging process.';
    END IF;
END $$;

-- ============================================================================
-- SUMMARY REPORT
-- ============================================================================

SELECT
    'Schema Validation Summary' as report_type,
    (SELECT COUNT(*) FROM files) as total_files,
    (SELECT COUNT(*) FROM musical_metadata) as files_with_metadata,
    (SELECT COUNT(*) FROM tags) as total_tags,
    (SELECT COUNT(*) FROM file_tags) as tag_relationships,
    (SELECT COUNT(*) FROM pg_indexes WHERE schemaname = 'public') as total_indexes,
    NOW() as validation_timestamp;

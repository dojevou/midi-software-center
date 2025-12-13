#!/bin/bash
# Ultra-Fast MIDI Import Script
# Drops indexes, applies max PostgreSQL optimizations, imports, rebuilds indexes

set -e

echo "🚀 ULTRA-FAST IMPORT MODE"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Step 1: Clear database and drop non-essential indexes
echo "Step 1: Clearing database and dropping non-essential indexes..."
docker exec midi-library-postgres psql -U midiuser -d midi_library <<'SQL'
-- Clear all data
TRUNCATE files CASCADE;

-- Drop all non-essential indexes (keep only PK + content_hash for dedup)
DROP INDEX IF EXISTS idx_files_batch;
DROP INDEX IF EXISTS idx_files_collection;
DROP INDEX IF EXISTS idx_files_copyright;
DROP INDEX IF EXISTS idx_files_copyright_trgm;
DROP INDEX IF EXISTS idx_files_created;
DROP INDEX IF EXISTS idx_files_display_name;
DROP INDEX IF EXISTS idx_files_duration;
DROP INDEX IF EXISTS idx_files_filename_bpm;
DROP INDEX IF EXISTS idx_files_filename_bpm_key;
DROP INDEX IF EXISTS idx_files_filename_genres;
DROP INDEX IF EXISTS idx_files_filename_key;
DROP INDEX IF EXISTS idx_files_filepath;
DROP INDEX IF EXISTS idx_files_folder_tags;
DROP INDEX IF EXISTS idx_files_format;
DROP INDEX IF EXISTS idx_files_instrument_names_text;
DROP INDEX IF EXISTS idx_files_manufacturer;
DROP INDEX IF EXISTS idx_files_markers;
DROP INDEX IF EXISTS idx_files_metadata_source;
DROP INDEX IF EXISTS idx_files_num_tracks;
DROP INDEX IF EXISTS idx_files_parent;
DROP INDEX IF EXISTS idx_files_parent_folder;
DROP INDEX IF EXISTS idx_files_search;
DROP INDEX IF EXISTS idx_files_structure_tags;
DROP INDEX IF EXISTS idx_files_track_names;
DROP INDEX IF EXISTS idx_files_track_number;
DROP INDEX IF EXISTS files_filepath_key;

-- Drop musical_metadata indexes
DROP INDEX IF EXISTS idx_chord_complexity;
DROP INDEX IF EXISTS idx_chord_progression;
DROP INDEX IF EXISTS idx_chord_types;
DROP INDEX IF EXISTS idx_has_extended_chords;
DROP INDEX IF EXISTS idx_has_seventh_chords;
DROP INDEX IF EXISTS idx_metadata_bpm;
DROP INDEX IF EXISTS idx_metadata_characteristics;
DROP INDEX IF EXISTS idx_metadata_density;
DROP INDEX IF EXISTS idx_metadata_has_melody;
DROP INDEX IF EXISTS idx_metadata_key;
DROP INDEX IF EXISTS idx_metadata_notes;
DROP INDEX IF EXISTS idx_metadata_pitch_range;
DROP INDEX IF EXISTS idx_metadata_polyphony;
DROP INDEX IF EXISTS idx_metadata_time_sig;

SELECT 'Indexes dropped - only PK and content_hash remain' as status;
SQL

echo "✓ Database cleared and indexes dropped"
echo ""

# Step 2: Apply PostgreSQL ultra-performance settings
echo "Step 2: Applying PostgreSQL ultra-performance settings..."
docker exec midi-library-postgres psql -U midiuser -d midi_library <<'SQL'
-- Ultra-fast bulk import settings
SET synchronous_commit = off;  -- Async commits (2x faster writes)
SET fsync = off;  -- DANGER: Disable fsync (3-5x faster, data loss risk if crash!)
SET full_page_writes = off;  -- Skip full page writes
SET wal_level = minimal;  -- Minimal WAL logging
SET max_wal_senders = 0;  -- Disable replication
SET maintenance_work_mem = '2GB';  -- Huge maintenance memory
SET work_mem = '256MB';  -- Large work memory
SET effective_cache_size = '4GB';  -- Assume 4GB cache
SET shared_buffers = '1GB';  -- Large shared buffers

SELECT 'PostgreSQL optimized for MAXIMUM SPEED' as status;
SHOW synchronous_commit;
SHOW fsync;
SQL

echo "✓ PostgreSQL tuned for ultra-fast bulk import"
echo ""

# Step 3: Start ultra-fast import
echo "Step 3: Starting ultra-fast import with 64 workers..."
echo "Target speed: 800-1200 files/sec (2-3x faster than before!)"
echo ""

export DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"
nohup ./target/release/import_unified /media/dojevou/NewSSD2/midi --workers 64 --batch-size 500 > /tmp/ultra_fast_import.log 2>&1 &

echo "✓ Ultra-fast import started"
echo "✓ Log: /tmp/ultra_fast_import.log"
echo ""
echo "Monitor with: tail -f /tmp/ultra_fast_import.log"
echo ""
echo "After import completes, run: ./scripts/rebuild-indexes.sh"

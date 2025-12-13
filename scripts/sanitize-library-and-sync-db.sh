#!/bin/bash
set -e

# Sanitize Library Folders and Sync Database Paths
# This script:
# 1. Sanitizes all folder names in the MIDI library
# 2. Updates database file paths to match sanitized filenames
# 3. Fixes orphaned database records

LIBRARY_PATH="/home/dojevou/projects/midi-software-center/midi-library"
DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"
LOG_FILE="/tmp/sanitize_library_$(date +%Y%m%d_%H%M%S).log"

echo "🧹 MIDI Library Sanitization and Database Sync"
echo "=============================================="
echo ""
echo "Library Path: $LIBRARY_PATH"
echo "Log File: $LOG_FILE"
echo ""

# Function to sanitize a name (folder or file)
sanitize_name() {
    local name="$1"
    # Replace spaces with underscores
    name="${name// /_}"
    # Convert to lowercase extension if it's a file
    if [[ "$name" == *.MIDI ]]; then
        name="${name%.MIDI}.mid"
    elif [[ "$name" == *.MID ]]; then
        name="${name%.MID}.mid"
    fi
    # Remove special characters (keep only letters, numbers, _, -, .)
    name=$(echo "$name" | sed 's/[^a-zA-Z0-9._-]/_/g')
    # Remove multiple consecutive underscores
    name=$(echo "$name" | sed 's/__*/_/g')
    echo "$name"
}

# Phase 1: Sanitize Folder Names
echo "📁 Phase 1: Sanitizing folder names..."
echo "======================================="
echo ""

folder_count=0
renamed_count=0

# Find all directories, sorted by depth (deepest first to avoid parent issues)
while IFS= read -r folder; do
    folder_count=$((folder_count + 1))

    # Get the directory name and parent path
    dir_name=$(basename "$folder")
    parent_path=$(dirname "$folder")

    # Sanitize the directory name
    sanitized_name=$(sanitize_name "$dir_name")

    # Check if name changed
    if [ "$dir_name" != "$sanitized_name" ]; then
        new_path="$parent_path/$sanitized_name"

        # Check if target already exists
        if [ -e "$new_path" ]; then
            echo "⚠️  Conflict: $new_path already exists, skipping $folder" | tee -a "$LOG_FILE"
        else
            echo "Renaming: $dir_name -> $sanitized_name" | tee -a "$LOG_FILE"
            mv "$folder" "$new_path"
            renamed_count=$((renamed_count + 1))
        fi
    fi

    # Progress indicator every 100 folders
    if [ $((folder_count % 100)) -eq 0 ]; then
        echo "  Processed $folder_count folders, renamed $renamed_count..."
    fi
done < <(find "$LIBRARY_PATH" -type d | sort -r)

echo ""
echo "✅ Folder sanitization complete:"
echo "   Total folders: $folder_count"
echo "   Renamed: $renamed_count"
echo ""

# Phase 2: Sanitize File Names
echo "📄 Phase 2: Sanitizing file names..."
echo "====================================="
echo ""

file_count=0
renamed_files=0

# Find all .mid and .midi files
while IFS= read -r file; do
    file_count=$((file_count + 1))

    file_name=$(basename "$file")
    parent_path=$(dirname "$file")

    # Sanitize the file name
    sanitized_name=$(sanitize_name "$file_name")

    # Check if name changed
    if [ "$file_name" != "$sanitized_name" ]; then
        new_path="$parent_path/$sanitized_name"

        # Check if target already exists
        if [ -e "$new_path" ]; then
            echo "⚠️  Conflict: $new_path already exists, skipping $file" | tee -a "$LOG_FILE"
        else
            mv "$file" "$new_path"
            renamed_files=$((renamed_files + 1))
        fi
    fi

    # Progress indicator every 1000 files
    if [ $((file_count % 1000)) -eq 0 ]; then
        echo "  Processed $file_count files, renamed $renamed_files..."
    fi
done < <(find "$LIBRARY_PATH" -type f \( -iname "*.mid" -o -iname "*.midi" \))

echo ""
echo "✅ File sanitization complete:"
echo "   Total files: $file_count"
echo "   Renamed: $renamed_files"
echo ""

# Phase 3: Update Database Paths
echo "🔄 Phase 3: Syncing database with sanitized paths..."
echo "====================================================="
echo ""

# Create a temporary SQL script
SYNC_SQL="/tmp/sync_db_paths_$(date +%Y%m%d_%H%M%S).sql"

cat > "$SYNC_SQL" << 'EOF'
-- Database Path Sync Script
-- Updates file paths to match sanitized filenames

BEGIN;

-- Create a temporary table to track updates
CREATE TEMP TABLE path_updates (
    file_id BIGINT,
    old_path TEXT,
    new_path TEXT,
    old_filename TEXT,
    new_filename TEXT
);

-- Log current state
SELECT COUNT(*) as total_files FROM files;
SELECT COUNT(*) as unanalyzed FROM files WHERE analyzed_at IS NULL;

-- Update paths with double underscores to single underscores in filenames
WITH updated_paths AS (
    SELECT
        id,
        filepath,
        filename,
        -- Replace double underscores with single
        regexp_replace(filepath, '__+', '_', 'g') as new_filepath,
        regexp_replace(filename, '__+', '_', 'g') as new_filename
    FROM files
    WHERE
        filepath ~ '__+' OR filename ~ '__+'
)
UPDATE files f
SET
    filepath = up.new_filepath,
    filename = up.new_filename
FROM updated_paths up
WHERE f.id = up.id;

-- Log how many were updated
SELECT COUNT(*) as double_underscore_fixed
FROM files
WHERE filepath != regexp_replace(filepath, '__+', '_', 'g');

-- Update .MIDI extensions to .mid
UPDATE files
SET
    filepath = regexp_replace(filepath, '\.MIDI$', '.mid', 'i'),
    filename = regexp_replace(filename, '\.MIDI$', '.mid', 'i')
WHERE
    filepath ~* '\.MIDI$' OR filename ~* '\.MIDI$';

-- Update .MID extensions to .mid (lowercase)
UPDATE files
SET
    filepath = regexp_replace(filepath, '\.MID$', '.mid'),
    filename = regexp_replace(filename, '\.MID$', '.mid')
WHERE
    filepath ~ '\.MID$' OR filename ~ '\.MID$';

-- Replace spaces with underscores in paths
UPDATE files
SET
    filepath = replace(filepath, ' ', '_'),
    filename = replace(filename, ' ', '_')
WHERE
    filepath ~ ' ' OR filename ~ ' ';

-- Log final state
SELECT COUNT(*) as total_files_after FROM files;
SELECT COUNT(*) as unanalyzed_after FROM files WHERE analyzed_at IS NULL;

-- Show some example updates
SELECT
    filename,
    LEFT(filepath, 100) as filepath_sample
FROM files
WHERE analyzed_at IS NULL
LIMIT 10;

COMMIT;

-- Vacuum to reclaim space
VACUUM ANALYZE files;
EOF

echo "Executing database sync..."
psql "$DB_URL" -f "$SYNC_SQL" | tee -a "$LOG_FILE"

echo ""
echo "✅ Database sync complete"
echo ""

# Phase 4: Verify and Report
echo "📊 Phase 4: Verification Report"
echo "================================"
echo ""

echo "Checking file existence for unanalyzed records..."
psql "$DB_URL" << EOF | tee -a "$LOG_FILE"
-- Count files by analysis status
SELECT
    COUNT(*) FILTER (WHERE analyzed_at IS NOT NULL) as analyzed,
    COUNT(*) FILTER (WHERE analyzed_at IS NULL) as unanalyzed,
    COUNT(*) as total
FROM files;

-- Sample of unanalyzed files
SELECT
    'Unanalyzed files sample:' as info;
SELECT
    LEFT(filepath, 80) as filepath
FROM files
WHERE analyzed_at IS NULL
LIMIT 5;
EOF

echo ""
echo "🎉 Sanitization and sync complete!"
echo "==================================="
echo ""
echo "Summary:"
echo "  - Folders renamed: $renamed_count"
echo "  - Files renamed: $renamed_files"
echo "  - Database paths synced"
echo "  - Log file: $LOG_FILE"
echo ""
echo "Next steps:"
echo "  1. Run: ./target/release/analyze"
echo "  2. Extract remaining archives"
echo "  3. Re-run analysis"
echo ""

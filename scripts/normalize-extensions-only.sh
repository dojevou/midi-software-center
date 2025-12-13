#!/bin/bash
set -e

# Normalize MIDI File Extensions Only
# This script ONLY fixes .MID, .midi, and .Mid extensions to .mid
# Keeps database and filesystem in sync

DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"
LOG_FILE="/tmp/extension_normalize_$(date +%Y%m%d_%H%M%S).log"

echo "🎵 MIDI Extension Normalization"
echo "================================"
echo ""
echo "Target extensions: .MID, .midi, .Mid"
echo "Target extension:  .mid"
echo "Log File: $LOG_FILE"
echo ""

# Get counts from database
echo "📊 Current database state:"
psql "$DB_URL" -t -c "
SELECT
  SUBSTRING(filename FROM '\.([^.]+)$') as extension,
  COUNT(*) as count
FROM files
GROUP BY extension
ORDER BY count DESC;
" | tee -a "$LOG_FILE"

echo ""
echo "⚠️  This will rename files on disk AND update database records."
read -p "Continue? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 1
fi

echo ""
echo "🔄 Phase 1: Renaming files on disk..."
echo "======================================"
echo ""

# Create temporary SQL to get files needing rename
QUERY_SQL="/tmp/files_to_rename.sql"
cat > "$QUERY_SQL" << 'EOF'
COPY (
    SELECT
        id,
        filepath,
        filename
    FROM files
    WHERE filename ~ '\.(MID|midi|Mid)$'
    ORDER BY id
) TO STDOUT WITH (FORMAT CSV, HEADER);
EOF

# Export to CSV
RENAME_LIST="/tmp/files_to_rename.csv"
psql "$DB_URL" -f "$QUERY_SQL" > "$RENAME_LIST"

# Count files to rename
file_count=$(tail -n +2 "$RENAME_LIST" | wc -l)
echo "Files to rename: $file_count"
echo ""

renamed_count=0
error_count=0

# Process each file
while IFS=',' read -r id filepath filename; do
    # Skip header
    if [ "$id" = "id" ]; then
        continue
    fi

    # Determine new filename based on current extension
    if [[ "$filename" == *.MID ]]; then
        new_filename="${filename%.MID}.mid"
    elif [[ "$filename" == *.midi ]]; then
        new_filename="${filename%.midi}.mid"
    elif [[ "$filename" == *.Mid ]]; then
        new_filename="${filename%.Mid}.mid"
    else
        continue
    fi

    # Construct new filepath
    dir_path=$(dirname "$filepath")
    new_filepath="$dir_path/$new_filename"

    # Check if old file exists
    if [ ! -f "$filepath" ]; then
        echo "⚠️  File not found: $filepath" | tee -a "$LOG_FILE"
        error_count=$((error_count + 1))
        continue
    fi

    # Check if new file already exists
    if [ -f "$new_filepath" ] && [ "$filepath" != "$new_filepath" ]; then
        echo "⚠️  Target exists: $new_filepath (skipping $filepath)" | tee -a "$LOG_FILE"
        error_count=$((error_count + 1))
        continue
    fi

    # Rename the file
    if mv "$filepath" "$new_filepath" 2>/dev/null; then
        renamed_count=$((renamed_count + 1))

        # Progress indicator every 100 files
        if [ $((renamed_count % 100)) -eq 0 ]; then
            echo "  Renamed $renamed_count files..."
        fi
    else
        echo "⚠️  Failed to rename: $filepath" | tee -a "$LOG_FILE"
        error_count=$((error_count + 1))
    fi

done < "$RENAME_LIST"

echo ""
echo "✅ File renaming complete:"
echo "   Files renamed: $renamed_count"
echo "   Errors: $error_count"
echo ""

# Phase 2: Update Database
echo "🔄 Phase 2: Updating database records..."
echo "========================================="
echo ""

UPDATE_SQL="/tmp/update_extensions.sql"
cat > "$UPDATE_SQL" << 'EOF'
BEGIN;

-- Update .MID to .mid
UPDATE files
SET
    filepath = regexp_replace(filepath, '\.MID$', '.mid'),
    filename = regexp_replace(filename, '\.MID$', '.mid')
WHERE
    filename ~ '\.MID$';

SELECT 'Updated .MID files: ' || COUNT(*)::text as result
FROM files
WHERE filename ~ '\.mid$' AND filepath ~ '\.mid$';

-- Update .midi to .mid
UPDATE files
SET
    filepath = regexp_replace(filepath, '\.midi$', '.mid'),
    filename = regexp_replace(filename, '\.midi$', '.mid')
WHERE
    filename ~ '\.midi$';

SELECT 'Updated .midi files: ' || COUNT(*)::text as result
FROM files
WHERE filename ~ '\.mid$' AND filepath ~ '\.mid$';

-- Update .Mid to .mid
UPDATE files
SET
    filepath = regexp_replace(filepath, '\.Mid$', '.mid'),
    filename = regexp_replace(filename, '\.Mid$', '.mid')
WHERE
    filename ~ '\.Mid$';

SELECT 'Updated .Mid files: ' || COUNT(*)::text as result
FROM files
WHERE filename ~ '\.mid$' AND filepath ~ '\.mid$';

COMMIT;

-- Show final counts
SELECT
  SUBSTRING(filename FROM '\.([^.]+)$') as extension,
  COUNT(*) as count
FROM files
GROUP BY extension
ORDER BY count DESC;
EOF

echo "Executing database updates..."
psql "$DB_URL" -f "$UPDATE_SQL" | tee -a "$LOG_FILE"

echo ""
echo "✅ Database update complete"
echo ""

# Phase 3: Verification
echo "📊 Phase 3: Verification"
echo "========================"
echo ""

echo "Final extension counts:"
psql "$DB_URL" -c "
SELECT
  SUBSTRING(filename FROM '\.([^.]+)$') as extension,
  COUNT(*) as count
FROM files
GROUP BY extension
ORDER BY count DESC;
"

echo ""
echo "🎉 Extension normalization complete!"
echo "===================================="
echo ""
echo "Summary:"
echo "  - Files renamed on disk: $renamed_count"
echo "  - Database records updated: (see counts above)"
echo "  - Errors encountered: $error_count"
echo "  - Log file: $LOG_FILE"
echo ""

# Cleanup temp files
rm -f "$QUERY_SQL" "$RENAME_LIST" "$UPDATE_SQL"

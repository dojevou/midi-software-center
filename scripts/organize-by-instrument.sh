#!/bin/bash
set -e

# Organize MIDI Library by Instrument
# Uses database metadata to categorize and create organized structure

DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"
LIBRARY_PATH="/home/dojevou/projects/midi-software-center/midi-library"
ORGANIZED_PATH="$LIBRARY_PATH/by-instrument"

echo "🎼 Organize MIDI Library by Instrument"
echo "======================================"
echo ""
echo "This will:"
echo "  - Analyze existing metadata in database"
echo "  - Create instrument-based directory structure"
echo "  - Create symlinks (not copies) to original files"
echo "  - Preserve original files in their locations"
echo ""

# Phase 1: Analyze metadata for instrument detection
echo "📊 Phase 1: Detect Instruments from Metadata"
echo "============================================="
echo ""

psql "$DB_URL" << 'EOSQL'
-- Check how many files have instrument names
SELECT
    COUNT(*) as files_with_instruments
FROM musical_metadata
WHERE instrument_names IS NOT NULL AND array_length(instrument_names, 1) > 0;

-- Most common instruments
SELECT
    unnest(instrument_names) as instrument,
    COUNT(*) as count
FROM musical_metadata
WHERE instrument_names IS NOT NULL
GROUP BY instrument
ORDER BY count DESC
LIMIT 20;

-- Tag-based categorization
SELECT
    unnest(tags) as tag,
    COUNT(*) as count
FROM files
WHERE tags IS NOT NULL
GROUP BY tag
ORDER BY count DESC
LIMIT 20;
EOSQL

echo ""
read -p "Continue with organization? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Organization cancelled"
    exit 0
fi

# Phase 2: Create directory structure
echo ""
echo "📁 Phase 2: Create Instrument Directories"
echo "=========================================="
echo ""

mkdir -p "$ORGANIZED_PATH"/{drums,bass,piano,guitar,strings,brass,woodwind,synth,pads,fx,loops,melodic,harmonic,other}

echo "Created directories:"
ls -la "$ORGANIZED_PATH"

# Phase 3: Categorize and create symlinks
echo ""
echo "🔗 Phase 3: Create Symlinks by Category"
echo "========================================"
echo ""

# Export file lists by category from database
echo "Exporting drums..."
psql "$DB_URL" -t -c "
SELECT filepath FROM files f
WHERE EXISTS (
    SELECT 1 FROM musical_metadata m
    WHERE m.file_id = f.id AND m.is_percussive = true
)
OR tags @> ARRAY['drums']
LIMIT 100000;
" | while IFS= read -r filepath; do
    filepath=$(echo "$filepath" | xargs)
    if [ -n "$filepath" ] && [ -f "$filepath" ]; then
        filename=$(basename "$filepath")
        ln -sf "$filepath" "$ORGANIZED_PATH/drums/$filename" 2>/dev/null || true
    fi
done

echo "Exporting bass..."
psql "$DB_URL" -t -c "
SELECT filepath FROM files
WHERE tags && ARRAY['bass']
   OR lower(filename) LIKE '%bass%'
LIMIT 100000;
" | while IFS= read -r filepath; do
    filepath=$(echo "$filepath" | xargs)
    if [ -n "$filepath" ] && [ -f "$filepath" ]; then
        filename=$(basename "$filepath")
        ln -sf "$filepath" "$ORGANIZED_PATH/bass/$filename" 2>/dev/null || true
    fi
done

echo "Exporting loops..."
psql "$DB_URL" -t -c "
SELECT filepath FROM files
WHERE tags && ARRAY['loop', 'loops']
LIMIT 100000;
" | while IFS= read -r filepath; do
    filepath=$(echo "$filepath" | xargs)
    if [ -n "$filepath" ] && [ -f "$filepath" ]; then
        filename=$(basename "$filepath")
        ln -sf "$filepath" "$ORGANIZED_PATH/loops/$filename" 2>/dev/null || true
    fi
done

# Phase 4: Summary
echo ""
echo "📊 Phase 4: Organization Summary"
echo "================================="
echo ""

for dir in "$ORGANIZED_PATH"/*; do
    if [ -d "$dir" ]; then
        count=$(find "$dir" -type l 2>/dev/null | wc -l)
        echo "$(basename "$dir"): $count files"
    fi
done

echo ""
echo "🎉 Organization Complete!"
echo "========================="
echo ""
echo "Organized library location: $ORGANIZED_PATH"
echo ""
echo "Usage:"
echo "  - Browse by instrument in: $ORGANIZED_PATH"
echo "  - Original files remain in: $LIBRARY_PATH/archives, extracted, splits"
echo "  - Symlinks allow access without duplication"
echo ""

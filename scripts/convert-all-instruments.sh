#!/bin/bash
# Convert 2,000 Patterns Per Instrument for All 97 Expansions
# Created: November 23, 2025

set -e

# Configuration
DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"
PROJECT_ROOT="/home/dojevou/projects/midi-software-center"
FORCE_DRIVE="/media/dojevou/RYXSTR"
EXPANSIONS_DIR="$FORCE_DRIVE/Expansions"
CONVERTER="$PROJECT_ROOT/target/release/midi_to_mpcpattern_parallel"
INSTRUMENT_LIST="$PROJECT_ROOT/INSTRUMENT_LIST.txt"
PATTERNS_PER_INSTRUMENT=2000
TEMP_DIR="/tmp/instrument_conversion"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Converting 97 Instrument Expansions${NC}"
echo -e "${BLUE}  2,000 patterns per instrument${NC}"
echo -e "${BLUE}  ~194,000 total patterns${NC}"
echo -e "${BLUE}========================================${NC}"
echo

# Create temp directory
mkdir -p "$TEMP_DIR"

# Counters
total_instruments=0
total_patterns=0
total_failed=0
start_time=$(date +%s)

# Read instrument list and process each
while IFS=':' read -r instrument count; do
    # Clean up instrument name
    instrument=$(echo "$instrument" | xargs)
    count=$(echo "$count" | xargs)

    # Skip empty lines
    [ -z "$instrument" ] && continue

    # Convert to expansion name format
    instrument_upper=$(echo "$instrument" | tr '[:lower:]' '[:upper:]' | tr '-' '_' | tr '&' 'AND')
    expansion_name="MIDI_${instrument_upper}"
    expansion_dir="$EXPANSIONS_DIR/$expansion_name"
    patterns_dir="$expansion_dir/Patterns"

    # Skip if expansion doesn't exist
    if [ ! -d "$expansion_dir" ]; then
        echo -e "${YELLOW}⚠️  Skipping $expansion_name (folder not found)${NC}"
        continue
    fi

    ((total_instruments++))

    echo -e "${BLUE}[${total_instruments}/97] Processing: $expansion_name${NC}"
    echo "  Available files in database: $count"

    # Extract files for this instrument from database
    psql -h localhost -p 5433 -U midiuser -d midi_library -t -c "
        SELECT DISTINCT f.filepath
        FROM files f
        LEFT JOIN file_tags ft ON f.id = ft.file_id
        LEFT JOIN tags t ON ft.tag_id = t.id
        WHERE f.num_tracks = 1
          AND (
            t.name = '$instrument'
            OR f.filename ILIKE '%$instrument%'
          )
        ORDER BY RANDOM()
        LIMIT $PATTERNS_PER_INSTRUMENT;
    " > "$TEMP_DIR/${instrument}_files.txt" 2>/dev/null

    # Count extracted files
    file_count=$(wc -l < "$TEMP_DIR/${instrument}_files.txt" | xargs)

    if [ "$file_count" -eq 0 ]; then
        echo -e "${YELLOW}  ⚠️  No files found for $instrument${NC}"
        echo
        continue
    fi

    echo "  Files to convert: $file_count"

    # Convert files
    converted=0
    failed=0

    while IFS= read -r filepath; do
        filepath=$(echo "$filepath" | xargs)

        # Skip empty lines
        [ -z "$filepath" ] && continue

        # Check file exists
        if [ ! -f "$filepath" ]; then
            ((failed++))
            continue
        fi

        # Get filename without extension
        filename=$(basename "$filepath" .mid)
        output_file="$patterns_dir/${filename}.mpcpattern"

        # Convert
        if "$CONVERTER" "$filepath" "$output_file" >/dev/null 2>&1; then
            ((converted++))
        else
            ((failed++))
        fi

        # Progress update every 100 files
        if [ $((converted % 100)) -eq 0 ] && [ $converted -gt 0 ]; then
            echo -n "."
        fi

    done < "$TEMP_DIR/${instrument}_files.txt"

    echo
    echo -e "${GREEN}  ✓ Converted: $converted patterns${NC}"
    [ $failed -gt 0 ] && echo -e "${RED}  ✗ Failed: $failed files${NC}"

    total_patterns=$((total_patterns + converted))
    total_failed=$((total_failed + failed))

    # Clean up temp file
    rm -f "$TEMP_DIR/${instrument}_files.txt"

    echo

done < "$INSTRUMENT_LIST"

# Final summary
end_time=$(date +%s)
total_time=$((end_time - start_time))
minutes=$((total_time / 60))
seconds=$((total_time % 60))

echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}  CONVERSION COMPLETE!${NC}"
echo -e "${BLUE}========================================${NC}"
echo
echo "📊 Summary:"
echo "  - Instruments processed: $total_instruments/97"
echo "  - Total patterns converted: $total_patterns"
echo "  - Failed conversions: $total_failed"
echo "  - Total time: ${minutes}m ${seconds}s"
echo "  - Average speed: $((total_patterns / (total_time > 0 ? total_time : 1))) files/sec"
echo
echo "📂 Location: $EXPANSIONS_DIR"
echo
echo "🎯 Next Steps:"
echo "  1. Test expansions on Force hardware"
echo "  2. Browser > Expansions > MIDI_[INSTRUMENT]"
echo "  3. Enjoy your library!"
echo
echo -e "${GREEN}✓ All done!${NC}"

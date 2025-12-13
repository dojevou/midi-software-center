#!/bin/bash
# Convert 5 test MIDI files with fixed converter

CONVERTER="./target/release/midi_to_mpcpattern"
MIDI_DIR="/media/dojevou/RYXSTR/Expansions/Test_Fixed_5_Patterns/MIDI"
OUTPUT_DIR="/media/dojevou/RYXSTR/Expansions/Test_Fixed_5_Patterns/Patterns"
DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

echo "Querying database for 5 clean drum files (120 BPM)..."

# Get 5 simple drum files with good BPM
psql "$DB_URL" -t -A -c "
SELECT f.filepath
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE t.name IN ('drums', 'groove')
  AND m.bpm BETWEEN 120 AND 122
  AND m.bpm IS NOT NULL
ORDER BY m.bpm, f.filepath
LIMIT 5;
" > /tmp/files_to_convert_5.txt

count=0

echo "Converting and copying files..."
echo ""

while IFS= read -r filepath; do
    # Skip empty lines
    [ -z "$filepath" ] && continue

    # Get filename without extension
    filename=$(basename "$filepath")
    basename="${filename%.mid}"

    # Copy MIDI file
    if [ -f "$filepath" ]; then
        cp "$filepath" "$MIDI_DIR/${basename}.mid"
        echo "[$((count+1))] Copied MIDI: ${basename}.mid"

        # Convert to .mpcpattern
        $CONVERTER "$filepath" "$OUTPUT_DIR/${basename}.mpcpattern" 2>&1 | grep -E "(✓|events)"

        count=$((count + 1))
        echo ""
    fi
done < /tmp/files_to_convert_5.txt

echo "✓ Complete!"
echo "  MIDI files: $MIDI_DIR"
echo "  Patterns: $OUTPUT_DIR"
echo "  Total: $count files"

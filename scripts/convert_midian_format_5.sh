#!/bin/bash
# Convert 5 test files with Midian-compatible converter

CONVERTER="./target/release/midi_to_mpcpattern"
MIDI_DIR="/media/dojevou/RYXSTR/Expansions/Test_Midian_Format/MIDI"
OUTPUT_DIR="/media/dojevou/RYXSTR/Expansions/Test_Midian_Format/Patterns"
DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

echo "Querying database for 5 drum files (120 BPM)..."

# Get 5 simple drum files
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
" > /tmp/files_midian_5.txt

count=0

echo "Converting with Midian-compatible format..."
echo ""

while IFS= read -r filepath; do
    [ -z "$filepath" ] && continue

    filename=$(basename "$filepath")
    basename="${filename%.mid}"

    if [ -f "$filepath" ]; then
        # Copy MIDI file
        cp "$filepath" "$MIDI_DIR/${basename}.mid"
        echo "[$((count+1))] MIDI: ${basename}.mid"

        # Convert to .mpcpattern
        $CONVERTER "$filepath" "$OUTPUT_DIR/${basename}.mpcpattern" 2>&1 | grep -E "✓"

        count=$((count + 1))
        echo ""
    fi
done < /tmp/files_midian_5.txt

echo "✓ Complete!"
echo "  Location: /media/dojevou/RYXSTR/Expansions/Test_Midian_Format"
echo "  Total: $count files"

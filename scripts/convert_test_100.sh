#!/bin/bash
# Convert 100 test MIDI files to .mpcpattern for Force

CONVERTER="./target/release/midi_to_mpcpattern"
OUTPUT_DIR="/media/dojevou/RYXSTR/Expansions/Database_Test_100"
DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

echo "Querying database for 100 drum files..."

# Get 100 files with BPM
psql "$DB_URL" -t -c "
SELECT f.filepath, m.bpm, f.filename
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE t.name IN ('drums', 'groove')
  AND m.bpm BETWEEN 100 AND 140
  AND m.bpm IS NOT NULL
ORDER BY m.bpm, f.filepath
LIMIT 100;
" > /tmp/files_to_convert.txt

count=0
total=$(wc -l < /tmp/files_to_convert.txt)

echo "Converting $total files..."

while IFS='|' read -r filepath bpm filename; do
    # Trim whitespace
    filepath=$(echo "$filepath" | xargs)
    bpm=$(echo "$bpm" | xargs)
    filename=$(echo "$filename" | xargs)

    # Skip empty lines
    [ -z "$filepath" ] && continue

    # Create output filename with BPM prefix
    bpm_int=$(printf "%.0f" "$bpm")
    basename="${filename%.mid}"
    output_name="${bpm_int}bpm-${basename}.mpcpattern"
    output_path="${OUTPUT_DIR}/${output_name}"

    # Convert
    if [ -f "$filepath" ]; then
        $CONVERTER "$filepath" "$output_path" 2>/dev/null
        if [ $? -eq 0 ]; then
            count=$((count + 1))
            echo "[$count/$total] Converted: $output_name"
        else
            echo "[$count/$total] Failed: $filepath"
        fi
    fi
done < /tmp/files_to_convert.txt

echo ""
echo "✓ Conversion complete!"
echo "  Converted: $count files"
echo "  Location: $OUTPUT_DIR"

#!/bin/bash
set -e

# Create 97 Instrument-Specific Expansion Folders
# One expansion per instrument found in database

EXPANSIONS_DIR="${1:-/media/dojevou/NewSSD2/Expansions}"
INSTRUMENTS_FILE="INSTRUMENT_LIST.txt"

echo "🎸 Creating 97 Instrument-Specific MPC Expansion Packs"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📁 Target directory: $EXPANSIONS_DIR"
echo "📋 Reading instruments from: $INSTRUMENTS_FILE"
echo ""

# Check if instruments file exists
if [ ! -f "$INSTRUMENTS_FILE" ]; then
    echo "❌ Error: $INSTRUMENTS_FILE not found"
    exit 1
fi

# Create base expansions directory
mkdir -p "$EXPANSIONS_DIR"

# Counter
created=0
failed=0

# Read each instrument and create expansion folder
while IFS=':' read -r instrument count; do
    # Clean up instrument name (trim whitespace)
    instrument=$(echo "$instrument" | xargs)
    count=$(echo "$count" | xargs)

    # Convert to uppercase and replace hyphens with underscores
    instrument_upper=$(echo "$instrument" | tr '[:lower:]' '[:upper:]' | tr '-' '_' | tr '&' 'AND')
    expansion_name="MIDI_${instrument_upper}"
    expansion_dir="$EXPANSIONS_DIR/$expansion_name"

    # Create folder structure
    mkdir -p "$expansion_dir/Patterns"
    mkdir -p "$expansion_dir/[Previews]"

    # Create Cache.json
    cat > "$expansion_dir/Cache.json" << EOF
{
  "name": "$expansion_name",
  "version": "1.0.0",
  "author": "MIDI Software Center",
  "description": "$count ${instrument} MIDI patterns",
  "category": "Instrument",
  "instrument": "$instrument",
  "pattern_count": $count,
  "created": "$(date -I)"
}
EOF

    # Create placeholder expansion image (if ImageMagick available)
    if command -v convert &> /dev/null; then
        convert -size 512x512 xc:black \
            -fill white -pointsize 48 -gravity center \
            -annotate +0-20 "MIDI" \
            -pointsize 72 -annotate +0+40 "${instrument_upper}" \
            -pointsize 24 -annotate +0+100 "${count} patterns" \
            "$expansion_dir/expansion-image.jpg" 2>/dev/null && \
            echo "✅ [$((created + 1))/97] $expansion_name ($count patterns)" || \
            echo "⚠️  [$((created + 1))/97] $expansion_name ($count patterns) - no image"
    else
        echo "✅ [$((created + 1))/97] $expansion_name ($count patterns)"
    fi

    created=$((created + 1))

done < "$INSTRUMENTS_FILE"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎉 EXPANSION FOLDERS CREATED"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Created: $created expansions"
echo "📁 Location: $EXPANSIONS_DIR"
echo ""
echo "Next steps:"
echo "  1. Build converter: ./scripts/build-ultra-fast-converter.sh"
echo "  2. Convert patterns: ./scripts/convert_all_97_instruments.sh"
echo "  3. Copy to MPC device"
echo ""

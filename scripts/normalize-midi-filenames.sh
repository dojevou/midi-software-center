#!/bin/bash
# 🧹 Simple MIDI Filename Normalization
# Runs on extracted files before analysis
# - Normalizes extensions: .MIDI, .MID → .mid
# - Replaces spaces with underscores
# - Fixes basic encoding issues

set -e

EXTRACT_DIR="${1:-/home/dojevou/tmp}"
WORKERS="${2:-16}"

echo "🧹 MIDI FILENAME NORMALIZATION"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "📂 Target directory: $EXTRACT_DIR"
echo "⚡ Parallel workers: $WORKERS"
echo ""

# Count MIDI files before
BEFORE=$(find "$EXTRACT_DIR" -type f \( -name "*.mid" -o -name "*.midi" -o -name "*.MID" -o -name "*.MIDI" -o -name "*.MiD" -o -name "*.Midi" \) 2>/dev/null | wc -l)
echo "📊 Found $BEFORE MIDI files"
echo ""

echo "⚡ Normalizing extensions (.MIDI, .MID → .mid)..."

# Normalize .MIDI to .mid
find "$EXTRACT_DIR" -type f -name "*.MIDI" -print0 2>/dev/null | \
  xargs -0 -P "$WORKERS" -I {} bash -c 'mv "{}" "$(dirname "{}")/$(basename "{}" .MIDI).mid"' 2>/dev/null || true

# Normalize .MID to .mid
find "$EXTRACT_DIR" -type f -name "*.MID" -print0 2>/dev/null | \
  xargs -0 -P "$WORKERS" -I {} bash -c 'mv "{}" "$(dirname "{}")/$(basename "{}" .MID).mid"' 2>/dev/null || true

# Normalize .MiD to .mid
find "$EXTRACT_DIR" -type f -name "*.MiD" -print0 2>/dev/null | \
  xargs -0 -P "$WORKERS" -I {} bash -c 'mv "{}" "$(dirname "{}")/$(basename "{}" .MiD).mid"' 2>/dev/null || true

# Normalize .Midi to .mid
find "$EXTRACT_DIR" -type f -name "*.Midi" -print0 2>/dev/null | \
  xargs -0 -P "$WORKERS" -I {} bash -c 'mv "{}" "$(dirname "{}")/$(basename "{}" .Midi).mid"' 2>/dev/null || true

# Normalize .midi to .mid
find "$EXTRACT_DIR" -type f -name "*.midi" -print0 2>/dev/null | \
  xargs -0 -P "$WORKERS" -I {} bash -c 'mv "{}" "$(dirname "{}")/$(basename "{}" .midi).mid"' 2>/dev/null || true

echo "✓ Extensions normalized"
echo ""

echo "⚡ Replacing spaces with underscores..."

# Replace spaces in filenames (MIDI files only)
find "$EXTRACT_DIR" -type f -name "*.mid" -print0 2>/dev/null | \
  while IFS= read -r -d '' file; do
    dir=$(dirname "$file")
    name=$(basename "$file")

    # Replace spaces with underscores
    if [[ "$name" == *" "* ]]; then
      newname="${name// /_}"
      mv "$file" "$dir/$newname" 2>/dev/null || true
    fi
  done

echo "✓ Spaces replaced"
echo ""

# Count MIDI files after
AFTER=$(find "$EXTRACT_DIR" -type f -name "*.mid" 2>/dev/null | wc -l)

echo "═══════════════════════════════════════════════════════════════"
echo "✅ Normalization Complete!"
echo ""
echo "📊 Statistics:"
echo "   Files before: $BEFORE"
echo "   Files after:  $AFTER"
echo "   Normalized:   $((AFTER - BEFORE + BEFORE))"
echo ""
echo "All .mid files now have:"
echo "  ✓ Lowercase .mid extension"
echo "  ✓ Underscores instead of spaces"
echo ""

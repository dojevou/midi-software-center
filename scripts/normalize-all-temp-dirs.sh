#!/bin/bash
# 🧹 Normalize All MIDI Files in /home/dojevou/tmp
# Ultra-fast batch normalization using Rust binary

set -e

NORMALIZE_BIN="${1:-./target/release/normalize_filenames}"
WORKERS="${2:-64}"
TMP_DIR="/home/dojevou/tmp"

echo "🧹 BATCH MIDI FILENAME NORMALIZATION"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "📂 Scanning directory: $TMP_DIR"
echo "⚡ Workers per directory: $WORKERS"
echo ""

# Find all midi_unified_* directories
DIRS=$(find "$TMP_DIR" -maxdepth 1 -type d -name "midi_unified_*" 2>/dev/null)
DIR_COUNT=$(echo "$DIRS" | grep -v '^$' | wc -l)

if [ "$DIR_COUNT" -eq 0 ]; then
  echo "❌ No midi_unified_* directories found in $TMP_DIR"
  exit 1
fi

echo "✓ Found $DIR_COUNT directories to normalize"
echo ""

# Process each directory
CURRENT=0
TOTAL_FILES=0
TOTAL_EXTENSIONS=0
TOTAL_SPACES=0
TOTAL_ENCODING=0
TOTAL_ERRORS=0

echo "$DIRS" | while read -r dir; do
  if [ -z "$dir" ]; then
    continue
  fi

  CURRENT=$((CURRENT + 1))
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "📦 Directory [$CURRENT/$DIR_COUNT]: $(basename "$dir")"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""

  # Run normalization
  "$NORMALIZE_BIN" "$dir" "$WORKERS"

  echo ""
done

echo "═══════════════════════════════════════════════════════════════"
echo "✅ BATCH NORMALIZATION COMPLETE!"
echo ""
echo "All MIDI files in $DIR_COUNT directories have been normalized"
echo ""

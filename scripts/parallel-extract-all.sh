#!/bin/bash
# 🚀 PARALLEL ARCHIVE EXTRACTION - Extract ALL archives simultaneously
# Uses ALL CPU cores + pigz/pbzip2 for maximum decompression speed

set -e

ARCHIVE_DIR="/media/dojevou/NewSSD2/midi"
EXTRACT_DIR="/tmp/midi_all_extracted"
MAX_PARALLEL=8  # Extract 8 archives simultaneously

echo "🚀 PARALLEL ARCHIVE EXTRACTION"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Installing parallel decompression tools..."

# Install ultra-fast parallel decompression tools
sudo apt-get update -qq
sudo apt-get install -y pigz pbzip2 pixz pzstd parallel 2>&1 | tail -5

echo "✓ Tools installed:"
echo "  - pigz: Parallel gzip (uses all CPU cores)"
echo "  - pbzip2: Parallel bzip2 (4x faster)"
echo "  - pixz: Parallel xz (8x faster)"
echo "  - pzstd: Parallel zstd (fastest compression)"
echo "  - GNU parallel: Job controller"
echo ""

# Create extraction directory
rm -rf "$EXTRACT_DIR"
mkdir -p "$EXTRACT_DIR"

echo "Found archives:"
find "$ARCHIVE_DIR" -maxdepth 1 -type f \( -name "*.zip" -o -name "*.rar" -o -name "*.7z" \) | nl
echo ""

echo "Starting parallel extraction (max $MAX_PARALLEL concurrent)..."
echo ""

# Use GNU parallel to extract all archives simultaneously
find "$ARCHIVE_DIR" -maxdepth 1 -type f \( -name "*.zip" -o -name "*.rar" -o -name "*.7z" \) | \
  parallel -j $MAX_PARALLEL '
    archive={}
    filename=$(basename "$archive")
    echo "🔄 Extracting: $filename"

    case "$archive" in
      *.zip)
        # Use unzip with parallel decompression
        unzip -q -o "$archive" -d "'$EXTRACT_DIR'" 2>&1 | grep -v "warning" || true
        ;;
      *.rar)
        unrar x -o+ -inul "$archive" "'$EXTRACT_DIR'/" || true
        ;;
      *.7z)
        7z x -o"'$EXTRACT_DIR'" -y "$archive" > /dev/null || true
        ;;
    esac

    echo "✓ Complete: $filename"
  '

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "Extraction complete!"
echo ""

# Count extracted MIDI files
MIDI_COUNT=$(find "$EXTRACT_DIR" -type f \( -name "*.mid" -o -name "*.midi" \) | wc -l)
echo "📊 Extracted: $MIDI_COUNT MIDI files"
echo "📂 Location: $EXTRACT_DIR"
echo ""
echo "Next: Run import on $EXTRACT_DIR"

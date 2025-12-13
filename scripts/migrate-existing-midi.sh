#!/bin/bash
# Migrate existing MIDI files to project library structure
# Moves files from /home/dojevou/Uncontaminated/floorp_downloads/midi/ to midi-library/

set -e

SOURCE_DIR="/home/dojevou/Uncontaminated/floorp_downloads/midi"
PROJECT_DIR="/home/dojevou/projects/midi-software-center"
LIBRARY_DIR="$PROJECT_DIR/midi-library"
IMPORTED_DIR="$LIBRARY_DIR/imported"
ARCHIVES_DIR="$LIBRARY_DIR/archives"

echo "=== MIDI Library Migration ==="
echo ""
echo "Source:      $SOURCE_DIR"
echo "Destination: $LIBRARY_DIR"
echo ""

# Check source directory exists
if [ ! -d "$SOURCE_DIR" ]; then
    echo "ERROR: Source directory does not exist: $SOURCE_DIR"
    exit 1
fi

# Create destination directories
echo "Creating directory structure..."
mkdir -p "$IMPORTED_DIR"
mkdir -p "$ARCHIVES_DIR"
mkdir -p "$LIBRARY_DIR/temp"
mkdir -p "$LIBRARY_DIR/organized"

# Count files
MIDI_COUNT=$(find "$SOURCE_DIR" -type f \( -iname "*.mid" -o -iname "*.midi" \) 2>/dev/null | wc -l)
ARCHIVE_COUNT=$(find "$SOURCE_DIR" -type f \( -iname "*.zip" -o -iname "*.rar" -o -iname "*.7z" \) 2>/dev/null | wc -l)

echo ""
echo "Found:"
echo "  - $MIDI_COUNT MIDI files (.mid, .midi)"
echo "  - $ARCHIVE_COUNT archive files (.zip, .rar, .7z)"
echo ""

# Ask for confirmation
read -p "Proceed with migration? (y/n) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Migration cancelled."
    exit 0
fi

echo ""
echo "Step 1: Moving MIDI files to imported/ ..."
MOVED_MIDI=0
find "$SOURCE_DIR" -type f \( -iname "*.mid" -o -iname "*.midi" \) -print0 | while IFS= read -r -d '' file; do
    filename=$(basename "$file")
    # Check if file already exists
    if [ -f "$IMPORTED_DIR/$filename" ]; then
        echo "  SKIP (exists): $filename"
    else
        mv "$file" "$IMPORTED_DIR/"
        echo "  MOVED: $filename"
        ((MOVED_MIDI++)) || true
    fi
done

echo ""
echo "Step 2: Moving archive files to archives/ ..."
MOVED_ARCHIVES=0
find "$SOURCE_DIR" -type f \( -iname "*.zip" -o -iname "*.rar" -o -iname "*.7z" \) -print0 | while IFS= read -r -d '' file; do
    filename=$(basename "$file")
    if [ -f "$ARCHIVES_DIR/$filename" ]; then
        echo "  SKIP (exists): $filename"
    else
        mv "$file" "$ARCHIVES_DIR/"
        echo "  MOVED: $filename"
        ((MOVED_ARCHIVES++)) || true
    fi
done

echo ""
echo "=== Migration Complete ==="
echo ""
echo "Files moved to:"
echo "  - MIDI files: $IMPORTED_DIR/"
echo "  - Archives:   $ARCHIVES_DIR/"
echo ""
echo "Next steps:"
echo "  1. Run pipeline import: make dev-pipeline"
echo "  2. Import from GUI or use: scripts/import_and_analyze.sh"
echo "  3. Files will be organized into: $LIBRARY_DIR/organized/"
echo ""
echo "Note: Source directory preserved at: $SOURCE_DIR"
echo "      (Remove manually if migration successful)"

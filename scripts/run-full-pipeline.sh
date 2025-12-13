#!/bin/bash
# Run complete MIDI pipeline on all files
# Phase 0: Sanitize → Phase 1: Organize → Phase 2: Import → Phase 3: Analyze

set -e

SOURCE_DIR="/home/dojevou/Uncontaminated/floorp_downloads/midi"
PROJECT_DIR="/home/dojevou/projects/midi-software-center"
LIBRARY_DIR="$PROJECT_DIR/midi-library"

echo "╔════════════════════════════════════════════════════════════════════╗"
echo "║           MIDI Software Center - Full Pipeline Execution          ║"
echo "╚════════════════════════════════════════════════════════════════════╝"
echo ""
echo "Source:  $SOURCE_DIR"
echo "Library: $LIBRARY_DIR"
echo ""

# Ensure database is running
echo "Step 0: Checking database..."
if ! docker ps | grep -q postgres; then
    echo "  Starting PostgreSQL..."
    cd "$PROJECT_DIR"
    make docker-up
    sleep 5
fi
echo "  ✓ Database running"
echo ""

# Create directory structure
echo "Step 1: Creating library structure..."
mkdir -p "$LIBRARY_DIR"/{imported,organized,archives,temp}
echo "  ✓ Directories created"
echo ""

# Count files first
echo "Step 2: Scanning source directory..."
echo "  Counting files (this may take a moment)..."
MIDI_COUNT=$(find "$SOURCE_DIR" -type f \( -iname "*.mid" -o -iname "*.midi" \) 2>/dev/null | wc -l || echo "0")
ARCHIVE_COUNT=$(find "$SOURCE_DIR" -type f \( -iname "*.zip" -o -iname "*.rar" -o -iname "*.7z" \) 2>/dev/null | wc -l || echo "0")

echo ""
echo "  Found:"
echo "    • $MIDI_COUNT MIDI files (.mid, .midi)"
echo "    • $ARCHIVE_COUNT archives (.zip, .rar, .7z)"
echo ""

if [ "$MIDI_COUNT" -eq 0 ] && [ "$ARCHIVE_COUNT" -eq 0 ]; then
    echo "ERROR: No MIDI files or archives found in $SOURCE_DIR"
    exit 1
fi

# Confirm
read -p "Continue with full pipeline? (y/n) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Pipeline cancelled."
    exit 0
fi

echo ""
echo "════════════════════════════════════════════════════════════════════"
echo "PHASE 0: SANITIZATION (spaces→underscores, .midi→.mid)"
echo "════════════════════════════════════════════════════════════════════"
echo ""

# Use CLI import tool for sanitization and import
cd "$PROJECT_DIR/pipeline/src-tauri"

# Build the pipeline binary if not exists
if [ ! -f "../../target/release/midi_pipeline" ]; then
    echo "Building pipeline binary..."
    cargo build --release
fi

echo ""
echo "════════════════════════════════════════════════════════════════════"
echo "PHASE 1-2: IMPORT & DATABASE INSERT"
echo "════════════════════════════════════════════════════════════════════"
echo ""
echo "Importing all files to database..."
echo "  - Calculating BLAKE3 hashes"
echo "  - Deduplication checks"
echo "  - MIDI parsing"
echo "  - Filename metadata extraction"
echo ""

# Import using the pipeline command
# This will handle sanitization automatically
DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library" \
    cargo run --release --bin import -- \
    --input "$SOURCE_DIR" \
    --recursive \
    --threads 8

echo ""
echo "════════════════════════════════════════════════════════════════════"
echo "PHASE 3: ANALYSIS"
echo "════════════════════════════════════════════════════════════════════"
echo ""
echo "Running analysis on all imported files..."
echo "  - BPM detection (interval + onset based)"
echo "  - Key detection (Krumhansl-Schmuckler)"
echo "  - Chord analysis"
echo "  - Drum analysis (if applicable)"
echo "  - Time signature extraction"
echo ""

DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library" \
    cargo run --release --bin analyze -- \
    --batch-size 100 \
    --threads 8

echo ""
echo "════════════════════════════════════════════════════════════════════"
echo "PHASE 4: AUTO-TAGGING"
echo "════════════════════════════════════════════════════════════════════"
echo ""
echo "Generating tags for all files..."
echo "  - Category-based tags"
echo "  - Filename pattern tags"
echo "  - MIDI content tags"
echo "  - Musical characteristic tags"
echo ""

# Auto-tagging happens automatically during analysis
# Just verify tag counts
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" \
    -c "SELECT COUNT(DISTINCT tag_name) as total_tags FROM tags;" \
    -c "SELECT COUNT(*) as tagged_files FROM file_tags;"

echo ""
echo "════════════════════════════════════════════════════════════════════"
echo "PIPELINE COMPLETE ✓"
echo "════════════════════════════════════════════════════════════════════"
echo ""

# Get final statistics
echo "Final Statistics:"
echo ""

psql "postgresql://midiuser:145278963@localhost:5433/midi_library" <<EOF
SELECT
    COUNT(*) as total_files,
    COUNT(DISTINCT category) as categories,
    ROUND(AVG(file_size_bytes)::numeric, 2) as avg_size_bytes,
    SUM(file_size_bytes) as total_size_bytes
FROM files;

SELECT
    COUNT(*) as analyzed_files,
    ROUND(AVG(bpm)::numeric, 2) as avg_bpm,
    ROUND(AVG(duration_seconds)::numeric, 2) as avg_duration_sec
FROM file_metadata
WHERE bpm IS NOT NULL;

SELECT
    category,
    COUNT(*) as file_count
FROM files
WHERE category IS NOT NULL
GROUP BY category
ORDER BY file_count DESC
LIMIT 10;
EOF

echo ""
echo "════════════════════════════════════════════════════════════════════"
echo "Next Steps:"
echo "  1. Launch GUI: make dev-pipeline"
echo "  2. Browse library at http://localhost:5173"
echo "  3. Search, filter, and explore your MIDI collection"
echo "  4. Open files in DAW: make dev-daw"
echo ""
echo "Files are stored at:"
echo "  • Database: PostgreSQL container"
echo "  • Original location: $SOURCE_DIR"
echo "  • Library path stored in DB"
echo "════════════════════════════════════════════════════════════════════"

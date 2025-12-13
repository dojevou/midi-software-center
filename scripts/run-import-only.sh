#!/bin/bash
# Import MIDI files to database (corrected script)
# Uses batch_import binary with correct arguments

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Paths
PROJECT_DIR="/home/dojevou/projects/midi-software-center"
SOURCE_DIR="$PROJECT_DIR/midi-library"
DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  MIDI Library - File Import (Corrected)${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Step 1: Check source directory
echo -e "${YELLOW}[1/5] Checking source directory...${NC}"
if [ ! -d "$SOURCE_DIR" ]; then
    echo -e "${RED}✗ Source directory not found: $SOURCE_DIR${NC}"
    echo "Available directories in midi-library:"
    ls -la "$PROJECT_DIR/midi-library/"
    exit 1
fi

FILE_COUNT=$(find "$SOURCE_DIR" -type f \( -name "*.mid" -o -name "*.midi" \) | wc -l)
echo -e "${GREEN}✓ Found $FILE_COUNT MIDI files${NC}"
echo ""

if [ "$FILE_COUNT" -eq 0 ]; then
    echo -e "${RED}✗ No MIDI files found in $SOURCE_DIR${NC}"
    exit 1
fi

# Step 2: Check database connection
echo -e "${YELLOW}[2/5] Checking database connection...${NC}"
if ! psql "$DB_URL" -c "SELECT 1" > /dev/null 2>&1; then
    echo -e "${RED}✗ Cannot connect to database${NC}"
    echo "Please start database:"
    echo "  cd $PROJECT_DIR && make docker-up"
    exit 1
fi
echo -e "${GREEN}✓ Database connected${NC}"
echo ""

# Step 3: Build binary if needed
echo -e "${YELLOW}[3/5] Checking binaries...${NC}"
cd "$PROJECT_DIR/pipeline/src-tauri"

if [ ! -f "../../target/release/batch_import" ]; then
    echo "  Compiling batch_import binary..."
    RUSTFLAGS="-C target-cpu=native -C opt-level=3" \
        cargo build --release --bin batch_import
fi
echo -e "${GREEN}✓ Binary ready${NC}"
echo ""

# Step 4: Import files
echo -e "${YELLOW}[4/5] Importing files...${NC}"
echo "  Source: $SOURCE_DIR"
echo "  Database: $DB_URL"
echo "  Workers: 16 threads"
echo "  Batch size: 1000"
echo ""

START_TIME=$(date +%s)

DATABASE_URL="$DB_URL" \
    ../../target/release/batch_import \
    --directory "$SOURCE_DIR" \
    --workers 16 \
    2>&1 | tee /tmp/import_log.txt

IMPORT_END=$(date +%s)
IMPORT_DURATION=$((IMPORT_END - START_TIME))

echo ""
echo -e "${GREEN}✓ Import complete!${NC}"
echo "  Duration: $IMPORT_DURATION seconds (~$((IMPORT_DURATION / 60)) minutes)"
echo ""

# Step 5: Verify results
echo -e "${YELLOW}[5/5] Verifying import results...${NC}"

IMPORTED_COUNT=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM files;" | xargs)
METADATA_COUNT=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM musical_metadata;" | xargs)
ANALYZED_COUNT=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM musical_metadata WHERE bpm IS NOT NULL;" | xargs)

echo "  Files imported: $IMPORTED_COUNT"
echo "  Metadata records: $METADATA_COUNT"
echo "  Analyzed files: $ANALYZED_COUNT"
echo ""

if [ "$IMPORTED_COUNT" -gt 0 ]; then
    IMPORT_RATE=$(echo "scale=2; $IMPORTED_COUNT / $IMPORT_DURATION" | bc)
    echo -e "${GREEN}✓ Import successful - $IMPORT_RATE files/sec${NC}"
    echo ""
    echo "Next step:"
    echo "  ./scripts/organize-database.sh"
else
    echo -e "${RED}✗ No files were imported${NC}"
    echo "Check /tmp/import_log.txt for errors"
    exit 1
fi

echo ""

#!/bin/bash
# Test Complete Pipeline on Sample Files
# Order: Import → Sanitize → Analyze → Split
# Then verify parent-child relationships

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Paths
PROJECT_DIR="/home/dojevou/projects/midi-software-center"
TEST_DIR="$PROJECT_DIR/midi-library/test-sample"
SPLITS_DIR="$PROJECT_DIR/midi-library/test-splits"
DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  PIPELINE TEST - Complete 4-Phase Test on Sample Files${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Step 0: Create test sample directory with 20 multi-track files
echo -e "${YELLOW}[0/5] Preparing test sample...${NC}"
rm -rf "$TEST_DIR" "$SPLITS_DIR"
mkdir -p "$TEST_DIR" "$SPLITS_DIR"

# Find 20 multi-track MIDI files from archives
echo "  Finding 20 multi-track files for testing..."
find "$PROJECT_DIR/midi-library/archives" -type f -name "*.mid" | head -20 | while read file; do
    cp "$file" "$TEST_DIR/"
done

TEST_COUNT=$(ls -1 "$TEST_DIR"/*.mid 2>/dev/null | wc -l)
echo -e "${GREEN}✓ Copied $TEST_COUNT files to test directory${NC}"
echo ""

if [ "$TEST_COUNT" -eq 0 ]; then
    echo -e "${RED}✗ No test files found${NC}"
    exit 1
fi

# Step 1: Clean database and import test files
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  PHASE 1: Import Test Files${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo -e "${YELLOW}[1/5] Cleaning database...${NC}"
psql "$DB_URL" -c "TRUNCATE files, file_tags, musical_metadata, track_splits CASCADE;" > /dev/null 2>&1
echo -e "${GREEN}✓ Database cleaned${NC}"
echo ""

echo "Importing $TEST_COUNT test files..."
cd "$PROJECT_DIR/pipeline/src-tauri"
DATABASE_URL="$DB_URL" \
    ../../target/release/batch_import \
    --directory "$TEST_DIR" \
    --workers 4 \
    2>&1 | grep -E "Processing:|COMPLETE|imported:"

IMPORTED=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM files;" | xargs)
echo ""
echo -e "${GREEN}✓ Phase 1 Complete: $IMPORTED files imported${NC}"
echo ""

# Step 2: Sanitize filenames
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  PHASE 2: Filename Sanitization${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo -e "${YELLOW}[2/5] Sanitizing filenames...${NC}"
DATABASE_URL="$DB_URL" \
    ../../target/release/normalize_filenames \
    --directory "$TEST_DIR" \
    2>&1 | tail -10

echo -e "${GREEN}✓ Phase 2 Complete: Filenames sanitized${NC}"
echo ""

# Step 3: Deep Analysis
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  PHASE 3: Deep Musical Analysis${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo -e "${YELLOW}[3/5] Running deep analysis...${NC}"
echo "  • BPM, Key, Chords"
echo "  • Drum patterns"
echo "  • Note complexity"
echo ""

DATABASE_URL="$DB_URL" \
    ../../target/release/analyze \
    2>&1 | grep -E "Found|analyzed|Analysis|COMPLETE"

ANALYZED=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM files WHERE analyzed_at IS NOT NULL;" | xargs)
echo ""
echo -e "${GREEN}✓ Phase 3 Complete: $ANALYZED files analyzed${NC}"
echo ""

# Step 4: Track Splitting with Parent-Child Tracking
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  PHASE 4: Track Splitting (TEST MODE)${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo -e "${YELLOW}[4/5] Splitting multi-track files...${NC}"
echo "  Output: $SPLITS_DIR"
echo ""

# Check how many multi-track files we have
MULTI_TRACK=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM files WHERE num_tracks > 1;" | xargs)
echo "  Multi-track files found: $MULTI_TRACK"
echo ""

if [ "$MULTI_TRACK" -gt 0 ]; then
    DATABASE_URL="$DB_URL" \
        ../../target/release/batch_split_optimized \
        --output-dir "$SPLITS_DIR" \
        --workers 4 \
        --test-limit 20 \
        2>&1 | grep -E "Files|Tracks|Split|COMPLETE|Statistics"

    SPLITS_CREATED=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM track_splits;" | xargs)
    echo ""
    echo -e "${GREEN}✓ Phase 4 Complete: $SPLITS_CREATED track splits created${NC}"
else
    echo -e "${YELLOW}⚠️  No multi-track files in test sample${NC}"
    SPLITS_CREATED=0
fi
echo ""

# Step 5: VERIFY Parent-Child Relationships (NO ORPHANS!)
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  VERIFICATION: Parent-Child Relationships${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo -e "${YELLOW}[5/5] Verifying data integrity...${NC}"
echo ""

# Test 1: All split files must have parent
echo "Test 1: Checking for orphaned split files..."
ORPHANED_SPLITS=$(psql "$DB_URL" -t -c "
    SELECT COUNT(*)
    FROM track_splits ts
    WHERE NOT EXISTS (
        SELECT 1 FROM files f WHERE f.id = ts.parent_file_id
    );
" | xargs)

if [ "$ORPHANED_SPLITS" -eq 0 ]; then
    echo -e "  ${GREEN}✓ No orphaned split files (all have valid parent)${NC}"
else
    echo -e "  ${RED}✗ Found $ORPHANED_SPLITS orphaned split files!${NC}"
fi

# Test 2: All split file IDs must exist in files table
echo "Test 2: Checking split file IDs exist in files table..."
MISSING_FILES=$(psql "$DB_URL" -t -c "
    SELECT COUNT(*)
    FROM track_splits ts
    WHERE NOT EXISTS (
        SELECT 1 FROM files f WHERE f.id = ts.split_file_id
    );
" | xargs)

if [ "$MISSING_FILES" -eq 0 ]; then
    echo -e "  ${GREEN}✓ All split file IDs exist in files table${NC}"
else
    echo -e "  ${RED}✗ Found $MISSING_FILES split records with missing file IDs!${NC}"
fi

# Test 3: No duplicate parent-child pairs
echo "Test 3: Checking for duplicate parent-child relationships..."
DUPLICATES=$(psql "$DB_URL" -t -c "
    SELECT COUNT(*) FROM (
        SELECT parent_file_id, split_file_id, COUNT(*) as cnt
        FROM track_splits
        GROUP BY parent_file_id, split_file_id
        HAVING COUNT(*) > 1
    ) as dupes;
" | xargs)

if [ "$DUPLICATES" -eq 0 ]; then
    echo -e "  ${GREEN}✓ No duplicate parent-child pairs${NC}"
else
    echo -e "  ${RED}✗ Found $DUPLICATES duplicate relationships!${NC}"
fi

# Test 4: Show sample parent-child relationships
echo ""
echo "Sample parent-child relationships:"
psql "$DB_URL" -c "
    SELECT
        pf.filename as parent_file,
        ts.track_number,
        ts.track_name,
        ts.instrument,
        sf.filename as split_file,
        ts.note_count
    FROM track_splits ts
    JOIN files pf ON ts.parent_file_id = pf.id
    JOIN files sf ON ts.split_file_id = sf.id
    LIMIT 10;
"

# Final statistics
echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  TEST RESULTS SUMMARY${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

psql "$DB_URL" <<EOF
SELECT '=== File Statistics ===' as section;
SELECT 'Test files imported:     ' || COUNT(*)::text FROM files;
SELECT 'Files analyzed:          ' || COUNT(*)::text FROM files WHERE analyzed_at IS NOT NULL;
SELECT 'Multi-track files:       ' || COUNT(*)::text FROM files WHERE num_tracks > 1;
SELECT 'Total track splits:      ' || COUNT(*)::text FROM track_splits;
SELECT 'Unique parent files:     ' || COUNT(DISTINCT parent_file_id)::text FROM track_splits;
SELECT 'Unique split files:      ' || COUNT(DISTINCT split_file_id)::text FROM track_splits;

SELECT '';
SELECT '=== Integrity Checks ===' as section;
SELECT 'Orphaned splits:         ' || $ORPHANED_SPLITS::text as check;
SELECT 'Missing file IDs:        ' || $MISSING_FILES::text as check;
SELECT 'Duplicate relationships: ' || $DUPLICATES::text as check;
EOF

echo ""
if [ "$ORPHANED_SPLITS" -eq 0 ] && [ "$MISSING_FILES" -eq 0 ] && [ "$DUPLICATES" -eq 0 ]; then
    echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}  ✓✓✓ ALL TESTS PASSED! Pipeline is working correctly.${NC}"
    echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo "Ready to run full pipeline on 1.7M files!"
    echo "  ./scripts/run-complete-pipeline.sh"
else
    echo -e "${RED}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${RED}  ✗✗✗ TESTS FAILED! Fix issues before running full pipeline.${NC}"
    echo -e "${RED}═══════════════════════════════════════════════════════════════${NC}"
    exit 1
fi
echo ""

#!/bin/bash
# Test Phase 4: Track Splitting on 50 Multi-Track Files
# Verify parent-child relationships work correctly

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Paths
PROJECT_DIR="/home/dojevou/projects/midi-software-center"
SPLITS_DIR="$PROJECT_DIR/midi-library/test-splits-phase4"
DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  PHASE 4 TEST: Track Splitting on 50 Multi-Track Files${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Check prerequisites
echo -e "${YELLOW}[1/3] Checking prerequisites...${NC}"

FILE_COUNT=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM files;" | xargs)
ANALYZED=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM files WHERE analyzed_at IS NOT NULL;" | xargs)
MULTI_TRACK=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM files WHERE num_tracks > 1;" | xargs)

echo "  Total files: $FILE_COUNT"
echo "  Analyzed files: $ANALYZED"
echo "  Multi-track files: $MULTI_TRACK"
echo ""

if [ "$ANALYZED" -lt "$FILE_COUNT" ]; then
    echo -e "${RED}✗ Not all files analyzed. Run phases 1-3 first.${NC}"
    exit 1
fi

if [ "$MULTI_TRACK" -eq 0 ]; then
    echo -e "${RED}✗ No multi-track files found in database.${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Prerequisites met${NC}"
echo ""

# Create test splits directory
rm -rf "$SPLITS_DIR"
mkdir -p "$SPLITS_DIR"

# Run splitting on 50 files (test mode)
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Running Track Splitting Test${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${YELLOW}[2/3] Splitting 50 multi-track files...${NC}"
echo "  Output: $SPLITS_DIR"
echo "  Workers: 4 (test mode)"
echo ""

cd "$PROJECT_DIR/pipeline/src-tauri"

TEST_START=$(date +%s)

DATABASE_URL="$DB_URL" \
    ../../target/release/batch_split_optimized \
    --output-dir "$SPLITS_DIR" \
    --workers 4 \
    --test-limit 50 \
    2>&1 | tee /tmp/split_test_log.txt

TEST_END=$(date +%s)
TEST_DURATION=$((TEST_END - TEST_START))

echo ""
echo -e "${GREEN}✓ Test splitting complete (${TEST_DURATION}s)${NC}"
echo ""

# Verify parent-child relationships
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  VERIFICATION: Parent-Child Relationships${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${YELLOW}[3/3] Verifying data integrity...${NC}"
echo ""

# Test 1: All split files must have valid parent
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

# Test 4: All parent files are multi-track
echo "Test 4: Verifying parent files are multi-track..."
INVALID_PARENTS=$(psql "$DB_URL" -t -c "
    SELECT COUNT(DISTINCT ts.parent_file_id)
    FROM track_splits ts
    JOIN files f ON ts.parent_file_id = f.id
    WHERE f.num_tracks <= 1;
" | xargs)

if [ "$INVALID_PARENTS" -eq 0 ]; then
    echo -e "  ${GREEN}✓ All parent files are multi-track${NC}"
else
    echo -e "  ${RED}✗ Found $INVALID_PARENTS single-track parents!${NC}"
fi

# Show sample relationships
echo ""
echo "Sample parent-child relationships:"
psql "$DB_URL" -c "
    SELECT
        pf.filename as parent_file,
        pf.num_tracks as parent_tracks,
        ts.track_number,
        ts.track_name,
        ts.instrument,
        sf.filename as split_file,
        ts.note_count
    FROM track_splits ts
    JOIN files pf ON ts.parent_file_id = pf.id
    JOIN files sf ON ts.split_file_id = sf.id
    ORDER BY pf.filename, ts.track_number
    LIMIT 15;
"

# Statistics
echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  TEST RESULTS${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

psql "$DB_URL" <<EOF
SELECT '=== Split Statistics ===' as section;
SELECT 'Parent files split:      ' || COUNT(DISTINCT parent_file_id)::text FROM track_splits;
SELECT 'Total track splits:      ' || COUNT(*)::text FROM track_splits;
SELECT 'Unique split files:      ' || COUNT(DISTINCT split_file_id)::text FROM track_splits;
SELECT 'Avg tracks per parent:   ' || ROUND(AVG(cnt), 2)::text
FROM (SELECT parent_file_id, COUNT(*) as cnt FROM track_splits GROUP BY parent_file_id) t;

SELECT '';
SELECT '=== Integrity Checks ===' as section;
SELECT 'Orphaned splits:         ' || $ORPHANED_SPLITS::text as check;
SELECT 'Missing file IDs:        ' || $MISSING_FILES::text as check;
SELECT 'Duplicate relationships: ' || $DUPLICATES::text as check;
SELECT 'Invalid parents:         ' || $INVALID_PARENTS::text as check;
EOF

echo ""
if [ "$ORPHANED_SPLITS" -eq 0 ] && [ "$MISSING_FILES" -eq 0 ] && [ "$DUPLICATES" -eq 0 ] && [ "$INVALID_PARENTS" -eq 0 ]; then
    echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}  ✓✓✓ ALL TESTS PASSED! Phase 4 is working correctly.${NC}"
    echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo "Ready to run full Phase 4 on all $MULTI_TRACK multi-track files!"
    echo "  ./scripts/run-phase4-splitting.sh"
else
    echo -e "${RED}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${RED}  ✗✗✗ TESTS FAILED! Fix issues before running full Phase 4.${NC}"
    echo -e "${RED}═══════════════════════════════════════════════════════════════${NC}"
    exit 1
fi
echo ""

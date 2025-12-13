#!/bin/bash
# Run Phases 1-3 Only (Import → Organization → Sanitization → Analysis)
# Phase 4 (Splitting) will be tested and run separately

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Paths
PROJECT_DIR="/home/dojevou/projects/midi-software-center"
DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  MIDI Pipeline - Phases 1-3 (Import, Org, Sanitize, Analyze)${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Verify import completed
echo -e "${YELLOW}[1/3] Verifying import status...${NC}"
FILE_COUNT=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM files;" | xargs)
echo "  Files in database: $FILE_COUNT"

if [ "$FILE_COUNT" -lt 100000 ]; then
    echo -e "${YELLOW}⚠️  Import still running or incomplete ($FILE_COUNT files)${NC}"
    echo "  Waiting for import to complete..."

    # Wait for import to finish
    while ps aux | grep -q "[b]atch_import"; do
        sleep 30
        NEW_COUNT=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM files;" | xargs)
        echo "  Progress: $NEW_COUNT files imported..."
    done

    FILE_COUNT=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM files;" | xargs)
    echo -e "${GREEN}✓ Import complete: $FILE_COUNT files${NC}"
fi
echo ""

# Phase 1: Database Organization (OPTIMIZED)
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  PHASE 1: OPTIMIZED Database Organization (97 Instrument Tags)${NC}"
echo -e "${BLUE}  Single scan, parallel processing - 97x faster${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${YELLOW}[1/3] Applying OPTIMIZED instrument organization...${NC}"
echo "  Estimated time: 2-5 minutes (vs. 32+ hours sequential)"
echo ""

cd "$PROJECT_DIR"

ORG_START=$(date +%s)

# Run organize-database.sh with auto-confirm
echo "y" | ./scripts/organize-database.sh || {
    echo -e "${RED}✗ Organization failed${NC}"
    exit 1
}

ORG_END=$(date +%s)
ORG_DURATION=$((ORG_END - ORG_START))

echo ""
echo -e "${GREEN}✓ Organization complete!${NC}"
echo "  Duration: ${ORG_DURATION}s (~$((ORG_DURATION / 60)) minutes)"
echo ""

# Phase 2: Deep Analysis (MOVED UP - doesn't need sanitized filenames)
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  PHASE 2: Deep Musical Analysis${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${YELLOW}[2/3] Running deep analysis...${NC}"
echo "  • Chord progressions"
echo "  • Drum pattern detection"
echo "  • Enhanced harmonic analysis"
echo "  • Complexity scoring"
echo ""
echo "  This may take 30-60 minutes for $FILE_COUNT files..."
echo ""

ANALYSIS_START=$(date +%s)

DATABASE_URL="$DB_URL" \
    ../../target/release/analyze \
    2>&1 | tee /tmp/analysis_log.txt

ANALYSIS_END=$(date +%s)
ANALYSIS_DURATION=$((ANALYSIS_END - ANALYSIS_START))
ANALYSIS_RATE=$(echo "scale=2; $FILE_COUNT / $ANALYSIS_DURATION" | bc)

echo ""
echo -e "${GREEN}✓ Analysis complete!${NC}"
echo "  Duration: ${ANALYSIS_DURATION}s (~$((ANALYSIS_DURATION / 60)) minutes)"
echo "  Rate: $ANALYSIS_RATE files/sec"
echo ""

# Phase 3: Filename Sanitization - DISABLED (CRITICAL BUG)
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  PHASE 3: Filename Sanitization - SKIPPED${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${RED}⚠️  CRITICAL BUG DETECTED:${NC}"
echo "  normalize_filenames does NOT update database filepath/filename!"
echo "  This would cause complete database corruption."
echo ""
echo -e "${YELLOW}  Sanitization disabled until fixed version is created.${NC}"
echo "  See: Fix normalize_filenames to update DB when renaming files"
echo ""

# Summary
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  PHASES 1-3 COMPLETE${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

psql "$DB_URL" <<EOF
SELECT '=== File Statistics ===' as section;
SELECT 'Total Files:       ' || COUNT(*)::text FROM files;
SELECT 'With Metadata:     ' || COUNT(*)::text FROM musical_metadata;
SELECT 'With BPM:          ' || COUNT(*)::text FROM musical_metadata WHERE bpm IS NOT NULL;
SELECT 'With Key:          ' || COUNT(*)::text FROM musical_metadata WHERE key_signature IS NOT NULL;
SELECT 'Analyzed:          ' || COUNT(*)::text FROM files WHERE analyzed_at IS NOT NULL;

SELECT '';
SELECT '=== Tag Statistics ===' as section;
SELECT 'Total Tags:        ' || COUNT(*)::text FROM tags;
SELECT 'File-Tag Links:    ' || COUNT(*)::text FROM file_tags;

SELECT '';
SELECT '=== Top 10 Instruments ===' as section;
SELECT
    name || ': ' || file_count::text || ' files (' || percentage::text || '%)'
FROM v_tag_stats
WHERE category IS NOT NULL
ORDER BY file_count DESC
LIMIT 10;
EOF

echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  ✓ Phases 1-3 Complete! Ready for Phase 4 testing.${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo "Next steps:"
echo "  1. Test Phase 4 (splitting): ./scripts/test-phase4-splitting.sh"
echo "  2. Run Phase 4 (full): ./scripts/run-phase4-splitting.sh"
echo ""

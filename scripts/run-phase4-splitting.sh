#!/bin/bash
# Run Phase 4: Track Splitting on ALL Multi-Track Files
# Only run after test-phase4-splitting.sh passes

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Paths
PROJECT_DIR="/home/dojevou/projects/midi-software-center"
SPLITS_DIR="$PROJECT_DIR/midi-library/splits"
DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  PHASE 4: Multi-Track Splitting (FULL RUN)${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Check prerequisites
MULTI_TRACK=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM files WHERE num_tracks > 1;" | xargs)
echo "  Multi-track files to process: $MULTI_TRACK"
echo ""

if [ "$MULTI_TRACK" -eq 0 ]; then
    echo -e "${YELLOW}⚠️  No multi-track files found, nothing to split${NC}"
    exit 0
fi

# Confirm before running
echo -e "${YELLOW}This will split $MULTI_TRACK multi-track files.${NC}"
echo "  Output directory: $SPLITS_DIR"
echo "  Estimated time: 15-30 minutes"
echo ""
read -p "Continue? (y/n) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Cancelled."
    exit 0
fi

# Create splits directory
mkdir -p "$SPLITS_DIR"

# Run full splitting
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Running Track Splitting${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo "  Workers: 48 parallel"
echo "  Batch size: 1000"
echo "  Features: Auto-repair, parent-child tracking, deduplication"
echo ""

cd "$PROJECT_DIR/pipeline/src-tauri"

SPLIT_START=$(date +%s)

DATABASE_URL="$DB_URL" \
    ../../target/release/batch_split_optimized \
    --output-dir "$SPLITS_DIR" \
    --workers 48 \
    --batch-size 1000 \
    2>&1 | tee /tmp/split_full_log.txt

SPLIT_END=$(date +%s)
SPLIT_DURATION=$((SPLIT_END - SPLIT_START))

# Get statistics
SPLITS_CREATED=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM track_splits;" | xargs)
SPLIT_FILES=$(psql "$DB_URL" -t -c "SELECT COUNT(DISTINCT split_file_id) FROM track_splits;" | xargs)
PARENTS_SPLIT=$(psql "$DB_URL" -t -c "SELECT COUNT(DISTINCT parent_file_id) FROM track_splits;" | xargs)

echo ""
echo -e "${GREEN}✓ Splitting complete!${NC}"
echo "  Duration: ${SPLIT_DURATION}s (~$((SPLIT_DURATION / 60)) minutes)"
echo "  Parents split: $PARENTS_SPLIT files"
echo "  Track splits created: $SPLITS_CREATED"
echo "  New split files: $SPLIT_FILES"
echo ""

# Verify parent-child relationships
echo "Verifying parent-child relationships..."
ORPHANS=$(psql "$DB_URL" -t -c "
    SELECT COUNT(*) FROM track_splits ts
    WHERE NOT EXISTS (SELECT 1 FROM files WHERE id = ts.parent_file_id)
       OR NOT EXISTS (SELECT 1 FROM files WHERE id = ts.split_file_id);
" | xargs)

if [ "$ORPHANS" -eq 0 ]; then
    echo -e "${GREEN}✓ All parent-child relationships valid (no orphans)${NC}"
else
    echo -e "${RED}✗ Found $ORPHANS orphaned relationships!${NC}"
fi
echo ""

# Final summary
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  PHASE 4 COMPLETE - Final Statistics${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

psql "$DB_URL" <<EOF
SELECT '=== Complete Pipeline Statistics ===' as section;

SELECT '';
SELECT '=== Files ===' as section;
SELECT 'Total imported files:    ' || COUNT(*)::text FROM files;
SELECT 'Multi-track parents:     ' || COUNT(*)::text FROM files WHERE num_tracks > 1;
SELECT 'Single-track files:      ' || COUNT(*)::text FROM files WHERE num_tracks = 1;

SELECT '';
SELECT '=== Track Splits ===' as section;
SELECT 'Parents split:           ' || COUNT(DISTINCT parent_file_id)::text FROM track_splits;
SELECT 'Total track splits:      ' || COUNT(*)::text FROM track_splits;
SELECT 'Split files created:     ' || COUNT(DISTINCT split_file_id)::text FROM track_splits;
SELECT 'Avg tracks per parent:   ' || ROUND(AVG(cnt), 2)::text
FROM (SELECT parent_file_id, COUNT(*) as cnt FROM track_splits GROUP BY parent_file_id) t;

SELECT '';
SELECT '=== Analysis ===' as section;
SELECT 'Files analyzed:          ' || COUNT(*)::text FROM files WHERE analyzed_at IS NOT NULL;
SELECT 'With BPM:                ' || COUNT(*)::text FROM musical_metadata WHERE bpm IS NOT NULL;
SELECT 'With Key:                ' || COUNT(*)::text FROM musical_metadata WHERE key_signature IS NOT NULL;

SELECT '';
SELECT '=== Organization ===' as section;
SELECT 'Instrument tags:         ' || COUNT(*)::text FROM tags;
SELECT 'File-tag relationships:  ' || COUNT(*)::text FROM file_tags;

SELECT '';
SELECT '=== Database Size ===' as section;
SELECT 'Total Size: ' || pg_size_pretty(pg_database_size('midi_library'));
EOF

echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  ✓✓✓ COMPLETE 4-PHASE PIPELINE FINISHED!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo "All phases complete:"
echo "  ✓ Phase 1: Import & Organization"
echo "  ✓ Phase 2: Filename Sanitization"
echo "  ✓ Phase 3: Deep Musical Analysis"
echo "  ✓ Phase 4: Multi-Track Splitting"
echo ""
echo "Ready for production use!"
echo ""

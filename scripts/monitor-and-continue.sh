#!/bin/bash
# Monitor Phase 1, then run deduplication and Phase 2

DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"
LOG_FILE="/tmp/import_direct_log.txt"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}📊 Monitoring Phase 1 Import...${NC}"
echo ""

# Wait for import to complete
while true; do
    # Check if process is still running
    if ! pgrep -f "batch_import.*archives" > /dev/null; then
        echo -e "${GREEN}✅ Phase 1 Complete!${NC}"
        break
    fi

    # Show last line of progress
    PROGRESS=$(tail -1 "$LOG_FILE" | grep "Processing:" || echo "Processing...")
    echo -ne "\r$PROGRESS"
    sleep 5
done

echo ""
echo ""

# Get final stats
TOTAL=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM files")
echo -e "${GREEN}Total files imported: $TOTAL${NC}"
echo ""

# Check for duplicates in database
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}DEDUPLICATION CHECK${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Check if blake3_hash column exists and has duplicates
DUP_COUNT=$(psql "$DB_URL" -t -c "
    SELECT COUNT(*)
    FROM (
        SELECT blake3_hash, COUNT(*) as cnt
        FROM files
        WHERE blake3_hash IS NOT NULL
        GROUP BY blake3_hash
        HAVING COUNT(*) > 1
    ) dups" 2>/dev/null || echo "0")

echo "Checking for duplicate BLAKE3 hashes..."
echo "Duplicates found: $DUP_COUNT"
echo ""

if [ "$DUP_COUNT" -gt "0" ]; then
    echo -e "${YELLOW}⚠️  Found $DUP_COUNT duplicate file groups${NC}"
    echo "Running filesystem deduplication..."
    echo ""

    cd ~/projects/midi-software-center
    export DATABASE_URL="$DB_URL"
    ./target/release/find_duplicates ~/projects/midi-software-center/midi-library/archives \
        2>&1 | tee /tmp/dedup_log.txt

    echo ""
    echo -e "${GREEN}✅ Deduplication complete${NC}"
else
    echo -e "${GREEN}✅ No duplicates found - import already deduplicated!${NC}"
fi

echo ""
echo ""

# Phase 2: Sanitize filenames
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}PHASE 2: SANITIZE FILENAMES${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Cleaning filenames: spaces→underscores, special chars removed"
echo "Log: /tmp/sanitize_log.txt"
echo ""

PHASE2_START=$(date +%s)

cd ~/projects/midi-software-center
export DATABASE_URL="$DB_URL"
./target/release/normalize_filenames 2>&1 | tee /tmp/sanitize_log.txt

PHASE2_END=$(date +%s)
PHASE2_DURATION=$((PHASE2_END - PHASE2_START))

echo ""
echo -e "${GREEN}✅ Phase 2 complete: $PHASE2_DURATION seconds${NC}"
echo ""

# Final stats
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}FINAL DATABASE STATS${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

psql "$DB_URL" <<EOF
SELECT '  Total files:      ' || COUNT(*)::text FROM files;
SELECT '  Unique hashes:    ' || COUNT(DISTINCT blake3_hash)::text FROM files WHERE blake3_hash IS NOT NULL;
SELECT '  Database size:    ' || pg_size_pretty(pg_database_size('midi_library'));
EOF

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}READY FOR PHASE 3 (Track Splitting) or PHASE 4 (Analysis)${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

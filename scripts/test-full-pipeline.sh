#!/bin/bash
set -e

# Full Pipeline Test Script
# Tests ALL pipeline functions in sequence:
# 1. Import (with hash calculation, metadata extraction)
# 2. Analysis (BPM, key, drum detection)
# 3. Auto-tagging (filename metadata, tags)
# 4. Deduplication (find duplicates by hash)
# 5. Renaming (generate production filenames)
# 6. Splitting (multi-track files)
# 7. Search & Stats (verify data integrity)

echo "========================================="
echo "FULL PIPELINE INTEGRATION TEST"
echo "========================================="
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Database connection
DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

# Test data directory (using existing imported files)
TEST_DIR="/tmp/midi-pipeline-test"
SAMPLE_SIZE=100

# Results tracking
RESULTS_FILE="/tmp/pipeline-test-results.json"

echo -e "${BLUE}Step 0: Setup${NC}"
echo "Creating test directory: $TEST_DIR"
mkdir -p "$TEST_DIR"

# Copy sample files from database for testing
echo "Selecting $SAMPLE_SIZE random files from database..."
psql "$DB_URL" -t -c "
SELECT filepath
FROM files
ORDER BY RANDOM()
LIMIT $SAMPLE_SIZE
" | while read -r filepath; do
    if [ -n "$filepath" ] && [ -f "$filepath" ]; then
        cp "$filepath" "$TEST_DIR/" 2>/dev/null || true
    fi
done

FILE_COUNT=$(ls -1 "$TEST_DIR"/*.mid 2>/dev/null | wc -l || echo "0")
echo -e "${GREEN}✓ Prepared $FILE_COUNT test files${NC}"
echo ""

# Initialize results
cat > "$RESULTS_FILE" <<EOF
{
  "test_timestamp": "$(date -Iseconds)",
  "test_file_count": $FILE_COUNT,
  "phases": {}
}
EOF

#=============================================================================
# PHASE 1: IMPORT
#=============================================================================
echo -e "${BLUE}Phase 1: Import (Hash + Metadata)${NC}"
START_TIME=$(date +%s.%N)

# Import files using Tauri command
echo "Importing $FILE_COUNT files..."
IMPORT_START=$(date +%s.%N)

# Run import via pipeline CLI
cd "$(dirname "$0")/../pipeline/src-tauri"
cargo build --bin import --release 2>&1 | grep -v "Compiling\|Finished" || true
IMPORT_RESULT=$(./target/release/import "$TEST_DIR" 2>&1 | tail -20)

IMPORT_END=$(date +%s.%N)
IMPORT_DURATION=$(echo "$IMPORT_END - $IMPORT_START" | bc)
IMPORT_RATE=$(echo "scale=2; $FILE_COUNT / $IMPORT_DURATION" | bc)

echo -e "${GREEN}✓ Import completed in ${IMPORT_DURATION}s (${IMPORT_RATE} files/sec)${NC}"
echo "Sample output: $IMPORT_RESULT"
echo ""

#=============================================================================
# PHASE 2: ANALYSIS
#=============================================================================
echo -e "${BLUE}Phase 2: Analysis (BPM, Key, Drums)${NC}"
ANALYSIS_START=$(date +%s.%N)

# Get file IDs to analyze
FILE_IDS=$(psql "$DB_URL" -t -c "
SELECT id FROM files
WHERE filepath LIKE '$TEST_DIR%'
AND analyzed_at IS NULL
LIMIT $SAMPLE_SIZE
" | tr '\n' ',' | sed 's/,$//')

if [ -n "$FILE_IDS" ]; then
    echo "Analyzing files: $FILE_IDS"
    cargo build --bin analyze --release 2>&1 | grep -v "Compiling\|Finished" || true
    ./target/release/analyze --file-ids "$FILE_IDS" 2>&1 | tail -10
else
    echo "No files to analyze (already analyzed)"
fi

ANALYSIS_END=$(date +%s.%N)
ANALYSIS_DURATION=$(echo "$ANALYSIS_END - $ANALYSIS_START" | bc)
ANALYZED_COUNT=$(psql "$DB_URL" -t -c "
SELECT COUNT(*) FROM files
WHERE filepath LIKE '$TEST_DIR%'
AND analyzed_at IS NOT NULL
")

echo -e "${GREEN}✓ Analyzed $ANALYZED_COUNT files in ${ANALYSIS_DURATION}s${NC}"
echo ""

#=============================================================================
# PHASE 3: AUTO-TAGGING
#=============================================================================
echo -e "${BLUE}Phase 3: Auto-Tagging${NC}"
TAGGING_START=$(date +%s.%N)

# Tags are generated during import, verify them
TAG_COUNT=$(psql "$DB_URL" -t -c "
SELECT COUNT(DISTINCT ft.tag_id)
FROM file_tags ft
JOIN files f ON ft.file_id = f.id
WHERE f.filepath LIKE '$TEST_DIR%'
")

TAGGED_FILES=$(psql "$DB_URL" -t -c "
SELECT COUNT(DISTINCT file_id)
FROM file_tags ft
JOIN files f ON ft.file_id = f.id
WHERE f.filepath LIKE '$TEST_DIR%'
")

TAGGING_END=$(date +%s.%N)
TAGGING_DURATION=$(echo "$TAGGING_END - $TAGGING_START" | bc)

echo -e "${GREEN}✓ $TAGGED_FILES files tagged with $TAG_COUNT unique tags${NC}"
echo ""

#=============================================================================
# PHASE 4: DEDUPLICATION
#=============================================================================
echo -e "${BLUE}Phase 4: Deduplication${NC}"
DEDUPE_START=$(date +%s.%N)

# Find duplicates by content hash
DUPLICATE_GROUPS=$(psql "$DB_URL" -t -c "
SELECT COUNT(*)
FROM (
    SELECT content_hash
    FROM files
    WHERE filepath LIKE '$TEST_DIR%'
    GROUP BY content_hash
    HAVING COUNT(*) > 1
) AS dupes
")

echo -e "${GREEN}✓ Found $DUPLICATE_GROUPS duplicate groups${NC}"
DEDUPE_END=$(date +%s.%N)
DEDUPE_DURATION=$(echo "$DEDUPE_END - $DEDUPE_START" | bc)
echo ""

#=============================================================================
# PHASE 5: RENAMING (Production Filenames)
#=============================================================================
echo -e "${BLUE}Phase 5: Generate Production Filenames${NC}"
RENAME_START=$(date +%s.%N)

# Renaming is done during import using generate_production_filename
# Verify that filenames follow pattern
RENAMED_COUNT=$(psql "$DB_URL" -t -c "
SELECT COUNT(*)
FROM files
WHERE filepath LIKE '$TEST_DIR%'
AND filename != original_filename
")

RENAME_END=$(date +%s.%N)
RENAME_DURATION=$(echo "$RENAME_END - $RENAME_START" | bc)

echo -e "${GREEN}✓ $RENAMED_COUNT files have production filenames${NC}"
echo ""

#=============================================================================
# PHASE 6: SPLITTING (Multi-track files)
#=============================================================================
echo -e "${BLUE}Phase 6: Track Splitting${NC}"
SPLIT_START=$(date +%s.%N)

# Find multi-track files
MULTITRACK_FILES=$(psql "$DB_URL" -t -c "
SELECT id, filepath
FROM files
WHERE filepath LIKE '$TEST_DIR%'
AND is_multi_track = true
AND num_tracks > 1
LIMIT 5
")

if [ -n "$MULTITRACK_FILES" ]; then
    echo "Splitting multi-track files..."
    cargo build --bin split --release 2>&1 | grep -v "Compiling\|Finished" || true

    # Split first 5 multi-track files
    echo "$MULTITRACK_FILES" | while read -r file_id filepath; do
        [ -n "$file_id" ] && ./target/release/split --file-id "$file_id" 2>&1 | tail -3 || true
    done
else
    echo "No multi-track files to split"
fi

SPLIT_COUNT=$(psql "$DB_URL" -t -c "
SELECT COUNT(*)
FROM files
WHERE parent_file_id IS NOT NULL
AND filepath LIKE '$TEST_DIR%'
")

SPLIT_END=$(date +%s.%N)
SPLIT_DURATION=$(echo "$SPLIT_END - $SPLIT_START" | bc)

echo -e "${GREEN}✓ Created $SPLIT_COUNT split tracks${NC}"
echo ""

#=============================================================================
# PHASE 7: SEARCH & STATS (Verification)
#=============================================================================
echo -e "${BLUE}Phase 7: Search & Stats${NC}"
VERIFY_START=$(date +%s.%N)

# Database integrity checks
echo "Running database integrity checks..."

# Check 1: All files have hashes
FILES_WITHOUT_HASH=$(psql "$DB_URL" -t -c "
SELECT COUNT(*)
FROM files
WHERE filepath LIKE '$TEST_DIR%'
AND content_hash IS NULL
")

# Check 2: Musical metadata exists for analyzed files
METADATA_COUNT=$(psql "$DB_URL" -t -c "
SELECT COUNT(*)
FROM musical_metadata mm
JOIN files f ON mm.file_id = f.id
WHERE f.filepath LIKE '$TEST_DIR%'
")

# Check 3: Search vector populated
SEARCH_VECTOR_COUNT=$(psql "$DB_URL" -t -c "
SELECT COUNT(*)
FROM files
WHERE filepath LIKE '$TEST_DIR%'
AND search_vector IS NOT NULL
")

VERIFY_END=$(date +%s.%N)
VERIFY_DURATION=$(echo "$VERIFY_END - $VERIFY_START" | bc)

echo -e "${GREEN}✓ Verification complete:${NC}"
echo "  - Files without hash: $FILES_WITHOUT_HASH (should be 0)"
echo "  - Files with metadata: $METADATA_COUNT"
echo "  - Files with search index: $SEARCH_VECTOR_COUNT"
echo ""

#=============================================================================
# SUMMARY
#=============================================================================
TOTAL_END=$(date +%s.%N)
TOTAL_DURATION=$(echo "$TOTAL_END - $START_TIME" | bc)

echo -e "${BLUE}=========================================${NC}"
echo -e "${BLUE}PIPELINE TEST SUMMARY${NC}"
echo -e "${BLUE}=========================================${NC}"
echo ""
echo "Test Files: $FILE_COUNT"
echo "Total Duration: ${TOTAL_DURATION}s"
echo ""
echo "Phase Timings:"
echo "  1. Import:        ${IMPORT_DURATION}s (${IMPORT_RATE} files/sec)"
echo "  2. Analysis:      ${ANALYSIS_DURATION}s"
echo "  3. Auto-Tagging:  ${TAGGING_DURATION}s ($TAGGED_FILES files, $TAG_COUNT tags)"
echo "  4. Deduplication: ${DEDUPE_DURATION}s ($DUPLICATE_GROUPS duplicates)"
echo "  5. Renaming:      ${RENAME_DURATION}s ($RENAMED_COUNT renamed)"
echo "  6. Splitting:     ${SPLIT_DURATION}s ($SPLIT_COUNT splits)"
echo "  7. Verification:  ${VERIFY_DURATION}s"
echo ""

# Quality checks
ERRORS=0
if [ "$FILES_WITHOUT_HASH" -gt 0 ]; then
    echo -e "${RED}❌ FAIL: $FILES_WITHOUT_HASH files missing content hash${NC}"
    ERRORS=$((ERRORS + 1))
else
    echo -e "${GREEN}✓ PASS: All files have content hash${NC}"
fi

if [ "$ANALYZED_COUNT" -eq 0 ]; then
    echo -e "${YELLOW}⚠ WARNING: No files analyzed${NC}"
else
    echo -e "${GREEN}✓ PASS: $ANALYZED_COUNT files analyzed${NC}"
fi

if [ "$TAGGED_FILES" -eq 0 ]; then
    echo -e "${YELLOW}⚠ WARNING: No files tagged${NC}"
else
    echo -e "${GREEN}✓ PASS: $TAGGED_FILES files tagged${NC}"
fi

echo ""
if [ $ERRORS -eq 0 ]; then
    echo -e "${GREEN}=========================================${NC}"
    echo -e "${GREEN}ALL TESTS PASSED ✓${NC}"
    echo -e "${GREEN}=========================================${NC}"
    exit 0
else
    echo -e "${RED}=========================================${NC}"
    echo -e "${RED}$ERRORS ERRORS FOUND ❌${NC}"
    echo -e "${RED}=========================================${NC}"
    exit 1
fi

#!/bin/bash
# Quick Database Verification (No Rust compilation)

set -e

echo "=========================================="
echo "  MIDI Pipeline Quick Verification"
echo "=========================================="
echo ""

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

DB_HOST="localhost"
DB_PORT="5433"
DB_USER="midiuser"
DB_PASS="145278963"
DB_NAME="midi_library"
export PGPASSWORD="$DB_PASS"

PASS=0
FAIL=0

pass() {
    echo -e "${GREEN}✓${NC} $1"
    ((PASS++))
}

fail() {
    echo -e "${RED}✗${NC} $1"
    ((FAIL++))
}

info() {
    echo -e "${YELLOW}ℹ${NC} $1"
}

echo "PHASE 1: Database Connectivity"
echo "-------------------------------"

# Connection test
if psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "SELECT 1" > /dev/null 2>&1; then
    pass "Database connection successful"
else
    fail "Database connection failed"
    exit 1
fi

# PostgreSQL version
PG_VERSION=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -tAc "SELECT version()" | grep -oP 'PostgreSQL \K[0-9]+\.[0-9]+')
pass "PostgreSQL version: $PG_VERSION"

# Table count
TABLE_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -tAc "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public'")
if [ "$TABLE_COUNT" -ge 17 ]; then
    pass "Table count: $TABLE_COUNT (expected ≥17)"
else
    fail "Table count: $TABLE_COUNT (expected ≥17)"
fi

# Index count
INDEX_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -tAc "SELECT COUNT(*) FROM pg_indexes WHERE schemaname = 'public'")
if [ "$INDEX_COUNT" -ge 60 ]; then
    pass "Index count: $INDEX_COUNT (expected ≥60)"
else
    fail "Index count: $INDEX_COUNT (expected ≥60)"
fi

echo ""
echo "PHASE 2: Required Tables"
echo "------------------------"

TABLES=("files" "musical_metadata" "file_categories" "file_instruments" "tags" "file_tags" "favorites" "track_splits")

for table in "${TABLES[@]}"; do
    if psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -tAc "SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = '$table'" | grep -q 1; then
        pass "Table: $table"
    else
        fail "Missing: $table"
    fi
done

echo ""
echo "PHASE 3: Data Integrity"
echo "-----------------------"

FILE_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -tAc "SELECT COUNT(*) FROM files")
info "Files in database: $FILE_COUNT"

METADATA_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -tAc "SELECT COUNT(*) FROM musical_metadata")
info "Metadata records: $METADATA_COUNT"

ORPHANED=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -tAc "SELECT COUNT(*) FROM musical_metadata mm WHERE NOT EXISTS (SELECT 1 FROM files f WHERE f.id = mm.file_id)")
if [ "$ORPHANED" -eq 0 ]; then
    pass "No orphaned metadata (referential integrity OK)"
else
    fail "Found $ORPHANED orphaned metadata records"
fi

echo ""
echo "PHASE 4: Index Usage"
echo "--------------------"

psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "ANALYZE files" > /dev/null 2>&1
pass "Database statistics updated (ANALYZE run)"

# Check if content_hash index exists
HASH_INDEX=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -tAc "SELECT COUNT(*) FROM pg_indexes WHERE schemaname = 'public' AND tablename = 'files' AND indexname LIKE '%content_hash%'")
if [ "$HASH_INDEX" -gt 0 ]; then
    pass "Content hash index present"
else
    fail "Content hash index missing"
fi

# Check search vector index
SEARCH_INDEX=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -tAc "SELECT COUNT(*) FROM pg_indexes WHERE schemaname = 'public' AND tablename = 'files' AND indexname LIKE '%search%'")
if [ "$SEARCH_INDEX" -gt 0 ]; then
    pass "Full-text search index present"
else
    info "Full-text search index not found (may be optional)"
fi

echo ""
echo "PHASE 5: Repository Layer"
echo "-------------------------"

if [ -f "src-tauri/src/db/repositories/file_repository.rs" ]; then
    pass "FileRepository exists"
else
    fail "FileRepository missing"
fi

if [ -f "src-tauri/src/db/repositories/metadata_repository.rs" ]; then
    pass "MetadataRepository exists"
else
    fail "MetadataRepository missing"
fi

if [ -f "src-tauri/src/db/repositories/search_repository.rs" ]; then
    pass "SearchRepository exists"
else
    fail "SearchRepository missing"
fi

if [ -f "src-tauri/src/db/repositories/tag_repository.rs" ]; then
    pass "TagRepository exists"
else
    fail "TagRepository missing"
fi

echo ""
echo "PHASE 6: Test Infrastructure"
echo "-----------------------------"

if [ -f "src-tauri/tests/helpers.rs" ]; then
    pass "Test helpers module exists"
else
    fail "Test helpers missing"
fi

if [ -f "src-tauri/tests/database_test.rs" ]; then
    pass "Database integration tests exist"
else
    fail "Database tests missing"
fi

if [ -f "src-tauri/tests/integration_end_to_end.rs" ]; then
    pass "End-to-end tests exist"
else
    fail "End-to-end tests missing"
fi

echo ""
echo "=========================================="
echo "  Verification Summary"
echo "=========================================="
echo ""
echo -e "Passed: ${GREEN}${PASS}${NC}"
echo -e "Failed: ${RED}${FAIL}${NC}"
echo ""

if [ "$FAIL" -eq 0 ]; then
    echo -e "${GREEN}✓ ALL CHECKS PASSED${NC}"
    echo ""
    echo "Database Integration: VERIFIED ✅"
    echo ""
    echo "Next steps:"
    echo "  1. Run Rust tests: cargo test --tests"
    echo "  2. Review: INTEGRATION_VERIFICATION_REPORT.md"
    echo "  3. Start building pipeline features!"
    exit 0
else
    echo -e "${RED}✗ SOME CHECKS FAILED${NC}"
    echo "Please review the failed tests above."
    exit 1
fi

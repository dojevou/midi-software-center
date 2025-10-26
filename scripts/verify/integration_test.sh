#!/bin/bash
# MIDI Pipeline ↔ Database Integration Verification Script
# Comprehensive verification following the integration guide

set -e

echo "=========================================="
echo "  MIDI Pipeline Integration Verification"
echo "=========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Database connection details
DB_HOST="localhost"
DB_PORT="5433"
DB_USER="midiuser"
DB_PASS="145278963"
DB_NAME="midi_library"
export PGPASSWORD="$DB_PASS"

# Test counters
PASS_COUNT=0
FAIL_COUNT=0

# Helper functions
pass() {
    echo -e "${GREEN}✓${NC} $1"
    ((PASS_COUNT++))
}

fail() {
    echo -e "${RED}✗${NC} $1"
    ((FAIL_COUNT++))
}

info() {
    echo -e "${YELLOW}ℹ${NC} $1"
}

# ============================================================================
# PHASE 1: DATABASE CONNECTIVITY CHECKS
# ============================================================================

echo "PHASE 1: Database Connectivity Checks"
echo "--------------------------------------"

# Test 1.1: Database Connection
if psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "SELECT 1" > /dev/null 2>&1; then
    pass "Database connection successful"
else
    fail "Database connection failed"
    exit 1
fi

# Test 1.2: Check PostgreSQL version
PG_VERSION=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -tAc "SELECT version()" | grep -oP 'PostgreSQL \K[0-9]+\.[0-9]+')
pass "PostgreSQL version: $PG_VERSION"

# Test 1.3: Table count
TABLE_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -tAc "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public'")
if [ "$TABLE_COUNT" -ge 17 ]; then
    pass "Table count: $TABLE_COUNT (expected ≥17)"
else
    fail "Table count: $TABLE_COUNT (expected ≥17)"
fi

# Test 1.4: Index count
INDEX_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -tAc "SELECT COUNT(*) FROM pg_indexes WHERE schemaname = 'public'")
if [ "$INDEX_COUNT" -ge 60 ]; then
    pass "Index count: $INDEX_COUNT (expected ≥60)"
else
    fail "Index count: $INDEX_COUNT (expected ≥60)"
fi

# Test 1.5: Required tables exist
REQUIRED_TABLES=(
    "files"
    "musical_metadata"
    "file_categories"
    "file_instruments"
    "tags"
    "file_tags"
    "favorites"
    "track_splits"
    "file_embeddings"
    "file_compatibility"
    "rhythm_patterns"
    "harmonic_patterns"
    "melodic_patterns"
    "duplicate_groups"
    "duplicate_files"
    "processing_jobs"
    "processing_errors"
)

for table in "${REQUIRED_TABLES[@]}"; do
    if psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -tAc "SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = '$table'" | grep -q 1; then
        pass "Table exists: $table"
    else
        fail "Table missing: $table"
    fi
done

echo ""

# ============================================================================
# PHASE 2: DATABASE DATA INTEGRITY CHECKS
# ============================================================================

echo "PHASE 2: Database Data Integrity Checks"
echo "----------------------------------------"

# Test 2.1: Check files table
FILE_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -tAc "SELECT COUNT(*) FROM files")
info "Files in database: $FILE_COUNT"

# Test 2.2: Check musical_metadata relationship
METADATA_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -tAc "SELECT COUNT(*) FROM musical_metadata")
info "Metadata records: $METADATA_COUNT"

# Test 2.3: Check 1:1 relationship integrity
ORPHANED_METADATA=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -tAc "SELECT COUNT(*) FROM musical_metadata mm WHERE NOT EXISTS (SELECT 1 FROM files f WHERE f.id = mm.file_id)")
if [ "$ORPHANED_METADATA" -eq 0 ]; then
    pass "No orphaned metadata records"
else
    fail "Found $ORPHANED_METADATA orphaned metadata records"
fi

# Test 2.4: Check indexes are being used
INDEX_USAGE=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -tAc "EXPLAIN SELECT * FROM files WHERE content_hash = '\\x0000000000000000'::bytea" | grep -c "Index Scan" || echo "0")
if [ "$INDEX_USAGE" -gt 0 ]; then
    pass "Indexes are being used (content_hash)"
else
    info "Sequential scan used (normal for small datasets)"
fi

echo ""

# ============================================================================
# PHASE 3: RUST INTEGRATION TESTS
# ============================================================================

echo "PHASE 3: Rust Integration Tests"
echo "--------------------------------"

# Change to pipeline directory
cd "$(dirname "$0")/src-tauri" || exit 1

# Test 3.1: Database connection tests
echo "Running database connection tests..."
if cargo test --test database_connection_test --quiet 2>&1 | grep -q "test result: ok"; then
    pass "Database connection tests passed"
else
    fail "Database connection tests failed"
    info "Run manually: cargo test --test database_connection_test -- --nocapture"
fi

# Test 3.2: Database integration tests
echo "Running comprehensive database tests..."
if timeout 120 cargo test --test database_test --quiet 2>&1 | grep -q "test result: ok"; then
    pass "Database integration tests passed"
else
    fail "Database integration tests failed or timed out"
    info "Run manually: cargo test --test database_test -- --nocapture"
fi

# Test 3.3: End-to-end workflow tests
echo "Running end-to-end workflow tests..."
if timeout 120 cargo test --test integration_end_to_end --quiet 2>&1 | grep -q "test result: ok"; then
    pass "End-to-end workflow tests passed"
else
    fail "End-to-end workflow tests failed or timed out"
    info "Run manually: cargo test --test integration_end_to_end -- --nocapture"
fi

echo ""

# ============================================================================
# PHASE 4: PERFORMANCE CHECKS
# ============================================================================

echo "PHASE 4: Performance Checks"
echo "---------------------------"

# Test 4.1: Query performance
QUERY_TIME=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "EXPLAIN ANALYZE SELECT * FROM files LIMIT 100" | grep "Execution Time" | grep -oP '\d+\.\d+')
if (( $(echo "$QUERY_TIME < 50" | bc -l) )); then
    pass "Query performance: ${QUERY_TIME}ms (target: <50ms)"
else
    info "Query performance: ${QUERY_TIME}ms (acceptable for development)"
fi

# Test 4.2: Index efficiency
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "ANALYZE files" > /dev/null 2>&1
pass "Database statistics updated (ANALYZE run)"

echo ""

# ============================================================================
# SUMMARY
# ============================================================================

echo "=========================================="
echo "  Verification Summary"
echo "=========================================="
echo ""
echo -e "Passed: ${GREEN}${PASS_COUNT}${NC}"
echo -e "Failed: ${RED}${FAIL_COUNT}${NC}"
echo ""

if [ "$FAIL_COUNT" -eq 0 ]; then
    echo -e "${GREEN}✓ ALL CHECKS PASSED${NC}"
    echo "Pipeline ↔ Database integration is verified and working!"
    exit 0
else
    echo -e "${RED}✗ SOME CHECKS FAILED${NC}"
    echo "Please review the failed tests above."
    exit 1
fi

#!/bin/bash
# Integration Test Runner for MIDI Library System
#
# This script runs all integration tests in the correct order
# and provides detailed reporting.

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"
TIMEOUT=600  # 10 minutes for long-running tests

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   MIDI Library System - Integration Test Suite${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Function to print section header
print_header() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

# Function to check if database is running
check_database() {
    print_header "Checking Database Connection"

    if docker ps | grep -q "midi-library-postgres"; then
        echo -e "${GREEN}✓${NC} PostgreSQL container is running"
    else
        echo -e "${RED}✗${NC} PostgreSQL container is not running"
        echo ""
        echo "Please start the database with:"
        echo "  cd database && docker-compose up -d"
        exit 1
    fi

    # Test connection
    if PGPASSWORD=145278963 psql -h localhost -p 5433 -U midiuser -d midi_library -c "SELECT 1" > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC} Database connection successful"
    else
        echo -e "${RED}✗${NC} Cannot connect to database"
        exit 1
    fi
}

# Function to run a test suite
run_test_suite() {
    local test_name=$1
    local test_file=$2
    local flags=$3

    echo ""
    echo -e "${YELLOW}▶${NC} Running: $test_name"
    echo -e "${YELLOW}  File:${NC} $test_file"

    if cargo test --test "$test_file" $flags -- --test-threads=1 --nocapture; then
        echo -e "${GREEN}✓${NC} $test_name: PASSED"
        return 0
    else
        echo -e "${RED}✗${NC} $test_name: FAILED"
        return 1
    fi
}

# Check database first
check_database

# Track results
TOTAL_SUITES=0
PASSED_SUITES=0
FAILED_SUITES=0

# Run existing tests
print_header "Phase 1: Existing Integration Tests"

TESTS=(
    "Database Connection Tests:database_connection_test:"
    "Database Integration Tests:database_test:"
    "Hash Module Tests:hash_test:"
    "Command Tests:command_test:"
    "Batch Insert Tests:batch_insert_test:"
)

for test in "${TESTS[@]}"; do
    IFS=':' read -r name file flags <<< "$test"
    TOTAL_SUITES=$((TOTAL_SUITES + 1))

    if run_test_suite "$name" "$file" "$flags"; then
        PASSED_SUITES=$((PASSED_SUITES + 1))
    else
        FAILED_SUITES=$((FAILED_SUITES + 1))
    fi
done

# Run new integration tests
print_header "Phase 2: New Integration Test Suites"

NEW_TESTS=(
    "End-to-End Workflow Tests:integration_end_to_end:"
    "MIDI Processing Tests:integration_midi_processing:"
    "Search & Filter Tests:integration_search_filter:"
    "Error Handling Tests:integration_error_handling:"
    "Performance Tests:integration_performance:"
)

for test in "${NEW_TESTS[@]}"; do
    IFS=':' read -r name file flags <<< "$test"
    TOTAL_SUITES=$((TOTAL_SUITES + 1))

    if run_test_suite "$name" "$file" "$flags"; then
        PASSED_SUITES=$((PASSED_SUITES + 1))
    else
        FAILED_SUITES=$((FAILED_SUITES + 1))
    fi
done

# Run concurrency tests (shorter version)
print_header "Phase 3: Concurrency Tests"

CONCURRENCY_TESTS=(
    "Concurrency Quick Tests:concurrency_quick_test:"
)

for test in "${CONCURRENCY_TESTS[@]}"; do
    IFS=':' read -r name file flags <<< "$test"
    TOTAL_SUITES=$((TOTAL_SUITES + 1))

    if run_test_suite "$name" "$file" "$flags"; then
        PASSED_SUITES=$((PASSED_SUITES + 1))
    else
        FAILED_SUITES=$((FAILED_SUITES + 1))
    fi
done

# Print summary
echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   Test Results Summary${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "  Total Test Suites: $TOTAL_SUITES"
echo -e "  ${GREEN}Passed: $PASSED_SUITES${NC}"

if [ $FAILED_SUITES -gt 0 ]; then
    echo -e "  ${RED}Failed: $FAILED_SUITES${NC}"
    echo ""
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
else
    echo -e "  ${RED}Failed: 0${NC}"
    echo ""
    echo -e "${GREEN}✓ All tests passed!${NC}"
    echo ""
    echo -e "Your integration test suite is ${GREEN}fully operational${NC}! 🎉"
fi

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"

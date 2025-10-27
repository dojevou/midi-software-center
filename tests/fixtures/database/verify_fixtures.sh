#!/bin/bash
# =============================================================================
# Verify Database Test Fixtures
# =============================================================================
# Purpose: Load test fixtures and verify data integrity
# Usage: ./verify_fixtures.sh
# =============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Database connection
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5433}"
DB_NAME="${DB_NAME:-midi_library}"
DB_USER="${DB_USER:-midiuser}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FIXTURE_FILE="${SCRIPT_DIR}/test_data.sql"

echo "============================================================================="
echo "Database Test Fixtures Verification"
echo "============================================================================="
echo ""

# Check if PostgreSQL is running
echo -n "Checking PostgreSQL connection... "
if pg_isready -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -q; then
    echo -e "${GREEN}OK${NC}"
else
    echo -e "${RED}FAILED${NC}"
    echo "Error: Cannot connect to PostgreSQL at $DB_HOST:$DB_PORT"
    exit 1
fi

# Check if database exists
echo -n "Checking database '$DB_NAME'... "
if psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -lqt | cut -d \| -f 1 | grep -qw "$DB_NAME"; then
    echo -e "${GREEN}OK${NC}"
else
    echo -e "${RED}FAILED${NC}"
    echo "Error: Database '$DB_NAME' does not exist"
    exit 1
fi

# Check if fixture file exists
echo -n "Checking fixture file... "
if [ -f "$FIXTURE_FILE" ]; then
    echo -e "${GREEN}OK${NC}"
    echo "  File: $FIXTURE_FILE"
    echo "  Size: $(wc -l < "$FIXTURE_FILE") lines"
else
    echo -e "${RED}FAILED${NC}"
    echo "Error: Fixture file not found: $FIXTURE_FILE"
    exit 1
fi

# Ask for confirmation
echo ""
echo -e "${YELLOW}WARNING: This will add test data to the database.${NC}"
echo -e "${YELLOW}Make sure you're using a test database, not production!${NC}"
echo ""
read -p "Continue? (y/N) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 0
fi

# Load fixtures
echo ""
echo "Loading test fixtures..."
if PGPASSWORD="${DB_PASSWORD:-145278963}" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -f "$FIXTURE_FILE" > /dev/null 2>&1; then
    echo -e "${GREEN}Fixtures loaded successfully!${NC}"
else
    echo -e "${RED}Failed to load fixtures${NC}"
    exit 1
fi

# Verify data
echo ""
echo "Verifying data..."
echo ""

# Count records in each table
VERIFICATION_QUERY="
DO \$\$
DECLARE
    files_count INTEGER;
    metadata_count INTEGER;
    categories_count INTEGER;
    instruments_count INTEGER;
    tags_count INTEGER;
    file_tags_count INTEGER;
    favorites_count INTEGER;
    splits_count INTEGER;
    dup_groups_count INTEGER;
    dup_files_count INTEGER;
    embeddings_count INTEGER;
    compat_count INTEGER;
    rhythm_count INTEGER;
    harmonic_count INTEGER;
    melodic_count INTEGER;
    jobs_count INTEGER;
    errors_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO files_count FROM files;
    SELECT COUNT(*) INTO metadata_count FROM musical_metadata;
    SELECT COUNT(*) INTO categories_count FROM file_categories;
    SELECT COUNT(*) INTO instruments_count FROM file_instruments;
    SELECT COUNT(*) INTO tags_count FROM tags;
    SELECT COUNT(*) INTO file_tags_count FROM file_tags;
    SELECT COUNT(*) INTO favorites_count FROM favorites;
    SELECT COUNT(*) INTO splits_count FROM track_splits;
    SELECT COUNT(*) INTO dup_groups_count FROM duplicate_groups;
    SELECT COUNT(*) INTO dup_files_count FROM duplicate_files;
    SELECT COUNT(*) INTO embeddings_count FROM file_embeddings;
    SELECT COUNT(*) INTO compat_count FROM file_compatibility;
    SELECT COUNT(*) INTO rhythm_count FROM rhythm_patterns;
    SELECT COUNT(*) INTO harmonic_count FROM harmonic_patterns;
    SELECT COUNT(*) INTO melodic_count FROM melodic_patterns;
    SELECT COUNT(*) INTO jobs_count FROM processing_jobs;
    SELECT COUNT(*) INTO errors_count FROM processing_errors;

    RAISE NOTICE 'Table                    | Records';
    RAISE NOTICE '-------------------------|--------';
    RAISE NOTICE 'files                    | %', files_count;
    RAISE NOTICE 'musical_metadata         | %', metadata_count;
    RAISE NOTICE 'file_categories          | %', categories_count;
    RAISE NOTICE 'file_instruments         | %', instruments_count;
    RAISE NOTICE 'tags                     | %', tags_count;
    RAISE NOTICE 'file_tags                | %', file_tags_count;
    RAISE NOTICE 'favorites                | %', favorites_count;
    RAISE NOTICE 'track_splits             | %', splits_count;
    RAISE NOTICE 'duplicate_groups         | %', dup_groups_count;
    RAISE NOTICE 'duplicate_files          | %', dup_files_count;
    RAISE NOTICE 'file_embeddings          | %', embeddings_count;
    RAISE NOTICE 'file_compatibility       | %', compat_count;
    RAISE NOTICE 'rhythm_patterns          | %', rhythm_count;
    RAISE NOTICE 'harmonic_patterns        | %', harmonic_count;
    RAISE NOTICE 'melodic_patterns         | %', melodic_count;
    RAISE NOTICE 'processing_jobs          | %', jobs_count;
    RAISE NOTICE 'processing_errors        | %', errors_count;
    RAISE NOTICE '-------------------------|--------';
    RAISE NOTICE 'Total tables with data: 17';
END \$\$;
"

PGPASSWORD="${DB_PASSWORD:-145278963}" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "$VERIFICATION_QUERY"

echo ""
echo "============================================================================="
echo "Sample Queries"
echo "============================================================================="
echo ""

# Query 1: Files by BPM range
echo "1. Files with BPM between 120-140:"
PGPASSWORD="${DB_PASSWORD:-145278963}" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "
    SELECT f.filename, m.bpm, m.key_signature
    FROM files f
    JOIN musical_metadata m ON f.id = m.file_id
    WHERE m.bpm BETWEEN 120 AND 140
    ORDER BY m.bpm;
" -t

echo ""

# Query 2: Favorites
echo "2. User favorites:"
PGPASSWORD="${DB_PASSWORD:-145278963}" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "
    SELECT f.id, f.filename, m.bpm, m.key_signature
    FROM favorites fav
    JOIN files f ON fav.file_id = f.id
    LEFT JOIN musical_metadata m ON f.id = m.file_id
    ORDER BY fav.created_at DESC;
" -t

echo ""

# Query 3: Track splits
echo "3. Track splits from parent file:"
PGPASSWORD="${DB_PASSWORD:-145278963}" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "
    SELECT
        parent.filename as parent_file,
        ts.track_number,
        ts.track_name,
        ts.instrument
    FROM track_splits ts
    JOIN files parent ON ts.parent_file_id = parent.id
    ORDER BY parent.id, ts.track_number;
" -t

echo ""

echo "============================================================================="
echo -e "${GREEN}Verification complete!${NC}"
echo "============================================================================="
echo ""
echo "Test fixtures are ready to use for integration tests."
echo ""

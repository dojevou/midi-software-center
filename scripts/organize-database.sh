#!/bin/bash
# =============================================================================
# MIDI Library - Database Organization Script
# =============================================================================
# Purpose: Apply instrument-based organization to database
# Source: 97 instruments extracted from 1.7M files
# Date: 2025-11-22
# =============================================================================

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Database connection
DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  MIDI Library - Instrument-Based Database Organization${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# =============================================================================
# Step 1: Verify database connection
# =============================================================================
echo -e "${YELLOW}[1/5] Verifying database connection...${NC}"
if psql "$DB_URL" -c "SELECT 1" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Database connected${NC}"
else
    echo -e "${RED}✗ Cannot connect to database${NC}"
    echo "Please ensure PostgreSQL is running:"
    echo "  make docker-up"
    exit 1
fi

# =============================================================================
# Step 2: Check current state
# =============================================================================
echo -e "${YELLOW}[2/5] Checking current database state...${NC}"
FILE_COUNT=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM files;" | xargs)
TAG_COUNT=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM tags;" | xargs)
FILE_TAG_COUNT=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM file_tags;" | xargs)

echo "  Files in database: $FILE_COUNT"
echo "  Existing tags: $TAG_COUNT"
echo "  Existing file-tag relationships: $FILE_TAG_COUNT"
echo ""

if [ "$FILE_COUNT" -eq 0 ]; then
    echo -e "${RED}✗ No files in database. Please import files first:${NC}"
    echo "  ./target/release/import --source ~/projects/midi-software-center/midi-library/"
    exit 1
fi

# =============================================================================
# Step 3: Apply organization schema
# =============================================================================
echo -e "${YELLOW}[3/5] Applying OPTIMIZED instrument organization schema...${NC}"
echo -e "${BLUE}Note: filepath matching automatically includes folder keywords!${NC}"
if [ ! -f "database/organize_by_instruments_optimized.sql" ]; then
    echo -e "${RED}✗ Schema file not found: database/organize_by_instruments_optimized.sql${NC}"
    exit 1
fi

echo "  This will:"
echo "    • Insert 97 instrument tags"
echo "    • Create tagging functions"
echo "    • Tag files by keyword matching (filename + filepath)"
echo "    • Filepath matching catches folder keywords (e.g., /Drums/Rock/track01.mid)"
echo "    • Create virtual folder views"
echo "    • Add helper functions and indexes"
echo ""
echo -e "${YELLOW}  Estimated time: 2-5 minutes for ${FILE_COUNT} files (OPTIMIZED)${NC}"
echo ""
read -p "Continue? (y/n) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Cancelled."
    exit 0
fi

echo "  Applying optimized schema (single scan, parallel processing)..."
psql "$DB_URL" -f database/organize_by_instruments_optimized.sql > /tmp/organize_output.log 2>&1

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Schema applied successfully${NC}"
else
    echo -e "${RED}✗ Error applying schema. See /tmp/organize_output.log${NC}"
    tail -20 /tmp/organize_output.log
    exit 1
fi

# =============================================================================
# Step 4: Verify results
# =============================================================================
echo ""
echo -e "${YELLOW}[4/5] Verifying organization results...${NC}"

NEW_TAG_COUNT=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM tags;" | xargs)
NEW_FILE_TAG_COUNT=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM file_tags;" | xargs)

echo "  Tags created: $NEW_TAG_COUNT (expected: 97+)"
echo "  File-tag relationships: $NEW_FILE_TAG_COUNT"
echo ""

# Top 10 instruments
echo "  Top 10 instruments by file count:"
psql "$DB_URL" -c "
    SELECT
        name,
        category,
        file_count,
        percentage || '%' AS pct
    FROM v_tag_stats
    WHERE category IS NOT NULL
    ORDER BY file_count DESC
    LIMIT 10;
"

echo ""

# Summary by category
echo "  Summary by instrument category:"
psql "$DB_URL" -c "
    SELECT
        category,
        COUNT(*) AS instrument_count,
        SUM(file_count) AS total_files,
        ROUND(SUM(file_count)::NUMERIC / $FILE_COUNT * 100, 2) || '%' AS percentage
    FROM v_tag_stats
    WHERE category IS NOT NULL
    GROUP BY category
    ORDER BY total_files DESC;
"

# =============================================================================
# Step 5: Generate usage examples
# =============================================================================
echo ""
echo -e "${YELLOW}[5/5] Organization complete!${NC}"
echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  ✓ Database organized by 97 instruments${NC}"
echo -e "${GREEN}  ✓ Virtual folders created${NC}"
echo -e "${GREEN}  ✓ Search functions enabled${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo "Example queries:"
echo ""
echo "  # View all drum files"
echo "  psql \"$DB_URL\" -c \"SELECT * FROM v_drums LIMIT 20;\""
echo ""
echo "  # Find 120 BPM drum loops in C major"
echo "  psql \"$DB_URL\" -c \""
echo "    SELECT f.filename, m.bpm, m.key_signature"
echo "    FROM files f"
echo "    JOIN file_tags ft ON f.id = ft.file_id"
echo "    JOIN tags t ON ft.tag_id = t.id"
echo "    JOIN musical_metadata m ON f.id = m.file_id"
echo "    WHERE t.name = 'loop'"
echo "      AND t.category = 'drums'"
echo "      AND m.bpm BETWEEN 118 AND 122"
echo "      AND m.key_signature = 'C';"
echo "  \""
echo ""
echo "  # Find jazz piano files"
echo "  psql \"$DB_URL\" -c \"SELECT * FROM get_files_by_instruments(ARRAY['jazz', 'piano']);\""
echo ""
echo "  # Export drum files to text"
echo "  psql \"$DB_URL\" -c \"\\\\COPY (SELECT filepath FROM v_drums) TO '/tmp/drum_files.txt';\""
echo ""
echo "Full guide: DATABASE-INSTRUMENT-ORGANIZATION-GUIDE.md"
echo ""

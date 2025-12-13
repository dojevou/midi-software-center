#!/bin/bash
#
# MIDI Software Center - Database Sync Verification
#
# Verifies that the database schema matches the code expectations
# and checks for data integrity issues.
#
# Usage:
#   ./scripts/verify/verify-database-sync.sh [OPTIONS]
#
# Options:
#   --quick      Quick schema check only
#   --full       Full verification with data integrity
#   --fix        Attempt to fix simple issues
#   --report     Generate detailed report
#   --help       Show this help message

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Options
QUICK_MODE=false
FULL_MODE=false
FIX_MODE=false
REPORT_MODE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --quick)
            QUICK_MODE=true
            shift
            ;;
        --full)
            FULL_MODE=true
            shift
            ;;
        --fix)
            FIX_MODE=true
            shift
            ;;
        --report)
            REPORT_MODE=true
            shift
            ;;
        --help)
            head -20 "$0" | tail -15
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Default to quick mode if none specified
if [ "$QUICK_MODE" = false ] && [ "$FULL_MODE" = false ]; then
    QUICK_MODE=true
fi

# Project root
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$PROJECT_ROOT"

# Database connection
DB_URL="${DATABASE_URL:-postgresql://midiuser:145278963@localhost:5433/midi_library}"

echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     Database Sync Verification                               ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check database connection
echo -e "${CYAN}Checking database connection...${NC}"
if ! psql "$DB_URL" -c "SELECT 1" &>/dev/null; then
    echo -e "${RED}Cannot connect to database${NC}"
    echo "DATABASE_URL: $DB_URL"
    exit 1
fi
echo -e "${GREEN}Connected to database${NC}"
echo ""

# ============================================================================
# Expected Schema Definition
# ============================================================================

EXPECTED_TABLES=("files" "musical_metadata" "tags" "file_tags")

declare -A EXPECTED_COLUMNS
EXPECTED_COLUMNS[files]="id,filename,filepath,hash,size_bytes,created_at"
EXPECTED_COLUMNS[musical_metadata]="id,file_id,bpm,key_signature,time_signature_numerator,time_signature_denominator,duration_seconds"
EXPECTED_COLUMNS[tags]="id,name"
EXPECTED_COLUMNS[file_tags]="file_id,tag_id"

declare -A EXPECTED_INDEXES
EXPECTED_INDEXES[files]="idx_files_filepath,idx_files_filename,idx_files_hash"
EXPECTED_INDEXES[musical_metadata]="idx_musical_metadata_file_id,idx_musical_metadata_bpm"
EXPECTED_INDEXES[tags]="idx_tags_name"
EXPECTED_INDEXES[file_tags]="idx_file_tags_file_id,idx_file_tags_tag_id"

# ============================================================================
# 1. Table Existence Check
# ============================================================================

echo -e "${BLUE}┌─────────────────────────────────────────────────────────────┐${NC}"
echo -e "${BLUE}│ 1. Table Existence                                          │${NC}"
echo -e "${BLUE}└─────────────────────────────────────────────────────────────┘${NC}"

MISSING_TABLES=()
for table in "${EXPECTED_TABLES[@]}"; do
    EXISTS=$(psql "$DB_URL" -t -c "
        SELECT COUNT(*) FROM information_schema.tables
        WHERE table_schema = 'public' AND table_name = '$table'
    " | tr -d ' ')

    if [ "$EXISTS" -eq 1 ]; then
        echo -e "  ${GREEN}✓${NC} Table: $table"
    else
        echo -e "  ${RED}✗${NC} Table: $table (MISSING)"
        MISSING_TABLES+=("$table")
    fi
done
echo ""

# ============================================================================
# 2. Column Verification
# ============================================================================

echo -e "${BLUE}┌─────────────────────────────────────────────────────────────┐${NC}"
echo -e "${BLUE}│ 2. Column Verification                                      │${NC}"
echo -e "${BLUE}└─────────────────────────────────────────────────────────────┘${NC}"

MISSING_COLUMNS=()
for table in "${EXPECTED_TABLES[@]}"; do
    # Skip if table doesn't exist
    if [[ " ${MISSING_TABLES[*]} " =~ " ${table} " ]]; then
        continue
    fi

    echo -e "  ${CYAN}$table:${NC}"

    # Get actual columns
    ACTUAL_COLUMNS=$(psql "$DB_URL" -t -c "
        SELECT string_agg(column_name, ',' ORDER BY ordinal_position)
        FROM information_schema.columns
        WHERE table_schema = 'public' AND table_name = '$table'
    " | tr -d ' ')

    IFS=',' read -ra EXPECTED <<< "${EXPECTED_COLUMNS[$table]}"
    for col in "${EXPECTED[@]}"; do
        if [[ ",$ACTUAL_COLUMNS," == *",$col,"* ]]; then
            echo -e "    ${GREEN}✓${NC} $col"
        else
            echo -e "    ${RED}✗${NC} $col (MISSING)"
            MISSING_COLUMNS+=("$table.$col")
        fi
    done
done
echo ""

# ============================================================================
# 3. Index Verification
# ============================================================================

echo -e "${BLUE}┌─────────────────────────────────────────────────────────────┐${NC}"
echo -e "${BLUE}│ 3. Index Verification                                       │${NC}"
echo -e "${BLUE}└─────────────────────────────────────────────────────────────┘${NC}"

# Get all indexes
ALL_INDEXES=$(psql "$DB_URL" -t -c "
    SELECT indexname FROM pg_indexes WHERE schemaname = 'public'
" | tr -d ' ' | tr '\n' ',')

MISSING_INDEXES=()
for table in "${EXPECTED_TABLES[@]}"; do
    if [[ -z "${EXPECTED_INDEXES[$table]}" ]]; then
        continue
    fi

    echo -e "  ${CYAN}$table:${NC}"

    IFS=',' read -ra INDEXES <<< "${EXPECTED_INDEXES[$table]}"
    for idx in "${INDEXES[@]}"; do
        if [[ ",$ALL_INDEXES," == *",$idx,"* ]]; then
            echo -e "    ${GREEN}✓${NC} $idx"
        else
            echo -e "    ${YELLOW}⚠${NC} $idx (MISSING)"
            MISSING_INDEXES+=("$idx")
        fi
    done
done
echo ""

# ============================================================================
# 4. Data Integrity (Full Mode)
# ============================================================================

if [ "$FULL_MODE" = true ]; then
    echo -e "${BLUE}┌─────────────────────────────────────────────────────────────┐${NC}"
    echo -e "${BLUE}│ 4. Data Integrity                                           │${NC}"
    echo -e "${BLUE}└─────────────────────────────────────────────────────────────┘${NC}"

    # Check for orphan file_tags
    ORPHAN_FILE_TAGS=$(psql "$DB_URL" -t -c "
        SELECT COUNT(*) FROM file_tags ft
        WHERE NOT EXISTS (SELECT 1 FROM files f WHERE f.id = ft.file_id)
    " | tr -d ' ')

    if [ "$ORPHAN_FILE_TAGS" -eq 0 ]; then
        echo -e "  ${GREEN}✓${NC} No orphan file_tags (files)"
    else
        echo -e "  ${RED}✗${NC} Orphan file_tags (files): $ORPHAN_FILE_TAGS"

        if [ "$FIX_MODE" = true ]; then
            echo -e "    ${YELLOW}Fixing...${NC}"
            psql "$DB_URL" -c "
                DELETE FROM file_tags ft
                WHERE NOT EXISTS (SELECT 1 FROM files f WHERE f.id = ft.file_id)
            " &>/dev/null
            echo -e "    ${GREEN}Fixed${NC}"
        fi
    fi

    # Check for orphan file_tags (tags)
    ORPHAN_TAG_REFS=$(psql "$DB_URL" -t -c "
        SELECT COUNT(*) FROM file_tags ft
        WHERE NOT EXISTS (SELECT 1 FROM tags t WHERE t.id = ft.tag_id)
    " | tr -d ' ')

    if [ "$ORPHAN_TAG_REFS" -eq 0 ]; then
        echo -e "  ${GREEN}✓${NC} No orphan file_tags (tags)"
    else
        echo -e "  ${RED}✗${NC} Orphan file_tags (tags): $ORPHAN_TAG_REFS"

        if [ "$FIX_MODE" = true ]; then
            echo -e "    ${YELLOW}Fixing...${NC}"
            psql "$DB_URL" -c "
                DELETE FROM file_tags ft
                WHERE NOT EXISTS (SELECT 1 FROM tags t WHERE t.id = ft.tag_id)
            " &>/dev/null
            echo -e "    ${GREEN}Fixed${NC}"
        fi
    fi

    # Check for orphan musical_metadata
    ORPHAN_METADATA=$(psql "$DB_URL" -t -c "
        SELECT COUNT(*) FROM musical_metadata m
        WHERE NOT EXISTS (SELECT 1 FROM files f WHERE f.id = m.file_id)
    " | tr -d ' ')

    if [ "$ORPHAN_METADATA" -eq 0 ]; then
        echo -e "  ${GREEN}✓${NC} No orphan musical_metadata"
    else
        echo -e "  ${RED}✗${NC} Orphan musical_metadata: $ORPHAN_METADATA"

        if [ "$FIX_MODE" = true ]; then
            echo -e "    ${YELLOW}Fixing...${NC}"
            psql "$DB_URL" -c "
                DELETE FROM musical_metadata m
                WHERE NOT EXISTS (SELECT 1 FROM files f WHERE f.id = m.file_id)
            " &>/dev/null
            echo -e "    ${GREEN}Fixed${NC}"
        fi
    fi

    # Check for duplicate hashes
    DUPLICATE_HASHES=$(psql "$DB_URL" -t -c "
        SELECT COUNT(*) FROM (
            SELECT hash FROM files WHERE hash IS NOT NULL
            GROUP BY hash HAVING COUNT(*) > 1
        ) dups
    " | tr -d ' ')

    if [ "$DUPLICATE_HASHES" -eq 0 ]; then
        echo -e "  ${GREEN}✓${NC} No duplicate hashes"
    else
        echo -e "  ${YELLOW}⚠${NC} Duplicate hashes: $DUPLICATE_HASHES"
    fi

    # Check for invalid BPM
    INVALID_BPM=$(psql "$DB_URL" -t -c "
        SELECT COUNT(*) FROM musical_metadata
        WHERE bpm IS NOT NULL AND (bpm < 20 OR bpm > 400)
    " | tr -d ' ')

    if [ "$INVALID_BPM" -eq 0 ]; then
        echo -e "  ${GREEN}✓${NC} No invalid BPM values"
    else
        echo -e "  ${YELLOW}⚠${NC} Invalid BPM values: $INVALID_BPM"
    fi

    echo ""
fi

# ============================================================================
# 5. Row Count Summary
# ============================================================================

echo -e "${BLUE}┌─────────────────────────────────────────────────────────────┐${NC}"
echo -e "${BLUE}│ 5. Row Count Summary                                        │${NC}"
echo -e "${BLUE}└─────────────────────────────────────────────────────────────┘${NC}"

for table in "${EXPECTED_TABLES[@]}"; do
    if [[ " ${MISSING_TABLES[*]} " =~ " ${table} " ]]; then
        continue
    fi

    COUNT=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM $table" | tr -d ' ')
    printf "  %-20s %'12d rows\n" "$table:" "$COUNT"
done
echo ""

# ============================================================================
# Generate Report (if requested)
# ============================================================================

if [ "$REPORT_MODE" = true ]; then
    REPORT_FILE="/tmp/database-sync-report-$(date +%Y%m%d-%H%M%S).txt"

    {
        echo "MIDI Software Center - Database Sync Report"
        echo "Generated: $(date)"
        echo "=============================================="
        echo ""
        echo "Missing Tables: ${#MISSING_TABLES[@]}"
        for t in "${MISSING_TABLES[@]}"; do
            echo "  - $t"
        done
        echo ""
        echo "Missing Columns: ${#MISSING_COLUMNS[@]}"
        for c in "${MISSING_COLUMNS[@]}"; do
            echo "  - $c"
        done
        echo ""
        echo "Missing Indexes: ${#MISSING_INDEXES[@]}"
        for i in "${MISSING_INDEXES[@]}"; do
            echo "  - $i"
        done
        echo ""
        echo "Row Counts:"
        for table in "${EXPECTED_TABLES[@]}"; do
            if [[ ! " ${MISSING_TABLES[*]} " =~ " ${table} " ]]; then
                COUNT=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM $table" | tr -d ' ')
                printf "  %-20s %'12d\n" "$table:" "$COUNT"
            fi
        done
    } > "$REPORT_FILE"

    echo -e "${GREEN}Report saved to: $REPORT_FILE${NC}"
    echo ""
fi

# ============================================================================
# Summary
# ============================================================================

TOTAL_ISSUES=$((${#MISSING_TABLES[@]} + ${#MISSING_COLUMNS[@]} + ${#MISSING_INDEXES[@]}))

echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                       SUMMARY                                ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "  Missing Tables:  ${#MISSING_TABLES[@]}"
echo -e "  Missing Columns: ${#MISSING_COLUMNS[@]}"
echo -e "  Missing Indexes: ${#MISSING_INDEXES[@]}"
echo ""

if [ "$TOTAL_ISSUES" -eq 0 ]; then
    echo -e "${GREEN}Database schema is in sync!${NC}"
    exit 0
else
    echo -e "${YELLOW}Found $TOTAL_ISSUES schema issue(s)${NC}"
    echo ""
    echo "To fix missing tables/columns, run migrations:"
    echo "  psql \$DATABASE_URL -f database/migrations/001_init.sql"
    echo ""
    echo "To create missing indexes:"
    echo "  psql \$DATABASE_URL -f database/optimizations/add_tagging_indexes.sql"
    exit 1
fi
